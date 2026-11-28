//! Colours, fonts and the shared widgets (cards, buttons, badges, bars).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, Response, RichText, Sense, Shadow, Stroke, TextStyle, Ui, vec2,
};


// Light or dark, chosen each frame from the learner's setting and the system
// (`set_dark`); every colour below has both.
static DARK: AtomicBool = AtomicBool::new(false);

pub fn dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

macro_rules! palette {
    ($($(#[$doc:meta])* $name:ident = $light:expr, $dark:expr;)*) => {$(
        $(#[$doc])*
        #[allow(non_snake_case)]
        pub fn $name() -> Color32 {
            let (r, g, b) = if dark() { $dark } else { $light };
            Color32::from_rgb(r, g, b)
        }
    )*};
}

// Station signage: a warm grey ground, white enamel panels, a near-black
// wall for the sidebar. Colour comes from the line (below), not the chrome.
palette! {
    BG = (243, 242, 239), (18, 18, 18);
    CARD = (255, 255, 255), (30, 30, 30);
    INK = (28, 28, 28), (236, 236, 236);
    MUTED = (100, 100, 100), (158, 158, 158);
    LINE = (226, 223, 218), (52, 52, 52);
    TRACK = (232, 229, 224), (50, 50, 50);
    NAV = (28, 28, 28), (12, 12, 12);
    NAV_TEXT = (214, 214, 214), (214, 214, 214);
    NAV_MUTED = (150, 150, 150), (150, 150, 150);
    // GOOD and WARN are dark enough on the light theme to be read as text (4.5:1).
    GOOD = (32, 128, 72), (84, 190, 122);
    GOOD_SOFT = (224, 240, 229), (30, 52, 38);
    WARN = (160, 96, 8), (230, 166, 70);
    WARN_SOFT = (250, 238, 214), (62, 48, 26);
    BAD = (192, 60, 50), (236, 108, 98);
    HIGHLIGHT = (255, 240, 196), (74, 62, 28);
    /// Text boxes, so they stand out on cards.
    FIELD = (247, 246, 243), (24, 24, 24);
    /// Unchecked boxes, slider rails, quiet buttons' borders.
    CONTROL = (234, 231, 226), (56, 56, 56);
    CONTROL_LINE = (210, 206, 199), (76, 76, 76);
    /// The spelled-out pronunciation line.
    FIGURED = (150, 92, 30), (222, 168, 104);
    /// Lessons not started yet.
    PENDING = (196, 192, 184), (90, 90, 90);
}

/// Switches the palette; returns true when it changed (visuals must follow).
pub fn set_dark(on: bool) -> bool {
    DARK.swap(on, Ordering::Relaxed) != on
}

// Each course is a metro line with its own colour (model::LangInfo::color):
// it fills the primary button, the selected item, the band and the rail.
static LINE_RGB: AtomicU32 = AtomicU32::new(0x284A9A);

/// Sets the line colour of the course in use; true when it changed.
pub fn set_line(c: Color32) -> bool {
    let v = u32::from_be_bytes([0, c.r(), c.g(), c.b()]);
    LINE_RGB.swap(v, Ordering::Relaxed) != v
}

fn line_rgb() -> Color32 {
    let [_, r, g, b] = LINE_RGB.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgb(r, g, b)
}

/// Relative luminance (WCAG), 0 to 1.
fn luminance(c: Color32) -> f32 {
    let ch = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
}

/// Text on a fill of `color`: white, or ink on a light colour (yellow).
pub fn text_on(color: Color32) -> Color32 {
    if luminance(color) > 0.3 { Color32::from_rgb(28, 28, 28) } else { Color32::WHITE }
}

/// The line colour as a fill: buttons, the selected item, the band.
#[allow(non_snake_case)]
pub fn FILL() -> Color32 {
    line_rgb()
}

/// Text on FILL: white, or ink on a light line (yellow).
#[allow(non_snake_case)]
pub fn ON_FILL() -> Color32 {
    text_on(FILL())
}

/// The line colour as ink: links, small marks, readable on the ground.
#[allow(non_snake_case)]
pub fn ACCENT() -> Color32 {
    let l = line_rgb();
    if dark() {
        lerp(l, Color32::WHITE, 0.35)
    } else if luminance(l) > 0.3 {
        lerp(l, Color32::BLACK, 0.45)
    } else {
        l
    }
}

#[allow(non_snake_case)]
pub fn ACCENT_DARK() -> Color32 {
    lerp(line_rgb(), Color32::BLACK, 0.25)
}

#[allow(non_snake_case)]
pub fn ACCENT_SOFT() -> Color32 {
    lerp(line_rgb(), CARD(), if dark() { 0.78 } else { 0.88 })
}

#[allow(non_snake_case)]
pub fn HOVER() -> Color32 {
    lerp(line_rgb(), CARD(), if dark() { 0.88 } else { 0.95 })
}

#[allow(non_snake_case)]
pub fn HOVER_FILL() -> Color32 {
    lerp(line_rgb(), CARD(), if dark() { 0.68 } else { 0.8 })
}

/// A font that has Chinese characters, from the usual system places.
fn cjk_font() -> Option<Vec<u8>> {
    let mut candidates: Vec<String> = Vec::new();
    if cfg!(target_os = "linux") {
        // fc-list, not fc-match: fc-match returns the default font (e.g. Inter)
        // even when it has no Chinese, which left Chinese as empty boxes.
        if let Ok(out) = std::process::Command::new("fc-list")
            .args([":lang=zh", "-f", "%{file}\n"])
            .output()
        {
            let mut files: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|f| f.ends_with(".ttf") || f.ends_with(".otf") || f.ends_with(".ttc"))
                .map(String::from)
                .collect();
            // Full Noto/Source fonts first, then the rest (e.g. Droid fallback).
            files.sort_by_key(|f| match f {
                f if f.contains("CJK") || f.contains("Han") => 0,
                f if f.contains("Full") => 1,
                _ => 2,
            });
            candidates.extend(files);
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

/// Titles and the wordmark: Archivo Bold, the station sign's lettering.
pub fn display_family() -> FontFamily {
    FontFamily::Name("display".into())
}

/// Band text and small sign labels: Archivo SemiBold.
pub fn sign_family() -> FontFamily {
    FontFamily::Name("sign".into())
}

pub fn display(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).family(display_family())
}

pub fn setup(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Noto Sans (OFL, assets/NotoSans-OFL.txt) covers Vietnamese tone letters
    // and every Latin, Greek and Cyrillic accent; egui's own font does not.
    for (name, bytes) in [
        ("noto", &include_bytes!("../assets/NotoSans-Regular.ttf")[..]),
        ("noto-bold", &include_bytes!("../assets/NotoSans-Bold.ttf")[..]),
        ("noto-medium", &include_bytes!("../assets/NotoSans-Medium.ttf")[..]),
        // Archivo (OFL, assets/Archivo-OFL.txt): the signage voice of titles.
        ("archivo-bold", &include_bytes!("../assets/Archivo-Bold.ttf")[..]),
        ("archivo-semibold", &include_bytes!("../assets/Archivo-SemiBold.ttf")[..]),
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
    for (family, first) in [
        (bold_family(), "noto-bold"),
        (medium_family(), "noto-medium"),
        (display_family(), "archivo-bold"),
        (sign_family(), "archivo-semibold"),
    ] {
        let mut list = vec![first.to_string()];
        list.extend(regular.iter().cloned());
        fonts.families.insert(family, list);
    }
    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(26.0, display_family())),
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
    });
    apply_visuals(ctx);
}

/// The colours of every widget, from the palette; again after a switch
/// between light and dark.
pub fn apply_visuals(ctx: &egui::Context) {
    let base = if dark() { egui::Visuals::dark() } else { egui::Visuals::light() };
    ctx.all_styles_mut(|style| {
        style.visuals = base.clone();
        let v = &mut style.visuals;
        v.panel_fill = BG();
        v.window_fill = CARD();
        v.window_stroke = Stroke::new(1.0, LINE());
        v.window_corner_radius = CornerRadius::same(6);
        v.menu_corner_radius = CornerRadius::same(6);
        v.window_shadow = shadow(18, 24);
        v.popup_shadow = shadow(12, 20);
        // Text boxes must stand out on cards.
        v.extreme_bg_color = FIELD();
        v.override_text_color = Some(INK());
        // A selected tab or option is a soft step of the line with ink text (the
        // override above keeps text ink, so a full line fill was unreadable);
        // the stroke is also the focus ring of text boxes, so it must show.
        v.selection.bg_fill = ACCENT_SOFT();
        v.selection.stroke = Stroke::new(1.5, ACCENT());
        v.hyperlink_color = ACCENT();
        v.slider_trailing_fill = true;
        v.handle_shape = egui::style::HandleShape::Circle;
        v.striped = false;
        // Checkboxes take this radius too, so it stays small; buttons set their own.
        let r = CornerRadius::same(4);
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
        // Pressed controls sink a pixel, so a click is felt (no scale in egui).
        v.widgets.active.expansion = -1.0;
        v.widgets.inactive.weak_bg_fill = CARD();
        v.widgets.inactive.bg_fill = CONTROL();
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, CONTROL_LINE());
        v.widgets.inactive.fg_stroke = Stroke::new(1.5, INK());
        v.widgets.hovered.weak_bg_fill = HOVER();
        v.widgets.hovered.bg_fill = HOVER_FILL();
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT());
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, INK());
        v.widgets.active.weak_bg_fill = ACCENT_SOFT();
        v.widgets.active.bg_fill = FILL();
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_DARK());
        v.widgets.open.weak_bg_fill = ACCENT_SOFT();
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE());
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
    RichText::new(text.into()).color(MUTED())
}

