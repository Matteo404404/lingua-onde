//! The guided daily lesson (the listening and reading steps of the method),
//! the sentence-by-sentence voice with pauses for repeating, and the review
//! screen (spaced sentence review and the active "second wave").

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText};

use crate::app::{App, Screen};
use crate::model::{Item, Lesson, lang_color, lang_name};
use crate::practice::{Practice, Verdict, compare};
use crate::store::{Card, Grade, day_key, today};
use crate::theme::{
    self, ACCENT, ACCENT_DARK, ACCENT_SOFT, BAD, CARD, GOOD, GOOD_SOFT, HIGHLIGHT, INK, MUTED, TRACK, WARN, WARN_SOFT, bold, caps, medium,
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
            Step::Recap => "Revision · the grammar of the week",
            Step::BlindListen => "Blind listening",
            Step::ListenBase => "Listen, reading your language",
            Step::ListenTarget => "Listen, reading the original",
            Step::Read => "Read the dialogue",
            Step::Notes => "Notes",
            Step::Ex1 => "Exercise 1 · Translate",
            Step::Ex2 => "Exercise 2 · Fill in the missing words",
            Step::Tasks => "Open tasks",
            Step::ReadAgain => "Read it again",
            Step::FinalListen => "Last blind listening",
            Step::Previous => "Review the two previous dialogues",
            Step::SecondWave => "Active wave",
        }
    }

    fn short(self) -> &'static str {
        match self {
            Step::Recap => "Grammar",
            Step::BlindListen => "Blind",
            Step::ListenBase => "+ your language",
            Step::ListenTarget => "+ original",
            Step::Read => "Read",
            Step::Notes => "Notes",
            Step::Ex1 => "Ex. 1",
            Step::Ex2 => "Ex. 2",
            Step::Tasks => "Tasks",
            Step::ReadAgain => "Read again",
            Step::FinalListen => "Blind again",
            Step::Previous => "2 before",
            Step::SecondWave => "Active",
        }
    }

    fn how(self) -> &'static str {
        match self {
            Step::Recap => {
                "A revision lesson: nothing new. Read how the grammar of the last six lessons fits together, with examples you already know, some from the revision dialogue you just heard."
            }
            Step::BlindListen => {
                "Don't look at any text. Listen to the whole dialogue and repeat every sentence aloud in the pause, imitating the voice. It's fine not to understand yet: you are tuning your ear."
            }
            Step::ListenBase => {
                "Listen again while you read the translation in your own language. Keep repeating each sentence aloud: the sound and the meaning start to connect."
            }
            Step::ListenTarget => {
                "Now follow the original text while you listen, and repeat each sentence aloud. Notice how the words are written."
            }
            Step::Read => {
                "Read the dialogue aloud, sentence by sentence, with the translation beside it. The literal sense in brackets shows how the language thinks. Click ▶ to hear a sentence."
            }
            Step::Notes => {
                "Read the pronunciation observations and the notes. They explain what you just met; don't try to memorise anything, it settles by itself as you go on."
            }
            Step::Ex1 => "Read each sentence aloud and translate it in your head, then check.",
            Step::Ex2 => "Write the missing words. The translation tells you what the sentence says.",
            Step::Tasks => {
                "Open tasks have no single right answer. Write or say yours first, then compare it with a model answer and tick the checklist honestly."
            }
            Step::ReadAgain => {
                "Read the dialogue once more, aloud. Try hiding the translation: you should understand almost all of it now."
            }
            Step::FinalListen => {
                "Last time, eyes off the text: listen and repeat each sentence aloud. Notice how much more you understand than at the start."
            }
            Step::Previous => {
                "Read and listen to the two lessons before this one, aloud. Old material comes back easily and keeps growing roots."
            }
            Step::SecondWave => {
                "The active wave: an earlier lesson's dialogue and exercise 1, shown in your language only. Say each sentence aloud in the language you learn (type it if you like), then reveal it and compare. Mistakes are part of it."
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
    /// An older lesson whose second wave is due today.
    pub wave2: Option<usize>,
    pub wave: Practice,
    pub tasks: Practice,
    /// "Go over this lesson again tomorrow", chosen at the end.
    pub again: bool,
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
}

/// The lesson column's widest size: lines stay easy to read.
const STUDY_W: f32 = 900.0;

/// The figured pronunciation line: warm, so it reads apart from the translation.
const FIGURED: Color32 = Color32::from_rgb(156, 98, 36);

fn texts(items: &[Item]) -> Vec<String> {
    items.iter().map(|i| i.plain()).collect()
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
        if wave2.is_some() {
            steps.push(Step::SecondWave);
        }
        let p = self.saved.progress(&key).cloned().unwrap_or_default();
        let mut done = vec![false; steps.len()];
        if p.completed.is_none() {
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
        for j in [li.checked_sub(1), li.checked_sub(2)].into_iter().flatten().chain(wave2) {
            speak.extend(texts(&c.lessons[j].dialogue));
        }
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
            again: p.again,
        });
        self.screen = Screen::Study;
    }

    fn set_step(&mut self, to: usize) {
        self.stop_audio();
        if let Some(s) = self.session.as_mut() {
            s.step = to.min(s.steps.len() - 1);
            s.shown.clear();
            s.hide_tr = s.steps[s.step] == Step::ReadAgain;
        }
    }

    fn complete_step(&mut self) {
        let Some(s) = self.session.as_mut() else { return };
        s.done[s.step] = true;
        let steps_done: Vec<usize> = s.done.iter().enumerate().filter(|(_, d)| **d).map(|(i, _)| i).collect();
        let (ci, li, next) = (s.course, s.lesson, s.step + 1);
        let key = self.courses[ci].lesson_key(self.courses[ci].lessons[li].number);
        let p = self.saved.lessons.entry(key).or_default();
        if p.completed.is_none() {
            p.steps_done = steps_done;
        }
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
        if s.score.1 > 0 {
            p.score = s.score;
        }
        if self.saved.cards_after_lesson {
            for i in 0..lesson.dialogue.len() {
                self.saved.cards.entry(format!("{key}#{i}")).or_default();
            }
        }
        if let (Some(w), Some(wi)) = (s.wave2, s.steps.iter().position(|x| *x == Step::SecondWave)) {
            if s.done[wi] {
                let wl = &c.lessons[w];
                let wkey = c.lesson_key(wl.number);
                self.saved.lessons.entry(wkey.clone()).or_default().wave2 = Some(today.clone());
                for i in 0..wl.dialogue.len() {
                    self.saved.cards.entry(format!("{wkey}#{i}")).or_default();
                }
            }
        }
        self.saved.day().lessons += 1;
        let msg = if s.again {
            format!(
                "Lesson {} done. Tomorrow you'll go over it once more — that's how it settles.",
                lesson.number
            )
        } else {
            format!(
                "Lesson {} done{} See you tomorrow for the next one.",
                lesson.number,
                if first { " — well done!" } else { " again." }
            )
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

    fn start_seq(&mut self, tag: &str, parts: Vec<(String, String)>, stride: usize, from: usize, single: bool) {
        self.player.stop();
        if parts.is_empty() {
            self.seq = None;
            return;
        }
        self.seq = Some(Seq {
            tag: tag.to_string(),
            idx: (from * stride).min(parts.len() - stride),
            parts,
            stride,
            phase: Phase::Wait,
            repeat: self.saved.repeat_pauses && !single,
            single,
            asked: usize::MAX,
        });
    }

    pub fn drive_audio(&mut self, ctx: &egui::Context) {
        let mut error = None;
        if let Some(seq) = self.seq.as_mut() {
            let now = Instant::now();
            match seq.phase {
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
                            match self.player.play(&file, self.saved.speed) {
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
    /// there is one, else the voice sentence by sentence, else how to get audio.
    pub(crate) fn listen_panel(&mut self, ui: &mut egui::Ui, tag: &str, lang: &str, items: &[Item], recording: Option<PathBuf>) {
        let voice = self.voices.ready(lang);
        let base = self.base();
        theme::tinted(ACCENT_SOFT).corner_radius(14).show(ui, |ui| {
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
                        if theme::round_button(ui, glyph, 46.0, ACCENT).clicked() {
                            if self.player.controllable {
                                self.player.toggle_pause();
                            } else {
                                self.player.stop();
                            }
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.label(medium(format!("🎧 {name}")));
                            let (pos, dur) = self.player.position();
                            if dur > 0.0 {
                                ui.horizontal(|ui| {
                                    theme::bar_on(ui, (pos / dur) as f32, 220.0, ACCENT, CARD);
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
                            if ui.add(theme::ghost("⏹ Stop")).clicked() {
                                self.player.stop();
                            }
                            if self.player.controllable && ui.add(theme::ghost("⏪ 5 s")).clicked() {
                                self.player.seek(-5.0);
                            }
                        });
                    } else {
                        if theme::round_button(ui, "▶", 46.0, ACCENT)
                            .on_hover_text("Play the recording")
                            .clicked()
                        {
                            self.seq = None;
                            if let Err(e) = self.player.play(&rec, self.saved.speed) {
                                self.say(e);
                            }
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.label(medium("Play the lesson recording"));
                            ui.label(muted(format!("🎧 {name} · pause after each sentence and repeat it aloud")).size(13.5));
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
                    .map(|s| (s.phase, s.sentence(), s.sentences()));
                let running = matches!(seq_state, Some((p, ..)) if !matches!(p, Phase::Stopped | Phase::Done));
                ui.horizontal(|ui| {
                    let (glyph, hint) = if running { ("■", "Stop") } else { ("▶", "Listen") };
                    if theme::round_button(ui, glyph, 46.0, ACCENT).on_hover_text(hint).clicked() {
                        if running {
                            self.stop_audio();
                        } else {
                            self.play_seq(tag, lang, texts(items), 0, false);
                        }
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;
                        match seq_state {
                            Some((phase, idx, n)) if running => {
                                ui.label(medium(format!("Sentence {} of {n}", idx + 1)));
                                ui.horizontal(|ui| match phase {
                                    Phase::Wait => {
                                        ui.spinner();
                                        ui.label(muted("preparing the voice…"));
                                    }
                                    Phase::Play(_) => {
                                        ui.label(medium("🔊 listen…").color(ACCENT));
                                    }
                                    Phase::Gap(until, len) if self.saved.repeat_pauses => {
                                        ui.label(medium("● repeat aloud now").color(WARN));
                                        let left =
                                            until.saturating_duration_since(Instant::now()).as_secs_f32() / len.as_secs_f32().max(0.01);
                                        theme::bar_on(ui, left, 120.0, WARN, CARD);
                                    }
                                    _ => {}
                                });
                            }
                            Some((Phase::Done, ..)) => {
                                ui.label(medium("✔ Finished").color(GOOD));
                                ui.label(muted("Press ▶ to listen again.").size(13.5));
                            }
                            _ => {
                                ui.label(medium(format!("Listen · {} sentences", items.len())));
                                ui.label(muted("The voice reads each sentence; repeat it aloud in the pause.").size(13.5));
                            }
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let (true, Some((_, idx, _))) = (running, seq_state) {
                            if ui.add(theme::ghost("⏭")).on_hover_text("Next sentence").clicked() {
                                self.play_seq(tag, lang, texts(items), idx + 1, false);
                            }
                            if ui.add(theme::ghost("⟲")).on_hover_text("Repeat this sentence").clicked() {
                                self.play_seq(tag, lang, texts(items), idx, false);
                            }
                            if ui.add(theme::ghost("⏮")).on_hover_text("Previous sentence").clicked() {
                                self.play_seq(tag, lang, texts(items), idx.saturating_sub(1), false);
                            }
                        }
                        if ui
                            .checkbox(&mut self.saved.repeat_pauses, "Pause to repeat")
                            .on_hover_text("The voice waits after each sentence so you can repeat it aloud")
                            .changed()
                        {
                            self.mark_dirty();
                            if let Some(s) = self.seq.as_mut() {
                                s.repeat = self.saved.repeat_pauses;
                            }
                        }
                    });
                });
                return;
            }
            let status = self.voices.status_of(lang);
            ui.horizontal(|ui| {
                theme::icon_circle(ui, "🔈", ACCENT, CARD, 46.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.label(medium("No audio for this lesson yet"));
                    ui.horizontal_wrapped(|ui| {
                        if status.starts_with("Downloading") || status.starts_with("Finishing") {
                            ui.spinner();
                            let mb = self.voices.partial_mb(lang).map(|m| format!(" {m:.0} MB")).unwrap_or_default();
                            ui.label(format!("{status}{mb}"));
                            ui.ctx().request_repaint_after(Duration::from_millis(400));
                        } else {
                            if ui
                                .add(theme::primary(format!("⬇ Get the {} voice (free, ~60 MB)", lang_name(lang, &base))))
                                .clicked()
                            {
                                self.voices.download(lang, ui.ctx().clone());
                            }
                            if status.starts_with("Failed") {
                                ui.label(RichText::new(&status).color(BAD));
                            }
                        }
                    });
                    ui.label(muted("Or link your own recordings in Courses. Meanwhile, read the sentences aloud yourself.").size(13.5));
                });
            });
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
            ui.label(RichText::new(&item.pron).size(14.0).italics().color(ACCENT));
        }
        if let Some(f) = item.figured(&self.base()) {
            ui.label(RichText::new(format!("🔈 {f}")).size(14.0).color(FIGURED))
                .on_hover_text("How it sounds, spelled for your language");
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
        let fill = if highlight { ACCENT_SOFT } else { Color32::TRANSPARENT };
        egui::Frame::new()
            .fill(fill)
            .corner_radius(10)
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_top(|ui| {
                    theme::number(ui, n, highlight);
                    if can_play {
                        clicked = ui.add(theme::ghost("▶").small()).on_hover_text("Listen").clicked();
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
                                        .on_hover_text("The lessons these words come from");
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

    /// The step bar: one segment per step, done in green, the current one
    /// in the accent colour; a click jumps there. Returns the step clicked.
    fn stepper(&self, ui: &mut egui::Ui, steps: &[Step], done: &[bool], at: usize) -> Option<usize> {
        let n = steps.len();
        let gap = 4.0;
        let width = ui.available_width();
        let seg = (width - gap * (n as f32 - 1.0)) / n as f32;
        let labels = seg >= 62.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, if labels { 30.0 } else { 10.0 }), egui::Sense::hover());
        let mut jump = None;
        for (i, st) in steps.iter().enumerate() {
            let x = rect.left() + i as f32 * (seg + gap);
            let hit = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(seg, rect.height()));
            let resp = ui
                .interact(hit, ui.id().with(("step", i)), egui::Sense::click())
                .on_hover_text(st.title())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            let color = if i == at {
                ACCENT
            } else if done[i] {
                GOOD
            } else {
                TRACK
            };
            let color = if resp.hovered() && i != at {
                theme::lerp(color, INK, 0.12)
            } else {
                color
            };
            let bar = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(seg, 6.0));
            ui.painter().rect_filled(bar, 3, color);
            if labels {
                let (text, col) = if i == at {
                    (st.short(), ACCENT)
                } else {
                    (st.short(), if done[i] { GOOD } else { MUTED })
                };
                ui.painter().text(
                    egui::pos2(x + seg / 2.0, rect.top() + 20.0),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::new(
                        11.5,
                        if i == at {
                            theme::medium_family()
                        } else {
                            egui::FontFamily::Proportional
                        },
                    ),
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
                p.circle_stroke(c, r, egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(31, 122, 140, alpha)));
            }
            p.circle_filled(c, 38.0, ACCENT_SOFT);
            p.text(c, egui::Align2::CENTER_CENTER, "👂", egui::FontId::proportional(36.0), ACCENT_DARK);
            ui.label(medium("Eyes off the text — listen and repeat aloud").size(17.0).color(MUTED));
            if let Some(i) = current {
                ui.label(muted(format!("sentence {} of {total}", i + 1)));
            }
            ui.add_space(10.0);
        });
    }

    pub fn study_screen(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.session.as_ref() else {
            ui.label("No lesson open.");
            return;
        };
        let (ci, li, step_i) = (s.course, s.lesson, s.step);
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
                    .on_hover_text("Close — your place in the lesson is kept")
                    .clicked()
                {
                    self.stop_audio();
                    self.session = None;
                    self.screen = Screen::Home;
                }
                ui.add_space(4.0);
                theme::badge(ui, lesson.level.name(), lesson.level.color());
                if lesson.numeral.is_empty() {
                    ui.label(muted(format!("Lesson {}", lesson.number)).size(15.0));
                } else {
                    let label = ui.label(muted(&lesson.numeral).size(15.0));
                    let pron = lesson.numeral_pron.get(&base).map(String::as_str).unwrap_or("");
                    label.on_hover_text(format!("Lesson {} {pron}", lesson.number));
                }
                ui.label(bold(&lesson.title).size(21.0));
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
                        if step_i > 0 && ui.add(theme::secondary("‹  Back").min_size(egui::vec2(100.0, 46.0))).clicked() {
                            back = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let label = if last {
                                "✔  Finish the lesson"
                            } else {
                                "Done — next step  ›"
                            };
                            if ui.add(theme::big_primary(label)).clicked() {
                                next = true;
                            }
                            ui.label(muted(format!("Step {} of {}", step_i + 1, steps.len())));
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
                ui.label(caps(&format!("Step {} of {}", step_i + 1, steps.len())));
                ui.label(bold(step.title()).size(25.0));
                ui.label(muted(step.how()).size(15.0));
                ui.add_space(8.0);
                let rec = self.recording(ci, li);
                if lesson.dialogue.is_empty() && !matches!(step, Step::Previous | Step::SecondWave) {
                    theme::tinted(HIGHLIGHT).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label("📖 This lesson has no text yet: follow it in your book while you listen. You can add the text any time in “My lessons”.");
                    });
                }
                match step {
                    Step::BlindListen | Step::FinalListen => {
                        self.listen_panel(ui, "dlg", &lang, &lesson.dialogue, rec);
                        let cur = self.current("dlg");
                        self.listening_hero(ui, cur, lesson.dialogue.len());
                        if step == Step::FinalListen {
                            egui::CollapsingHeader::new(muted("Peek at the text (only if you are stuck)")).show(ui, |ui| {
                                self.dialogue_block(ui, "dlg", &lang, &lesson.dialogue, true, false);
                            });
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
                        if let Some(s) = self.session.as_mut() {
                            ui.checkbox(&mut s.hide_tr, "Hide the translation");
                        }
                        let hide = self.session.as_ref().is_some_and(|s| s.hide_tr);
                        self.dialogue_block(ui, "dlg", &lang, &lesson.dialogue, true, !hide);
                    }
                    Step::Notes => {
                        let beginner = lesson.level <= crate::model::Level::A2;
                        if beginner
                            && crate::store::pronunciation_key(&lang).is_some()
                            && ui.add(theme::ghost("🔈 Pronunciation guide")).clicked()
                        {
                            self.key_open = true;
                        }
                        titled(ui, "Pronunciation", lesson.sounds_in(&base));
                        if !lesson.sounds_in(&base).trim().is_empty() {
                            ui.label(caps("Notes"));
                        }
                        notes(ui, lesson.notes_in(&base));
                        titled(ui, "Culture", lesson.culture_in(&base));
                    }
                    Step::Recap => notes(ui, lesson.notes_in(&base)),
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
                        let prev: Vec<usize> = [li.checked_sub(2), li.checked_sub(1)].into_iter().flatten().collect();
                        if prev.is_empty() {
                            ui.label(RichText::new("This is the first lesson: nothing to review yet. Just listen to today's dialogue once more if you like.").size(16.0));
                            self.listen_panel(ui, "dlg", &lang, &lesson.dialogue, rec);
                        }
                        for j in prev {
                            let pl = course.lessons[j].clone();
                            let prec = self.recording(ci, j);
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                theme::badge(ui, pl.level.name(), pl.level.color());
                                ui.label(bold(format!("Lesson {} · {}", pl.number, pl.title)).size(18.0));
                                ui.label(muted(pl.title_in(&base)));
                            });
                            let tag = format!("prev{j}");
                            self.listen_panel(ui, &tag, &lang, &pl.dialogue, prec);
                            self.dialogue_block(ui, &tag, &lang, &pl.dialogue, true, true);
                        }
                    }
                    Step::SecondWave => {
                        if let Some(w) = self.session.as_ref().and_then(|s| s.wave2) {
                            let mut p = self.session.as_mut().map(|s| std::mem::take(&mut s.wave)).unwrap_or_default();
                            self.reconstruct(ui, ci, w, &mut p);
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

    fn ex1(&mut self, ui: &mut egui::Ui, lang: &str, lesson: &Lesson) {
        let base = self.base();
        let can_play = self.voices.ready(lang);
        let mut shown = self.session.as_ref().map(|s| s.shown.clone()).unwrap_or_default();
        if ui.add(theme::ghost("Show all translations")).clicked() {
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
                                ui.label(RichText::new(format!("→ {}", it.tr(&base))).color(GOOD).size(16.0));
                            } else if ui.add(theme::secondary("Check my translation")).clicked() {
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
                                            Verdict::Right => ui.label(medium("✔").color(GOOD)),
                                            Verdict::Accents => ui
                                                .label(medium(format!("≈ {text}")).color(WARN))
                                                .on_hover_text("Almost: check the accents"),
                                            Verdict::Wrong => ui.label(medium(format!("✗ {text}")).color(BAD)),
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
                                    ui.label(RichText::new(it.plain()).color(ACCENT));
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
                if ui.add(theme::primary("Check my answers")).clicked() {
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
                    GOOD
                } else if r * 2 >= t {
                    WARN
                } else {
                    BAD
                };
                theme::pill(ui, &format!("{r} of {t} right"), color, theme::soft(color));
                if ui.add(theme::secondary("Try again")).clicked() {
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

    fn card_target(&self, key: &str) -> Option<(usize, usize, usize)> {
        let mut parts = key.rsplitn(3, '#');
        let idx: usize = parts.next()?.parse().ok()?;
        let number: u32 = parts.next()?.parse().ok()?;
        let ci = self.course_index(parts.next()?)?;
        let li = self.courses[ci].lessons.iter().position(|l| l.number == number)?;
        (idx < self.courses[ci].lessons[li].dialogue.len()).then_some((ci, li, idx))
    }

    pub fn review_screen(&mut self, ui: &mut egui::Ui) {
        let due = self.saved.cards_due();
        let ci = self.active();
        let wave = self.wave2_due(ci);
        theme::page(ui, 780.0, |ui| {
            theme::page_title(ui, "Review", "");
            ui.add_space(-6.0);
            // A segmented control for the two kinds of review.
            egui::Frame::new().fill(TRACK).corner_radius(12).inner_margin(4).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let tabs = [
                        format!("Daily sentences · {due}"),
                        format!("Active wave · {}", wave.len()),
                        "Search".to_string(),
                    ];
                    for (i, t) in tabs.iter().enumerate() {
                        let sel = self.review.tab == i;
                        let b = egui::Button::new(medium(t).color(if sel { INK } else { MUTED }))
                            .fill(if sel { CARD } else { Color32::TRANSPARENT })
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
                0 => self.sentence_review(ui),
                1 => self.wave_review(ui, ci, &wave),
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
                    theme::icon_circle(ui, "✔", GOOD, GOOD_SOFT, 60.0);
                    ui.label(bold("Nothing left to review today").size(21.0));
                    let next = self.saved.cards.values().map(|c| c.due.clone()).min();
                    ui.label(muted(match next {
                        Some(d) => format!("{} sentences in review · next ones on {d}", self.saved.cards.len()),
                        None => "Sentences arrive here after each finished lesson.".into(),
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
        let label = format!("{} · lesson {} · {}", lang_name(&lang, &base), lesson.number, lesson.title);
        let tag = format!("card{key}");

        ui.horizontal(|ui| {
            ui.label(muted("Read the sentence in your language, say it aloud in the language you are learning, then check and be honest with yourself.").size(14.0));
        });
        ui.add_space(4.0);
        let mut grade = None;
        theme::card().inner_margin(24).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(caps(&label));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::pill(ui, &format!("{} left", self.review.queue.len()), ACCENT, ACCENT_SOFT);
                });
            });
            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(item.tr(&base)).size(24.0));
                ui.add_space(16.0);
                if !self.review.shown {
                    if ui.add(theme::big_primary("Said it aloud — show me")).clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                        self.review.shown = true;
                        if self.voices.ready(&lang) {
                            self.play_seq(&tag, &lang, vec![item.plain()], 0, true);
                        }
                    }
                    ui.label(muted("or press Space").size(12.5));
                    return;
                }
                ui.horizontal(|ui| {
                    // Centre the answer and its ▶ together.
                    let w = (item.plain().chars().count() as f32 * 14.0 + 40.0).min(ui.available_width());
                    ui.add_space(((ui.available_width() - w) / 2.0).max(0.0));
                    if self.voices.ready(&lang) && ui.add(theme::ghost("▶")).clicked() {
                        self.play_seq(&tag, &lang, vec![item.plain()], 0, true);
                    }
                    ui.label(bold(item.plain()).size(25.0).color(ACCENT));
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
                        (Grade::Again, "Again", BAD),
                        (Grade::Hard, "Hard", WARN),
                        (Grade::Good, "Good", GOOD),
                        (Grade::Easy, "Easy", ACCENT),
                    ])
                    .enumerate()
                {
                    let mut preview = card.clone();
                    preview.grade(g, today());
                    let when = if preview.interval == 0 {
                        "today".to_string()
                    } else {
                        format!("in {} d", preview.interval)
                    };
                    let b = egui::Button::new(medium(format!("{name}\n{when}")).color(color))
                        .fill(theme::soft(color))
                        .stroke(egui::Stroke::new(1.0, theme::lerp(color, Color32::WHITE, 0.6)))
                        .corner_radius(12);
                    let w = col.available_width();
                    let resp = col.add_sized(egui::vec2(w, 56.0), b).on_hover_text(format!("Key {}", k + 1));
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

    fn wave_review(&mut self, ui: &mut egui::Ui, ci: usize, wave: &[usize]) {
        let base = self.base();
        let course = self.courses[ci].clone();
        ui.label(muted(format!(
            "The active wave is the second half of the method: from lesson {} on, each day you also rebuild an earlier lesson — you see it in your language and say it aloud in {}. After the last lesson the remaining ones come back one a day. Settings → “Second wave starts” changes the distance.",
            self.saved.wave2_gap + 1,
            lang_name(&course.lang, &base)
        )).size(14.0));
        ui.add_space(6.0);
        if let Some((_, pli)) = self.review.practice.filter(|(c, _)| *c == ci) {
            let l = course.lessons[pli].clone();
            if ui.add(theme::secondary("‹  All lessons")).clicked() {
                self.review.practice = None;
                self.stop_audio();
                return;
            }
            let mut p = std::mem::take(&mut self.review.practice_state);
            self.reconstruct(ui, ci, pli, &mut p);
            self.review.practice_state = p;
            if ui.add(theme::big_primary("✔  Second wave done for this lesson")).clicked() {
                let key = course.lesson_key(l.number);
                self.saved.lessons.entry(key.clone()).or_default().wave2 = Some(day_key(today()));
                for i in 0..l.dialogue.len() {
                    self.saved.cards.entry(format!("{key}#{i}")).or_default();
                }
                self.saved.day().reviews += 1;
                self.review.practice = None;
                self.stop_audio();
                self.mark_dirty();
                self.say(format!("Second wave of lesson {} done.", l.number));
            }
            return;
        }
        let done: Vec<usize> = (0..course.lessons.len())
            .filter(|&i| self.saved.is_done(&course.lesson_key(course.lessons[i].number)))
            .collect();
        if done.is_empty() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Finish a lesson first: it will be ready here for active practice.").size(16.0));
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
                                theme::primary("Practise")
                            } else {
                                theme::ghost("Practise")
                            };
                            if ui.add(btn).clicked() {
                                open = Some(i);
                            }
                            if due {
                                theme::pill(ui, "due today", WARN, WARN_SOFT);
                            } else if let Some(d) = &p.wave2 {
                                theme::pill(ui, &format!("done {d}"), GOOD, GOOD_SOFT);
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
            ui.label(bold("How did it go?").size(18.0));
            if struggled {
                ui.label(muted(
                    "The exercises were hard today. Going over this lesson again tomorrow often makes it click.",
                ));
            }
            ui.radio_value(&mut s.again, false, "Fine — the next lesson tomorrow");
            ui.radio_value(&mut s.again, true, "Hard — go over this lesson again tomorrow");
        });
    }

    /// Every sentence and word of the language being studied, all courses,
    /// searched in either language, accents ignored.
    fn search(&mut self, ui: &mut egui::Ui) {
        let base = self.base();
        let lang = self.courses.get(self.active()).map(|c| c.lang.clone()).unwrap_or_default();
        ui.add(
            egui::TextEdit::singleline(&mut self.review.search)
                .hint_text(format!("A word in {} or in your language", lang_name(&lang, &base)))
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(17.0)),
        );
        let q = crate::practice::fold(self.review.search.trim());
        if q.chars().count() < 2 {
            ui.label(muted(
                "Type two letters or more. The search covers the words and dialogues of every course in this language.",
            ));
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
        ui.label(muted(format!(
            "{} found{}",
            hits.len(),
            if hits.len() > 80 { " · the first 80" } else { "" }
        )));
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
                            ui.label(muted(format!("{} · lesson {n}", self.course_label(*ci))).size(12.0));
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

/// The lesson notes: one numbered row per note; the lines under a numbered
/// line flow into it.
/// A titled card of free text (pronunciation observations, culture); nothing when empty.
fn titled(ui: &mut egui::Ui, title: &str, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    ui.label(caps(title));
    notes(ui, text);
    ui.add_space(4.0);
}

pub(crate) fn notes(ui: &mut egui::Ui, notes: &str) {
    theme::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        if notes.trim().is_empty() {
            ui.label(muted("This lesson has no notes."));
        }
        let mut paragraphs: Vec<(Option<usize>, String)> = Vec::new();
        for line in notes.lines() {
            let t = line.trim();
            let numbered = t
                .split_once(". ")
                .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
                .map(|(n, rest)| (n.parse().ok(), rest.to_string()));
            match (numbered, paragraphs.last_mut()) {
                (Some((n, rest)), _) => paragraphs.push((n, rest)),
                (None, Some((_, p))) if !t.is_empty() && !p.is_empty() => {
                    p.push(' ');
                    p.push_str(t);
                }
                (None, _) => paragraphs.push((None, t.to_string())),
            }
        }
        for (n, p) in paragraphs.iter().filter(|(_, p)| !p.is_empty()) {
            ui.horizontal_top(|ui| {
                if let Some(n) = n {
                    theme::number(ui, *n, true);
                }
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(p).size(16.0));
                });
            });
            ui.add_space(8.0);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_wave_comes_with_its_partner_lesson() {
        // SAFETY: the only other reader of this variable is in ignored tests.
        unsafe { std::env::set_var("LINGUA_ONDE_HOME", std::env::temp_dir().join("lingua-onde-test-home")) };
        let mut app = App::new(&egui::Context::default(), None, None);
        app.saved = crate::store::Saved {
            wave2_gap: 2,
            ..Default::default()
        };
        let ci = app.course_index("core-es").unwrap();
        let key = |app: &App, i: usize| app.courses[ci].lesson_key(app.courses[ci].lessons[i].number);
        for i in 0..2 {
            let k = key(&app, i);
            app.saved.lessons.entry(k).or_default().completed = Some("2026-01-01".into());
        }
        assert_eq!(app.wave2_for(ci, 2), Some(0), "lesson 3 brings lesson 1 back");
        assert!(app.wave2_due(ci).is_empty(), "nothing overdue yet");
        let k = key(&app, 0);
        app.saved.lessons.get_mut(&k).unwrap().wave2 = Some("2026-01-02".into());
        assert_eq!(app.wave2_for(ci, 2), None);
    }

    /// Needs mpv and a downloaded Spanish voice in LINGUA_ONDE_HOME; sound
    /// goes nowhere through MPV_HOME=<dir with "ao=null" in mpv.conf>.
    /// Run on purpose: `cargo test -- --ignored listening`.
    #[test]
    #[ignore]
    fn listening_plays_pauses_and_moves_on() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, None, None);
        assert!(app.voices.ready("es"), "download the Spanish voice first");
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
        let names: Vec<&str> = seen.iter().map(|(n, _)| *n).collect();
        assert_eq!(names.last(), Some(&"done"), "{seen:?}");
        assert!(seen.contains(&("gap", 0)) && seen.contains(&("play", 1)), "{seen:?}");
    }
}
