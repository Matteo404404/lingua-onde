//! Find your level: a short test of reading, listening, writing and speaking
//! built from the language's own lessons. It goes up from A1, stops at the
//! first level not passed, and starts the learner at the lesson that fits;
//! the lessons before it are marked as placed (passed by, not studied).

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui::{self, RichText};

use crate::app::{App, Screen};
use crate::audio::Player;
use crate::i18n::{tr, trf};
use crate::model::{Course, Item, Lesson, Level, lang_name};
use crate::practice::{Verdict, compare, fold};
use crate::store::{LevelTest, day_key, today};
use crate::theme::{self, ACCENT, ACCENT_SOFT, BAD, CARD, GOOD, INK, LINE, bold, caps, medium, muted};

use crate::speech;

/// Items per skill and level: six to eight a level, the whole test within
/// about ten minutes.
const PER_SKILL: usize = 2;
/// A level is passed with this share of the items that counted.
const PASS: f32 = 0.7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Skill {
    Reading,
    Listening,
    Writing,
    Speaking,
}

impl Skill {
    pub const ALL: [Skill; 4] = [Skill::Reading, Skill::Listening, Skill::Writing, Skill::Speaking];

    pub fn name(self) -> &'static str {
        match self {
            Skill::Reading => tr("Reading"),
            Skill::Listening => tr("Listening"),
            Skill::Writing => tr("Writing"),
            Skill::Speaking => tr("Speaking"),
        }
    }
}

/// The lessons a level takes in one course (`Stage::plan`), when that
/// course has lessons there.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Band {
    pub level: Level,
    pub course: usize,
    pub from: u32,
    pub to: u32,
}

/// Every band the courses `of` (one language's, in stage order) have
/// lessons in: Foundation A1 1–25 ... Mastery C2 26–100.
pub fn bands(courses: &[Course], of: &[usize]) -> Vec<Band> {
    let mut out = Vec::new();
    for &ci in of {
        let c = &courses[ci];
        let Some(stage) = c.stage else { continue };
        for level in Level::ALL {
            if let Some((from, to)) = stage.plan(level)
                && c.lessons.iter().any(|l| (from..=to).contains(&l.number))
            {
                out.push(Band { level, course: ci, from, to });
            }
        }
    }
    out
}

/// Where a learner who passed `passed` starts: the first band of a higher
/// level, else the last band there is.
pub fn start_band(bands: &[Band], passed: Option<Level>) -> Option<Band> {
    bands.iter().find(|b| passed.is_none_or(|p| b.level > p)).or(bands.last()).copied()
}

#[derive(Clone, Debug)]
pub enum Kind {
    /// A line, read or heard, and its meaning among four translations.
    Choice { text: String, options: Vec<String>, answer: usize },
    /// An exercise 2 sentence and its missing words.
    Blank(Item),
    /// A line to say aloud.
    Say(Item),
}

#[derive(Clone, Debug)]
pub struct Question {
    pub skill: Skill,
    pub kind: Kind,
}

/// The test: questions level by level, asked until a level is not passed.
pub struct Test {
    pub levels: Vec<Level>,
    questions: Vec<Vec<Question>>,
    /// The level being asked (index in `levels`) and the question in it.
    pub level: usize,
    pub at: usize,
    /// Each answered question's skill and score, by level; None: it did not count.
    scores: Vec<Vec<(Skill, Option<f32>)>>,
    /// Skills that do not count: listening without a voice, speaking without
    /// the speech module.
    pub off: Vec<Skill>,
    pub done: bool,
}

impl Test {
    pub fn new(courses: &[Course], bands: &[Band], base: &str, seed: u64, speaking: bool) -> Test {
        let mut rng = Rng(seed | 1);
        let mut levels: Vec<Level> = bands.iter().map(|b| b.level).collect();
        levels.sort();
        levels.dedup();
        let questions: Vec<Vec<Question>> = levels
            .iter()
            .map(|&level| {
                let lessons: Vec<&Lesson> = bands
                    .iter()
                    .filter(|b| b.level == level)
                    .flat_map(|b| courses[b.course].lessons.iter().filter(move |l| !l.revision && (b.from..=b.to).contains(&l.number)))
                    .collect();
                level_questions(&lessons, base, &mut rng, speaking)
            })
            .collect();
        let mut test = Test {
            scores: vec![Vec::new(); levels.len()],
            levels,
            questions,
            level: 0,
            at: 0,
            off: if speaking { Vec::new() } else { vec![Skill::Speaking] },
            done: false,
        };
        test.settle();
        test
    }

    pub fn current(&self) -> Option<&Question> {
        if self.done {
            return None;
        }
        self.questions.get(self.level)?.get(self.at)
    }

    /// (this question's number, questions) in the level being asked.
    pub fn progress(&self) -> (usize, usize) {
        let n = self.questions.get(self.level).map_or(0, Vec::len);
        (self.at + 1, n)
    }

