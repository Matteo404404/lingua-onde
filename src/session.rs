//! The guided daily lesson (the listening and reading steps of the method),
//! the sentence-by-sentence voice with pauses for repeating, and the review
//! screen (spaced sentence review and the active "second wave").

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText};

use crate::app::{App, Screen};
use crate::i18n::{in_sentence, plural, tr, trf};
use crate::model::{Item, Lesson, lang_color, lang_name};
use crate::practice::{Practice, Verdict, compare};
use crate::store::{Card, Grade, day_key, today};
use crate::theme::{
    self, ACCENT, ACCENT_SOFT, BAD, CARD, GOOD, GOOD_SOFT, HIGHLIGHT, INK, MUTED, TRACK, WARN, WARN_SOFT, bold, caps, medium,
    muted,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Step {
    /// Revision lesson: the grammar of the six lessons before it.
    Recap,
    BlindListen,
    ListenBase,
    ListenTarget,
    Read,
    Notes,
    Ex1,
    Ex2,
    Tasks,
    ReadAgain,
    FinalListen,
    Previous,
    SecondWave,
}

impl Step {
    pub fn title(self) -> &'static str {
        match self {
            Step::Recap => tr("Revision · the grammar of the week"),
            Step::BlindListen => tr("Blind listening"),
            Step::ListenBase => tr("Listen, reading your language"),
            Step::ListenTarget => tr("Listen, reading the original"),
            Step::Read => tr("Read the dialogue"),
            Step::Notes => tr("Notes"),
            Step::Ex1 => tr("Exercise 1 · Translate"),
            Step::Ex2 => tr("Exercise 2 · Fill in the missing words"),
            Step::Tasks => tr("Open tasks"),
            Step::ReadAgain => tr("Read it again"),
            Step::FinalListen => tr("Last blind listening"),
            Step::Previous => tr("Review the two previous dialogues"),
            Step::SecondWave => tr("Active wave"),
        }
    }

    fn short(self) -> &'static str {
        match self {
            Step::Recap => tr("Grammar"),
            Step::BlindListen => tr("Blind"),
            Step::ListenBase => tr("+ your language"),
            Step::ListenTarget => tr("+ original"),
            Step::Read => tr("Read"),
            Step::Notes => tr("Notes"),
            Step::Ex1 => tr("Ex. 1"),
            Step::Ex2 => tr("Ex. 2"),
            Step::Tasks => tr("Tasks"),
            Step::ReadAgain => tr("Read again"),
            Step::FinalListen => tr("Blind again"),
            Step::Previous => tr("2 before"),
            Step::SecondWave => tr("Active"),
        }
    }

    fn how(self) -> &'static str {
        match self {
            Step::Recap => {
                tr("A revision lesson: nothing new. Read how the grammar of the last six lessons fits together, with examples you already know.")
            }
            Step::BlindListen => {
                tr("Don't look at any text. Listen to the whole dialogue, as on a recording. It's fine not to understand yet: you are tuning your ear. To repeat each sentence aloud, imitating the voice, tick “Pause to repeat”.")
            }
            Step::ListenBase => {
                tr("Listen again while you read the translation in your own language: the sound and the meaning start to connect. With “Pause to repeat” you say each sentence aloud after the voice.")
            }
            Step::ListenTarget => {
                tr("Now follow the original text while you listen. Notice how the words are written. With “Pause to repeat” you say each sentence aloud after the voice.")
            }
            Step::Read => {
                tr("Read the dialogue aloud, sentence by sentence, with the translation beside it. The literal sense in brackets shows how the language thinks. Click ▶ to hear a sentence.")
            }
            Step::Notes => {
                tr("Read the pronunciation observations and the notes. They explain what you just met; don't try to memorise anything, it settles by itself as you go on.")
            }
            Step::Ex1 => tr("Read each sentence aloud and translate it in your head, then check."),
            Step::Ex2 => tr("Write the missing words. The translation tells you what the sentence says."),
            Step::Tasks => {
                tr("Open tasks have no single right answer. Write or say yours first, then compare it with a model answer and tick the checklist honestly.")
            }
            Step::ReadAgain => {
                tr("Read the dialogue once more, aloud. Try hiding the translation: you should understand almost all of it now.")
            }
            Step::FinalListen => {
                tr("Last time, eyes off the text: listen to the whole dialogue. Notice how much more you understand than at the start.")
            }
            Step::Previous => {
                tr("Read and listen to the two lessons before this one, aloud. Old material comes back easily and keeps growing roots.")
            }
            Step::SecondWave => {
                tr("The active wave: an earlier lesson's dialogue and exercise 1, shown in your language only. Say each sentence aloud in the language you learn (type it if you like), then reveal it and compare. Mistakes are part of it.")
            }
        }
    }
}

pub struct Session {
    pub course: usize,
    pub lesson: usize,
    pub steps: Vec<Step>,
    pub step: usize,
    pub done: Vec<bool>,
    pub shown: HashSet<usize>,
    pub answers: Vec<Vec<String>>,
    pub checked: bool,
    pub score: (u32, u32),
    pub hide_tr: bool,
    /// An older lesson whose second wave is due today, as (course, lesson):
    /// it can belong to an earlier course of the same language.
    pub wave2: Option<(usize, usize)>,
    pub wave: Practice,
    pub tasks: Practice,
    /// "Go over this lesson again tomorrow", chosen at the end.
    pub again: bool,
    /// The note under the pointer and the note clicked, whose dialogue lines
    /// are lit; `note_seek` brings the lines into view once.
    pub note_hover: Option<usize>,
    pub note_pick: Option<usize>,
    pub note_seek: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Wait,
    Play(Instant),
    Gap(Instant, Duration),
    Stopped,
    Done,
}

/// Sentences spoken one after another by the voice. In prompt-and-answer
/// mode each sentence is preceded by its translation in the learner's
/// language, and the pause after the prompt is the learner's turn.
pub struct Seq {
    pub tag: String,
    /// What to say, in order: (language, text).
    pub parts: Vec<(String, String)>,
    /// Parts per sentence: 1, or 2 with a prompt before each sentence.
    pub stride: usize,
    pub idx: usize,
    pub phase: Phase,
    pub repeat: bool,
    pub single: bool,
    asked: usize,
    /// The sentences play as one recording, as the books' recordings do;
    /// `starts` are the seconds each starts at in it, from sentence `first`.
    pub whole: bool,
    first: usize,
    starts: Vec<f64>,
}

impl Seq {
    fn running(&self) -> bool {
        matches!(self.phase, Phase::Wait | Phase::Play(_) | Phase::Gap(..))
    }

    /// The sentence being spoken.
    pub fn sentence(&self) -> usize {
        self.idx / self.stride
    }

    pub fn sentences(&self) -> usize {
        self.parts.len() / self.stride
    }

    fn is_prompt(&self) -> bool {
        self.stride == 2 && self.idx.is_multiple_of(2)
    }
}

#[derive(Default)]
pub struct ReviewState {
    pub tab: usize,
    pub queue: Vec<String>,
    pub built: bool,
    pub shown: bool,
    pub practice: Option<(usize, usize)>,
    pub practice_state: Practice,
    pub search: String,
    /// The course whose active wave the tab lists (an earlier course of the
    /// language while its wave is still going on).
    pub wave_course: Option<usize>,
}

/// The lesson column's widest size: lines stay easy to read.
const STUDY_W: f32 = 900.0;

use theme::FIGURED;

fn texts(items: &[Item]) -> Vec<String> {
    items.iter().map(|i| i.plain()).collect()
}

/// The two nearest lessons before `li` that have a dialogue, oldest first.
fn previous_dialogues(c: &crate::model::Course, li: usize) -> Vec<usize> {
    let mut v: Vec<usize> = (0..li).rev().filter(|&j| !c.lessons[j].dialogue.is_empty()).take(2).collect();
    v.reverse();
    v
}

impl App {
    pub fn start_session(&mut self, ci: usize, li: usize) {
        self.stop_audio();
        let Some(lesson) = self.courses.get(ci).and_then(|c| c.lessons.get(li)).cloned() else {
            return;
        };
        let key = self.courses[ci].lesson_key(lesson.number);
        let wave2 = self.wave2_for(ci, li);
        let mut steps = lesson_steps(&lesson);
        let wave_text = wave2.map(|(wc, w)| texts(&self.courses[wc].lessons[w].dialogue));
        if wave2.is_some() {
            steps.push(Step::SecondWave);
        }
        let p = self.saved.progress(&key).cloned().unwrap_or_default();
        let mut done = vec![false; steps.len()];
        {
            for &s in &p.steps_done {
                if let Some(d) = done.get_mut(s) {
                    *d = true;
                }
            }
        }
        let step = done.iter().position(|d| !d).unwrap_or(0);
        let answers = lesson.ex2.iter().map(|i| vec![String::new(); i.blanks().len().max(1)]).collect();

        // Let the voice prepare everything this session will speak.
        let lang = self.courses[ci].lang.clone();
        let c = &self.courses[ci];
        let mut speak = texts(&lesson.dialogue);
        speak.extend(texts(&lesson.ex1));
        speak.extend(texts(&lesson.ex2));
        for j in previous_dialogues(c, li) {
            speak.extend(texts(&c.lessons[j].dialogue));
        }
        speak.extend(wave_text.into_iter().flatten());
        self.voices.set_speed(self.voice_speed_at(ci, li));
        self.voices.prepare(&lang, &speak);

        self.session = Some(Session {
            course: ci,
            lesson: li,
            steps,
            step,
            done,
            shown: HashSet::new(),
            answers,
            checked: false,
            score: (0, 0),
            hide_tr: false,
            wave2,
            wave: Practice::default(),
            tasks: Practice::default(),
            again: false,
            note_hover: None,
            note_pick: None,
            note_seek: false,
        });
        self.screen = Screen::Study;
    }

    fn set_step(&mut self, to: usize) {
        self.stop_audio();
        if let Some(s) = self.session.as_mut() {
            s.step = to.min(s.steps.len() - 1);
            s.shown.clear();
            s.hide_tr = s.steps[s.step] == Step::ReadAgain;
            (s.note_hover, s.note_pick) = (None, None);
        }
    }

    fn complete_step(&mut self) {
        let Some(s) = self.session.as_mut() else { return };
        s.done[s.step] = true;
        let steps_done: Vec<usize> = s.done.iter().enumerate().filter(|(_, d)| **d).map(|(i, _)| i).collect();
        let (ci, li, next) = (s.course, s.lesson, s.step + 1);
        let key = self.courses[ci].lesson_key(self.courses[ci].lessons[li].number);
        self.saved.lessons.entry(key).or_default().steps_done = steps_done;
        self.mark_dirty();
        if next >= self.session.as_ref().map(|s| s.steps.len()).unwrap_or(0) {
            self.finish_session();
        } else {
            self.set_step(next);
        }
    }

