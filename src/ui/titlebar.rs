use eframe::egui::{self, Context, Frame, TopBottomPanel};

use crate::state::{ConnState, UiState};
use crate::theme as t;

pub const TITLEBAR_H: f32 = 30.0;

pub(crate) fn show_titlebar(ctx: &Context, s: &mut UiState) {
    TopBottomPanel::top("titlebar")
        .exact_height(TITLEBAR_H)
        .frame(Frame::NONE.fill(t::BG_SURFACE))
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(12.0);

                ui.with_layout(
                    egui::Layout::left_to_right(egui::Align::Center).with_main_justify(true),
                    |ui| {
                        let w = ui.available_width();
                        if w.is_finite() && w > 0.0 {
                            ui.set_width(w);
                        }
                        let title = match &s.conn {
                            ConnState::Connected { node_name, .. } => {
                                format!("VNOX  ·  {node_name}")
                            }
                            ConnState::Connecting => "VNOX  ·  connecting…".into(),
                            ConnState::Reconnecting { attempt } => {
                                format!("VNOX  ·  reconnect #{attempt}")
                            }
                            ConnState::Disconnected => "VNOX  ·  offline".into(),
                        };
                        ui.label(
                            egui::RichText::new(title)
                                .font(egui::FontId::monospace(11.0))
                                .color(t::TEXT_MUTED),
                        );
                    },
                );

                if matches!(s.conn, ConnState::Connected { .. })
                    && s.overlay_latency
                    && let Some(rtt) = s.rtt_ms
                {
                    let color = if rtt < 80 {
                        t::SUCCESS
                    } else if rtt < 200 {
                        t::WARNING
                    } else {
                        t::ERROR
                    };
                    ui.label(
                        egui::RichText::new(format!("{rtt} ms"))
                            .font(egui::FontId::monospace(10.0))
                            .color(color),
                    );
                    ui.add_space(8.0);
                }

                let transport = match &s.conn {
                    ConnState::Connected { .. } => "quic/v1",
                    ConnState::Connecting => "connecting…",
                    ConnState::Reconnecting { .. } => "reconnecting…",
                    ConnState::Disconnected => "idle",
                };
                ui.label(
                    egui::RichText::new(transport)
                        .font(egui::FontId::monospace(10.0))
                        .color(t::TEXT_DISABLED),
                );
                ui.add_space(10.0);

                let settings_color = if s.settings_open {
                    Some(t::ACCENT)
                } else {
                    None
                };
                if t::icon_btn_drawn(
                    ui,
                    crate::theme::icons::gear,
                    "Settings",
                    settings_color,
                    18.0,
                )
                .clicked()
                {
                    s.settings_open = !s.settings_open;
                }
                ui.add_space(12.0);
            });
        });
}