/// A small label above a group or a value, in sentence case.
pub fn caps(text: &str) -> RichText {
    medium(text).size(13.0).color(MUTED())
}

/// The title and one line of explanation at the top of a screen.
pub fn page_title(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.add_space(4.0);
    ui.label(display(title).size(30.0));
    if !subtitle.is_empty() {
        ui.label(muted(subtitle).size(15.0));
    }
    ui.add_space(10.0);
}

pub fn section(ui: &mut Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(display(title).size(19.0));
}

// ---------------------------------------------------------------- frames

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD())
        .stroke(Stroke::new(1.0, LINE()))
        .corner_radius(6)
        .inner_margin(Margin::same(20))
}

pub fn tinted(fill: Color32) -> egui::Frame {
    egui::Frame::new().fill(fill).corner_radius(6).inner_margin(Margin::same(16))
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

/// A level or line mark: light letters on an ink square, as on a station
/// sign. `color` is kept for callers; every level is a stretch of one line.
pub fn badge(ui: &mut Ui, text: &str, _color: Color32) {
    egui::Frame::new()
        .fill(INK())
        .corner_radius(3)
        .inner_margin(Margin::symmetric(6, 1))
        .show(ui, |ui| ui.label(RichText::new(text).family(sign_family()).color(BG()).size(12.5)));
}

/// A rounded label: `fg` text on `bg`.
pub fn pill(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(4)
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| ui.label(medium(text).color(on_soft(fg)).size(13.0)))
        .response
}