    fn finish_session(&mut self) {
        self.stop_audio();
        let Some(s) = self.session.take() else { return };
        let c = &self.courses[s.course];
        let lesson = &c.lessons[s.lesson];
        let key = c.lesson_key(lesson.number);
        let today = day_key(today());
        let p = self.saved.lessons.entry(key.clone()).or_default();
        let first = p.completed.is_none();
        if first {
            p.completed = Some(today.clone());
        }
        p.times += 1;
        p.steps_done.clear();
        p.again = s.again;
        p.last = Some(today.clone());
        if s.score.1 > 0 {
            p.score = s.score;
        }
        if self.saved.cards_after_lesson {
            for i in 0..lesson.dialogue.len() {
                self.saved.cards.entry(format!("{key}#{i}")).or_default();
            }
        }
        if let (Some((wc, w)), Some(wi)) = (s.wave2, s.steps.iter().position(|x| *x == Step::SecondWave)) {
            if s.done[wi] {
                let c = &self.courses[wc];
                let wl = &c.lessons[w];
                let wkey = c.lesson_key(wl.number);
                let wp = self.saved.lessons.entry(wkey.clone()).or_default();
                if wp.wave2.is_none() {
                    wp.wave2 = Some(today.clone());
                    if self.saved.cards_after_lesson {
                        for i in 0..wl.dialogue.len() {
                            self.saved.cards.entry(format!("{wkey}#{i}")).or_default();
                        }
                    }
                }
            }
        }
        self.saved.day().lessons += 1;
        let msg = if s.again {
            trf("Lesson {0} done. Tomorrow you'll go over it once more — that's how it settles.", &[&lesson.number])
        } else if first {
            trf("Lesson {0} done — well done! See you tomorrow for the next one.", &[&lesson.number])
        } else {
            trf("Lesson {0} done again. See you tomorrow for the next one.", &[&lesson.number])
        };
        self.mark_dirty();
        self.screen = Screen::Home;
        self.say(msg);
    }

    // ------------------------------------------------------------ audio

    pub(crate) fn play_seq(&mut self, tag: &str, lang: &str, texts: Vec<String>, from: usize, single: bool) {
        let parts = texts.into_iter().map(|t| (lang.to_string(), t)).collect();
        self.start_seq(tag, parts, 1, from, single);
    }

    /// Prompt and answer: each prompt (in `base`) is followed by the learner's
    /// turn, then the answer (in `lang`) and a pause to repeat it.
    pub(crate) fn play_pairs(&mut self, tag: &str, base: &str, prompts: Vec<String>, lang: &str, answers: Vec<String>) {
        self.voices.prepare(base, &prompts);
        let parts = prompts
            .into_iter()
            .zip(answers)
            .flat_map(|(p, a)| [(base.to_string(), p), (lang.to_string(), a)])
            .collect();
        self.start_seq(tag, parts, 2, 0, false);
    }

    /// What the learner's choice to pause after each sentence is kept under:
    /// the lesson step being studied, else the list itself.
    fn pause_key(&self, tag: &str) -> String {
        match self.session.as_ref().filter(|_| self.screen == Screen::Study) {
            Some(s) => format!("{:?}", s.steps[s.step]),
            None => tag.to_string(),
        }
    }

    fn start_seq(&mut self, tag: &str, parts: Vec<(String, String)>, stride: usize, from: usize, single: bool) {
        self.player.stop();
        if parts.is_empty() {
            self.seq = None;
            return;
        }
        let repeat = !single && self.saved.pause_steps.contains(&self.pause_key(tag));
        self.seq = Some(Seq {
            tag: tag.to_string(),
            idx: (from * stride).min(parts.len() - stride),
            parts,
            stride,
            phase: Phase::Wait,
            repeat,
            single,
            asked: usize::MAX,
            whole: stride == 1 && !single && !repeat,
            first: from,
            starts: Vec::new(),
        });
    }

