use std::collections::BTreeMap;
use std::path::PathBuf;

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

use crate::i18n::tr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize, Default)]
pub enum Level {
    #[default]
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
}

impl Level {
    pub const ALL: [Level; 6] = [Level::A1, Level::A2, Level::B1, Level::B2, Level::C1, Level::C2];

    pub fn name(self) -> &'static str {
        match self {
            Level::A1 => "A1",
            Level::A2 => "A2",
            Level::B1 => "B1",
            Level::B2 => "B2",
            Level::C1 => "C1",
            Level::C2 => "C2",
        }
    }

    pub fn parse(s: &str) -> Option<Level> {
        Level::ALL.into_iter().find(|l| l.name().eq_ignore_ascii_case(s.trim()))
    }

    pub fn describe(self) -> &'static str {
        match self {
            Level::A1 => tr("Beginner"),
            Level::A2 => tr("Elementary"),
            Level::B1 => tr("Intermediate"),
            Level::B2 => tr("Upper intermediate"),
            Level::C1 => tr("Advanced"),
            Level::C2 => tr("Mastery"),
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            Level::A1 => Color32::from_rgb(76, 175, 120),
            Level::A2 => Color32::from_rgb(52, 152, 160),
            Level::B1 => Color32::from_rgb(66, 120, 200),
            Level::B2 => Color32::from_rgb(120, 96, 200),
            Level::C1 => Color32::from_rgb(196, 98, 150),
            Level::C2 => Color32::from_rgb(214, 120, 60),
        }
    }
}

/// The three built-in courses of a language, one after the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    /// First words to B2: lessons 1–49 passive, then the active wave.
    Foundation,
    /// B2 to C1: richer texts, the same passive and active cycle.
    Perfectionnement,
    /// C1 to C2: long authentic texts, reformulation and open production.
    Mastery,
}

impl Stage {
    pub const ALL: [Stage; 3] = [Stage::Foundation, Stage::Perfectionnement, Stage::Mastery];

    pub fn code(self) -> &'static str {
        match self {
            Stage::Foundation => "foundation",
            Stage::Perfectionnement => "perfectionnement",
            Stage::Mastery => "mastery",
        }
    }

    pub fn parse(s: &str) -> Option<Stage> {
        Stage::ALL.into_iter().find(|st| st.code() == s.trim())
    }

    pub fn name(self, base: &str) -> &'static str {
        match (self, base) {
            (Stage::Foundation, "it") => "Fondamenti",
            (Stage::Foundation, "es") => "Fundamentos",
            (Stage::Foundation, _) => "Foundation",
            (Stage::Perfectionnement, "it") => "Perfezionamento",
            (Stage::Perfectionnement, "es") => "Perfeccionamiento",
            (Stage::Perfectionnement, _) => "Perfectionnement",
            (Stage::Mastery, "it") => "Padronanza",
            (Stage::Mastery, "es") => "Maestría",
            (Stage::Mastery, _) => "Mastery",
        }
    }

    pub fn span(self) -> &'static str {
        match self {
            Stage::Foundation => "A1 → B2",
            Stage::Perfectionnement => "B2 → C1",
            Stage::Mastery => "C1 → C2",
        }
    }

    /// Lessons in the finished course.
    pub fn size(self) -> u32 {
        match self {
            Stage::Foundation => 100,
            Stage::Perfectionnement => 70,
            Stage::Mastery => 100,
        }
    }

    /// The lesson numbers a level takes in this course, if it has that level.
    pub fn plan(self, level: Level) -> Option<(u32, u32)> {
        use Level::*;
        match (self, level) {
            (Stage::Foundation, A1) => Some((1, 25)),
            (Stage::Foundation, A2) => Some((26, 50)),
            (Stage::Foundation, B1) => Some((51, 75)),
            (Stage::Foundation, B2) => Some((76, 100)),
            (Stage::Perfectionnement, B2) => Some((1, 45)),
            (Stage::Perfectionnement, C1) => Some((46, 70)),
            (Stage::Mastery, C1) => Some((1, 25)),
            (Stage::Mastery, C2) => Some((26, 100)),
            _ => None,
        }
    }

    /// Built-in course id. Foundation keeps the original id, so progress
    /// saved before the courses were split still matches.
    /// Never change these ids: the learner's progress is keyed by them.
    pub fn course_id(self, lang: &str) -> String {
        match self {
            Stage::Foundation => format!("core-{lang}"),
            st => format!("core-{lang}-{}", st.code()),
        }
    }
}