/// A key of the keyboard as a small cap ("Space").
pub fn keycap(ui: &mut Ui, key: &str) -> Response {
    let galley = ui.painter().layout_no_wrap(key.to_string(), FontId::new(12.0, medium_family()), INK());
    let (rect, resp) = ui.allocate_exact_size(galley.size() + vec2(12.0, 4.0), Sense::hover());
    ui.painter().rect(rect, 4, CARD(), Stroke::new(1.0, CONTROL_LINE()), egui::StrokeKind::Inside);
    ui.painter().galley(rect.min + vec2(6.0, 2.0), galley, INK());
    resp
}

/// The app's one set of buttons: the line-coloured primary, the outlined
/// secondary and the quiet ghost. Each answers the pointer (a hover step,
/// a press that sinks), which a fixed `Button::fill` would switch off.
pub struct Btn {
    button: egui::Button<'static>,
    kind: Kind,
}

#[derive(Clone, Copy)]
enum Kind {
    Primary,
    Secondary,
    Ghost,
    /// On a band of a line colour, in that band's text colour.
    OnLine(Color32),
}

impl Btn {
    pub fn min_size(self, size: egui::Vec2) -> Self {
        Btn { button: self.button.min_size(size), ..self }
    }
}

impl egui::Widget for Btn {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            let w = &mut ui.visuals_mut().widgets;
            match self.kind {
                Kind::Primary => {
                    let fill = FILL();
                    for (state, t) in [(&mut w.inactive, 0.0), (&mut w.hovered, 0.12), (&mut w.active, 0.2)] {
                        state.weak_bg_fill = lerp(fill, Color32::BLACK, t);
                        state.bg_stroke = Stroke::NONE;
                    }
                }
                // The widget visuals of `apply_visuals` are this button's.
                Kind::Secondary => {}
                Kind::Ghost => {
                    w.hovered.bg_stroke = Stroke::NONE;
                    w.active.bg_stroke = Stroke::NONE;
                }
                Kind::OnLine(fg) => {
                    for (state, a) in [(&mut w.hovered, 36), (&mut w.active, 56)] {
                        state.weak_bg_fill = Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), a);
                        state.bg_stroke = Stroke::NONE;
                    }
                }
            }
            ui.add(self.button)
        })
        .inner
    }
}

pub fn primary(text: impl Into<String>) -> Btn {
    let button = egui::Button::new(medium(text).color(ON_FILL())).corner_radius(4).min_size(vec2(0.0, 36.0));
    Btn { button, kind: Kind::Primary }
}

pub fn big_primary(text: impl Into<String>) -> Btn {
    let button = egui::Button::new(medium(text).color(ON_FILL()).size(16.0)).corner_radius(4).min_size(vec2(200.0, 48.0));
    Btn { button, kind: Kind::Primary }
}

pub fn secondary(text: impl Into<String>) -> Btn {
    let button = egui::Button::new(medium(text).color(INK())).corner_radius(4).min_size(vec2(0.0, 36.0));
    Btn { button, kind: Kind::Secondary }
}

/// A quiet action on a band of the line colour, in `fg` (the band's text).
pub fn on_line(text: impl Into<String>, fg: Color32) -> Btn {
    let button = egui::Button::new(medium(text).color(fg)).frame_when_inactive(false).corner_radius(4);
    Btn { button, kind: Kind::OnLine(fg) }
}

