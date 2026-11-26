//! Where things live on disk: the built-in courses, the person's own courses,
//! and their progress (one JSON file).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::format::{parse_course, write_course};
use crate::model::{Course, Stage};

/// The layout of the progress file; older files are converted on load.
const PROGRESS_VERSION: u32 = 1;

/// `content/<lang>/<stage>-<level>.txt` for each language: the three
/// courses (Foundation, Perfectionnement, Mastery), each split by level.
macro_rules! files {
    ($($lang:literal),*) => {
        &[$(
            files!(@one $lang, "foundation-a1"),
            files!(@one $lang, "foundation-a2"),
            files!(@one $lang, "foundation-b1"),
            files!(@one $lang, "foundation-b2"),
            files!(@one $lang, "perfectionnement-b2"),
            files!(@one $lang, "perfectionnement-c1"),
            files!(@one $lang, "mastery-c1"),
            files!(@one $lang, "mastery-c2"),
        )*]
    };
    (@one $lang:literal, $file:literal) => {
        (concat!($lang, "/", $file, ".txt"), include_str!(concat!("../content/", $lang, "/", $file, ".txt")))
    };
}

/// Built-in lesson files. Files with the same `course:` and `stage:` lines
/// add lessons to one course; the first file of a course names it. Courses
/// appear in this order.
pub const BUILTIN: &[(&str, &str)] = files!("en", "es", "fr", "de", "nl", "pt", "zh", "la", "vi");

/// The key to a language's figured pronunciation, where one exists.
const PRONUNCIATION: &[(&str, &str)] = &[
    ("en", include_str!("../content/en/pronunciation.md")),
    ("es", include_str!("../content/es/pronunciation.md")),
    ("nl", include_str!("../content/nl/pronunciation.md")),
];

pub fn pronunciation_key(lang: &str) -> Option<&'static str> {
    PRONUNCIATION.iter().find(|(l, _)| *l == lang).map(|(_, k)| *k)
}

/// The built-in courses, three per language, merged from their files, and
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
        let Some(stage) = c.stage else {
            problems.push(format!("built-in {file}: no `stage:` line"));
            continue;
        };
        match out.iter_mut().find(|o| o.lang == c.lang && o.stage == c.stage) {
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
                id: stage.course_id(&c.lang),
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
    write_atomic(&path, &write_course(c)).map_err(|e| crate::i18n::trf("could not save {0}: {1}", &[&path.display(), &e]))?;
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
    /// The learner chose to go over this lesson again the next day.
    pub again: bool,
    /// Day the lesson was last finished, first time or repeat.
    pub last: Option<String>,
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
    /// The layout this file follows (0: before the three courses, v1.0.0).
    #[serde(default)]
    pub version: u32,
    /// A message for the learner about the file itself, shown once.
    #[serde(skip)]
    pub notice: Option<String>,
    /// The learner's own language: translations and notes are shown in it.
    pub base: String,
    pub active_course: String,
    pub audio_dirs: BTreeMap<String, String>,
    pub speed: f32,
    /// The voice is slower in Foundation's first lessons and reaches the
    /// natural pace at lesson 50, as the books' recordings do.
    pub gradual_pace: bool,
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
    /// Courses whose introduction the learner has been through (or skipped),
    /// so lesson 1 no longer sends them to it first.
    pub read_intro: std::collections::BTreeSet<String>,
    /// "light", "dark", or anything else to follow the system.
    pub theme: String,
}

impl Default for Saved {
    fn default() -> Self {
        Saved {
            version: PROGRESS_VERSION,
            notice: None,
            base: "it".into(),
            active_course: "core-en".into(),
            audio_dirs: BTreeMap::new(),
            speed: 1.0,
            gradual_pace: true,
            pause_factor: 1.3,
            wave2_gap: 49,
            daily_goal_min: 20,
            // Not part of the Assimil method: the learner can turn it on.
            cards_after_lesson: false,
            show_pron: true,
            prefer_voice: false,
            repeat_pauses: true,
            lessons: BTreeMap::new(),
            cards: BTreeMap::new(),
            days: BTreeMap::new(),
            read_intro: Default::default(),
            theme: "system".into(),
        }
    }
}

