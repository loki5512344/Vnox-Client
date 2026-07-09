use eframe::egui::{self, FontId, Frame, RichText};
use vnox_client::{
    identity::Identity,
    net::{NetCommand, NetHandle},
};

use crate::state::UiState;
use crate::theme as t;
use crate::theme::icons;

pub fn show(ui: &mut egui::Ui, s: &mut UiState, identity: &Identity, net: &NetHandle) {
    let r = ui.available_rect_before_wrap();
    ui.painter().hline(
        r.x_range(),
        r.top(),
        egui::Stroke::new(1.0, t::BORDER_SUBTLE),
    );

    Frame::NONE
        .fill(t::BG_ELEVATED)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let init = identity
                    .nickname
                    .chars()
                    .next()
                    .unwrap_or('U')
                    .to_uppercase()
                    .to_string();
                t::avatar(
                    ui,
                    &init,
                    avatar_color_for(&identity.nickname, s.local_speaking),
                    30.0,
                );
                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&identity.nickname)
                                .font(egui::FontId::proportional(12.5))
                                .color(if s.local_speaking {
                                    t::SUCCESS
                                } else {
                                    t::TEXT_PRIMARY
                                })
                                .strong(),
                        );
                        if s.local_speaking {
                            let (r, _) = ui
                                .allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                            icons::speaker(ui.painter(), r, t::SUCCESS);
                        }
                    });

                    // Status text — shows voice state, custom status, or activity.
                    let status_text = if let Some(ch) = s.active_voice_channel() {
                        if s.remote_speaking {
                            format!("in voice · {} · talking", ch.name)
                        } else {
                            format!("in voice · {}", ch.name)
                        }
                    } else if let Some(ref custom) = s.custom_status {
                        custom.clone()
                    } else if let (Some(_kind), Some(text)) = (&s.activity_type, &s.activity_text) {
                        text.clone()
                    } else if s.overlay_latency {
                        match s.rtt_ms {
                            Some(ms) => format!("online · {ms}ms"),
                            None => "online".into(),
                        }
                    } else {
                        "online".into()
                    };

                    let status_label = ui.add(
                        egui::Button::new(
                            RichText::new(status_text)
                                .font(FontId::proportional(10.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    );
                    status_label.context_menu(|ui| {
                        status_picker(ui, s, net);
                        ui.separator();
                        custom_status_editor(ui, s, net);
                        ui.separator();
                        activity_editor(ui, s, net);
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 1.0;
                    if t::icon_btn_drawn(ui, icons::gear, "settings", None, 22.0).clicked() {
                        s.settings_open = true;
                    }
                    let deaf_c = if s.deafen { Some(t::ERROR) } else { None };
                    let deaf_icon = if s.deafen {
                        icons::headphones_off
                    } else {
                        icons::headphones
                    };
                    if t::icon_btn_drawn(ui, deaf_icon, "deafen", deaf_c, 22.0).clicked() {
                        s.deafen = !s.deafen;
                    }
                    let mic_icon = if s.mic_enabled {
                        icons::microphone
                    } else {
                        icons::microphone_off
                    };
                    let mic_c = if !s.mic_enabled {
                        Some(t::ERROR)
                    } else if s.local_speaking {
                        Some(t::SUCCESS)
                    } else {
                        None
                    };
                    if t::icon_btn_drawn(ui, mic_icon, "microphone", mic_c, 22.0).clicked() {
                        s.mic_enabled = !s.mic_enabled;
                    }
                });
            });
        });
}