/// A quiet action inside running copy: underlined text that gains a soft
/// surface under the pointer (painted beneath it, so the text stays sharp).
pub fn text_link(ui: &mut Ui, text: impl Into<String>, size: f32) -> Response {
    let under = ui.painter().add(egui::Shape::Noop);
    let r = ui.add(egui::Label::new(RichText::new(text.into()).size(size).color(MUTED()).underline()).sense(Sense::click()));
    if r.hovered() || r.has_focus() {
        ui.painter().set(under, egui::Shape::rect_filled(r.rect.expand2(vec2(4.0, 2.0)), 4, HOVER()));
    }
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A borderless button for quiet actions; its surface shows on hover.
pub fn ghost(text: impl Into<String>) -> Btn {
    let button = egui::Button::new(RichText::new(text.into()).color(MUTED())).frame_when_inactive(false).corner_radius(4);
    Btn { button, kind: Kind::Ghost }
}

/// A round filled button with one drawn icon (play, pause, stop).
pub fn round_button(ui: &mut Ui, icon: Icon, size: f32, fill: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let fill = if resp.hovered() { lerp(fill, Color32::BLACK, 0.15) } else { fill };
    // Pressed: the button sinks a little, so the click is felt.
    let r = if resp.is_pointer_button_down_on() { size * 0.47 } else { size / 2.0 };
    ui.painter().circle_filled(rect.center(), r, fill);
    icon.paint(ui.painter(), egui::Rect::from_center_size(rect.center(), vec2(size * 0.42, size * 0.42)), text_on(fill));
    focus_ring(ui, &resp, rect.expand(2.0), size / 2.0 + 2.0);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A drawn icon centred in a soft circle, for empty and finished states.
pub fn icon_circle(ui: &mut Ui, icon: Icon, fg: Color32, bg: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().circle_filled(rect.center(), size / 2.0, bg);
    icon.paint(ui.painter(), egui::Rect::from_center_size(rect.center(), vec2(size * 0.44, size * 0.44)), fg);
}

/// A quiet square button holding one drawn icon (play a sentence, next,
/// remove); `hint` is its tooltip and its name for screen readers.
pub fn icon_ghost(ui: &mut Ui, icon: Icon, hint: &str) -> Response {
    let side = ui.spacing().interact_size.y.min(28.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(side, side), Sense::click());
    let hot = resp.hovered() || resp.has_focus();
    if hot {
        ui.painter().rect_filled(rect, 4, HOVER());
    }
    // Pressed: the icon sinks, as the other buttons do.
    let s = if resp.is_pointer_button_down_on() { 12.0 } else { 13.0 };
    icon.paint(ui.painter(), egui::Rect::from_center_size(rect.center(), vec2(s, s)), if hot { INK() } else { MUTED() });
    focus_ring(ui, &resp, rect, 4.0);
    let label = hint.to_string();
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
    resp.on_hover_text(hint).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The keyboard focus ring of the drawn buttons, in the line's ink.
fn focus_ring(ui: &Ui, resp: &Response, rect: egui::Rect, radius: f32) {
    if resp.has_focus() {
        ui.painter().rect_stroke(rect, radius, Stroke::new(1.5, ACCENT()), egui::StrokeKind::Outside);
    }
}

/// A drawn icon before a short status label, both in `color` ("Finished",
/// "Right", "paused"): one icon set instead of typed symbols.
pub fn icon_label(ui: &mut Ui, icon: Icon, text: RichText, size: f32, color: Color32) -> Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = (size * 0.4).round();
        let (r, _) = ui.allocate_exact_size(vec2(size * 0.8, size), Sense::hover());
        let label = ui.label(text.size(size).color(color));
        let y = label.rect.top() + label.rect.height().min(size * 1.4) / 2.0;
        icon.paint(ui.painter(), egui::Rect::from_center_size(egui::pos2(r.center().x, y), vec2(size * 0.72, size * 0.72)), color);
        label
    })
    .inner
}

/// A drawn icon in a fixed box, for the status column of a list.
pub fn icon_mark(ui: &mut Ui, icon: Icon, color: Color32) -> Response {
    let (r, resp) = ui.allocate_exact_size(vec2(18.0, 22.0), Sense::hover());
    icon.paint(ui.painter(), egui::Rect::from_center_size(r.center(), vec2(12.0, 12.0)), color);
    resp
}

/// A number in a small circle (sentence and note numbers).
pub fn number(ui: &mut Ui, n: usize, active: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
    let (bg, fg) = if active { (FILL(), ON_FILL()) } else { (TRACK(), MUTED()) };
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
/// A language's badge: its code on its line colour.
pub fn lang_badge(ui: &mut Ui, code: &str, color: Color32, size: f32) -> Response {
    // A line's marker, as on a station sign: its code on a square of its colour.
    let (r, resp) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().rect_filled(r, size * 0.12, color);
    let fg = text_on(color);
    ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, code.to_uppercase(), FontId::new(size * 0.38, sign_family()), fg);
    resp
}

/// Text in a coloured label's own colour, kept readable on the dark theme.
pub fn on_soft(color: Color32) -> Color32 {
    if dark() { lerp(color, Color32::WHITE, 0.5) } else { color }
}

pub fn bar(ui: &mut Ui, frac: f32, width: f32, color: Color32) {
    bar_on(ui, frac, width, color, TRACK());
}

pub fn bar_on(ui: &mut Ui, frac: f32, width: f32, color: Color32, track: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 6.0), Sense::hover());
    ui.painter().rect_filled(rect, 2, track);
    let mut fill = rect;
    fill.set_width(rect.width() * frac.clamp(0.0, 1.0));
    if fill.width() > 0.5 {
        ui.painter().rect_filled(fill, 2, color);
    }
}

pub fn dot(ui: &mut Ui, color: Color32, r: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(r * 2.0 + 2.0, r * 2.0 + 2.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), r, color);
}