    pub fn drive_audio(&mut self, ctx: &egui::Context) {
        let mut error = None;
        self.voices.set_speed(self.voice_speed());
        if let Some(seq) = self.seq.as_mut() {
            let now = Instant::now();
            match seq.phase {
                Phase::Wait if seq.whole => {
                    let lang = seq.parts[seq.idx].0.clone();
                    let texts: Vec<String> = seq.parts[seq.idx..].iter().map(|(_, t)| t.clone()).collect();
                    let joined = if self.voices.ready(&lang) { self.voices.joined(&lang, &texts) } else { Some(Err(String::new())) };
                    match joined {
                        None if seq.asked != seq.idx => {
                            seq.asked = seq.idx;
                            self.voices.prepare(&lang, &texts);
                        }
                        None => {}
                        Some(Ok((file, starts))) => match self.player.play(&file, 1.0) {
                            Ok(()) => {
                                (seq.first, seq.starts) = (seq.idx, starts);
                                seq.phase = Phase::Play(now);
                            }
                            Err(e) => {
                                error = Some(e);
                                seq.phase = Phase::Stopped;
                            }
                        },
                        Some(Err(e)) => {
                            error = Some(e).filter(|e| !e.is_empty());
                            seq.phase = Phase::Stopped;
                        }
                    }
                }
                Phase::Play(t0) if seq.whole => {
                    if self.player.busy() {
                        // mpv knows where it is, also after a pause; the others play straight through.
                        let at = if self.player.controllable { self.player.position().0 } else { t0.elapsed().as_secs_f64() };
                        seq.idx = seq.first + crate::audio::sentence_at(&seq.starts, at);
                    } else {
                        seq.phase = Phase::Done;
                    }
                }
                Phase::Wait => {
                    let (lang, text) = seq.parts[seq.idx].clone();
                    if !self.voices.ready(&lang) && seq.is_prompt() {
                        // No voice for the learner's language: the prompt is read on screen.
                        let turn = Duration::from_millis(1500 + 60 * seq.parts[seq.idx + 1].1.chars().count() as u64);
                        seq.phase = Phase::Gap(now + turn, turn);
                    } else if !self.voices.ready(&lang) {
                        seq.phase = Phase::Stopped;
                    } else if let Some(file) = self.voices.speech_file(&lang, &text) {
                        if file.is_file() {
                            match self.player.play(&file, 1.0) {
                                Ok(()) => seq.phase = Phase::Play(now),
                                Err(e) => {
                                    error = Some(e);
                                    seq.phase = Phase::Stopped;
                                }
                            }
                        } else if seq.asked != seq.idx {
                            seq.asked = seq.idx;
                            self.voices.prepare(&lang, &[text]);
                        }
                    }
                }
                Phase::Play(t0) => {
                    if !self.player.busy() {
                        let took = t0.elapsed();
                        seq.phase = if seq.single {
                            Phase::Done
                        } else if seq.is_prompt() {
                            // The learner's turn: say the sentence before hearing it.
                            let turn = took.mul_f32(self.saved.pause_factor + 0.5) + Duration::from_millis(1000);
                            Phase::Gap(now + turn, turn)
                        } else {
                            let gap = if seq.repeat || seq.stride == 2 {
                                took.mul_f32(self.saved.pause_factor) + Duration::from_millis(300)
                            } else {
                                Duration::from_millis(600)
                            };
                            Phase::Gap(now + gap, gap)
                        };
                    }
                }
                Phase::Gap(until, _) => {
                    if now >= until {
                        seq.idx += 1;
                        seq.phase = if seq.idx >= seq.parts.len() { Phase::Done } else { Phase::Wait };
                    }
                }
                Phase::Stopped | Phase::Done => {}
            }
            if seq.running() {
                ctx.request_repaint_after(Duration::from_millis(80));
            }
        } else if self.player.busy() {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        if let Some(e) = error {
            self.say(e);
        }
    }

    /// The sentence the voice is on, when it is speaking the list `tag`.
    pub(crate) fn current(&self, tag: &str) -> Option<usize> {
        self.seq.as_ref().filter(|s| s.tag == tag && s.running()).map(|s| s.sentence())
    }

    /// Play/pause controls for a list of sentences: the lesson recording when
    /// there is one, else the voice (the whole list in one go, or sentence by
    /// sentence with pauses), else how to get audio.
    pub(crate) fn listen_panel(&mut self, ui: &mut egui::Ui, tag: &str, lang: &str, items: &[Item], recording: Option<PathBuf>) {
        let voice = self.voices.ready(lang);
        theme::tinted(ACCENT_SOFT()).corner_radius(14).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let use_recording = recording.is_some() && !(self.saved.prefer_voice && voice);
            if let (true, Some(rec)) = (use_recording, recording) {
                let this = self.seq.is_none() && self.player.file.as_ref() == Some(&rec);
                let name = rec.file_name().unwrap_or_default().to_string_lossy().to_string();
                ui.horizontal(|ui| {
                    if this && self.player.busy() {
                        let glyph = if self.player.paused || !self.player.controllable {
                            "▶"
                        } else {
                            "⏸"
                        };
                        let glyph = if self.player.controllable { glyph } else { "■" };
                        if theme::round_button(ui, glyph, 46.0, ACCENT()).clicked() {
                            if self.player.controllable {
                                self.player.toggle_pause();
                            } else {
                                self.player.stop();
                            }
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.label(medium(format!("{name}")));
                            let (pos, dur) = self.player.position();
                            if dur > 0.0 {
                                ui.horizontal(|ui| {
                                    theme::bar_on(ui, (pos / dur) as f32, 220.0, ACCENT(), CARD());
                                    ui.label(muted(format!(
                                        "{}:{:02} / {}:{:02}",
                                        pos as u32 / 60,
                                        pos as u32 % 60,
                                        dur as u32 / 60,
                                        dur as u32 % 60
                                    )));
                                });
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(theme::ghost(tr("Stop"))).clicked() {
                                self.player.stop();
                            }
                            if self.player.controllable && ui.add(theme::ghost("⏪ 5 s")).clicked() {
                                self.player.seek(-5.0);
                            }
                        });
                    } else {
                        if theme::round_button(ui, "▶", 46.0, ACCENT())
                            .on_hover_text(tr("Play the recording"))
                            .clicked()
                        {
                            self.seq = None;
                            if let Err(e) = self.player.play(&rec, self.saved.speed) {
                                self.say(e);
                            }
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.label(medium(tr("Play the lesson recording")));
                            ui.label(muted(trf("{0} · pause after each sentence and repeat it aloud", &[&name])).size(13.5));
                        });
                    }
                });
                return;
            }
            if voice {
                let seq_state = self
                    .seq
                    .as_ref()
                    .filter(|s| s.tag == tag)
                    .map(|s| (s.phase, s.sentence(), s.sentences(), s.whole, s.repeat));
                let running = matches!(seq_state, Some((p, ..)) if !matches!(p, Phase::Stopped | Phase::Done));
                // One recording under mpv can pause; everything else stops.
                let pausable = matches!(seq_state, Some((Phase::Play(_), _, _, true, _))) && self.player.controllable;
                let key = self.pause_key(tag);
                let mut pauses = self.saved.pause_steps.contains(&key);
                ui.horizontal(|ui| {
                    let (glyph, hint) = match (running, pausable, self.player.paused) {
                        (true, true, false) => ("⏸", tr("Pause")),
                        (true, true, true) => ("▶", tr("Listen")),
                        (true, false, _) => ("■", tr("Stop")),
                        _ => ("▶", tr("Listen")),
                    };
                    if theme::round_button(ui, glyph, 46.0, ACCENT()).on_hover_text(hint).clicked() {
                        if pausable {
                            self.player.toggle_pause();
                        } else if running {
                            self.stop_audio();
                        } else {
                            self.play_seq(tag, lang, texts(items), 0, false);
                        }
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;
                        match seq_state {
                            Some((phase, idx, n, _, repeat)) if running => {
                                ui.label(medium(trf("Sentence {0} of {1}", &[&(idx + 1), &n])));
                                ui.horizontal(|ui| match phase {
                                    Phase::Wait => {
                                        ui.spinner();
                                        ui.label(muted(tr("preparing the voice…")));
                                    }
                                    Phase::Play(_) if self.player.paused => {
                                        ui.label(medium(tr("⏸ paused")).color(MUTED()));
                                    }
                                    Phase::Play(_) => {
                                        ui.label(medium(tr("listen…")).color(ACCENT()));
                                    }
                                    Phase::Gap(until, len) if repeat => {
                                        ui.label(medium(tr("● repeat aloud now")).color(WARN()));
                                        let left =
                                            until.saturating_duration_since(Instant::now()).as_secs_f32() / len.as_secs_f32().max(0.01);
                                        theme::bar_on(ui, left, 120.0, WARN(), CARD());
                                    }
                                    _ => {}
                                });
                            }
                            Some((Phase::Done, ..)) => {
                                ui.label(medium(tr("✔ Finished")).color(GOOD()));
                                ui.label(muted(tr("Press ▶ to listen again.")).size(13.5));
                            }
                            _ => {
                                ui.label(medium(plural(items.len(), "Listen · {0} sentence", "Listen · {0} sentences")));
                                ui.label(
                                    muted(if pauses {
                                        tr("The voice reads each sentence; repeat it aloud in the pause.")
                                    } else {
                                        tr("The whole dialogue in one go, as on a recording.")
                                    })
                                    .size(13.5),
                                );
                            }
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let (true, Some((_, idx, ..))) = (running, seq_state) {
                            if pausable && ui.add(theme::ghost(tr("Stop"))).clicked() {
                                self.stop_audio();
                            }
                            if ui.add(theme::ghost("⏭")).on_hover_text(tr("Next sentence")).clicked() {
                                self.play_seq(tag, lang, texts(items), idx + 1, false);
                            }
                            if ui.add(theme::ghost("⟲")).on_hover_text(tr("Repeat this sentence")).clicked() {
                                self.play_seq(tag, lang, texts(items), idx, false);
                            }
                            if ui.add(theme::ghost("⏮")).on_hover_text(tr("Previous sentence")).clicked() {
                                self.play_seq(tag, lang, texts(items), idx.saturating_sub(1), false);
                            }
                        }
                        // The speed, here as well as in Settings: the learner finds the voice too fast or slow while listening.
                        let speed = self.saved.speed;
                        egui::ComboBox::from_id_salt("listen-speed")
                            .width(64.0)
                            .selected_text(format!("{speed:.2}×"))
                            .show_ui(ui, |ui| {
                                for v in [0.7_f32, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3] {
                                    if ui.selectable_label((self.saved.speed - v).abs() < 0.01, format!("{v:.1}×")).clicked() {
                                        self.saved.speed = v;
                                        self.mark_dirty();
                                    }
                                }
                            })
                            .response
                            .on_hover_text(tr("Speed"));
                        // Chosen per step: one may be for listening straight through, another for repeating.
                        if ui
                            .checkbox(&mut pauses, tr("Pause to repeat"))
                            .on_hover_text(tr("The voice waits after each sentence so you can repeat it aloud"))
                            .changed()
                        {
                            if pauses {
                                self.saved.pause_steps.insert(key.clone());
                            } else {
                                self.saved.pause_steps.remove(&key);
                            }
                            self.mark_dirty();
                            if running {
                                self.stop_audio();
                            }
                        }
                    });
                });
                return;
            }
            ui.horizontal(|ui| {
                theme::icon_circle(ui, "🔈", ACCENT(), CARD(), 46.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.label(medium(tr("No audio for this lesson yet")));
                    self.get_voice(ui, lang);
                    ui.label(muted(tr("Or link your own recordings in Courses. Meanwhile, read the sentences aloud yourself.")).size(13.5));
                    if ui.add(theme::ghost(tr("Link my recordings of this course…"))).clicked() {
                        let ci = self.active();
                        self.pick_audio_folder(ci);
                    }
                });
            });
        });
    }

    /// The voice's download button, or how far the download is.
    pub(crate) fn get_voice(&mut self, ui: &mut egui::Ui, lang: &str) {
        let status = self.voices.status_of(lang);
        ui.horizontal_wrapped(|ui| {
            if crate::audio::downloading(&status) {
                ui.spinner();
                let mb = self.voices.partial_mb(lang).map(|m| format!(" {m:.0} MB")).unwrap_or_default();
                ui.label(format!("{status}{mb}"));
                ui.ctx().request_repaint_after(Duration::from_millis(400));
            } else {
                let name = in_sentence(&lang_name(lang, &self.base()));
                if ui.add(theme::primary(trf("Get the {0} voice (free, ~60 MB)", &[&name]))).clicked() {
                    self.voices.download(lang, ui.ctx().clone());
                }
                if !status.is_empty() {
                    ui.label(RichText::new(&status).color(BAD()));
                }
            }
        });
    }

    // --------------------------------------------------------- rendering

    /// Under a sentence: pinyin or vowel lengths, then how it sounds spelled
    /// for the reader's language (Assimil's figured pronunciation).
    pub(crate) fn pron_lines(&self, ui: &mut egui::Ui, item: &Item) {
        if !self.saved.show_pron {
            return;
        }
        if !item.pron.is_empty() {
            ui.label(RichText::new(&item.pron).size(14.0).italics().color(ACCENT()));
        }
        if let Some(f) = item.figured(&self.base()) {
            match theme::tone_rows(ui, &item.text, f, 14.0, FIGURED()) {
                Some(r) => r.on_hover_text(tr("How it sounds, spelled for your language: high tones above, low tones below")),
                None => ui
                    .label(theme::starred(&format!("{f}"), 14.0, FIGURED()))
                    .on_hover_text(tr("How it sounds, spelled for your language; the stressed syllable is in bold")),
            };
        }
    }

    /// One sentence row; returns true when its ▶ was clicked.
    #[allow(clippy::too_many_arguments)]
    fn sentence(
        &self,
        ui: &mut egui::Ui,
        n: usize,
        item: &Item,
        show_target: bool,
        show_tr: bool,
        highlight: bool,
        can_play: bool,
        big: bool,
    ) -> bool {
        let base = self.base();
        let mut clicked = false;
        let fill = if highlight { ACCENT_SOFT() } else { Color32::TRANSPARENT };
        egui::Frame::new()
            .fill(fill)
            .corner_radius(10)
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_top(|ui| {
                    theme::number(ui, n, highlight);
                    if can_play {
                        clicked = ui.add(theme::ghost("▶").small()).on_hover_text(tr("Listen")).clicked();
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;
                        let size = if big { 21.0 } else { 19.0 };
                        if show_target {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(item.plain()).size(size));
                                if !item.refs.is_empty() {
                                    let refs: Vec<String> = item.refs.iter().map(u32::to_string).collect();
                                    ui.label(muted(format!("({})", refs.join(", "))).size(13.0))
                                        .on_hover_text(tr("The lessons these words come from"));
                                }
                            });
                            self.pron_lines(ui, item);
                        }
                        if show_tr {
                            let tr = item.tr(&base);
                            ui.label(if show_target {
                                muted(tr).size(15.0)
                            } else {
                                RichText::new(tr).size(size)
                            });
                        }
                    });
                });
            });
        clicked
    }

    pub(crate) fn dialogue_block(&mut self, ui: &mut egui::Ui, tag: &str, lang: &str, items: &[Item], show_target: bool, show_tr: bool) {
        let cur = self.current(tag);
        let can_play = self.voices.ready(lang);
        let mut play = None;
        theme::card().inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, it) in items.iter().enumerate() {
                if self.sentence(ui, i + 1, it, show_target, show_tr, cur == Some(i), can_play, false) {
                    play = Some(i);
                }
            }
        });
        if let Some(i) = play {
            self.play_seq(tag, lang, texts(items), i, true);
        }
    }

    /// The step bar, drawn as the lesson's own line: one station per step,
    /// filled when done, a large ring where the learner is, hollow ahead;
    /// a click jumps there. Returns the step clicked.
    fn stepper(&self, ui: &mut egui::Ui, steps: &[Step], done: &[bool], at: usize) -> Option<usize> {
        let n = steps.len().max(1);
        let width = ui.available_width();
        let pad = 10.0;
        let seg = (width - 2.0 * pad) / (n as f32 - 1.0).max(1.0);
        let labels = seg >= 80.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, if labels { 44.0 } else { 40.0 }), egui::Sense::hover());
        let y = rect.top() + 12.0;
        let x = |i: usize| rect.left() + pad + seg * i as f32;
        let line = theme::FILL();
        let track = theme::lerp(line, theme::BG(), 0.7);
        let p = ui.painter();
        p.line_segment([egui::pos2(x(0), y), egui::pos2(x(n - 1), y)], egui::Stroke::new(5.0, track));
        p.line_segment([egui::pos2(x(0), y), egui::pos2(x(at), y)], egui::Stroke::new(5.0, line));
        if !labels {
            // Too narrow for every name: the current step is named under the line.
            if let Some(st) = steps.get(at) {
                p.text(
                    egui::pos2(rect.left(), rect.bottom() - 6.0),
                    egui::Align2::LEFT_CENTER,
                    format!("{}/{} · {}", at + 1, n, st.title()),
                    egui::FontId::new(12.5, theme::medium_family()),
                    INK(),
                );
            }
        }
        let mut jump = None;
        for (i, st) in steps.iter().enumerate() {
            let c = egui::pos2(x(i), y);
            let hit = egui::Rect::from_center_size(egui::pos2(c.x, rect.center().y), egui::vec2(seg.min(80.0), rect.height()));
            let resp = ui
                .interact(hit, ui.id().with(("step", i)), egui::Sense::click())
                .on_hover_text(st.title())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            let grow = if resp.hovered() { 1.15 } else { 1.0 };
            let p = ui.painter();
            if i == at {
                p.circle_filled(c, 9.0 * grow, theme::CARD());
                p.circle_stroke(c, 9.0 * grow, egui::Stroke::new(3.5, line));
            } else if done[i] {
                p.circle_filled(c, 6.0 * grow, line);
                p.circle_filled(c, 2.5 * grow, theme::CARD());
            } else {
                p.circle_filled(c, 6.0 * grow, theme::CARD());
                p.circle_stroke(c, 6.0 * grow, egui::Stroke::new(2.5, track));
            }
            if labels {
                let (family, col) = if i == at { (theme::sign_family(), INK()) } else { (egui::FontFamily::Proportional, MUTED()) };
                p.text(
                    egui::pos2(c.x, rect.bottom() - 8.0),
                    egui::Align2::CENTER_CENTER,
                    st.short(),
                    egui::FontId::new(12.0, family),
                    col,
                );
            }
            if resp.clicked() {
                jump = Some(i);
            }
        }
        jump
    }

    /// The picture for the blind listening steps: waves around an ear,
    /// moving while the voice speaks.
    fn listening_hero(&self, ui: &mut egui::Ui, current: Option<usize>, total: usize) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(170.0, 170.0), egui::Sense::hover());
            let c = rect.center();
            let p = ui.painter();
            let t = if current.is_some() { ui.input(|i| i.time) as f32 } else { 0.0 };
            for k in 0..3 {
                let phase = (t * 0.6 + k as f32 / 3.0).fract();
                let r = if current.is_some() {
                    40.0 + phase * 44.0
                } else {
                    46.0 + k as f32 * 16.0
                };
                let alpha = if current.is_some() {
                    ((1.0 - phase) * 120.0) as u8
                } else {
                    70 - k as u8 * 20
                };
                let l = theme::FILL();
                p.circle_stroke(c, r, egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(l.r(), l.g(), l.b(), alpha)));
            }
            p.circle_filled(c, 38.0, ACCENT_SOFT());
            theme::Icon::Speaker.paint(p, egui::Rect::from_center_size(c, egui::vec2(34.0, 34.0)), ACCENT());
            ui.label(medium(tr("Eyes off the text — listen and repeat aloud")).size(17.0).color(MUTED()));
            if let Some(i) = current {
                ui.label(muted(trf("sentence {0} of {1}", &[&(i + 1), &total])));
            }
            ui.add_space(10.0);
        });
    }

    pub fn study_screen(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.session.as_ref() else {
            ui.label(tr("No lesson open."));
            return;
        };
        let (ci, li, step_i) = (s.course, s.lesson, s.step);
        if self.courses.get(ci).is_none_or(|c| li >= c.lessons.len())
            || s.wave2
                .is_some_and(|(wc, w)| self.courses.get(wc).is_none_or(|c| w >= c.lessons.len()))
        {
            self.session = None;
            self.screen = Screen::Home;
            return;
        }
        let steps = s.steps.clone();
        let done = s.done.clone();
        let course = self.courses[ci].clone();
        let lesson = course.lessons[li].clone();
        let lang = course.lang.clone();
        let base = self.base();
        let step = steps[step_i];

        // Header and step bar, in the same column as the content.
        ui.add_space(6.0);
        let jump = theme::page(ui, STUDY_W, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(theme::secondary("×"))
                    .on_hover_text(tr("Close — your place in the lesson is kept"))
                    .clicked()
                {
                    self.stop_audio();
                    self.session = None;
                    self.screen = Screen::Home;
                }
                ui.add_space(4.0);
                theme::badge(ui, lesson.level.name(), lesson.level.color());
                if lesson.numeral.is_empty() {
                    ui.label(muted(trf("Lesson {0}", &[&lesson.number])).size(15.0));
                } else {
                    let label = ui.label(muted(format!("{} · {}", trf("Lesson {0}", &[&lesson.number]), lesson.numeral)).size(15.0));
                    let pron = lesson.numeral_pron.get(&base).map(String::as_str).unwrap_or("");
                    label.on_hover_text(format!("{} {}", trf("Lesson {0}", &[&lesson.number]), pron.replace('*', "")));
                }
                ui.label(theme::display(&lesson.title).size(22.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::pill(ui, &lang_name(&lang, &base), lang_color(&lang), theme::soft(lang_color(&lang)));
                    ui.add(egui::Label::new(muted(lesson.title_in(&base)).size(15.0)).truncate());
                });
            });
            ui.add_space(6.0);
            self.stepper(ui, &steps, &done, step_i)
        });
        if self.session.is_none() {
            return;
        }
        if let Some(i) = jump {
            self.set_step(i);
            return;
        }

        // Footer with the step buttons.
        let last = step_i + 1 == steps.len();
        let mut next = false;
        let mut back = false;
        egui::Panel::bottom("study_footer")
            .frame(egui::Frame::new().inner_margin(egui::Margin {
                left: 0,
                right: 0,
                top: 12,
                bottom: 6,
            }))
            .show(ui, |ui| {
                theme::page(ui, STUDY_W, |ui| {
                    ui.horizontal(|ui| {
                        if step_i > 0 && ui.add(theme::secondary(tr("‹  Back")).min_size(egui::vec2(100.0, 46.0))).clicked() {
                            back = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let label = if last {
                                tr("✔  Finish the lesson")
                            } else {
                                tr("Done — next step  ›")
                            };
                            if ui.add(theme::big_primary(label)).clicked() {
                                next = true;
                            }
                            ui.label(muted(trf("Step {0} of {1}", &[&(step_i + 1), &steps.len()])));
                        });
                    });
                });
            });
        if back {
            self.set_step(step_i - 1);
            return;
        }
        if next {
            self.complete_step();
            return;
        }

        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            theme::page(ui, STUDY_W, |ui| {
                ui.add_space(12.0);
                ui.label(theme::display(step.title()).size(28.0));
                ui.label(muted(step.how()).size(15.0));
                ui.add_space(8.0);
                let rec = self.recording(ci, li);
                let grammar_only = lesson.revision && !lesson.notes.is_empty();
                if lesson.dialogue.is_empty() && !grammar_only && !matches!(step, Step::Previous | Step::SecondWave) {
                    theme::tinted(HIGHLIGHT()).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(tr("This lesson has no text yet: follow it in your book while you listen. You can add the text any time in “My lessons”."));
                    });
                }
                match step {
                    Step::BlindListen | Step::FinalListen => {
                        self.listen_panel(ui, "dlg", &lang, &lesson.dialogue, rec);
                        let cur = self.current("dlg");
                        self.listening_hero(ui, cur, lesson.dialogue.len());
                        if step == Step::FinalListen {
                            egui::CollapsingHeader::new(muted(tr("Peek at the text (only if you are stuck)"))).show(ui, |ui| {
                                self.dialogue_block(ui, "dlg", &lang, &lesson.dialogue, true, false);
                            });
                            pronunciation_check(ui, &lang, &lesson.dialogue);
                        }
                    }
                    Step::ListenBase => {
                        self.listen_panel(ui, "dlg", &lang, &lesson.dialogue, rec);
                        self.dialogue_block(ui, "dlg", &lang, &lesson.dialogue, false, true);
                    }
                    Step::ListenTarget => {
                        self.listen_panel(ui, "dlg", &lang, &lesson.dialogue, rec);
                        self.dialogue_block(ui, "dlg", &lang, &lesson.dialogue, true, false);
                    }
                    Step::Read | Step::ReadAgain => {
                        self.listen_panel(ui, "dlg", &lang, &lesson.dialogue, rec);
                        // The guide is offered where the spelled-out sounds are read, not only with the notes.
                        let figured = lesson.dialogue.iter().any(|i| i.figured(&base).is_some());
                        ui.horizontal(|ui| {
                            if let Some(s) = self.session.as_mut() {
                                ui.checkbox(&mut s.hide_tr, tr("Hide the translation"));
                            }
                            if figured && crate::store::pronunciation_key(&lang).is_some() {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(theme::ghost(tr("Pronunciation guide"))).clicked() {
                                        self.key_open = true;
                                    }
                                });
                            }
                        });
                        let hide = self.session.as_ref().is_some_and(|s| s.hide_tr);
                        self.dialogue_block(ui, "dlg", &lang, &lesson.dialogue, true, !hide);
                    }
                    Step::Notes | Step::Recap => self.notes_step(ui, &lang, &lesson, step == Step::Recap),
                    Step::Tasks => {
                        let mut p = self.session.as_mut().map(|s| std::mem::take(&mut s.tasks)).unwrap_or_default();
                        self.tasks_step(ui, &lang, &lesson.tasks, &mut p);
                        if let Some(s) = self.session.as_mut() {
                            s.tasks = p;
                        }
                    }
                    Step::Ex1 => self.ex1(ui, &lang, &lesson),
                    Step::Ex2 => self.ex2(ui, &lang, &lesson),
                    Step::Previous => {
                        let prev = previous_dialogues(&course, li);
                        if prev.is_empty() {
                            ui.label(RichText::new(tr("This is the first lesson: nothing to review yet. Just listen to today's dialogue once more if you like.")).size(16.0));
                            self.listen_panel(ui, "dlg", &lang, &lesson.dialogue, rec);
                        }
                        for j in prev {
                            let pl = course.lessons[j].clone();
                            let prec = self.recording(ci, j);
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                theme::badge(ui, pl.level.name(), pl.level.color());
                                ui.label(bold(trf("Lesson {0} · {1}", &[&pl.number, &pl.title])).size(18.0));
                                ui.label(muted(pl.title_in(&base)));
                            });
                            let tag = format!("prev{j}");
                            self.listen_panel(ui, &tag, &lang, &pl.dialogue, prec);
                            self.dialogue_block(ui, &tag, &lang, &pl.dialogue, true, true);
                        }
                    }
                    Step::SecondWave => {
                        if let Some((wc, w)) = self.session.as_ref().and_then(|s| s.wave2) {
                            let mut p = self.session.as_mut().map(|s| std::mem::take(&mut s.wave)).unwrap_or_default();
                            if wc != ci {
                                ui.label(
                                    muted(trf(
                                        "The active wave of {0} goes on alongside this course: today one of its lessons comes back.",
                                        &[&self.stage_name(wc)],
                                    ))
                                    .size(14.0),
                                );
                            }
                            self.reconstruct(ui, wc, w, &mut p);
                            if let Some(s) = self.session.as_mut() {
                                s.wave = p;
                            }
                        }
                    }
                }
                if step_i + 1 == steps.len() {
                    self.how_did_it_go(ui);
                }
                ui.add_space(16.0);
            });
        });
    }

    /// The notes (a revision lesson's grammar) with the lesson's dialogue
    /// beside them, or above them on a narrow window: pointing at a note
    /// lights the lines it is about, clicking keeps them lit.
    fn notes_step(&mut self, ui: &mut egui::Ui, lang: &str, lesson: &Lesson, recap: bool) {
        let text = lesson.notes_in(&self.base()).to_string();
        let refs: Vec<Vec<usize>> = paragraphs(&text).iter().map(|(_, p, _)| note_lines(p, &lesson.dialogue)).collect();
        let lit_note = self.session.as_ref().and_then(|s| s.note_hover.or(s.note_pick));
        let lit = lit_note.and_then(|k| refs.get(k)).cloned().unwrap_or_default();
        let mut event = None;
        if lesson.dialogue.is_empty() {
            event = self.notes_column(ui, lang, lesson, recap, &refs, lit_note);
        } else if ui.available_width() >= 760.0 {
            // Both columns scroll on their own, so the dialogue stays in sight.
            let height = (ui.clip_rect().bottom() - ui.cursor().top() - 30.0).max(260.0);
            ui.horizontal_top(|ui| {
                let left = ((ui.available_width() - 16.0) * 0.58).floor();
                ui.allocate_ui_with_layout(egui::vec2(left, height), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_width(left);
                    egui::ScrollArea::vertical().id_salt("notes-left").max_height(height).show(ui, |ui| {
                        event = self.notes_column(ui, lang, lesson, recap, &refs, lit_note);
                    });
                });
                ui.add_space(8.0);
                ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), height), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(caps(tr("The dialogue")));
                    egui::ScrollArea::vertical().id_salt("notes-dialogue").max_height(height - 24.0).show(ui, |ui| {
                        self.side_dialogue(ui, lesson, &lit);
                    });
                });
            });
        } else {
            egui::CollapsingHeader::new(medium(tr("The dialogue")))
                .default_open(true)
                .show(ui, |ui| self.side_dialogue(ui, lesson, &lit));
            event = self.notes_column(ui, lang, lesson, recap, &refs, lit_note);
        }
        if let Some(s) = self.session.as_mut() {
            let hover = event.map(|(k, _)| k);
            if hover != s.note_hover {
                // On a narrow window the dialogue is drawn before the notes.
                ui.ctx().request_repaint();
            }
            s.note_hover = hover;
            if let Some((k, true)) = event {
                s.note_pick = if s.note_pick == Some(k) { None } else { Some(k) };
                s.note_seek = true;
            }
        }
    }

    /// What the notes step reads: the pronunciation, the notes, the culture
    /// and the author's word, or a revision lesson's grammar.
    fn notes_column(
        &mut self,
        ui: &mut egui::Ui,
        lang: &str,
        lesson: &Lesson,
        recap: bool,
        refs: &[Vec<usize>],
        lit: Option<usize>,
    ) -> Option<(usize, bool)> {
        let base = self.base();
        if !lesson.dialogue.is_empty() && refs.iter().any(|r| !r.is_empty()) {
            ui.label(muted(tr("Point at a note to light up its line in the dialogue; click to keep it lit.")).size(13.5));
        }
        if recap {
            let event = notes_rows(ui, lesson.notes_in(&base), Some(refs), lit);
            titled(ui, tr("From the author"), lesson.aside_in(&base));
            return event;
        }
        let figured = lesson.dialogue.iter().any(|i| i.figured(&base).is_some());
        if figured && crate::store::pronunciation_key(lang).is_some() && ui.add(theme::ghost(tr("Pronunciation guide"))).clicked() {
            self.key_open = true;
        }
        titled(ui, tr("Pronunciation"), lesson.sounds_in(&base));
        if !lesson.sounds_in(&base).trim().is_empty() {
            ui.label(caps(tr("Notes")));
        }
        let event = notes_rows(ui, lesson.notes_in(&base), Some(refs), lit);
        titled(ui, tr("Culture"), lesson.culture_in(&base));
        titled(ui, tr("From the author"), lesson.aside_in(&base));
        event
    }

    /// The dialogue beside the notes: numbered, the original with its
    /// translation small under it, the lines in `lit` highlighted.
    fn side_dialogue(&mut self, ui: &mut egui::Ui, lesson: &Lesson, lit: &[usize]) {
        let base = self.base();
        let seek = self.session.as_ref().is_some_and(|s| s.note_seek);
        theme::card().inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, it) in lesson.dialogue.iter().enumerate() {
                let on = lit.contains(&i);
                let row = egui::Frame::new()
                    .fill(if on { HIGHLIGHT() } else { Color32::TRANSPARENT })
                    .corner_radius(8)
                    .inner_margin(egui::Margin::symmetric(8, 5))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_top(|ui| {
                            theme::number(ui, i + 1, on);
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 1.0;
                                ui.add(egui::Label::new(RichText::new(it.plain()).size(16.0)).wrap());
                                ui.add(egui::Label::new(muted(it.tr(&base)).size(13.0)).wrap());
                            });
                        });
                    })
                    .response;
                if seek && lit.first() == Some(&i) {
                    row.scroll_to_me(Some(egui::Align::Center));
                }
            }
        });
        if let Some(s) = self.session.as_mut() {
            s.note_seek = false;
        }
    }

    fn ex1(&mut self, ui: &mut egui::Ui, lang: &str, lesson: &Lesson) {
        let base = self.base();
        let can_play = self.voices.ready(lang);
        let mut shown = self.session.as_ref().map(|s| s.shown.clone()).unwrap_or_default();
        if ui.add(theme::ghost(tr("Show all translations"))).clicked() {
            shown.extend(0..lesson.ex1.len());
        }
        let mut play = None;
        theme::card().inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (i, it) in lesson.ex1.iter().enumerate() {
                egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 8)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_top(|ui| {
                        theme::number(ui, i + 1, false);
                        if can_play && ui.add(theme::ghost("▶").small()).clicked() {
                            play = Some(i);
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.label(RichText::new(it.plain()).size(19.0));
                            self.pron_lines(ui, it);
                            if shown.contains(&i) {
                                ui.label(RichText::new(format!("→ {}", it.tr(&base))).color(GOOD()).size(16.0));
                            } else if ui.add(theme::secondary(tr("Check my translation"))).clicked() {
                                shown.insert(i);
                            }
                        });
                    });
                });
            }
        });
        if let Some(i) = play {
            self.play_seq("ex1", lang, texts(&lesson.ex1), i, true);
        }
        if let Some(s) = self.session.as_mut() {
            s.shown = shown;
        }
    }

    fn ex2(&mut self, ui: &mut egui::Ui, lang: &str, lesson: &Lesson) {
        let base = self.base();
        let can_play = self.voices.ready(lang);
        let Some(mut s) = self.session.take() else { return };
        let mut play = None;
        theme::card().inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (i, it) in lesson.ex2.iter().enumerate() {
                let blanks = it.blanks();
                if s.answers.len() <= i {
                    s.answers.push(vec![String::new(); blanks.len()]);
                }
                egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 8)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_top(|ui| {
                        theme::number(ui, i + 1, false);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.label(muted(it.tr(&base)).size(15.0));
                            ui.horizontal_wrapped(|ui| {
                                let mut k = 0;
                                for (text, blank) in it.parts() {
                                    if !blank {
                                        ui.label(RichText::new(text).size(19.0));
                                        continue;
                                    }
                                    let answer = &mut s.answers[i][k];
                                    let width = (text.chars().count() as f32 * 12.0 + 34.0).max(70.0);
                                    // As in the books: one dot for each missing letter.
                                    let dots = "·".repeat(text.chars().filter(|c| c.is_alphanumeric()).count());
                                    let edit = egui::TextEdit::singleline(answer)
                                        .hint_text(dots)
                                        .desired_width(width)
                                        .font(egui::FontId::proportional(18.0))
                                        .margin(egui::Margin::symmetric(8, 4));
                                    ui.add_enabled(!s.checked, edit);
                                    if s.checked {
                                        match compare(answer, &text).verdict {
                                            Verdict::Right => ui.label(medium("✔").color(GOOD())),
                                            Verdict::Accents => ui
                                                .label(medium(format!("≈ {text}")).color(WARN()))
                                                .on_hover_text(tr("Almost: check the accents")),
                                            Verdict::Wrong => ui.label(medium(format!("✗ {text}")).color(BAD())),
                                        };
                                    }
                                    k += 1;
                                }
                            });
                            if s.checked {
                                ui.horizontal(|ui| {
                                    if can_play && ui.add(theme::ghost("▶").small()).clicked() {
                                        play = Some(i);
                                    }
                                    ui.label(RichText::new(it.plain()).color(ACCENT()));
                                });
                                self.pron_lines(ui, it);
                            }
                        });
                    });
                });
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if !s.checked {
                if ui.add(theme::primary(tr("Check my answers"))).clicked() {
                    let mut right = 0;
                    let mut total = 0;
                    for (i, it) in lesson.ex2.iter().enumerate() {
                        let typed = |k: usize| s.answers[i].get(k).map(|a| a.as_str()).unwrap_or("");
                        for (k, b) in it.blanks().iter().enumerate() {
                            total += 1;
                            right += (compare(typed(k), b).verdict != Verdict::Wrong) as u32;
                        }
                    }
                    s.score = (right, total);
                    s.checked = true;
                }
            } else {
                let (r, t) = s.score;
                let color = if r == t {
                    GOOD()
                } else if r * 2 >= t {
                    WARN()
                } else {
                    BAD()
                };
                theme::pill(ui, &trf("{0} of {1} right", &[&r, &t]), color, theme::soft(color));
                if ui.add(theme::secondary(tr("Try again"))).clicked() {
                    s.checked = false;
                    for a in s.answers.iter_mut().flatten() {
                        a.clear();
                    }
                }
            }
        });
        self.session = Some(s);
        if let Some(i) = play {
            self.play_seq("ex2", lang, texts(&lesson.ex2), i, true);
        }
    }

    // ------------------------------------------------------------ review

    /// Daily sentences due today whose sentence still exists (a lesson the
    /// editor removed or shortened leaves cards behind).
    pub fn cards_due(&self) -> usize {
        let t = day_key(today());
        self.saved
            .cards
            .iter()
            .filter(|(k, c)| c.due <= t && self.card_target(k).is_some())
            .count()
    }

    fn card_target(&self, key: &str) -> Option<(usize, usize, usize)> {
        let mut parts = key.rsplitn(3, '#');
        let idx: usize = parts.next()?.parse().ok()?;
        let number: u32 = parts.next()?.parse().ok()?;
        let ci = self.course_index(parts.next()?)?;
        let li = self.courses[ci].lessons.iter().position(|l| l.number == number)?;
        (idx < self.courses[ci].lessons[li].dialogue.len()).then_some((ci, li, idx))
    }

    pub fn review_screen(&mut self, ui: &mut egui::Ui) {
        let due = self.cards_due();
        let ci = self.active();
        let pending = self.wave2_pending(ci);
        // The tab lists the chosen course of this language, else the oldest
        // one whose wave is still going on.
        let wci = self
            .review
            .wave_course
            .filter(|&w| w == ci || pending.iter().any(|&(c, _)| c == w))
            .or_else(|| pending.first().map(|&(c, _)| c))
            .unwrap_or(ci);
        let wave = self.wave2_due(wci);
        theme::page(ui, 780.0, |ui| {
            theme::page_title(ui, tr("Review"), tr("The active wave of the method, the sentences of the day, and a search across every sentence you have met."));
            ui.add_space(-6.0);
            // The spaced sentence review is not part of the Assimil method: it
            // shows only when the learner turned it on (Settings) or has cards.
            let cards = self.saved.cards_after_lesson || !self.saved.cards.is_empty();
            if !cards && self.review.tab == 0 {
                self.review.tab = 1;
            }
            // A segmented control for the two kinds of review.
            egui::Frame::new().fill(TRACK()).corner_radius(12).inner_margin(4).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    // The daily sentences tab is shown even when off, so the learner can find and turn it on.
                    let tabs = vec![
                        (1, trf("Active wave · {0}", &[&pending.len()])),
                        (2, tr("Search").to_string()),
                        (0, trf("Daily sentences · {0}", &[&due])),
                    ];
                    for (i, t) in tabs {
                        let sel = self.review.tab == i;
                        let b = egui::Button::new(medium(&t).color(if sel { INK() } else { MUTED() }))
                            .fill(if sel { CARD() } else { Color32::TRANSPARENT })
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(9)
                            .min_size(egui::vec2(150.0, 34.0));
                        if ui.add(b).clicked() {
                            self.review.tab = i;
                            self.review.practice = None;
                            self.stop_audio();
                        }
                    }
                });
            });
            ui.add_space(10.0);
            match self.review.tab {
                0 if !cards => {
                    theme::card().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if ui
                            .checkbox(
                                &mut self.saved.cards_after_lesson,
                                tr("Add each finished dialogue's sentences to a spaced review (an extra, beside the method)"),
                            )
                            .changed()
                        {
                            self.mark_dirty();
                        }
                    });
                }
                0 => self.sentence_review(ui),
                1 => self.wave_review(ui, ci, wci, &wave),
                _ => self.search(ui),
            }
        });
    }

    fn sentence_review(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        if !self.review.built {
            let t = day_key(today());
            let mut due: Vec<(&String, &Card)> = self.saved.cards.iter().filter(|(_, c)| c.due <= t).collect();
            due.sort_by(|a, b| a.1.due.cmp(&b.1.due).then(a.0.cmp(b.0)));
            self.review.queue = due.into_iter().map(|(k, _)| k.clone()).collect();
            self.review.built = true;
            self.review.shown = false;
        }
        let Some(key) = self.review.queue.first().cloned() else {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);
                    theme::icon_circle(ui, "✔", GOOD(), GOOD_SOFT(), 60.0);
                    ui.label(bold(tr("Nothing left to review today")).size(21.0));
                    let next = self.saved.cards.values().map(|c| c.due.clone()).min();
                    ui.label(muted(match next {
                        Some(d) => trf(
                            "{0} · next ones on {1}",
                            &[&plural(self.saved.cards.len(), "{0} sentence in review", "{0} sentences in review"), &d],
                        ),
                        None => tr("Sentences arrive here after each finished lesson.").into(),
                    }));
                    ui.add_space(12.0);
                });
            });
            return;
        };
        let Some((ci, li, idx)) = self.card_target(&key) else {
            self.review.queue.remove(0);
            return;
        };
        let course = &self.courses[ci];
        let lesson = &course.lessons[li];
        let item = lesson.dialogue[idx].clone();
        let lang = course.lang.clone();
        let label = trf("{0} · lesson {1} · {2}", &[&lang_name(&lang, &base), &lesson.number, &lesson.title]);
        let tag = format!("card{key}");

        ui.horizontal(|ui| {
            ui.label(muted(tr("Read the sentence in your language, say it aloud in the language you are learning, then check and be honest with yourself.")).size(14.0));
        });
        ui.add_space(4.0);
        let mut grade = None;
        theme::card().inner_margin(24).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(caps(&label));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::pill(ui, &trf("{0} left", &[&self.review.queue.len()]), ACCENT(), ACCENT_SOFT());
                });
            });
            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(item.tr(&base)).size(24.0));
                ui.add_space(16.0);
                if !self.review.shown {
                    if ui.add(theme::big_primary(tr("Said it aloud — show me"))).clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                        self.review.shown = true;
                        if self.voices.ready(&lang) {
                            self.play_seq(&tag, &lang, vec![item.plain()], 0, true);
                        }
                    }
                    ui.label(muted(tr("or press Space")).size(12.5));
                    return;
                }
                ui.horizontal(|ui| {
                    // Centre the answer and its ▶ together.
                    let w = (item.plain().chars().count() as f32 * 14.0 + 40.0).min(ui.available_width());
                    ui.add_space(((ui.available_width() - w) / 2.0).max(0.0));
                    if self.voices.ready(&lang) && ui.add(theme::ghost("▶")).clicked() {
                        self.play_seq(&tag, &lang, vec![item.plain()], 0, true);
                    }
                    ui.label(bold(item.plain()).size(25.0).color(ACCENT()));
                });
                self.pron_lines(ui, &item);
            });
            if !self.review.shown {
                return;
            }
            ui.add_space(16.0);
            let card = self.saved.cards.get(&key).cloned().unwrap_or_default();
            let keys = [egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4];
            ui.columns(4, |cols| {
                for (k, (col, (g, name, color))) in cols
                    .iter_mut()
                    .zip([
                        (Grade::Again, tr("Again"), BAD()),
                        (Grade::Hard, tr("Hard"), WARN()),
                        (Grade::Good, tr("Good"), GOOD()),
                        (Grade::Easy, tr("Easy"), ACCENT()),
                    ])
                    .enumerate()
                {
                    let mut preview = card.clone();
                    preview.grade(g, today());
                    let when = if preview.interval == 0 {
                        tr("today").to_string()
                    } else {
                        trf("in {0} d", &[&preview.interval])
                    };
                    let b = egui::Button::new(medium(format!("{name}\n{when}")).color(color))
                        .fill(theme::soft(color))
                        .stroke(egui::Stroke::new(1.0, theme::lerp(color, Color32::WHITE, 0.6)))
                        .corner_radius(12);
                    let w = col.available_width();
                    let resp = col.add_sized(egui::vec2(w, 56.0), b).on_hover_text(trf("Key {0}", &[&(k + 1)]));
                    if resp.clicked() || col.input(|i| i.key_pressed(keys[k])) {
                        grade = Some(g);
                    }
                }
            });
        });
        if let Some(g) = grade {
            let again = self.saved.cards.entry(key.clone()).or_default().grade(g, today());
            self.review.queue.remove(0);
            if again {
                self.review.queue.push(key);
            }
            self.review.shown = false;
            self.saved.day().reviews += 1;
            self.stop_audio();
            self.mark_dirty();
        }
    }

    /// The active-wave tab for course `ci` (the one being studied), listing
    /// the lessons of `wci`: the same course, or an earlier one whose wave is
    /// still going on.
    fn wave_review(&mut self, ui: &mut egui::Ui, active: usize, ci: usize, wave: &[usize]) {
        let base = self.base();
        let course = self.courses[ci].clone();
        ui.label(muted(trf(
            "The active wave is the second half of the method: from lesson {0} on, each day you also rebuild an earlier lesson — you see it in your language and say it aloud in {1}. After the last lesson the remaining ones come back one a day, also while you go on with the next course, until every lesson has been rebuilt. Settings → “Second wave starts” changes the distance.",
            &[&(self.saved.wave2_gap + 1), &in_sentence(&lang_name(&course.lang, &base))],
        )).size(14.0));
        ui.add_space(6.0);
        // One chip per course whose wave is open, when there is more than one.
        let mut courses: Vec<usize> = self.wave2_pending(active).into_iter().map(|(c, _)| c).collect();
        courses.push(active);
        courses.dedup();
        if courses.len() > 1 {
            ui.horizontal_wrapped(|ui| {
                for &k in &courses {
                    let n = self.wave2_due(k).len();
                    if ui.selectable_label(k == ci, format!("{} · {n}", self.stage_name(k))).clicked() && k != ci {
                        self.review.wave_course = Some(k);
                        self.review.practice = None;
                        self.stop_audio();
                    }
                }
            });
            ui.add_space(6.0);
        }
        if let Some((_, pli)) = self.review.practice.filter(|&(c, li)| c == ci && li < course.lessons.len()) {
            let l = course.lessons[pli].clone();
            if ui.add(theme::secondary(tr("‹  All lessons"))).clicked() {
                self.review.practice = None;
                self.stop_audio();
                return;
            }
            let due = wave.contains(&pli);
            if !due {
                ui.label(muted(tr(
                    "Extra practice: this lesson's active wave is not due today, so it is not recorded. It comes back on its own day.",
                )));
            }
            let mut p = std::mem::take(&mut self.review.practice_state);
            self.reconstruct(ui, ci, pli, &mut p);
            self.review.practice_state = p;
            let label = if due {
                tr("✔  Second wave done for this lesson")
            } else {
                tr("✔  Done")
            };
            if ui.add(theme::big_primary(label)).clicked() {
                if due {
                    let key = course.lesson_key(l.number);
                    self.saved.lessons.entry(key.clone()).or_default().wave2 = Some(day_key(today()));
                    if self.saved.cards_after_lesson {
                        for i in 0..l.dialogue.len() {
                            self.saved.cards.entry(format!("{key}#{i}")).or_default();
                        }
                    }
                    self.say(trf("Second wave of lesson {0} done.", &[&l.number]));
                }
                self.saved.day().reviews += 1;
                self.review.practice = None;
                self.stop_audio();
                self.mark_dirty();
            }
            return;
        }
        let done: Vec<usize> = (0..course.lessons.len())
            .filter(|&i| self.saved.is_done(&course.lesson_key(course.lessons[i].number)))
            .collect();
        if done.is_empty() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(tr("Finish a lesson first: it will be ready here for active practice.")).size(16.0));
            });
            return;
        }
        let mut open = None;
        theme::card().inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for i in wave.iter().copied().chain(done.iter().copied().filter(|i| !wave.contains(i))) {
                let l = &course.lessons[i];
                let p = self.saved.progress(&course.lesson_key(l.number)).cloned().unwrap_or_default();
                egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        theme::badge(ui, l.level.name(), l.level.color());
                        ui.label(medium(format!("{} · {}", l.number, l.title)));
                        ui.add(egui::Label::new(muted(l.title_in(&base))).truncate());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let due = wave.contains(&i);
                            let btn = if due {
                                theme::primary(tr("Practise"))
                            } else {
                                theme::ghost(tr("Practise"))
                            };
                            if ui.add(btn).clicked() {
                                open = Some(i);
                            }
                            if due {
                                theme::pill(ui, tr("due today"), WARN(), WARN_SOFT());
                            } else if let Some(d) = &p.wave2 {
                                theme::pill(ui, &trf("done {0}", &[&d]), GOOD(), GOOD_SOFT());
                            }
                        });
                    });
                });
            }
        });
        if let Some(i) = open {
            self.review.practice = Some((ci, i));
            self.review.practice_state = Practice::default();
        }
    }
}