    /// The score of the current question (0 to 1), then the next one.
    pub fn answer(&mut self, score: f32) {
        if let Some(skill) = self.current().map(|q| q.skill) {
            self.scores[self.level].push((skill, Some(score.clamp(0.0, 1.0))));
            self.at += 1;
            self.settle();
        }
    }

    /// From now on this skill's questions are left out and do not count.
    pub fn turn_off(&mut self, skill: Skill) {
        if !self.off.contains(&skill) {
            self.off.push(skill);
        }
        self.settle();
    }

    /// Passes over questions that do not count; at the end of a level, goes
    /// up to the next one if it was passed, else ends the test.
    fn settle(&mut self) {
        while !self.done {
            match self.current().map(|q| q.skill) {
                Some(skill) if self.off.contains(&skill) => {
                    self.scores[self.level].push((skill, None));
                    self.at += 1;
                }
                Some(_) => break,
                None if self.scores.get(self.level).is_some_and(|s| passed(s)) && self.level + 1 < self.levels.len() => {
                    self.level += 1;
                    self.at = 0;
                }
                None => self.done = true,
            }
        }
    }

    /// The highest level passed, with every level below it.
    pub fn reached(&self) -> Option<Level> {
        self.reached_by(|_| true)
    }

    /// The same for one skill; None when it never counted.
    pub fn skill_reached(&self, skill: Skill) -> Option<Level> {
        self.reached_by(|s| s == skill)
    }

    fn reached_by(&self, which: impl Fn(Skill) -> bool) -> Option<Level> {
        let mut out = None;
        for (i, scores) in self.scores.iter().enumerate() {
            let mine: Vec<(Skill, Option<f32>)> = scores.iter().filter(|(s, _)| which(*s)).copied().collect();
            if !passed(&mine) {
                break;
            }
            out = Some(self.levels[i]);
        }
        out
    }

    /// What the voice will say in the listening questions.
    fn heard(&self) -> Vec<String> {
        self.questions
            .iter()
            .flatten()
            .filter(|q| q.skill == Skill::Listening)
            .filter_map(|q| match &q.kind {
                Kind::Choice { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }
}

/// At least `PASS` of the scores that counted (and some counted).
fn passed(scores: &[(Skill, Option<f32>)]) -> bool {
    let counted: Vec<f32> = scores.iter().filter_map(|(_, s)| *s).collect();
    !counted.is_empty() && counted.iter().sum::<f32>() >= PASS * counted.len() as f32 - 1e-4
}

/// One level's questions from its lessons: reading and listening from the
/// dialogues, writing from exercise 2, speaking from the dialogues.
fn level_questions(lessons: &[&Lesson], base: &str, rng: &mut Rng, speaking: bool) -> Vec<Question> {
    // Lines with their meaning in the learner's language, not too short to
    // tell apart, not too long for one screen.
    let lines: Vec<(&Item, String)> = lessons
        .iter()
        .flat_map(|l| &l.dialogue)
        .filter_map(|it| {
            let tr = clean(it.tr.get(base)?);
            ((12..=140).contains(&it.plain().chars().count()) && !tr.is_empty()).then_some((it, tr))
        })
        .collect();
    let mut order: Vec<usize> = (0..lines.len()).collect();
    rng.shuffle(&mut order);
    let mut order = order.into_iter();
    let mut out = Vec::new();
    for skill in [Skill::Reading, Skill::Listening] {
        for _ in 0..PER_SKILL {
            if let Some(kind) = order.by_ref().find_map(|i| choice(&lines, i, rng)) {
                out.push(Question { skill, kind });
            }
        }
    }
    let mut blanks: Vec<&Item> = lessons
        .iter()
        .flat_map(|l| &l.ex2)
        .filter(|it| it.tr.contains_key(base) && (1..=3).contains(&it.blanks().len()))
        .collect();
    rng.shuffle(&mut blanks);
    out.extend(blanks.into_iter().take(PER_SKILL).map(|it| Question {
        skill: Skill::Writing,
        kind: Kind::Blank(it.clone()),
    }));
    if speaking {
        out.extend(order.filter(|&i| lines[i].0.plain().chars().count() <= 80).take(PER_SKILL).map(|i| Question {
            skill: Skill::Speaking,
            kind: Kind::Say(lines[i].0.clone()),
        }));
    }
    out
}

/// Line `i` and its meaning among three others of about its length.
fn choice(lines: &[(&Item, String)], i: usize, rng: &mut Rng) -> Option<Kind> {
    let (item, answer) = &lines[i];
    let mut seen = HashSet::from([fold(answer)]);
    let mut others: Vec<&String> = lines.iter().map(|(_, t)| t).filter(|t| seen.insert(fold(t))).collect();
    others.sort_by_key(|t| t.chars().count().abs_diff(answer.chars().count()));
    others.truncate(8);
    if others.len() < 3 {
        return None;
    }
    rng.shuffle(&mut others);
    let mut options: Vec<String> = others[..3].iter().map(|t| t.to_string()).collect();
    let at = rng.below(4);
    options.insert(at, answer.clone());
    Some(Kind::Choice {
        text: item.plain(),
        options,
        answer: at,
    })
}

/// A translation without its literal sense in brackets and the [ ] around
/// words the original leaves out.
fn clean(tr: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in tr.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '[' | ']' => {}
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    let out = out.split_whitespace().collect::<Vec<_>>().join(" ");
    [",", ".", "?", "!", ":", ";"].iter().fold(out, |s, p| s.replace(&format!(" {p}"), p))
}

/// Each missing word right, or right but for an accent (half), over the words.
fn blank_score(item: &Item, typed: &[String]) -> f32 {
    let blanks = item.blanks();
    let sum: f32 = blanks
        .iter()
        .enumerate()
        .map(|(k, b)| match compare(typed.get(k).map_or("", String::as_str), b).verdict {
            Verdict::Right => 1.0,
            Verdict::Accents => 0.5,
            Verdict::Wrong => 0.0,
        })
        .sum();
    sum / blanks.len().max(1) as f32
}

/// A small random generator (xorshift): the test's lines change from one
/// try to the next.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i + 1);
            v.swap(i, j);
        }
    }
}

