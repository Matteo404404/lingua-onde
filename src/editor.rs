//! "My lessons": add and edit lessons in the person's own courses.

use eframe::egui::{self, RichText};

use crate::app::{App, Screen, open_path};
use crate::model::{Item, Lesson, Level, lang_name};
use crate::store;
use crate::theme::{self, ACCENT_SOFT, BAD, GOOD, muted};

#[derive(Default)]
pub struct EditorState {
    pub course: Option<String>,
    /// Index of the lesson being edited; None while adding a new one.
    pub lesson: Option<usize>,
    pub editing: bool,
    pub draft: Lesson,
    pub paste_target: String,
    pub paste_tr: String,
    pub msg: String,
    pub confirm_delete: bool,
}

impl App {
    /// Open the editor on a course (and a lesson in it); by default the
    /// person's first course in the language they are studying.
    pub fn open_editor(&mut self, target: Option<(String, Option<usize>)>) {
        let (course, lesson) = match target {
            Some(t) => (Some(t.0), t.1),
            None => {
                let lang = self.courses.get(self.active()).map(|c| c.lang.clone()).unwrap_or_default();
                let mine = |same: bool| {
                    self.courses
                        .iter()
                        .find(|c| !c.builtin && (!same || c.lang == lang))
                        .map(|c| c.id.clone())
                };
                (mine(true).or_else(|| mine(false)), None)
            }
        };
        self.editor = EditorState {
            course,
            ..Default::default()
        };
        if let Some(li) = lesson {
            self.edit_lesson(li);
        }
        self.screen = Screen::Editor;
    }

    pub fn edit_lesson(&mut self, li: usize) {
        let Some(ci) = self.editor.course.as_deref().and_then(|id| self.course_index(id)) else {
            return;
        };
        if let Some(l) = self.courses[ci].lessons.get(li) {
            self.editor.draft = l.clone();
            self.editor.lesson = Some(li);
            self.editor.editing = true;
            self.editor.msg.clear();
            self.editor.confirm_delete = false;
        }
    }

    fn new_lesson(&mut self, ci: usize) {
        let c = &self.courses[ci];
        let number = c.lessons.iter().map(|l| l.number).max().unwrap_or(0) + 1;
        let level = c.lessons.last().map(|l| l.level).unwrap_or(Level::A1);
        self.editor.draft = Lesson {
            number,
            level,
            dialogue: vec![Item::default()],
            ..Default::default()
        };
        self.editor.lesson = None;
        self.editor.editing = true;
        self.editor.msg.clear();
        self.editor.paste_target.clear();
        self.editor.paste_tr.clear();
    }