/// The steps of a day's lesson, as in an Assimil book: listen, read with the
/// translation, notes, exercise 1 (translate), exercise 2 (fill in), read and
/// listen again. A revision lesson is its grammar recap and its revision
/// dialogue (exercises only if it has them); a Mastery lesson ends with open
/// tasks; the active wave comes after, from lesson 50.
/// After the last listening: say each sentence and see which words were
/// understood. Optional; it needs the speech engine, offered once here.
fn pronunciation_check(ui: &mut egui::Ui, lang: &str, dialogue: &[Item]) {
    if !crate::speech::supported(lang) || dialogue.is_empty() {
        return;
    }
    ui.add_space(12.0);
    egui::CollapsingHeader::new(medium(tr("Check your pronunciation"))).id_salt("pron-check").show(ui, |ui| {
        ui.label(muted(tr("Say each sentence aloud after pressing Record: the words in green were understood, the red ones are worth another try.")).size(14.5));
        crate::speech::offer(ui, lang);
        if !crate::speech::available(lang) {
            return;
        }
        for (i, item) in dialogue.iter().enumerate() {
            ui.add_space(6.0);
            ui.label(RichText::new(format!("{}. {}", i + 1, item.plain())).size(17.0));
            let _ = crate::speech::check_button(ui, lang, &item.plain());
        }
    });
}

