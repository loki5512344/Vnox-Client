use crate::theme as t;
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Stroke};

pub fn page_header(ui: &mut egui::Ui, title: &str, sub: &str) {
    ui.label(
        RichText::new(title)
            .font(FontId::monospace(13.0))
            .color(t::TEXT_SECONDARY)
            .strong(),
    );
    ui.add_space(2.0);
    ui.label(
        RichText::new(sub)
            .font(FontId::monospace(9.0))
            .color(t::TEXT_DEAD),
    );
    ui.add_space(14.0);
}

pub fn group_label(ui: &mut egui::Ui, label: &str) {
    ui.add_space(2.0);
    ui.label(
        RichText::new(label.to_uppercase())
            .font(FontId::monospace(9.0))
            .color(t::TEXT_DEAD),
    );
    ui.add(egui::Separator::default().spacing(6.0));
    ui.add_space(2.0);
}

pub fn s_row(
    ui: &mut egui::Ui,
    label: &str,
    desc: Option<&str>,
    content: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_min_width(200.0);
            ui.label(
                RichText::new(label)
                    .font(FontId::monospace(11.0))
                    .color(t::TEXT_DIM),
            );
            if let Some(d) = desc {
                ui.label(
                    RichText::new(d)
                        .font(FontId::monospace(9.0))
                        .color(t::TEXT_DEAD),
                );
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), content);
    });
    ui.add_space(4.0);
    ui.add(egui::Separator::default().horizontal().spacing(0.0));
    ui.add_space(4.0);
}

pub fn device_meta_row(ui: &mut egui::Ui, meta: Option<&vnox_client::audio::devices::DeviceLabel>) {
    if let Some(m) = meta {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            super::controls::badge(ui, &m.host_api, Color32::from_rgb(120, 180, 255));
            if m.channels > 0 {
                super::controls::badge(
                    ui,
                    &format!("{}ch", m.channels),
                    Color32::from_rgb(180, 200, 120),
                );
            }
        });
        ui.add_space(2.0);
    }
}

pub fn id_button(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .font(FontId::monospace(9.0))
                .color(t::TEXT_MUTED),
        )
        .fill(Color32::from_rgb(23, 23, 23))
        .stroke(Stroke::new(1.0, t::BORDER_SUBTLE))
        .corner_radius(4u8),
    )
    .clicked()
}
