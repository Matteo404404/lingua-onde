//! Updates from the GitHub releases: one check when the app starts, and on
//! request the new build replaces this one (the Linux binary, the Windows exe
//! or the macOS app bundle) and starts again. Progress lives in the data
//! folder, apart from the program, so it is untouched.
//!
//! Only release builds know their version (`MAREA_VERSION`, set by
//! .github/workflows/release.yml from the tag); a build from source never
//! checks.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::audio::{command, curl, find_program};
use crate::i18n::{tr, trf};

pub const REPO: &str = "Matteo404404/marea";

/// This build's release tag, such as "v1.3.1".
pub fn current() -> Option<&'static str> {
    option_env!("MAREA_VERSION")
}

/// "v1.10.2" → (1, 10, 2); anything else is not a release.
fn parse(tag: &str) -> Option<(u32, u32, u32)> {
    let mut n = tag.strip_prefix('v')?.split('.').map(|p| p.parse().ok());
    let v = (n.next()??, n.next()??, n.next()??);
    n.next().is_none().then_some(v)
}

/// The download for this computer, as release.yml names it.
fn asset() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "marea-linux-x86_64",
        ("windows", "x86_64") => "marea-windows-x86_64.exe",
        ("macos", "aarch64") => "marea-macos-arm64.zip",
        ("macos", "x86_64") => "marea-macos-x86_64.zip",
        _ => return None,
    })
}

#[derive(Clone, Default, PartialEq, Debug)]
pub enum State {
    #[default]
    Idle,
    /// Checked: this is the latest release.
    Current,
    /// A newer release: its tag.
    Available(String),
    Installing(String),
    /// Installed; the app restarts into it.
    Ready(String),
    Failed(String),
}

#[derive(Clone, Default)]
pub struct Updater {
    state: Arc<Mutex<State>>,
    /// "Later": the banner stays hidden until the next start.
    pub hidden: bool,
}

impl Updater {
    pub fn state(&self) -> State {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn set(&self, s: State, ctx: &egui::Context) {
        *self.state.lock().unwrap_or_else(|e| e.into_inner()) = s;
        ctx.request_repaint();
    }

    /// Asks GitHub for the latest release, in the background.
    pub fn check(&self, ctx: &egui::Context) {
        let (Some(now), Some(_)) = (current().and_then(parse), asset()) else {
            return;
        };
        let me = self.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            match latest_tag().and_then(|t| Some((parse(&t)?, t))) {
                Some((v, tag)) if v > now => me.set(State::Available(tag), &ctx),
                Some(_) => me.set(State::Current, &ctx),
                None => {}
            }
        });
    }

    /// Downloads `tag` and puts it in place of this build, in the background.
    pub fn install(&self, tag: String, ctx: &egui::Context) {
        let me = self.clone();
        let ctx = ctx.clone();
        self.set(State::Installing(tag.clone()), &ctx);
        std::thread::spawn(move || {
            let s = match replace_self(&tag) {
                Ok(()) => State::Ready(tag),
                Err(e) => State::Failed(e),
            };
            me.set(s, &ctx);
        });
    }
}

/// The release page, for reading what changed or downloading by hand.
pub fn page(tag: &str) -> String {
    format!("https://github.com/{REPO}/releases/tag/{tag}")
}

fn latest_tag() -> Option<String> {
    let curl = find_program("curl")?;
    let out = command(curl)
        .args(["-L", "--fail", "--silent", "--max-time", "20"])
        .args(["-H", "Accept: application/vnd.github+json"])
        .arg(format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    Some(json.get("tag_name")?.as_str()?.to_string())
}

/// A leftover from the last update (Windows cannot delete a running exe).
fn old(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".old");
    PathBuf::from(s)
}

/// Removes what the last update left behind; called when the app starts.
pub fn clean_up() {
    if let Some(target) = target() {
        let o = old(&target);
        let _ = if o.is_dir() { std::fs::remove_dir_all(&o) } else { std::fs::remove_file(&o) };
    }
}