fn status_picker(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    ui.label(
        RichText::new("Status")
            .font(FontId::proportional(10.0))
            .color(t::TEXT_MUTED)
            .strong(),
    );
    let options = [
        ("online", "Online"),
        ("idle", "Idle"),
        ("dnd", "Do Not Disturb"),
        ("invisible", "Invisible"),
    ];
    for (key, label) in options {
        let selected = s.presence_status.as_deref() == Some(key);
        let btn = egui::Button::new(RichText::new(label).font(FontId::proportional(12.0)))
            .fill(if selected {
                t::ACCENT_10
            } else {
                egui::Color32::TRANSPARENT
            })
            .frame(false);
        if ui.add(btn).clicked() {
            s.presence_status = Some(key.to_string());
            send_presence(s, net);
            ui.close();
        }
    }
}

fn custom_status_editor(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    ui.label(
        RichText::new("Custom status")
            .font(FontId::proportional(10.0))
            .color(t::TEXT_MUTED)
            .strong(),
    );
    let mut buf = s.custom_status.clone().unwrap_or_default();
    let resp = ui.add(
        egui::TextEdit::singleline(&mut buf)
            .font(FontId::proportional(12.0))
            .hint_text("what's on your mind?")
            .desired_width(180.0)
            .text_color(t::TEXT_PRIMARY),
    );
    if resp.changed() {
        s.custom_status = if buf.trim().is_empty() {
            None
        } else {
            Some(buf)
        };
    }
    if resp.lost_focus() {
        send_presence(s, net);
    }
    if ui
        .add(
            egui::Button::new(
                RichText::new("Clear")
                    .font(FontId::proportional(11.0))
                    .color(t::TEXT_MUTED),
            )
            .fill(egui::Color32::TRANSPARENT)
            .frame(false),
        )
        .clicked()
    {
        s.custom_status = None;
        send_presence(s, net);
        ui.close();
    }
}

fn activity_editor(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    ui.label(
        RichText::new("Activity")
            .font(FontId::proportional(10.0))
            .color(t::TEXT_MUTED)
            .strong(),
    );
    let kinds = [
        ("playing", "Playing"),
        ("listening", "Listening"),
        ("watching", "Watching"),
        ("streaming", "Streaming"),
    ];
    let mut current_kind = s.activity_type.clone();
    let combo = egui::ComboBox::from_id_salt("activity_kind")
        .selected_text(current_kind.clone().unwrap_or_else(|| "— none —".into()))
        .width(160.0);
    combo.show_ui(ui, |ui| {
        for (k, label) in kinds {
            ui.selectable_value(&mut current_kind, Some(k.to_string()), label);
        }
        ui.selectable_value(&mut current_kind, None, "— none —");
    });
    let mut buf = s.activity_text.clone().unwrap_or_default();
    let resp = ui.add(
        egui::TextEdit::singleline(&mut buf)
            .font(FontId::proportional(12.0))
            .hint_text("doing what?")
            .desired_width(180.0)
            .text_color(t::TEXT_PRIMARY),
    );
    let mut changed = false;
    if resp.changed() {
        s.activity_text = if buf.trim().is_empty() {
            None
        } else {
            Some(buf)
        };
        changed = true;
    }
    if current_kind != s.activity_type {
        s.activity_type = current_kind;
        changed = true;
    }
    if changed && (resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter))) {
        send_presence(s, net);
    }
}

fn send_presence(s: &UiState, net: &NetHandle) {
    let net = net.clone();
    let status = s.presence_status.clone().unwrap_or_else(|| "online".into());
    let custom = s.custom_status.clone();
    let activity = s.activity_type.clone();
    let activity_text = s.activity_text.clone();
    tokio::spawn(async move {
        net.send(NetCommand::PresenceUpdate {
            status,
            custom_status: custom,
            activity,
            activity_text,
        })
        .await;
    });
}

fn avatar_color_for(nick: &str, speaking: bool) -> egui::Color32 {
    if speaking {
        return t::SUCCESS;
    }
    let colors = [
        t::ACCENT_10,
        egui::Color32::from_rgb(50, 70, 100),
        egui::Color32::from_rgb(60, 80, 50),
        egui::Color32::from_rgb(80, 60, 80),
    ];
    colors[nick.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len()]
}
