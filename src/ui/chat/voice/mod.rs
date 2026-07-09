mod panel;

use eframe::egui::{self, FontId, Frame, RichText};

use crate::state::{Channel, UiState, VoiceUiState};
use crate::theme as t;

pub(crate) fn voice_panel(ui: &mut egui::Ui, s: &mut UiState, ch: &Channel) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        Frame::NONE
            .fill(t::BG_SURFACE)
            .inner_margin(egui::Margin::symmetric(16, 0))
            .show(ui, |ui| {
                ui.set_height(40.0);
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new("~").color(t::TEXT_MUTED).size(15.0));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(&ch.name)
                            .font(FontId::proportional(14.0))
                            .color(t::TEXT_PRIMARY)
                            .strong(),
                    );
                    if let Some(elapsed) = s.voice_elapsed() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(elapsed)
                                    .font(FontId::monospace(11.0))
                                    .color(t::TEXT_DISABLED),
                            );
                        });
                    }
                });
            });

        let r = ui.max_rect();
        ui.painter().hline(
            r.x_range(),
            r.top() + 40.0,
            egui::Stroke::new(1.0, t::BORDER_SUBTLE),
        );

        ui.add_space(16.0);
        Frame::NONE
            .inner_margin(egui::Margin::symmetric(20, 0))
            .show(ui, |ui| {
                // Voice activity banner — shows when local or remote user is speaking
                if s.local_speaking || s.remote_speaking {
                    voice_activity_banner(ui, s);
                    ui.add_space(8.0);
                }
                if ch.members.is_empty() {
                    ui.label(
                        RichText::new("no members yet")
                            .font(FontId::proportional(12.0))
                            .color(t::TEXT_DISABLED),
                    );
                }
                // Identify which member is currently speaking by sender_id.
                let speaking_uid = if !s.last_remote_speaker_id.is_empty() {
                    Some(s.last_remote_speaker_id.clone())
                } else {
                    None
                };
                for nick in &ch.members {
                    let user_id = s
                        .user_names
                        .iter()
                        .find(|(_, n)| *n == nick)
                        .map(|(id, _)| id.clone())
                        .unwrap_or_else(|| nick.clone());
                    let member_speaking = speaking_uid.as_deref() == Some(&user_id);
                    panel::member_card(
                        ui,
                        nick,
                        &user_id,
                        &mut s.per_user_volumes,
                        member_speaking,
                    );
                    ui.add_space(6.0);
                }
            });

        if let VoiceUiState::Error(ref err) = s.voice {
            ui.add_space(16.0);
            Frame::NONE
                .fill(egui::Color32::from_rgba_premultiplied(217, 96, 74, 20))
                .corner_radius(t::R_MD)
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_premultiplied(217, 96, 74, 60),
                ))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("microphone error: {err}"))
                            .font(FontId::proportional(12.0))
                            .color(t::ERROR),
                    );
                });
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(16.0);
            panel::voice_controls(ui, s);
        });
    });
}

fn voice_activity_banner(ui: &mut egui::Ui, s: &UiState) {
    let remote_nick: Option<String> = if !s.last_remote_speaker_id.is_empty() {
        Some(s.nick_for(&s.last_remote_speaker_id).to_string())
    } else {
        None
    };
    let (label, color) = match (s.local_speaking, s.remote_speaking, remote_nick.as_deref()) {
        (true, true, Some(nick)) => (format!("you and {nick} are talking"), t::SUCCESS),
        (true, true, None) => ("you and others are talking".to_string(), t::SUCCESS),
        (true, false, _) => ("you are talking".to_string(), t::SUCCESS),
        (false, true, Some(nick)) => (format!("{nick} is talking"), t::INFO),
        (false, true, None) => ("someone is talking".to_string(), t::INFO),
        _ => return,
    };
    Frame::NONE
        .fill(egui::Color32::from_rgba_premultiplied(
            color.r(),
            color.g(),
            color.b(),
            20,
        ))
        .corner_radius(t::R_MD)
        .stroke(egui::Stroke::new(1.0, color))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Pulsating dot
                let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 5.0, color);
                ui.label(
                    RichText::new(label)
                        .font(FontId::proportional(12.0))
                        .color(color)
                        .strong(),
                );
            });
        });
}
