use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{Datelike, Duration as Days, Local, Timelike};
use eframe::egui::{self, Color32, RichText};

use crate::audio::{Player, Voices, scan_audio_dir};
use crate::editor::EditorState;
use crate::i18n::{plural, tr, trf};
use crate::model::{Course, LANGS, Level, Stage, lang_color, lang_name};
use crate::session::{ReviewState, Seq, Session};
use crate::store::{self, Saved, day_key, today};
use crate::theme::{
    self, ACCENT, ACCENT_DARK, ACCENT_SOFT, BAD, GOOD, GOOD_SOFT, INK, LINE, MUTED, NAV, NAV_MUTED, NAV_TEXT, WARN, WARN_SOFT, WAVE, bold,
    caps, medium, muted,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Screen {
    Home,
    Courses,
    Study,
    Review,
    Editor,
    Settings,
    Appendix,
}

struct Shot {
    path: PathBuf,
    frames: u32,
    requested: bool,
}

pub struct App {
    pub courses: Vec<Course>,
    pub problems: Vec<String>,
    pub saved: Saved,
    pub dirty: bool,
    last_save: Instant,
    pub screen: Screen,
    pub player: Player,
    pub voices: Voices,
    pub seq: Option<Seq>,
    /// Recordings found in each course's audio folder, by lesson number.
    pub audio_maps: HashMap<String, BTreeMap<u32, PathBuf>>,
    pub session: Option<Session>,
    pub review: ReviewState,
    pub editor: EditorState,
    pub toast: Option<(String, Instant)>,
    shot: Option<Shot>,
    tick: Instant,
    new_course_lang: String,
    /// The pronunciation guide window is open.
    pub key_open: bool,
    /// The back of each built-in course: grammar, indexes, words.
    pub appendices: Vec<crate::appendix::Appendix>,
    pub appendix_ui: crate::appendix::AppendixState,
}

/// The widest a screen's text column grows; wider windows centre it.
const PAGE: f32 = 1080.0;

impl App {
    pub fn new(ctx: &egui::Context, screenshot: Option<PathBuf>, open: Option<String>) -> App {
        let (mut courses, problems) = store::load_courses();
        // A language is offered once its Foundation is complete; the others are
        // drafts, shown only with LINGUA_ONDE_DRAFTS=1 (for writing them).
        if std::env::var_os("LINGUA_ONDE_DRAFTS").is_none() {
            let ready: Vec<String> = courses
                .iter()
                .filter(|c| c.builtin && c.stage == Some(Stage::Foundation) && c.lessons.len() >= Stage::Foundation.size() as usize)
                .map(|c| c.lang.clone())
                .collect();
            courses.retain(|c| !c.builtin || ready.contains(&c.lang));
        }
        // An unreadable progress file is replaced by the default, whose language its notice uses.
        crate::i18n::set_base(&Saved::default().base);
        let saved = Saved::load();
        crate::i18n::set_base(&saved.base);
        let voices = Voices::new(store::data_dir().join("voices"), ctx.clone());
        let mut app = App {
            courses,
            problems,
            saved,
            dirty: false,
            last_save: Instant::now(),
            screen: Screen::Home,
            player: Player::default(),
            voices,
            seq: None,
            audio_maps: HashMap::new(),
            session: None,
            review: ReviewState::default(),
            editor: EditorState::default(),
            toast: None,
            shot: screenshot.map(|path| Shot {
                path,
                frames: 0,
                requested: false,
            }),
            tick: Instant::now(),
            new_course_lang: "es".into(),
            key_open: false,
            appendices: crate::appendix::builtin(),
            appendix_ui: Default::default(),
        };
        if let Some(n) = app.saved.notice.take() {
            app.say(n);
        }
        if app.course_index(&app.saved.active_course.clone()).is_none() {
            app.saved.active_course = app.courses.first().map(|c| c.id.clone()).unwrap_or_default();
        }
        match open.as_deref() {
            Some("courses") => app.screen = Screen::Courses,
            Some(s) if s.starts_with("review") => {
                // "review" or "review:<tab>": daily sentences, active wave, search.
                app.screen = Screen::Review;
                app.review.tab = s.split(':').nth(1).and_then(|n| n.parse().ok()).unwrap_or(0).min(2);
                app.review.search = s.split(':').nth(2).unwrap_or_default().to_string();
            }
            Some("guide") => app.key_open = true,
            Some(s) if s.starts_with("editor") => {
                app.open_editor(None);
                if let Some(li) = s.split(':').nth(1).and_then(|n| n.parse().ok()) {
                    app.edit_lesson(li);
                }
            }
            Some("settings") => app.screen = Screen::Settings,
            Some(s) if s.starts_with("appendix") => {
                // "appendix" or "appendix:<page>": introduction, pronunciation, lessons, grammar, index, words, expressions.
                app.screen = Screen::Appendix;
                app.appendix_ui.tab = s.split(':').nth(1).and_then(|n| n.parse().ok()).unwrap_or(0).min(6);
            }
            Some(s) if s.starts_with("study") => {
                // "study" or "study:<step>" — used to check a screen quickly.
                let ci = app.active();
                let li = app.next_lesson(ci).unwrap_or(0);
                app.start_session(ci, li);
                if let (Some(step), Some(sess)) = (s.split(':').nth(1).and_then(|n| n.parse::<usize>().ok()), app.session.as_mut()) {
                    sess.step = step.min(sess.steps.len() - 1);
                }
            }
            _ => {}
        }
        app
    }

    // ------------------------------------------------------------ helpers

    pub fn base(&self) -> String {
        self.saved.base.clone()
    }

    pub fn course_index(&self, id: &str) -> Option<usize> {
        self.courses.iter().position(|c| c.id == id)
    }

    pub fn active(&self) -> usize {
        self.course_index(&self.saved.active_course).unwrap_or(0)
    }

    /// Today's lesson: one the learner chose to go over again (from the day
    /// after it was studied), else the first not done yet.
    pub fn next_lesson(&self, ci: usize) -> Option<usize> {
        let c = self.courses.get(ci)?;
        let today = day_key(today());
        let again = c.lessons.iter().position(|l| {
            self.saved
                .progress(&c.lesson_key(l.number))
                .is_some_and(|p| p.again && p.last.as_deref() != Some(today.as_str()))
        });
        again.or_else(|| c.lessons.iter().position(|l| !self.saved.is_done(&c.lesson_key(l.number))))
    }

    /// Lessons whose active wave is due: lesson n comes back once lesson
    /// n + gap has been studied (classic: lesson 1 with lesson 50). Once the
    /// course is behind the learner, the lessons it never brought back come
    /// back one a day, oldest first, as the book's last 49 days do.
    pub fn wave2_due(&self, ci: usize) -> Vec<usize> {
        let c = &self.courses[ci];
        let gap = self.saved.wave2_gap.max(1);
        let done = |n: u32| c.lessons.iter().any(|l| l.number == n && self.saved.is_done(&c.lesson_key(n)));
        let open = self.wave2_open(ci);
        let mut due: Vec<usize> = open.iter().copied().filter(|&i| done(c.lessons[i].number + gap)).collect();
        let today = day_key(today());
        let rebuilt_today = c.lessons.iter().any(|l| {
            self.saved
                .progress(&c.lesson_key(l.number))
                .is_some_and(|p| p.wave2.as_deref() == Some(today.as_str()))
        });
        if self.wave2_tail(ci)
            && !rebuilt_today
            && let Some(&i) = open.iter().find(|i| !due.contains(i))
        {
            due.push(i);
            due.sort_unstable();
        }
        due
    }

    /// Studied lessons whose active wave has not been done yet.
    pub fn wave2_open(&self, ci: usize) -> Vec<usize> {
        let c = &self.courses[ci];
        (0..c.lessons.len())
            .filter(|&i| {
                let p = self.saved.progress(&c.lesson_key(c.lessons[i].number));
                p.is_some_and(|p| p.completed.is_some() && p.wave2.is_none())
            })
            .collect()
    }

    /// Whether the course is behind the learner, so its last lessons need no
    /// partner to come back: every lesson read (a built-in course still being
    /// written is not finished), or a later course of the language started.
    /// Someone who starts from a later course has studied nothing here, so
    /// nothing comes back from it.
    fn wave2_tail(&self, ci: usize) -> bool {
        let c = &self.courses[ci];
        let read = !c.lessons.is_empty() && c.lessons.iter().all(|l| self.saved.is_done(&c.lesson_key(l.number))) && !self.unwritten(ci);
        let moved_on = c.stage.filter(|_| c.builtin && self.saved.active_course != c.id).is_some_and(|st| {
            self.stages_of(&c.lang)
                .into_iter()
                .any(|si| self.courses[si].stage.is_some_and(|s| s as u8 > st as u8) && self.done_count(si) > 0)
        });
        read || moved_on
    }

    /// The active-wave lesson that goes with studying lesson `li`, as
    /// (course, lesson): its partner (lesson 1 with lesson 50), else one left
    /// behind in this course, else one still open in an earlier course of the
    /// same language (Foundation 51–100 come back with Perfectionnement 1–50).
    pub fn wave2_for(&self, ci: usize, li: usize) -> Option<(usize, usize)> {
        let c = &self.courses[ci];
        let partner = c.lessons[li]
            .number
            .checked_sub(self.saved.wave2_gap.max(1))
            .and_then(|n| c.lessons.iter().position(|l| l.number == n))
            .filter(|&w| {
                let k = c.lesson_key(c.lessons[w].number);
                self.saved.is_done(&k) && self.saved.progress(&k).is_some_and(|p| p.wave2.is_none())
            });
        partner
            .or_else(|| self.wave2_due(ci).into_iter().find(|&w| w != li))
            .map(|w| (ci, w))
            .or_else(|| self.wave2_pending(ci).into_iter().find(|&(wc, _)| wc != ci))
    }

    /// Every active-wave lesson due for this course's learner, as (course,
    /// lesson): the earlier courses of the same language first, oldest first.
    /// The second wave of a finished course goes on while the next one starts.
    pub fn wave2_pending(&self, ci: usize) -> Vec<(usize, usize)> {
        self.wave_courses(ci)
            .into_iter()
            .flat_map(|si| self.wave2_due(si).into_iter().map(move |w| (si, w)))
            .collect()
    }

    /// Lessons still to rebuild, today or on the days to come, in the waves
    /// that go on for this course's learner.
    pub fn wave2_left(&self, ci: usize) -> usize {
        self.wave_courses(ci)
            .into_iter()
            .filter(|&si| si == ci || self.wave2_tail(si))
            .map(|si| self.wave2_open(si).len())
            .sum()
    }

    /// The earlier built-in courses of this course's language, then itself.
    fn wave_courses(&self, ci: usize) -> Vec<usize> {
        let c = &self.courses[ci];
        let mut v: Vec<usize> = match c.stage.filter(|_| c.builtin) {
            Some(st) => self
                .stages_of(&c.lang)
                .into_iter()
                .filter(|&si| self.courses[si].stage.is_some_and(|s| (s as u8) < st as u8))
                .collect(),
            None => Vec::new(),
        };
        v.push(ci);
        v
    }

    /// A built-in course whose planned lessons are not all written yet.
    pub fn unwritten(&self, ci: usize) -> bool {
        let c = &self.courses[ci];
        c.builtin && c.stage.is_some_and(|st| c.lessons.len() < st.size() as usize)
    }

    /// The built-in courses of a language, in stage order, by index.
    pub fn stages_of(&self, lang: &str) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.courses.len())
            .filter(|&i| self.courses[i].builtin && self.courses[i].lang == lang)
            .collect();
        v.sort_by_key(|&i| self.courses[i].stage.map(|s| s as u8));
        v
    }

    /// "Spanish · Foundation", or the course's own name for the person's courses.
    pub fn course_label(&self, ci: usize) -> String {
        let base = self.base();
        let c = &self.courses[ci];
        match c.stage {
            Some(st) if c.builtin => format!("{} · {}", lang_name(&c.lang, &base), st.name(&base)),
            _ => c.name_in(&base).to_string(),
        }
    }

    /// The synthetic voice's speed: the learner's setting, slower in a built-in
    /// Foundation's first lessons (85% at lesson 1) and natural from lesson 50,
    /// as the books' recordings are. Real recordings keep their own pace.
    pub fn voice_speed(&self) -> f32 {
        match self.session.as_ref().filter(|_| self.screen == Screen::Study) {
            Some(s) => self.voice_speed_at(s.course, s.lesson),
            None => self.saved.speed,
        }
    }

    /// The voice's speed while studying lesson `li` of course `ci`.
    pub fn voice_speed_at(&self, ci: usize, li: usize) -> f32 {
        let c = &self.courses[ci];
        let lesson = c
            .lessons
            .get(li)
            .filter(|_| self.saved.gradual_pace && c.builtin && c.stage == Some(Stage::Foundation))
            .map(|l| l.number);
        match lesson {
            Some(n) => self.saved.speed * (0.85 + 0.15 * (n.clamp(1, 50) - 1) as f32 / 49.0),
            None => self.saved.speed,
        }
    }

    /// Opens the active wave of lesson `w` of course `wc` in Review.
    pub fn open_wave(&mut self, wc: usize, w: usize) {
        self.review.tab = 1;
        self.review.wave_course = Some(wc);
        self.review.practice = Some((wc, w));
        self.review.practice_state = Default::default();
        self.go(Screen::Review);
    }

    /// "Foundation" for a built-in course, else the course's own name.
    pub fn stage_name(&self, ci: usize) -> String {
        let base = self.base();
        let c = &self.courses[ci];
        match c.stage {
            Some(st) if c.builtin => st.name(&base).to_string(),
            _ => c.name_in(&base).to_string(),
        }
    }

    /// As in the books, a new learner reads the introduction before lesson 1:
    /// true while the active course has one and nothing is done or skipped yet.
    pub fn needs_intro(&self, ci: usize) -> bool {
        ci == self.active()
            && self.done_count(ci) == 0
            && !self.saved.read_intro.contains(&self.courses[ci].id)
            && self.appendix().is_some_and(|a| !a.intro.is_empty())
    }

    /// Opens the book at one of its pages (see `appendix::Page`).
    pub fn open_book(&mut self, page: usize) {
        self.appendix_ui.tab = page;
        self.go(Screen::Appendix);
    }

    /// Marks the introduction as read and starts the first lesson.
    pub fn begin_course(&mut self, ci: usize) {
        self.saved.read_intro.insert(self.courses[ci].id.clone());
        self.mark_dirty();
        if let Some(li) = self.next_lesson(ci) {
            self.start_session(ci, li);
        }
    }

    /// A lesson of this course was finished today.
    pub fn did_lesson_today(&self, ci: usize) -> bool {
        let c = &self.courses[ci];
        let t = day_key(today());
        c.lessons
            .iter()
            .any(|l| self.saved.progress(&c.lesson_key(l.number)).is_some_and(|p| p.completed.as_deref() == Some(t.as_str())))
    }

    pub fn done_count(&self, ci: usize) -> usize {
        let c = &self.courses[ci];
        c.lessons.iter().filter(|l| self.saved.is_done(&c.lesson_key(l.number))).count()
    }

    /// The recording for a lesson: its own `audio:` file, else a file in the
    /// course's audio folder whose name ends with the lesson number.
    pub fn recording(&mut self, ci: usize, li: usize) -> Option<PathBuf> {
        let c = &self.courses[ci];
        let l = c.lessons.get(li)?;
        let dir = self.saved.audio_dirs.get(&c.id).map(PathBuf::from);
        if !l.audio.trim().is_empty() {
            let p = PathBuf::from(l.audio.trim());
            let p = if p.is_absolute() {
                p
            } else if let Some(d) = &dir {
                d.join(p)
            } else {
                c.path.as_ref().and_then(|f| f.parent()).map(|d| d.join(&p)).unwrap_or(p)
            };
            if p.is_file() {
                return Some(p);
            }
        }
        let dir = dir?;
        let number = l.number;
        self.audio_maps
            .entry(c.id.clone())
            .or_insert_with(|| scan_audio_dir(&dir))
            .get(&number)
            .cloned()
    }

    pub fn say(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn save_now(&mut self) {
        if self.shot.is_some() {
            return; // a check run never writes the person's progress
        }
        if let Err(e) = self.saved.save() {
            self.say(trf("Could not save progress: {0}", &[&e]));
        }
        self.dirty = false;
        self.last_save = Instant::now();
    }

    pub fn stop_audio(&mut self) {
        self.seq = None;
        self.player.stop();
    }

    fn pick_audio_folder(&mut self, ci: usize) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title(tr("Choose the folder with this course's recordings"))
            .pick_folder()
        {
            let id = self.courses[ci].id.clone();
            let found = scan_audio_dir(&dir);
            self.say(trf(
                "Recordings linked to {1}: {0}",
                &[&found.len(), &self.courses[ci].name_in(&self.saved.base)],
            ));
            self.saved.audio_dirs.insert(id.clone(), dir.display().to_string());
            self.audio_maps.insert(id, found);
            self.mark_dirty();
        }
    }

    // ----------------------------------------------------------------- ui

    fn nav(&mut self, ui: &mut egui::Ui, compact: bool) {
        ui.horizontal(|ui| {
            if compact {
                ui.add_space((ui.available_width() - 36.0) / 2.0 - ui.spacing().item_spacing.x);
            }
            theme::logo(ui, 36.0);
            if !compact {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(bold("Lingua Onde").size(18.0).color(Color32::WHITE));
                    ui.label(RichText::new(tr("learn by assimilation")).size(12.0).color(NAV_MUTED));
                });
            }
        });
        ui.add_space(20.0);

        let base = self.base();
        let ci = self.active();
        let mut pick = None;
        if let (false, Some(c)) = (compact, self.courses.get(ci)) {
            egui::Frame::new()
                .fill(Color32::from_rgb(33, 47, 72))
                .corner_radius(12)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 6.0;
                    ui.label(caps(tr("Studying")).color(NAV_MUTED));
                    ui.horizontal(|ui| {
                        theme::dot(ui, lang_color(&c.lang), 5.0);
                        ui.label(bold(lang_name(&c.lang, &base)).size(16.0).color(Color32::WHITE));
                    });
                    if let Some(st) = c.stage.filter(|_| c.builtin) {
                        ui.label(
                            RichText::new(format!("{} · {}", st.name(&base), st.span()))
                                .size(12.5)
                                .color(NAV_TEXT),
                        );
                    }
                    let done = self.done_count(ci);
                    theme::bar_on(
                        ui,
                        done as f32 / c.lessons.len().max(1) as f32,
                        ui.available_width(),
                        WAVE,
                        Color32::from_rgb(52, 68, 96),
                    );
                    ui.label(
                        RichText::new(trf("{0} of {1}", &[&done, &plural(c.lessons.len(), "{0} lesson", "{0} lessons")]))
                            .size(12.5)
                            .color(NAV_TEXT),
                    );
                    let today = self.saved.days.get(&day_key(today())).map(|d| d.seconds).unwrap_or(0.0);
                    ui.label(
                        RichText::new(trf("🔥 {0} · {1} min today", &[&days(self.saved.streak()), &(today / 60.0).floor()]))
                            .size(12.5)
                            .color(NAV_TEXT),
                    );
                    pick = self.course_menu(ui, RichText::new(tr("⇄ Change course")).size(12.5).color(WAVE));
                });
            ui.add_space(14.0);
        } else if compact {
            ui.vertical_centered(|ui| {
                pick = self.course_menu(ui, RichText::new("⇄").size(18.0).color(WAVE));
            });
            ui.add_space(10.0);
        }
        if let Some(i) = pick.filter(|&i| i != ci) {
            self.saved.active_course = self.courses[i].id.clone();
            self.session = None;
            self.mark_dirty();
        }

        let due = self.cards_due() + self.wave2_pending(ci).len();
        let items = [
            (Screen::Home, "🏠", tr("Home"), 0),
            (Screen::Study, "▶", tr("Today's lesson"), 0),
            (Screen::Review, "⟳", tr("Review"), due),
            (Screen::Courses, "📚", tr("Courses"), 0),
            (Screen::Appendix, "📖", tr("The book"), 0),
            (Screen::Editor, "✏", tr("My lessons"), 0),
            (Screen::Settings, "⚙", tr("Settings & voices"), 0),
        ];
        ui.spacing_mut().item_spacing.y = 4.0;
        for (screen, icon, label, count) in items {
            if nav_item(ui, icon, label, self.screen == screen, count, compact).clicked() {
                self.go(screen);
            }
        }
    }

    /// "Change course" menu: every course with lessons, one submenu per language.
    fn course_menu(&self, ui: &mut egui::Ui, label: RichText) -> Option<usize> {
        let base = self.base();
        let ci = self.active();
        let mut pick = None;
        ui.menu_button(label, |ui| {
            let mut langs: Vec<&str> = Vec::new();
            for c in &self.courses {
                if !langs.contains(&c.lang.as_str()) {
                    langs.push(&c.lang);
                }
            }
            for lang in langs {
                ui.menu_button(lang_name(lang, &base), |ui| {
                    let mut own: Vec<usize> = (0..self.courses.len())
                        .filter(|&i| self.courses[i].lang == lang && !self.courses[i].lessons.is_empty())
                        .collect();
                    own.sort_by_key(|&i| (!self.courses[i].builtin, self.courses[i].stage.map(|s| s as u8)));
                    for i in own {
                        let sc = &self.courses[i];
                        let name = match sc.stage.filter(|_| sc.builtin) {
                            Some(st) => format!("{} · {}", st.name(&base), st.span()),
                            None => sc.name_in(&base).to_string(),
                        };
                        if ui.add(egui::Button::selectable(i == ci, name)).clicked() {
                            pick = Some(i);
                        }
                    }
                });
            }
        })
        .response
        .on_hover_text(tr("Change course"));
        pick
    }

    pub fn go(&mut self, screen: Screen) {
        if screen == Screen::Study && self.session.is_none() {
            let ci = self.active();
            if self.needs_intro(ci) {
                self.say(tr("As in the book, the introduction comes before lesson 1."));
                self.appendix_ui.tab = 0;
                self.screen = Screen::Appendix;
                return;
            }
            match self.next_lesson(ci) {
                // One lesson a day: after it, Home says so and offers the next
                // one only on request (a lesson already begun just resumes).
                Some(li)
                    if self.did_lesson_today(ci)
                        && !self
                            .saved
                            .progress(&self.courses[ci].lesson_key(self.courses[ci].lessons[li].number))
                            .is_some_and(|p| !p.steps_done.is_empty()) =>
                {
                    self.say(tr("You're done for today. If you really want another lesson, the link is under today's card."));
                    self.screen = Screen::Home;
                    return;
                }
                Some(li) => self.start_session(ci, li),
                None => {
                    self.say(if self.unwritten(ci) {
                        tr("You have done every lesson written so far; the rest of this course is still being written. Pick another in Courses meanwhile.")
                    } else {
                        tr("You finished every lesson of this course. Pick another in Courses or add your own.")
                    });
                    self.screen = Screen::Courses;
                    return;
                }
            }
        }
        if screen == Screen::Editor && self.editor.course.is_none() {
            self.open_editor(None);
        }
        if screen != self.screen {
            self.stop_audio();
        }
        if screen == Screen::Review {
            self.review.queue.clear();
            self.review.built = false;
        }
        self.screen = screen;
    }

    fn home(&mut self, ui: &mut egui::Ui) {
        let ci = self.active();
        let t = today();
        let today_log = self.saved.days.get(&day_key(t)).cloned().unwrap_or_default();
        let total_done: usize = (0..self.courses.len()).map(|i| self.done_count(i)).sum();
        let due = self.cards_due();
        let wide = ui.available_width() >= 760.0;

        let hello = match Local::now().hour() {
            5..=11 => tr("Good morning"),
            12..=17 => tr("Good afternoon"),
            _ => tr("Good evening"),
        };
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(bold(hello).size(28.0));
                ui.label(muted(trf("{0} · listen, repeat aloud, understand", &[&crate::i18n::long_date(t)])).size(15.0));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let streak = self.saved.streak();
                if streak > 0 {
                    theme::pill(ui, &plural(streak as usize, "🔥 {0} day in a row", "🔥 {0} days in a row"), WARN, WARN_SOFT);
                }
            });
        });
        ui.add_space(12.0);

        if let Some(c) = self.courses.get(ci).cloned() {
            let new_today = self.did_lesson_today(ci);
            self.hero(ui, ci, &c, new_today, due);
        }
        ui.add_space(12.0);

        let minutes = (today_log.seconds / 60.0).floor() as u32;
        let goal = self.saved.daily_goal_min.max(1);
        // Without the optional sentence review, the tile counts what the method
        // asks for: lessons of the second wave still to do.
        let (due_value, due_label) = if self.saved.cards_after_lesson || !self.saved.cards.is_empty() {
            (due, tr("sentences to review"))
        } else {
            (self.wave2_pending(ci).len(), tr("second-wave lessons to do"))
        };
        let stats = [
            ("🔥", WARN, WARN_SOFT, days(self.saved.streak()), tr("study streak"), None),
            (
                "⏱",
                ACCENT,
                ACCENT_SOFT,
                trf("{0} / {1} min", &[&minutes, &goal]),
                tr("studied today"),
                Some(minutes as f32 / goal as f32),
            ),
            (
                "⟳",
                Color32::from_rgb(120, 96, 200),
                Color32::from_rgb(236, 232, 250),
                due_value.to_string(),
                due_label,
                None,
            ),
            ("✔", GOOD, GOOD_SOFT, total_done.to_string(), tr("lessons completed"), None),
        ];
        let per_row = if wide { 4 } else { 2 };
        for row in stats.chunks(per_row) {
            ui.columns(per_row, |cols| {
                for (col, (icon, fg, bg, value, label, frac)) in cols.iter_mut().zip(row) {
                    // Columns are justified, which would stretch wrapped text.
                    col.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        theme::card().inner_margin(14).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_min_height(48.0);
                            ui.horizontal(|ui| {
                                theme::icon_circle(ui, icon, *fg, *bg, 42.0);
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 2.0;
                                    ui.label(bold(value.as_str()).size(21.0));
                                    ui.label(muted(*label).size(13.0));
                                    if let Some(f) = frac {
                                        theme::bar(ui, *f, ui.available_width().min(140.0), ACCENT);
                                    }
                                });
                            });
                        })
                    });
                }
            });
        }
        ui.add_space(6.0);

        if let Some(c) = self.courses.get(ci).cloned() {
            self.path_card(ui, ci, &c, wide);
        }
        ui.add_space(6.0);

        if wide {
            ui.columns(2, |cols| {
                cols[0].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| self.calendar_card(ui));
                cols[1].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| self.languages_card(ui));
            });
        } else {
            self.calendar_card(ui);
            self.languages_card(ui);
        }
    }

    /// The "continue" card: today's lesson, big, with its buttons.
    fn hero(&mut self, ui: &mut egui::Ui, ci: usize, c: &Course, studied_today: bool, due: usize) {
        let base = self.base();
        let light = Color32::from_rgb(196, 226, 233);
        egui::Frame::new()
            .fill(ACCENT_DARK)
            .corner_radius(18)
            .inner_margin(egui::Margin::symmetric(26, 22))
            .shadow(egui::Shadow {
                offset: [0, 6],
                blur: 18,
                spread: 0,
                color: Color32::from_black_alpha(30),
            })
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let decor = ui.painter().add(egui::Shape::Noop);
                ui.label(caps(&trf("Continue · {0}", &[&self.course_label(ci)])).color(WAVE));
                ui.add_space(4.0);
                match self.next_lesson(ci) {
                    Some(li) => {
                        let l = &c.lessons[li];
                        ui.horizontal(|ui| {
                            theme::badge(ui, l.level.name(), l.level.color());
                            let planned = c.stage.filter(|_| c.builtin).map_or(c.lessons.len() as u32, |st| st.size());
                            ui.label(medium(trf("Lesson {0} of {1}", &[&l.number, &planned])).color(light));
                        });
                        ui.label(bold(&l.title).size(30.0).color(Color32::WHITE));
                        ui.label(RichText::new(l.title_in(&base)).size(16.0).color(light));
                        ui.add_space(6.0);
                        if studied_today {
                            ui.label(
                                RichText::new(tr(
                                    "✔ You did today's lesson. One lesson a day is the rhythm of the method: the next one will be easier tomorrow.",
                                ))
                                .size(14.0)
                                .color(light),
                            );
                        }
                        if self.saved.progress(&c.lesson_key(l.number)).is_some_and(|p| p.again) {
                            ui.label(
                                RichText::new(tr("↻ You chose to go over this lesson again. Repeating is part of the method."))
                                    .size(14.0)
                                    .color(light),
                            );
                        }
                        // Today's path: the new lesson, the active wave, the sentence review.
                        let gold = Color32::from_rgb(255, 214, 140);
                        ui.horizontal_wrapped(|ui| {
                            let step = |ui: &mut egui::Ui, done: bool, text: String| {
                                let (icon, color) = if done { ("✔", light) } else { ("○", Color32::WHITE) };
                                ui.label(RichText::new(format!("{icon} {text}")).size(14.0).color(color));
                            };
                            ui.label(caps(tr("Today")).color(WAVE));
                            step(ui, studied_today, trf("new lesson {0}", &[&l.number]));
                            match self.wave2_for(ci, li) {
                                Some((wc, w)) if wc == ci => {
                                    step(ui, false, trf("active wave: lesson {0}", &[&c.lessons[w].number]))
                                }
                                Some((wc, w)) => step(
                                    ui,
                                    false,
                                    trf("active wave: {0} lesson {1}", &[&self.stage_name(wc), &self.courses[wc].lessons[w].number]),
                                ),
                                None if l.number < self.saved.wave2_gap + 1 => {
                                    let carried = self.wave2_left(ci) > self.wave2_open(ci).len();
                                    ui.label(
                                        RichText::new(if carried {
                                            tr("✔ today's active wave done").to_string()
                                        } else {
                                            trf("· active wave from lesson {0}", &[&(self.saved.wave2_gap + 1)])
                                        })
                                            .size(14.0)
                                            .color(light),
                                    );
                                }
                                None => {}
                            }
                            if due > 0 {
                                ui.label(RichText::new(plural(due, "○ review {0} sentence", "○ review {0} sentences")).size(14.0).color(gold));
                            }
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let resumable = self
                                .saved
                                .progress(&c.lesson_key(l.number))
                                .is_some_and(|p| !p.steps_done.is_empty());
                            let start = egui::Button::new(
                                medium(if resumable {
                                    tr("▶  Continue the lesson")
                                } else {
                                    tr("▶  Start today's lesson")
                                })
                                .size(16.5)
                                .color(ACCENT_DARK),
                            )
                            .fill(Color32::WHITE)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(12)
                            .min_size(egui::vec2(220.0, 46.0));
                            let intro_first = self.needs_intro(ci);
                            let start = if intro_first {
                                egui::Button::new(medium(tr("📖  Start with the introduction")).size(16.5).color(ACCENT_DARK))
                                    .fill(Color32::WHITE)
                                    .stroke(egui::Stroke::NONE)
                                    .corner_radius(12)
                                    .min_size(egui::vec2(220.0, 46.0))
                            } else {
                                start
                            };
                            // After today's lesson the method says stop: the next one
                            // stays reachable, but only through a quieter link below.
                            let rest = studied_today && !resumable && !intro_first;
                            if rest {
                                egui::Frame::new()
                                    .fill(Color32::from_white_alpha(20))
                                    .corner_radius(12)
                                    .inner_margin(egui::Margin::symmetric(18, 12))
                                    .show(ui, |ui| {
                                        ui.label(medium(tr("✔  You're done for today")).size(16.5).color(Color32::WHITE));
                                    });
                            } else if ui.add(start).clicked() {
                                if intro_first {
                                    self.open_book(0);
                                } else {
                                    self.start_session(ci, li);
                                }
                            }
                            if due > 0 {
                                let review = egui::Button::new(medium(plural(due, "⟳  Review {0} sentence", "⟳  Review {0} sentences")).color(Color32::WHITE))
                                    .fill(Color32::from_white_alpha(20))
                                    .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(70)))
                                    .corner_radius(12)
                                    .min_size(egui::vec2(0.0, 46.0));
                                if ui.add(review).clicked() {
                                    self.go(Screen::Review);
                                }
                            }
                        });
                        let resumable = self
                            .saved
                            .progress(&c.lesson_key(l.number))
                            .is_some_and(|p| !p.steps_done.is_empty());
                        if studied_today && !resumable && !self.needs_intro(ci) {
                            ui.add_space(6.0);
                            let more = egui::Button::new(
                                RichText::new(trf("I really want to do another lesson today: start lesson {0}", &[&l.number]))
                                    .size(13.5)
                                    .color(light)
                                    .underline(),
                            )
                            .frame(false);
                            if ui.add(more).clicked() {
                                self.start_session(ci, li);
                            }
                        }
                        // As in the books, the introduction comes before lesson 1.
                        if self.needs_intro(ci) {
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(tr("New here? The introduction comes first, as in the book: ten minutes on how the course works."))
                                    .size(14.0)
                                    .color(light),
                            );
                            let skip = egui::Button::new(
                                RichText::new(tr("I already know the method: start lesson 1")).size(13.5).color(light).underline(),
                            )
                            .frame(false);
                            if ui.add(skip).clicked() {
                                self.begin_course(ci);
                            }
                        }
                        // The books' advice for a day without time: a few minutes with
                        // the last dialogue rather than nothing.
                        let last = (0..li).rev().find(|&j| {
                            let lj = &c.lessons[j];
                            !lj.dialogue.is_empty() && self.saved.is_done(&c.lesson_key(lj.number))
                        });
                        if let (false, Some(j)) = (studied_today, last) {
                            let playing = self.seq.as_ref().is_some_and(|s| s.tag == "short");
                            let pl = &c.lessons[j];
                            let label = if playing {
                                tr("■  Stop").to_string()
                            } else if self.voices.ready(&c.lang) {
                                trf("⏱  Only a few minutes today? Listen to lesson {0} again and repeat it aloud.", &[&pl.number])
                            } else {
                                trf("⏱  Only a few minutes today? Read lesson {0} again, aloud.", &[&pl.number])
                            };
                            ui.add_space(6.0);
                            if ui.add(egui::Button::new(RichText::new(label).size(14.0).color(light)).frame(false)).clicked() {
                                if playing {
                                    self.stop_audio();
                                } else if self.voices.ready(&c.lang) {
                                    let texts: Vec<String> = pl.dialogue.iter().map(|i| i.plain()).collect();
                                    self.voices.prepare(&c.lang, &texts);
                                    self.play_seq("short", &c.lang, texts, 0, false);
                                } else {
                                    self.start_session(ci, j);
                                }
                            }
                        }
                    }
                    None if self.unwritten(ci) => {
                        ui.label(bold(tr("Next lessons on the way")).size(26.0).color(Color32::WHITE));
                        ui.label(
                            RichText::new(tr(
                                "You have done every lesson written so far; the rest of this course is still being written. Meanwhile, review, or choose another course in Courses.",
                            ))
                            .color(light),
                        );
                    }
                    None => {
                        // The book is read; its second wave goes on, one lesson a day,
                        // also once the next course of the language has started.
                        let pending = self.wave2_pending(ci);
                        let next_stage = c.stage.filter(|_| c.builtin).and_then(|st| {
                            self.stages_of(&c.lang)
                                .into_iter()
                                .find(|&si| {
                                    let n = &self.courses[si];
                                    n.stage.is_some_and(|s| s as u8 > st as u8) && !n.lessons.is_empty()
                                })
                        });
                        let then = match next_stage {
                            Some(si) => trf(
                                "You can start {0} now: each of its lessons brings one of these back with it.",
                                &[&self.stage_name(si)],
                            ),
                            None => tr("Then choose a new course in Courses, or add your own lessons.").to_string(),
                        };
                        let left = self.wave2_left(ci);
                        if let Some(&(wc, w)) = pending.first() {
                            let wl = &self.courses[wc].lessons[w];
                            ui.label(bold(tr("✔ Every lesson read")).size(26.0).color(Color32::WHITE));
                            ui.label(
                                RichText::new(format!(
                                    "{} {then}",
                                    trf(
                                        "The active wave goes on: {0} still to rebuild, one a day, as the method asks.",
                                        &[&plural(left, "{0} lesson", "{0} lessons")]
                                    )
                                ))
                                .size(15.0)
                                .color(light),
                            );
                            ui.add_space(8.0);
                            let label = if wc == ci {
                                trf("▶  Active wave: lesson {0}", &[&wl.number])
                            } else {
                                trf("▶  Active wave: {0} lesson {1}", &[&self.stage_name(wc), &wl.number])
                            };
                            let go = egui::Button::new(medium(label).size(16.5).color(ACCENT_DARK))
                                .fill(Color32::WHITE)
                                .stroke(egui::Stroke::NONE)
                                .corner_radius(12)
                                .min_size(egui::vec2(220.0, 46.0));
                            if ui.add(go).clicked() {
                                self.open_wave(wc, w);
                            }
                        } else if left > 0 {
                            ui.label(bold(tr("✔ Today's active wave is done")).size(26.0).color(Color32::WHITE));
                            ui.label(
                                RichText::new(format!(
                                    "{} {then}",
                                    trf(
                                        "{0} still to rebuild: the next one comes back tomorrow.",
                                        &[&plural(left, "{0} lesson", "{0} lessons")]
                                    )
                                ))
                                .size(15.0)
                                .color(light),
                            );
                        } else {
                            ui.label(bold(tr("🎉 Course complete")).size(26.0).color(Color32::WHITE));
                            ui.label(
                                RichText::new(match next_stage {
                                    Some(si) => trf(
                                        "You finished every lesson of this course, and its active wave too. {0} is next.",
                                        &[&self.stage_name(si)],
                                    ),
                                    None => tr("You finished every lesson of this course, and its active wave too. Choose a new one in Courses, or add your own lessons.").to_string(),
                                })
                                .color(light),
                            );
                        }
                        // The book ends with its appendix: the grammar to look back on.
                        if self.appendix().is_some_and(|a| !a.grammar.is_empty()) {
                            ui.add_space(6.0);
                            let book = egui::Button::new(
                                RichText::new(tr("📖  The end of the book: the grammar summary and the indexes")).size(14.0).color(light),
                            )
                            .frame(false);
                            if ui.add(book).clicked() {
                                self.open_book(3);
                            }
                        }
                    }
                }
                // Waves on the right half, behind the text.
                let r = ui.min_rect();
                if r.width() > 520.0 {
                    let area = egui::Rect::from_min_max(egui::pos2(r.right() - r.width() * 0.38, r.top()), r.max);
                    let shapes = [(0.0, 26), (1.3, 18), (2.6, 12)]
                        .into_iter()
                        .flat_map(|(phase, alpha)| wave_shapes(area, phase, Color32::from_white_alpha(alpha)))
                        .collect();
                    ui.painter().set(decor, egui::Shape::Vec(shapes));
                }
            });
    }

    /// Progress through the language's three courses and the levels of this one.
    fn path_card(&mut self, ui: &mut egui::Ui, ci: usize, c: &Course, wide: bool) {
        let base = self.base();
        let current = self.next_lesson(ci).map(|li| c.lessons[li].level);
        let stages = self.stages_of(&c.lang);
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(bold(tr("Your path")).size(18.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(muted(trf(
                        "{0} · {1} of {2} lessons",
                        &[&self.course_label(ci), &self.done_count(ci), &c.lessons.len()],
                    )));
                });
            });
            ui.add_space(4.0);
            if c.builtin && stages.len() > 1 {
                let mut pick = None;
                ui.columns(stages.len(), |cols| {
                    for (col, &si) in cols.iter_mut().zip(&stages) {
                        let sc = &self.courses[si];
                        let st = sc.stage.unwrap_or(Stage::Foundation);
                        let here = si == ci;
                        let done = self.done_count(si);
                        col.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            let r = egui::Frame::new()
                                .fill(if here { ACCENT_SOFT } else { Color32::TRANSPARENT })
                                .stroke(egui::Stroke::new(1.0, if here { ACCENT } else { LINE }))
                                .corner_radius(12)
                                .inner_margin(10)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.spacing_mut().item_spacing.y = 3.0;
                                    ui.label(bold(st.name(&base)).size(16.0));
                                    ui.label(muted(trf("{0} · {1} lessons planned", &[&st.span(), &st.size()])).size(12.5));
                                    theme::bar(ui, done as f32 / st.size() as f32, ui.available_width(), ACCENT);
                                    ui.label(muted(trf("{0} done · {1} written so far", &[&done, &sc.lessons.len()])).size(12.5));
                                })
                                .response
                                .interact(egui::Sense::click())
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text(if sc.lessons.is_empty() {
                                    tr("Not written yet")
                                } else {
                                    tr("Study this course")
                                });
                            if r.clicked() && !here && !sc.lessons.is_empty() {
                                pick = Some(si);
                            }
                        });
                    }
                });
                if let Some(si) = pick {
                    self.saved.active_course = self.courses[si].id.clone();
                    self.session = None;
                    self.mark_dirty();
                }
                ui.add_space(6.0);
            }
            let levels: Vec<Level> = Level::ALL
                .into_iter()
                .filter(|lvl| c.lessons.iter().any(|l| l.level == *lvl))
                .collect();
            let per_row = if wide { levels.len().clamp(1, 6) } else { 3 };
            for row in levels.chunks(per_row) {
                ui.columns(per_row, |cols| {
                    for (col, &lvl) in cols.iter_mut().zip(row) {
                        let keys: Vec<String> = c
                            .lessons
                            .iter()
                            .filter(|l| l.level == lvl)
                            .map(|l| c.lesson_key(l.number))
                            .collect();
                        let done = keys.iter().filter(|k| self.saved.is_done(k)).count();
                        let here = current == Some(lvl);
                        col.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            egui::Frame::new()
                                .fill(if here { theme::soft(lvl.color()) } else { Color32::TRANSPARENT })
                                .stroke(egui::Stroke::new(1.0, if here { lvl.color() } else { LINE }))
                                .corner_radius(12)
                                .inner_margin(10)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.spacing_mut().item_spacing.y = 4.0;
                                    ui.horizontal(|ui| {
                                        theme::badge(ui, lvl.name(), lvl.color());
                                        if !keys.is_empty() && done == keys.len() {
                                            ui.label(RichText::new("✔").color(GOOD));
                                        }
                                    });
                                    ui.add(egui::Label::new(muted(lvl.describe()).size(12.5)).truncate());
                                    ui.label(bold(format!("{done} / {}", keys.len())).size(16.0));
                                    let frac = if keys.is_empty() { 0.0 } else { done as f32 / keys.len() as f32 };
                                    theme::bar(ui, frac, ui.available_width(), lvl.color());
                                });
                        });
                    }
                });
            }
        });
    }

    fn calendar_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let weeks = (((ui.available_width() - 22.0) / 17.0).floor() as usize).clamp(8, 30);
            ui.horizontal(|ui| {
                ui.label(bold(tr("Study calendar")).size(18.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(muted(trf("last {0} weeks", &[&weeks])).size(13.0));
                });
            });
            self.heatmap(ui, weeks);
        });
    }

    fn languages_card(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        let active = self.active();
        let active_lang = self.courses.get(active).map(|c| c.lang.clone()).unwrap_or_default();
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold(tr("Your languages")).size(18.0));
            ui.add_space(2.0);
            let bar_w = (ui.available_width() - 290.0).clamp(40.0, 200.0);
            let mut choose = None;
            let mut langs: Vec<String> = Vec::new();
            for c in &self.courses {
                if !langs.contains(&c.lang) {
                    langs.push(c.lang.clone());
                }
            }
            for lang in langs {
                // The course in progress for this language: the active one, else the first not finished.
                let mine: Vec<usize> = (0..self.courses.len()).filter(|&i| self.courses[i].lang == lang).collect();
                let ci = mine
                    .iter()
                    .copied()
                    .find(|&i| i == active)
                    .or_else(|| mine.iter().copied().find(|&i| self.next_lesson(i).is_some()))
                    .unwrap_or(mine[0]);
                let c = &self.courses[ci];
                let done = self.done_count(ci);
                ui.horizontal(|ui| {
                    theme::dot(ui, lang_color(&lang), 5.0);
                    let label = ui
                        .allocate_ui_with_layout(egui::vec2(110.0, 22.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.set_min_width(110.0);
                            ui.add(
                                egui::Label::new(medium(lang_name(&lang, &base)))
                                    .truncate()
                                    .sense(egui::Sense::click()),
                            )
                        })
                        .inner;
                    let stage = c
                        .stage
                        .filter(|_| c.builtin)
                        .map(|s| s.name(&base).to_string())
                        .unwrap_or_else(|| tr("your lessons").into());
                    let (bar, _) = ui.allocate_exact_size(egui::vec2(bar_w, 8.0), egui::Sense::hover());
                    ui.put(bar, |ui: &mut egui::Ui| {
                        theme::bar(ui, done as f32 / c.lessons.len().max(1) as f32, bar_w, lang_color(&lang));
                        ui.response()
                    })
                    .on_hover_text(stage);
                    ui.label(muted(format!("{done}/{}", c.lessons.len())).size(13.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if lang == active_lang {
                            theme::pill(ui, tr("studying"), ACCENT, ACCENT_SOFT);
                        } else if ui.add(theme::ghost(tr("Study"))).clicked() || label.clicked() {
                            choose = Some(ci);
                        }
                    });
                });
            }
            if let Some(i) = choose {
                self.saved.active_course = self.courses[i].id.clone();
                self.session = None;
                self.mark_dirty();
            }
        });
    }

    fn heatmap(&self, ui: &mut egui::Ui, weeks: usize) {
        let gap = 3.0;
        let cell = ((ui.available_width() - 22.0) / weeks as f32 - gap).clamp(9.0, 18.0);
        let t = today();
        // The last column is the current week, Monday first.
        let start = t - Days::days(t.weekday().num_days_from_monday() as i64 + ((weeks - 1) * 7) as i64);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(weeks as f32 * (cell + gap) + 22.0, 7.0 * (cell + gap) + 4.0),
            egui::Sense::hover(),
        );
        let p = ui.painter();
        for (i, d) in [tr("M"), "", tr("W"), "", tr("F"), "", tr("S")].iter().enumerate() {
            p.text(
                egui::pos2(rect.left() + 6.0, rect.top() + i as f32 * (cell + gap) + cell / 2.0),
                egui::Align2::CENTER_CENTER,
                *d,
                egui::FontId::proportional(10.5),
                MUTED,
            );
        }
        let mut studied = 0;
        for w in 0..weeks {
            for d in 0..7 {
                let day = start + Days::days((w * 7 + d) as i64);
                if day > t {
                    continue;
                }
                let mins = self
                    .saved
                    .days
                    .get(&day_key(day))
                    .map(|l| l.seconds / 60.0 + l.lessons as f64 * 5.0)
                    .unwrap_or(0.0);
                if mins > 0.0 {
                    studied += 1;
                }
                let color = HEAT[match mins {
                    m if m <= 0.0 => 0,
                    m if m < 5.0 => 1,
                    m if m < 15.0 => 2,
                    m if m < 30.0 => 3,
                    _ => 4,
                }];
                let min = egui::pos2(rect.left() + 22.0 + w as f32 * (cell + gap), rect.top() + d as f32 * (cell + gap));
                let r = egui::Rect::from_min_size(min, egui::vec2(cell, cell));
                p.rect_filled(r, 3, color);
                if day == t {
                    p.rect_stroke(r, 3, egui::Stroke::new(1.5, INK), egui::StrokeKind::Outside);
                }
            }
        }
        ui.horizontal(|ui| {
            ui.label(muted(plural(studied, "{0} day with study", "{0} days with study")).size(12.5));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                ui.label(muted(tr("More")).size(12.0));
                for c in HEAT.iter().rev() {
                    let (r, _) = ui.allocate_exact_size(egui::vec2(11.0, 11.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, 2, *c);
                }
                ui.label(muted(tr("Less")).size(12.0));
            });
        });
    }

    fn courses_screen(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        theme::page_title(
            ui,
            tr("Courses"),
            tr("Each language has three courses in a row: Foundation (first words to B2, 100 lessons), Perfectionnement (B2 to C1) and Mastery (C1 to C2). Pick one to study, link your own recordings, or start a course for your own material."),
        );
        let active = self.active();
        let mut choose = None;
        let mut link = None;
        let mut copy = None;
        let mut from_rec = None;
        let mut guide = false;
        let gap = 12.0;
        let width = ui.available_width();

        // A card per language with its built-in courses, then the person's own.
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for (i, c) in self.courses.iter().enumerate() {
            if !c.builtin {
                groups.push(vec![i]);
            } else if let Some(g) = groups
                .iter_mut()
                .find(|g| self.courses[g[0]].builtin && self.courses[g[0]].lang == c.lang)
            {
                g.push(i);
            } else {
                groups.push(vec![i]);
            }
        }
        // Two cards a row on a wide window, one card alone at full width.
        let per_row = if width >= 760.0 && groups.len() > 1 { 2 } else { 1 };
        let card_w = (width - (per_row as f32 - 1.0) * gap) / per_row as f32 - 38.0;
        for row in groups.chunks(per_row) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for g in row {
                    let first = &self.courses[g[0]];
                    let lang = first.lang.clone();
                    let color = lang_color(&lang);
                    let holds_active = g.contains(&active);
                    let stroke = if holds_active { egui::Stroke::new(2.0, ACCENT) } else { egui::Stroke::new(1.0, LINE) };
                    theme::card().stroke(stroke).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(card_w);
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(egui::vec2(42.0, 42.0), egui::Sense::hover());
                                ui.painter().circle_filled(r.center(), 21.0, color);
                                ui.painter().text(
                                    r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    lang.to_uppercase(),
                                    egui::FontId::new(14.0, theme::bold_family()),
                                    Color32::WHITE,
                                );
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    let native = crate::model::lang(&lang).map(|l| l.native).unwrap_or(&lang);
                                    let title = if first.builtin { native.to_string() } else { first.name_in(&base).to_string() };
                                    ui.label(bold(title).size(18.0));
                                    ui.label(muted(if first.builtin {
                                        lang_name(&lang, &base)
                                    } else {
                                        trf("{0} · your material", &[&lang_name(&lang, &base)])
                                    }));
                                });
                                if first.builtin && store::pronunciation_key(&lang).is_some() {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                        if ui.add(theme::ghost(tr("🔈 Pronunciation"))).clicked() {
                                            if !holds_active {
                                                choose = Some(g[0]);
                                            }
                                            guide = true;
                                        }
                                    });
                                }
                            });
                            ui.add_space(4.0);
                            for &i in g {
                                let c = &self.courses[i];
                                let done = c.lessons.iter().filter(|l| self.saved.is_done(&c.lesson_key(l.number))).count();
                                let planned = c.stage.filter(|_| c.builtin).map(|s| s.size() as usize).unwrap_or(c.lessons.len());
                                egui::Frame::new()
                                    .fill(if i == active { ACCENT_SOFT } else { Color32::TRANSPARENT })
                                    .corner_radius(10)
                                    .inner_margin(egui::Margin::symmetric(8, 6))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.spacing_mut().item_spacing.y = 2.0;
                                                let name = match c.stage.filter(|_| c.builtin) {
                                                    Some(st) => format!("{} · {}", st.name(&base), st.span()),
                                                    None => tr("Your lessons").into(),
                                                };
                                                ui.label(medium(name));
                                                theme::bar(ui, done as f32 / planned.max(1) as f32, 150.0, color);
                                                let mut parts = vec![trf("{0} done", &[&done])];
                                                if c.builtin && c.lessons.is_empty() {
                                                    parts.push(tr("being written").to_string());
                                                } else if c.builtin && c.lessons.len() < planned {
                                                    parts.push(trf("{0} of {1} ready", &[&c.lessons.len(), &planned]));
                                                }
                                                if !c.speaks(&base) {
                                                    let names: Vec<String> = c.bases().iter().map(|b| lang_name(b, &base)).collect();
                                                    parts.push(trf("for now in {0}", &[&names.join(", ")]));
                                                }
                                                ui.label(muted(parts.join(" · ")).size(12.5));
                                            });
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if i == active {
                                                    theme::pill(ui, tr("studying"), ACCENT, ACCENT_SOFT);
                                                } else if c.lessons.is_empty() {
                                                    ui.label(muted(tr("coming soon")));
                                                } else if ui.add(theme::primary(tr("Study"))).clicked() {
                                                    choose = Some(i);
                                                }
                                                // Rare actions sit in a labelled menu, not as bare icons.
                                                if !c.lessons.is_empty() {
                                                    ui.menu_button(RichText::new(tr("More…")).size(14.0), |ui| {
                                                        if ui
                                                            .button(tr("🎧  Link my recordings of this course…"))
                                                            .on_hover_text(tr("A folder of audio files named by lesson, e.g. \"Lesson 12.mp3\""))
                                                            .clicked()
                                                        {
                                                            link = Some(i);
                                                        }
                                                        if c.builtin && ui.button(tr("📋  Copy to my own courses, to change it")).clicked() {
                                                            copy = Some(i);
                                                        }
                                                        if !c.builtin
                                                            && self.saved.audio_dirs.contains_key(&c.id)
                                                            && ui.button(tr("＋  A lesson for each recording without text")).clicked()
                                                        {
                                                            from_rec = Some(i);
                                                        }
                                                    })
                                                    .response
                                                    .on_hover_text(tr("More actions"));
                                                }
                                            });
                                        });
                                    });
                            }
                        });
                    });
                }
            });
            ui.add_space(gap - ui.spacing().item_spacing.y);
        }
        if let Some(i) = choose {
            self.saved.active_course = self.courses[i].id.clone();
            self.session = None;
            self.mark_dirty();
        }
        if guide {
            self.key_open = true;
        }
        if let Some(i) = link {
            self.pick_audio_folder(i);
        }
        if let Some(i) = copy {
            self.copy_course(i);
        }
        if let Some(i) = from_rec {
            self.lessons_from_recordings(i);
        }

        ui.add_space(4.0);
        theme::tinted(ACCENT_SOFT).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(medium(tr("New course for your own material:")));
                egui::ComboBox::from_id_salt("newlang")
                    .selected_text(lang_name(&self.new_course_lang, &base))
                    .show_ui(ui, |ui| {
                        for l in LANGS {
                            ui.selectable_value(&mut self.new_course_lang, l.code.to_string(), lang_name(l.code, &base));
                        }
                    });
                if ui.add(theme::primary(tr("＋ Create"))).clicked() {
                    let lang = self.new_course_lang.clone();
                    self.create_course(&lang);
                }
            });
            ui.label(muted(tr("Then add lessons in “My lessons”: type or paste the dialogue, notes and exercises.")).size(13.5));
        });

        ui.add_space(10.0);
        self.lesson_list(ui, active);
    }

    fn lesson_list(&mut self, ui: &mut egui::Ui, ci: usize) {
        let base = self.base();
        let Some(c) = self.courses.get(ci).cloned() else { return };
        theme::section(ui, &trf("Lessons · {0}", &[&self.course_label(ci)]));
        let next = self.next_lesson(ci);
        let next_level = next.map(|li| c.lessons[li].level);
        let wave = self.wave2_due(ci);
        let mut start = None;
        let mut edit = None;
        for lvl in Level::ALL {
            let lessons: Vec<(usize, &crate::model::Lesson)> = c.lessons.iter().enumerate().filter(|(_, l)| l.level == lvl).collect();
            if lessons.is_empty() {
                continue;
            }
            let done = lessons.iter().filter(|(_, l)| self.saved.is_done(&c.lesson_key(l.number))).count();
            let header = bold(format!("{} · {}   {done} / {}", lvl.name(), lvl.describe(), lessons.len())).size(15.5);
            egui::CollapsingHeader::new(header)
                .id_salt((&c.id, lvl.name()))
                .default_open(next_level == Some(lvl) || (next_level.is_none() && lvl == Level::A1))
                .show(ui, |ui| {
                    theme::card().inner_margin(egui::Margin::symmetric(8, 6)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 2.0;
                        for (li, l) in lessons {
                            let key = c.lesson_key(l.number);
                            let p = self.saved.progress(&key).cloned().unwrap_or_default();
                            let is_next = Some(li) == next;
                            egui::Frame::new()
                                .fill(if is_next { ACCENT_SOFT } else { Color32::TRANSPARENT })
                                .corner_radius(8)
                                .inner_margin(egui::Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        let (icon, color) = if p.completed.is_some() {
                                            ("✔", GOOD)
                                        } else if is_next {
                                            ("▶", ACCENT)
                                        } else {
                                            ("○", Color32::from_rgb(190, 186, 176))
                                        };
                                        ui.add_sized(egui::vec2(18.0, 22.0), egui::Label::new(RichText::new(icon).color(color)));
                                        ui.add_sized(
                                            egui::vec2(30.0, 22.0),
                                            egui::Label::new(medium(l.number.to_string()).color(MUTED).size(13.5)),
                                        );
                                        ui.label(medium(&l.title));
                                        ui.add(egui::Label::new(muted(l.title_in(&base)).size(14.0)).truncate());
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // A built-in course goes in order, as the book does: done
                                            // lessons can be reread, the next one studied, the rest wait.
                                            let open = !c.builtin || p.completed.is_some() || is_next;
                                            let label = if p.completed.is_some() { tr("Again") } else { tr("Study") };
                                            let btn = if is_next { theme::primary(label) } else { theme::ghost(label) };
                                            if open && ui.add(btn).clicked() {
                                                start = Some(li);
                                            }
                                            if !c.builtin && ui.add(theme::ghost("✏")).on_hover_text(tr("Edit this lesson")).clicked() {
                                                edit = Some(li);
                                            }
                                            if p.wave2.is_some() {
                                                theme::pill(ui, tr("active ✔"), GOOD, GOOD_SOFT);
                                            } else if wave.contains(&li) {
                                                theme::pill(ui, tr("active wave due"), WARN, WARN_SOFT);
                                            }
                                            if p.again {
                                                theme::pill(ui, tr("again tomorrow"), ACCENT, ACCENT_SOFT);
                                            }
                                            if l.revision {
                                                theme::pill(ui, tr("revision"), MUTED, theme::soft(MUTED));
                                            }
                                            if p.score.1 > 0 {
                                                ui.label(muted(trf("ex. {0}/{1}", &[&p.score.0, &p.score.1])).size(12.5));
                                            }
                                        });
                                    });
                                });
                        }
                    });
                });
        }
        if c.lessons.is_empty() {
            ui.label(muted(tr("No lessons yet. Add them in “My lessons”.")));
        }
        if let Some(li) = start {
            if self.needs_intro(ci) {
                self.open_book(0);
            } else {
                self.start_session(ci, li);
            }
        }
        if let Some(li) = edit {
            let id = c.id.clone();
            self.open_editor(Some((id, Some(li))));
        }
    }

    fn create_course(&mut self, lang: &str) {
        let id = store::new_course_id(lang, &self.courses);
        let mut name_tr = BTreeMap::new();
        name_tr.insert("it".to_string(), format!("Il mio {}", lang_name(lang, "it").to_lowercase()));
        name_tr.insert("en".to_string(), format!("My {}", lang_name(lang, "en")));
        let course = Course {
            id: id.clone(),
            lang: lang.to_string(),
            name: format!("My {}", lang_name(lang, "en")),
            name_tr,
            ..Default::default()
        };
        match store::save_course(&course) {
            Ok(path) => {
                self.courses.push(Course {
                    path: Some(path),
                    ..course
                });
                self.open_editor(Some((id, None)));
                self.say(tr("Course created. Add your first lesson."));
            }
            Err(e) => self.say(e),
        }
    }

    /// One text-less lesson per linked recording that has no lesson yet, so
    /// the person can study with the printed book and add the text later.
    fn lessons_from_recordings(&mut self, ci: usize) {
        let Some(dir) = self.saved.audio_dirs.get(&self.courses[ci].id).map(PathBuf::from) else {
            return;
        };
        let found = scan_audio_dir(&dir);
        let c = &mut self.courses[ci];
        let mut added = 0;
        for &n in found.keys() {
            if n == 0 || c.lessons.iter().any(|l| l.number == n) {
                continue;
            }
            // shortcut: level guessed from the position in a ~100-lesson course; edit it in My lessons.
            let level = match n {
                0..=20 => Level::A1,
                21..=45 => Level::A2,
                46..=75 => Level::B1,
                _ => Level::B2,
            };
            let mut title_tr = BTreeMap::new();
            title_tr.insert("it".to_string(), format!("Lezione {n}"));
            title_tr.insert("en".to_string(), format!("Lesson {n}"));
            c.lessons.push(crate::model::Lesson {
                number: n,
                level,
                title: format!("Lesson {n}"),
                title_tr,
                ..Default::default()
            });
            added += 1;
        }
        c.sort();
        let saved = store::save_course(c);
        self.audio_maps.insert(self.courses[ci].id.clone(), found);
        match saved {
            Ok(_) => self.say(trf(
                "Lessons added from the recordings: {0}. Follow them in your book; add the text any time in My lessons.",
                &[&added],
            )),
            Err(e) => self.say(e),
        }
    }

    fn copy_course(&mut self, ci: usize) {
        let src = self.courses[ci].clone();
        let id = store::new_course_id(&src.lang, &self.courses);
        let mut c = Course {
            id: id.clone(),
            builtin: false,
            path: None,
            ..src
        };
        for (lang, n) in c.name_tr.iter_mut() {
            n.push_str(if lang == "it" { " (mio)" } else { " (mine)" });
        }
        c.name.push_str(" (mine)");
        match store::save_course(&c) {
            Ok(path) => {
                c.path = Some(path);
                self.courses.push(c);
                self.say(tr("Copied to your courses: you can now expand and edit it in “My lessons”."));
            }
            Err(e) => self.say(e),
        }
    }

    fn settings_screen(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        theme::page_title(
            ui,
            tr("Settings & voices"),
            tr("How lessons are shown and read aloud. Changes are saved by themselves."),
        );
        let mut changed = false;
        let label_w = 210.0;
        let row = |ui: &mut egui::Ui, title: &str, hint: &str, add: &mut dyn FnMut(&mut egui::Ui)| {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(egui::vec2(label_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_width(label_w);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(medium(title));
                    if !hint.is_empty() {
                        ui.label(muted(hint).size(12.5));
                    }
                });
                // A vertical column lets long checkbox texts wrap instead of running off the card.
                ui.vertical(|ui| add(ui));
            });
            ui.add_space(6.0);
        };

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold(tr("Your language")).size(18.0));
            ui.label(muted(tr("Translations, notes and the spelled-out pronunciation are shown in this language. The built-in courses are written for Italian, English and Spanish speakers, one pair at a time: Courses shows when a course is not yet in your language. Your own lessons can use any.")).size(14.0));
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("base").selected_text(lang_name(&self.saved.base, &base)).show_ui(ui, |ui| {
                for code in ["it", "en", "es", "fr", "de", "pt"] {
                    changed |= ui
                        .selectable_value(&mut self.saved.base, code.to_string(), crate::model::lang(code).map(|l| l.native).unwrap_or(code))
                        .changed();
                }
            });
        });
        ui.add_space(4.0);

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold(tr("Method")).size(18.0));
            ui.add_space(4.0);
            row(ui, tr("Daily goal"), tr("minutes of study a day"), &mut |ui| {
                changed |= ui
                    .add(egui::Slider::new(&mut self.saved.daily_goal_min, 5..=90).suffix(" min"))
                    .changed();
            });
            row(ui, tr("Second wave starts"), tr("lessons later (classic: 49)"), &mut |ui| {
                changed |= ui.add(egui::DragValue::new(&mut self.saved.wave2_gap).range(1..=120)).changed();
            });
            row(ui, tr("Daily sentence review"), "", &mut |ui| {
                changed |= ui
                    .checkbox(
                        &mut self.saved.cards_after_lesson,
                        tr("Add each finished dialogue's sentences to a spaced review (an extra, not part of the Assimil method)"),
                    )
                    .changed();
            });
            row(ui, tr("Pronunciation hints"), "", &mut |ui| {
                changed |= ui
                    .checkbox(
                        &mut self.saved.show_pron,
                        tr("Show how to pronounce: spelled-out sounds, pinyin, Latin vowel lengths"),
                    )
                    .changed();
            });
        });
        ui.add_space(4.0);

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold(tr("Listening")).size(18.0));
            ui.add_space(4.0);
            row(ui, tr("Speed"), "", &mut |ui| {
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.saved.speed, 0.6..=1.4)
                            .step_by(0.05)
                            .fixed_decimals(2)
                            .suffix("×"),
                    )
                    .changed();
            });
            row(ui, tr("Gradual pace"), tr("as the books' recordings"), &mut |ui| {
                changed |= ui
                    .checkbox(
                        &mut self.saved.gradual_pace,
                        tr("Slower voice in the first lessons of Foundation, natural from lesson 50"),
                    )
                    .changed();
            });
            row(ui, tr("Pause to repeat"), tr("how long the voice waits after each sentence"), &mut |ui| {
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.saved.pause_factor, 0.5..=3.0)
                            .step_by(0.1)
                            .fixed_decimals(1)
                            .suffix(tr("× the sentence")),
                    )
                    .changed();
            });
            row(ui, tr("Recordings vs voice"), "", &mut |ui| {
                changed |= ui
                    .checkbox(
                        &mut self.saved.prefer_voice,
                        tr("Use the voice even when a recording is linked (sentence by sentence)"),
                    )
                    .changed();
            });
            row(ui, tr("Audio player"), "", &mut |ui| {
                if crate::audio::find_program("mpv").is_some() {
                    theme::pill(ui, "mpv ✔", GOOD, GOOD_SOFT);
                } else if Player::available() {
                    theme::pill(ui, tr("System player ✔"), GOOD, GOOD_SOFT);
                    ui.label(muted(tr("mpv adds pause and rewind, and plays your own MP3 recordings")).size(13.0));
                } else {
                    ui.label(RichText::new(tr("No player found: install mpv to hear audio")).color(BAD));
                }
            });
        });
        ui.add_space(4.0);

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold(tr("Voices")).size(18.0));
            ui.label(muted(tr("Free and offline after the download. A natural-sounding voice reads every sentence aloud when you have no recordings. Each voice is about 60 MB; the first one also fetches the voice engine (25 MB). Latin is read by the Italian voice with Latin pronunciation rules.")).size(14.0));
            ui.add_space(6.0);
            for l in LANGS {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(egui::vec2(label_w, 30.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.set_width(label_w);
                        theme::dot(ui, l.color, 5.0);
                        ui.label(medium(lang_name(l.code, &base)));
                    });
                    let status = self.voices.status_of(l.code);
                    if self.voices.ready(l.code) {
                        theme::pill(ui, tr("Ready ✔"), GOOD, GOOD_SOFT);
                    } else if crate::audio::downloading(&status) {
                        let mb = self.voices.partial_mb(l.code).map(|m| format!(" {m:.0} MB")).unwrap_or_default();
                        ui.spinner();
                        ui.label(format!("{status}{mb}"));
                        ui.ctx().request_repaint_after(Duration::from_millis(400));
                    } else {
                        if ui.add(theme::secondary(tr("⬇ Download"))).clicked() {
                            self.voices.download(l.code, ui.ctx().clone());
                        }
                        if !status.is_empty() {
                            ui.label(RichText::new(&status).color(BAD));
                        }
                    }
                });
            }
        });
        ui.add_space(4.0);

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold(tr("Your files")).size(18.0));
            let dir = store::data_dir();
            ui.label(muted(trf("Progress, your courses and voices are kept in {0}", &[&dir.display()])).size(14.0));
            if ui.add(theme::secondary(tr("📂 Open the folder"))).clicked() {
                let _ = std::fs::create_dir_all(&dir);
                open_path(&dir);
            }
            for p in &self.problems {
                ui.label(RichText::new(format!("⚠ {p}")).color(BAD));
            }
        });

        if changed {
            self.mark_dirty();
        }
    }
}

