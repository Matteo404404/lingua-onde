//! The guided daily lesson (the listening and reading steps of the method),
//! the sentence-by-sentence voice with pauses for repeating, and the review
//! screen (spaced sentence review and the active "second wave").

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText};

use crate::app::{App, Screen};
use crate::model::{Item, Lesson, lang_name};
use crate::store::{Card, Grade, day_key, today};
use crate::theme::{self, ACCENT, ACCENT_SOFT, BAD, GOOD, HIGHLIGHT, MUTED, WARN, muted};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Step {
    BlindListen,
    ListenBase,
    ListenTarget,
    Read,
    Notes,
    Ex1,
    Ex2,
    ReadAgain,
    FinalListen,
    Previous,
    SecondWave,
}

impl Step {
    pub fn title(self) -> &'static str {
        match self {
            Step::BlindListen => "Blind listening",
            Step::ListenBase => "Listen, reading your language",
            Step::ListenTarget => "Listen, reading the original",
            Step::Read => "Read the dialogue",
            Step::Notes => "Notes",
            Step::Ex1 => "Exercise 1 · Translate",
            Step::Ex2 => "Exercise 2 · Missing words",
            Step::ReadAgain => "Read it again",
            Step::FinalListen => "Last blind listening",
            Step::Previous => "Review the two previous dialogues",
            Step::SecondWave => "Second wave",
        }
    }

    fn short(self) -> &'static str {
        match self {
            Step::BlindListen => "Blind",
            Step::ListenBase => "+ your language",
            Step::ListenTarget => "+ original",
            Step::Read => "Read",
            Step::Notes => "Notes",
            Step::Ex1 => "Ex. 1",
            Step::Ex2 => "Ex. 2",
            Step::ReadAgain => "Read again",
            Step::FinalListen => "Blind again",
            Step::Previous => "2 before",
            Step::SecondWave => "2nd wave",
        }
    }

    fn how(self) -> &'static str {
        match self {
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
                "Read the notes. They explain what you just met; don't try to memorise anything, it settles by itself as you go on."
            }
            Step::Ex1 => "Read each sentence aloud and translate it in your head, then check.",
            Step::Ex2 => "Write the missing words. The translation tells you what the sentence says.",
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
                "Active phase: read the sentence in your language and say it aloud in the language you are learning, then check. Don't worry about mistakes."
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
}

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Wait,
    Play(Instant),
    Gap(Instant, Duration),
    Stopped,
    Done,
}

/// Sentences spoken one after another by the voice.
pub struct Seq {
    pub tag: String,
    pub lang: String,
    pub texts: Vec<String>,
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
}

#[derive(Default)]
pub struct ReviewState {
    pub tab: usize,
    pub queue: Vec<String>,
    pub built: bool,
    pub shown: bool,
    pub practice: Option<(usize, usize)>,
    pub practice_shown: HashSet<usize>,
}

#[derive(PartialEq)]
enum Mark {
    Right,
    Accent,
    Wrong,
}

fn fold(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'ē' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'ī' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ō' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'ū' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'ß' => 's',
            c => c,
        })
        .collect()
}

