// No console window behind the app on Windows (debug builds keep it for messages).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod appendix;
mod audio;
mod editor;
mod format;
mod model;
mod practice;
mod session;
mod store;
mod theme;

use std::path::PathBuf;

use eframe::egui;

/// `--check <file>...`: parse lesson files and list what is wrong in a
/// built-in lesson (`format::builtin_problems`) and what it still lacks
/// (`format::builtin_gaps`, lines with "gap:"). Exit code 1 when anything is listed.
fn check(files: &[String]) -> i32 {
    let mut code = 0;
    for f in files {
        let parsed = std::fs::read_to_string(f)
            .map_err(|e| e.to_string())
            .and_then(|s| format::parse_course(&s, ""));
        let problems = match &parsed {
            Ok(c) => c
                .lessons
                .iter()
                .flat_map(|l| [format::builtin_problems(c, l), format::builtin_gaps(c, l)].concat())
                .collect(),
            Err(e) => vec![e.clone()],
        };
        for p in &problems {
            println!("{f}: {p}");
        }
        match parsed {
            Ok(c) if problems.is_empty() => println!("{f}: ok, {} lessons", c.lessons.len()),
            _ => code = 1,
        }
    }
    code
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--check") {
        std::process::exit(check(&args[i + 1..]));
    }
    let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    let screenshot = value("--screenshot").map(PathBuf::from);
    let open = value("--open");

    // `--size 950x520` checks how a small or tiled window looks.
    let size = value("--size")
        .and_then(|s| s.split_once('x').and_then(|(w, h)| Some([w.parse().ok()?, h.parse().ok()?])))
        .unwrap_or([1280.0, 820.0]);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Lingua Onde")
        .with_app_id("lingua-onde")
        .with_inner_size(size)
        .with_min_inner_size([900.0, 600.0]);
    if screenshot.is_some() {
        // A check must never steal focus from what the person is doing.
        viewport = viewport.with_active(false);
        if cfg!(windows) {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
    }
    #[allow(unused_mut)]
    let mut options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    #[cfg(target_os = "macos")]
    if screenshot.is_some() {
        options.event_loop_builder = Some(Box::new(|b| {
            use winit::platform::macos::EventLoopBuilderExtMacOS;
            b.with_activate_ignoring_other_apps(false);
        }));
    }

    eframe::run_native(
        "Lingua Onde",
        options,
        Box::new(move |cc| {
            theme::setup(&cc.egui_ctx);
            Ok(Box::new(app::App::new(&cc.egui_ctx, screenshot, open)))
        }),
    )
}
