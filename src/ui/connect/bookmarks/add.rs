use crate::state::UiState;
use crate::theme as t;
use eframe::egui::{self, Frame};

pub fn add_form(ui: &mut egui::Ui, s: &mut UiState) {
    ui.add_space(8.0);
    let mut addr = s.add_server_input.clone();
    Frame::NONE
        .fill(t::BG_INTERACTIVE)
        .corner_radius(t::R_SM)
        .stroke(egui::Stroke::new(1.0, t::BORDER_ACCENT))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut addr)
                        .font(egui::FontId::monospace(12.0))
                        .text_color(t::TEXT_PRIMARY)
                        .hint_text(
                            egui::RichText::new("ip:port or host:port").color(t::TEXT_DISABLED),
                        )
                        .desired_width(ui.available_width() - 120.0)
                        .frame(false),
                );
                let save_btn = egui::Button::new(
                    egui::RichText::new("Save")
                        .font(egui::FontId::proportional(11.0))
                        .color(t::TEXT_PRIMARY)
                        .strong(),
                )
                .fill(t::BG_INTERACTIVE)
                .stroke(egui::Stroke::new(1.0, t::BORDER_ACCENT))
                .corner_radius(t::R_SM)
                .min_size(egui::vec2(50.0, 24.0));
                if ui.add(save_btn).clicked() && !addr.trim().is_empty() {
                    s.connect_input = addr.trim().to_string();
                    s.add_bookmark_from_input();
                    s.add_server_open = false;
                    s.add_server_input = String::new();
                    s.connect_input = String::new();
                }
                if t::icon_btn_drawn(ui, crate::theme::icons::close, "cancel", None, 18.0).clicked()
                {
                    s.add_server_open = false;
                    s.add_server_input = String::new();
                }
            });
        });
    s.add_server_input = addr;
    ui.add_space(6.0);
}