/// The Marea mark: a station ring half filled by the tide, white on a red
/// enamel tile; the same drawing as packaging/marea.svg.
pub fn logo(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    paint_logo(ui.painter(), rect);
}

pub const BRAND: Color32 = Color32::from_rgb(200, 52, 36);

pub fn paint_logo(p: &egui::Painter, rect: egui::Rect) {
    // SVG units: the tile is 896 wide, the ring's centre in its middle.
    let u = rect.width() / 896.0;
    let c = rect.center();
    p.rect_filled(rect, 200.0 * u, BRAND);
    p.circle_stroke(c, 290.0 * u, Stroke::new(72.0 * u, Color32::WHITE));
    // The water: under a wave, inside the ring, drawn as thin vertical strips.
    let r = 300.0 * u;
    let wave = |x: f32| c.y + 8.0 * u - 28.0 * u * (std::f32::consts::PI * (x / u + 312.0) / 160.0).sin();
    let mut mesh = egui::Mesh::default();
    let steps = 64;
    for i in 0..=steps {
        let x = -r + 2.0 * r * i as f32 / steps as f32;
        let bottom = c.y + (r * r - x * x).max(0.0).sqrt();
        let top = wave(x).min(bottom);
        let n = mesh.vertices.len() as u32;
        mesh.colored_vertex(egui::pos2(c.x + x, top), Color32::WHITE);
        mesh.colored_vertex(egui::pos2(c.x + x, bottom), Color32::WHITE);
        if i > 0 {
            mesh.add_triangle(n - 2, n - 1, n);
            mesh.add_triangle(n - 1, n, n + 1);
        }
    }
    p.add(egui::Shape::mesh(mesh));
}

/// The interface's icons, drawn in one stroke so they match at any size.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Home,
    Play,
    Pause,
    Stop,
    Review,
    Lines,
    Book,
    Pencil,
    Sliders,
    Check,
    Speaker,
    Close,
    Back,
    Forward,
    Arrow,
    /// Today's path: a step still to do, drawn as a hollow station.
    Station,
    /// A live status ("repeat aloud now").
    Dot,
    Rewind,
    SkipBack,
    SkipNext,
    Up,
}

impl Icon {
    pub fn paint(self, p: &egui::Painter, r: egui::Rect, color: Color32) {
        let w = r.width();
        let st = Stroke::new((w * 0.1).clamp(1.5, 3.0), color);
        let at = |x: f32, y: f32| egui::pos2(r.left() + x * w, r.top() + y * r.height());
        let line = |pts: &[(f32, f32)]| egui::Shape::line(pts.iter().map(|&(x, y)| at(x, y)).collect(), st);
        let fill_poly = |pts: &[(f32, f32)]| {
            egui::Shape::convex_polygon(pts.iter().map(|&(x, y)| at(x, y)).collect(), color, Stroke::NONE)
        };
        match self {
            Icon::Home => {
                p.add(line(&[(0.1, 0.48), (0.5, 0.12), (0.9, 0.48)]));
                p.add(line(&[(0.22, 0.38), (0.22, 0.88), (0.78, 0.88), (0.78, 0.38)]));
            }
            Icon::Play => {
                p.add(fill_poly(&[(0.22, 0.1), (0.9, 0.5), (0.22, 0.9)]));
            }
            Icon::Pause => {
                p.rect_filled(egui::Rect::from_min_max(at(0.2, 0.12), at(0.4, 0.88)), 1.0, color);
                p.rect_filled(egui::Rect::from_min_max(at(0.6, 0.12), at(0.8, 0.88)), 1.0, color);
            }
            Icon::Stop => {
                p.rect_filled(egui::Rect::from_min_max(at(0.18, 0.18), at(0.82, 0.82)), 2.0, color);
            }
            Icon::Review => {
                let c = r.center();
                let rad = w * 0.36;
                let arc: Vec<egui::Pos2> = (0..=24)
                    .map(|i| {
                        let a = 0.6 + i as f32 / 24.0 * 4.9;
                        c + vec2(a.cos(), a.sin()) * rad
                    })
                    .collect();
                let end = *arc.last().unwrap();
                p.add(egui::Shape::line(arc, st));
                p.add(egui::Shape::line(vec![end + vec2(-w * 0.2, -w * 0.02), end, end + vec2(-w * 0.02, w * 0.2)], st));
            }
            Icon::Lines => {
                // A route: a line that bends, through two hollow stations.
                let r = w * 0.14;
                p.add(line(&[(0.02, 0.78), (0.24 - 0.14, 0.78)]));
                p.circle_stroke(at(0.24, 0.78), r, st);
                p.add(line(&[(0.34, 0.7), (0.66, 0.3)]));
                p.circle_stroke(at(0.76, 0.22), r, st);
                p.add(line(&[(0.9, 0.22), (0.98, 0.22)]));
            }
            Icon::Book => {
                p.add(line(&[(0.5, 0.22), (0.5, 0.86)]));
                p.add(line(&[(0.5, 0.22), (0.3, 0.14), (0.08, 0.16), (0.08, 0.8), (0.3, 0.78), (0.5, 0.86)]));
                p.add(line(&[(0.5, 0.22), (0.7, 0.14), (0.92, 0.16), (0.92, 0.8), (0.7, 0.78), (0.5, 0.86)]));
            }
            Icon::Pencil => {
                p.add(line(&[(0.16, 0.84), (0.2, 0.64), (0.68, 0.16), (0.86, 0.34), (0.38, 0.82), (0.16, 0.84)]));
                p.add(line(&[(0.58, 0.26), (0.76, 0.44)]));
            }
            Icon::Sliders => {
                for (y, x) in [(0.22, 0.66), (0.5, 0.32), (0.78, 0.58)] {
                    p.add(line(&[(0.08, y), (0.92, y)]));
                    p.circle_filled(at(x, y), w * 0.12, color);
                }
            }
            Icon::Check => {
                p.add(line(&[(0.12, 0.52), (0.4, 0.8), (0.9, 0.2)]));
            }
            Icon::Close => {
                p.add(line(&[(0.2, 0.2), (0.8, 0.8)]));
                p.add(line(&[(0.8, 0.2), (0.2, 0.8)]));
            }
            Icon::Back => {
                p.add(line(&[(0.64, 0.16), (0.3, 0.5), (0.64, 0.84)]));
            }
            Icon::Forward => {
                p.add(line(&[(0.36, 0.16), (0.7, 0.5), (0.36, 0.84)]));
            }
            Icon::Arrow => {
                p.add(line(&[(0.1, 0.5), (0.86, 0.5)]));
                p.add(line(&[(0.58, 0.22), (0.86, 0.5), (0.58, 0.78)]));
            }
            Icon::Speaker => {
                p.add(fill_poly(&[(0.08, 0.36), (0.3, 0.36), (0.56, 0.12), (0.56, 0.88), (0.3, 0.64), (0.08, 0.64)]));
                p.add(line(&[(0.72, 0.32), (0.8, 0.5), (0.72, 0.68)]));
            }
            Icon::Station => {
                p.circle_stroke(r.center(), w * 0.34, st);
            }
            Icon::Dot => {
                p.circle_filled(r.center(), w * 0.3, color);
            }
            Icon::Rewind => {
                p.add(fill_poly(&[(0.5, 0.18), (0.5, 0.82), (0.04, 0.5)]));
                p.add(fill_poly(&[(0.96, 0.18), (0.96, 0.82), (0.5, 0.5)]));
            }
            Icon::SkipBack => {
                p.rect_filled(egui::Rect::from_min_max(at(0.12, 0.16), at(0.26, 0.84)), 1.0, color);
                p.add(fill_poly(&[(0.88, 0.14), (0.88, 0.86), (0.3, 0.5)]));
            }
            Icon::SkipNext => {
                p.add(fill_poly(&[(0.12, 0.14), (0.7, 0.5), (0.12, 0.86)]));
                p.rect_filled(egui::Rect::from_min_max(at(0.74, 0.16), at(0.88, 0.84)), 1.0, color);
            }
            Icon::Up => {
                p.add(line(&[(0.16, 0.66), (0.5, 0.32), (0.84, 0.66)]));
            }
        }
    }
}

