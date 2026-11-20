use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{Datelike, Duration as Days, Local, Timelike};
use eframe::egui::{self, Color32, RichText};

use crate::audio::{Player, Voices, scan_audio_dir};
use crate::editor::EditorState;
use crate::model::{Course, LANGS, Level, lang_color, lang_name};
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
}

/// The widest a screen's text column grows; wider windows centre it.
const PAGE: f32 = 1080.0;

impl App {
    pub fn new(ctx: &egui::Context, screenshot: Option<PathBuf>, open: Option<String>) -> App {
        let (courses, problems) = store::load_courses();
        let saved = Saved::load();
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
        };
        if app.course_index(&app.saved.active_course.clone()).is_none() {
            app.saved.active_course = app.courses.first().map(|c| c.id.clone()).unwrap_or_default();
        }
        match open.as_deref() {
            Some("courses") => app.screen = Screen::Courses,
            Some("review") => app.screen = Screen::Review,
            Some(s) if s.starts_with("editor") => {
                app.open_editor(None);
                if let Some(li) = s.split(':').nth(1).and_then(|n| n.parse().ok()) {
                    app.edit_lesson(li);
                }
            }
            Some("settings") => app.screen = Screen::Settings,
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

    pub fn next_lesson(&self, ci: usize) -> Option<usize> {
        let c = self.courses.get(ci)?;
        c.lessons.iter().position(|l| !self.saved.is_done(&c.lesson_key(l.number)))
    }

    /// Lessons whose active "second wave" is due: lesson i comes back once
    /// lesson i + gap has been studied (classic: lesson 1 with lesson 50).
    pub fn wave2_due(&self, ci: usize) -> Vec<usize> {
        let c = &self.courses[ci];
        let gap = self.saved.wave2_gap.max(1) as usize;
        (0..c.lessons.len())
            .filter(|&i| {
                let key = c.lesson_key(c.lessons[i].number);
                let done = |j: usize| c.lessons.get(j).is_some_and(|l| self.saved.is_done(&c.lesson_key(l.number)));
                done(i) && self.saved.progress(&key).is_some_and(|p| p.wave2.is_none()) && done(i + gap)
            })
            .collect()
    }

    /// The second-wave lesson that goes with studying lesson `li`: its partner
    /// (lesson 1 with lesson 50), else the oldest one left behind.
    pub fn wave2_for(&self, ci: usize, li: usize) -> Option<usize> {
        let c = &self.courses[ci];
        let partner = li.checked_sub(self.saved.wave2_gap.max(1) as usize).filter(|&w| {
            let k = c.lesson_key(c.lessons[w].number);
            self.saved.is_done(&k) && self.saved.progress(&k).is_some_and(|p| p.wave2.is_none())
        });
        partner.or_else(|| self.wave2_due(ci).into_iter().find(|&w| w != li))
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
            self.say(format!("Could not save progress: {e}"));
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
            .set_title("Choose the folder with this course's recordings")
            .pick_folder()
        {
            let id = self.courses[ci].id.clone();
            let found = scan_audio_dir(&dir);
            self.say(format!(
                "Linked {} recordings to {}",
                found.len(),
                self.courses[ci].name_in(&self.saved.base)
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
                    ui.label(RichText::new("learn by assimilation").size(12.0).color(NAV_MUTED));
                });
            }
        });
        ui.add_space(20.0);

