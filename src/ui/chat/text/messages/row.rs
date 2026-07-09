use crate::state::{ChatMessage, UiState};
use crate::theme as t;
use eframe::egui::{self, FontId, Frame, RichText};
use vnox_client::{
    identity::Identity,
    net::{NetCommand, NetHandle},
};

use super::{fmt_ts, nick_color, own_actions, reaction_buttons, read_receipt_indicator};

pub fn message_row(
    ui: &mut egui::Ui,
    s: &mut UiState,
    net: &NetHandle,
    identity: &Identity,
    msg: &ChatMessage,
    ch_id: &str,
    group_start: bool,
) {
    let nick = s.nick_for(&msg.sender_id).to_string();
    let is_own = msg.sender_id == identity.pubkey_hex;
    let msg_id = msg.message_id.clone();
    let sender_id = msg.sender_id.clone();
    let content_clone = msg.content.clone();

    let row_response = Frame::NONE
        .inner_margin(egui::Margin {
            left: 16,
            right: 16,
            top: if group_start { 6 } else { 1 },
            bottom: 0,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                if group_start {
                    let init = nick
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string();
                    t::avatar(ui, &init, nick_color(s, &msg.sender_id), 32.0);
                } else {
                    ui.add_space(32.0);
                }
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    if group_start {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&nick)
                                    .font(FontId::proportional(13.0))
                                    .color(t::TEXT_PRIMARY)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(fmt_ts(msg.timestamp))
                                    .font(FontId::proportional(11.0))
                                    .color(t::TEXT_DISABLED),
                            );
                            if msg.edited {
                                ui.label(
                                    RichText::new("(edited)")
                                        .font(FontId::proportional(10.0))
                                        .color(t::TEXT_DISABLED),
                                );
                            }
                            if is_own {
                                own_actions(ui, s, net, msg, ch_id);
                            }
                        });
                    }
                    // Reply indicator
                    if let Some(reply_id) = &msg.reply_to {
                        let reply_snippet = s
                            .messages
                            .get(ch_id)
                            .and_then(|msgs| msgs.iter().find(|m| &m.message_id == reply_id))
                            .map(|m| {
                                let reply_nick = s.nick_for(&m.sender_id).to_string();
                                let snippet: String = m.content.chars().take(60).collect();
                                format!("→ {reply_nick}: {snippet}")
                            })
                            .unwrap_or_else(|| "→ (deleted message)".to_string());
                        ui.label(
                            RichText::new(reply_snippet)
                                .font(FontId::proportional(11.0))
                                .color(t::TEXT_MUTED)
                                .italics(),
                        );
                    }
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&msg.content)
                                .font(FontId::proportional(13.0))
                                .color(t::TEXT_SECONDARY),
                        );
                        read_receipt_indicator(ui, s);
                    });
                    if !msg.reactions.is_empty() {
                        reaction_buttons(ui, net, msg, ch_id, identity);
                    }
                });
            });
        })
        .response;

    // Right-click context menu (egui 0.33 API: Response::context_menu).
    row_response.context_menu(|ui| {
        s.context_menu_message_id = Some(msg_id.clone());
        let quick_reactions = ["👍", "❤️", "😂", "🎉", "👀", "🤔"];
        ui.horizontal(|ui| {
            for emoji in quick_reactions {
                let has_reacted = msg
                    .reactions
                    .get(emoji)
                    .is_some_and(|users| users.contains(&identity.pubkey_hex));
                let btn_text = if has_reacted {
                    format!("{emoji} +")
                } else {
                    emoji.to_string()
                };
                if ui
                    .button(RichText::new(btn_text).font(FontId::proportional(14.0)))
                    .clicked()
                {
                    let net = net.clone();
                    let mid = msg_id.clone();
                    let cid = ch_id.to_string();
                    let e = emoji.to_string();
                    if has_reacted {
                        tokio::spawn(async move {
                            net.send(NetCommand::ReactionRemove {
                                channel_id: cid,
                                message_id: mid,
                                emoji: e,
                            })
                            .await;
                        });
                    } else {
                        tokio::spawn(async move {
                            net.send(NetCommand::ReactionAdd {
                                channel_id: cid,
                                message_id: mid,
                                emoji: e,
                            })
                            .await;
                        });
                    }
                    ui.close();
                }
            }
        });
        ui.separator();
        if ui
            .add(
                egui::Button::new(RichText::new("Reply").font(FontId::proportional(12.0)))
                    .fill(egui::Color32::TRANSPARENT)
                    .frame(false),
            )
            .clicked()
        {
            s.replying_to = Some((msg_id.clone(), sender_id.clone()));
            s.editing_message_id = None;
            ui.close();
        }
        if ui
            .add(
                egui::Button::new(RichText::new("Copy text").font(FontId::proportional(12.0)))
                    .fill(egui::Color32::TRANSPARENT)
                    .frame(false),
            )
            .clicked()
        {
            ui.ctx().copy_text(content_clone.clone());
            ui.close();
        }
        if is_own {
            ui.separator();
            if ui
                .add(
                    egui::Button::new(RichText::new("Edit").font(FontId::proportional(12.0)))
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                )
                .clicked()
            {
                s.chat_input = content_clone.clone();
                s.editing_message_id = Some(msg_id.clone());
                s.replying_to = None;
                ui.close();
            }
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("Delete")
                            .font(FontId::proportional(12.0))
                            .color(t::ERROR),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .frame(false),
                )
                .clicked()
            {
                let net = net.clone();
                let mid = msg_id.clone();
                let cid = ch_id.to_string();
                tokio::spawn(async move {
                    net.send(NetCommand::MessageDelete {
                        channel_id: cid,
                        message_id: mid,
                    })
                    .await;
                });
                ui.close();
            }
        }
    });
}
