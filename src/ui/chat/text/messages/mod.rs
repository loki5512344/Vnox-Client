pub mod list;
pub mod row;

pub(crate) use list::show;
pub use row::message_row;

use crate::state::{ChatMessage, UiState};
use crate::theme as t;
use crate::theme::icons;
use eframe::egui::{self, RichText};
use vnox_client::{
    identity::Identity,
    net::{NetCommand, NetHandle},
};

fn own_actions(
    ui: &mut egui::Ui,
    s: &mut UiState,
    net: &NetHandle,
    msg: &ChatMessage,
    ch_id: &str,
) {
    if t::icon_btn_drawn(ui, icons::pencil, "edit", None, 16.0).clicked() {
        s.chat_input = msg.content.clone();
        s.editing_message_id = Some(msg.message_id.clone());
    }
    if t::icon_btn_drawn(ui, icons::trash, "delete", None, 16.0).clicked() {
        let net = net.clone();
        let mid = msg.message_id.clone();
        let cid = ch_id.to_string();
        tokio::spawn(async move {
            net.send(NetCommand::MessageDelete {
                channel_id: cid,
                message_id: mid,
            })
            .await;
        });
    }
}

fn reaction_buttons(
    ui: &mut egui::Ui,
    net: &NetHandle,
    msg: &ChatMessage,
    ch_id: &str,
    identity: &Identity,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (emoji, users) in &msg.reactions {
            let count = users.len();
            let has_reacted = users.iter().any(|u| u == &identity.pubkey_hex);
            let bg = if has_reacted {
                t::ACCENT_20
            } else {
                t::BG_ELEVATED
            };
            let border = if has_reacted {
                t::BORDER_ACCENT
            } else {
                t::BORDER_SUBTLE
            };
            let label = format!("{} {}", emoji, count);
            if ui
                .add(
                    egui::Button::new(RichText::new(&label).size(12.0).color(t::TEXT_SECONDARY))
                        .fill(bg)
                        .stroke(egui::Stroke::new(1.0, border))
                        .corner_radius(4.0)
                        .min_size(egui::Vec2::ZERO),
                )
                .clicked()
            {
                let net = net.clone();
                let mid = msg.message_id.clone();
                let cid = ch_id.to_string();
                let e = emoji.clone();
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
            }
        }
    });
}

fn read_receipt_indicator(ui: &mut egui::Ui, s: &UiState) {
    if let Some(ch) = &s.active_channel
        && s.read_receipts.contains_key(ch)
    {
        ui.add_space(6.0);
        let (rr, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
        icons::check(ui.painter(), rr, t::TEXT_DISABLED);
    }
}

fn fmt_ts(ts: i64) -> String {
    let s = (ts / 1000).abs() % 86400;
    format!("{:02}:{:02}", s / 3600, (s % 3600) / 60)
}

fn nick_color(s: &UiState, sender_id: &str) -> egui::Color32 {
    if let Some(color) = s.user_colors.get(sender_id) {
        return *color;
    }
    let nick = s.nick_for(sender_id);
    let c = [
        t::ACCENT,
        t::SUCCESS,
        t::WARNING,
        t::INFO,
        egui::Color32::from_rgb(200, 100, 180),
    ];
    c[nick.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % c.len()]
}
