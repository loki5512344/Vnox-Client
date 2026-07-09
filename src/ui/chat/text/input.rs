use crate::state::{ChatMessage, UiState};
use crate::theme as t;
use eframe::egui::{self, Context, FontId, Frame, RichText};
use std::collections::HashMap;
use std::time::Instant;
use vnox_client::{
    identity::Identity,
    net::{NetCommand, NetHandle},
};

pub(crate) fn chat_input_bar(
    ui: &mut egui::Ui,
    ctx: &Context,
    s: &mut UiState,
    net: &NetHandle,
    ch_id: &str,
    identity: &Identity,
) {
    let r = ui.available_rect_before_wrap();
    ui.painter().hline(
        r.x_range(),
        r.top(),
        egui::Stroke::new(1.0, t::BORDER_SUBTLE),
    );

    Frame::NONE
        .fill(t::BG_SURFACE)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            // Reply / edit preview bar
            if let Some((ref _reply_id, ref reply_sender)) = s.replying_to.clone() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("replying to")
                            .font(FontId::proportional(11.0))
                            .color(t::ACCENT),
                    );
                    let sender_nick = s.nick_for(reply_sender).to_string();
                    ui.label(
                        RichText::new(sender_nick)
                            .font(FontId::proportional(11.0))
                            .color(t::TEXT_SECONDARY),
                    );
                    if t::icon_btn_drawn(ui, crate::theme::icons::close, "cancel reply", None, 14.0)
                        .clicked()
                    {
                        s.replying_to = None;
                    }
                });
            }
            if s.editing_message_id.is_some() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("editing")
                            .font(FontId::proportional(11.0))
                            .color(t::WARNING),
                    );
                    if t::icon_btn_drawn(ui, crate::theme::icons::close, "cancel edit", None, 14.0)
                        .clicked()
                    {
                        s.chat_input.clear();
                        s.editing_message_id = None;
                    }
                });
            }
            ui.horizontal(|ui| {
                let hint_text = if s.editing_message_id.is_some() {
                    "edit your message...".to_string()
                } else if s.replying_to.is_some() {
                    "write your reply...".to_string()
                } else {
                    format!("message in #{ch_id}")
                };
                let input = egui::TextEdit::singleline(&mut s.chat_input)
                    .font(FontId::proportional(13.0))
                    .text_color(t::TEXT_PRIMARY)
                    .hint_text(RichText::new(hint_text).color(t::TEXT_DISABLED))
                    .frame(false)
                    .desired_width(ui.available_width() - 44.0);

                let enter = Frame::NONE
                    .fill(t::BG_INTERACTIVE)
                    .corner_radius(t::R_MD)
                    .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .show(ui, |ui| ui.add(input))
                    .inner;

                if !s.chat_input.is_empty() && s.last_typing_send.elapsed().as_secs() >= 3 {
                    s.last_typing_send = Instant::now();
                    let net = net.clone();
                    let c = ch_id.to_string();
                    tokio::spawn(async move {
                        net.send(NetCommand::TypingStart { channel_id: c }).await;
                    });
                }

                let submit = (enter.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui
                        .add(
                            egui::Button::new(RichText::new("→").color(t::ACCENT).size(14.0))
                                .fill(t::ACCENT_10)
                                .stroke(egui::Stroke::new(1.0, t::BORDER_ACCENT))
                                .corner_radius(t::R_MD)
                                .min_size(egui::vec2(32.0, 32.0)),
                        )
                        .clicked();

                if submit && !s.chat_input.trim().is_empty() {
                    if let Some(edit_id) = s.editing_message_id.clone() {
                        let content = s.chat_input.trim().to_string();
                        s.chat_input.clear();
                        s.editing_message_id = None;
                        let net = net.clone();
                        let cid = ch_id.to_string();
                        tokio::spawn(async move {
                            net.send(NetCommand::MessageEdit {
                                channel_id: cid,
                                message_id: edit_id,
                                content,
                            })
                            .await;
                        });
                    } else {
                        let reply_to = s.replying_to.clone().map(|(id, _)| id);
                        send_chat(s, net, ch_id, &identity.pubkey_hex, reply_to);
                        s.replying_to = None;
                    }
                    ctx.request_repaint();
                }
            });
        });
}

fn send_chat(
    s: &mut UiState,
    net: &NetHandle,
    ch_id: &str,
    sender_id: &str,
    reply_to: Option<String>,
) {
    let content = s.chat_input.trim().to_string();
    if content.is_empty() {
        return;
    }
    s.chat_input.clear();

    s.messages
        .entry(ch_id.to_string())
        .or_default()
        .push(ChatMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender_id: sender_id.to_string(),
            content: content.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
            edited: false,
            reactions: HashMap::new(),
            reply_to: reply_to.clone(),
        });
    s.scroll_bottom = true;

    let need_join = s.server_channel.as_deref() != Some(ch_id);
    if need_join {
        s.server_channel = Some(ch_id.to_string());
    }

    let net = net.clone();
    let ch = ch_id.to_string();
    tokio::spawn(async move {
        if need_join {
            net.send(NetCommand::JoinChannel {
                channel_id: ch.clone(),
            })
            .await;
        }
        net.send(NetCommand::SendChat {
            channel_id: ch,
            content,
            reply_to,
        })
        .await;
    });
}