// ------------------------------------------------------------------ screen

/// The level test screen: the start choice, one question at a time, the result.
#[derive(Default)]
pub struct LevelState {
    pub lang: String,
    pub test: Option<Test>,
    choice: Option<usize>,
    typed: Vec<String>,
    /// The question last played aloud, as (level, question).
    played: Option<(usize, usize)>,
    attempt: Option<speech::Attempt>,
}

/// What the learner asked for on the screen.
enum Act {
    None,
    Back,
    First(Option<Level>),
    Find,
    Stop,
    Answer(f32),
    SkipListening,
    Start(usize, usize, Option<Level>),
}

impl App {
    /// The built-in courses of a language written for the learner's
    /// language and with lessons, in order.
    fn level_courses(&self, lang: &str) -> Vec<usize> {
        let base = self.base();
        self.stages_of(lang)
            .into_iter()
            .filter(|&i| self.courses[i].speaks(&base) && !self.courses[i].lessons.is_empty())
            .collect()
    }

    pub fn level_bands(&self, lang: &str) -> Vec<Band> {
        bands(&self.courses, &self.level_courses(lang))
    }

    /// A built-in language with nothing done in it yet: starting it offers
    /// the first lesson or the level test.
    pub fn offer_placement(&self, ci: usize) -> bool {
        let c = &self.courses[ci];
        let ids: Vec<&str> = self.stages_of(&c.lang).iter().map(|&i| self.courses[i].id.as_str()).collect();
        c.builtin
            && !self.saved.level_tests.contains_key(&c.lang)
            && !self.saved.lessons.keys().any(|k| k.split_once('#').is_some_and(|(id, _)| ids.contains(&id)))
            && !self.level_bands(&c.lang).is_empty()
    }

    /// The start choice for `lang`: its first lesson, or the level test.
    pub fn open_placement(&mut self, lang: &str) {
        self.placement = Some(LevelState {
            lang: lang.to_string(),
            ..Default::default()
        });
        self.go(Screen::Placement);
    }

    fn unplace(&mut self, lang: &str) {
        let ids: Vec<String> = self.stages_of(lang).iter().map(|&i| self.courses[i].id.clone()).collect();
        self.saved.unplace(&ids);
    }

    /// The first lesson chosen: the last test's marks come off and lesson 1
    /// (or the introduction before it) comes next.
    pub(crate) fn first_lesson(&mut self, lang: &str, level: Option<Level>) {
        self.unplace(lang);
        let day = day_key(today());
        self.saved.level_tests.insert(lang.to_string(), LevelTest { day, level, start: None });
        // The first course with a lesson still to study: someone further on keeps their place.
        let courses = self.level_courses(lang);
        if let Some(ci) = courses.iter().copied().find(|&ci| self.next_lesson(ci).is_some()).or(courses.first().copied()) {
            self.saved.active_course = self.courses[ci].id.clone();
        }
        self.placement = None;
        self.mark_dirty();
        self.go(Screen::Study);
    }

    /// Starts the learner at lesson `li` of course `ci`: every lesson of the
    /// language before it is placed, none is marked as studied.
    pub fn place_at(&mut self, ci: usize, li: usize, level: Option<Level>) {
        let lang = self.courses[ci].lang.clone();
        self.unplace(&lang);
        let stage = self.courses[ci].stage.map(|s| s as u8);
        let mut keys = Vec::new();
        for si in self.stages_of(&lang) {
            let c = &self.courses[si];
            let before = if si == ci {
                li
            } else if c.stage.map(|s| s as u8) < stage {
                c.lessons.len()
            } else {
                0
            };
            keys.extend(c.lessons[..before].iter().map(|l| c.lesson_key(l.number)));
        }
        let day = day_key(today());
        self.saved.place(&keys, &day);
        let start = Some(self.courses[ci].lesson_key(self.courses[ci].lessons[li].number));
        self.saved.level_tests.insert(lang, LevelTest { day, level, start });
        self.saved.active_course = self.courses[ci].id.clone();
        // The introduction belongs before lesson 1; the book still has it.
        self.saved.read_intro.insert(self.courses[ci].id.clone());
        self.mark_dirty();
    }