/// The part of a line not yet travelled: light on the light theme, and
/// still visible (3:1 or more) but dimmer than `line_ink` on the dark one.
pub fn line_track() -> Color32 {
    if dark() { lerp(FILL(), Color32::from_gray(112), 0.7) } else { lerp(FILL(), BG(), 0.7) }
}

/// A colour drawn as a mark on the ground (a line, a bar): as it is on the
/// light theme, lifted on the dark one, where a deep line colour fell under 3:1.
pub fn mark(color: Color32) -> Color32 {
    if dark() { lerp(color, Color32::WHITE, 0.35) } else { color }
}

/// The travelled line: the line colour, lifted on the dark theme.
pub fn line_ink() -> Color32 {
    mark(FILL())
}

/// 0 to 1 over `secs` with a strong ease-out, from 0 the first time `id` is
/// seen, so a mark can grow in once; later changes of `on` ease both ways.
pub fn ease(ui: &Ui, id: egui::Id, on: bool, secs: f32) -> f32 {
    let ctx = ui.ctx();
    if on && !ctx.data(|d| d.get_temp::<bool>(id).is_some()) {
        ctx.data_mut(|d| d.insert_temp(id, true));
        ctx.animate_bool_with_time(id, false, secs);
    }
    ctx.animate_bool_with_time_and_easing(id, on, secs, egui::emath::easing::cubic_out)
}

/// A position that moves to a new `target` over `secs` with the same
/// ease-out, from wherever it was (egui's own value tween is linear).
pub fn glide(ui: &Ui, id: egui::Id, target: f32, secs: f32) -> f32 {
    let now = ui.input(|i| i.time);
    let (from, to, start) = ui.ctx().data(|d| d.get_temp::<(f32, f32, f64)>(id)).unwrap_or((target, target, now));
    let t = ((now - start) as f32 / secs).clamp(0.0, 1.0);
    let at = from + (to - from) * egui::emath::easing::cubic_out(t);
    if to != target {
        ui.ctx().data_mut(|d| d.insert_temp(id, (at, target, now)));
        ui.ctx().request_repaint();
        return at;
    }
    if t < 1.0 {
        ui.ctx().request_repaint();
    }
    at
}

