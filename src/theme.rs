//! Colours, fonts and the shared widgets (cards, buttons, badges, bars).

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, Response, RichText, Sense, Shadow, Stroke, TextStyle, Ui, vec2,
};

pub const BG: Color32 = Color32::from_rgb(245, 243, 238);
pub const CARD: Color32 = Color32::from_rgb(255, 255, 255);
pub const INK: Color32 = Color32::from_rgb(28, 34, 48);
pub const MUTED: Color32 = Color32::from_rgb(107, 112, 128);
pub const LINE: Color32 = Color32::from_rgb(230, 225, 214);
pub const TRACK: Color32 = Color32::from_rgb(235, 231, 222);
pub const ACCENT: Color32 = Color32::from_rgb(31, 122, 140);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(20, 88, 104);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(227, 241, 243);
pub const NAV: Color32 = Color32::from_rgb(23, 34, 53);
pub const NAV_TEXT: Color32 = Color32::from_rgb(196, 206, 222);
pub const NAV_MUTED: Color32 = Color32::from_rgb(132, 148, 172);
pub const WAVE: Color32 = Color32::from_rgb(120, 205, 222);
pub const GOOD: Color32 = Color32::from_rgb(46, 154, 92);
pub const GOOD_SOFT: Color32 = Color32::from_rgb(222, 242, 229);
pub const WARN: Color32 = Color32::from_rgb(212, 138, 30);
pub const WARN_SOFT: Color32 = Color32::from_rgb(252, 239, 214);
pub const BAD: Color32 = Color32::from_rgb(200, 70, 60);
pub const HIGHLIGHT: Color32 = Color32::from_rgb(255, 243, 204);

/// A font that has Chinese characters, from the usual system places.
fn cjk_font() -> Option<Vec<u8>> {
    let mut candidates: Vec<String> = Vec::new();
    if cfg!(target_os = "linux") {
        if let Ok(out) = std::process::Command::new("fc-match")
            .args(["-f", "%{file}", ":lang=zh-cn"])
            .output()
        {
            let file = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if file.ends_with(".ttf") || file.ends_with(".otf") || file.ends_with(".ttc") {
                candidates.push(file);
            }
        }
    }
    candidates.extend(
        [
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simsun.ttc",
        ]
        .map(String::from),
    );
    candidates.iter().find_map(|p| std::fs::read(p).ok())
}

pub fn bold_family() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn medium_family() -> FontFamily {
    FontFamily::Name("medium".into())
}