    /// A new test, with new lines, for the language on the screen.
    pub fn start_level_test(&mut self) {
        let Some(lang) = self.placement.as_ref().map(|st| st.lang.clone()) else { return };
        let bands = self.level_bands(&lang);
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
        let test = Test::new(&self.courses, &bands, &self.base(), seed, speech::available(&lang));
        // The voice gets the listening lines ready while the reading ones are asked.
        self.voices.prepare(&lang, &test.heard());
        self.placement = Some(LevelState {
            lang,
            test: Some(test),
            ..Default::default()
        });
    }

    /// The course and lesson a band starts at.
    fn band_start(&self, b: Band) -> Option<(usize, usize)> {
        let li = self.courses[b.course].lessons.iter().position(|l| l.number >= b.from)?;
        Some((b.course, li))
    }

    pub fn placement_screen(&mut self, ui: &mut egui::Ui) {
        let Some(mut st) = self.placement.take() else {
            self.screen = Screen::Home;
            return;
        };
        let act = theme::page(ui, 760.0, |ui| match st.test.as_ref().map(|t| t.done) {
            None => self.level_choice(ui, &st),
            Some(false) => self.level_question(ui, &mut st),
            Some(true) => self.level_result(ui, &st),
        });
        let lang = st.lang.clone();
        self.placement = Some(st);
        let reset = |st: &mut LevelState| {
            st.choice = None;
            st.typed.clear();
            st.attempt = None;
        };
        match act {
            Act::None => {}
            Act::Back => {
                self.placement = None;
                self.go(Screen::Home);
            }
            Act::First(level) => self.first_lesson(&lang, level),
            Act::Find => self.start_level_test(),
            Act::Stop => {
                self.stop_audio();
                if let Some(st) = self.placement.as_mut() {
                    st.test = None;
                }
            }
            Act::Answer(score) => {
                self.stop_audio();
                if let Some(st) = self.placement.as_mut() {
                    reset(st);
                    if let Some(t) = st.test.as_mut() {
                        t.answer(score);
                    }
                }
            }
            Act::SkipListening => {
                if let Some(st) = self.placement.as_mut() {
                    reset(st);
                    if let Some(t) = st.test.as_mut() {
                        t.turn_off(Skill::Listening);
                    }
                }
            }
            Act::Start(ci, li, level) => {
                self.place_at(ci, li, level);
                self.placement = None;
                self.start_session(ci, li);
            }
        }
    }