        let base = self.base();
        let ci = self.active();
        if let (false, Some(c)) = (compact, self.courses.get(ci)) {
            egui::Frame::new()
                .fill(Color32::from_rgb(33, 47, 72))
                .corner_radius(12)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 6.0;
                    ui.label(caps("Studying").color(NAV_MUTED));
                    ui.horizontal(|ui| {
                        theme::dot(ui, lang_color(&c.lang), 5.0);
                        ui.label(bold(lang_name(&c.lang, &base)).size(16.0).color(Color32::WHITE));
                    });
                    let done = self.done_count(ci);
                    theme::bar_on(
                        ui,
                        done as f32 / c.lessons.len().max(1) as f32,
                        ui.available_width(),
                        WAVE,
                        Color32::from_rgb(52, 68, 96),
                    );
                    ui.label(
                        RichText::new(format!("{done} of {}", plural(c.lessons.len(), "lesson")))
                            .size(12.5)
                            .color(NAV_TEXT),
                    );
                    let today = self.saved.days.get(&day_key(today())).map(|d| d.seconds).unwrap_or(0.0);
                    ui.label(
                        RichText::new(format!("🔥 {} · {} min today", days(self.saved.streak()), (today / 60.0).floor()))
                            .size(12.5)
                            .color(NAV_TEXT),
                    );
                });
            ui.add_space(14.0);
        }

        let due = self.saved.cards_due() + self.wave2_due(ci).len();
        let items = [
            (Screen::Home, "🏠", "Home", 0),
            (Screen::Study, "▶", "Today's lesson", 0),
            (Screen::Review, "⟳", "Review", due),
            (Screen::Courses, "📚", "Courses", 0),
            (Screen::Editor, "✏", "My lessons", 0),
            (Screen::Settings, "⚙", "Settings & voices", 0),
        ];
        ui.spacing_mut().item_spacing.y = 4.0;
        for (screen, icon, label, count) in items {
            if nav_item(ui, icon, label, self.screen == screen, count, compact).clicked() {
                self.go(screen);
            }
        }
    }

    pub fn go(&mut self, screen: Screen) {
        if screen == Screen::Study && self.session.is_none() {
            let ci = self.active();
            match self.next_lesson(ci) {
                Some(li) => self.start_session(ci, li),
                None => {
                    self.say("You finished every lesson of this course. Pick another in Courses or add your own.");
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
        let due = self.saved.cards_due();
        let wide = ui.available_width() >= 760.0;

        let hello = match Local::now().hour() {
            5..=11 => "Good morning",
            12..=17 => "Good afternoon",
            _ => "Good evening",
        };
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(bold(hello).size(28.0));
                ui.label(muted(format!("{} · listen, repeat aloud, understand", t.format("%A %-d %B"))).size(15.0));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let streak = self.saved.streak();
                if streak > 0 {
                    theme::pill(ui, &format!("🔥 {streak}-day streak"), WARN, WARN_SOFT);
                }
            });
        });
        ui.add_space(12.0);

        if let Some(c) = self.courses.get(ci).cloned() {
            self.hero(ui, ci, &c, today_log.lessons > 0, due);
        }
        ui.add_space(12.0);

        let minutes = (today_log.seconds / 60.0).floor() as u32;
        let goal = self.saved.daily_goal_min.max(1);
        let stats = [
            ("🔥", WARN, WARN_SOFT, days(self.saved.streak()), "study streak", None),
            (
                "⏱",
                ACCENT,
                ACCENT_SOFT,
                format!("{minutes} / {goal} min"),
                "studied today",
                Some(minutes as f32 / goal as f32),
            ),
            (
                "⟳",
                Color32::from_rgb(120, 96, 200),
                Color32::from_rgb(236, 232, 250),
                due.to_string(),
                "sentences to review",
                None,
            ),
            ("✔", GOOD, GOOD_SOFT, total_done.to_string(), "lessons completed", None),
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
                ui.label(caps(&format!("Continue · {}", lang_name(&c.lang, &base))).color(WAVE));
                ui.add_space(4.0);
                match self.next_lesson(ci) {
                    Some(li) => {
                        let l = &c.lessons[li];
                        ui.horizontal(|ui| {
                            theme::badge(ui, l.level.name(), l.level.color());
                            ui.label(medium(format!("Lesson {} of {}", l.number, c.lessons.len())).color(light));
                        });
                        ui.label(bold(&l.title).size(30.0).color(Color32::WHITE));
                        ui.label(RichText::new(l.title_in(&base)).size(16.0).color(light));
                        ui.add_space(6.0);
                        if studied_today {
                            ui.label(
                                RichText::new(
                                    "✔ You already did a lesson today. One a day is the rhythm of the method; you can still go on.",
                                )
                                .size(14.0)
                                .color(light),
                            );
                        }
                        if let Some(w) = self.wave2_for(ci, li) {
                            ui.label(
                                RichText::new(format!("➕ Second wave today: lesson {} (active review)", c.lessons[w].number))
                                    .size(14.0)
                                    .color(Color32::from_rgb(255, 214, 140)),
                            );
                        }
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let resumable = self
                                .saved
                                .progress(&c.lesson_key(l.number))
                                .is_some_and(|p| !p.steps_done.is_empty());
                            let start = egui::Button::new(
                                medium(if resumable {
                                    "▶  Continue the lesson"
                                } else {
                                    "▶  Start today's lesson"
                                })
                                .size(16.5)
                                .color(ACCENT_DARK),
                            )
                            .fill(Color32::WHITE)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(12)
                            .min_size(egui::vec2(220.0, 46.0));
                            if ui.add(start).clicked() {
                                self.start_session(ci, li);
                            }
                            if due > 0 {
                                let review = egui::Button::new(medium(format!("⟳  Review {due} sentences")).color(Color32::WHITE))
                                    .fill(Color32::from_white_alpha(20))
                                    .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(70)))
                                    .corner_radius(12)
                                    .min_size(egui::vec2(0.0, 46.0));
                                if ui.add(review).clicked() {
                                    self.go(Screen::Review);
                                }
                            }
                        });
                    }
                    None => {
                        ui.label(bold("🎉 Course complete").size(26.0).color(Color32::WHITE));
                        ui.label(
                            RichText::new(
                                "You finished every lesson of this course. Choose a new one in Courses, or add your own lessons.",
                            )
                            .color(light),
                        );
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

    /// Progress through the six levels of the course being studied.
    fn path_card(&mut self, ui: &mut egui::Ui, ci: usize, c: &Course, wide: bool) {
        let base = self.base();
        let current = self.next_lesson(ci).map(|li| c.lessons[li].level);
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(bold("Your path").size(18.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(muted(format!(
                        "{} · {} of {} lessons",
                        lang_name(&c.lang, &base),
                        self.done_count(ci),
                        c.lessons.len()
                    )));
                });
            });
            ui.add_space(4.0);
            let per_row = if wide { 6 } else { 3 };
            for row in Level::ALL.chunks(per_row) {
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
                ui.label(bold("Study calendar").size(18.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(muted(format!("last {weeks} weeks")).size(13.0));
                });
            });
            self.heatmap(ui, weeks);
        });
    }

    fn languages_card(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        let ci = self.active();
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold("Your languages").size(18.0));
            ui.add_space(2.0);
            let bar_w = (ui.available_width() - 250.0).clamp(40.0, 220.0);
            let mut choose = None;
            for (i, c) in self.courses.iter().enumerate() {
                let done = c.lessons.iter().filter(|l| self.saved.is_done(&c.lesson_key(l.number))).count();
                ui.horizontal(|ui| {
                    theme::dot(ui, lang_color(&c.lang), 5.0);
                    let name = format!("{}{}", lang_name(&c.lang, &base), if c.builtin { "" } else { " · mine" });
                    let label = ui
                        .allocate_ui_with_layout(egui::vec2(110.0, 22.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.set_min_width(110.0);
                            ui.add(egui::Label::new(medium(name)).truncate().sense(egui::Sense::click()))
                        })
                        .inner;
                    theme::bar(ui, done as f32 / c.lessons.len().max(1) as f32, bar_w, lang_color(&c.lang));
                    ui.label(muted(format!("{done}/{}", c.lessons.len())).size(13.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if i == ci {
                            theme::pill(ui, "studying", ACCENT, ACCENT_SOFT);
                        } else if ui.add(theme::ghost("Study")).clicked() || label.clicked() {
                            choose = Some(i);
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
        for (i, d) in ["M", "", "W", "", "F", "", "S"].iter().enumerate() {
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
            ui.label(muted(format!("{studied} days with study")).size(12.5));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                ui.label(muted("More").size(12.0));
                for c in HEAT.iter().rev() {
                    let (r, _) = ui.allocate_exact_size(egui::vec2(11.0, 11.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, 2, *c);
                }
                ui.label(muted("Less").size(12.0));
            });
        });
    }

    fn courses_screen(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        theme::page_title(
            ui,
            "Courses",
            "Each language runs from A1 (first words) to C2 (mastery). Pick one to study, link your own recordings, or start a course for your own material.",
        );

        let active = self.active();
        let mut choose = None;
        let mut link = None;
        let mut copy = None;
        let mut from_rec = None;
        let gap = 12.0;
        let width = ui.available_width();
        let per_row = ((width / 310.0).floor() as usize).clamp(1, 3);
        // Card content width: the column minus margins (18 each side) and stroke.
        let card_w = (width - (per_row as f32 - 1.0) * gap) / per_row as f32 - 38.0;
        for row in (0..self.courses.len()).collect::<Vec<_>>().chunks(per_row) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for &i in row {
                    let c = &self.courses[i];
                    let done = c.lessons.iter().filter(|l| self.saved.is_done(&c.lesson_key(l.number))).count();
                    let levels: Vec<&str> =
                        Level::ALL.iter().filter(|l| c.lessons.iter().any(|x| x.level == **l)).map(|l| l.name()).collect();
                    let color = lang_color(&c.lang);
                    let stroke = if i == active { egui::Stroke::new(2.0, ACCENT) } else { egui::Stroke::new(1.0, LINE) };
                    theme::card().stroke(stroke).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(card_w);
                            ui.set_min_height(150.0);
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(egui::vec2(42.0, 42.0), egui::Sense::hover());
                                ui.painter().circle_filled(r.center(), 21.0, color);
                                ui.painter().text(
                                    r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    c.lang.to_uppercase(),
                                    egui::FontId::new(14.0, theme::bold_family()),
                                    Color32::WHITE,
                                );
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    let native = crate::model::lang(&c.lang).map(|l| l.native).unwrap_or(&c.lang);
                                    ui.label(bold(native).size(18.0));
                                    ui.label(muted(if c.builtin {
                                        lang_name(&c.lang, &base)
                                    } else {
                                        format!("{} · your material", lang_name(&c.lang, &base))
                                    }));
                                });
                                if i == active {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                        theme::pill(ui, "studying", ACCENT, ACCENT_SOFT);
                                    });
                                }
                            });
                            ui.add_space(2.0);
                            ui.add(egui::Label::new(muted(c.name_in(&base)).size(13.0)).truncate());
                            theme::bar(ui, done as f32 / c.lessons.len().max(1) as f32, card_w, color);
                            ui.horizontal(|ui| {
                                ui.label(medium(format!("{done} of {}", plural(c.lessons.len(), "lesson"))).size(13.5));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let span = match (levels.first(), levels.last()) {
                                        (Some(a), Some(b)) if a == b => a.to_string(),
                                        (Some(a), Some(b)) => format!("{a}–{b}"),
                                        _ => "no lessons yet".into(),
                                    };
                                    ui.label(muted(span).size(13.0));
                                });
                            });
                            let rec = self.saved.audio_dirs.get(&c.id);
                            ui.add(
                                egui::Label::new(
                                    muted(match rec {
                                        Some(d) => format!("🎧 {}", d.rsplit(['/', '\\']).next().unwrap_or(d)),
                                        None => "🎧 read by the voice".into(),
                                    })
                                    .size(12.5),
                                )
                                .truncate(),
                            );
                            ui.horizontal_wrapped(|ui| {
                                if i != active && ui.add(theme::primary("Study")).clicked() {
                                    choose = Some(i);
                                }
                                if ui
                                    .add(theme::secondary("Audio…"))
                                    .on_hover_text("Choose the folder with this course's audio files (e.g. \"Lesson 12.mp3\")")
                                    .clicked()
                                {
                                    link = Some(i);
                                }
                                if c.builtin
                                    && ui
                                        .add(theme::ghost("Copy"))
                                        .on_hover_text("Copy this course to your own courses to expand or change it")
                                        .clicked()
                                {
                                    copy = Some(i);
                                }
                                if !c.builtin
                                    && rec.is_some()
                                    && ui
                                        .add(theme::ghost("＋ A lesson per recording"))
                                        .on_hover_text("Creates lessons for recordings that have no text yet: study them with your book, add the text later")
                                        .clicked()
                                {
                                    from_rec = Some(i);
                                }
                            });
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
                ui.label(medium("New course for your own material:"));
                egui::ComboBox::from_id_salt("newlang")
                    .selected_text(lang_name(&self.new_course_lang, &base))
                    .show_ui(ui, |ui| {
                        for l in LANGS {
                            ui.selectable_value(&mut self.new_course_lang, l.code.to_string(), lang_name(l.code, &base));
                        }
                    });
                if ui.add(theme::primary("＋ Create")).clicked() {
                    let lang = self.new_course_lang.clone();
                    self.create_course(&lang);
                }
            });
            ui.label(muted("Then add lessons in “My lessons”: type or paste the dialogue, notes and exercises.").size(13.5));
        });

        ui.add_space(10.0);
        self.lesson_list(ui, active);
    }

    fn lesson_list(&mut self, ui: &mut egui::Ui, ci: usize) {
        let base = self.base();
        let Some(c) = self.courses.get(ci).cloned() else { return };
        theme::section(ui, &format!("Lessons · {}", c.name_in(&base)));
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
                                            let label = if p.completed.is_some() { "Again" } else { "Study" };
                                            let btn = if is_next { theme::primary(label) } else { theme::ghost(label) };
                                            if ui.add(btn).clicked() {
                                                start = Some(li);
                                            }
                                            if !c.builtin && ui.add(theme::ghost("✏")).on_hover_text("Edit this lesson").clicked() {
                                                edit = Some(li);
                                            }
                                            if p.wave2.is_some() {
                                                theme::pill(ui, "2nd wave ✔", GOOD, GOOD_SOFT);
                                            } else if wave.contains(&li) {
                                                theme::pill(ui, "2nd wave due", WARN, WARN_SOFT);
                                            }
                                            if p.score.1 > 0 {
                                                ui.label(muted(format!("ex. {}/{}", p.score.0, p.score.1)).size(12.5));
                                            }
                                        });
                                    });
                                });
                        }
                    });
                });
        }
        if c.lessons.is_empty() {
            ui.label(muted("No lessons yet. Add them in “My lessons”."));
        }
        if let Some(li) = start {
            self.start_session(ci, li);
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
                self.say("Course created. Add your first lesson.");
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
            Ok(_) => self.say(format!(
                "Added {added} lessons from the recordings. Follow them in your book; add the text any time in My lessons."
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
        for n in c.name_tr.values_mut() {
            n.push_str(" (mine)");
        }
        c.name.push_str(" (mine)");
        match store::save_course(&c) {
            Ok(path) => {
                c.path = Some(path);
                self.courses.push(c);
                self.say("Copied to your courses: you can now expand and edit it in “My lessons”.");
            }
            Err(e) => self.say(e),
        }
    }

    fn settings_screen(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        theme::page_title(
            ui,
            "Settings & voices",
            "How lessons are shown and read aloud. Changes are saved by themselves.",
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
                add(ui);
            });
            ui.add_space(6.0);
        };

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold("Your language").size(18.0));
            ui.label(muted("Translations and notes are shown in this language. The built-in lessons are written in Italian, English and Spanish; your own lessons can use any.").size(14.0));
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
            ui.label(bold("Method").size(18.0));
            ui.add_space(4.0);
            row(ui, "Daily goal", "minutes of study a day", &mut |ui| {
                changed |= ui
                    .add(egui::Slider::new(&mut self.saved.daily_goal_min, 5..=90).suffix(" min"))
                    .changed();
            });
            row(ui, "Second wave starts", "lessons later (classic: 49)", &mut |ui| {
                changed |= ui.add(egui::DragValue::new(&mut self.saved.wave2_gap).range(1..=120)).changed();
            });
            row(ui, "Daily sentence review", "", &mut |ui| {
                changed |= ui
                    .checkbox(
                        &mut self.saved.cards_after_lesson,
                        "Add each finished dialogue's sentences to spaced review",
                    )
                    .changed();
            });
            row(ui, "Pronunciation hints", "", &mut |ui| {
                changed |= ui
                    .checkbox(&mut self.saved.show_pron, "Show pinyin / vowel lengths under sentences")
                    .changed();
            });
        });
        ui.add_space(4.0);

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold("Listening").size(18.0));
            ui.add_space(4.0);
            row(ui, "Speed", "", &mut |ui| {
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.saved.speed, 0.6..=1.4)
                            .step_by(0.05)
                            .fixed_decimals(2)
                            .suffix("×"),
                    )
                    .changed();
            });
            row(ui, "Pause to repeat", "how long the voice waits after each sentence", &mut |ui| {
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.saved.pause_factor, 0.5..=3.0)
                            .step_by(0.1)
                            .fixed_decimals(1)
                            .suffix("× the sentence"),
                    )
                    .changed();
            });
            row(ui, "Recordings vs voice", "", &mut |ui| {
                changed |= ui
                    .checkbox(
                        &mut self.saved.prefer_voice,
                        "Use the voice even when a recording is linked (sentence by sentence)",
                    )
                    .changed();
            });
            row(ui, "Audio player", "", &mut |ui| {
                if Player::available() {
                    theme::pill(ui, "mpv found ✔", GOOD, GOOD_SOFT);
                } else {
                    ui.label(RichText::new("No player found: install mpv to hear audio").color(BAD));
                }
            });
        });
        ui.add_space(4.0);

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold("Voices").size(18.0));
            ui.label(muted("Free and offline after the download. A natural-sounding voice reads every sentence aloud when you have no recordings. Each voice is about 60 MB; the first one also fetches the voice engine (25 MB). Latin is read by the Italian voice with Latin pronunciation rules.").size(14.0));
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
                        theme::pill(ui, "Ready ✔", GOOD, GOOD_SOFT);
                    } else if status.starts_with("Downloading") || status.starts_with("Finishing") {
                        let mb = self.voices.partial_mb(l.code).map(|m| format!(" {m:.0} MB")).unwrap_or_default();
                        ui.spinner();
                        ui.label(format!("{status}{mb}"));
                        ui.ctx().request_repaint_after(Duration::from_millis(400));
                    } else {
                        if ui.add(theme::secondary("⬇ Download")).clicked() {
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
            ui.label(bold("Your files").size(18.0));
            let dir = store::data_dir();
            ui.label(muted(format!("Progress, your courses and voices are kept in {}", dir.display())).size(14.0));
            if ui.add(theme::secondary("📂 Open the folder")).clicked() {
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
    plural(n as usize, "day")
}

/// "1 lesson", "5 lessons".
fn plural(n: usize, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
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
                        Screen::Study => {}
                    });
                    ui.add_space(24.0);
                });
            }
        });

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