/// Two or three choices side by side on a track; the chosen one sits on a
/// white chip that slides to a new choice. Returns the choice clicked.
pub fn segmented(ui: &mut Ui, options: &[String], selected: usize, min_width: f32) -> Option<usize> {
    let mut clicked = None;
    egui::Frame::new().fill(TRACK()).corner_radius(6).inner_margin(4).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            // Painted under the labels once their places are known.
            let (hover_slot, chip_slot) = (ui.painter().add(egui::Shape::Noop), ui.painter().add(egui::Shape::Noop));
            let mut rects = Vec::new();
            for (i, text) in options.iter().enumerate() {
                let on = i == selected;
                let b = egui::Button::new(medium(text).color(if on { INK() } else { MUTED() }))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                    .corner_radius(4)
                    .min_size(vec2(min_width, 34.0));
                let r = ui.add(b);
                if r.hovered() && !on {
                    ui.painter().set(hover_slot, egui::Shape::rect_filled(r.rect, 4, lerp(TRACK(), CARD(), 0.5)));
                }
                if r.clicked() {
                    clicked = Some(i);
                }
                rects.push(r.rect);
            }
            // Glides by choice, not by pixel, so a resize never drags it.
            if selected < rects.len() {
                let at = glide(ui, ui.id().with("segmented-chip"), selected as f32, 0.18);
                let (a, b) = (rects[at.floor() as usize], rects[(at.ceil() as usize).min(rects.len() - 1)]);
                let t = at.fract();
                let chip = egui::Rect::from_min_max(a.min + (b.min - a.min) * t, a.max + (b.max - a.max) * t);
                ui.painter().set(chip_slot, egui::Shape::rect_filled(chip, 4, CARD()));
            }
        });
    });
    clicked
}

/// A button with a drawn icon beside its text: before it, or after it when
/// `trailing` (next, more). The icon takes the text's colour.
pub fn icon_button(ui: &mut Ui, button: Btn, icon: Icon, color: Color32, trailing: bool) -> Response {
    let r = ui.add(button);
    button_icon(ui, &r, icon, color, trailing);
    r
}

/// Paints `icon` in the room `icon_room` left in a button already added.
pub fn button_icon(ui: &Ui, r: &Response, icon: Icon, color: Color32, trailing: bool) {
    let x = if trailing { r.rect.right() - 20.0 } else { r.rect.left() + 20.0 };
    icon.paint(ui.painter(), egui::Rect::from_center_size(egui::pos2(x, r.rect.center().y), vec2(13.0, 13.0)), color);
}

/// Room for the drawn icon of `icon_button`, before or after the text.
pub fn icon_room(text: &str, trailing: bool) -> String {
    if trailing { format!("{text}      ") } else { format!("      {text}") }
}

/// One station on the line.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// Done: a filled station.
    Done,
    /// Where the learner is: a large ring.
    Here,
    /// Ahead: a hollow station.
    Ahead,
}

/// The signature of the app: a course as a metro line. `stops` are drawn
/// evenly along a thick line in the line colour; done stations are filled,
/// today's is a large ring, those ahead hollow, and an interchange (a
/// revision lesson) carries a second ring. The line behind is solid as far
/// as the learner has travelled, then a lighter track. Returns the hovered
/// station's index; each station's label is written under it.
pub fn line_rail(ui: &mut Ui, stops: &[(Stop, bool, String)], height: f32) -> (Response, Option<usize>) {
    let width = ui.available_width();
    let (full, resp) = ui.allocate_exact_size(vec2(width, height + 22.0), Sense::hover());
    let rect = egui::Rect::from_min_size(full.min, vec2(width, height));
    let p = ui.painter();
    let n = stops.len().max(1);
    let pad = height * 0.5;
    let x = |i: usize| {
        if n == 1 { rect.center().x } else { rect.left() + pad + (rect.width() - 2.0 * pad) * i as f32 / (n - 1) as f32 }
    };
    let y = rect.center().y;
    let thick = (height * 0.22).clamp(4.0, 10.0);
    let line = line_ink();
    let track = line_track();
    let here = stops.iter().position(|s| s.0 == Stop::Here).unwrap_or_else(|| stops.iter().rposition(|s| s.0 == Stop::Done).unwrap_or(0));
    p.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], Stroke::new(thick, track));
    p.line_segment([egui::pos2(rect.left(), y), egui::pos2(x(here), y)], Stroke::new(thick, line));
    let r = (height * 0.2).clamp(5.0, 12.0);
    let hovered = resp.hover_pos().and_then(|m| (0..n).min_by_key(|&i| (x(i) - m.x).abs() as i32).filter(|&i| (x(i) - m.x).abs() < r * 2.0));
    for (i, (stop, interchange, _)) in stops.iter().enumerate() {
        let (stop, interchange) = (*stop, *interchange);
        let c = egui::pos2(x(i), y);
        // A pointed station grows a little, quickly, so it answers the pointer.
        let grow = 1.0 + 0.15 * ease(ui, ui.id().with(("station-hover", i)), hovered == Some(i), 0.12);
        match stop {
            Stop::Done => {
                p.circle_filled(c, r * grow, line);
                p.circle_filled(c, r * 0.42 * grow, CARD());
            }
            Stop::Here => {
                // The current station grows in once, so the eye finds it.
                let t = ease(ui, ui.id().with("here-ring"), true, 0.24);
                let big = r * (1.0 + 0.5 * t) * grow;
                p.circle_filled(c, big, CARD());
                p.circle_stroke(c, big, Stroke::new(r * 0.55, line));
            }
            Stop::Ahead => {
                p.circle_filled(c, r * grow, CARD());
                p.circle_stroke(c, r * grow, Stroke::new(r * 0.4, track));
            }
        }
        if interchange {
            p.circle_stroke(c, r * 1.9 * grow, Stroke::new(1.5, if stop == Stop::Ahead { track } else { line }));
        }
    }
    // Names under the stations: today's first, then the others where they
    // fit, so a long name hides its neighbours' numbers instead of covering them.
    let mut order: Vec<usize> = (0..n.min(stops.len())).collect();
    order.sort_by_key(|&i| (stops[i].0 != Stop::Here, !stops[i].1));
    let mut taken: Vec<(f32, f32)> = Vec::new();
    for i in order {
        let (stop, _, label) = &stops[i];
        let (font, color) = if *stop == Stop::Here {
            (FontId::new(14.0, sign_family()), INK())
        } else {
            (FontId::new(12.5, FontFamily::Proportional), MUTED())
        };
        let galley = p.layout_no_wrap(label.clone(), font, color);
        let w = galley.size().x;
        let left = (x(i) - w / 2.0).clamp(rect.left(), rect.right() - w);
        let span = (left - 6.0, left + w + 6.0);
        if taken.iter().any(|&(a, b)| span.0 < b && a < span.1) {
            continue;
        }
        taken.push(span);
        p.galley(egui::pos2(left, rect.bottom() + 4.0), galley, color);
    }
    (resp, hovered)
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