    pub fn editor_screen(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        ui.heading("My lessons");
        ui.label(muted("Expand your courses with your own material: type or paste each dialogue with its translation, notes and exercises. Lessons are saved as plain text files, so you can also edit them in any text editor."));
        ui.add_space(4.0);

        let mine: Vec<(String, String)> = self
            .courses
            .iter()
            .filter(|c| !c.builtin)
            .map(|c| (c.id.clone(), format!("{} — {}", lang_name(&c.lang, &base), c.name_in(&base))))
            .collect();
        if mine.is_empty() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("You have no course of your own yet.").size(17.0));
                ui.label(muted("Create one for any language in Courses (“New course for your own material”), or copy a built-in course there to expand it."));
                if ui.add(theme::primary("Go to Courses")).clicked() {
                    self.go(Screen::Courses);
                }
            });
            return;
        }
        if self.editor.course.as_ref().is_none_or(|id| !mine.iter().any(|(m, _)| m == id)) {
            self.editor.course = Some(mine[0].0.clone());
        }
        let id = self.editor.course.clone().unwrap_or_default();
        let Some(ci) = self.course_index(&id) else { return };

        ui.horizontal_wrapped(|ui| {
            ui.label("Course:");
            let current = mine.iter().find(|(m, _)| *m == id).map(|(_, n)| n.clone()).unwrap_or_default();
            let mut pick = None;
            egui::ComboBox::from_id_salt("edcourse")
                .selected_text(current)
                .width(320.0)
                .show_ui(ui, |ui| {
                    for (m, n) in &mine {
                        if ui.selectable_label(*m == id, n).clicked() {
                            pick = Some(m.clone());
                        }
                    }
                });
            if let Some(m) = pick {
                self.open_editor(Some((m, None)));
            }
            if ui.button("📂 Open the lesson files").clicked() {
                let dir = store::courses_dir();
                let _ = std::fs::create_dir_all(&dir);
                open_path(&dir);
            }
            if ui.button("Study this course").clicked() {
                self.saved.active_course = id.clone();
                self.mark_dirty();
                self.go(Screen::Courses);
            }
        });

        if !self.editor.msg.is_empty() {
            let ok = !self.editor.msg.starts_with('⚠');
            ui.label(RichText::new(&self.editor.msg).color(if ok { GOOD } else { BAD }));
        }

        if !self.editor.editing {
            self.lesson_index(ui, ci);
        } else {
            self.lesson_form(ui, ci);
        }
    }

    fn lesson_index(&mut self, ui: &mut egui::Ui, ci: usize) {
        let base = self.base();
        let c = self.courses[ci].clone();
        if ui.add(theme::big_primary("＋ New lesson")).clicked() {
            self.new_lesson(ci);
            return;
        }
        if c.lessons.is_empty() {
            ui.label(muted("No lessons yet."));
        }
        let mut edit = None;
        for (li, l) in c.lessons.iter().enumerate() {
            theme::card().inner_margin(egui::Margin::symmetric(14, 8)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    theme::badge(ui, l.level.name(), l.level.color());
                    ui.label(RichText::new(format!("{} · {}", l.number, l.title)).strong());
                    ui.label(muted(format!(
                        "{} · {} sentences · {} exercises",
                        l.title_in(&base),
                        l.dialogue.len(),
                        l.ex1.len() + l.ex2.len()
                    )));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✏ Edit").clicked() {
                            edit = Some(li);
                        }
                    });
                });
            });
        }
        if let Some(li) = edit {
            self.edit_lesson(li);
        }
    }

    fn lesson_form(&mut self, ui: &mut egui::Ui, ci: usize) {
        let base = self.base();
        let lang = self.courses[ci].lang.clone();
        let target_name = lang_name(&lang, &base);
        let base_name = lang_name(&base, &base);
        let mut d = std::mem::take(&mut self.editor.draft);

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(if self.editor.lesson.is_some() {
                    "Edit lesson"
                } else {
                    "New lesson"
                })
                .size(19.0)
                .strong(),
            );
            egui::Grid::new("lessonhead").num_columns(2).spacing([14.0, 8.0]).show(ui, |ui| {
                ui.label("Number");
                ui.add(egui::DragValue::new(&mut d.number).range(1..=9999));
                ui.end_row();
                ui.label("Level");
                egui::ComboBox::from_id_salt("edlevel")
                    .selected_text(format!("{} · {}", d.level.name(), d.level.describe()))
                    .show_ui(ui, |ui| {
                        for l in Level::ALL {
                            ui.selectable_value(&mut d.level, l, format!("{} · {}", l.name(), l.describe()));
                        }
                    });
                ui.end_row();
                ui.label(format!("Title in {target_name}"));
                ui.add(egui::TextEdit::singleline(&mut d.title).desired_width(420.0));
                ui.end_row();
                ui.label(format!("Title in {base_name}"));
                ui.add(egui::TextEdit::singleline(d.title_tr.entry(base.clone()).or_default()).desired_width(420.0));
                ui.end_row();
                ui.label("Recording (optional)");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut d.audio)
                            .desired_width(320.0)
                            .hint_text("found automatically in the course's recordings folder"),
                    );
                    if ui.button("Choose…").clicked() {
                        if let Some(f) = rfd::FileDialog::new()
                            .add_filter("Audio", &["mp3", "ogg", "opus", "m4a", "wav", "flac"])
                            .pick_file()
                        {
                            d.audio = f.display().to_string();
                        }
                    }
                });
                ui.end_row();
            });
        });

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Dialogue").size(17.0).strong());
            ui.label(muted(format!("One sentence per row: the {target_name} sentence, its {base_name} translation (put the literal sense in brackets if useful) and, if you like, a pronunciation hint.")));
            item_rows(ui, &mut d.dialogue, &base, "dlg", &target_name, &base_name);
            ui.collapsing("Paste many sentences at once", |ui| {
                ui.label(muted("Paste the dialogue in the left box and the translation in the right box, one sentence per line, in the same order."));
                ui.columns(2, |cols| {
                    cols[0].label(format!("Sentences in {target_name}"));
                    cols[0].add(egui::TextEdit::multiline(&mut self.editor.paste_target).desired_rows(6).desired_width(f32::INFINITY));
                    cols[1].label(format!("Translations in {base_name}"));
                    cols[1].add(egui::TextEdit::multiline(&mut self.editor.paste_tr).desired_rows(6).desired_width(f32::INFINITY));
                });
                let lines: Vec<&str> = self.editor.paste_target.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
                let trs: Vec<&str> = self.editor.paste_tr.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
                if !trs.is_empty() && trs.len() != lines.len() {
                    ui.label(RichText::new(format!("⚠ {} sentences but {} translations", lines.len(), trs.len())).color(BAD));
                }
                if !lines.is_empty() && ui.add(theme::primary(format!("Add these {} sentences", lines.len()))).clicked() {
                    d.dialogue.retain(|i| !i.text.trim().is_empty());
                    for (k, l) in lines.iter().enumerate() {
                        let mut it = Item { text: l.to_string(), ..Default::default() };
                        if let Some(t) = trs.get(k) {
                            it.tr.insert(base.clone(), t.to_string());
                        }
                        d.dialogue.push(it);
                    }
                    self.editor.paste_target.clear();
                    self.editor.paste_tr.clear();
                }
            });
        });

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(format!("Notes (in {base_name})")).size(17.0).strong());
            ui.label(muted("Number each note: “1. word — explanation”."));
            ui.add(
                egui::TextEdit::multiline(d.notes.entry(base.clone()).or_default())
                    .desired_rows(7)
                    .desired_width(f32::INFINITY),
            );
        });

        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Exercise 1 · sentences to translate").size(17.0).strong());
            item_rows(ui, &mut d.ex1, &base, "ex1", &target_name, &base_name);
            ui.add_space(6.0);
            ui.label(RichText::new("Exercise 2 · missing words").size(17.0).strong());
            ui.label(muted("Put each missing word in [square brackets], e.g. “Muy [bien], [gracias].”"));
            item_rows(ui, &mut d.ex2, &base, "ex2", &target_name, &base_name);
        });

        let mut save = false;
        let mut cancel = false;
        let mut delete = false;
        theme::tinted(ACCENT_SOFT).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                save = ui.add(theme::big_primary("💾 Save lesson")).clicked();
                cancel = ui.button("Back to the list").clicked();
                if self.editor.lesson.is_some() {
                    if self.editor.confirm_delete {
                        ui.label(RichText::new("Delete this lesson for good?").color(BAD));
                        delete = ui.button("Yes, delete").clicked();
                        if ui.button("No").clicked() {
                            self.editor.confirm_delete = false;
                        }
                    } else if ui.button("🗑 Delete lesson").clicked() {
                        self.editor.confirm_delete = true;
                    }
                }
            });
        });

        if save {
            match self.store_lesson(ci, &d) {
                Ok(li) => {
                    self.editor.lesson = Some(li);
                    self.editor.msg = format!("Saved lesson {}.", d.number);
                }
                Err(e) => self.editor.msg = format!("⚠ {e}"),
            }
        }
        self.editor.draft = d;
        if delete {
            if let Some(li) = self.editor.lesson {
                let removed = self.courses[ci].lessons.remove(li);
                self.editor.msg = match store::save_course(&self.courses[ci]) {
                    Ok(_) => format!("Deleted lesson {}.", removed.number),
                    Err(e) => format!("⚠ {e}"),
                };
            }
            self.editor.editing = false;
            self.editor.lesson = None;
        }
        if cancel {
            self.editor.editing = false;
            self.editor.msg.clear();
        }
    }

    fn store_lesson(&mut self, ci: usize, d: &Lesson) -> Result<usize, String> {
        let mut lesson = d.clone();
        lesson.title = lesson.title.trim().to_string();
        if lesson.title.is_empty() {
            return Err("give the lesson a title".into());
        }
        for list in [&mut lesson.dialogue, &mut lesson.ex1, &mut lesson.ex2] {
            list.retain(|i| !i.text.trim().is_empty());
        }
        if let Some(bad) = lesson.ex2.iter().find(|i| i.blanks().is_empty()) {
            return Err(format!("exercise 2 needs [brackets] around the missing words: “{}”", bad.text));
        }
        let c = &mut self.courses[ci];
        let current = self.editor.lesson;
        if c.lessons
            .iter()
            .enumerate()
            .any(|(i, l)| l.number == lesson.number && Some(i) != current)
        {
            return Err(format!("lesson {} already exists; choose another number", lesson.number));
        }
        let number = lesson.number;
        match current {
            Some(i) => c.lessons[i] = lesson,
            None => c.lessons.push(lesson),
        }
        c.sort();
        store::save_course(c)?;
        Ok(c.lessons.iter().position(|l| l.number == number).unwrap_or(0))
    }
}

