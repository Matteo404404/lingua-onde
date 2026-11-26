//! The front and back of the book, as in the Assimil volumes: the
//! introduction (the language, how to study, the two waves), and at the end
//! a grammar appendix, a grammar index (topic → lessons), a word index in
//! both directions and an index of expressions, each entry with the lessons
//! where it is met. The pronunciation key has its own file.
//!
//! One file per built-in course and reader language,
//! `content/<lang>/appendix-<stage>.<base>.txt`:
//!
//! ```text
//! course: en
//! stage: foundation
//! base: it
//!
//! == intro
//! (markdown)
//!
//! == grammar
//! (markdown, as in the lesson notes: ## headings, tables, *bold*)
//!
//! == index
//! past perfect | 57 58 63
//!
//! == words
//! wallet | portafoglio | 57
//!
//! == phrases
//! look forward to | non vedere l'ora di | 59
//! ```

use eframe::egui::{self, RichText};

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::model::{Stage, lang_name};
use crate::practice::fold;
use crate::theme::{self, ACCENT, ACCENT_SOFT, MUTED, bold, medium, muted};

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub target: String,
    pub base: String,
    pub lessons: Vec<u32>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Appendix {
    pub lang: String,
    pub stage: Option<Stage>,
    pub base: String,
    pub intro: String,
    pub grammar: String,
    /// Topic and its lessons.
    pub index: Vec<(String, Vec<u32>)>,
    pub words: Vec<Entry>,
    pub phrases: Vec<Entry>,
}

fn lessons(s: &str, n: usize) -> Result<Vec<u32>, String> {
    let v: Result<Vec<u32>, _> = s.split([' ', ',']).filter(|x| !x.is_empty()).map(str::parse).collect();
    match v {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => Err(format!("line {n}: lesson numbers expected, found `{s}`")),
    }
}

pub fn parse(src: &str) -> Result<Appendix, String> {
    let mut a = Appendix::default();
    let mut section = "";
    for (i, line) in src.lines().enumerate() {
        let n = i + 1;
        if let Some(s) = line.strip_prefix("== ") {
            section = match s.trim() {
                "intro" => "intro",
                "grammar" => "grammar",
                "index" => "index",
                "words" => "words",
                "phrases" => "phrases",
                other => return Err(format!("line {n}: unknown section `{other}`")),
            };
            continue;
        }
        let t = line.trim();
        match section {
            "" if t.is_empty() => {}
            "" => match t.split_once(':') {
                Some(("course", v)) => a.lang = v.trim().to_string(),
                Some(("stage", v)) => a.stage = Some(Stage::parse(v).ok_or(format!("line {n}: unknown stage `{}`", v.trim()))?),
                Some(("base", v)) => a.base = v.trim().to_string(),
                _ => return Err(format!("line {n}: `course:`, `stage:` or `base:` expected")),
            },
            "intro" | "grammar" => {
                let text = if section == "intro" { &mut a.intro } else { &mut a.grammar };
                text.push_str(line);
                text.push('\n');
            }
            _ if t.is_empty() => {}
            "index" => {
                let (topic, ls) = t.split_once(" | ").ok_or(format!("line {n}: `topic | lessons` expected"))?;
                a.index.push((topic.trim().to_string(), lessons(ls, n)?));
            }
            _ => {
                let f: Vec<&str> = t.split(" | ").map(str::trim).collect();
                let [target, base, ls] = f[..] else {
                    return Err(format!("line {n}: `word | translation | lessons` expected"));
                };
                let e = Entry {
                    target: target.to_string(),
                    base: base.to_string(),
                    lessons: lessons(ls, n)?,
                };
                if section == "words" { &mut a.words } else { &mut a.phrases }.push(e);
            }
        }
    }
    a.grammar = a.grammar.trim().to_string();
    a.intro = a.intro.trim().to_string();
    if a.lang.is_empty() || a.stage.is_none() || a.base.is_empty() {
        return Err("the header needs `course:`, `stage:` and `base:`".into());
    }
    Ok(a)
}

