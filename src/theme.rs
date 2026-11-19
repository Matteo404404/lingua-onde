//! Colours, fonts and the few shared widgets (cards, badges).

use std::sync::Arc;

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Margin, RichText, Stroke, TextStyle};

pub const BG: Color32 = Color32::from_rgb(246, 243, 236);
pub const CARD: Color32 = Color32::from_rgb(255, 255, 255);
pub const INK: Color32 = Color32::from_rgb(34, 38, 49);
pub const MUTED: Color32 = Color32::from_rgb(110, 114, 126);
pub const LINE: Color32 = Color32::from_rgb(226, 221, 210);
pub const ACCENT: Color32 = Color32::from_rgb(33, 112, 140);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(222, 238, 243);
pub const NAV: Color32 = Color32::from_rgb(28, 40, 62);
pub const GOOD: Color32 = Color32::from_rgb(46, 150, 90);
pub const WARN: Color32 = Color32::from_rgb(214, 140, 30);
pub const BAD: Color32 = Color32::from_rgb(200, 70, 60);
pub const HIGHLIGHT: Color32 = Color32::from_rgb(255, 244, 205);

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

pub fn setup(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Noto Sans (OFL, assets/NotoSans-OFL.txt) covers Vietnamese tone letters
    // and every Latin, Greek and Cyrillic accent; egui's own font does not.
    fonts.font_data.insert(
        "noto".into(),
        Arc::new(egui::FontData::from_static(include_bytes!("../assets/NotoSans-Regular.ttf"))),
    );
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "noto".into());
    fonts.families.entry(FontFamily::Monospace).or_default().push("noto".into());
    if let Some(bytes) = cjk_font() {
        fonts.font_data.insert("cjk".into(), Arc::new(egui::FontData::from_owned(bytes)));
        for fam in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts.families.entry(fam).or_default().push("cjk".into());
        }
    }
    ctx.set_fonts(fonts);

    ctx.set_visuals(egui::Visuals::light());
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(15.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(15.0, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.interact_size.y = 30.0;
        let v = &mut style.visuals;
        v.panel_fill = BG;
        v.window_fill = CARD;
        // Text boxes must stand out on white cards.
        v.extreme_bg_color = Color32::from_rgb(243, 240, 232);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(205, 198, 184));
        v.override_text_color = Some(INK);
        v.selection.bg_fill = ACCENT;
        v.hyperlink_color = ACCENT;
        let r = CornerRadius::same(8);
        for w in [
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
            &mut v.widgets.noninteractive,
        ] {
            w.corner_radius = r;
        }
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(236, 232, 223);
        v.widgets.inactive.bg_fill = Color32::from_rgb(236, 232, 223);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(222, 236, 241);
        v.widgets.hovered.bg_fill = Color32::from_rgb(222, 236, 241);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    });
}

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(12)
        .inner_margin(Margin::same(16))
}

pub fn tinted(fill: Color32) -> egui::Frame {
    egui::Frame::new().fill(fill).corner_radius(10).inner_margin(Margin::same(12))
}

pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color)
        .corner_radius(6)
        .inner_margin(Margin::symmetric(7, 2))
        .show(ui, |ui| ui.label(RichText::new(text).color(Color32::WHITE).strong().size(12.5)));
}

pub fn primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.into()).color(Color32::WHITE).strong()).fill(ACCENT)
}

pub fn big_primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.into()).color(Color32::WHITE).strong().size(17.0))
        .fill(ACCENT)
        .min_size(egui::vec2(220.0, 44.0))
}

pub fn muted(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).color(MUTED)
}

/// A thin bar showing `frac` of `color`.
pub fn bar(ui: &mut egui::Ui, frac: f32, width: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 8.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4, Color32::from_rgb(232, 228, 219));
    let mut fill = rect;
    fill.set_width(rect.width() * frac.clamp(0.0, 1.0));
    if fill.width() > 0.5 {
        ui.painter().rect_filled(fill, 4, color);
    }
}

pub fn dot(ui: &mut egui::Ui, color: Color32, r: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(r * 2.0 + 2.0, r * 2.0 + 2.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), r, color);
}