fn lesson_steps(lesson: &Lesson) -> Vec<Step> {
    use Step::*;
    // Every lesson of every course goes the same way; a revision lesson reads
    // its grammar recap where the others read their notes.
    let mut steps = vec![
        BlindListen,
        ListenBase,
        ListenTarget,
        Read,
        if lesson.revision { Recap } else { Notes },
        Ex1,
        Ex2,
        ReadAgain,
        FinalListen,
        Previous,
    ];
    // A revision lesson of grammar and practice only (Perfectionnement).
    if lesson.revision && lesson.dialogue.is_empty() && !lesson.notes.is_empty() {
        steps = vec![Recap, Ex1, Ex2, Previous];
    }
    if !lesson.tasks.is_empty() {
        let at = steps.iter().position(|s| *s == Ex2).map_or(steps.len(), |i| i + 1);
        steps.insert(at, Tasks);
    }
    if lesson.ex1.is_empty() {
        steps.retain(|s| *s != Ex1);
    }
    if lesson.ex2.is_empty() {
        steps.retain(|s| *s != Ex2);
    }
    steps
}

impl App {
    /// The end of a lesson: the next one tomorrow, or the same one again.
    /// Going over a lesson twice is part of the method, not a failure.
    fn how_did_it_go(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.session.as_mut() else { return };
        let low = |sc: (u32, u32)| sc.1 > 0 && sc.0 * 2 < sc.1;
        let struggled = low(s.score);
        ui.add_space(12.0);
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(bold(tr("How did it go?")).size(18.0));
            if struggled {
                ui.label(muted(tr(
                    "The exercises were hard today. Going over this lesson again tomorrow often makes it click.",
                )));
            }
            ui.radio_value(&mut s.again, false, tr("Fine — the next lesson tomorrow"));
            ui.radio_value(&mut s.again, true, tr("Hard — go over this lesson again tomorrow"));
        });
    }

    /// Every sentence and word of the language being studied, all courses,
    /// searched in either language, accents ignored.
    fn search(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        let lang = self.courses.get(self.active()).map(|c| c.lang.clone()).unwrap_or_default();
        ui.add(
            egui::TextEdit::singleline(&mut self.review.search)
                .hint_text(trf("A word in {0} or in your language", &[&in_sentence(&lang_name(&lang, &base))]))
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(17.0)),
        );
        let q = crate::practice::fold(self.review.search.trim());
        if q.chars().count() < 2 {
            ui.label(muted(tr(
                "Type two letters or more. The search covers the words and dialogues of every course in this language.",
            )));
            return;
        }
        let mut hits = Vec::new();
        for (ci, c) in self.courses.iter().enumerate().filter(|(_, c)| c.lang == lang) {
            for l in &c.lessons {
                for it in l.dialogue.iter().chain(&l.ex1) {
                    let found = crate::practice::fold(&it.plain()).contains(&q) || crate::practice::fold(it.tr(&base)).contains(&q);
                    if found {
                        hits.push((ci, l.number, it.clone()));
                    }
                }
            }
        }
        let found = plural(hits.len(), "{0} result", "{0} results");
        ui.label(muted(if hits.len() > 80 { format!("{found} · {}", tr("the first 80")) } else { found }));
        let can_play = self.voices.ready(&lang);
        let mut play = None;
        theme::card().inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (ci, n, it) in hits.iter().take(80) {
                egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 5)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_top(|ui| {
                        if can_play && ui.add(theme::ghost("▶").small()).clicked() {
                            play = Some(it.plain());
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.label(medium(it.plain()).size(16.5));
                            ui.label(muted(it.tr(&base)).size(14.5));
                            ui.label(muted(trf("{0} · lesson {1}", &[&self.course_label(*ci), &n])).size(12.0));
                        });
                    });
                });
            }
        });
        if let Some(t) = play {
            self.play_seq("search", &lang, vec![t], 0, true);
        }
    }
}