/// Every built-in appendix.
const BUILTIN: &[&str] = &[
    include_str!("../content/en/appendix-foundation.it.txt"),
    include_str!("../content/en/appendix-perfectionnement.it.txt"),
    include_str!("../content/en/appendix-mastery.it.txt"),
    include_str!("../content/vi/appendix-foundation.es.txt"),
    include_str!("../content/vi/appendix-perfectionnement.es.txt"),
];

pub fn builtin() -> Vec<Appendix> {
    BUILTIN.iter().filter_map(|s| parse(s).ok()).collect()
}

/// The book's pages, in its order (`AppendixState::tab` indexes them).
fn page_names() -> [&'static str; 7] {
    [tr("Introduction"), tr("Pronunciation"), tr("Lessons"), tr("Grammar"), tr("Index"), tr("Words"), tr("Expressions")]
}

fn parts() -> [(&'static str, &'static [usize]); 3] {
    [(tr("Before the lessons"), &[0, 1]), (tr("The course"), &[2]), (tr("End of the book"), &[3, 4, 5, 6])]
}

#[derive(Default)]
pub struct AppendixState {
    pub tab: usize,
    pub query: String,
    /// Words listed from the reader's language.
    pub reverse: bool,
    /// List entries from lessons not studied yet too.
    pub all: bool,
}

/// The grammar index has no second column.
fn r_empty(rows: &[(String, String, Vec<u32>)]) -> bool {
    rows.iter().all(|(_, r, _)| r.is_empty())
}

impl App {
    /// The appendix of the active course for the reader's language.
    pub fn appendix(&self) -> Option<&Appendix> {
        let c = self.courses.get(self.active())?;
        let base = self.base();
        let stage = c.stage.filter(|_| c.builtin)?;
        self.appendices
            .iter()
            .find(|a| a.lang == c.lang && a.stage == Some(stage) && a.base == base)
    }

