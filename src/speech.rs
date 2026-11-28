//! Speech recognition: what the learner says aloud, recorded from the
//! default microphone, transcribed offline by whisper.cpp (`whisper-cli`,
//! run as a helper program like the voice engine) and set against the
//! sentence it should be. The engine comes from this version's GitHub release
//! (built by release.yml), the model from Hugging Face; both are downloaded
//! on request into the data folder's `speech/`. A `whisper-cli` on the PATH
//! serves a build from source.
//!
//! One recording at a time: the state is shared by every button and polled
//! each frame, like the voices and the updater.

// shortcut: until the lesson and placement screens call `check_button`; remove then.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui::{self, RichText};

use crate::audio::{command, curl, downloading, find_program, untar};
use crate::i18n::{tr, trf};
use crate::practice::{fold, tokens};
use crate::theme::{self, BAD, GOOD, medium, muted};

/// whisper.cpp's multilingual small model, 5-bit. The base model misheard
/// correctly spoken European Portuguese (scores of 0.2–0.8 on piper's
/// Lisbon voice); small gets those sentences right, in about 5 s each.
const MODEL: &str = "ggml-small-q5_1.bin";
const MODEL_BYTES: u64 = 190_085_487;
const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-small-q5_1.bin";
/// The engine (2–4 MB) and the model together, for the download prompt.
pub const DOWNLOAD_MB: u32 = 195;

/// The longest a transcription may take before it is given up.
const TIMEOUT: Duration = Duration::from_secs(120);

pub fn dir() -> PathBuf {
    crate::store::data_dir().join("speech")
}

/// The engine archive for this computer, as release.yml names it.
fn engine_asset() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "marea-speech-linux-x86_64.tar.gz",
        ("windows", "x86_64") => "marea-speech-windows-x86_64.zip",
        ("macos", "aarch64") => "marea-speech-macos-arm64.tar.gz",
        ("macos", "x86_64") => "marea-speech-macos-x86_64.tar.gz",
        _ => return None,
    })
}

/// The downloaded engine, else one on the PATH (or beside the app).
fn engine() -> Option<PathBuf> {
    let own = dir().join(if cfg!(windows) { "whisper-cli.exe" } else { "whisper-cli" });
    if own.is_file() { Some(own) } else { find_program("whisper-cli") }
}

fn model() -> PathBuf {
    dir().join(MODEL)
}

fn model_ready() -> bool {
    std::fs::metadata(model()).is_ok_and(|m| m.len() == MODEL_BYTES)
}

/// Whisper's code for a course language, when it can listen to it.
fn whisper_code(lang: &str) -> Option<&'static str> {
    Some(match lang {
        "it" => "it",
        "en" => "en",
        "es" => "es",
        "pt" => "pt",
        "nl" => "nl",
        "de" => "de",
        "fr" => "fr",
        "vi" => "vi",
        "zh" => "zh",
        "la" => "la",
        _ => return None,
    })
}

/// Speech recognition knows this language (installed or not).
pub fn supported(lang: &str) -> bool {
    whisper_code(lang).is_some()
}

/// Set once the engine and the model are found, so buttons drawn every frame
/// do not look at the disk each time.
static READY: AtomicBool = AtomicBool::new(false);

/// The engine and the model are installed and the language is known.
pub fn available(lang: &str) -> bool {
    if !supported(lang) {
        return false;
    }
    if !READY.load(Ordering::Relaxed) && engine().is_some() && model_ready() {
        READY.store(true, Ordering::Relaxed);
    }
    READY.load(Ordering::Relaxed)
}

// ------------------------------------------------------------- download

/// What the download is doing ("" = idle), as the voices report it: a
/// running download ends with an ellipsis (`audio::downloading`).
static STATUS: Mutex<String> = Mutex::new(String::new());