/// A titled card of free text (pronunciation observations, culture); nothing when empty.
fn titled(ui: &mut egui::Ui, title: &str, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    ui.label(caps(title));
    notes(ui, text);
    ui.add_space(4.0);
}

/// The lesson notes: one numbered row per note.
pub(crate) fn notes(ui: &mut egui::Ui, notes: &str) {
    notes_rows(ui, notes, None, None);
}

/// The notes as paragraphs: (number, text, is a table). The lines under a
/// numbered line flow into it; a table's rows stay lines of their own.
fn paragraphs(notes: &str) -> Vec<(Option<usize>, String, bool)> {
    let mut paragraphs: Vec<(Option<usize>, String, bool)> = Vec::new();
    for line in notes.lines() {
        let t = line.trim();
        if t.starts_with('|') {
            match paragraphs.last_mut() {
                Some((_, p, true)) => {
                    p.push('\n');
                    p.push_str(t);
                }
                _ => paragraphs.push((None, t.to_string(), true)),
            }
            continue;
        }
        let numbered = t
            .split_once(". ")
            .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
            .map(|(n, rest)| (n.parse().ok(), rest.to_string()));
        match (numbered, paragraphs.last_mut()) {
            (Some((n, rest)), _) => paragraphs.push((n, rest, false)),
            (None, Some((_, p, false))) if !t.is_empty() && !p.is_empty() => {
                p.push(' ');
                p.push_str(t);
            }
            (None, _) => paragraphs.push((None, t.to_string(), false)),
        }
    }
    paragraphs.retain(|(_, p, _)| !p.is_empty());
    paragraphs
}

