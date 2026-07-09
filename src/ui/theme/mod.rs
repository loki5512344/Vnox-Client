pub mod icons;
pub mod widgets;

pub use widgets::{avatar, icon_btn, icon_btn_drawn};

use eframe::egui::{self, Color32, CornerRadius, FontDefinitions, Stroke, Visuals};

pub const BG_BASE: Color32 = Color32::from_rgb(13, 13, 13);
pub const BG_SURFACE: Color32 = Color32::from_rgb(22, 22, 22);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(28, 28, 28);
pub const BG_INTERACTIVE: Color32 = Color32::from_rgb(30, 30, 30);

pub const ACCENT: Color32 = Color32::from_rgb(230, 126, 34);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(245, 145, 55);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(200, 110, 30);
pub const ACCENT_10: Color32 = Color32::from_rgba_premultiplied(230, 126, 34, 25);
pub const ACCENT_20: Color32 = Color32::from_rgba_premultiplied(230, 126, 34, 50);
pub const BORDER_ACCENT: Color32 = Color32::from_rgba_premultiplied(230, 126, 34, 60);

pub const SUCCESS: Color32 = Color32::from_rgb(80, 180, 90);
pub const WARNING: Color32 = Color32::from_rgb(220, 170, 60);
pub const ERROR: Color32 = Color32::from_rgb(210, 90, 80);
pub const INFO: Color32 = Color32::from_rgb(90, 155, 210);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(224, 224, 224);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(170, 170, 170);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(136, 136, 136);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(90, 90, 90);
pub const TEXT_DEAD: Color32 = Color32::from_rgb(55, 55, 55);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(42, 42, 42);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(51, 51, 51);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(64, 64, 64);

pub const BG_STRIP: Color32 = BG_SURFACE;
pub const BG_PANEL: Color32 = BG_SURFACE;
pub const BORDER: Color32 = BORDER_DEFAULT;
pub const BORDER_FAINT: Color32 = BORDER_SUBTLE;
pub const BORDER_MID: Color32 = BORDER_STRONG;
pub const WARN: Color32 = WARNING;
pub const DANGER: Color32 = ERROR;
pub const BLUE: Color32 = INFO;
pub const TEXT_GHOST: Color32 = TEXT_DISABLED;
pub const TEXT_DIM: Color32 = TEXT_MUTED;
pub const ACCENT_DIM: Color32 = Color32::from_rgb(160, 90, 25);
pub const TEXT_FAINT: Color32 = TEXT_DEAD;

pub const R_SM: CornerRadius = CornerRadius::same(6);
pub const R_MD: CornerRadius = CornerRadius::same(10);
pub const R_LG: CornerRadius = CornerRadius::same(14);
pub const R_FULL: CornerRadius = CornerRadius::same(255);

const NAME_PALETTE: [Color32; 6] = [
    Color32::from_rgb(170, 170, 170),
    Color32::from_rgb(106, 158, 207),
    Color32::from_rgb(124, 184, 122),
    Color32::from_rgb(220, 170, 60),
    Color32::from_rgb(190, 140, 200),
    Color32::from_rgb(120, 190, 180),
];

pub fn user_color(user_id: &str) -> Color32 {
    if user_id.is_empty() {
        return TEXT_SECONDARY;
    }
    let hash: usize = user_id
        .bytes()
        .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
    NAME_PALETTE[hash % NAME_PALETTE.len()]
}

pub fn apply_visuals(ctx: &egui::Context) {
    let mut v = Visuals::dark();

    v.window_fill = BG_BASE;
    v.panel_fill = BG_BASE;
    v.extreme_bg_color = BG_INTERACTIVE;
    v.faint_bg_color = BG_ELEVATED;
    v.code_bg_color = BG_SURFACE;
    v.hyperlink_color = ACCENT;
    v.override_text_color = Some(TEXT_PRIMARY);

    v.widgets.noninteractive.bg_fill = BG_SURFACE;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);
    v.widgets.noninteractive.corner_radius = R_MD;

    v.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);
    v.widgets.inactive.corner_radius = R_MD;

    v.widgets.hovered.bg_fill = BG_INTERACTIVE;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.hovered.corner_radius = R_MD;

    v.widgets.active.bg_fill = BG_INTERACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, BORDER_ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.active.corner_radius = R_MD;

    v.widgets.open.bg_fill = BG_INTERACTIVE;
    v.widgets.open.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    v.widgets.open.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.open.corner_radius = R_MD;

    v.selection.bg_fill = ACCENT_10;
    v.selection.stroke = Stroke::new(0.0, Color32::TRANSPARENT);

    v.window_corner_radius = R_LG;
    v.window_shadow = egui::epaint::Shadow::NONE;
    v.popup_shadow = egui::epaint::Shadow::NONE;
    v.window_stroke = Stroke::new(1.0, BORDER_DEFAULT);

    ctx.set_visuals(v);
    set_fonts(ctx);
    set_spacing(ctx);
}

fn set_fonts(ctx: &egui::Context) {
    let fonts = FontDefinitions::default();
    ctx.set_fonts(fonts);
}

fn set_spacing(ctx: &egui::Context) {
    let mut s = (*ctx.style()).clone();
    s.text_styles = [
        (egui::TextStyle::Small, egui::FontId::proportional(11.0)),
        (egui::TextStyle::Body, egui::FontId::proportional(13.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(13.0)),
        (egui::TextStyle::Heading, egui::FontId::proportional(15.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(12.0)),
    ]
    .into();
    s.spacing.item_spacing = egui::vec2(6.0, 4.0);
    s.spacing.button_padding = egui::vec2(10.0, 5.0);
    s.spacing.window_margin = egui::Margin::same(0);
    s.spacing.menu_margin = egui::Margin::same(4);
    s.spacing.indent = 16.0;
    s.spacing.interact_size = egui::vec2(40.0, 28.0);
    s.spacing.scroll.bar_width = 4.0;
    ctx.set_style(s);
}