/// One sentence: the target-language text, its translations per base language,
/// an optional pronunciation hint (pinyin, Latin vowel lengths...) and its
/// sound spelled out for a reader of each base language (`pron.it:`...).
/// Open tasks (Mastery) carry an instruction (`task.it:`) and a self-check
/// list (`rubric.it:`, criteria separated by `;`); their text is a model answer.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Item {
    pub text: String,
    pub tr: BTreeMap<String, String>,
    pub pron: String,
    pub figured: BTreeMap<String, String>,
    pub task: BTreeMap<String, String>,
    pub rubric: BTreeMap<String, String>,
    /// In a revision dialogue: the lessons this line's words come from (`refs: 1 3`).
    pub refs: Vec<u32>,
}

impl Item {
    /// The pronunciation spelled for a reader of `base`; none for other
    /// readers, since each language spells sounds its own way.
    pub fn figured(&self, base: &str) -> Option<&str> {
        self.figured.get(base).map(String::as_str).filter(|s| !s.is_empty())
    }

    pub fn tr(&self, base: &str) -> &str {
        pick(&self.tr, base)
    }

    pub fn task_in(&self, base: &str) -> &str {
        pick(&self.task, base)
    }

    /// The self-check criteria, one per entry.
    pub fn rubric_in(&self, base: &str) -> Vec<String> {
        pick(&self.rubric, base)
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect()
    }

    /// The sentence without the `[ ]` that mark missing words.
    pub fn plain(&self) -> String {
        self.text.replace(['[', ']'], "")
    }

    /// The words between `[ ]`, in order.
    pub fn blanks(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = self.text.as_str();
        while let Some(start) = rest.find('[') {
            let Some(len) = rest[start..].find(']') else { break };
            out.push(rest[start + 1..start + len].to_string());
            rest = &rest[start + len + 1..];
        }
        out
    }

    /// The sentence split into plain text and blanks: (text, is_blank).
    pub fn parts(&self) -> Vec<(String, bool)> {
        let mut out = Vec::new();
        let mut rest = self.text.as_str();
        while let Some(start) = rest.find('[') {
            let Some(len) = rest[start..].find(']') else { break };
            if start > 0 {
                out.push((rest[..start].to_string(), false));
            }
            out.push((rest[start + 1..start + len].to_string(), true));
            rest = &rest[start + len + 1..];
        }
        if !rest.is_empty() {
            out.push((rest.to_string(), false));
        }
        out
    }
}

/// Free text for `base`: its own, else the one written for every language,
/// else the usual fallback.
fn text_in<'a>(map: &'a BTreeMap<String, String>, base: &str) -> &'a str {
    map.get(base)
        .or_else(|| map.get(""))
        .map(|s| s.as_str())
        .unwrap_or_else(|| pick(map, base))
}

/// The learners' languages built-in courses are written for.
pub const BASES: [&str; 3] = ["it", "en", "es"];

