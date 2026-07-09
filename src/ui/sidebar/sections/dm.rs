use eframe::egui::{self, Frame};
use vnox_client::net::{NetCommand, NetHandle};

use crate::state::{ConnState, UiState};
use crate::theme as t;

pub fn show(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    if !matches!(s.conn, ConnState::Connected { .. }) {
        return;
    }
    Frame::NONE
        .inner_margin(egui::Margin {
            left: 12,
            right: 8,
            top: 2,
            bottom: 2,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("DIRECT MESSAGES")
                        .font(egui::FontId::proportional(10.0))
                        .color(t::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("+")
                                    .font(egui::FontId::proportional(12.0))
                                    .color(t::ACCENT),
                            )
                            .frame(false)
                            .min_size(egui::vec2(16.0, 16.0)),
                        )
                        .on_hover_text("new DM")
                        .clicked()
                    {
                        s.new_dm_input = Some(String::new());
                    }
                });
            });
        });
    new_dm_input(ui, s, net);
    for conv in s.dm_conversations.clone() {
        let active = s.active_dm_id.as_deref() == Some(&conv.dm_id);
        let bg = if active {
            t::ACCENT_10
        } else {
            egui::Color32::TRANSPARENT
        };
        let stroke = if active {
            egui::Stroke::new(1.0, t::BORDER_ACCENT)
        } else {
            egui::Stroke::NONE
        };
        let text_c = if active {
            t::ACCENT_SOFT
        } else {
            t::TEXT_MUTED
        };

        let resp = Frame::NONE
            .fill(bg)
            .stroke(stroke)
            .corner_radius(t::R_SM)
            .inner_margin(egui::Margin::symmetric(8, 3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("@").color(text_c).size(13.0));
                    ui.label(
                        egui::RichText::new(&conv.other_nickname)
                            .color(text_c)
                            .size(13.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if conv.unread_count > 0 {
                            let text = if conv.unread_count > 99 {
                                "99+".to_string()
                            } else {
                                conv.unread_count.to_string()
                            };
                            let (r, _) = ui
                                .allocate_exact_size(egui::vec2(18.0, 14.0), egui::Sense::hover());
                            ui.painter()
                                .rect_filled(r, egui::CornerRadius::same(4), t::ACCENT);
                            ui.painter().text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                &text,
                                egui::FontId::monospace(10.0),
                                egui::Color32::WHITE,
                            );
                        }
                    });
                });
            })
            .response
            .interact(egui::Sense::click());

        if resp.clicked() {
            let clicked_dm_id = conv.dm_id.clone();
            s.active_dm_id = Some(clicked_dm_id.clone());
            if let Some(c) = s
                .dm_conversations
                .iter_mut()
                .find(|c| c.dm_id == conv.dm_id)
            {
                c.unread_count = 0;
            }
            let net = net.clone();
            tokio::spawn(async move {
                net.send(NetCommand::DmReadAck {
                    dm_id: clicked_dm_id,
                })
                .await;
            });
        }
    }
}

pub fn new_dm_input(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    let mut input_buf = s.new_dm_input.clone().unwrap_or_default();
    let mut should_start = false;
    if s.new_dm_input.is_some() {
        Frame::NONE
            .inner_margin(egui::Margin::symmetric(12, 2))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let resp = ui.add_sized(
                        egui::vec2(120.0, 20.0),
                        egui::TextEdit::singleline(&mut input_buf)
                            .font(egui::FontId::proportional(11.0))
                            .text_color(t::TEXT_PRIMARY)
                            .hint_text(egui::RichText::new("user id").color(t::TEXT_DISABLED))
                            .frame(true),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        || ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Start")
                                        .font(egui::FontId::proportional(10.0))
                                        .color(t::ACCENT),
                                )
                                .fill(t::ACCENT_10)
                                .stroke(egui::Stroke::new(1.0, t::BORDER_ACCENT))
                                .corner_radius(t::R_SM)
                                .min_size(egui::vec2(40.0, 18.0)),
                            )
                            .clicked()
                            && !input_buf.trim().is_empty()
                    {
                        should_start = true;
                    }
                });
            });
    }
    if should_start {
        let target = input_buf.trim().to_string();
        let net = net.clone();
        tokio::spawn(async move {
            net.send(NetCommand::DmStart {
                target_user_id: target,
            })
            .await;
        });
        s.new_dm_input = None;
    }
}