    pub fn appendix_screen(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        let ci = self.active();
        theme::page_title(ui, tr("The book"), &self.course_label(ci));
        let a = self.appendix().cloned();
        let lang_code = self.courses.get(ci).map(|c| c.lang.clone()).unwrap_or_default();
        let key = crate::store::pronunciation_key(&lang_code);
        // The contents in the book's order: what comes before the lessons,
        // the lessons, and what comes at the end of the book.
        // Only Foundation has a pronunciation key among its pages, as in the
        // books; Perfectionnement marks irregular stress in its notes, and its
        // book ends with expressions only, no word list.
        let foundation = self.courses.get(ci).is_none_or(|c| c.stage.is_none_or(|s| s == Stage::Foundation));
        ui.horizontal_wrapped(|ui| {
            for (k, (part, pages)) in parts().iter().enumerate() {
                if k > 0 {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                }
                ui.vertical(|ui| {
                    ui.label(theme::caps(part));
                    ui.horizontal_wrapped(|ui| {
                        for &i in pages.iter().filter(|&&i| foundation || (i != 1 && i != 5)) {
                            if ui.selectable_label(self.appendix_ui.tab == i, medium(page_names()[i]).size(15.0)).clicked() {
                                self.appendix_ui.tab = i;
                            }
                        }
                    });
                });
            }
        });
        ui.add_space(8.0);
        let newcomer = self.needs_intro(ci);
        if newcomer && self.appendix_ui.tab < 2 {
            theme::tinted(ACCENT_SOFT()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label(medium(if foundation {
                        tr("Before lesson 1: read the introduction, then the pronunciation key.")
                    } else {
                        tr("Before lesson 1: read the introduction.")
                    }));
                    if ui.add(theme::ghost(tr("I already know the method: start lesson 1"))).clicked() {
                        self.begin_course(ci);
                    }
                });
            });
            ui.add_space(8.0);
        }
        if self.appendix_ui.tab == 2 {
            self.lessons_page(ui, ci);
            return;
        }
        let text = |s: &str| (!s.is_empty()).then(|| s.to_string());
        let page = match self.appendix_ui.tab {
            0 => Some(a.as_ref().and_then(|a| text(&a.intro))),
            1 => Some(key.map(|k| crate::app::reader_part(k, &base))),
            3 => Some(a.as_ref().and_then(|a| text(&a.grammar))),
            _ => None,
        };
        if let Some(page) = page {
            let written = page.is_some();
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                match page {
                    Some(p) => theme::markdown(ui, &p),
                    None => {
                        ui.label(RichText::new(tr(
                            "Not written yet for this course. As in the books, the introduction and the appendix come with the finished course: a grammar summary, an index of grammar topics, a word list in both directions and the expressions, each with its lessons.",
                        )).size(16.0));
                    }
                }
            });
            // Turn the page: introduction, pronunciation, then the lessons.
            if written && self.appendix_ui.tab < 2 {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if self.appendix_ui.tab == 0 && foundation {
                        if ui.add(theme::primary(tr("Next: the pronunciation key  →"))).clicked() {
                            self.appendix_ui.tab = 1;
                        }
                    } else if newcomer {
                        if ui.add(theme::primary(tr("Begin: lesson 1  →"))).clicked() {
                            self.begin_course(ci);
                        }
                    } else if ui.add(theme::primary(tr("Next: the lessons  →"))).clicked() {
                        self.appendix_ui.tab = 2;
                    }
                });
            }
            return;
        }
        let Some(a) = a else {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(tr("Not written yet for this course: the indexes come with the finished course.")).size(16.0));
            });
            return;
        };
        let empty = match self.appendix_ui.tab {
            4 => a.index.is_empty(),
            5 => a.words.is_empty(),
            _ => a.phrases.is_empty(),
        };
        if empty {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(tr("Not written yet for this course: the indexes come with the finished course.")).size(16.0));
            });
            return;
        }
        let c = self.courses[ci].clone();
        let studied = |n: u32| self.saved.is_done(&c.lesson_key(n));
        let lang = lang_name(&a.lang, &base);
        let st = &mut self.appendix_ui;
        // Wraps on a narrow window instead of running off the page.
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut st.query)
                    .hint_text(tr("Search"))
                    .desired_width(260.0)
                    .font(egui::FontId::proportional(16.0)),
            );
            if st.tab == 5 {
                let mine = lang_name(&base, &base);
                let dir = if st.reverse {
                    format!("{mine} → {lang}")
                } else {
                    format!("{lang} → {mine}")
                };
                if ui
                    .add(theme::ghost(dir))
                    .on_hover_text(tr("Change the direction of the list"))
                    .clicked()
                {
                    st.reverse = !st.reverse;
                }
            }
            ui.checkbox(&mut st.all, tr("Also lessons not studied yet"));
        });
        ui.add_space(6.0);
        let q = fold(st.query.trim());
        let all = st.all;
        // Rows: left, right, lessons.
        let mut rows: Vec<(String, String, Vec<u32>)> = match st.tab {
            4 => a.index.iter().map(|(t, l)| (t.clone(), String::new(), l.clone())).collect(),
            5 if st.reverse => a
                .words
                .iter()
                .map(|e| (e.base.clone(), e.target.clone(), e.lessons.clone()))
                .collect(),
            5 => a
                .words
                .iter()
                .map(|e| (e.target.clone(), e.base.clone(), e.lessons.clone()))
                .collect(),
            _ => a
                .phrases
                .iter()
                .map(|e| (e.target.clone(), e.base.clone(), e.lessons.clone()))
                .collect(),
        };
        rows.retain(|(l, r, ls)| (all || ls.iter().any(|&n| studied(n))) && (q.is_empty() || fold(l).contains(&q) || fold(r).contains(&q)));
        rows.sort_by_key(|(l, _, _)| fold(l));
        let total = match st.tab {
            4 => a.index.len(),
            5 => a.words.len(),
            _ => a.phrases.len(),
        };
        ui.label(muted(trf("{0} of {1}", &[&rows.len(), &total])));
        if rows.is_empty() && !all {
            ui.label(muted(tr(
                "Nothing from the lessons you have studied yet. Tick “Also lessons not studied yet” to see the whole list.",
            )));
        }
        let title = |n: u32| {
            c.lessons
                .iter()
                .find(|l| l.number == n)
                .map(|l| l.title.clone())
                .unwrap_or_default()
        };
        // Three columns of fixed shares, so long words wrap inside their column
        // and the lesson numbers wrap as whole pills.
        theme::card().inner_margin(egui::Margin::symmetric(14, 10)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let w = ui.available_width();
            let gap = 14.0;
            let (left, mid) = if r_empty(&rows) { (w * 0.55, 0.0) } else { (w * 0.3, w * 0.36) };
            let right = (w - left - mid - 2.0 * gap).max(80.0);
            for (i, (l, r, ls)) in rows.iter().enumerate() {
                let fill = if i % 2 == 1 { theme::TRACK() } else { egui::Color32::TRANSPARENT };
                egui::Frame::new()
                    .fill(fill)
                    .corner_radius(6)
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            ui.allocate_ui(egui::vec2(left - 6.0, 0.0), |ui| {
                                ui.set_width(left - 6.0);
                                ui.add(egui::Label::new(bold(l).size(15.5)).wrap());
                            });
                            if mid > 0.0 {
                                ui.allocate_ui(egui::vec2(mid, 0.0), |ui| {
                                    ui.set_width(mid);
                                    ui.add(egui::Label::new(RichText::new(r).size(15.5)).wrap());
                                });
                            }
                            ui.allocate_ui(egui::vec2(right - 6.0, 0.0), |ui| {
                                ui.set_width(right - 6.0);
                                // Pills in lines that fit the column (a pill is about 44 px).
                                let per_line = ((right - 6.0) / 46.0).floor().max(1.0) as usize;
                                ui.vertical(|ui| {
                                    for chunk in ls.chunks(per_line) {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 4.0;
                                            for &n in chunk {
                                                let (fg, bg) = if studied(n) {
                                                    (ACCENT(), ACCENT_SOFT())
                                                } else {
                                                    (MUTED(), egui::Color32::TRANSPARENT)
                                                };
                                                theme::pill(ui, &n.to_string(), fg, bg).on_hover_text(trf("Lesson {0} · {1}", &[&n, &title(n)]));
                                            }
                                        });
                                    }
                                });
                            });
                        });
                    });
            }
        });
    }
}