/// The text for `base`, else English, else Italian, else any.
pub fn pick<'a>(map: &'a BTreeMap<String, String>, base: &str) -> &'a str {
    map.get(base)
        .or_else(|| map.get("en"))
        .or_else(|| map.get("it"))
        .or_else(|| map.values().next())
        .map(|s| s.as_str())
        .unwrap_or("")
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Lesson {
    pub number: u32,
    pub level: Level,
    pub title: String,
    pub title_tr: BTreeMap<String, String>,
    /// The lesson's number spelled in the target language ("Lección primera"),
    /// learnt along the way, and its figured pronunciation per base language.
    pub numeral: String,
    pub numeral_pron: BTreeMap<String, String>,
    pub audio: String,
    /// A revision lesson (`kind: revision`, every 7th built-in lesson): its
    /// notes recap the grammar of the six lessons before it, and its dialogue
    /// reuses their words; exercises are optional.
    pub revision: bool,
    pub dialogue: Vec<Item>,
    /// Pronunciation observations per base language (`sounds.it:`).
    pub sounds: BTreeMap<String, String>,
    /// Notes per base language; key "" = written for every language.
    pub notes: BTreeMap<String, String>,
    /// Exercise 1: translate into your language (the translation is the correction).
    pub ex1: Vec<Item>,
    /// Exercise 2: fill in the missing [words] (the full sentence is the correction).
    pub ex2: Vec<Item>,
    /// A cultural note per base language (`culture.it:`).
    pub culture: BTreeMap<String, String>,
    /// A word from the author: encouragement, an announcement (`aside.it:`).
    pub aside: BTreeMap<String, String>,
    /// Open tasks (Mastery): instruction, model answer, self-check.
    pub tasks: Vec<Item>,
}

impl Lesson {
    pub fn title_in(&self, base: &str) -> &str {
        pick(&self.title_tr, base)
    }

    pub fn notes_in(&self, base: &str) -> &str {
        text_in(&self.notes, base)
    }

    pub fn sounds_in(&self, base: &str) -> &str {
        text_in(&self.sounds, base)
    }

    pub fn culture_in(&self, base: &str) -> &str {
        text_in(&self.culture, base)
    }

    pub fn aside_in(&self, base: &str) -> &str {
        text_in(&self.aside, base)
    }
}

#[derive(Clone, Default, Debug)]
pub struct Course {
    pub id: String,
    pub lang: String,
    /// Which built-in course this is; None for the person's own courses.
    pub stage: Option<Stage>,
    /// The learners' languages a built-in course is written for (`bases: it`);
    /// empty means Italian, English and Spanish. Courses are finished one
    /// pair at a time, so a course may start with one of them.
    pub bases: Vec<String>,
    pub name: String,
    pub name_tr: BTreeMap<String, String>,
    pub lessons: Vec<Lesson>,
    pub builtin: bool,
    /// Where a user course is saved.
    pub path: Option<PathBuf>,
}

impl Course {
    /// The base languages this course gives translations and notes in.
    pub fn bases(&self) -> Vec<&str> {
        if !self.bases.is_empty() {
            return self.bases.iter().map(String::as_str).collect();
        }
        BASES.iter().copied().filter(|b| *b != self.lang).collect()
    }

    /// Whether a learner of `base` gets the course in their own language.
    pub fn speaks(&self, base: &str) -> bool {
        !self.builtin || self.bases().contains(&base)
    }

    pub fn name_in(&self, base: &str) -> &str {
        match self.name_tr.get(base) {
            Some(n) => n,
            None => &self.name,
        }
    }

    pub fn sort(&mut self) {
        self.lessons.sort_by_key(|l| l.number);
    }

    pub fn lesson_key(&self, number: u32) -> String {
        format!("{}#{}", self.id, number)
    }
}

pub struct LangInfo {
    pub code: &'static str,
    pub english: &'static str,
    pub italian: &'static str,
    pub spanish: &'static str,
    pub native: &'static str,
    pub color: Color32,
    /// Piper voice path on the voice repository, and an espeak language
    /// override (Latin is spoken by the Italian voice with Latin phonemes).
    pub voice: (&'static str, Option<&'static str>),
}

