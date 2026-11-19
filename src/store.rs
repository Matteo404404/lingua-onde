//! Where things live on disk: the built-in courses, the person's own courses,
//! and their progress (one JSON file).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::format::{parse_course, write_course};
use crate::model::Course;

/// Built-in lesson files. Several files may add lessons to one language's
/// course (the `course:` line says which); a language's first file names it.
pub const BUILTIN: &[(&str, &str)] = &[
    ("es.txt", include_str!("../content/es.txt")),
    ("es-part2.txt", include_str!("../content/es-part2.txt")),
    ("fr.txt", include_str!("../content/fr.txt")),
    ("fr-part2.txt", include_str!("../content/fr-part2.txt")),
    ("de.txt", include_str!("../content/de.txt")),
    ("de-part2.txt", include_str!("../content/de-part2.txt")),
    ("pt.txt", include_str!("../content/pt.txt")),
    ("pt-part2.txt", include_str!("../content/pt-part2.txt")),
    ("zh.txt", include_str!("../content/zh.txt")),
    ("zh-part2.txt", include_str!("../content/zh-part2.txt")),
    ("la.txt", include_str!("../content/la.txt")),
    ("la-part2.txt", include_str!("../content/la-part2.txt")),
    ("vi.txt", include_str!("../content/vi.txt")),
    ("vi-part2.txt", include_str!("../content/vi-part2.txt")),
];

/// The built-in courses, one per language, merged from their files, and
/// what went wrong reading them.
pub fn builtin_courses() -> (Vec<Course>, Vec<String>) {
    let mut out: Vec<Course> = Vec::new();
    let mut problems = Vec::new();
    for (file, src) in BUILTIN {
        let c = match parse_course(src, "") {
            Ok(c) => c,
            Err(e) => {
                problems.push(format!("built-in {file}: {e}"));
                continue;
            }
        };
        match out.iter_mut().find(|o| o.lang == c.lang) {
            Some(o) => {
                for l in c.lessons {
                    if o.lessons.iter().any(|x| x.number == l.number) {
                        problems.push(format!("built-in {file}: lesson {} is already in another file", l.number));
                    } else {
                        o.lessons.push(l);
                    }
                }
                o.sort();
            }
            None => out.push(Course {
                id: format!("core-{}", c.lang),
                builtin: true,
                ..c
            }),
        }
    }
    (out, problems)
}

pub fn data_dir() -> PathBuf {
    if let Ok(p) = std::env::var("LINGUA_ONDE_HOME") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    if cfg!(windows) {
        let base = std::env::var("APPDATA").unwrap_or(home);
        return Path::new(&base).join("LinguaOnde");
    }
    if cfg!(target_os = "macos") {
        return Path::new(&home).join("Library/Application Support/LinguaOnde");
    }
    let base = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{home}/.local/share"));
    Path::new(&base).join("lingua-onde")
}

pub fn courses_dir() -> PathBuf {
    data_dir().join("courses")
}

/// Write via a temporary file so a crash never leaves half a file.
pub fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

/// Built-in courses first, then the person's own (each file in courses/).
/// Returns the courses and the files that could not be read.
pub fn load_courses() -> (Vec<Course>, Vec<String>) {
    let (mut out, mut problems) = builtin_courses();
    let mut files: Vec<PathBuf> = std::fs::read_dir(courses_dir())
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "txt"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    for path in files {
        let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|s| parse_course(&s, &id));
        match parsed {
            Ok(mut c) => {
                c.path = Some(path);
                out.push(c);
            }
            Err(e) => problems.push(format!("{}: {e}", path.display())),
        }
    }
    (out, problems)
}

pub fn save_course(c: &Course) -> Result<PathBuf, String> {
    let path = c.path.clone().unwrap_or_else(|| courses_dir().join(format!("{}.txt", c.id)));
    write_atomic(&path, &write_course(c)).map_err(|e| format!("could not save {}: {e}", path.display()))?;
    Ok(path)
}

