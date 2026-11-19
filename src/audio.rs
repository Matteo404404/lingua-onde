//! Sound: plays recordings with mpv (pause, rewind and position over its IPC
//! socket), and speaks text with downloaded Piper voices.

use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::model::lang;

pub fn find_program(name: &str) -> Option<PathBuf> {
    let exe = if cfg!(windows) { format!("{name}.exe") } else { name.to_string() };
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(&exe))
        .find(|p| p.is_file())
}

// ---------------------------------------------------------------- player

pub struct Player {
    child: Option<Child>,
    sock: PathBuf,
    /// mpv can pause and rewind; the fallback players only play and stop.
    pub controllable: bool,
    pub paused: bool,
    pub started: Option<Instant>,
    pub file: Option<PathBuf>,
    pos: (f64, f64),
    last_poll: Option<Instant>,
}

impl Default for Player {
    fn default() -> Self {
        let sock = std::env::temp_dir().join(format!("lingua-onde-{}.sock", std::process::id()));
        Player {
            child: None,
            sock,
            controllable: false,
            paused: false,
            started: None,
            file: None,
            pos: (0.0, 0.0),
            last_poll: None,
        }
    }
}

impl Player {
    pub fn available() -> bool {
        find_program("mpv").is_some() || find_program("ffplay").is_some() || find_program("afplay").is_some()
    }

    pub fn play(&mut self, file: &Path, speed: f32) -> Result<(), String> {
        self.stop();
        let speed = speed.clamp(0.5, 2.0);
        let (mut cmd, controllable) = if let Some(mpv) = find_program("mpv") {
            let mut c = Command::new(mpv);
            c.args([
                "--no-video",
                "--no-terminal",
                "--force-window=no",
                "--keep-open=no",
                "--idle=no",
                "--save-position-on-quit=no",
                "--resume-playback=no",
                "--audio-file-auto=no",
            ])
            .arg(format!("--speed={speed:.2}"))
            .arg(format!("--input-ipc-server={}", self.sock.display()))
            .arg("--")
            .arg(file);
            (c, cfg!(unix))
        } else if let Some(ffplay) = find_program("ffplay") {
            let mut c = Command::new(ffplay);
            c.args(["-nodisp", "-autoexit", "-loglevel", "quiet"])
                .arg("-af")
                .arg(format!("atempo={speed:.2}"))
                .arg(file);
            (c, false)
        } else if let Some(afplay) = find_program("afplay") {
            let mut c = Command::new(afplay);
            c.arg("-r").arg(format!("{speed:.2}")).arg(file);
            (c, false)
        } else {
            return Err("No audio player found. Install mpv to hear recordings and voices.".into());
        };
        cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let child = cmd.spawn().map_err(|e| format!("could not start the audio player: {e}"))?;
        self.child = Some(child);
        self.controllable = controllable;
        self.paused = false;
        self.started = Some(Instant::now());
        self.file = Some(file.to_path_buf());
        self.pos = (0.0, 0.0);
        self.last_poll = None;
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        self.started = None;
        self.file = None;
        self.paused = false;
    }

    /// Still playing (or paused)?
    pub fn busy(&mut self) -> bool {
        let finished = match self.child.as_mut() {
            None => return false,
            Some(c) => !matches!(c.try_wait(), Ok(None)),
        };
        if finished {
            self.child = None;
            self.started = None;
            self.file = None;
        }
        !finished
    }

    pub fn toggle_pause(&mut self) {
        if self.ipc(r#"["cycle","pause"]"#).is_some() {
            self.paused = !self.paused;
        }
    }