pub const LANGS: &[LangInfo] = &[
    LangInfo {
        code: "es",
        english: "Spanish",
        italian: "Spagnolo",
        spanish: "Español",
        native: "Español",
        color: Color32::from_rgb(214, 74, 60),
        voice: ("es/es_ES/davefx/medium/es_ES-davefx-medium", None),
    },
    LangInfo {
        code: "fr",
        english: "French",
        italian: "Francese",
        spanish: "Francés",
        native: "Français",
        color: Color32::from_rgb(58, 102, 196),
        voice: ("fr/fr_FR/siwis/medium/fr_FR-siwis-medium", None),
    },
    LangInfo {
        code: "de",
        english: "German",
        italian: "Tedesco",
        spanish: "Alemán",
        native: "Deutsch",
        color: Color32::from_rgb(60, 60, 70),
        voice: ("de/de_DE/thorsten/medium/de_DE-thorsten-medium", None),
    },
    LangInfo {
        code: "nl",
        english: "Dutch",
        italian: "Olandese",
        spanish: "Neerlandés",
        native: "Nederlands",
        color: Color32::from_rgb(232, 118, 34),
        voice: ("nl/nl_NL/pim/medium/nl_NL-pim-medium", None),
    },
    LangInfo {
        code: "pt",
        english: "Portuguese",
        italian: "Portoghese",
        spanish: "Portugués",
        native: "Português",
        color: Color32::from_rgb(40, 150, 90),
        voice: ("pt/pt_BR/faber/medium/pt_BR-faber-medium", None),
    },
    LangInfo {
        code: "zh",
        english: "Chinese",
        italian: "Cinese",
        spanish: "Chino",
        native: "中文",
        color: Color32::from_rgb(200, 40, 50),
        voice: ("zh/zh_CN/huayan/medium/zh_CN-huayan-medium", None),
    },
    LangInfo {
        code: "la",
        english: "Latin",
        italian: "Latino",
        spanish: "Latín",
        native: "Latina",
        color: Color32::from_rgb(150, 110, 50),
        voice: ("it/it_IT/paola/medium/it_IT-paola-medium", Some("la")),
    },
    LangInfo {
        code: "vi",
        english: "Vietnamese",
        italian: "Vietnamita",
        spanish: "Vietnamita",
        native: "Tiếng Việt",
        color: Color32::from_rgb(218, 160, 30),
        voice: ("vi/vi_VN/vais1000/medium/vi_VN-vais1000-medium", None),
    },
    LangInfo {
        code: "it",
        english: "Italian",
        italian: "Italiano",
        spanish: "Italiano",
        native: "Italiano",
        color: Color32::from_rgb(30, 140, 70),
        voice: ("it/it_IT/paola/medium/it_IT-paola-medium", None),
    },
    LangInfo {
        code: "en",
        english: "English",
        italian: "Inglese",
        spanish: "Inglés",
        native: "English",
        color: Color32::from_rgb(40, 70, 140),
        voice: ("en/en_GB/cori/medium/en_GB-cori-medium", None),
    },
];

pub fn lang(code: &str) -> Option<&'static LangInfo> {
    LANGS.iter().find(|l| l.code == code)
}

/// The language's name as the learner reads it.
pub fn lang_name(code: &str, base: &str) -> String {
    match lang(code) {
        Some(l) if base == "it" => l.italian.to_string(),
        Some(l) if base == "es" => l.spanish.to_string(),
        Some(l) => l.english.to_string(),
        None => code.to_string(),
    }
}

pub fn lang_color(code: &str) -> Color32 {
    lang(code).map(|l| l.color).unwrap_or(Color32::GRAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blanks_and_parts() {
        let it = Item {
            text: "Muy [bien], [gracias].".into(),
            ..Default::default()
        };
        assert_eq!(it.blanks(), vec!["bien", "gracias"]);
        assert_eq!(it.plain(), "Muy bien, gracias.");
        let parts = it.parts();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[1], ("bien".to_string(), true));
        assert_eq!(parts[4], (".".to_string(), false));
    }
}