impl App {
    /// The table of contents: every lesson in order, done ones and the next
    /// open to read, the rest still ahead; the appendix follows them.
    fn lessons_page(&mut self, ui: &mut egui::Ui, ci: usize) {
        let base = self.base();
        let c = self.courses[ci].clone();
        let next = self.next_lesson(ci);
        let newcomer = self.needs_intro(ci);
        let mut open = None;
        theme::card().inner_margin(egui::Margin::symmetric(14, 10)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            if c.lessons.is_empty() {
                ui.label(RichText::new(tr("No lessons written yet for this course.")).size(16.0));
            }
            let mut level = None;
            for (li, l) in c.lessons.iter().enumerate() {
                if level != Some(l.level) {
                    level = Some(l.level);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        theme::badge(ui, l.level.name(), l.level.color());
                        ui.label(muted(l.level.describe()));
                    });
                    ui.add_space(2.0);
                }
                let done = self.saved.is_done(&c.lesson_key(l.number));
                let is_next = next == Some(li);
                let reachable = done || (is_next && !newcomer);
                let fill = if is_next { ACCENT_SOFT() } else { egui::Color32::TRANSPARENT };
                let r = egui::Frame::new()
                    .fill(fill)
                    .corner_radius(8)
                    .inner_margin(egui::Margin::symmetric(8, 2))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let mark = if done { "✔" } else if is_next { "▶" } else { "·" };
                            ui.label(RichText::new(mark).size(15.0).color(if done || is_next { ACCENT() } else { MUTED() }));
                            ui.allocate_ui_with_layout(
                                egui::vec2(28.0, 20.0),
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.set_min_width(28.0);
                                    ui.label(muted(l.number.to_string()).size(14.5));
                                },
                            );
                            let title = if l.revision && !l.title.to_lowercase().contains("revision") {
                                trf("{0} · revision", &[&l.title])
                            } else {
                                l.title.clone()
                            };
                            let t = bold(title).size(15.5);
                            ui.label(if reachable { t } else { t.color(MUTED()) });
                            let translated = l.title_in(&base);
                            if !translated.is_empty() && translated != l.title {
                                ui.label(muted(translated).size(14.0));
                            }
                        });
                    })
                    .response;
                if reachable {
                    let r = r.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
                    if r.clicked() {
                        open = Some(li);
                    }
                }
            }
        });
        if !c.lessons.is_empty() {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.add(theme::ghost(tr("Then, at the end of the book: the grammar  →"))).clicked() {
                    self.appendix_ui.tab = 3;
                }
            });
        }
        if let Some(li) = open {
            self.start_session(ci, li);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_section() {
        let src = "course: en\nstage: foundation\nbase: it\n\n== intro\nBenvenuti.\n\n== grammar\n## 1 Il verbo be\n\n| a | b |\n|---|---|\n| x | y |\n\n== index\npast perfect | 57 58, 63\n\n== words\nwallet | portafoglio | 57\n\n== phrases\nlook forward to | non vedere l'ora di | 59\n";
        let a = parse(src).unwrap();
        assert_eq!((a.lang.as_str(), a.stage, a.base.as_str()), ("en", Some(Stage::Foundation), "it"));
        assert!(a.grammar.starts_with("## 1 Il verbo be"));
        assert_eq!(a.intro, "Benvenuti.");
        assert_eq!(a.index, vec![("past perfect".to_string(), vec![57, 58, 63])]);
        assert_eq!(a.words[0].lessons, vec![57]);
        assert_eq!(a.phrases[0].base, "non vedere l'ora di");
        assert!(parse("course: en\nstage: foundation\nbase: it\n\n== words\nwallet | portafoglio\n").is_err());
    }

    /// Every built-in appendix parses, names real lessons, and each word or
    /// expression is in the dialogue or exercises of the first lesson it names.
    #[test]
    fn builtin_appendices_match_their_lessons() {
        let (courses, _) = crate::store::builtin_courses();
        for src in BUILTIN {
            let a = parse(src).unwrap();
            let c = courses.iter().find(|c| c.lang == a.lang && c.stage == a.stage).expect("its course");
            let text = |n: u32| {
                c.lessons.iter().find(|l| l.number == n).map(|l| {
                    l.dialogue
                        .iter()
                        .chain(&l.ex1)
                        .chain(&l.ex2)
                        .map(|i| fold(&i.plain()))
                        .collect::<Vec<_>>()
                        .join(" ")
                })
            };
            for (t, ls) in &a.index {
                for n in ls {
                    assert!(text(*n).is_some(), "index `{t}`: no lesson {n}");
                }
            }
            for e in a.words.iter().chain(&a.phrases) {
                let first = text(e.lessons[0]).unwrap_or_else(|| panic!("`{}`: no lesson {}", e.target, e.lessons[0]));
                // "be (am, is, are)": one of the forms is enough.
                let word = fold(&e.target);
                let found = word
                    .split(|ch: char| !ch.is_alphanumeric() && ch != '\'')
                    .filter(|w| w.len() > 1)
                    .any(|w| first.contains(w));
                assert!(found, "`{}` is not in lesson {}", e.target, e.lessons[0]);
            }
        }
    }
}