pub fn status() -> String {
    STATUS.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Megabytes downloaded so far for a running download.
pub fn partial_mb() -> Option<f64> {
    let meta = std::fs::metadata(dir().join("download.part")).ok()?;
    Some(meta.len() as f64 / 1_048_576.0)
}

/// Downloads the engine and the model in the background.
pub fn prepare(ctx: &egui::Context) {
    if downloading(&status()) {
        return;
    }
    let ctx = ctx.clone();
    let set = move |s: &str| {
        *STATUS.lock().unwrap_or_else(|e| e.into_inner()) = s.to_string();
        ctx.request_repaint();
    };
    set(tr("Starting…"));
    std::thread::spawn(move || match download(&set) {
        Ok(()) => set(tr("Ready")),
        Err(e) => set(&trf("Failed: {0}", &[&e])),
    });
}

fn download(set: &dyn Fn(&str)) -> Result<(), String> {
    let dir = dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let part = dir.join("download.part");
    let result = (|| {
        if engine().is_none() {
            set(tr("Downloading the speech engine…"));
            let file = engine_asset().ok_or(tr("no speech engine is published for this computer"))?;
            let repo = crate::update::REPO;
            // A build from source has no version: the latest release's engine.
            let url = match crate::update::current() {
                Some(tag) => format!("https://github.com/{repo}/releases/download/{tag}/{file}"),
                None => format!("https://github.com/{repo}/releases/latest/download/{file}"),
            };
            curl(&url, &part)?;
            untar(&part, &dir, &[])?;
            if engine().is_none() {
                return Err(tr("the speech engine is missing from the download").to_string());
            }
        }
        if !model_ready() {
            set(&trf("Downloading the speech model ({0} MB)…", &[&(MODEL_BYTES / 1_000_000)]));
            curl(MODEL_URL, &part)?;
            if std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0) != MODEL_BYTES {
                return Err(tr("the download is incomplete").to_string());
            }
            std::fs::rename(&part, model()).map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    let _ = std::fs::remove_file(&part);
    result
}

// ------------------------------------------------------------ recording

/// Records the default input device until `stop` is set or `max` passes;
/// returns mono samples (-1..1) and their rate.
fn record(stop: &AtomicBool, max: Duration) -> Result<(Vec<f32>, u32), String> {
    let no_mic = || tr("No microphone found. Connect one, or check the system's sound settings.").to_string();
    let device = cpal::default_host().default_input_device().ok_or_else(no_mic)?;
    let supported = device.default_input_config().map_err(|e| trf("The microphone cannot be opened: {0}", &[&e]))?;
    let config = supported.config();
    let samples: Arc<Mutex<Vec<f32>>> = Arc::default();
    let failed: Arc<Mutex<Option<String>>> = Arc::default();
    let (s, f) = (samples.clone(), failed.clone());
    let stream = match supported.sample_format() {
        cpal::SampleFormat::I8 => input::<i8>(&device, &config, s, f),
        cpal::SampleFormat::I16 => input::<i16>(&device, &config, s, f),
        cpal::SampleFormat::I32 => input::<i32>(&device, &config, s, f),
        cpal::SampleFormat::U8 => input::<u8>(&device, &config, s, f),
        cpal::SampleFormat::U16 => input::<u16>(&device, &config, s, f),
        cpal::SampleFormat::F32 => input::<f32>(&device, &config, s, f),
        cpal::SampleFormat::F64 => input::<f64>(&device, &config, s, f),
        other => return Err(trf("The microphone cannot be opened: {0}", &[&format!("{other:?}")])),
    }
    .map_err(|e| trf("The microphone cannot be opened: {0}", &[&e]))?;
    stream.play().map_err(|e| trf("The microphone cannot be opened: {0}", &[&e]))?;
    let start = Instant::now();
    while !stop.load(Ordering::Relaxed) && start.elapsed() < max && failed.lock().is_ok_and(|f| f.is_none()) {
        std::thread::sleep(Duration::from_millis(30));
    }
    drop(stream);
    if let Some(e) = failed.lock().ok().and_then(|f| f.clone()) {
        return Err(trf("The microphone stopped: {0}", &[&e]));
    }
    let samples = std::mem::take(&mut *samples.lock().unwrap_or_else(|e| e.into_inner()));
    Ok((samples, config.sample_rate))
}

/// An input stream that mixes every frame to mono into `samples`.
fn input<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    failed: Arc<Mutex<Option<String>>>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = (config.channels as usize).max(1);
    device.build_input_stream(
        config.clone(),
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            if let Ok(mut out) = samples.lock() {
                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().map(|&x| <f32 as cpal::FromSample<T>>::from_sample_(x)).sum();
                    out.push(sum / frame.len() as f32);
                }
            }
        },
        move |e: cpal::Error| {
            use cpal::ErrorKind::{DeviceChanged, RealtimeDenied, Xrun};
            if !matches!(e.kind(), DeviceChanged | Xrun | RealtimeDenied) {
                if let Ok(mut f) = failed.lock() {
                    f.get_or_insert(e.to_string());
                }
            }
        },
        None,
    )
}

/// Mono samples at `rate` to whisper's input: 16 kHz, 16-bit, the loudest
/// point raised to 90% (a quiet microphone is heard as well as a loud one).
fn to_16k(x: &[f32], rate: u32) -> Vec<i16> {
    const OUT: f64 = 16_000.0;
    let step = rate as f64 / OUT;
    let n = (x.len() as f64 / step).floor() as usize;
    // Band-limited interpolation: a Hann-windowed sinc, its cutoff just under
    // the lower of the two Nyquist frequencies, so nothing folds back.
    let cutoff = 0.45 / step.max(1.0);
    let half = (8.0 * step.max(1.0)).ceil();
    let mut y: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f64 * step;
            let from = ((t - half).floor() as isize + 1).max(0) as usize;
            let to = ((t + half).floor() as usize).min(x.len() - 1);
            let (mut acc, mut sum) = (0.0f64, 0.0f64);
            for (k, &v) in x.iter().enumerate().take(to + 1).skip(from) {
                let d = t - k as f64;
                let a = std::f64::consts::PI * 2.0 * cutoff * d;
                let sinc = if a.abs() < 1e-9 { 1.0 } else { a.sin() / a };
                let w = sinc * 0.5 * (1.0 + (std::f64::consts::PI * d / half).cos());
                acc += v as f64 * w;
                sum += w;
            }
            if sum.abs() > 1e-9 { (acc / sum) as f32 } else { 0.0 }
        })
        .collect();
    let peak = y.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.0 {
        let gain = (0.9 / peak).min(20.0);
        y.iter_mut().for_each(|s| *s *= gain);
    }
    y.iter().map(|s| (s.clamp(-1.0, 1.0) * 32767.0) as i16).collect()
}