    pub fn seek(&mut self, secs: f64) {
        self.ipc(&format!(r#"["seek",{secs},"relative"]"#));
    }

    /// (position, duration) in seconds, polled at most four times a second.
    pub fn position(&mut self) -> (f64, f64) {
        if self.controllable && self.child.is_some() && self.last_poll.is_none_or(|t| t.elapsed() > Duration::from_millis(250)) {
            self.last_poll = Some(Instant::now());
            let num = |v: Option<serde_json::Value>| v.and_then(|v| v.as_f64());
            if let Some(p) = num(self.ipc(r#"["get_property","time-pos"]"#)) {
                self.pos.0 = p;
            }
            if let Some(d) = num(self.ipc(r#"["get_property","duration"]"#)) {
                self.pos.1 = d;
            }
        }
        self.pos
    }

    #[cfg(unix)]
    fn ipc(&self, command: &str) -> Option<serde_json::Value> {
        use std::os::unix::net::UnixStream;
        if !self.controllable || self.child.is_none() {
            return None;
        }
        let mut s = UnixStream::connect(&self.sock).ok()?;
        s.set_read_timeout(Some(Duration::from_millis(150))).ok()?;
        writeln!(s, r#"{{"command":{command},"request_id":7}}"#).ok()?;
        for line in BufReader::new(s).lines() {
            let v: serde_json::Value = serde_json::from_str(&line.ok()?).ok()?;
            if v.get("request_id").and_then(|r| r.as_i64()) == Some(7) {
                return if v["error"] == "success" { Some(v["data"].clone()) } else { None };
            }
        }
        None
    }

    // shortcut: Windows mpv uses a named pipe; pause/rewind are left out there for now.
    #[cfg(not(unix))]
    fn ipc(&self, _command: &str) -> Option<serde_json::Value> {
        None
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
        let _ = std::fs::remove_file(&self.sock);
    }
}

// ---------------------------------------------------------- recordings

const AUDIO_EXT: [&str; 8] = ["mp3", "ogg", "opus", "m4a", "aac", "wav", "flac", "wma"];

/// The last number in a file name: "Leccion 12.mp3" → 12, "L012.mp3" → 12.
fn last_number(stem: &str) -> Option<u32> {
    let mut last = None;
    let mut cur = String::new();
    for ch in stem.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_digit() {
            cur.push(ch);
        } else if !cur.is_empty() {
            last = cur.parse().ok();
            cur.clear();
        }
    }
    last
}

/// Audio files in a folder (and below), by the lesson number in their name.
pub fn scan_audio_dir(dir: &Path) -> BTreeMap<u32, PathBuf> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    let mut seen = 0;
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            seen += 1;
            if seen > 20_000 {
                return out;
            }
            if p.is_dir() {
                stack.push(p);
            } else if p
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| AUDIO_EXT.contains(&e.to_ascii_lowercase().as_str()))
            {
                let stem = p.file_stem().unwrap_or_default().to_string_lossy();
                if let Some(n) = last_number(&stem) {
                    out.entry(n).or_insert(p);
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------- voices

const VOICE_BASE: &str = "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0";

fn engine_url() -> Option<String> {
    let file = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "piper_linux_x86_64.tar.gz",
        ("linux", "aarch64") => "piper_linux_aarch64.tar.gz",
        ("macos", "x86_64") => "piper_macos_x64.tar.gz",
        ("macos", "aarch64") => "piper_macos_aarch64.tar.gz",
        ("windows", _) => "piper_windows_amd64.zip",
        _ => return None,
    };
    Some(format!("https://github.com/rhasspy/piper/releases/download/2023.11.14-2/{file}"))
}

/// FNV-1a: a stable name for cached speech files.
fn fnv(text: &str) -> u64 {
    text.bytes()
        .fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}

struct Job {
    lang: String,
    text: String,
    out: PathBuf,
}

pub struct Voices {
    pub dir: PathBuf,
    /// Per language: what the download is doing ("" = idle).
    pub status: Arc<Mutex<BTreeMap<String, String>>>,
    jobs: Sender<Job>,
}

impl Voices {
    pub fn new(dir: PathBuf, ctx: egui::Context) -> Voices {
        let (tx, rx) = channel::<Job>();
        let worker_dir = dir.clone();
        std::thread::spawn(move || speech_worker(worker_dir, rx, ctx));
        Voices {
            dir,
            status: Arc::default(),
            jobs: tx,
        }
    }

    fn piper_bin(dir: &Path) -> PathBuf {
        dir.join("piper").join(if cfg!(windows) { "piper.exe" } else { "piper" })
    }

    pub fn engine_ready(&self) -> bool {
        Self::piper_bin(&self.dir).is_file()
    }

    fn files(dir: &Path, code: &str) -> Option<(PathBuf, PathBuf)> {
        let (path, espeak) = lang(code)?.voice;
        let name = path.rsplit('/').next()?;
        let model = dir.join(format!("{name}.onnx"));
        let config = match espeak {
            Some(_) => dir.join(format!("{code}-{name}.onnx.json")),
            None => dir.join(format!("{name}.onnx.json")),
        };
        Some((model, config))
    }

    pub fn ready(&self, code: &str) -> bool {
        self.engine_ready() && Self::files(&self.dir, code).is_some_and(|(m, c)| m.is_file() && c.is_file())
    }

    pub fn status_of(&self, code: &str) -> String {
        self.status
            .lock()
            .map(|s| s.get(code).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Megabytes downloaded so far for a running download.
    pub fn partial_mb(&self, code: &str) -> Option<f64> {
        let meta = std::fs::metadata(self.dir.join(format!("{code}.part"))).ok()?;
        Some(meta.len() as f64 / 1_048_576.0)
    }

    /// Download the engine (once) and this language's voice in the background.
    pub fn download(&self, code: &str, ctx: egui::Context) {
        let status = self.status_of(code);
        if status.starts_with("Downloading") || status.starts_with("Finishing") {
            return;
        }
        let dir = self.dir.clone();
        let status = self.status.clone();
        let code = code.to_string();
        std::thread::spawn(move || {
            let set = |s: &str| {
                if let Ok(mut m) = status.lock() {
                    m.insert(code.clone(), s.to_string());
                }
                ctx.request_repaint();
            };
            match download_voice(&dir, &code, &set) {
                Ok(()) => set("Ready"),
                Err(e) => set(&format!("Failed: {e}")),
            }
            let _ = std::fs::remove_file(dir.join(format!("{code}.part")));
        });
    }
}

fn curl(url: &str, to: &Path) -> Result<(), String> {
    let curl = find_program("curl").ok_or("curl is needed to download voices")?;
    let ok = Command::new(curl)
        .args(["-L", "--fail", "--silent", "--show-error", "-o"])
        .arg(to)
        .arg(url)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if ok.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&ok.stderr).trim().to_string())
    }
}

fn download_voice(dir: &Path, code: &str, set: &dyn Fn(&str)) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let part = dir.join(format!("{code}.part"));
    // Two languages downloading at once must not fetch the engine twice.
    static ENGINE: Mutex<()> = Mutex::new(());
    let _engine = ENGINE.lock().unwrap_or_else(|e| e.into_inner());
    if !Voices::piper_bin(dir).is_file() {
        set("Downloading the voice engine (25 MB)…");
        let url = engine_url().ok_or("no voice engine is published for this computer")?;
        curl(&url, &part)?;
        let tar = find_program("tar").ok_or("tar is needed to unpack the voice engine")?;
        let out = Command::new(tar)
            .arg("-xf")
            .arg(&part)
            .arg("-C")
            .arg(dir)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(format!(
                "could not unpack the engine: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        let _ = std::fs::remove_file(&part);
    }
    let info = lang(code).ok_or("unknown language")?;
    let (path, espeak) = info.voice;
    let (model, config) = Voices::files(dir, code).ok_or("unknown language")?;
    let name = path.rsplit('/').next().unwrap_or(path);
    if !model.is_file() {
        set("Downloading the voice (about 60 MB)…");
        curl(&format!("{VOICE_BASE}/{path}.onnx"), &part)?;
        std::fs::rename(&part, &model).map_err(|e| e.to_string())?;
    }
    if !config.is_file() {
        set("Finishing…");
        let plain = dir.join(format!("{name}.onnx.json"));
        if !plain.is_file() {
            curl(&format!("{VOICE_BASE}/{path}.onnx.json"), &part)?;
            std::fs::rename(&part, &plain).map_err(|e| e.to_string())?;
        }
        if let Some(phonemes) = espeak {
            // Same voice, other pronunciation rules (Latin read by the Italian voice).
            let mut v: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&plain).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            v["espeak"]["voice"] = serde_json::Value::String(phonemes.to_string());
            std::fs::write(&config, v.to_string()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

impl Voices {
    /// Where the spoken form of `text` is (or will be) cached.
    pub fn speech_file(&self, code: &str, text: &str) -> Option<PathBuf> {
        let (model, config) = Self::files(&self.dir, code)?;
        let key = format!("{}|{}|{}", model.display(), config.display(), text);
        Some(self.dir.join("cache").join(format!("{:016x}.wav", fnv(&key))))
    }

    /// Queue sentences to be spoken ahead of time; files appear as they're done.
    pub fn prepare(&self, code: &str, texts: &[String]) {
        if !self.ready(code) {
            return;
        }
        for t in texts {
            let t = t.replace(['[', ']'], "");
            if t.trim().is_empty() {
                continue;
            }
            if let Some(out) = self.speech_file(code, &t) {
                if !out.is_file() {
                    let _ = self.jobs.send(Job {
                        lang: code.to_string(),
                        text: t,
                        out,
                    });
                }
            }
        }
    }
}

/// One long-running piper per language, fed one JSON line per sentence.
fn speech_worker(dir: PathBuf, rx: Receiver<Job>, ctx: egui::Context) {
    let mut engines: HashMap<String, (Child, ChildStdin, BufReader<ChildStdout>)> = HashMap::new();
    let _ = std::fs::create_dir_all(dir.join("cache"));
    while let Ok(job) = rx.recv() {
        if job.out.is_file() {
            continue;
        }
        if !engines.contains_key(&job.lang) {
            let Some((model, config)) = Voices::files(&dir, &job.lang) else {
                continue;
            };
            let spawned = Command::new(Voices::piper_bin(&dir))
                .arg("--model")
                .arg(&model)
                .arg("--config")
                .arg(&config)
                .args(["--json-input", "--quiet"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();
            let Ok(mut child) = spawned else { continue };
            let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
                continue;
            };
            engines.insert(job.lang.clone(), (child, stdin, BufReader::new(stdout)));
        }
        let (_, stdin, stdout) = engines.get_mut(&job.lang).expect("inserted above");
        let tmp = job.out.with_extension("part.wav");
        let line = serde_json::json!({ "text": job.text, "output_file": tmp }).to_string();
        let mut reply = String::new();
        let ok = writeln!(stdin, "{line}").is_ok() && stdin.flush().is_ok() && stdout.read_line(&mut reply).is_ok_and(|n| n > 0);
        if ok && tmp.is_file() {
            let _ = std::fs::rename(&tmp, &job.out);
            ctx.request_repaint();
        } else if let Some((mut child, _, _)) = engines.remove(&job.lang) {
            let _ = child.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Downloads the engine and a voice (~85 MB), then speaks one sentence.
    /// Run on purpose: `cargo test -- --ignored voice`.
    #[test]
    #[ignore]
    fn voice_download_and_speech() {
        let dir = std::env::temp_dir().join("lingua-onde-voice-test");
        let v = Voices::new(dir.clone(), egui::Context::default());
        for code in ["es", "la"] {
            download_voice(&dir, code, &|s| eprintln!("{code}: {s}")).unwrap();
            assert!(v.ready(code), "{code} not ready");
            let text = if code == "la" { "Salve, amice." } else { "Hola, ¿qué tal?" };
            let out = v.speech_file(code, text).unwrap();
            let _ = std::fs::remove_file(&out);
            v.prepare(code, &[text.to_string()]);
            let start = Instant::now();
            while !out.is_file() && start.elapsed() < Duration::from_secs(30) {
                std::thread::sleep(Duration::from_millis(100));
            }
            assert!(std::fs::metadata(&out).unwrap().len() > 10_000, "{code}: no speech written");
        }
    }

    /// Needs mpv and the file from `voice_download_and_speech`; mute it with
    /// MPV_HOME=<dir with "ao=null" in mpv.conf>.
    #[test]
    #[ignore]
    fn recording_pause_and_position() {
        let dir = std::env::temp_dir().join("lingua-onde-voice-test");
        let v = Voices::new(dir, egui::Context::default());
        let file = v.speech_file("es", "Hola, ¿qué tal?").unwrap();
        let mut p = Player::default();
        p.play(&file, 0.6).unwrap();
        std::thread::sleep(Duration::from_millis(500));
        assert!(p.busy());
        let (_, dur) = p.position();
        assert!(dur > 0.5, "duration {dur}");
        p.toggle_pause();
        assert!(p.paused);
        p.stop();
        assert!(!p.busy());
    }

    #[test]
    fn numbers_in_file_names() {
        assert_eq!(last_number("Leccion 12"), Some(12));
        assert_eq!(last_number("L012"), Some(12));
        assert_eq!(last_number("cd1 - Lektion 7"), Some(7));
        assert_eq!(last_number("intro"), None);
    }
}
