use eframe::egui::{self, Frame};

use crate::state::{ConnState, UiState};
use crate::theme as t;

pub fn show(ui: &mut egui::Ui, s: &UiState) {
    let title = if let Some(gid) = &s.active_guild_id
        && let Some(g) = s.guilds.iter().find(|g| &g.guild_id == gid)
    {
        g.name.clone()
    } else {
        match &s.conn {
            ConnState::Connected { node_name, .. } => node_name.clone(),
            ConnState::Reconnecting { attempt } => format!("reconnect #{attempt}"),
            ConnState::Connecting => "connecting…".into(),
            ConnState::Disconnected => "not connected".into(),
        }
    };

    let is_guild = s.active_guild_id.is_some();

    Frame::NONE
        .fill(t::BG_SURFACE)
        .inner_margin(egui::Margin::symmetric(12, 0))
        .show(ui, |ui| {
            ui.set_height(38.0);
            ui.horizontal_centered(|ui| {
                if is_guild {
                    let (r, _) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, t::R_FULL, t::ACCENT);
                    ui.painter().text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        title
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string(),
                        egui::FontId::monospace(11.0),
                        egui::Color32::WHITE,
                    );
                } else {
                    let (r, _) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, t::R_SM, t::ACCENT_20);
                    ui.painter().rect_stroke(
                        r,
                        t::R_SM,
                        egui::Stroke::new(1.0, t::BORDER_ACCENT),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        "V",
                        egui::FontId::monospace(11.0),
                        t::TEXT_PRIMARY,
                    );
                }

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(title)
                        .font(egui::FontId::proportional(13.0))
                        .color(t::TEXT_PRIMARY)
                        .strong(),
                );

                if s.private_mode {
                    ui.add_space(6.0);
                    Frame::NONE
                        .fill(t::WARNING)
                        .corner_radius(t::R_SM)
                        .inner_margin(egui::Margin::symmetric(4, 1))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("PRIVATE")
                                    .font(egui::FontId::monospace(8.0))
                                    .color(egui::Color32::from_rgb(15, 15, 15))
                                    .strong(),
                            );
                        });
                }
            });
        });

    let sep = ui.max_rect();
    ui.painter().hline(
        sep.x_range(),
        sep.top() + 38.0,
        egui::Stroke::new(1.0, t::BORDER_SUBTLE),
    );
}