/// A 16 kHz mono 16-bit PCM WAV file.
fn wav(pcm: &[i16]) -> Vec<u8> {
    let data = (pcm.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&16_000u32.to_le_bytes());
    out.extend_from_slice(&32_000u32.to_le_bytes()); // bytes per second
    out.extend_from_slice(&2u16.to_le_bytes()); // bytes per frame
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in pcm {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

// --------------------------------------------------------- transcription

/// What whisper hears in a WAV file, in `lang`.
pub fn transcribe(lang: &str, file: &Path) -> Result<String, String> {
    let code = whisper_code(lang).ok_or(tr("Speech recognition does not know this language"))?;
    let engine = engine().ok_or(tr("The speech engine is not installed"))?;
    if !model_ready() {
        return Err(tr("The speech model is not installed").to_string());
    }
    if !file.is_file() {
        return Err(format!("{}: {}", file.display(), std::io::Error::from(std::io::ErrorKind::NotFound)));
    }
    let dir = dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let id = std::process::id();
    let (text, log) = (dir.join(format!("heard-{id}.txt")), dir.join(format!("whisper-{id}.log")));
    let file_for = |p: &Path| std::fs::File::create(p).map_err(|e| e.to_string());
    let mut c = command(engine);
    c.arg("-m").arg(model()).arg("-f").arg(file).args(["-l", code, "-nt", "-np"]);
    if code == "zh" {
        // Only the script: without it the base model often writes Mandarin
        // in traditional characters. It says nothing of the expected words.
        c.args(["--prompt", "以下是普通话的句子。"]);
    }
    let mut child = c
        .stdin(Stdio::null())
        .stdout(file_for(&text)?)
        .stderr(file_for(&log)?)
        .spawn()
        .map_err(|e| trf("could not start the speech engine: {0}", &[&e]))?;
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if start.elapsed() < TIMEOUT => std::thread::sleep(Duration::from_millis(50)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(tr("The speech engine took too long").to_string());
            }
        }
    };
    let heard = std::fs::read_to_string(&text).unwrap_or_default();
    let why = std::fs::read_to_string(&log).unwrap_or_default();
    let _ = std::fs::remove_file(&text);
    let _ = std::fs::remove_file(&log);
    if !status.success() {
        // Its error line, else its last words.
        let lines: Vec<&str> = why.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        let line = lines.iter().find(|l| l.to_lowercase().contains("error")).or(lines.last()).copied().unwrap_or_default();
        return Err(trf("the speech engine failed: {0}", &[&format!("{status} {line}")]));
    }
    Ok(clean(&heard))
}