pub fn setup(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Noto Sans (OFL, assets/NotoSans-OFL.txt) covers Vietnamese tone letters
    // and every Latin, Greek and Cyrillic accent; egui's own font does not.
    for (name, bytes) in [
        ("noto", &include_bytes!("../assets/NotoSans-Regular.ttf")[..]),
        ("noto-bold", &include_bytes!("../assets/NotoSans-Bold.ttf")[..]),
        ("noto-medium", &include_bytes!("../assets/NotoSans-Medium.ttf")[..]),
    ] {
        fonts.font_data.insert(name.into(), Arc::new(egui::FontData::from_static(bytes)));
    }
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "noto".into());
    fonts.families.entry(FontFamily::Monospace).or_default().push("noto".into());
    if let Some(bytes) = cjk_font() {
        fonts.font_data.insert("cjk".into(), Arc::new(egui::FontData::from_owned(bytes)));
        for fam in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts.families.entry(fam).or_default().push("cjk".into());
        }
    }
    // Bold and medium fall back to the regular list (CJK, symbols, emoji).
    let regular = fonts.families[&FontFamily::Proportional].clone();
    for (family, first) in [(bold_family(), "noto-bold"), (medium_family(), "noto-medium")] {
        let mut list = vec![first.to_string()];
        list.extend(regular.iter().cloned());
        fonts.families.insert(family, list);
    }
    ctx.set_fonts(fonts);

    ctx.set_visuals(egui::Visuals::light());
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(26.0, bold_family())),
            (TextStyle::Body, FontId::new(15.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(15.0, medium_family())),
            (TextStyle::Small, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        ]
        .into();
        let s = &mut style.spacing;
        s.item_spacing = vec2(8.0, 8.0);
        s.button_padding = vec2(14.0, 7.0);
        s.interact_size.y = 32.0;
        s.icon_width = 18.0;
        s.icon_width_inner = 10.0;
        s.icon_spacing = 8.0;
        s.slider_width = 220.0;
        s.slider_rail_height = 6.0;
        s.combo_width = 180.0;
        let v = &mut style.visuals;
        v.panel_fill = BG;
        v.window_fill = CARD;
        v.window_stroke = Stroke::new(1.0, LINE);
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(10);
        v.window_shadow = shadow(18, 24);
        v.popup_shadow = shadow(12, 20);
        // Text boxes must stand out on white cards.
        v.extreme_bg_color = Color32::from_rgb(246, 244, 239);
        v.override_text_color = Some(INK);
        v.selection.bg_fill = ACCENT;
        v.selection.stroke = Stroke::new(1.5, Color32::WHITE);
        v.hyperlink_color = ACCENT;
        v.slider_trailing_fill = true;
        v.handle_shape = egui::style::HandleShape::Circle;
        v.striped = false;
        // Checkboxes take this radius too, so it stays small; buttons set their own.
        let r = CornerRadius::same(6);
        for w in [
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
            &mut v.widgets.noninteractive,
        ] {
            w.corner_radius = r;
            w.expansion = 0.0;
        }
        v.widgets.inactive.weak_bg_fill = CARD;
        v.widgets.inactive.bg_fill = Color32::from_rgb(232, 228, 219);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(214, 208, 196));
        v.widgets.inactive.fg_stroke = Stroke::new(1.5, INK);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(240, 247, 248);
        v.widgets.hovered.bg_fill = Color32::from_rgb(214, 234, 238);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, INK);
        v.widgets.active.weak_bg_fill = ACCENT_SOFT;
        v.widgets.active.bg_fill = ACCENT;
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_DARK);
        v.widgets.open.weak_bg_fill = ACCENT_SOFT;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    });
}

fn shadow(blur: u8, alpha: u8) -> Shadow {
    Shadow {
        offset: [0, 2],
        blur,
        spread: 0,
        color: Color32::from_black_alpha(alpha),
    }
}

// ------------------------------------------------------------------ text

pub fn bold(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).family(bold_family())
}

pub fn medium(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).family(medium_family())
}

pub fn muted(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).color(MUTED)
}

/// A small uppercase label above a section or a value.
pub fn caps(text: &str) -> RichText {
    medium(text.to_uppercase()).size(11.5).color(MUTED).extra_letter_spacing(0.6)
}

/// The title and one line of explanation at the top of a screen.
pub fn page_title(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.add_space(4.0);
    ui.label(bold(title).size(28.0));
    if !subtitle.is_empty() {
        ui.label(muted(subtitle).size(15.0));
    }
    ui.add_space(10.0);
}

pub fn section(ui: &mut Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(bold(title).size(18.0));
}

// ---------------------------------------------------------------- frames

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(14)
        .inner_margin(Margin::same(18))
        .shadow(shadow(8, 8))
}

pub fn tinted(fill: Color32) -> egui::Frame {
    egui::Frame::new().fill(fill).corner_radius(12).inner_margin(Margin::same(14))
}

/// Space kept on each side of a screen's content.
const GUTTER: f32 = 22.0;

/// Lays out a screen's content as a centred column at most `max` wide.
pub fn page<R>(ui: &mut Ui, max: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let width = ui.available_width();
    let content = (width - 2.0 * GUTTER).min(max).max(width.min(320.0));
    let pad = ((width - content) / 2.0 - ui.spacing().item_spacing.x).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(pad);
        ui.vertical(|ui| {
            ui.set_width(content);
            add(ui)
        })
        .inner
    })
    .inner
}

// --------------------------------------------------------------- widgets

pub fn badge(ui: &mut Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color)
        .corner_radius(6)
        .inner_margin(Margin::symmetric(7, 2))
        .show(ui, |ui| ui.label(bold(text).color(Color32::WHITE).size(12.0)));
}

