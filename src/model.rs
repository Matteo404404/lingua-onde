use std::collections::BTreeMap;
use std::path::PathBuf;

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

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
            Level::A1 => "Beginner",
            Level::A2 => "Elementary",
            Level::B1 => "Intermediate",
            Level::B2 => "Upper intermediate",
            Level::C1 => "Advanced",
            Level::C2 => "Mastery",
        }
    }

    /// Lesson numbers this level takes in a full built-in course, shaped like
    /// the person's books: 1–100 up to B2, 101–200 from B2 to C1, C2 after.
    /// shortcut: the C2 range (201–300) is an assumption; the books don't say.
    pub fn plan(self) -> (u32, u32) {
        match self {
            Level::A1 => (1, 25),
            Level::A2 => (26, 50),
            Level::B1 => (51, 75),
            Level::B2 => (76, 150),
            Level::C1 => (151, 200),
            Level::C2 => (201, 300),
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

/// One sentence: the target-language text, its translations per base language
/// and an optional pronunciation hint (pinyin, Latin vowel lengths...).
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Item {
    pub text: String,
    pub tr: BTreeMap<String, String>,
    pub pron: String,
}

impl Item {
    pub fn tr(&self, base: &str) -> &str {
        pick(&self.tr, base)
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
    pub audio: String,
    pub dialogue: Vec<Item>,
    /// Notes per base language; key "" = written for every language.
    pub notes: BTreeMap<String, String>,
    pub ex1: Vec<Item>,
    pub ex2: Vec<Item>,
}

impl Lesson {
    pub fn title_in(&self, base: &str) -> &str {
        pick(&self.title_tr, base)
    }

    pub fn notes_in(&self, base: &str) -> &str {
        self.notes
            .get(base)
            .or_else(|| self.notes.get(""))
            .map(|s| s.as_str())
            .unwrap_or_else(|| pick(&self.notes, base))
    }
}

#[derive(Clone, Default, Debug)]
pub struct Course {
    pub id: String,
    pub lang: String,
    pub name: String,
    pub name_tr: BTreeMap<String, String>,
    pub lessons: Vec<Lesson>,
    pub builtin: bool,
    /// Where a user course is saved.
    pub path: Option<PathBuf>,
}

impl Course {
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
        native: "Español",
        color: Color32::from_rgb(214, 74, 60),
        voice: ("es/es_ES/davefx/medium/es_ES-davefx-medium", None),
    },
    LangInfo {
        code: "fr",
        english: "French",
        italian: "Francese",
        native: "Français",
        color: Color32::from_rgb(58, 102, 196),
        voice: ("fr/fr_FR/siwis/medium/fr_FR-siwis-medium", None),
    },
    LangInfo {
        code: "de",
        english: "German",
        italian: "Tedesco",
        native: "Deutsch",
        color: Color32::from_rgb(60, 60, 70),
        voice: ("de/de_DE/thorsten/medium/de_DE-thorsten-medium", None),
    },
    LangInfo {
        code: "pt",
        english: "Portuguese",
        italian: "Portoghese",
        native: "Português",
        color: Color32::from_rgb(40, 150, 90),
        voice: ("pt/pt_BR/faber/medium/pt_BR-faber-medium", None),
    },
    LangInfo {
        code: "zh",
        english: "Chinese",
        italian: "Cinese",
        native: "中文",
        color: Color32::from_rgb(200, 40, 50),
        voice: ("zh/zh_CN/huayan/medium/zh_CN-huayan-medium", None),
    },
    LangInfo {
        code: "la",
        english: "Latin",
        italian: "Latino",
        native: "Latina",
        color: Color32::from_rgb(150, 110, 50),
        voice: ("it/it_IT/paola/medium/it_IT-paola-medium", Some("la")),
    },
    LangInfo {
        code: "vi",
        english: "Vietnamese",
        italian: "Vietnamita",
        native: "Tiếng Việt",
        color: Color32::from_rgb(218, 160, 30),
        voice: ("vi/vi_VN/vais1000/medium/vi_VN-vais1000-medium", None),
    },
    LangInfo {
        code: "it",
        english: "Italian",
        italian: "Italiano",
        native: "Italiano",
        color: Color32::from_rgb(30, 140, 70),
        voice: ("it/it_IT/paola/medium/it_IT-paola-medium", None),
    },
    LangInfo {
        code: "en",
        english: "English",
        italian: "Inglese",
        native: "English",
        color: Color32::from_rgb(40, 70, 140),
        voice: ("en/en_US/lessac/medium/en_US-lessac-medium", None),
    },
];

pub fn lang(code: &str) -> Option<&'static LangInfo> {
    LANGS.iter().find(|l| l.code == code)
}

/// The language's name as the learner reads it.
pub fn lang_name(code: &str, base: &str) -> String {
    match lang(code) {
        Some(l) if base == "it" => l.italian.to_string(),
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