fn norm(s: &str) -> String {
    s.trim()
        .trim_matches(|c: char| c.is_ascii_punctuation() || "¿¡«»“”‘’。，！？、".contains(c))
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn mark(answer: &str, right: &str) -> Mark {
    let (a, r) = (norm(answer), norm(right));
    if a == r {
        Mark::Right
    } else if !a.is_empty() && fold(&a) == fold(&r) {
        Mark::Accent
    } else {
        Mark::Wrong
    }
}

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
        let mut steps = vec![
            Step::BlindListen,
            Step::ListenBase,
            Step::ListenTarget,
            Step::Read,
            Step::Notes,
            Step::Ex1,
            Step::Ex2,
            Step::ReadAgain,
            Step::FinalListen,
            Step::Previous,
        ];
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
        let answers = lesson.ex2.iter().map(|i| vec![String::new(); i.blanks().len()]).collect();

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
        let msg = format!(
            "Lesson {} done{} See you tomorrow for the next one.",
            lesson.number,
            if first { " — well done!" } else { " again." }
        );
        self.mark_dirty();
        self.screen = Screen::Home;
        self.say(msg);
    }

    // ------------------------------------------------------------ audio

    pub fn play_seq(&mut self, tag: &str, lang: &str, texts: Vec<String>, from: usize, single: bool) {
        self.player.stop();
        if texts.is_empty() {
            self.seq = None;
            return;
        }
        self.seq = Some(Seq {
            tag: tag.to_string(),
            lang: lang.to_string(),
            idx: from.min(texts.len() - 1),
            texts,
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
                    let text = seq.texts[seq.idx].clone();
                    if !self.voices.ready(&seq.lang) {
                        seq.phase = Phase::Stopped;
                    } else if let Some(file) = self.voices.speech_file(&seq.lang, &text) {
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
                            self.voices.prepare(&seq.lang, &[text]);
                        }
                    }
                }
                Phase::Play(t0) => {
                    if !self.player.busy() {
                        let took = t0.elapsed();
                        seq.phase = if seq.single {
                            Phase::Done
                        } else {
                            let gap = if seq.repeat {
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
                        seq.phase = if seq.idx >= seq.texts.len() { Phase::Done } else { Phase::Wait };
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
    fn current(&self, tag: &str) -> Option<usize> {
        self.seq.as_ref().filter(|s| s.tag == tag && s.running()).map(|s| s.idx)
    }

    /// Play/pause controls for a list of sentences: the lesson recording when
    /// there is one, else the voice sentence by sentence, else how to get audio.
    fn listen_panel(&mut self, ui: &mut egui::Ui, tag: &str, lang: &str, items: &[Item], recording: Option<PathBuf>) {
        let voice = self.voices.ready(lang);
        let base = self.base();
        theme::tinted(ACCENT_SOFT).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let use_recording = recording.is_some() && !(self.saved.prefer_voice && voice);
            if let (true, Some(rec)) = (use_recording, recording) {
                let this = self.seq.is_none() && self.player.file.as_ref() == Some(&rec);
                if this && self.player.busy() {
                    ui.horizontal(|ui| {
                        if self.player.controllable {
                            if ui
                                .add(theme::primary(if self.player.paused { "▶ Resume" } else { "⏸ Pause" }))
                                .clicked()
                            {
                                self.player.toggle_pause();
                            }
                            if ui.button("⏪ 5 s").clicked() {
                                self.player.seek(-5.0);
                            }
                        }
                        if ui.button("⏹ Stop").clicked() {
                            self.player.stop();
                        }
                        let (pos, dur) = self.player.position();
                        if dur > 0.0 {
                            theme::bar(ui, (pos / dur) as f32, 220.0, ACCENT);
                            ui.label(muted(format!(
                                "{}:{:02} / {}:{:02}",
                                pos as u32 / 60,
                                pos as u32 % 60,
                                dur as u32 / 60,
                                dur as u32 % 60
                            )));
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        if ui.add(theme::primary("▶ Play the recording")).clicked() {
                            self.seq = None;
                            if let Err(e) = self.player.play(&rec, self.saved.speed) {
                                self.say(e);
                            }
                        }
                        ui.label(muted(format!("🎧 {}", rec.file_name().unwrap_or_default().to_string_lossy())));
                    });
                }
                ui.label(muted("Pause after each sentence and repeat it aloud before going on.").size(13.0));
                return;
            }
            if voice {
                let seq_state = self.seq.as_ref().filter(|s| s.tag == tag).map(|s| (s.phase, s.idx, s.texts.len()));
                ui.horizontal(|ui| match seq_state {
                    Some((phase, idx, n)) if !matches!(phase, Phase::Stopped | Phase::Done) => {
                        if ui.add(theme::primary("⏹ Stop")).clicked() {
                            self.stop_audio();
                        }
                        if ui.button("⏮").on_hover_text("Previous sentence").clicked() {
                            self.play_seq(tag, lang, texts(items), idx.saturating_sub(1), false);
                        }
                        if ui.button("⟲").on_hover_text("Repeat this sentence").clicked() {
                            self.play_seq(tag, lang, texts(items), idx, false);
                        }
                        if ui.button("⏭").on_hover_text("Next sentence").clicked() {
                            self.play_seq(tag, lang, texts(items), idx + 1, false);
                        }
                        ui.label(RichText::new(format!("Sentence {} of {n}", idx + 1)).strong());
                        match phase {
                            Phase::Wait => {
                                ui.spinner();
                                ui.label(muted("preparing the voice…"));
                            }
                            Phase::Play(_) => {
                                ui.label(RichText::new("🔊 listen…").color(ACCENT));
                            }
                            Phase::Gap(until, len) => {
                                if self.saved.repeat_pauses {
                                    ui.label(RichText::new("● repeat aloud now").color(WARN).strong());
                                    let left = until.saturating_duration_since(Instant::now()).as_secs_f32() / len.as_secs_f32().max(0.01);
                                    theme::bar(ui, left, 120.0, WARN);
                                }
                            }
                            _ => {}
                        }
                    }
                    Some((Phase::Done, ..)) => {
                        if ui.add(theme::primary("▶ Listen again")).clicked() {
                            self.play_seq(tag, lang, texts(items), 0, false);
                        }
                        ui.label(RichText::new("✔ Finished").color(GOOD));
                    }
                    _ => {
                        if ui.add(theme::primary("▶ Start listening")).clicked() {
                            self.play_seq(tag, lang, texts(items), 0, false);
                        }
                    }
                });
                if ui
                    .checkbox(&mut self.saved.repeat_pauses, "Pause after each sentence so I can repeat it")
                    .changed()
                {
                    self.mark_dirty();
                    if let Some(s) = self.seq.as_mut() {
                        s.repeat = self.saved.repeat_pauses;
                    }
                }
                return;
            }
            ui.label(RichText::new("No audio for this lesson yet").strong());
            let status = self.voices.status_of(lang);
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
                ui.label(muted(
                    "or link your own recordings in Courses. Meanwhile, read the sentences aloud yourself.",
                ));
            });
        });
    }

    // --------------------------------------------------------- rendering

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
        let fill = if highlight { HIGHLIGHT } else { Color32::TRANSPARENT };
        egui::Frame::new()
            .fill(fill)
            .corner_radius(8)
            .inner_margin(egui::Margin::symmetric(8, 5))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(muted(format!("{n:>2}")).monospace());
                    if can_play {
                        clicked = ui.small_button("▶").on_hover_text("Listen").clicked();
                    }
                    ui.vertical(|ui| {
                        let size = if big { 20.0 } else { 17.0 };
                        if show_target {
                            ui.label(RichText::new(item.plain()).size(size));
                            if self.saved.show_pron && !item.pron.is_empty() {
                                ui.label(RichText::new(&item.pron).size(13.5).italics().color(ACCENT));
                            }
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

    fn dialogue_block(&mut self, ui: &mut egui::Ui, tag: &str, lang: &str, items: &[Item], show_target: bool, show_tr: bool) {
        let cur = self.current(tag);
        let can_play = self.voices.ready(lang);
        let mut play = None;
        for (i, it) in items.iter().enumerate() {
            if self.sentence(ui, i + 1, it, show_target, show_tr, cur == Some(i), can_play, false) {
                play = Some(i);
            }
        }
        if let Some(i) = play {
            self.play_seq(tag, lang, texts(items), i, true);
        }
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

        // Header.
        ui.horizontal(|ui| {
            theme::badge(ui, lesson.level.name(), lesson.level.color());
            ui.label(
                RichText::new(format!("Lesson {} · {}", lesson.number, lesson.title))
                    .size(22.0)
                    .strong(),
            );
            ui.label(muted(format!("— {}", lesson.title_in(&base))).size(16.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕ Close").on_hover_text("Your place in the lesson is kept").clicked() {
                    self.stop_audio();
                    self.session = None;
                    self.screen = Screen::Home;
                }
                ui.label(muted(lang_name(&lang, &base)));
            });
        });
        if self.session.is_none() {
            return;
        }

        // Step chips.
        let mut jump = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (i, st) in steps.iter().enumerate() {
                let (fill, color) = if i == step_i {
                    (ACCENT, Color32::WHITE)
                } else if done[i] {
                    (Color32::from_rgb(214, 238, 222), GOOD)
                } else {
                    (Color32::from_rgb(236, 232, 223), MUTED)
                };
                let label = format!("{}{} {}", if done[i] { "✔ " } else { "" }, i + 1, st.short());
                if ui
                    .add(
                        egui::Button::new(RichText::new(label).size(13.0).color(color))
                            .fill(fill)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(14),
                    )
                    .clicked()
                {
                    jump = Some(i);
                }
            }
        });
        if let Some(i) = jump {
            self.set_step(i);
            return;
        }

        // Footer with the step buttons.
        let last = step_i + 1 == steps.len();
        let mut next = false;
        let mut back = false;
        egui::Panel::bottom("study_footer")
            .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(0, 10)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if step_i > 0 && ui.add(egui::Button::new("◀ Back").min_size(egui::vec2(100.0, 40.0))).clicked() {
                        back = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let label = if last { "✔ Finish the lesson" } else { "Done — next step ▶" };
                        if ui.add(theme::big_primary(label)).clicked() {
                            next = true;
                        }
                        ui.label(muted(format!("Step {} of {}", step_i + 1, steps.len())));
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
            ui.set_max_width(ui.available_width().min(1000.0));
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(format!("{}. {}", step_i + 1, step.title())).size(21.0).strong());
                ui.label(muted(step.how()));
                ui.add_space(6.0);
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
                        ui.add_space(10.0);
                        let cur = self.current("dlg");
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("👂").size(64.0));
                            ui.label(RichText::new("Eyes off the text — listen and repeat aloud").size(18.0).color(MUTED));
                            if let Some(i) = cur {
                                ui.label(RichText::new(format!("sentence {} of {}", i + 1, lesson.dialogue.len())).size(15.0));
                            }
                        });
                        if step == Step::FinalListen {
                            ui.collapsing("Peek at the text (only if you are stuck)", |ui| {
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
                        let notes = lesson.notes_in(&base);
                        theme::tinted(Color32::from_rgb(250, 247, 238)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            if notes.trim().is_empty() {
                                ui.label(muted("This lesson has no notes."));
                            }
                            // A numbered line starts a note; the lines under it flow into it.
                            let mut paragraphs: Vec<String> = Vec::new();
                            for line in notes.lines() {
                                let t = line.trim();
                                let numbered = t.split_once(". ").is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
                                match paragraphs.last_mut() {
                                    Some(p) if !numbered && !t.is_empty() && !p.is_empty() => {
                                        p.push(' ');
                                        p.push_str(t);
                                    }
                                    _ => paragraphs.push(t.to_string()),
                                }
                            }
                            for p in paragraphs.iter().filter(|p| !p.is_empty()) {
                                ui.label(RichText::new(p).size(16.0));
                                ui.add_space(4.0);
                            }
                        });
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
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                theme::badge(ui, pl.level.name(), pl.level.color());
                                ui.label(RichText::new(format!("Lesson {} · {}", pl.number, pl.title)).size(18.0).strong());
                                ui.label(muted(pl.title_in(&base)));
                            });
                            let tag = format!("prev{j}");
                            self.listen_panel(ui, &tag, &lang, &pl.dialogue, prec);
                            self.dialogue_block(ui, &tag, &lang, &pl.dialogue, true, true);
                        }
                    }
                    Step::SecondWave => {
                        if let Some(w) = self.session.as_ref().and_then(|s| s.wave2) {
                            let wl = course.lessons[w].clone();
                            ui.horizontal(|ui| {
                                theme::badge(ui, wl.level.name(), wl.level.color());
                                ui.label(RichText::new(format!("Lesson {} · {}", wl.number, wl.title)).size(18.0).strong());
                            });
                            let mut shown = self.session.as_ref().map(|s| s.shown.clone()).unwrap_or_default();
                            self.active_practice(ui, "wave", &lang, &wl.dialogue, &mut shown);
                            if let Some(s) = self.session.as_mut() {
                                s.shown = shown;
                            }
                        }
                    }
                }
            });
        });
    }

    /// Translation shown; the learner says the original aloud, then reveals it.
    fn active_practice(&mut self, ui: &mut egui::Ui, tag: &str, lang: &str, items: &[Item], shown: &mut HashSet<usize>) {
        let base = self.base();
        let can_play = self.voices.ready(lang);
        if ui.button("Show all").clicked() {
            shown.extend(0..items.len());
        }
        let mut play = None;
        for (i, it) in items.iter().enumerate() {
            theme::card().inner_margin(10).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(it.tr(&base)).size(17.0));
                if shown.contains(&i) {
                    ui.horizontal(|ui| {
                        if can_play && ui.small_button("▶").clicked() {
                            play = Some(i);
                        }
                        ui.label(RichText::new(it.plain()).size(18.0).color(ACCENT).strong());
                    });
                    if self.saved.show_pron && !it.pron.is_empty() {
                        ui.label(RichText::new(&it.pron).italics().color(ACCENT));
                    }
                } else if ui.button("Said it aloud — show me").clicked() {
                    shown.insert(i);
                    if can_play {
                        play = Some(i);
                    }
                }
            });
        }
        if let Some(i) = play {
            self.play_seq(tag, lang, texts(items), i, true);
        }
    }

    fn ex1(&mut self, ui: &mut egui::Ui, lang: &str, lesson: &Lesson) {
        let base = self.base();
        let can_play = self.voices.ready(lang);
        let mut shown = self.session.as_ref().map(|s| s.shown.clone()).unwrap_or_default();
        if ui.button("Show all translations").clicked() {
            shown.extend(0..lesson.ex1.len());
        }
        let mut play = None;
        for (i, it) in lesson.ex1.iter().enumerate() {
            theme::card().inner_margin(10).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(muted(format!("{}", i + 1)));
                    if can_play && ui.small_button("▶").clicked() {
                        play = Some(i);
                    }
                    ui.label(RichText::new(it.plain()).size(18.0));
                });
                if self.saved.show_pron && !it.pron.is_empty() {
                    ui.label(RichText::new(&it.pron).italics().color(ACCENT));
                }
                if shown.contains(&i) {
                    ui.label(RichText::new(format!("→ {}", it.tr(&base))).color(GOOD).size(16.0));
                } else if ui.button("Check my translation").clicked() {
                    shown.insert(i);
                }
            });
        }
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
        for (i, it) in lesson.ex2.iter().enumerate() {
            let blanks = it.blanks();
            if s.answers.len() <= i {
                s.answers.push(vec![String::new(); blanks.len()]);
            }
            theme::card().inner_margin(10).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(muted(format!("{}. {}", i + 1, it.tr(&base))));
                ui.horizontal_wrapped(|ui| {
                    let mut k = 0;
                    for (text, blank) in it.parts() {
                        if !blank {
                            ui.label(RichText::new(text).size(18.0));
                            continue;
                        }
                        let answer = &mut s.answers[i][k];
                        let width = (text.chars().count() as f32 * 11.0 + 30.0).max(60.0);
                        let edit = egui::TextEdit::singleline(answer)
                            .desired_width(width)
                            .font(egui::FontId::proportional(17.0));
                        ui.add_enabled(!s.checked, edit);
                        if s.checked {
                            match mark(answer, &text) {
                                Mark::Right => ui.label(RichText::new("✔").color(GOOD)),
                                Mark::Accent => ui
                                    .label(RichText::new(format!("≈ {text}")).color(WARN))
                                    .on_hover_text("Almost: check the accents"),
                                Mark::Wrong => ui.label(RichText::new(format!("✗ {text}")).color(BAD)),
                            };
                        }
                        k += 1;
                    }
                });
                if s.checked {
                    ui.horizontal(|ui| {
                        if can_play && ui.small_button("▶").clicked() {
                            play = Some(i);
                        }
                        ui.label(RichText::new(it.plain()).color(ACCENT));
                    });
                    if self.saved.show_pron && !it.pron.is_empty() {
                        ui.label(RichText::new(&it.pron).italics().color(ACCENT));
                    }
                }
            });
        }
        ui.horizontal(|ui| {
            if !s.checked {
                if ui.add(theme::primary("Check my answers")).clicked() {
                    let mut right = 0;
                    let mut total = 0;
                    for (i, it) in lesson.ex2.iter().enumerate() {
                        for (k, b) in it.blanks().iter().enumerate() {
                            total += 1;
                            if mark(s.answers[i].get(k).map(|a| a.as_str()).unwrap_or(""), b) != Mark::Wrong {
                                right += 1;
                            }
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
                ui.label(RichText::new(format!("{r} of {t} right")).size(17.0).strong().color(color));
                if ui.button("Try again").clicked() {
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
        ui.heading("Review");
        let due = self.saved.cards_due();
        let ci = self.active();
        let wave = self.wave2_due(ci);
        ui.horizontal(|ui| {
            let tabs = [format!("Daily sentences ({due})"), format!("Second wave ({})", wave.len())];
            for (i, t) in tabs.iter().enumerate() {
                let sel = self.review.tab == i;
                let b = egui::Button::new(RichText::new(t).color(if sel { Color32::WHITE } else { theme::INK })).fill(if sel {
                    ACCENT
                } else {
                    Color32::from_rgb(236, 232, 223)
                });
                if ui.add(b).clicked() {
                    self.review.tab = i;
                    self.review.practice = None;
                    self.stop_audio();
                }
            }
        });
        ui.add_space(6.0);
        if self.review.tab == 0 {
            self.sentence_review(ui);
        } else {
            self.wave_review(ui, ci, &wave);
        }
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
        ui.label(muted("Read the sentence in your language, say it aloud in the language you are learning, then check and be honest with yourself. Sentences you know come back less and less often."));
        let Some(key) = self.review.queue.first().cloned() else {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("✔ Nothing left to review today").size(20.0).color(GOOD));
                let next = self.saved.cards.values().map(|c| c.due.clone()).min();
                ui.label(muted(match next {
                    Some(d) => format!("{} sentences in review · next ones on {d}", self.saved.cards.len()),
                    None => "Sentences arrive here after each finished lesson.".into(),
                }));
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

        ui.label(muted(format!("{} left", self.review.queue.len())));
        let mut grade = None;
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(muted(label).size(13.0));
            ui.add_space(6.0);
            ui.label(RichText::new(item.tr(&base)).size(22.0));
            ui.add_space(10.0);
            if !self.review.shown {
                if ui.add(theme::big_primary("Said it aloud — show me")).clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                    self.review.shown = true;
                    if self.voices.ready(&lang) {
                        self.play_seq(&tag, &lang, vec![item.plain()], 0, true);
                    }
                }
                return;
            }
            ui.horizontal(|ui| {
                if self.voices.ready(&lang) && ui.small_button("▶").clicked() {
                    self.play_seq(&tag, &lang, vec![item.plain()], 0, true);
                }
                ui.label(RichText::new(item.plain()).size(24.0).color(ACCENT).strong());
            });
            if self.saved.show_pron && !item.pron.is_empty() {
                ui.label(RichText::new(&item.pron).italics().color(ACCENT));
            }
            ui.add_space(10.0);
            let card = self.saved.cards.get(&key).cloned().unwrap_or_default();
            ui.horizontal(|ui| {
                for (g, name, color) in [
                    (Grade::Again, "Again", BAD),
                    (Grade::Hard, "Hard", WARN),
                    (Grade::Good, "Good", GOOD),
                    (Grade::Easy, "Easy", ACCENT),
                ] {
                    let mut preview = card.clone();
                    preview.grade(g, today());
                    let when = if preview.interval == 0 {
                        "today".to_string()
                    } else {
                        format!("{} d", preview.interval)
                    };
                    let b = egui::Button::new(RichText::new(format!("{name}\n{when}")).color(Color32::WHITE).strong())
                        .fill(color)
                        .min_size(egui::vec2(110.0, 48.0));
                    if ui.add(b).clicked() {
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
            "The second wave is the active phase of the method: from lesson {} on, each day you also go back to an old lesson, read it in your language and say it aloud in {}. Settings → “Second wave starts” changes the distance.",
            self.saved.wave2_gap + 1,
            lang_name(&course.lang, &base)
        )));
        if let Some((pci, pli)) = self.review.practice.filter(|(c, _)| *c == ci) {
            let l = course.lessons[pli].clone();
            ui.horizontal(|ui| {
                if ui.button("◀ All lessons").clicked() {
                    self.review.practice = None;
                    self.stop_audio();
                }
                theme::badge(ui, l.level.name(), l.level.color());
                ui.label(RichText::new(format!("Lesson {} · {}", l.number, l.title)).size(18.0).strong());
            });
            let mut shown = std::mem::take(&mut self.review.practice_shown);
            self.active_practice(ui, "wave", &course.lang, &l.dialogue, &mut shown);
            self.review.practice_shown = shown;
            if ui.add(theme::big_primary("✔ Second wave done for this lesson")).clicked() {
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
            let _ = pci;
            return;
        }
        let done: Vec<usize> = (0..course.lessons.len())
            .filter(|&i| self.saved.is_done(&course.lesson_key(course.lessons[i].number)))
            .collect();
        if done.is_empty() {
            ui.label(RichText::new("Finish a lesson first: it will be ready here for active practice.").size(16.0));
            return;
        }
        let mut open = None;
        for i in wave.iter().copied().chain(done.iter().copied().filter(|i| !wave.contains(i))) {
            let l = &course.lessons[i];
            let p = self.saved.progress(&course.lesson_key(l.number)).cloned().unwrap_or_default();
            theme::card().inner_margin(egui::Margin::symmetric(14, 8)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    theme::badge(ui, l.level.name(), l.level.color());
                    ui.label(RichText::new(format!("{} · {}", l.number, l.title)).strong());
                    ui.label(muted(l.title_in(&base)));
                    if wave.contains(&i) {
                        ui.label(RichText::new("due today").color(WARN).strong());
                    } else if let Some(d) = &p.wave2 {
                        ui.label(RichText::new(format!("done {d}")).color(GOOD).size(13.0));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Practise actively").clicked() {
                            open = Some(i);
                        }
                    });
                });
            });
        }
        if let Some(i) = open {
            self.review.practice = Some((ci, i));
            self.review.practice_shown.clear();
        }
    }
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

    #[test]
    fn answers_are_forgiving_about_case_and_punctuation() {
        assert!(mark(" Bien ", "bien") == Mark::Right);
        assert!(mark("gracias.", "gracias") == Mark::Right);
        assert!(mark("cafe", "café") == Mark::Accent);
        assert!(mark("", "café") == Mark::Wrong);
        assert!(mark("hola", "adiós") == Mark::Wrong);
    }
}
