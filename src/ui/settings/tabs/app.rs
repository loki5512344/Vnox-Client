use crate::state::UiState;
use crate::theme as t;
use crate::ui::settings::widgets::{
    badge, group_label, page_header, s_row, slider_row, small_select, toggle,
};
use eframe::egui::{self, FontId, RichText};

pub fn appearance(ui: &mut egui::Ui, s: &mut UiState) {
    page_header(ui, "appearance", "theme · font · ui scale");
    group_label(ui, "theme");
    s_row(ui, "color scheme", None, |ui| {
        let mut v = 0usize;
        small_select(
            ui,
            "scheme",
            &mut v,
            &["hifi dark", "terminal green", "void"],
        );
    });
    s_row(ui, "ui scale", None, |ui| {
        slider_row(ui, &mut s.ui_scale, 80.0, 130.0, |v| {
            format!("{}%", v as u32)
        });
    });
    s_row(ui, "font", None, |ui| {
        let mut v = 0usize;
        small_select(
            ui,
            "font",
            &mut v,
            &["IBM Plex Mono", "Geist Mono", "Fira Code"],
        );
    });
}

pub fn plugins(ui: &mut egui::Ui) {
    page_header(ui, "plugins", "installed · runtime: deno");
    group_label(ui, "installed");
    s_row(ui, "relay-switcher", Some("v0.1.2 · raven"), |ui| {
        let mut v = true;
        toggle(ui, &mut v);
    });
    s_row(ui, "music-bot", Some("v0.3.0 · community"), |ui| {
        let mut v = false;
        toggle(ui, &mut v);
    });
}

pub fn advanced(ui: &mut egui::Ui, s: &mut UiState) {
    page_header(ui, "advanced", "debug · logging · protocol internals");
    group_label(ui, "logging");
    s_row(ui, "log level", None, |ui| {
        small_select(
            ui,
            "log_level",
            &mut s.log_level,
            &["info", "debug", "trace"],
        );
    });
    s_row(ui, "log voice packets", Some("verbose · high cpu"), |ui| {
        toggle(ui, &mut s.log_voice_packets);
    });
    ui.add_space(8.0);
    group_label(ui, "protocol");
    s_row(ui, "LNEx version", None, |ui| {
        badge(ui, "v1", t::ACCENT_SOFT);
    });
    s_row(ui, "show packet stats", None, |ui| {
        toggle(ui, &mut s.show_packet_stats);
    });
    s_row(
        ui,
        "disable encryption",
        Some("debug only · dangerous"),
        |ui| {
            ui.label(
                RichText::new("!")
                    .font(FontId::monospace(10.0))
                    .color(t::DANGER),
            );
            ui.add_space(4.0);
            toggle(ui, &mut s.disable_encryption);
        },
    );
}

pub fn keybinds(ui: &mut egui::Ui, s: &mut UiState) {
    page_header(ui, "keybinds", "global hotkeys");
    group_label(ui, "voice");
    kb_row(
        ui,
        "push-to-talk",
        &mut s.kb_ptt,
        &mut s.kb_listening,
        "ptt",
    );
    kb_row(
        ui,
        "mute toggle",
        &mut s.kb_mute,
        &mut s.kb_listening,
        "mute",
    );
    kb_row(
        ui,
        "deafen toggle",
        &mut s.kb_deafen,
        &mut s.kb_listening,
        "deafen",
    );
    ui.add_space(8.0);
    group_label(ui, "overlay");
    kb_row(
        ui,
        "toggle overlay",
        &mut s.kb_overlay,
        &mut s.kb_listening,
        "overlay",
    );
}

fn kb_row(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut String,
    listening: &mut Option<String>,
    listen_key: &str,
) {
    s_row(ui, label, None, |ui| {
        let is_listening = listening.as_deref() == Some(listen_key);
        if is_listening {
            if let Some(binding) = crate::keybind::listen_pressed(ui.ctx()) {
                *val = binding;
                *listening = None;
            }
            let btn = egui::Button::new(
                egui::RichText::new("Listening… press a key")
                    .font(egui::FontId::monospace(10.0))
                    .color(egui::Color32::WHITE),
            )
            .fill(crate::theme::ACCENT)
            .corner_radius(crate::theme::R_SM)
            .min_size(egui::vec2(120.0, 20.0));
            if ui.add(btn).clicked() {
                *listening = None;
            }
        } else {
            ui.add(
                egui::TextEdit::singleline(val)
                    .font(egui::FontId::monospace(10.0))
                    .text_color(crate::theme::TEXT_MUTED)
                    .desired_width(90.0),
            );
            let rec = egui::Button::new(
                egui::RichText::new("●")
                    .font(egui::FontId::monospace(9.0))
                    .color(crate::theme::ERROR),
            )
            .fill(egui::Color32::TRANSPARENT)
            .frame(false)
            .min_size(egui::vec2(16.0, 16.0));
            if ui.add(rec).on_hover_text("record keybind").clicked() {
                *listening = Some(listen_key.to_string());
            }
        }
    });
}