impl Saved {
    fn path() -> PathBuf {
        data_dir().join("progress.json")
    }

    pub fn load() -> Saved {
        Self::load_from(&Self::path())
    }

    /// Progress survives updates: lessons, courses and languages are added
    /// around it, and its keys (course id # lesson number) never change.
    /// A file that cannot be read is set aside, never overwritten, and a
    /// copy of the last good file is kept next to it.
    fn load_from(path: &Path) -> Saved {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Saved::default();
        };
        match serde_json::from_str::<Saved>(&text) {
            Ok(mut s) => {
                let _ = std::fs::copy(path, path.with_extension("backup.json"));
                s.migrate();
                s
            }
            Err(e) => {
                let stamp = Local::now().format("%Y%m%d-%H%M%S");
                let aside = path.with_extension(format!("unreadable-{stamp}.json"));
                let _ = std::fs::rename(path, &aside);
                Saved {
                    notice: Some(crate::i18n::trf(
                        "Your progress file could not be read ({0}). It is kept as {1} and a new one was started.",
                        &[&e, &aside.display()],
                    )),
                    ..Saved::default()
                }
            }
        }
    }

    /// Converts a file written by an older version.
    fn migrate(&mut self) {
        if self.version < 1 {
            // Up to v1.0.0 each language was one course of 300 lessons:
            // 106–175 are now Perfectionnement 1–70, 176–300 Mastery 1–125 (Mastery was later cut to 100).
            let moved = |key: &str| -> Option<String> {
                let (course, rest) = key.split_once('#')?;
                let lang = course.strip_prefix("core-").filter(|l| !l.contains('-'))?;
                let (num, tail) = rest.split_once('#').map_or((rest, ""), |(n, t)| (n, t));
                let n: u32 = num.parse().ok()?;
                let (stage, m) = match n {
                    106..=175 => (Stage::Perfectionnement, n - 105),
                    176..=300 => (Stage::Mastery, n - 175),
                    _ => return None,
                };
                let tail = if tail.is_empty() { String::new() } else { format!("#{tail}") };
                Some(format!("{}#{m}{tail}", stage.course_id(lang)))
            };
            fn rekey<T>(map: &mut BTreeMap<String, T>, moved: &dyn Fn(&str) -> Option<String>) {
                let keys: Vec<String> = map.keys().filter(|k| moved(k).is_some()).cloned().collect();
                for k in keys {
                    if let (Some(v), Some(new)) = (map.remove(&k), moved(&k)) {
                        map.insert(new, v);
                    }
                }
            }
            rekey(&mut self.lessons, &moved);
            rekey(&mut self.cards, &moved);
        }
        self.version = PROGRESS_VERSION;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_survives_updates() {
        let dir = std::env::temp_dir().join(format!("lingua-onde-progress-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("progress.json");

        // A v1.0.0 file: one 300-lesson course per language, no version.
        std::fs::write(
            &path,
            r#"{"base":"it","active_course":"core-es","lessons":{"core-es#3":{"times":2},"core-es#151":{"times":1},"core-es#201":{"times":1}},"cards":{"core-es#151#0":{"reps":4}},"future_field":true}"#,
        )
        .unwrap();
        let s = Saved::load_from(&path);
        assert_eq!(s.version, PROGRESS_VERSION);
        assert_eq!(s.lessons["core-es#3"].times, 2);
        assert!(s.lessons.contains_key("core-es-perfectionnement#46"));
        assert!(s.lessons.contains_key("core-es-mastery#26"));
        assert_eq!(s.cards["core-es-perfectionnement#46#0"].reps, 4);
        assert!(dir.join("progress.backup.json").exists());

        // A current file is left as it is.
        let mut again = s.clone();
        again.migrate();
        assert_eq!(again.lessons.len(), 3);

        // A file that cannot be read is kept aside, never lost.
        std::fs::write(&path, "{ not json").unwrap();
        let fresh = Saved::load_from(&path);
        assert!(fresh.notice.is_some() && fresh.lessons.is_empty());
        let kept: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("unreadable"))
            .collect();
        assert_eq!(kept.len(), 1);
        assert_eq!(std::fs::read_to_string(kept[0].path()).unwrap(), "{ not json");
        let _ = std::fs::remove_dir_all(&dir);
    }

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