/// A rounded label: `fg` text on `bg`.
pub fn pill(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(20)
        .inner_margin(Margin::symmetric(10, 3))
        .show(ui, |ui| ui.label(medium(text).color(fg).size(13.0)))
        .response
}

pub fn primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(medium(text).color(Color32::WHITE))
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(10)
        .min_size(vec2(0.0, 36.0))
}

pub fn big_primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(medium(text).color(Color32::WHITE).size(16.5))
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(12)
        .min_size(vec2(200.0, 46.0))
}

pub fn secondary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(medium(text).color(INK))
        .fill(CARD)
        .stroke(Stroke::new(1.0, Color32::from_rgb(214, 208, 196)))
        .corner_radius(10)
        .min_size(vec2(0.0, 36.0))
}

/// A borderless small button for icons and quiet actions.
pub fn ghost(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.into()).color(MUTED))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(10)
}

/// A round filled button with one glyph (play, stop...).
pub fn round_button(ui: &mut Ui, glyph: &str, size: f32, fill: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let fill = if resp.hovered() { lerp(fill, Color32::BLACK, 0.15) } else { fill };
    ui.painter().circle_filled(rect.center(), size / 2.0, fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        FontId::proportional(size * 0.42),
        Color32::WHITE,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A glyph centred in a soft circle, for stat tiles and empty states.
pub fn icon_circle(ui: &mut Ui, glyph: &str, fg: Color32, bg: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().circle_filled(rect.center(), size / 2.0, bg);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        FontId::proportional(size * 0.48),
        fg,
    );
}

/// A number in a small circle (sentence and note numbers).
pub fn number(ui: &mut Ui, n: usize, active: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
    let (bg, fg) = if active { (ACCENT, Color32::WHITE) } else { (TRACK, MUTED) };
    ui.painter().circle_filled(rect.center(), 12.0, bg);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        n.to_string(),
        FontId::new(12.0, medium_family()),
        fg,
    );
}

/// A bar showing `frac` of `color`.
pub fn bar(ui: &mut Ui, frac: f32, width: f32, color: Color32) {
    bar_on(ui, frac, width, color, TRACK);
}

pub fn bar_on(ui: &mut Ui, frac: f32, width: f32, color: Color32, track: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 8.0), Sense::hover());
    ui.painter().rect_filled(rect, 4, track);
    let mut fill = rect;
    fill.set_width(rect.width() * frac.clamp(0.0, 1.0));
    if fill.width() > 0.5 {
        ui.painter().rect_filled(fill, 4, color);
    }
}

pub fn dot(ui: &mut Ui, color: Color32, r: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(r * 2.0 + 2.0, r * 2.0 + 2.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), r, color);
}

/// The app's mark: a wave in a rounded square.
pub fn logo(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().rect_filled(rect, size * 0.28, ACCENT);
    waves(ui.painter(), rect.shrink(size * 0.18), 2, 0.0, Color32::WHITE, size * 0.075);
}

/// `rows` sine waves across `rect`, shifted by `phase` (for gentle motion).
pub fn waves(p: &egui::Painter, rect: egui::Rect, rows: usize, phase: f32, color: Color32, width: f32) {
    for row in 0..rows {
        let y0 = rect.top() + rect.height() * (row as f32 + 1.0) / (rows as f32 + 1.0);
        let amp = rect.height() / (rows as f32 + 1.0) * 0.32;
        let points: Vec<egui::Pos2> = (0..=40)
            .map(|i| {
                let t = i as f32 / 40.0;
                let x = rect.left() + t * rect.width();
                egui::pos2(x, y0 + amp * (t * std::f32::consts::TAU * 1.5 + phase + row as f32 * 0.9).sin())
            })
            .collect();
        p.add(egui::Shape::line(points, Stroke::new(width, color)));
    }
}

/// Text whose `*...*` parts are bold: the stressed syllable of a figured
/// pronunciation, printed in bold as in the Assimil books (*sè*tëdei).
pub fn starred(text: &str, size: f32, color: Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (n, part) in text.split('*').enumerate() {
        let family = if n % 2 == 1 { bold_family() } else { FontFamily::Proportional };
        job.append(
            part,
            0.0,
            egui::TextFormat {
                font_id: FontId::new(size, family),
                color,
                ..Default::default()
            },
        );
    }
    job
}

