use eframe::egui::{self, FontId, Rect, RichText, Stroke};

use crate::theme as t;

pub const GUILD_ICON_SIZE: f32 = 40.0;

pub fn guild_icon(
    ui: &mut egui::Ui,
    label: &str,
    tooltip: &str,
    active: bool,
    onclick: impl FnOnce(),
) {
    let (bg, stroke) = if active {
        (t::ACCENT_10, Stroke::new(1.5, t::ACCENT))
    } else {
        (t::BG_ELEVATED, Stroke::new(1.0, t::BORDER_DEFAULT))
    };

    let text_color = if active {
        t::ACCENT_SOFT
    } else {
        t::TEXT_MUTED
    };

    let btn = egui::Button::new(
        RichText::new(label)
            .font(FontId::proportional(14.0))
            .color(text_color),
    )
    .fill(bg)
    .stroke(stroke)
    .corner_radius(t::R_FULL)
    .min_size(egui::vec2(GUILD_ICON_SIZE, GUILD_ICON_SIZE));

    if ui.add(btn).on_hover_text(tooltip).clicked() {
        onclick();
    }
}

/// Guild icon that renders a vector-drawn glyph instead of a text label.
pub fn guild_icon_drawn(
    ui: &mut egui::Ui,
    draw_fn: fn(&egui::Painter, Rect, eframe::egui::Color32),
    tooltip: &str,
    active: bool,
    onclick: impl FnOnce(),
) {
    let (bg, stroke) = if active {
        (t::ACCENT_10, Stroke::new(1.5, t::ACCENT))
    } else {
        (t::BG_ELEVATED, Stroke::new(1.0, t::BORDER_DEFAULT))
    };
    let color = if active {
        t::ACCENT_SOFT
    } else {
        t::TEXT_MUTED
    };

    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(GUILD_ICON_SIZE, GUILD_ICON_SIZE),
        egui::Sense::click(),
    );
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, t::R_FULL, bg);
        painter.rect_stroke(rect, t::R_FULL, stroke, egui::StrokeKind::Middle);
        // Inset for the icon.
        let pad = 8.0;
        let inner = Rect::from_min_max(
            (rect.min.x + pad, rect.min.y + pad).into(),
            (rect.max.x - pad, rect.max.y - pad).into(),
        );
        draw_fn(painter, inner, color);
    }
    if resp.clicked() {
        onclick();
    }
    if !tooltip.is_empty() {
        resp.on_hover_text(tooltip);
    }
}
