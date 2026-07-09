use crate::theme as t;
use eframe::egui::{self, FontId, RichText};

pub fn nav_item(ui: &mut egui::Ui, is_active: bool, label: &str) -> bool {
    let text_color = if is_active {
        t::ACCENT_SOFT
    } else {
        t::TEXT_GHOST
    };
    let fill = if is_active {
        t::ACCENT_10
    } else {
        egui::Color32::TRANSPARENT
    };
    let stroke = if is_active {
        egui::Stroke::new(1.0, t::BORDER_ACCENT)
    } else {
        egui::Stroke::NONE
    };

    ui.add_sized(
        [ui.available_width().max(120.0), 26.0],
        egui::Button::new(
            RichText::new(label)
                .font(FontId::monospace(11.0))
                .color(text_color),
        )
        .fill(fill)
        .stroke(stroke)
        .corner_radius(t::R_SM),
    )
    .clicked()
}

pub fn nav_section(ui: &mut egui::Ui, label: &str) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(label.to_uppercase())
                .font(FontId::monospace(9.0))
                .color(t::TEXT_DEAD),
        );
    });
    ui.add_space(2.0);
}