/// The notes, one row per paragraph. With `refs` (the dialogue lines each
/// paragraph is about) a row with lines can be pointed at; returns the row
/// under the pointer and whether it was clicked. `lit` is drawn highlighted.
fn notes_rows(ui: &mut egui::Ui, notes: &str, refs: Option<&[Vec<usize>]>, lit: Option<usize>) -> Option<(usize, bool)> {
    let mut event = None;
    theme::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        if notes.trim().is_empty() {
            ui.label(muted(tr("This lesson has no notes.")));
        }
        for (k, (n, p, table)) in paragraphs(notes).iter().enumerate() {
            if *table {
                // Each table keeps its own column widths.
                ui.push_id(k, |ui| theme::markdown(ui, p));
                continue;
            }
            let linked = refs.and_then(|r| r.get(k)).is_some_and(|r| !r.is_empty());
            let row = egui::Frame::new()
                .fill(if linked && lit == Some(k) { HIGHLIGHT() } else { Color32::TRANSPARENT })
                .corner_radius(8)
                .inner_margin(egui::Margin::symmetric(6, 4))
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        if let Some(n) = n {
                            theme::number(ui, *n, true);
                        }
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width());
                            let color = ui.visuals().text_color();
                            ui.add(egui::Label::new(theme::starred(p, 16.0, color)).wrap());
                        });
                    });
                })
                .response;
            if linked {
                let r = ui
                    .interact(row.rect, ui.id().with(("note", k)), egui::Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if r.hovered() {
                    event = Some((k, r.clicked()));
                }
            }
            ui.add_space(2.0);
        }
    });
    event
}

/// Words compared without case, accents or punctuation.
fn words(s: &str) -> Vec<String> {
    crate::practice::tokens(&crate::practice::fold(s))
}