const HEAT: [Color32; 5] = [
    Color32::from_rgb(234, 230, 221),
    Color32::from_rgb(190, 224, 230),
    Color32::from_rgb(118, 186, 202),
    Color32::from_rgb(52, 142, 164),
    Color32::from_rgb(20, 88, 104),
];

fn days(n: u32) -> String {
    plural(n as usize, "{0} day", "{0} days")
}

/// Three soft waves filling `area`, for the home card's background.
fn wave_shapes(area: egui::Rect, phase: f32, color: Color32) -> Vec<egui::Shape> {
    (0..3)
        .map(|row| {
            let y0 = area.top() + area.height() * (0.3 + 0.2 * row as f32);
            let amp = area.height() * 0.08;
            let points = (0..=48)
                .map(|i| {
                    let t = i as f32 / 48.0;
                    egui::pos2(
                        area.left() + t * area.width(),
                        y0 + amp * (t * std::f32::consts::TAU * 1.2 + phase + row as f32 * 0.8).sin(),
                    )
                })
                .collect();
            egui::Shape::line(points, egui::Stroke::new(2.0, color))
        })
        .collect()
}

/// One row of the side navigation.
fn nav_item(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool, count: usize, compact: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 42.0), egui::Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, 10, ACCENT);
    } else if resp.hovered() {
        p.rect_filled(rect, 10, Color32::from_white_alpha(12));
    }
    let color = if selected { Color32::WHITE } else { NAV_TEXT };
    let y = rect.center().y;
    let icon_x = if compact { rect.center().x } else { rect.left() + 22.0 };
    p.text(
        egui::pos2(icon_x, y),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(16.0),
        color,
    );
    if !compact {
        p.text(
            egui::pos2(rect.left() + 44.0, y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::new(15.0, theme::medium_family()),
            color,
        );
    }
    if count > 0 {
        let (c, r) = if compact {
            (egui::pos2(rect.right() - 6.0, rect.top() + 8.0), 8.0)
        } else {
            (egui::pos2(rect.right() - 20.0, y), 10.0)
        };
        p.circle_filled(c, r, WARN);
        p.text(
            c,
            egui::Align2::CENTER_CENTER,
            count.min(99).to_string(),
            egui::FontId::new(11.0, theme::medium_family()),
            Color32::WHITE,
        );
    }
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if compact { resp.on_hover_text(label) } else { resp }
}

