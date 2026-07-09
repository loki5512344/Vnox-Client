use eframe::egui::{self, Frame};
use vnox_client::{identity::Identity, net::NetHandle};

use crate::state::{ConnState, UiState};
use crate::theme as t;

mod action;
mod bookmarks;

pub use action::{connect_to, disconnect};

pub fn show(ui: &mut egui::Ui, s: &mut UiState, _identity: &Identity, net: &NetHandle) {
    ui.vertical_centered(|ui| {
        ui.set_max_width(520.0);
        ui.add_space(40.0);

        ui.label(
            egui::RichText::new("VNOX")
                .font(egui::FontId::proportional(32.0))
                .color(t::TEXT_PRIMARY)
                .strong(),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("self-hosted voice & chat")
                .font(egui::FontId::proportional(13.0))
                .color(t::TEXT_MUTED),
        );
        ui.add_space(32.0);

        bookmarks::server_list(ui, s, net);

        ui.add_space(20.0);

        ui.label(
            egui::RichText::new("Quick Connect")
                .font(egui::FontId::proportional(11.0))
                .color(t::TEXT_MUTED),
        );
        ui.add_space(6.0);

        Frame::NONE
            .fill(t::BG_INTERACTIVE)
            .corner_radius(t::R_MD)
            .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(">")
                            .font(egui::FontId::monospace(12.0))
                            .color(t::ACCENT_SOFT),
                    );
                    let w = ui.available_width().max(80.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut s.connect_input)
                            .font(egui::FontId::monospace(13.0))
                            .text_color(t::TEXT_PRIMARY)
                            .frame(false)
                            .hint_text(
                                egui::RichText::new("127.0.0.1:7600").color(t::TEXT_DISABLED),
                            )
                            .desired_width(w),
                    );
                });
            });

        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.add_space(8.0);

        let label = match &s.conn {
            ConnState::Disconnected => "Connect",
            ConnState::Connecting | ConnState::Reconnecting { .. } => "Cancel",
            ConnState::Connected { .. } => "Disconnect",
        };
        let btn = egui::Button::new(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(13.0))
                .color(t::TEXT_PRIMARY)
                .strong(),
        )
        .fill(t::BG_ELEVATED)
        .stroke(egui::Stroke::new(1.0, t::BORDER_ACCENT))
        .corner_radius(t::R_MD)
        .min_size(egui::vec2(200.0, 38.0));

        if ui.add(btn).clicked() || enter_pressed {
            match &s.conn {
                ConnState::Disconnected => {
                    let addr = s.connect_input.trim().to_owned();
                    if !addr.is_empty() {
                        connect_to(s, net, &addr);
                    }
                }
                _ => disconnect(s, net),
            }
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut s.auto_reconnect, "");
            ui.label(
                egui::RichText::new("auto-reconnect on drop")
                    .font(egui::FontId::proportional(12.0))
                    .color(t::TEXT_MUTED),
            );
        });

        if let Some(ref msg) = s.status_msg {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(msg)
                    .font(egui::FontId::proportional(11.0))
                    .color(action::msg_color(msg)),
            );
        }

        ui.add_space(28.0);
        let sep = ui.available_rect_before_wrap();
        let painter = ui.painter();
        painter.hline(
            sep.x_range(),
            sep.top(),
            egui::Stroke::new(1.0, t::BORDER_SUBTLE),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("💡").font(egui::FontId::proportional(13.0)));
            ui.label(
                egui::RichText::new("Host your own —")
                    .font(egui::FontId::proportional(12.0))
                    .color(t::TEXT_MUTED),
            );
            ui.label(
                egui::RichText::new("github.com/loki5512344/Vnox")
                    .font(egui::FontId::proportional(12.0))
                    .color(t::ACCENT_SOFT),
            );
            ui.label(
                egui::RichText::new(" · self-host in 5 min")
                    .font(egui::FontId::proportional(12.0))
                    .color(t::TEXT_DISABLED),
            );
        });

        ui.add_space(20.0);
    });
}
