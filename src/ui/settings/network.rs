use super::widgets::{badge, group_label, page_header, s_row, toggle};
use crate::state::UiState;
use crate::theme as t;
use eframe::egui::{self, Align, FontId, Frame, Layout, Margin, RichText, Stroke};

pub fn network(ui: &mut egui::Ui, s: &mut UiState) {
    page_header(ui, "network", "LNEx protocol · relay · connection");

    // Live status card
    Frame::new()
        .fill(t::BG_STRIP)
        .stroke(Stroke::new(1.0, t::BORDER_FAINT))
        .corner_radius(6u8)
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            status_row(ui, "protocol", |ui| badge(ui, "LNEx v1", t::ACCENT_SOFT));
            status_row(ui, "voice transport", |ui| badge(ui, "UDP", t::BLUE));
            status_row(ui, "control transport", |ui| badge(ui, "TCP", t::WARN));
            status_row(ui, "encryption", |ui| {
                badge(ui, "on · chacha20", t::SUCCESS)
            });
            status_row(ui, "rtt", |ui| {
                let rtt = s
                    .rtt_ms
                    .map(|ms| format!("{ms}ms"))
                    .unwrap_or_else(|| "—".into());
                let color = s
                    .rtt_ms
                    .map(|ms| {
                        if ms < 50 {
                            t::SUCCESS
                        } else if ms < 120 {
                            t::WARN
                        } else {
                            t::DANGER
                        }
                    })
                    .unwrap_or(t::TEXT_GHOST);
                ui.label(
                    RichText::new(rtt)
                        .font(FontId::monospace(10.0))
                        .color(color),
                );
            });
            status_row(ui, "packet loss", |ui| {
                ui.label(
                    RichText::new("0.1%")
                        .font(FontId::monospace(10.0))
                        .color(t::SUCCESS),
                );
            });
        });
    ui.add_space(12.0);

    group_label(ui, "relay");
    s_row(
        ui,
        "use relay node",
        Some("route through relay if direct fails"),
        |ui| {
            toggle(ui, &mut s.relay_enabled);
        },
    );
    s_row(ui, "relay address", None, |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut s.relay_address)
                .font(FontId::monospace(10.0))
                .text_color(t::TEXT_MUTED)
                .desired_width(140.0),
        );
    });
    s_row(
        ui,
        "auto relay selection",
        Some("pick nearest by latency"),
        |ui| {
            toggle(ui, &mut s.auto_relay);
        },
    );
    ui.add_space(8.0);

    group_label(ui, "connection");
    s_row(
        ui,
        "auto reconnect",
        Some("retry with backoff after disconnect"),
        |ui| {
            toggle(ui, &mut s.auto_reconnect);
        },
    );
    ui.add_space(8.0);

    group_label(ui, "advanced");
    s_row(ui, "UDP port", None, |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut s.udp_port)
                .font(FontId::monospace(10.0))
                .text_color(t::TEXT_MUTED)
                .desired_width(70.0),
        );
    });
    s_row(ui, "force relay only", Some("disable direct p2p"), |ui| {
        toggle(ui, &mut s.force_relay);
    });
}

fn status_row(ui: &mut egui::Ui, key: &str, val: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(key)
                .font(FontId::monospace(10.0))
                .color(t::TEXT_DEAD),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), val);
    });
    ui.add_space(4.0);
}

pub fn overlay(ui: &mut egui::Ui, s: &mut UiState) {
    page_header(ui, "overlay", "in-game HUD · speaking indicators");

    group_label(ui, "general");
    s_row(ui, "enable overlay", None, |ui| {
        toggle(ui, &mut s.overlay_enabled);
    });
    s_row(ui, "show speaking users", None, |ui| {
        toggle(ui, &mut s.overlay_speakers);
    });
    s_row(ui, "show latency", None, |ui| {
        toggle(ui, &mut s.overlay_latency);
    });
}
