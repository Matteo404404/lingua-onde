//! The active side of the method: the active wave (rebuilding an earlier
//! lesson's dialogue and exercise 1 from your own language), Mastery's
//! whole-text recreation, open tasks with model answers, and the forgiving
//! answer check they share.

use std::collections::{HashMap, HashSet};

use eframe::egui::{self, RichText};

use crate::app::App;
use crate::model::{Item, Stage, lang_name};
use crate::theme::{self, ACCENT, ACCENT_SOFT, BAD, GOOD, HIGHLIGHT, WARN, bold, caps, medium, muted};

// ------------------------------------------------------------ checking

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Verdict {
    Right,
    /// Every word is there; only accents or tone marks differ.
    Accents,
    Wrong,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Word {
    Right,
    Accent,
    Missing,
}

/// A typed answer set against the model sentence.
#[derive(Debug)]
pub struct Comparison {
    pub verdict: Verdict,
    /// The model's words, each marked by what the answer did with it.
    pub words: Vec<(String, Word)>,
    /// Typed words the model does not have.
    pub extra: Vec<String>,
}

/// Lowercase and without accents, tone marks or other diacritics.
pub fn fold(s: &str) -> String {
    const TABLE: &[(&str, &str)] = &[
        ("àáâãäåāăąạảấầẩẫậắằẳẵặǎ", "a"),
        ("èéêëēĕėęěẹẻẽếềểễệ", "e"),
        ("ìíîïĩīĭįıỉịǐ", "i"),
        ("òóôõöøōŏőơọỏốồổỗộớờởỡợǒ", "o"),
        ("ùúûüũūŭůűųưụủứừửữựǔǖǘǚǜ", "u"),
        ("ýÿŷỳỵỷỹ", "y"),
        ("çćĉċč", "c"),
        ("ñńņňŉ", "n"),
        ("śŝşšș", "s"),
        ("źżž", "z"),
        ("đď", "d"),
        ("ğ", "g"),
        ("ł", "l"),
        ("ř", "r"),
        ("ţťț", "t"),
    ];
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        match c {
            'ß' => out.push_str("ss"),
            'æ' => out.push_str("ae"),
            'œ' => out.push_str("oe"),
            c => match TABLE.iter().find(|(set, _)| set.contains(c)) {
                Some((_, plain)) => out.push_str(plain),
                None => out.push(c),
            },
        }
    }
    out
}

fn is_cjk(c: char) -> bool {
    matches!(c, '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')
}