fn item_rows(ui: &mut egui::Ui, items: &mut Vec<Item>, base: &str, id: &str, target_name: &str, base_name: &str) {
    let mut remove = None;
    let mut up = None;
    let w = (ui.available_width() - 200.0).max(300.0);
    egui::Grid::new(id).num_columns(5).spacing([6.0, 6.0]).show(ui, |ui| {
        ui.label("");
        ui.label(muted(target_name));
        ui.label(muted(base_name));
        ui.label(muted("pronunciation"));
        ui.label("");
        ui.end_row();
        for (i, it) in items.iter_mut().enumerate() {
            ui.label(muted(format!("{}", i + 1)));
            ui.add(egui::TextEdit::singleline(&mut it.text).desired_width(w * 0.42));
            ui.add(egui::TextEdit::singleline(it.tr.entry(base.to_string()).or_default()).desired_width(w * 0.4));
            ui.add(egui::TextEdit::singleline(&mut it.pron).desired_width(w * 0.18));
            ui.horizontal(|ui| {
                if i > 0 && ui.small_button("↑").clicked() {
                    up = Some(i);
                }
                if ui.small_button("✕").clicked() {
                    remove = Some(i);
                }
            });
            ui.end_row();
        }
    });
    if let Some(i) = remove {
        items.remove(i);
    }
    if let Some(i) = up {
        items.swap(i, i - 1);
    }
    if ui.button("＋ Add sentence").clicked() {
        items.push(Item::default());
    }
}
