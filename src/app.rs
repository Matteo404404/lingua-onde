use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{Datelike, Duration as Days};
use eframe::egui::{self, Color32, RichText};

use crate::audio::{Player, Voices, scan_audio_dir};
use crate::editor::EditorState;
use crate::model::{Course, LANGS, Level, lang_color, lang_name};
use crate::session::{ReviewState, Seq, Session};
use crate::store::{self, Saved, day_key, today};
use crate::theme::{self, ACCENT, GOOD, INK, LINE, MUTED, NAV, WARN, muted};

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

    fn nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.label(RichText::new("〰").size(26.0).color(Color32::from_rgb(120, 200, 220)));
            ui.vertical(|ui| {
                ui.label(RichText::new("Lingua Onde").size(21.0).strong().color(Color32::WHITE));
                ui.label(
                    RichText::new("learn by assimilation")
                        .size(12.0)
                        .color(Color32::from_rgb(160, 175, 200)),
                );
            });
        });
        ui.add_space(18.0);

        let base = self.base();
        let ci = self.active();
        if let Some(c) = self.courses.get(ci) {
            egui::Frame::new()
                .fill(Color32::from_rgb(40, 55, 82))
                .corner_radius(10)
                .inner_margin(10)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new("STUDYING").size(11.0).color(Color32::from_rgb(150, 165, 190)));
                    ui.horizontal(|ui| {
                        theme::dot(ui, lang_color(&c.lang), 6.0);
                        ui.label(RichText::new(lang_name(&c.lang, &base)).strong().color(Color32::WHITE));
                    });
                    let done = self.done_count(ci);
                    ui.label(
                        RichText::new(format!("{done} / {} lessons", c.lessons.len()))
                            .size(12.5)
                            .color(Color32::from_rgb(190, 200, 215)),
                    );
                    theme::bar(
                        ui,
                        done as f32 / c.lessons.len().max(1) as f32,
                        ui.available_width(),
                        Color32::from_rgb(120, 200, 220),
                    );
                    let today = self.saved.days.get(&day_key(today())).map(|d| d.seconds).unwrap_or(0.0);
                    ui.label(
                        RichText::new(format!(
                            "🔥 {} day streak · {} min today",
                            self.saved.streak(),
                            (today / 60.0).floor()
                        ))
                        .size(12.5)
                        .color(Color32::from_rgb(190, 200, 215)),
                    );
                });
        }
        ui.add_space(14.0);

        let due = self.saved.cards_due() + self.wave2_due(ci).len();
        let items = [
            (Screen::Home, "🏠  Progress", String::new()),
            (Screen::Study, "▶  Today's lesson", String::new()),
            (Screen::Review, "⟳  Review", if due > 0 { due.to_string() } else { String::new() }),
            (Screen::Courses, "📚  Courses", String::new()),
            (Screen::Editor, "✏  My lessons", String::new()),
            (Screen::Settings, "⚙  Settings & voices", String::new()),
        ];
        for (screen, label, count) in items {
            let selected = self.screen == screen;
            let text = RichText::new(label).size(15.5).color(if selected {
                Color32::WHITE
            } else {
                Color32::from_rgb(200, 210, 225)
            });
            let btn = egui::Button::new(text)
                .fill(if selected { ACCENT } else { Color32::TRANSPARENT })
                .stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(ui.available_width(), 38.0));
            let resp = ui.add(btn);
            if !count.is_empty() {
                let r = resp.rect;
                let c = egui::pos2(r.right() - 20.0, r.center().y);
                ui.painter().circle_filled(c, 10.0, WARN);
                ui.painter().text(
                    c,
                    egui::Align2::CENTER_CENTER,
                    &count,
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }
            if resp.clicked() {
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
        let base = self.base();
        let ci = self.active();
        let t = today();
        let today_log = self.saved.days.get(&day_key(t)).cloned().unwrap_or_default();
        let total_done: usize = (0..self.courses.len()).map(|i| self.done_count(i)).sum();
        let due = self.saved.cards_due();

        ui.heading("Your progress");
        ui.label(muted(
            "A little every day: listen, repeat aloud, understand. The rest comes by itself.",
        ));
        ui.add_space(8.0);

        ui.columns(4, |cols| {
            let stat = |ui: &mut egui::Ui, icon: &str, big: String, small: &str, frac: Option<f32>| {
                // Columns are justified, which would stretch wrapped text.
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    theme::card().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(icon).size(20.0));
                        ui.label(RichText::new(big).size(26.0).strong());
                        ui.label(muted(small));
                        if let Some(f) = frac {
                            theme::bar(ui, f, ui.available_width(), ACCENT);
                        }
                    })
                });
            };
            let minutes = (today_log.seconds / 60.0).floor() as u32;
            let goal = self.saved.daily_goal_min.max(1);
            stat(&mut cols[0], "🔥", format!("{} days", self.saved.streak()), "study streak", None);
            stat(
                &mut cols[1],
                "⏱",
                format!("{minutes} / {goal} min"),
                "studied today",
                Some(minutes as f32 / goal as f32),
            );
            stat(&mut cols[2], "⟳", due.to_string(), "sentences to review today", None);
            stat(&mut cols[3], "✔", total_done.to_string(), "lessons completed", None);
        });
        ui.add_space(6.0);

        // Continue card.
        if let Some(c) = self.courses.get(ci).cloned() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    theme::dot(ui, lang_color(&c.lang), 7.0);
                    ui.label(RichText::new(c.name_in(&base)).size(18.0).strong());
                });
                match self.next_lesson(ci) {
                    Some(li) => {
                        let l = &c.lessons[li];
                        ui.horizontal(|ui| {
                            theme::badge(ui, l.level.name(), l.level.color());
                            ui.label(RichText::new(format!("Lesson {} · {}", l.number, l.title)).size(17.0));
                            ui.label(muted(format!("— {}", l.title_in(&base))));
                        });
                        if today_log.lessons > 0 {
                            ui.label(RichText::new("✔ You already did a lesson today. One a day is the rhythm of the method: tomorrow's will feel easier. You can still go on if you like.").color(GOOD));
                        }
                        if let Some(w) = self.wave2_for(ci, li) {
                            ui.label(RichText::new(format!("➕ Second wave today: lesson {} (active review)", c.lessons[w].number)).color(WARN));
                        }
                        ui.horizontal(|ui| {
                            let resumable = self.saved.progress(&c.lesson_key(l.number)).is_some_and(|p| !p.steps_done.is_empty());
                            if ui.add(theme::big_primary(if resumable { "▶  Continue the lesson" } else { "▶  Start today's lesson" })).clicked() {
                                self.start_session(ci, li);
                            }
                            if due > 0 && ui.add(egui::Button::new(format!("⟳  Review {due} sentences")).min_size(egui::vec2(0.0, 44.0))).clicked() {
                                self.go(Screen::Review);
                            }
                        });
                    }
                    None => {
                        ui.label("🎉 You completed every lesson of this course. Choose a new one in Courses, or add your own lessons.");
                    }
                }
            });

            ui.add_space(6.0);
            ui.label(RichText::new("Levels in this course").strong());
            ui.columns(6, |cols| {
                for (i, lvl) in Level::ALL.into_iter().enumerate() {
                    let keys: Vec<String> = c
                        .lessons
                        .iter()
                        .filter(|l| l.level == lvl)
                        .map(|l| c.lesson_key(l.number))
                        .collect();
                    let done = keys.iter().filter(|k| self.saved.is_done(k)).count();
                    let col = &mut cols[i];
                    col.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        theme::card().inner_margin(10).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                theme::badge(ui, lvl.name(), lvl.color());
                                if !keys.is_empty() && done == keys.len() {
                                    ui.label(RichText::new("✔").color(GOOD));
                                }
                            });
                            ui.label(muted(lvl.describe()).size(12.5));
                            ui.label(RichText::new(format!("{done} / {}", keys.len())).strong());
                            if c.builtin {
                                let (first, last) = lvl.plan();
                                ui.label(muted(format!("lessons {first}–{last}")).size(11.5));
                            }
                            theme::bar(
                                ui,
                                if keys.is_empty() { 0.0 } else { done as f32 / keys.len() as f32 },
                                ui.available_width(),
                                lvl.color(),
                            );
                        })
                    });
                }
            });
        }

        ui.add_space(6.0);
        let calendar = |app: &mut App, ui: &mut egui::Ui| {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Study calendar · last 18 weeks").strong());
                app.heatmap(ui);
            });
        };
        let languages = |app: &mut App, ui: &mut egui::Ui| {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Your languages").strong());
                let bar_w = (ui.available_width() - 260.0).clamp(50.0, 200.0);
                let mut choose = None;
                for (i, c) in app.courses.iter().enumerate() {
                    let done = c.lessons.iter().filter(|l| app.saved.is_done(&c.lesson_key(l.number))).count();
                    ui.horizontal(|ui| {
                        theme::dot(ui, lang_color(&c.lang), 5.0);
                        let name = format!("{}{}", lang_name(&c.lang, &base), if c.builtin { "" } else { " · mine" });
                        let label = ui
                            .allocate_ui_with_layout(egui::vec2(110.0, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.set_min_width(110.0);
                                ui.add(egui::Label::new(name).truncate().sense(egui::Sense::click()))
                            })
                            .inner;
                        theme::bar(ui, done as f32 / c.lessons.len().max(1) as f32, bar_w, lang_color(&c.lang));
                        ui.label(muted(format!("{done}/{}", c.lessons.len())));
                        if i == ci {
                            ui.label(RichText::new("studying").size(12.0).color(ACCENT));
                        } else if label.clicked() || ui.small_button("study").clicked() {
                            choose = Some(i);
                        }
                    });
                }
                if let Some(i) = choose {
                    app.saved.active_course = app.courses[i].id.clone();
                    app.mark_dirty();
                }
            });
        };
        if ui.available_width() >= 980.0 {
            ui.columns(2, |cols| {
                calendar(self, &mut cols[0]);
                languages(self, &mut cols[1]);
            });
        } else {
            calendar(self, ui);
            languages(self, ui);
        }
    }

    fn heatmap(&self, ui: &mut egui::Ui) {
        let weeks = 18;
        let gap = 3.0;
        let cell = ((ui.available_width() - 20.0) / weeks as f32 - gap).clamp(8.0, 16.0);
        let t = today();
        // The last column is the current week, Monday first.
        let start = t - Days::days(t.weekday().num_days_from_monday() as i64 + ((weeks - 1) * 7) as i64);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(weeks as f32 * (cell + gap) + 18.0, 7.0 * (cell + gap) + 4.0),
            egui::Sense::hover(),
        );
        let p = ui.painter();
        for (i, d) in ["M", "", "W", "", "F", "", "S"].iter().enumerate() {
            p.text(
                egui::pos2(rect.left() + 6.0, rect.top() + i as f32 * (cell + gap) + cell / 2.0),
                egui::Align2::CENTER_CENTER,
                *d,
                egui::FontId::proportional(10.0),
                MUTED,
            );
        }
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
                let color = match mins {
                    m if m <= 0.0 => Color32::from_rgb(234, 230, 221),
                    m if m < 5.0 => Color32::from_rgb(190, 222, 230),
                    m if m < 15.0 => Color32::from_rgb(120, 186, 205),
                    m if m < 30.0 => Color32::from_rgb(60, 145, 172),
                    _ => Color32::from_rgb(25, 95, 122),
                };
                let min = egui::pos2(rect.left() + 18.0 + w as f32 * (cell + gap), rect.top() + d as f32 * (cell + gap));
                let r = egui::Rect::from_min_size(min, egui::vec2(cell, cell));
                p.rect_filled(r, 3, color);
                if day == t {
                    p.rect_stroke(r, 3, egui::Stroke::new(1.5, INK), egui::StrokeKind::Outside);
                }
            }
        }
        ui.label(muted("Darker = more minutes that day.").size(12.0));
    }

    fn courses_screen(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        ui.heading("Courses");
        ui.label(muted("Each language goes from A1 (first words) to C2 (mastery). Pick the one to study, link your own recordings, or start a course for your own material."));
        ui.add_space(6.0);

        let active = self.active();
        let mut choose = None;
        let mut link = None;
        let mut copy = None;
        let mut from_rec = None;
        let width = ui.available_width();
        let per_row = ((width / 300.0).floor() as usize).clamp(1, 4);
        let card_w = (width - (per_row as f32 - 1.0) * 10.0) / per_row as f32 - 34.0;
        for row in (0..self.courses.len()).collect::<Vec<_>>().chunks(per_row) {
            ui.horizontal_top(|ui| {
                for &i in row {
                    let c = &self.courses[i];
                    let done = c.lessons.iter().filter(|l| self.saved.is_done(&c.lesson_key(l.number))).count();
                    let levels: Vec<&str> = Level::ALL.iter().filter(|l| c.lessons.iter().any(|x| x.level == **l)).map(|l| l.name()).collect();
                    let stroke = if i == active { egui::Stroke::new(2.0, ACCENT) } else { egui::Stroke::new(1.0, LINE) };
                    theme::card().stroke(stroke).show(ui, |ui| ui.vertical(|ui| {
                        ui.set_width(card_w);
                        ui.horizontal(|ui| {
                            theme::dot(ui, lang_color(&c.lang), 8.0);
                            ui.label(RichText::new(crate::model::lang(&c.lang).map(|l| l.native).unwrap_or(&c.lang)).size(19.0).strong());
                            ui.label(muted(lang_name(&c.lang, &base)));
                        });
                        ui.add(egui::Label::new(RichText::new(c.name_in(&base)).strong()).truncate());
                        ui.label(muted(format!(
                            "{} lessons · {} · {}",
                            c.lessons.len(),
                            if levels.is_empty() { "no lessons yet".to_string() } else { format!("{}–{}", levels[0], levels[levels.len() - 1]) },
                            if c.builtin { "built-in" } else { "your material" }
                        )));
                        if c.builtin {
                            ui.label(
                                muted(format!(
                                    "written so far: {} of the planned {} (100 to B2, 100 to C1, then C2)",
                                    c.lessons.len(),
                                    Level::C2.plan().1
                                ))
                                .size(12.5),
                            );
                        }
                        theme::bar(ui, done as f32 / c.lessons.len().max(1) as f32, card_w, lang_color(&c.lang));
                        let rec = self.saved.audio_dirs.get(&c.id);
                        ui.label(muted(match rec {
                            Some(d) => format!("🎧 recordings: {}", d.rsplit(['/', '\\']).next().unwrap_or(d)),
                            None => "🎧 no recordings linked (the voice reads the text)".into(),
                        }).size(12.5));
                        ui.horizontal_wrapped(|ui| {
                            if i == active {
                                ui.label(RichText::new("✔ studying").color(ACCENT).strong());
                            } else if ui.add(theme::primary("Study this")).clicked() {
                                choose = Some(i);
                            }
                            if ui.button("🎧 Recordings…").on_hover_text("Choose the folder with this course's audio files (e.g. \"Lesson 12.mp3\")").clicked() {
                                link = Some(i);
                            }
                            if c.builtin && ui.button("Copy").on_hover_text("Copy this course to your own courses to expand or change it").clicked() {
                                copy = Some(i);
                            }
                        });
                        if !c.builtin && rec.is_some() && ui.button("＋ A lesson for each recording").on_hover_text("Creates lessons for recordings that have no text yet: study them with your book, add the text later").clicked() {
                            from_rec = Some(i);
                        }
                    }));
                }
            });
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

        ui.add_space(6.0);
        theme::tinted(theme::ACCENT_SOFT).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("New course for your own material:").strong());
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
                ui.label(muted(
                    "Then add lessons in “My lessons”: type or paste the dialogue, notes and exercises.",
                ));
            });
        });

        ui.add_space(8.0);
        self.lesson_list(ui, active);
    }

    fn lesson_list(&mut self, ui: &mut egui::Ui, ci: usize) {
        let base = self.base();
        let Some(c) = self.courses.get(ci).cloned() else { return };
        ui.label(RichText::new(format!("Lessons — {}", c.name_in(&base))).size(18.0).strong());
        let next = self.next_lesson(ci);
        let wave = self.wave2_due(ci);
        let mut start = None;
        let mut edit = None;
        for lvl in Level::ALL {
            let lessons: Vec<(usize, &crate::model::Lesson)> = c.lessons.iter().enumerate().filter(|(_, l)| l.level == lvl).collect();
            if lessons.is_empty() {
                continue;
            }
            ui.horizontal(|ui| {
                theme::badge(ui, lvl.name(), lvl.color());
                ui.label(muted(lvl.describe()));
            });
            for (li, l) in lessons {
                let key = c.lesson_key(l.number);
                let p = self.saved.progress(&key).cloned().unwrap_or_default();
                theme::card().inner_margin(egui::Margin::symmetric(14, 8)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (icon, color) = if p.completed.is_some() {
                            ("✔", GOOD)
                        } else if Some(li) == next {
                            ("▶", ACCENT)
                        } else {
                            ("○", MUTED)
                        };
                        ui.label(RichText::new(icon).color(color).size(17.0));
                        ui.label(RichText::new(format!("{:>3}", l.number)).strong());
                        ui.label(RichText::new(&l.title).strong());
                        ui.label(muted(l.title_in(&base)));
                        if p.wave2.is_some() {
                            ui.label(RichText::new("2nd wave ✔").size(12.0).color(GOOD));
                        } else if wave.contains(&li) {
                            ui.label(RichText::new("2nd wave due").size(12.0).color(WARN));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let label = if p.completed.is_some() { "Study again" } else { "Study" };
                            if ui.button(label).clicked() {
                                start = Some(li);
                            }
                            if !c.builtin && ui.button("✏").on_hover_text("Edit this lesson").clicked() {
                                edit = Some(li);
                            }
                            if let Some(d) = &p.completed {
                                ui.label(muted(format!("done {d} · ×{}", p.times)).size(12.0));
                            }
                            if p.score.1 > 0 {
                                ui.label(muted(format!("ex. {}/{}", p.score.0, p.score.1)).size(12.0));
                            }
                        });
                    });
                });
            }
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
        ui.heading("Settings & voices");
        ui.add_space(4.0);
        let mut changed = false;

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Your language").strong());
            ui.label(muted("Translations and notes are shown in this language. The built-in lessons are written in Italian, English and Spanish; your own lessons can use any."));
            egui::ComboBox::from_id_salt("base").selected_text(lang_name(&self.saved.base, &base)).show_ui(ui, |ui| {
                for code in ["it", "en", "es", "fr", "de", "pt"] {
                    changed |= ui.selectable_value(&mut self.saved.base, code.to_string(), crate::model::lang(code).map(|l| l.native).unwrap_or(code)).changed();
                }
            });
        });

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Method").strong());
            egui::Grid::new("method").num_columns(2).spacing([16.0, 10.0]).show(ui, |ui| {
                ui.label("Daily goal");
                changed |= ui
                    .add(egui::Slider::new(&mut self.saved.daily_goal_min, 5..=90).suffix(" min"))
                    .changed();
                ui.end_row();
                ui.label("Second wave starts");
                ui.horizontal(|ui| {
                    changed |= ui.add(egui::DragValue::new(&mut self.saved.wave2_gap).range(1..=120)).changed();
                    ui.label(muted("lessons later (classic: 49)"));
                });
                ui.end_row();
                ui.label("Daily sentence review");
                changed |= ui
                    .checkbox(
                        &mut self.saved.cards_after_lesson,
                        "Add each finished dialogue's sentences to spaced review",
                    )
                    .changed();
                ui.end_row();
                ui.label("Pronunciation hints");
                changed |= ui
                    .checkbox(&mut self.saved.show_pron, "Show pinyin / vowel lengths under sentences")
                    .changed();
                ui.end_row();
            });
        });

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Listening").strong());
            egui::Grid::new("audio").num_columns(2).spacing([16.0, 10.0]).show(ui, |ui| {
                ui.label("Speed");
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.saved.speed, 0.6..=1.4)
                            .step_by(0.05)
                            .fixed_decimals(2)
                            .suffix("×"),
                    )
                    .changed();
                ui.end_row();
                ui.label("Pause to repeat");
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.saved.pause_factor, 0.5..=3.0)
                            .step_by(0.1)
                            .fixed_decimals(1)
                            .suffix("× the sentence"),
                    )
                    .on_hover_text("How long the voice waits after each sentence so you can repeat it aloud")
                    .changed();
                ui.end_row();
                ui.label("Recordings vs voice");
                changed |= ui
                    .checkbox(
                        &mut self.saved.prefer_voice,
                        "Use the voice even when a recording is linked (sentence by sentence)",
                    )
                    .changed();
                ui.end_row();
                ui.label("Audio player");
                ui.label(if Player::available() {
                    muted("mpv found ✔")
                } else {
                    RichText::new("No player found: install mpv to hear audio").color(theme::BAD)
                });
                ui.end_row();
            });
        });

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Voices (free, offline after download)").strong());
            ui.label(muted("A natural-sounding voice reads every sentence aloud when you have no recordings. Each voice is about 60 MB; the first one also fetches the voice engine (25 MB). Latin is read by the Italian voice with Latin pronunciation rules."));
            egui::Grid::new("voices").num_columns(3).spacing([16.0, 8.0]).striped(true).show(ui, |ui| {
                for l in LANGS {
                    ui.horizontal(|ui| {
                        theme::dot(ui, l.color, 5.0);
                        ui.label(lang_name(l.code, &base));
                    });
                    let status = self.voices.status_of(l.code);
                    if self.voices.ready(l.code) {
                        ui.label(RichText::new("Ready ✔").color(GOOD));
                        ui.label("");
                    } else if status.starts_with("Downloading") || status.starts_with("Finishing") {
                        let mb = self.voices.partial_mb(l.code).map(|m| format!(" {m:.0} MB")).unwrap_or_default();
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(format!("{status}{mb}"));
                        });
                        ui.ctx().request_repaint_after(Duration::from_millis(400));
                        ui.label("");
                    } else {
                        ui.label(if status.is_empty() { muted("not downloaded") } else { RichText::new(&status).color(theme::BAD) });
                        if ui.button("⬇ Download").clicked() {
                            self.voices.download(l.code, ui.ctx().clone());
                        }
                    }
                    ui.end_row();
                }
            });
        });

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Your files").strong());
            let dir = store::data_dir();
            ui.label(muted(format!("Progress, your courses and voices are kept in {}", dir.display())));
            if ui.button("📂 Open the folder").clicked() {
                let _ = std::fs::create_dir_all(&dir);
                open_path(&dir);
            }
            for p in &self.problems {
                ui.label(RichText::new(format!("⚠ {p}")).color(theme::BAD));
            }
        });

        if changed {
            self.mark_dirty();
        }
    }
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

        egui::Panel::left("nav")
            .exact_size(230.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(NAV).inner_margin(egui::Margin::symmetric(12, 6)))
            .show(ui, |ui| self.nav(ui));

        if let Some((msg, at)) = self.toast.clone() {
            if at.elapsed() < Duration::from_secs(5) {
                egui::Panel::bottom("toast")
                    .frame(egui::Frame::new().fill(INK).inner_margin(10))
                    .show(ui, |ui| {
                        ui.label(RichText::new(msg).color(Color32::WHITE));
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
                    ui.set_max_width(ui.available_width().min(1100.0));
                    match screen {
                        Screen::Home => self.home(ui),
                        Screen::Courses => self.courses_screen(ui),
                        Screen::Review => self.review_screen(ui),
                        Screen::Editor => self.editor_screen(ui),
                        Screen::Settings => self.settings_screen(ui),
                        Screen::Study => {}
                    }
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