/// A free id for a new course file.
pub fn new_course_id(lang: &str, taken: &[Course]) -> String {
    (1..)
        .map(|n| if n == 1 { format!("my-{lang}") } else { format!("my-{lang}-{n}") })
        .find(|id| !taken.iter().any(|c| &c.id == id) && !courses_dir().join(format!("{id}.txt")).exists())
        .expect("infinite range")
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn day_key(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct LessonProgress {
    /// Day the guided session was completed (first time).
    pub completed: Option<String>,
    pub times: u32,
    /// Day of the active "second wave" review.
    pub wave2: Option<String>,
    /// Exercise 2 score of the last session: (right, total).
    pub score: (u32, u32),
    /// Steps done in an unfinished session, to resume where it stopped.
    pub steps_done: Vec<usize>,
}

/// A sentence in spaced review (an SM-2 style schedule).
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Card {
    pub due: String,
    pub interval: u32,
    pub ease: f32,
    pub reps: u32,
    pub lapses: u32,
}

impl Default for Card {
    fn default() -> Self {
        Card {
            due: day_key(today()),
            interval: 0,
            ease: 2.5,
            reps: 0,
            lapses: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

impl Card {
    /// Updates the schedule; returns true when the card should come back in
    /// this same session.
    pub fn grade(&mut self, g: Grade, today: NaiveDate) -> bool {
        let next = match g {
            Grade::Again => {
                self.lapses += 1;
                self.reps = 0;
                self.ease = (self.ease - 0.2).max(1.3);
                0
            }
            Grade::Hard => {
                self.ease = (self.ease - 0.15).max(1.3);
                ((self.interval as f32) * 1.2).round().max(1.0) as u32
            }
            Grade::Good | Grade::Easy => {
                let base = match self.reps {
                    0 => 1,
                    1 => 3,
                    _ => ((self.interval as f32) * self.ease).round() as u32,
                };
                if g == Grade::Easy {
                    self.ease += 0.15;
                    ((base as f32) * 1.3).round().max(base as f32 + 1.0) as u32
                } else {
                    base.max(self.interval + 1)
                }
            }
        };
        if g != Grade::Again {
            self.reps += 1;
        }
        self.interval = next;
        self.due = day_key(today + Duration::days(next as i64));
        g == Grade::Again
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct DayLog {
    pub seconds: f64,
    pub lessons: u32,
    pub reviews: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Saved {
    /// The learner's own language: translations and notes are shown in it.
    pub base: String,
    pub active_course: String,
    pub audio_dirs: BTreeMap<String, String>,
    pub speed: f32,
    /// Pause after each sentence, as a multiple of the sentence's length.
    pub pause_factor: f32,
    /// The second wave of a lesson starts this many lessons later.
    pub wave2_gap: u32,
    pub daily_goal_min: u32,
    pub cards_after_lesson: bool,
    pub show_pron: bool,
    pub prefer_voice: bool,
    /// The voice pauses after each sentence so the learner can repeat it.
    pub repeat_pauses: bool,
    pub lessons: BTreeMap<String, LessonProgress>,
    pub cards: BTreeMap<String, Card>,
    pub days: BTreeMap<String, DayLog>,
}

impl Default for Saved {
    fn default() -> Self {
        Saved {
            base: "it".into(),
            active_course: "core-es".into(),
            audio_dirs: BTreeMap::new(),
            speed: 1.0,
            pause_factor: 1.3,
            wave2_gap: 49,
            daily_goal_min: 20,
            cards_after_lesson: true,
            show_pron: true,
            prefer_voice: false,
            repeat_pauses: true,
            lessons: BTreeMap::new(),
            cards: BTreeMap::new(),
            days: BTreeMap::new(),
        }
    }
}

impl Saved {
    fn path() -> PathBuf {
        data_dir().join("progress.json")
    }

    pub fn load() -> Saved {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        write_atomic(&Self::path(), &text).map_err(|e| e.to_string())
    }

    pub fn day(&mut self) -> &mut DayLog {
        self.days.entry(day_key(today())).or_default()
    }

    /// Consecutive days with any study, ending today (or yesterday when
    /// today has nothing yet, so the streak isn't shown as broken mid-day).
    pub fn streak(&self) -> u32 {
        let studied = |d: NaiveDate| {
            self.days
                .get(&day_key(d))
                .is_some_and(|l| l.seconds >= 60.0 || l.lessons > 0 || l.reviews > 0)
        };
        let mut d = today();
        if !studied(d) {
            d -= Duration::days(1);
        }
        let mut n = 0;
        while studied(d) {
            n += 1;
            d -= Duration::days(1);
        }
        n
    }

    pub fn progress(&self, key: &str) -> Option<&LessonProgress> {
        self.lessons.get(key)
    }

    pub fn is_done(&self, key: &str) -> bool {
        self.lessons.get(key).is_some_and(|p| p.completed.is_some())
    }

    pub fn cards_due(&self) -> usize {
        let t = day_key(today());
        self.cards.values().filter(|c| c.due <= t).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_schedule_grows_and_resets() {
        let t = today();
        let mut c = Card::default();
        assert!(!c.grade(Grade::Good, t));
        assert_eq!(c.interval, 1);
        c.grade(Grade::Good, t);
        assert_eq!(c.interval, 3);
        c.grade(Grade::Good, t);
        assert_eq!(c.interval, 8); // 3 × 2.5 rounded
        assert!(c.grade(Grade::Again, t));
        assert_eq!(c.interval, 0);
        assert_eq!(c.due, day_key(t));
        assert!(c.ease < 2.5);
    }
}