/// What an update replaces: the .app bundle on macOS, the program elsewhere.
/// Taken once, at start: after the update moves the running program aside,
/// Linux reports its new name (".old"), and the restart must not start that.
fn target() -> Option<PathBuf> {
    static TARGET: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    TARGET
        .get_or_init(|| {
            let exe = std::env::current_exe().ok()?;
            if cfg!(target_os = "macos") {
                return exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")).map(Path::to_path_buf);
            }
            Some(exe)
        })
        .clone()
}

fn replace_self(tag: &str) -> Result<(), String> {
    let name = asset().ok_or(tr("No ready-made build for this computer"))?;
    let target = target().ok_or(tr("This copy is not an installed app; download the new version from the release page"))?;
    // Opened straight from the mounted disk image, which is read-only.
    if cfg!(target_os = "macos") && target.starts_with("/Volumes") {
        return Err(tr("Marea is running from the disk image: drag it onto Applications, open it from there, then update").to_string());
    }
    let dir = target.parent().ok_or("no folder")?.to_path_buf();
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{name}");
    let part = dir.join(format!(".{name}.part"));
    let fail = |e: String| {
        let _ = std::fs::remove_file(&part);
        trf("Could not update: {0}", &[&e])
    };
    curl(&url, &part).map_err(fail)?;
    // A GitHub error page is small; the app is megabytes.
    if std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0) < 1_000_000 {
        return Err(fail(tr("the download is incomplete").to_string()));
    }
    let new = if cfg!(target_os = "macos") {
        let unpacked = dir.join(".marea-update");
        let _ = std::fs::remove_dir_all(&unpacked);
        let ok = command("ditto").args(["-x", "-k"]).arg(&part).arg(&unpacked).status().map_err(|e| fail(e.to_string()))?;
        let _ = std::fs::remove_file(&part);
        let app = unpacked.join(target.file_name().ok_or("no name")?);
        // A copy installed before the rename is "Lingua Onde.app": it keeps
        // its name, and the bundle in the zip ("Marea.app") takes its place.
        let app = if ok.success() && app.is_dir() {
            app
        } else {
            std::fs::read_dir(&unpacked)
                .ok()
                .and_then(|d| d.flatten().map(|e| e.path()).find(|p| p.extension().is_some_and(|x| x == "app")))
                .unwrap_or_else(|| unpacked.join("Lingua Onde.app"))
        };
        if !app.is_dir() {
            return Err(fail(tr("the download is incomplete").to_string()));
        }
        app
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&part, std::fs::Permissions::from_mode(0o755)).map_err(|e| fail(e.to_string()))?;
        }
        part.clone()
    };
    // Move the running build aside, then the new one into its place; put the
    // old one back if that fails, so the app is never left missing.
    let o = old(&target);
    let _ = if o.is_dir() { std::fs::remove_dir_all(&o) } else { std::fs::remove_file(&o) };
    std::fs::rename(&target, &o).map_err(|e| fail(e.to_string()))?;
    if let Err(e) = std::fs::rename(&new, &target) {
        let _ = std::fs::rename(&o, &target);
        return Err(fail(e.to_string()));
    }
    let _ = std::fs::remove_dir_all(dir.join(".marea-update"));
    // Installed with the setup (packaging/marea.iss): Settings, Apps
    // reads the version from the uninstall entry, keyed by the setup's AppId.
    if cfg!(windows) && dir.join("unins000.exe").is_file() {
        let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\{4AC3DCD0-819C-4F78-A97A-0F9342519D3B}_is1";
        let _ = command("reg").args(["add", key, "/v", "DisplayVersion", "/d", tag.trim_start_matches('v'), "/f"]).status();
    }
    Ok(())
}

/// Starts the installed build; the caller then closes this one.
pub fn restart() {
    let Some(target) = target() else { return };
    let _ = if cfg!(target_os = "macos") {
        // -n: a second instance, since this one is still closing.
        command("open").arg("-n").arg(&target).spawn()
    } else {
        command(&target).spawn()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_as_numbers() {
        assert_eq!(parse("v1.3.1"), Some((1, 3, 1)));
        assert!(parse("v1.10.0") > parse("v1.9.9"));
        assert_eq!(parse("1.3.1"), None);
        assert_eq!(parse("v1.3"), None);
        assert_eq!(parse("v1.3.1-beta"), None);
    }
}