/// The tone arrows of a figured Vietnamese line, as in Le Vietnamien sans
/// peine: → ngang and huyền, ✓ ngã and hỏi, ↑ sắc, ↓ nặng.
pub const TONE_ARROWS: [char; 4] = ['→', '✓', '↑', '↓'];

/// A Vietnamese syllable with a low tone (huyền, hỏi, nặng: grave, hook,
/// dot below); the others (ngang, sắc, ngã) are high.
pub fn low_tone(syllable: &str) -> bool {
    const LOW: &str = "àằầèềìòồờùừỳảẳẩẻểỉỏổởủửỷạặậẹệịọộợụựỵÀẰẦÈỀÌÒỒỜÙỪỲẢẲẨẺỂỈỎỔỞỦỬỶẠẶẬẸỆỊỌỘỢỤỰỴ";
    syllable.chars().any(|c| LOW.contains(c))
}

/// A figured line on two rows, high tones above and low tones below, each
/// syllable under or over its place, as the book prints it. `target` is
/// the Vietnamese sentence: one syllable per figured word. None when the
/// two do not line up, so the caller can show one plain line instead.
pub fn tone_rows(ui: &mut Ui, target: &str, figured: &str, size: f32, color: Color32) -> Option<Response> {
    let syllables: Vec<&str> = target.split_whitespace().collect();
    let tokens: Vec<&str> = figured.split_whitespace().collect();
    if tokens.len() != syllables.len() || !figured.contains(TONE_ARROWS) {
        return None;
    }
    let r = ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
        let (mark, _) = ui.allocate_exact_size(vec2(size, size * 1.3), Sense::hover());
        Icon::Speaker.paint(ui.painter(), egui::Rect::from_center_size(mark.center(), vec2(size * 0.8, size * 0.8)), color);
        for (tok, syl) in tokens.iter().zip(&syllables) {
            let low = low_tone(syl);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for row_low in [false, true] {
                    let c = if row_low == low { color } else { Color32::TRANSPARENT };
                    ui.label(RichText::new(*tok).size(size).color(c));
                }
            });
        }
    });
    Some(r.response)
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
/// The pronunciation key, learnt step by step: a line saying it need not
/// be read in one go, the part before the first `###` open, and every `###`
/// section folded until the learner opens it.
pub fn key_markdown(ui: &mut Ui, text: &str, lead: &str) {
    tinted(ACCENT_SOFT()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(medium(lead));
    });
    ui.add_space(8.0);
    let mut parts = text.split("\n### ");
    if let Some(head) = parts.next() {
        markdown(ui, head);
    }
    for (i, part) in parts.enumerate() {
        let (title, body) = part.split_once('\n').unwrap_or((part, ""));
        egui::CollapsingHeader::new(display(title.trim()).size(16.0))
            .id_salt(("key-section", i))
            .default_open(false)
            .show(ui, |ui| markdown(ui, body));
    }
}

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
                1 => display(h.as_str()).size(21.0),
                2 => display(h.as_str()).size(18.0),
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

/// `color` mixed into the card colour: soft backgrounds for coloured labels.
pub fn soft(color: Color32) -> Color32 {
    lerp(color, CARD(), if dark() { 0.78 } else { 0.85 })
}