/// The dialogue lines (from 0) a note is about: the ones it cites, as
/// "(sentence 3)", "(frase 3 e 5)", "(lines 2 to 4)", "(línea 3)" (not a
/// line of another lesson: "lesson 44, line 3"); else the lines holding the
/// words before its colon ("Buenos días: good morning..."), or one of the
/// words or phrases there ("Hallo, hoi: ..."), or its first words
/// ("Constipado means...").
pub fn note_lines(note: &str, dialogue: &[Item]) -> Vec<usize> {
    const LINE: [&str; 14] = [
        "sentence", "sentences", "line", "lines", "frase", "frasi", "frases", "riga", "righe", "linea", "linee", "lineas", "oracion",
        "oraciones",
    ];
    const LESSON: [&str; 8] = ["lesson", "lessons", "lezione", "lezioni", "leccion", "lecciones", "les", "lessen"];
    const AND: [&str; 3] = ["and", "e", "y"];
    const TO: [&str; 3] = ["to", "a", "al"];
    let mut cited: Vec<usize> = Vec::new();
    // Within a stretch between brackets, full stops and semicolons: after a
    // line word its numbers count, after a lesson word nothing does.
    for stretch in note.split(['(', ')', ';', '.', ':']) {
        let (mut on, mut foreign, mut range) = (false, false, false);
        for w in words(stretch) {
            if let Ok(n) = w.parse::<usize>() {
                if on && n > 0 {
                    // "2 to 4": the lines after the last one cited, up to this one.
                    let from = if range { cited.last().map_or(n, |&l| l + 2) } else { n };
                    cited.extend((from.min(n)..=n).map(|k| k - 1));
                }
                range = false;
            } else if LINE.contains(&w.as_str()) {
                on = !foreign;
            } else if LESSON.contains(&w.as_str()) {
                (on, foreign) = (false, true);
            } else if on && TO.contains(&w.as_str()) {
                range = true;
            } else if !AND.contains(&w.as_str()) {
                on = false;
            }
        }
    }
    let lines: Vec<Vec<String>> = dialogue.iter().map(|it| words(&it.plain())).collect();
    let holding = |phrase: &[String]| -> Vec<usize> {
        (0..lines.len())
            .filter(|&i| !phrase.is_empty() && lines[i].windows(phrase.len()).any(|w| w == phrase))
            .collect()
    };
    let mut found: Vec<usize> = if !cited.is_empty() {
        cited.into_iter().filter(|&i| i < dialogue.len()).collect()
    } else {
        let head = note.split_once(':').map_or("", |(h, _)| h);
        let whole = words(head);
        let mut v = if whole.len() <= 30 { holding(&whole) } else { Vec::new() };
        if v.is_empty() && whole.len() <= 8 {
            for piece in head.split([',', '/', '…']).map(words) {
                for part in piece.split(|w| ["and", "or", "e", "o", "y", "u"].contains(&w.as_str())) {
                    if part.concat().chars().count() >= 2 {
                        v.extend(holding(part));
                    }
                }
            }
        }
        if v.is_empty() {
            // "Constipado means...", "¿Te gustó? is...": the longest run of
            // opening words found in a line, too short to be a stray "a" or "la".
            let opening = words(note);
            v = (1..=opening.len().min(6))
                .rev()
                .map(|k| &opening[..k])
                .filter(|p| p.concat().chars().count() >= 3)
                .map(|p| holding(p))
                .find(|v| !v.is_empty())
                .unwrap_or_default();
        }
        v
    };
    found.sort_unstable();
    found.dedup();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_wave_comes_with_its_partner_lesson() {
        // SAFETY: the only other reader of this variable is in ignored tests.
        unsafe { std::env::set_var("MAREA_HOME", std::env::temp_dir().join("marea-test-home")) };
        // The French draft course stands in for any language with two courses.
        unsafe { std::env::set_var("MAREA_DRAFTS", "1") };
        let mut app = App::new(&egui::Context::default(), None, None);
        app.saved = crate::store::Saved {
            wave2_gap: 2,
            ..Default::default()
        };
        let ci = app.course_index("core-fr").unwrap();
        let key = |app: &App, i: usize| app.courses[ci].lesson_key(app.courses[ci].lessons[i].number);
        for i in 0..2 {
            let k = key(&app, i);
            app.saved.lessons.entry(k).or_default().completed = Some("2026-01-01".into());
        }
        assert_eq!(app.wave2_for(ci, 2), Some((ci, 0)), "lesson 3 brings lesson 1 back");
        assert!(app.wave2_due(ci).is_empty(), "nothing overdue yet");
        let k = key(&app, 0);
        app.saved.lessons.get_mut(&k).unwrap().wave2 = Some("2026-01-02".into());
        assert_eq!(app.wave2_for(ci, 2), None);

        // The next course of the language carries the earlier one's wave on.
        let mut next = app.courses[ci].clone();
        next.stage = Some(crate::model::Stage::Perfectionnement);
        next.id = "core-fr-perfectionnement".into();
        app.courses.push(next);
        let pi = app.courses.len() - 1;
        for i in 2..4 {
            let k = key(&app, i);
            app.saved.lessons.entry(k).or_default().completed = Some("2026-01-03".into());
        }
        assert_eq!(app.wave2_for(pi, 0), Some((ci, 1)), "lesson 2 comes back with the next course");
        assert_eq!(app.wave2_pending(pi).first(), Some(&(ci, 1)));

        // A course still being written is not finished when all its lessons are done.
        if app.unwritten(ci) {
            for i in 0..app.courses[ci].lessons.len() {
                let k = key(&app, i);
                app.saved.lessons.entry(k).or_default().completed = Some("2026-01-04".into());
            }
            let n = app.courses[ci].lessons.len();
            let by_gap = app.wave2_due(ci);
            assert!(!by_gap.contains(&(n - 1)), "only the gap makes lessons due");
            // Once the next course is started, the rest come back one a day.
            let pk = app.courses[pi].lesson_key(app.courses[pi].lessons[0].number);
            app.saved.lessons.entry(pk).or_default().completed = Some("2026-01-05".into());
            let with_tail = app.wave2_due(ci);
            assert_eq!(with_tail.len(), by_gap.len() + 1, "one more a day");
            let k = key(&app, by_gap[0]);
            app.saved.lessons.get_mut(&k).unwrap().wave2 = Some(day_key(today()));
            assert_eq!(app.wave2_due(ci), by_gap[1..].to_vec(), "no tail lesson once one is rebuilt today");
        }
        // Starting from the later course leaves the earlier one out of it.
        app.saved.lessons.retain(|k, _| !k.starts_with("core-fr#"));
        assert!(app.wave2_pending(pi).iter().all(|&(c, _)| c == pi));
    }

    /// The dialogue lines each note of a built-in lesson is about.
    fn lines_of(course: &str, number: u32, base: &str) -> Vec<Vec<usize>> {
        let (courses, _) = crate::store::builtin_courses();
        let c = courses.iter().find(|c| c.id == course).unwrap();
        let l = c.lessons.iter().find(|l| l.number == number).unwrap();
        paragraphs(l.notes_in(base)).iter().map(|(_, p, _)| note_lines(p, &l.dialogue)).collect()
    }

    #[test]
    fn notes_find_their_lines() {
        // English for Italians, lesson 1: Good morning, A cup of tea, Please,
        // Milk?, Here you are, Thank you.
        assert_eq!(lines_of("core-en", 1, "it"), vec![vec![0, 1], vec![0, 3], vec![0, 2], vec![1], vec![3], vec![4]]);
        // Spanish for English speakers: ¡Buenos días!, ¿Qué le pongo?... with
        // their marks, accents and capitals.
        assert_eq!(
            lines_of("core-es", 1, "en"),
            vec![vec![0, 1], vec![1], vec![2], vec![2], vec![3], vec![5], vec![6]]
        );
        // Dutch for Italians, lesson 2: "Hallo, hoi" names two lines; the
        // whole of "Ja, graag" (lesson 1) one; Alstublieft is in two.
        let nl2 = lines_of("core-nl", 2, "it");
        assert_eq!(nl2[0], vec![0, 1]);
        assert_eq!(nl2[3], vec![2, 4], "Italiaans, Nederlands");
        let nl1 = lines_of("core-nl", 1, "it");
        assert_eq!(nl1[2], vec![0, 3], "Alstublieft");
        assert_eq!(nl1[4], vec![2], "Ja, graag");

        // Cited lines win over the opening words, never another lesson's.
        let d: Vec<Item> = (1..=8)
            .map(|n| Item {
                text: format!("Line number {n}."),
                ..Default::default()
            })
            .collect();
        assert_eq!(note_lines("take off (frasi 7 e 4, note 9 e 17)", &d), vec![3, 6]);
        assert_eq!(note_lines("ăn cơm (frases 2 y 5, notas 2 y 14)", &d), vec![1, 4]);
        assert_eq!(note_lines("Line number 1: see the past (sentences 2 to 4 and 6).", &d), vec![1, 2, 3, 5]);
        assert!(note_lines("As in lesson 44, line 3: the same past.", &d).is_empty());
        assert_eq!(note_lines("Nella frase 3 la stessa parola (Fondamenti, lezione 12).", &d), vec![2]);
        assert_eq!(note_lines("(línea 8; oración 1) ok", &d), vec![0, 7]);
        assert!(note_lines("Le espressioni di questa lezione: nessuna riga", &d).is_empty());
        assert!(note_lines("(sentence 99)", &d).is_empty());
    }

    /// Needs mpv and a downloaded Spanish voice in MAREA_HOME; sound
    /// goes nowhere through MPV_HOME=<dir with "ao=null" in mpv.conf>.
    /// Run on purpose: `cargo test -- --ignored listening`.
    #[test]
    #[ignore]
    fn listening_plays_pauses_and_moves_on() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, None, None);
        assert!(app.voices.ready("es"), "download the Spanish voice first");
        let run = |app: &mut App| {
            app.play_seq("t", "es", vec!["Hola.".into(), "Muy bien.".into()], 0, false);
            let mut seen = Vec::new();
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(30) {
                app.drive_audio(&ctx);
                let s = app.seq.as_ref().unwrap();
                let name = match s.phase {
                    Phase::Wait => "wait",
                    Phase::Play(_) => "play",
                    Phase::Gap(..) => "gap",
                    Phase::Stopped => "stopped",
                    Phase::Done => "done",
                };
                if seen.last().map(|(n, i)| (*n, *i)) != Some((name, s.idx)) {
                    seen.push((name, s.idx));
                }
                if matches!(s.phase, Phase::Done | Phase::Stopped) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            seen
        };
        // Sentence by sentence, with a pause to repeat after each.
        app.saved.pause_steps.insert("t".into());
        let seen = run(&mut app);
        assert_eq!(seen.last().map(|s| s.0), Some("done"), "{seen:?}");
        assert!(seen.contains(&("gap", 0)) && seen.contains(&("play", 1)), "{seen:?}");
        // The default: one recording, the second sentence lit while it plays.
        app.saved.pause_steps.clear();
        let seen = run(&mut app);
        assert_eq!(seen.last().map(|s| s.0), Some("done"), "{seen:?}");
        assert!(seen.contains(&("play", 0)) && seen.contains(&("play", 1)) && !seen.iter().any(|s| s.0 == "gap"), "{seen:?}");
    }
}