    /// The two ways to start a language.
    fn level_choice(&mut self, ui: &mut egui::Ui, st: &LevelState) -> Act {
        let base = self.base();
        let mut act = Act::None;
        if ui.add(theme::ghost(tr("‹  Back"))).clicked() {
            act = Act::Back;
        }
        theme::page_title(
            ui,
            &trf("{0}: where do you start?", &[&lang_name(&st.lang, &base)]),
            tr("From the very first lesson, or from the lesson that fits what you already know. You can take the test again any time from Courses."),
        );
        let card = |ui: &mut egui::Ui, icon: &str, title: &str, text: &str, button: egui::Button<'static>| -> bool {
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                theme::card()
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_min_height(190.0);
                        theme::icon_circle(ui, icon, ACCENT(), ACCENT_SOFT(), 46.0);
                        ui.label(bold(title).size(19.0));
                        ui.label(muted(text).size(14.5));
                        ui.add_space(8.0);
                        ui.add(button).clicked()
                    })
                    .inner
            })
            .inner
        };
        let first = |ui: &mut egui::Ui| {
            card(
                ui,
                "▶",
                tr("Start from the first lesson"),
                tr("The course takes you from your very first words, one lesson a day."),
                theme::secondary(tr("Start from the first lesson")),
            )
        };
        let find = |ui: &mut egui::Ui| {
            card(
                ui,
                "🎯",
                tr("Find your level"),
                tr("About ten minutes of reading, listening, writing and speaking, from A1 up. It stops at the first level that is too hard, and you start at the lesson that fits."),
                theme::primary(tr("Find your level")),
            )
        };
        let (mut a, mut b) = (false, false);
        if ui.available_width() >= 600.0 {
            ui.columns(2, |cols| {
                a = first(&mut cols[0]);
                b = find(&mut cols[1]);
            });
        } else {
            a = first(ui);
            b = find(ui);
        }
        let ids: Vec<String> = self.stages_of(&st.lang).iter().map(|&i| self.courses[i].id.clone()).collect();
        let placed = self
            .saved
            .lessons
            .iter()
            .filter(|(k, p)| p.placed.is_some() && k.split_once('#').is_some_and(|(c, _)| ids.iter().any(|i| i == c)))
            .count();
        if placed > 0 {
            ui.add_space(6.0);
            ui.label(muted(trf("Lessons skipped by your last level test: {0}. Choosing the first lesson takes them back; what you studied stays.", &[&placed])).size(13.5));
        }
        if a {
            act = Act::First(None);
        }
        if b {
            act = Act::Find;
        }
        act
    }

    /// One question, with the level and how far it is.
    fn level_question(&mut self, ui: &mut egui::Ui, st: &mut LevelState) -> Act {
        let base = self.base();
        let lang = st.lang.clone();
        let Some(test) = st.test.as_ref() else { return Act::None };
        let Some(q) = test.current().cloned() else { return Act::None };
        let (k, n) = test.progress();
        let at = (test.level, test.at);
        let levels = test.levels.clone();
        let mut act = Act::None;

        ui.horizontal(|ui| {
            ui.label(bold(tr("Find your level")).size(24.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(theme::ghost(tr("Stop the test"))).clicked() {
                    act = Act::Stop;
                }
            });
        });
        ui.horizontal(|ui| {
            for (i, l) in levels.iter().enumerate() {
                let color = match i.cmp(&at.0) {
                    std::cmp::Ordering::Less => GOOD(),
                    std::cmp::Ordering::Equal => l.color(),
                    std::cmp::Ordering::Greater => LINE(),
                };
                theme::badge(ui, l.name(), color);
            }
            ui.label(muted(trf("Level {0} · question {1} of {2}", &[&levels[at.0].name(), &k, &n])));
        });
        theme::bar(ui, (k - 1) as f32 / n.max(1) as f32, ui.available_width(), levels[at.0].color());
        ui.add_space(10.0);

        // Listening needs a voice and a player; without them it can only be skipped.
        let no_sound = q.skill == Skill::Listening && !(Player::available() && self.voices.ready(&lang));
        let mut score = None;
        theme::card().inner_margin(22).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(caps(q.skill.name()));
            ui.add_space(2.0);
            match &q.kind {
                Kind::Choice { text, options, answer } => {
                    if q.skill == Skill::Listening {
                        ui.label(medium(tr("Listen, then choose what it means.")).size(17.0));
                        if !Player::available() {
                            ui.label(RichText::new(tr("No player found: install mpv to hear audio")).color(BAD()));
                        } else if !self.voices.ready(&lang) {
                            ui.label(muted(tr("The listening questions need the voice of this language.")).size(14.5));
                            self.get_voice(ui, &lang);
                        } else {
                            if st.played != Some(at) {
                                st.played = Some(at);
                                self.play_seq("level-test", &lang, vec![text.clone()], 0, true);
                            }
                            let playing = self.current("level-test").is_some();
                            ui.horizontal(|ui| {
                                let glyph = if playing { "🔊" } else { "▶" };
                                if theme::round_button(ui, glyph, 46.0, ACCENT()).on_hover_text(tr("Listen again")).clicked() {
                                    self.play_seq("level-test", &lang, vec![text.clone()], 0, true);
                                }
                                ui.label(muted(if playing { tr("🔊 listen…") } else { tr("Listen again") }));
                            });
                        }
                        if no_sound && ui.add(theme::secondary(tr("Skip the listening questions"))).clicked() {
                            act = Act::SkipListening;
                        }
                    } else {
                        ui.label(medium(tr("What does it mean?")).size(17.0));
                        ui.label(RichText::new(text).size(22.0));
                    }
                    if !no_sound {
                        ui.add_space(8.0);
                        for (i, o) in options.iter().enumerate() {
                            let on = st.choice == Some(i);
                            let b = egui::Button::new(RichText::new(o).size(16.0).color(INK()))
                                .wrap()
                                .fill(if on { ACCENT_SOFT() } else { CARD() })
                                .stroke(egui::Stroke::new(if on { 2.0 } else { 1.0 }, if on { ACCENT() } else { LINE() }))
                                .corner_radius(12)
                                .min_size(egui::vec2(ui.available_width(), 44.0));
                            if ui.add(b).clicked() {
                                st.choice = Some(i);
                            }
                        }
                        score = st.choice.map(|c| if c == *answer { 1.0 } else { 0.0 });
                    }
                }
                Kind::Blank(item) => {
                    ui.label(medium(tr("Write the missing words.")).size(17.0));
                    ui.label(muted(clean(item.tr(&base))).size(15.5));
                    let blanks = item.blanks().len();
                    if st.typed.len() != blanks {
                        st.typed = vec![String::new(); blanks];
                    }
                    ui.horizontal_wrapped(|ui| {
                        let mut b = 0;
                        for (text, blank) in item.parts() {
                            if !blank {
                                ui.label(RichText::new(text).size(19.0));
                                continue;
                            }
                            // As in exercise 2: one dot for each missing letter.
                            let dots = "·".repeat(text.chars().filter(|c| c.is_alphanumeric()).count());
                            let width = (text.chars().count() as f32 * 12.0 + 34.0).max(70.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut st.typed[b])
                                    .hint_text(dots)
                                    .desired_width(width)
                                    .font(egui::FontId::proportional(18.0))
                                    .margin(egui::Margin::symmetric(8, 4)),
                            );
                            b += 1;
                        }
                    });
                    if st.typed.iter().any(|t| !t.trim().is_empty()) {
                        score = Some(blank_score(item, &st.typed));
                    }
                }
                Kind::Say(item) => {
                    ui.label(medium(tr("Say the sentence aloud.")).size(17.0));
                    ui.label(RichText::new(item.plain()).size(22.0));
                    ui.label(muted(clean(item.tr(&base))).size(15.0));
                    ui.add_space(6.0);
                    // The Record button draws the heard words itself.
                    if let Some(a) = speech::check_button(ui, &lang, &item.plain()) {
                        st.attempt = Some(a);
                    }
                    if let Some(a) = &st.attempt {
                        score = Some(a.score);
                    }
                }
            }
        });
        ui.add_space(10.0);
        if !no_sound {
            ui.horizontal(|ui| {
                // A skipped question is one not known: it counts as wrong.
                if ui.add(theme::ghost(tr("Skip this item"))).clicked() {
                    act = Act::Answer(0.0);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add_enabled(score.is_some(), theme::big_primary(tr("Next  ›"))).clicked() {
                        act = Act::Answer(score.unwrap_or(0.0));
                    }
                });
            });
        }
        act
    }

    /// The level found, skill by skill, and the lesson to start at.
    fn level_result(&mut self, ui: &mut egui::Ui, st: &LevelState) -> Act {
        let base = self.base();
        let Some(test) = st.test.as_ref() else { return Act::None };
        let level = test.reached();
        let start = start_band(&self.level_bands(&st.lang), level).and_then(|b| self.band_start(b));
        let mut act = Act::None;
        ui.label(bold(tr("Find your level")).size(24.0));
        ui.add_space(6.0);
        theme::card().inner_margin(22).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(caps(tr("Your level")));
            ui.horizontal(|ui| match level {
                Some(l) => {
                    theme::badge(ui, l.name(), l.color());
                    ui.label(bold(l.describe()).size(24.0));
                }
                None => {
                    ui.label(bold(tr("A fresh start: the first lesson is the right one")).size(22.0));
                }
            });
            ui.add_space(6.0);
            let skills: Vec<String> = Skill::ALL
                .iter()
                .map(|&s| format!("{} {}", s.name(), test.skill_reached(s).map_or("—", |l| l.name())))
                .collect();
            ui.label(medium(skills.join("   ·   ")).size(16.0));
            if test.off.contains(&Skill::Speaking) && !speech::available(&st.lang) {
                ui.label(muted(tr("Speaking is not part of the test yet: its questions were left out.")).size(13.5));
            }
            if test.off.contains(&Skill::Listening) {
                ui.label(muted(tr("The listening questions were skipped: they do not count.")).size(13.5));
            }
            ui.add_space(10.0);
            if let Some((ci, li)) = start {
                let l = &self.courses[ci].lessons[li];
                ui.label(caps(tr("You start at")));
                ui.horizontal(|ui| {
                    theme::badge(ui, l.level.name(), l.level.color());
                    ui.label(bold(format!("{} · {}", self.course_label(ci), trf("Lesson {0}", &[&l.number]))).size(18.0));
                });
                ui.label(medium(format!("{} · {}", l.title, l.title_in(&base))));
                if li > 0 || self.level_courses(&st.lang).first() != Some(&ci) {
                    ui.label(
                        muted(tr("The lessons before it are marked as skipped, not as studied: they never come back in the active wave or in the reviews, and you can still open them. Taking the test again, or choosing the first lesson, takes the marks off."))
                            .size(13.5),
                    );
                }
            }
        });
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if let Some((ci, li)) = start
                && ui.add(theme::big_primary(tr("Start here"))).clicked()
            {
                act = Act::Start(ci, li, level);
            }
            if level.is_some() && ui.add(theme::secondary(tr("Start from the first lesson instead"))).clicked() {
                act = Act::First(level);
            }
        });
        act
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Stage;

    /// A built-in course of `stage` whose lessons `numbers` each have four
    /// dialogue lines and two exercise 2 sentences, with Italian translations.
    fn course(stage: Stage, numbers: std::ops::RangeInclusive<u32>) -> Course {
        let lessons = numbers
            .map(|n| Lesson {
                number: n,
                level: Level::ALL.into_iter().find(|&l| stage.plan(l).is_some_and(|(a, b)| (a..=b).contains(&n))).unwrap(),
                revision: n % 7 == 0,
                dialogue: (0..4)
                    .map(|k| Item {
                        text: format!("Line {k} of lesson {n} in {}", stage.code()),
                        tr: [("it".to_string(), format!("Riga {k} della lezione {n}{} (alla lettera)", "!".repeat(k)))].into(),
                        ..Default::default()
                    })
                    .collect(),
                ex2: (0..2)
                    .map(|k| Item {
                        text: format!("The [answer] {k} of lesson {n}."),
                        tr: [("it".to_string(), "La risposta.".to_string())].into(),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            })
            .collect();
        Course {
            id: stage.course_id("xx"),
            lang: "xx".into(),
            stage: Some(stage),
            lessons,
            builtin: true,
            ..Default::default()
        }
    }

    #[test]
    fn levels_map_to_lessons() {
        let all = vec![
            course(Stage::Foundation, 1..=100),
            course(Stage::Perfectionnement, 1..=70),
            course(Stage::Mastery, 1..=30),
        ];
        let b = bands(&all, &[0, 1, 2]);
        let got: Vec<(Level, usize, u32, u32)> = b.iter().map(|b| (b.level, b.course, b.from, b.to)).collect();
        use Level::*;
        assert_eq!(
            got,
            vec![
                (A1, 0, 1, 25),
                (A2, 0, 26, 50),
                (B1, 0, 51, 75),
                (B2, 0, 76, 100),
                (B2, 1, 1, 45),
                (C1, 1, 46, 70),
                (C1, 2, 1, 25),
                (C2, 2, 26, 100),
            ]
        );
        let start = |passed| start_band(&b, passed).map(|b| (b.course, b.from));
        assert_eq!(start(None), Some((0, 1)), "nothing passed: lesson 1");
        assert_eq!(start(Some(A2)), Some((0, 51)));
        assert_eq!(start(Some(B2)), Some((1, 46)), "B2 passed: Perfectionnement 46");
        assert_eq!(start(Some(C1)), Some((2, 26)), "C1 passed: Mastery 26");
        assert_eq!(start(Some(C2)), Some((2, 26)), "the top: the last band");
        // A language with only Foundation: past B2 there is nowhere higher.
        let one = bands(&all, &[0]);
        assert_eq!(start_band(&one, Some(B2)).map(|b| b.from), Some(76));
        // A course whose lessons stop at 60 has no B2 band; one with none, no band.
        let short = vec![course(Stage::Foundation, 1..=60), Course::default()];
        assert_eq!(bands(&short, &[0, 1]).last().map(|b| (b.level, b.from)), Some((B1, 51)));
    }

    #[test]
    fn test_goes_up_until_a_level_is_not_passed() {
        let all = vec![course(Stage::Foundation, 1..=100)];
        let b = bands(&all, &[0]);
        let mut t = Test::new(&all, &b, "it", 7, false);
        assert_eq!(t.levels, vec![Level::A1, Level::A2, Level::B1, Level::B2]);
        assert_eq!(t.progress(), (1, 6), "two reading, two listening, two writing; no speaking yet");
        // Every question comes from its level's lessons, none from a revision lesson.
        for q in &t.questions[1] {
            let text = match &q.kind {
                Kind::Choice { text, options, answer } => {
                    assert_eq!(options.len(), 4);
                    assert!(!options[*answer].contains('('), "the literal sense is left out");
                    text.clone()
                }
                Kind::Blank(it) | Kind::Say(it) => it.plain(),
            };
            let n: u32 = text.split_whitespace().find_map(|w| w.trim_end_matches('.').parse().ok().filter(|n| *n > 3)).unwrap();
            assert!((26..=50).contains(&n) && n % 7 != 0, "{text}");
        }
        // A1 and A2 right, B1 at 4 out of 6 (67%): the test stops at B1.
        let mut asked = 0;
        while t.current().is_some() {
            asked += 1;
            t.answer(if t.level < 2 || t.at < 4 { 1.0 } else { 0.0 });
        }
        assert!(t.done);
        assert_eq!(asked, 18, "three levels asked, B2 never");
        assert_eq!(t.reached(), Some(Level::A2));
        assert_eq!(t.skill_reached(Skill::Reading), Some(Level::B1));
        assert_eq!(t.skill_reached(Skill::Writing), Some(Level::A2));
        assert_eq!(t.skill_reached(Skill::Speaking), None);

        // Without a voice, listening is left out and the rest decides.
        let mut t = Test::new(&all, &b, "it", 9, false);
        t.answer(1.0);
        t.answer(1.0);
        t.turn_off(Skill::Listening);
        assert_eq!(t.current().map(|q| q.skill), Some(Skill::Writing));
        t.answer(1.0);
        t.answer(0.5);
        assert_eq!((t.level, t.at), (1, 0), "3.5 of 4 counted: A1 passed");
        assert_eq!(t.skill_reached(Skill::Listening), None);
        // A level answered all wrong ends it.
        while t.current().is_some() {
            t.answer(0.0);
        }
        assert_eq!(t.reached(), Some(Level::A1));
    }

    #[test]
    fn placing_starts_the_learner_without_inventing_history() {
        // SAFETY: the only other reader of this variable is in ignored tests.
        unsafe { std::env::set_var("LINGUA_ONDE_HOME", std::env::temp_dir().join("lingua-onde-test-home")) };
        let mut app = App::new(&egui::Context::default(), None, None);
        app.saved = crate::store::Saved {
            base: "en".into(),
            ..Default::default()
        };
        let ci = app.course_index("core-es").unwrap();
        let key = |app: &App, c: usize, n: u32| app.courses[c].lesson_key(n);
        assert!(app.offer_placement(ci), "a new language offers the choice");
        let bands = app.level_bands("es");

        // Passed B2: Perfectionnement 46, everything before it placed.
        let (pc, pli) = app.band_start(start_band(&bands, Some(Level::B2)).unwrap()).unwrap();
        assert_eq!((app.courses[pc].id.as_str(), app.courses[pc].lessons[pli].number), ("core-es-perfectionnement", 46));
        app.place_at(pc, pli, Some(Level::B2));
        assert_eq!(app.saved.active_course, "core-es-perfectionnement");
        assert_eq!(app.next_lesson(pc), Some(pli));
        assert_eq!(app.next_lesson(ci), None, "Foundation is behind the learner");
        assert!(app.saved.is_passed(&key(&app, ci, 100)) && !app.saved.is_done(&key(&app, ci, 100)));
        assert!(app.wave2_pending(pc).is_empty() && app.wave2_left(pc) == 0, "no active wave for skipped lessons");
        assert!(!app.offer_placement(ci));

        // Taken again, passed A2: Foundation 51; Perfectionnement is free again.
        let (sc, li) = app.band_start(start_band(&bands, Some(Level::A2)).unwrap()).unwrap();
        assert_eq!((sc, app.courses[sc].lessons[li].number), (ci, 51));
        app.place_at(sc, li, Some(Level::A2));
        assert_eq!(app.next_lesson(ci), Some(li));
        assert_eq!(app.next_lesson(pc), Some(0));
        assert_eq!(app.done_count(ci), 0, "nothing counts as studied");
        assert!(app.saved.lessons.values().all(|p| p.completed.is_none() && p.wave2.is_none() && p.times == 0));
        assert_eq!(app.saved.level_tests["es"].start.as_deref(), Some("core-es#51"));

        // Lessons 51–100 studied: only they come back in the active wave.
        for n in 51..=100 {
            let k = key(&app, ci, n);
            app.saved.lessons.entry(k).or_default().completed = Some("2026-01-01".into());
        }
        let due = app.wave2_due(ci);
        assert!(!due.is_empty() && due.iter().all(|&i| app.courses[ci].lessons[i].number >= 51), "{due:?}");
        // Lesson 51's partner, lesson 2, was skipped: another studied one comes instead.
        assert!(app.wave2_for(ci, li).is_some_and(|(c, w)| c == ci && app.courses[ci].lessons[w].number > 51));
        assert!(app.wave2_open(ci).iter().all(|&i| app.courses[ci].lessons[i].number >= 51));

        // The first lesson instead: the marks come off, what was studied stays.
        app.unplace("es");
        assert_eq!(app.next_lesson(ci), Some(0));
        assert_eq!(app.done_count(ci), 50);
        assert!(app.saved.lessons.values().all(|p| p.placed.is_none()));
    }

    /// Every screen of the test is drawn once, off screen: the choice, each
    /// question, the result.
    #[test]
    fn test_screens_draw() {
        // SAFETY: the only other reader of this variable is in ignored tests.
        unsafe { std::env::set_var("LINGUA_ONDE_HOME", std::env::temp_dir().join("lingua-onde-test-home")) };
        let ctx = egui::Context::default();
        theme::setup(&ctx);
        let mut app = App::new(&ctx, None, None);
        app.saved = crate::store::Saved {
            base: "en".into(),
            ..Default::default()
        };
        let draw = |app: &mut App| {
            let _ = ctx.run_ui(Default::default(), |ui| app.placement_screen(ui));
        };
        app.open_placement("es");
        draw(&mut app);
        app.start_level_test();
        let mut asked = 0;
        while app.placement.as_ref().and_then(|s| s.test.as_ref()).is_some_and(|t| !t.done) {
            draw(&mut app);
            let t = app.placement.as_mut().and_then(|s| s.test.as_mut()).unwrap();
            t.answer(if t.level < 2 { 1.0 } else { 0.0 });
            asked += 1;
        }
        draw(&mut app);
        // three levels asked (A1, A2 passed, B1 not), speaking only where the engine is installed
        let skills = if speech::available("es") { 4 } else { 3 };
        assert_eq!(asked, 3 * PER_SKILL * skills);
        assert!(app.placement.is_some(), "the result waits for the learner's choice");
        app.stop_audio();
    }

    #[test]
    fn written_answers_need_their_accents() {
        let it = Item {
            text: "Está [aquí] [mismo].".into(),
            ..Default::default()
        };
        let s = |a: &str, b: &str| blank_score(&it, &[a.to_string(), b.to_string()]);
        assert_eq!(s(" Aquí ", "mismo."), 1.0);
        assert_eq!(s("aqui", "mismo"), 0.75, "a missing accent is half right");
        assert_eq!(s("alli", ""), 0.0);
        assert_eq!(clean("Ecco a lei (qui lei è). Una [bella] tazza ."), "Ecco a lei. Una bella tazza.");
    }
}