/// A label for notes text: `*...*` in bold, wrapping to the width.
fn note_label(ui: &mut Ui, text: &str, header: bool) {
    if text.contains('*') && !header {
        let size = ui.style().text_styles[&TextStyle::Body].size;
        let mut job = starred(text, size, ui.visuals().text_color());
        job.wrap.max_width = ui.available_width();
        ui.add(egui::Label::new(job).wrap());
    } else {
        let t = RichText::new(text);
        ui.add(egui::Label::new(if header { t.family(bold_family()) } else { t }).wrap());
    }
}

/// A small Markdown subset for the pronunciation keys and the notes:
/// headings, pipe tables, bullets and paragraphs (wrapped lines join up);
/// `**` and backticks are dropped, `*...*` (a stressed syllable) is bold.
pub fn markdown(ui: &mut Ui, text: &str) {
    enum Block {
        Para(String),
        Bullet(String),
        Table(Vec<Vec<String>>),
    }
    let clean = |s: &str| s.replace("**", "").replace('`', "");
    let mut blocks: Vec<Block> = Vec::new();
    let mut headings: Vec<(usize, u8, String)> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            continue;
        }
        if t.starts_with('|') {
            let cells: Vec<String> = t.trim_matches('|').split('|').map(|c| clean(c.trim())).collect();
            let rule = cells
                .iter()
                .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')));
            match blocks.last_mut() {
                Some(Block::Table(rows)) if !rule => rows.push(cells),
                Some(Block::Table(_)) => {}
                _ => blocks.push(Block::Table(vec![cells])),
            }
        } else if let Some(h) = t.strip_prefix('#') {
            let level = 1 + h.chars().take_while(|c| *c == '#').count() as u8;
            headings.push((blocks.len(), level, clean(h.trim_start_matches('#').trim())));
            blocks.push(Block::Para(String::new()));
        } else if let Some(b) = t.strip_prefix("- ") {
            blocks.push(Block::Bullet(clean(b)));
        } else if t.is_empty() {
            blocks.push(Block::Para(String::new()));
        } else {
            // A wrapped line continues the paragraph or bullet above it.
            match blocks.last_mut() {
                Some(Block::Para(p) | Block::Bullet(p)) if !p.is_empty() => {
                    p.push(' ');
                    p.push_str(&clean(t));
                }
                _ => blocks.push(Block::Para(clean(t))),
            }
        }
    }
    let mut next_heading = headings.iter().peekable();
    for (i, block) in blocks.iter().enumerate() {
        while let Some((_, level, h)) = next_heading.next_if(|(at, _, _)| *at == i) {
            ui.add_space(if *level <= 2 { 8.0 } else { 4.0 });
            ui.label(match level {
                1 => bold(h.as_str()).size(21.0),
                2 => bold(h.as_str()).size(18.0),
                _ => medium(h.as_str()).size(15.5),
            });
        }
        match block {
            Block::Para(p) if p.is_empty() => {}
            Block::Para(p) => note_label(ui, p, false),
            Block::Bullet(b) => note_label(ui, &format!("•  {b}"), false),
            Block::Table(rows) => {
                // Columns share the width, so cells wrap instead of widening the page.
                let cols = rows.iter().map(Vec::len).max().unwrap_or(1).max(1) as f32;
                let col_w = ((ui.available_width() - 14.0 * (cols - 1.0)) / cols).max(60.0);
                egui::Grid::new(("md-table", i))
                    .striped(true)
                    .spacing([14.0, 4.0])
                    .min_col_width(col_w)
                    .max_col_width(col_w)
                    .show(ui, |ui| {
                        for (r, row) in rows.iter().enumerate() {
                            for cell in row {
                                note_label(ui, cell, r == 0);
                            }
                            ui.end_row();
                        }
                    });
                ui.add_space(6.0);
            }
        }
    }
}

pub fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// `color` mixed into white: soft backgrounds for coloured labels.
pub fn soft(color: Color32) -> Color32 {
    lerp(color, Color32::WHITE, 0.85)
}