impl App {
    /// The figured-pronunciation key of the language studied: the part for
    /// every reader and the part for this reader.
    fn key_window(&mut self, ctx: &egui::Context) {
        let lang = self.courses.get(self.active()).map(|c| c.lang.clone()).unwrap_or_default();
        let Some(key) = store::pronunciation_key(&lang) else {
            self.key_open = false;
            return;
        };
        let base = self.base();
        let mut open = self.key_open;
        egui::Window::new(trf("How to read the pronunciation · {0}", &[&lang_name(&lang, &base)]))
            .open(&mut open)
            .default_width(680.0)
            .max_width(680.0)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .collapsible(false)
            .show(ctx, |ui| {
                let height = ctx.content_rect().height() * 0.7;
                egui::ScrollArea::vertical().max_height(height).show(ui, |ui| {
                    ui.set_max_width(ui.available_width());
                    theme::markdown(ui, &reader_part(key, &base));
                });
            });
        self.key_open = open;
    }
}

/// The sections of a pronunciation key meant for readers of `base`: the
/// shared ones and the one headed for that reader.
pub fn reader_part(key: &str, base: &str) -> String {
    let readers = [("it", "Italian"), ("en", "English"), ("es", "Spanish")];
    let mine = readers.iter().find(|(c, _)| *c == base).map(|(_, n)| *n).unwrap_or("English");
    let mut out = String::new();
    let mut keep = true;
    for line in key.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            let other = readers.iter().any(|(_, n)| *n != mine && h.contains(&format!("{n} readers")));
            keep = !other;
        }
        if keep {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

pub fn open_path(p: &std::path::Path) {
    let prog = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(prog).arg(p).spawn();
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        crate::i18n::set_base(&self.saved.base);

        // Study time counts while a lesson or review is open.
        let dt = self.tick.elapsed().min(Duration::from_secs(5)).as_secs_f64();
        self.tick = Instant::now();
        if matches!(self.screen, Screen::Study | Screen::Review) && self.shot.is_none() {
            if ctx.input(|i| i.focused) || self.player.busy() {
                self.saved.day().seconds += dt;
                self.dirty = true;
            }
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        self.drive_audio(&ctx);

        // A tiled or small window gets an icon-only navigation.
        let compact = ui.available_width() < 1000.0;
        egui::Panel::left("nav")
            .exact_size(if compact { 72.0 } else { 232.0 })
            .resizable(false)
            .frame(egui::Frame::new().fill(NAV).inner_margin(egui::Margin::symmetric(12, 18)))
            .show(ui, |ui| self.nav(ui, compact));

        if let Some((msg, at)) = self.toast.clone() {
            if at.elapsed() < Duration::from_secs(5) {
                egui::Area::new(egui::Id::new("toast"))
                    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(if compact { 36.0 } else { 116.0 }, -90.0))
                    .order(egui::Order::Foreground)
                    .interactable(false)
                    .show(&ctx, |ui| {
                        egui::Frame::new()
                            .fill(INK)
                            .corner_radius(12)
                            .inner_margin(egui::Margin::symmetric(18, 12))
                            .shadow(egui::Shadow {
                                offset: [0, 4],
                                blur: 16,
                                spread: 0,
                                color: Color32::from_black_alpha(50),
                            })
                            .show(ui, |ui| {
                                ui.set_max_width(560.0);
                                ui.label(RichText::new(msg).color(Color32::WHITE));
                            });
                    });
                ctx.request_repaint_after(Duration::from_millis(500));
            } else {
                self.toast = None;
            }
        }

        egui::CentralPanel::default_margins().show(ui, |ui| match self.screen {
            Screen::Study => self.study_screen(ui),
            screen => {
                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    ui.add_space(10.0);
                    theme::page(ui, PAGE, |ui| match screen {
                        Screen::Home => self.home(ui),
                        Screen::Courses => self.courses_screen(ui),
                        Screen::Review => self.review_screen(ui),
                        Screen::Editor => self.editor_screen(ui),
                        Screen::Settings => self.settings_screen(ui),
                        Screen::Appendix => self.appendix_screen(ui),
                        Screen::Study => {}
                    });
                    ui.add_space(24.0);
                });
            }
        });

        if self.key_open {
            self.key_window(&ctx);
        }

        if self.dirty && self.last_save.elapsed() > Duration::from_secs(5) {
            self.save_now();
        }

        // --screenshot: draw ~120 frames, save the picture, close.
        if let Some(shot) = self.shot.as_mut() {
            shot.frames += 1;
            ctx.request_repaint();
            if shot.frames >= 120 && !shot.requested {
                shot.requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            let image = ctx.input(|i| {
                i.raw.events.iter().chain(i.events.iter()).find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(image) = image {
                if let Err(e) = save_png(&shot.path, &image) {
                    eprintln!("could not save the screenshot: {e}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn on_exit(&mut self) {
        self.stop_audio();
        if self.dirty {
            self.save_now();
        }
    }
}

fn save_png(path: &std::path::Path, image: &egui::ColorImage) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), image.size[0] as u32, image.size[1] as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    enc.write_header()
        .and_then(|mut w| w.write_image_data(&bytes))
        .map_err(|e| e.to_string())
}
