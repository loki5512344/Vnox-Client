use eframe::egui::{self, Color32, Rect, Stroke};

use super::*;

pub fn avatar(ui: &mut egui::Ui, initials: &str, color: Color32, size: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.circle_filled(rect.center(), size / 2.0, color);
        painter.circle_stroke(rect.center(), size / 2.0, Stroke::new(1.0, BORDER_DEFAULT));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            initials,
            egui::FontId::proportional(size * 0.38),
            TEXT_PRIMARY,
        );
    }
    resp
}

pub fn icon_btn(
    ui: &mut egui::Ui,
    icon: &str,
    tooltip: &str,
    active_color: Option<Color32>,
) -> egui::Response {
    let color = active_color.unwrap_or(TEXT_MUTED);
    let btn = egui::Button::new(egui::RichText::new(icon).color(color).size(14.0))
        .frame(false)
        .min_size(egui::vec2(26.0, 26.0));
    let resp = ui.add(btn);
    if !tooltip.is_empty() {
        resp.clone().on_hover_text(tooltip);
    }
    resp
}

/// Draw a vector icon into a button-sized area.
/// `draw_fn` is one of the functions from `crate::theme::icons` (e.g. `icons::gear`).
/// The icon is rendered with the given color; pass `None` to use TEXT_MUTED.
pub fn icon_btn_drawn(
    ui: &mut egui::Ui,
    draw_fn: fn(&egui::Painter, Rect, Color32),
    tooltip: &str,
    active_color: Option<Color32>,
    size: f32,
) -> egui::Response {
    let color = active_color.unwrap_or(TEXT_MUTED);
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        // Inset to give padding.
        let pad = size * 0.10;
        let inner = Rect::from_min_max(
            (rect.min.x + pad, rect.min.y + pad).into(),
            (rect.max.x - pad, rect.max.y - pad).into(),
        );
        draw_fn(painter, inner, color);
    }
    if ui.is_rect_visible(rect) && resp.hovered() {
        let painter = ui.painter();
        painter.rect_filled(rect, egui::CornerRadius::same(4), BG_ELEVATED);
        // Re-draw the icon on top of the hover background.
        let pad = size * 0.10;
        let inner = Rect::from_min_max(
            (rect.min.x + pad, rect.min.y + pad).into(),
            (rect.max.x - pad, rect.max.y - pad).into(),
        );
        draw_fn(painter, inner, color);
    }
    if !tooltip.is_empty() {
        resp.clone().on_hover_text(tooltip);
    }
    resp
}