/// Words, without punctuation; Chinese characters one by one.
fn tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() && !is_cjk(c) {
            word.push(c);
            continue;
        }
        if !word.is_empty() {
            out.push(std::mem::take(&mut word));
        }
        if is_cjk(c) {
            out.push(c.to_string());
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// Compares word by word, forgiving case and punctuation. Accents count
/// apart, so the learner sees them without being failed for them.
pub fn compare(answer: &str, model: &str) -> Comparison {
    let (a, m) = (tokens(answer), tokens(model));
    let fa: Vec<String> = a.iter().map(|t| fold(t)).collect();
    let fm: Vec<String> = m.iter().map(|t| fold(t)).collect();
    // Longest common subsequence of the folded words.
    let mut dp = vec![vec![0u16; fm.len() + 1]; fa.len() + 1];
    for i in (0..fa.len()).rev() {
        for j in (0..fm.len()).rev() {
            dp[i][j] = if fa[i] == fm[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let (mut words, mut extra) = (Vec::new(), Vec::new());
    while i < a.len() || j < m.len() {
        if i < a.len() && j < m.len() && fa[i] == fm[j] {
            let w = if a[i].to_lowercase() == m[j].to_lowercase() {
                Word::Right
            } else {
                Word::Accent
            };
            words.push((m[j].clone(), w));
            i += 1;
            j += 1;
        } else if j < m.len() && (i == a.len() || dp[i][j + 1] >= dp[i + 1][j]) {
            words.push((m[j].clone(), Word::Missing));
            j += 1;
        } else {
            extra.push(a[i].clone());
            i += 1;
        }
    }
    let verdict = if !extra.is_empty() || words.iter().any(|(_, w)| *w == Word::Missing) {
        Verdict::Wrong
    } else if words.iter().any(|(_, w)| *w == Word::Accent) {
        Verdict::Accents
    } else {
        Verdict::Right
    };
    Comparison { verdict, words, extra }
}

/// Like `compare`, against the item's sentence; for Chinese typed in pinyin,
/// against its pinyin line, tones not required.
pub fn compare_item(answer: &str, item: &Item) -> Comparison {
    let latin_for_chinese = item.text.chars().any(is_cjk) && !answer.chars().any(is_cjk) && !item.pron.is_empty();
    if latin_for_chinese {
        let mut c = compare(answer, &item.pron);
        if c.verdict == Verdict::Accents {
            c.verdict = Verdict::Right;
        }
        return c;
    }
    compare(answer, &item.plain())
}

pub fn show_comparison(ui: &mut egui::Ui, c: &Comparison) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        match c.verdict {
            Verdict::Right => {
                ui.label(medium("✔ Right").color(GOOD));
                return;
            }
            Verdict::Accents => ui.label(medium("≈ Right, check the accents:").color(WARN)),
            Verdict::Wrong => ui.label(medium("Compare:").color(BAD)),
        };
        for (w, mark) in &c.words {
            let t = RichText::new(w).size(16.0);
            ui.label(match mark {
                Word::Right => t.color(GOOD),
                Word::Accent => t.color(WARN).underline(),
                Word::Missing => t.color(BAD).underline(),
            });
        }
        if !c.extra.is_empty() {
            ui.label(muted("· you added:"));
            for w in &c.extra {
                ui.label(RichText::new(w).color(BAD).strikethrough());
            }
        }
    })
    .response
    .on_hover_text("Green: right · orange: accents differ · red underlined: missing or different · struck: not in the original");
}

// --------------------------------------------------------------- state

/// What the learner did on one practice screen: typed answers, revealed
/// sentences, ticked self-check criteria.
#[derive(Default, Clone)]
pub struct Practice {
    pub typed: HashMap<usize, String>,
    pub shown: HashSet<usize>,
    pub ticks: HashSet<(usize, usize)>,
    /// Mastery: the whole text recreated at once.
    pub whole: String,
    pub whole_shown: bool,
    pub by_sentence: bool,
}

/// Self-check for recreating a whole Mastery text.
const MASTERY_RUBRIC: [&str; 5] = [
    "Same meaning: nothing important missing, nothing added",
    "Same register and tone (formal, ironic, warm, technical…)",
    "Natural word order and idiom, not your language's sentences in disguise",
    "The original's key expressions and collocations, or equally precise ones",
    "The same effect on the reader or listener",
];

fn ticklist(ui: &mut egui::Ui, items: &[String], ticks: &mut HashSet<(usize, usize)>, row: usize) {
    for (j, c) in items.iter().enumerate() {
        let mut on = ticks.contains(&(row, j));
        if ui.checkbox(&mut on, c.as_str()).changed() {
            if on {
                ticks.insert((row, j));
            } else {
                ticks.remove(&(row, j));
            }
        }
    }
    let n = (0..items.len()).filter(|j| ticks.contains(&(row, *j))).count();
    ui.label(muted(format!("{n} of {} — be honest; what is missing is next time's focus.", items.len())).size(13.0));
}

fn answer_box(ui: &mut egui::Ui, text: &mut String, hint: &str, rows: usize) -> egui::Response {
    ui.add(
        egui::TextEdit::multiline(text)
            .desired_rows(rows)
            .desired_width(f32::INFINITY)
            .hint_text(hint)
            .font(egui::FontId::proportional(16.0)),
    )
}

impl App {
    /// The active wave (Assimil's second wave): an earlier lesson's dialogue
    /// and its exercise 1, shown in the learner's language only. The learner
    /// says each sentence aloud in the language they learn (typing it is
    /// optional), then reveals it. A revision lesson also brings back its
    /// grammar. Mastery recreates the whole text, or goes sentence by sentence.
    pub(crate) fn reconstruct(&mut self, ui: &mut egui::Ui, ci: usize, wi: usize, p: &mut Practice) {
        let base = self.base();
        let course = &self.courses[ci];
        let (lang, stage) = (course.lang.clone(), course.stage);
        let lesson = course.lessons[wi].clone();
        let rec = self.recording(ci, wi);
        ui.horizontal(|ui| {
            theme::badge(ui, lesson.level.name(), lesson.level.color());
            ui.label(bold(format!("Lesson {} · {}", lesson.number, lesson.title)).size(18.0));
            ui.label(muted(lesson.title_in(&base)));
        });
        if lesson.revision {
            egui::CollapsingHeader::new(medium("The grammar of that week — read it again"))
                .id_salt("wave-recap")
                .show(ui, |ui| crate::session::notes(ui, lesson.notes_in(&base)));
        }
        let whole = stage == Some(Stage::Mastery) && !p.by_sentence;
        if stage == Some(Stage::Mastery) {
            let label = if p.by_sentence {
                "Recreate the whole text instead"
            } else {
                "Go sentence by sentence instead"
            };
            if ui.add(theme::ghost(label)).clicked() {
                p.by_sentence = !p.by_sentence;
            }
        }
        if whole {
            self.recreate_whole(ui, &lang, &lesson.dialogue, p);
        } else {
            ui.label(caps(if lesson.revision { "The revision dialogue" } else { "The dialogue" }));
            self.rebuild_rows(ui, "wave-dlg", &lang, &lesson.dialogue, 0, p);
        }
        if !lesson.ex1.is_empty() {
            ui.add_space(6.0);
            ui.label(caps("Exercise 1 — now from your language"));
            self.rebuild_rows(ui, "wave-ex1", &lang, &lesson.ex1, lesson.dialogue.len(), p);
        }
        self.original_context(ui, &lang, &lesson.dialogue, rec);
    }

    /// Sentences shown in the learner's language, to be said (and optionally
    /// typed) in the target language, then revealed and compared. `offset`
    /// keeps the answers of several lists apart in one `Practice`.
    fn rebuild_rows(&mut self, ui: &mut egui::Ui, tag: &str, lang: &str, items: &[Item], offset: usize, p: &mut Practice) {
        let base = self.base();
        let target_ready = self.voices.ready(lang);
        ui.horizontal_wrapped(|ui| {
            let drilling = self.current(tag).is_some();
            let label = if drilling { "■ Stop" } else { "▶ Prompt and answer" };
            let b = ui
                .add_enabled(target_ready, theme::secondary(label))
                .on_hover_text(
                    "The voice says each sentence in your language, waits while you say it, then says it in the language you learn",
                )
                .on_disabled_hover_text("Download this language's voice in Settings first");
            if b.clicked() {
                if drilling {
                    self.stop_audio();
                } else {
                    let prompts: Vec<String> = items.iter().map(|i| i.tr(&base).to_string()).collect();
                    let answers = items.iter().map(|i| i.plain()).collect();
                    self.play_pairs(tag, &base, prompts, lang, answers);
                }
            }
            if !self.voices.ready(&base) && target_ready {
                ui.label(
                    muted(format!(
                        "(get the {} voice in Settings to hear the prompts too)",
                        lang_name(&base, &base)
                    ))
                    .size(13.0),
                );
            }
            if ui.add(theme::ghost("Show all")).clicked() {
                p.shown.extend(offset..offset + items.len());
            }
        });
        let cur = self.current(tag);
        let mut play = None;
        theme::card().inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (i, it) in items.iter().enumerate() {
                let k = offset + i;
                let fill = if cur == Some(i) { ACCENT_SOFT } else { egui::Color32::TRANSPARENT };
                egui::Frame::new()
                    .fill(fill)
                    .corner_radius(10)
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_top(|ui| {
                            theme::number(ui, i + 1, cur == Some(i));
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 5.0;
                                ui.label(RichText::new(it.tr(&base)).size(18.0));
                                let typed = p.typed.entry(k).or_default();
                                if !p.shown.contains(&k) {
                                    ui.horizontal(|ui| {
                                        let field = ui.add(
                                            egui::TextEdit::singleline(typed)
                                                .desired_width((ui.available_width() - 120.0).max(160.0))
                                                .hint_text("say it aloud — and type it if you like")
                                                .font(egui::FontId::proportional(16.0)),
                                        );
                                        let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                        if ui.add(theme::secondary("Show")).clicked() || enter {
                                            p.shown.insert(k);
                                            if target_ready {
                                                play = Some(i);
                                            }
                                        }
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        if target_ready && ui.add(theme::ghost("▶").small()).clicked() {
                                            play = Some(i);
                                        }
                                        ui.label(bold(it.plain()).size(18.0).color(ACCENT));
                                    });
                                    self.pron_lines(ui, it);
                                    if !typed.trim().is_empty() {
                                        show_comparison(ui, &compare_item(typed, it));
                                    }
                                }
                            });
                        });
                    });
            }
        });
        if let Some(i) = play {
            self.play_seq(&format!("{tag}-one"), lang, items.iter().map(|i| i.plain()).collect(), i, true);
        }
    }

    /// The old dialogue as it was, with its audio: context for the active wave.
    fn original_context(&mut self, ui: &mut egui::Ui, lang: &str, items: &[Item], rec: Option<std::path::PathBuf>) {
        egui::CollapsingHeader::new(medium("The original dialogue — listen and read it again"))
            .id_salt("wave-original")
            .show(ui, |ui| {
                self.listen_panel(ui, "wave-orig", lang, items, rec);
                self.dialogue_block(ui, "wave-orig", lang, items, true, true);
            });
    }

    /// Mastery's wave: the whole text in your language, recreated in the
    /// target language, then set against the original with a self-check.
    fn recreate_whole(&mut self, ui: &mut egui::Ui, lang: &str, items: &[Item], p: &mut Practice) {
        let base = self.base();
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(caps("In your language"));
            let text: Vec<&str> = items.iter().map(|i| i.tr(&base)).collect();
            ui.label(RichText::new(text.join(" ")).size(16.5));
        });
        ui.label(muted(format!(
            "Recreate it in {}: the same meaning, tone, register and effect — not word for word. Say it aloud, or write it here.",
            lang_name(lang, &base)
        )));
        answer_box(ui, &mut p.whole, "Your version…", 8);
        if !p.whole_shown {
            if ui.add(theme::primary("Show the original and the checklist")).clicked() {
                p.whole_shown = true;
            }
            return;
        }
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(caps("The original"));
            let text: Vec<String> = items.iter().map(|i| i.plain()).collect();
            ui.label(RichText::new(text.join(" ")).size(16.5).color(ACCENT));
            ui.add_space(6.0);
            ui.label(caps("Check your version"));
            let rubric: Vec<String> = MASTERY_RUBRIC.iter().map(|s| s.to_string()).collect();
            ticklist(ui, &rubric, &mut p.ticks, usize::MAX);
        });
    }

    /// Mastery's open tasks: instruction, the learner's answer, then a model
    /// answer and a self-check list. There is no single right answer to mark.
    pub(crate) fn tasks_step(&mut self, ui: &mut egui::Ui, lang: &str, items: &[Item], p: &mut Practice) {
        let base = self.base();
        let can_play = self.voices.ready(lang);
        let mut play = None;
        for (i, it) in items.iter().enumerate() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(caps(&format!("Task {}", i + 1)));
                ui.label(RichText::new(it.task_in(&base)).size(17.0));
                answer_box(ui, p.typed.entry(i).or_default(), "Your answer — or say it aloud", 5);
                if !p.shown.contains(&i) {
                    if ui.add(theme::secondary("Show a model answer and the checklist")).clicked() {
                        p.shown.insert(i);
                    }
                    return;
                }
                theme::tinted(HIGHLIGHT).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(caps("A model answer"));
                        if can_play && ui.add(theme::ghost("▶").small()).clicked() {
                            play = Some(i);
                        }
                    });
                    ui.label(RichText::new(&it.text).size(16.0).color(ACCENT));
                    if !it.tr(&base).is_empty() {
                        egui::CollapsingHeader::new(muted("In your language"))
                            .id_salt(("task-tr", i))
                            .show(ui, |ui| {
                                ui.label(it.tr(&base));
                            });
                    }
                });
                ui.label(caps("Check your answer"));
                ticklist(ui, &it.rubric_in(&base), &mut p.ticks, i);
            });
        }
        if let Some(i) = play {
            self.play_seq("tasks", lang, items.iter().map(|i| i.plain()).collect(), i, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forgiving_but_precise() {
        assert_eq!(compare("hola, ¿qué tal?", "Hola, ¿qué tal?").verdict, Verdict::Right);
        assert_eq!(compare("HOLA QUE TAL", "Hola, ¿qué tal?").verdict, Verdict::Accents);
        let c = compare("ik woon Utrecht", "Ik woon in Utrecht.");
        assert_eq!(c.verdict, Verdict::Wrong);
        assert_eq!(c.words[2], ("in".to_string(), Word::Missing));
        let c = compare("je suis très très content", "Je suis très content.");
        assert_eq!(c.extra, ["très"]);
        assert_eq!(compare("", "Bien.").verdict, Verdict::Wrong);
        assert_eq!(compare("Strasse", "Straße").verdict, Verdict::Accents);
        assert_eq!(compare("toi la nguoi Y", "Tôi là người Ý.").verdict, Verdict::Accents);
        assert_eq!(compare("l eau", "l'eau").verdict, Verdict::Right);
    }

    #[test]
    fn chinese_by_character_or_pinyin() {
        let it = Item {
            text: "我是学生。".into(),
            pron: "Wǒ shì xuésheng.".into(),
            ..Default::default()
        };
        assert_eq!(compare_item("我是学生", &it).verdict, Verdict::Right);
        assert_eq!(compare_item("wo shi xuesheng", &it).verdict, Verdict::Right);
        let c = compare_item("我学生", &it);
        assert_eq!(c.words[1], ("是".to_string(), Word::Missing));
    }
}