/// One line of text, without whisper's notes such as "[BLANK_AUDIO]" or "(music)".
fn clean(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = (depth - 1).max(0),
            c if depth == 0 => out.push(if c.is_whitespace() { ' ' } else { c }),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// --------------------------------------------------------------- scoring

/// A sentence said aloud, set against the one it should be.
#[derive(Clone, Debug, PartialEq)]
pub struct Attempt {
    /// What the recognizer heard.
    pub heard: String,
    /// Matched words over expected words, 0 to 1.
    pub score: f32,
    /// The expected words in order (Chinese: characters), each with whether
    /// it was heard.
    pub words: Vec<(String, bool)>,
}

/// 0 to 20 written out, for whisper writing "3" for a spoken number;
/// alternatives (genders) apart with `|`. Chinese is made by `zh_number`.
const NUMBERS: &[(&str, [&str; 21])] = &[
    ("it", ["zero", "uno|una|un", "due", "tre", "quattro", "cinque", "sei", "sette", "otto", "nove", "dieci", "undici", "dodici", "tredici", "quattordici", "quindici", "sedici", "diciassette", "diciotto", "diciannove", "venti"]),
    ("en", ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty"]),
    ("es", ["cero", "uno|una|un", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez", "once", "doce", "trece", "catorce", "quince", "dieciséis", "diecisiete", "dieciocho", "diecinueve", "veinte"]),
    ("pt", ["zero", "um|uma", "dois|duas", "três", "quatro", "cinco", "seis", "sete", "oito", "nove", "dez", "onze", "doze", "treze", "catorze|quatorze", "quinze", "dezasseis|dezesseis", "dezassete|dezessete", "dezoito", "dezanove|dezenove", "vinte"]),
    ("fr", ["zéro", "un|une", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze", "douze", "treize", "quatorze", "quinze", "seize", "dix-sept", "dix-huit", "dix-neuf", "vingt"]),
    ("de", ["null", "eins|ein|eine|einen|einem|einer", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun", "zehn", "elf", "zwölf", "dreizehn", "vierzehn", "fünfzehn", "sechzehn", "siebzehn", "achtzehn", "neunzehn", "zwanzig"]),
    ("nl", ["nul", "een|één", "twee", "drie", "vier", "vijf", "zes", "zeven", "acht", "negen", "tien", "elf", "twaalf", "dertien", "veertien", "vijftien", "zestien", "zeventien", "achttien", "negentien", "twintig"]),
    ("vi", ["không", "một", "hai", "ba", "bốn|tư", "năm", "sáu", "bảy", "tám", "chín", "mười", "mười một", "mười hai", "mười ba", "mười bốn", "mười lăm", "mười sáu", "mười bảy", "mười tám", "mười chín", "hai mươi"]),
    ("la", ["nihil", "unus|una|unum", "duo|duae", "tres|tria", "quattuor", "quinque", "sex", "septem", "octo", "novem", "decem", "undecim", "duodecim", "tredecim", "quattuordecim", "quindecim", "sedecim", "septendecim", "duodeviginti", "undeviginti", "viginti"]),
];

/// 0 to 99 in Chinese characters ("两" too for 2).
fn zh_number(n: usize) -> Option<String> {
    const D: [&str; 10] = ["零", "一", "二|两", "三", "四", "五", "六", "七", "八", "九"];
    let plain = |d: usize| D[d].split('|').next().unwrap_or_default();
    Some(match n {
        0..=9 => D[n].to_string(),
        10 => "十".into(),
        11..=19 => format!("十{}", plain(n % 10)),
        20..=99 if n % 10 == 0 => format!("{}十", plain(n / 10)),
        20..=99 => format!("{}十{}", plain(n / 10), plain(n % 10)),
        _ => return None,
    })
}

/// A number heard as digits, written out in `lang` the way the expected
/// sentence has it (the gender it uses), else in its first form.
fn spell_number(lang: &str, digits: &str, expected: &[String]) -> Option<Vec<String>> {
    let n: usize = digits.parse().ok()?;
    let forms = if lang == "zh" {
        zh_number(n)?
    } else {
        NUMBERS.iter().find(|(l, _)| *l == lang)?.1.get(n)?.to_string()
    };
    let alternatives: Vec<Vec<String>> = forms.split('|').map(tokens).collect();
    let used = alternatives.iter().find(|a| a.first().is_some_and(|w| expected.iter().any(|e| fold(e) == fold(w))));
    used.or(alternatives.first()).cloned()
}

/// Character edit distance over the longer length, 0 to 1.
fn similarity(a: &str, b: &str) -> f32 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let longest = a.len().max(b.len());
    if longest == 0 {
        return 1.0;
    }
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut diag = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let next = (row[j] + 1).min(row[j - 1] + 1).min(diag + usize::from(a[i - 1] != b[j - 1]));
            diag = row[j];
            row[j] = next;
        }
    }
    1.0 - row[b.len()] as f32 / longest as f32
}

/// An expected word heard: the same, the same without accents or tone marks,
/// or (4 letters or more) close enough to be the same word misheard.
fn same(expected: &str, heard: &str) -> bool {
    let (e, h) = (expected.to_lowercase(), heard.to_lowercase());
    if e == h {
        return true;
    }
    let (e, h) = (fold(&e), fold(&h));
    e == h || (e.chars().count() >= 4 && similarity(&e, &h) >= 0.75)
}

/// Scores `heard` against `expected`: the expected words (Chinese:
/// characters) found in order in what was heard, over all expected words.
pub fn score(lang: &str, expected: &str, heard: &str) -> Attempt {
    let want = tokens(expected);
    let mut got = Vec::new();
    for t in tokens(heard) {
        match t.chars().all(|c| c.is_ascii_digit()).then(|| spell_number(lang, &t, &want)).flatten() {
            Some(words) => got.extend(words),
            None => got.push(t),
        }
    }
    // A word heard in two pieces ("Buon giorno" for "Buongiorno") is one.
    let mut joined: Vec<String> = Vec::new();
    let mut k = 0;
    while k < got.len() {
        let pair = got.get(k + 1).map(|next| format!("{}{next}", got[k]));
        match pair {
            Some(p) if !want.iter().any(|w| same(w, &got[k])) && want.iter().any(|w| fold(w) == fold(&p)) => {
                joined.push(p);
                k += 2;
            }
            _ => {
                joined.push(got[k].clone());
                k += 1;
            }
        }
    }
    let got = joined;
    // Longest common subsequence, words matching as `same` says.
    let (n, m) = (want.len(), got.len());
    let mut dp = vec![vec![0u16; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if same(&want[i], &got[j]) { dp[i + 1][j + 1] + 1 } else { dp[i + 1][j].max(dp[i][j + 1]) };
        }
    }
    let mut words: Vec<(String, bool)> = want.iter().map(|w| (w.clone(), false)).collect();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if same(&want[i], &got[j]) && dp[i][j] == dp[i + 1][j + 1] + 1 {
            words[i].1 = true;
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    let matched = words.iter().filter(|(_, ok)| *ok).count();
    Attempt {
        heard: heard.to_string(),
        score: if n == 0 { 0.0 } else { matched as f32 / n as f32 },
        words,
    }
}

// ----------------------------------------------------------------- takes

#[derive(Clone)]
enum Phase {
    Listening { since: Instant, max: Duration, stop: Arc<AtomicBool> },
    Checking,
    /// The result, and whether a button has handed it to its caller.
    Done(Result<Attempt, String>, bool),
}

/// Every sentence's last take, by button; at most one listening or checking.
static TAKES: LazyLock<Mutex<HashMap<egui::Id, Phase>>> = LazyLock::new(Mutex::default);

fn set_phase(id: egui::Id, phase: Phase) {
    TAKES.lock().unwrap_or_else(|e| e.into_inner()).insert(id, phase);
}

fn busy() -> bool {
    TAKES.lock().unwrap_or_else(|e| e.into_inner()).values().any(|p| !matches!(p, Phase::Done(..)))
}

/// Records, transcribes and scores one take in the background.
fn start(id: egui::Id, lang: &str, expected: &str, ctx: &egui::Context) {
    // Time to say it slowly, then the recording stops by itself.
    let words = tokens(expected).len() as u64;
    let max = Duration::from_millis((3000 + 700 * words).min(30_000));
    let stop = Arc::new(AtomicBool::new(false));
    set_phase(id, Phase::Listening { since: Instant::now(), max, stop: stop.clone() });
    let (lang, expected, ctx) = (lang.to_string(), expected.to_string(), ctx.clone());
    std::thread::spawn(move || {
        let result = record(&stop, max).and_then(|(samples, rate)| {
            set_phase(id, Phase::Checking);
            ctx.request_repaint();
            // Under half a second, or nothing above the noise: no voice reached it.
            let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            if samples.len() < rate as usize / 2 || peak < 0.003 {
                return Err(tr("No sound reached the microphone. Check that it is connected and that Marea may use it (macOS: System Settings, Privacy & Security, Microphone).").to_string());
            }
            let dir = dir();
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let file = dir.join("take.wav");
            std::fs::write(&file, wav(&to_16k(&samples, rate))).map_err(|e| e.to_string())?;
            let heard = transcribe(&lang, &file);
            // The learner's voice stays only as long as it is needed.
            let _ = std::fs::remove_file(&file);
            heard
        });
        set_phase(id, Phase::Done(result.map(|h| score(&lang, &expected, &h)), false));
        ctx.request_repaint();
    });
}

/// Offers the speech download, or shows how it is going; nothing once ready.
pub fn offer(ui: &mut egui::Ui, lang: &str) {
    if !supported(lang) || available(lang) {
        return;
    }
    let s = status();
    ui.horizontal_wrapped(|ui| {
        if downloading(&s) {
            ui.spinner();
            let mb = partial_mb().map(|m| format!(" {m:.0} MB")).unwrap_or_default();
            ui.label(muted(format!("{s}{mb}")));
            return;
        }
        if ui
            .add(theme::secondary(trf("Check your pronunciation: download speech recognition ({0} MB)", &[&DOWNLOAD_MB])))
            .on_hover_text(tr("Recognition runs on this computer; your voice is not sent anywhere."))
            .clicked()
        {
            prepare(ui.ctx());
        }
        if !s.is_empty() && s != tr("Ready") {
            ui.label(RichText::new(s).color(BAD()));
        }
    });
}

/// Record, then Stop while listening, then the sentence with the words
/// heard in green and the others in red, a score and "Try again". Returns
/// the attempt once, in the frame it is ready. Offers the download while
/// speech recognition is not installed; draws nothing for a language it
/// does not know.
pub fn check_button(ui: &mut egui::Ui, lang: &str, expected: &str) -> Option<Attempt> {
    if !supported(lang) {
        return None;
    }
    if !available(lang) {
        offer(ui, lang);
        return None;
    }
    let ctx = ui.ctx().clone();
    let id = egui::Id::new(("speech", lang, expected));
    let phase = TAKES.lock().unwrap_or_else(|e| e.into_inner()).get(&id).cloned();
    let mut fresh = None;
    ui.horizontal_wrapped(|ui| match phase {
        None => {
            let free = !busy();
            if ui
                .add_enabled(free, theme::secondary(tr("Record")))
                .on_hover_text(tr("Say the sentence aloud; the app checks what it hears"))
                .clicked()
            {
                start(id, lang, expected, &ctx);
            }
        }
        Some(Phase::Listening { since, max, stop }) => {
            if theme::icon_button(ui, theme::secondary(theme::icon_room(tr("Stop"), false)), theme::Icon::Stop, theme::INK(), false).clicked() {
                stop.store(true, Ordering::Relaxed);
            }
            let left = max.saturating_sub(since.elapsed()).as_secs() + 1;
            ui.label(medium(tr("Listening…")).color(BAD()));
            ui.label(muted(trf("stops by itself in {0} s", &[&left])));
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        Some(Phase::Checking) => {
            ui.spinner();
            ui.label(muted(tr("Checking…")));
        }
        Some(Phase::Done(result, delivered)) => {
            match &result {
                Ok(a) => {
                    ui.spacing_mut().item_spacing.x = if lang == "zh" { 0.0 } else { 5.0 };
                    for (w, ok) in &a.words {
                        let t = RichText::new(w).size(16.0);
                        ui.label(if *ok { t.color(GOOD()) } else { t.color(BAD()).underline() });
                    }
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.label(medium(format!("{:.0}%", a.score * 100.0)));
                    let heard = if a.heard.is_empty() { tr("nothing").to_string() } else { a.heard.clone() };
                    ui.label(muted(trf("heard: {0}", &[&heard])));
                    if !delivered {
                        fresh = Some(a.clone());
                        set_phase(id, Phase::Done(result.clone(), true));
                    }
                }
                Err(e) => {
                    ui.label(RichText::new(e).color(BAD()));
                }
            }
            if ui.add_enabled(!busy(), theme::ghost(tr("Try again"))).clicked() {
                start(id, lang, expected, &ctx);
            }
        }
    });
    fresh
}

// ---------------------------------------------------------------- the CLI

/// `--speech-test LANG FILE.wav "expected sentence"`: installs speech
/// recognition if needed, transcribes the file and prints the attempt.
/// Exit code 0 when the score is at least 0.6, 1 below, 2 on an error.
pub fn self_test(args: &[String]) -> i32 {
    let [lang, file, expected] = args else {
        eprintln!("usage: marea --speech-test LANG FILE.wav \"expected sentence\"");
        return 2;
    };
    if engine().is_none() || !model_ready() {
        if let Err(e) = download(&|s| eprintln!("{s}")) {
            eprintln!("{e}");
            return 2;
        }
    }
    eprintln!("engine: {}", engine().map(|p| p.display().to_string()).unwrap_or_default());
    let start = Instant::now();
    match transcribe(lang, Path::new(file)) {
        Ok(heard) => {
            let a = score(lang, expected, &heard);
            println!("heard: {}", a.heard);
            let marks: Vec<String> = a.words.iter().map(|(w, ok)| format!("{w}{}", if *ok { "+" } else { "-" })).collect();
            println!("words: {}", marks.join(" "));
            println!("score: {:.2} ({:.1} s)", a.score, start.elapsed().as_secs_f32());
            if a.score >= 0.6 { 0 } else { 1 }
        }
        Err(e) => {
            eprintln!("{e}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marks(a: &Attempt) -> Vec<bool> {
        a.words.iter().map(|(_, ok)| *ok).collect()
    }

    #[test]
    fn portuguese() {
        let a = score("pt", "Bom dia, eu me chamo Pedro.", "Bom dia! Eu me chamo Pedro.");
        assert_eq!(a.score, 1.0);
        assert_eq!(a.words[0], ("Bom".to_string(), true));
        // Accents forgiven.
        assert_eq!(score("pt", "Você está bem?", "voce esta bem").score, 1.0);
        // Missing and different words, in order.
        let a = score("pt", "Eu moro no Rio de Janeiro", "Eu moro em Janeiro");
        assert_eq!(marks(&a), [true, true, false, false, false, true]);
        assert!((a.score - 0.5).abs() < 1e-6);
        // Numbers heard as digits, in the expected gender.
        assert_eq!(score("pt", "Tenho uma casa e duas irmãs", "Tenho 1 casa e 2 irmãs").score, 1.0);
        assert_eq!(score("pt", "São três horas", "São 3 horas.").score, 1.0);
    }

    #[test]
    fn italian() {
        assert_eq!(score("it", "Vorrei un caffè, per favore.", "Vorrei un caffe per favore").score, 1.0);
        assert_eq!(score("it", "C'è un'altra cosa.", "C'è un altra cosa").score, 1.0);
        // A misheard ending on a long word passes; short words must be right.
        assert_eq!(marks(&score("it", "un cornetto", "un cornetti")), [true, true]);
        assert_eq!(marks(&score("it", "per te", "pre te")), [false, true]);
        // One letter in four is forgiven ("cosa" for "casa"), two are not.
        assert_eq!(marks(&score("it", "la casa", "la cosa")), [true, true]);
        assert_eq!(marks(&score("it", "la casa", "la cose")), [true, false]);
        assert_eq!(score("it", "Ho venti anni", "Ho 20 anni").score, 1.0);
        assert_eq!(score("it", "Buongiorno a tutti", "Buon giorno a tutti").score, 1.0);
        // Extra words heard cost nothing; the order counts.
        assert_eq!(score("it", "Ciao", "ciao ciao amico").score, 1.0);
        assert_eq!(marks(&score("it", "uno due tre", "tre due uno")), [false, false, true]);
    }

    #[test]
    fn vietnamese_tones() {
        let want = "Tôi là người Việt Nam.";
        assert_eq!(score("vi", want, "Tôi là người Việt Nam").score, 1.0);
        // Tone marks are forgiven like accents.
        assert_eq!(score("vi", want, "toi la nguoi viet nam").score, 1.0);
        assert_eq!(marks(&score("vi", want, "Tôi là người Pháp")), [true, true, true, false, false]);
        // Short syllables are never matched by likeness: "ba" is not "bà".
        assert_eq!(marks(&score("vi", "ba con", "ma con")), [false, true]);
        assert_eq!(score("vi", "Tôi có mười hai quyển sách", "Tôi có 12 quyển sách").score, 1.0);
    }

    #[test]
    fn chinese_by_character() {
        let a = score("zh", "我是北京人。", "我是上海人");
        assert_eq!(a.words.iter().map(|(w, _)| w.as_str()).collect::<String>(), "我是北京人");
        assert_eq!(marks(&a), [true, true, false, false, true]);
        assert!((a.score - 0.6).abs() < 1e-6);
        assert_eq!(score("zh", "你好，我叫王明。", "你好,我叫王明。").score, 1.0);
        assert_eq!(score("zh", "我有三个孩子", "我有3个孩子").score, 1.0);
        assert_eq!(score("zh", "我有两个孩子", "我有2个孩子").score, 1.0);
        assert_eq!(score("zh", "十二点", "12点").score, 1.0);
        assert_eq!(zh_number(35).as_deref(), Some("三十五"));
        assert_eq!(zh_number(20).as_deref(), Some("二十"));
    }

    #[test]
    fn other_languages_and_edges() {
        assert_eq!(score("fr", "J'ai dix-sept ans.", "J'ai 17 ans").score, 1.0);
        assert_eq!(score("de", "Ich möchte eine Fahrkarte.", "Ich mochte 1 Fahrkarte").score, 1.0);
        assert_eq!(score("es", "Tengo dos hermanos", "tengo 2 hermanos").score, 1.0);
        // A number outside the table stays digits.
        assert_eq!(marks(&score("en", "I am forty", "I am 40")), [true, true, false]);
        // Nothing heard.
        let a = score("en", "Hello there", "");
        assert_eq!((a.score, marks(&a)), (0.0, vec![false, false]));
        assert_eq!(score("en", "", "hello").score, 0.0);
    }

    #[test]
    fn whisper_notes_are_dropped() {
        assert_eq!(clean(" [BLANK_AUDIO]\n"), "");
        assert_eq!(clean(" Hola (música) amigo.\n Adiós.\n"), "Hola amigo. Adiós.");
    }

    #[test]
    fn similarity_is_edit_distance() {
        assert_eq!(similarity("casa", "casa"), 1.0);
        assert_eq!(similarity("cornetto", "cornetti"), 0.875);
        assert_eq!(similarity("", ""), 1.0);
    }

    #[test]
    fn resampled_to_16k() {
        // 48 kHz, one second of a quiet 440 Hz tone: a third of the samples,
        // raised to 90% of full scale.
        let x: Vec<f32> = (0..48_000).map(|i| 0.1 * (i as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin()).collect();
        let y = to_16k(&x, 48_000);
        assert_eq!(y.len(), 16_000);
        let peak = y.iter().map(|s| s.unsigned_abs()).max().unwrap();
        assert!((27_000..=29_500).contains(&peak), "peak {peak}");
        // 8 kHz is stretched to twice as many samples.
        assert_eq!(to_16k(&[0.5; 8000], 8000).len(), 16_000);
        assert_eq!(to_16k(&[0.5; 16_000], 16_000).len(), 16_000);
        let w = wav(&[1, -1]);
        assert_eq!((w.len(), &w[..4], &w[36..40]), (48, &b"RIFF"[..], &b"data"[..]));
        assert_eq!(u32::from_le_bytes(w[24..28].try_into().unwrap()), 16_000);
    }

    #[test]
    fn widget_hands_over_the_attempt_once() {
        READY.store(true, Ordering::Relaxed);
        let ctx = egui::Context::default();
        theme::setup(&ctx);
        let (lang, want) = ("pt", "Bom dia, Pedro");
        let id = egui::Id::new(("speech", lang, want));
        set_phase(id, Phase::Done(Ok(score(lang, want, "bom dia")), false));
        let mut got = Vec::new();
        for _ in 0..2 {
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| got.push(check_button(ui, lang, want)));
        }
        assert!(got[0].as_ref().is_some_and(|a| marks(a) == [true, true, false]), "{got:?}");
        assert_eq!(got[1], None);
        // Another sentence draws its Record button; Chinese and Latin are known, Japanese is not.
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            assert_eq!(check_button(ui, "zh", "你好"), None);
            assert_eq!(check_button(ui, "ja", "こんにちは"), None);
        });
    }

    #[test]
    fn every_course_language_is_known() {
        for code in ["it", "en", "es", "pt", "nl", "de", "fr", "vi", "zh", "la"] {
            assert!(supported(code), "{code}");
            assert!(crate::model::lang(code).is_some(), "{code} is not a course language");
        }
        for (code, _) in NUMBERS {
            assert!(supported(code), "{code}");
        }
    }

    /// A sentence spoken by the app's own voice (the voice engine and the
    /// voice are downloaded first), at `target/speech-check/<lang>.wav`.
    fn fixture(code: &str, text: &str) -> PathBuf {
        // Tests run side by side: one download of a voice at a time.
        static ONE: Mutex<()> = Mutex::new(());
        let _one = ONE.lock().unwrap_or_else(|e| e.into_inner());
        let to = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("target/speech-check/{code}.wav"));
        if to.is_file() {
            return to;
        }
        let v = crate::audio::Voices::new(std::env::temp_dir().join("marea-voice-test"), egui::Context::default());
        v.download(code, egui::Context::default());
        let start = Instant::now();
        while !v.ready(code) && start.elapsed() < Duration::from_secs(600) {
            assert!(!v.status_of(code).starts_with("Failed"), "{code}: {}", v.status_of(code));
            std::thread::sleep(Duration::from_millis(200));
        }
        let file = v.speech_file(code, text).unwrap();
        v.prepare(code, &[text.to_string()]);
        while !file.is_file() && start.elapsed() < Duration::from_secs(660) {
            std::thread::sleep(Duration::from_millis(100));
        }
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(&file, &to).unwrap_or_else(|e| panic!("{code}: no speech written: {e}"));
        to
    }

    /// Makes the WAV files the "Check platforms" workflow transcribes with
    /// `--speech-test`: two sentences spoken by the app's own voices (~150 MB
    /// of downloads the first time). Run on purpose:
    /// `cargo test --release -- --ignored speech_fixtures --nocapture`.
    #[test]
    #[ignore]
    fn speech_fixtures() {
        for (code, text) in FIXTURES {
            println!("{}\t{code}\t{text}", fixture(code, text).display());
        }
    }

    /// The microphone, wherever the tests run (CI machines have none): an
    /// answer within seconds, never a crash or a hang.
    /// `cargo test --release -- --ignored microphone_answers --nocapture`.
    #[test]
    #[ignore]
    fn microphone_answers() {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let r = record(&AtomicBool::new(false), Duration::from_secs(2));
            let _ = tx.send(r.map(|(s, rate)| format!("{} samples at {rate} Hz, peak {}", s.len(), s.iter().fold(0.0f32, |m, x| m.max(x.abs())))));
        });
        let r = rx.recv_timeout(Duration::from_secs(30)).expect("the microphone hangs");
        println!("microphone: {r:?}");
    }

    /// A fixture through the recording path (16 kHz, 16-bit, peak raised),
    /// then whisper: the files the app writes are ones it can read. Installs
    /// speech recognition when it is missing, as `--speech-test` does.
    #[test]
    #[ignore]
    fn recorded_format_transcribes() {
        let (code, text) = FIXTURES[0];
        let bytes = std::fs::read(fixture(code, text)).unwrap();
        if engine().is_none() || !model_ready() {
            download(&|s| eprintln!("{s}")).expect("speech recognition installs");
        }
        // Piper writes one plain PCM header: mono, 16-bit, its rate at byte 24.
        let rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        let data = bytes.windows(4).position(|w| w == b"data").unwrap() + 8;
        let samples: Vec<f32> = bytes[data..].chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0).collect();
        let take = std::env::temp_dir().join("marea-take.wav");
        std::fs::write(&take, wav(&to_16k(&samples, rate))).unwrap();
        let a = score(code, text, &transcribe(code, &take).unwrap());
        println!("{a:?}");
        assert!(a.score >= 0.6, "{a:?}");
    }

    /// The sentences of `speech_fixtures`, also passed to `--speech-test`.
    const FIXTURES: [(&str, &str); 2] = [
        ("pt", "Uma bica e um pastel de nata, se faz favor."),
        ("it", "Buongiorno, vorrei un caffè e un cornetto, per favore."),
    ];
}
