use super::super::text::message_row;
use crate::state::{ChatMessage, UiState};
use crate::theme as t;
use eframe::egui::{self, Context, FontId, Frame, RichText, ScrollArea, TextEdit};
use std::collections::HashMap;
use vnox_client::{
    identity::Identity,
    net::{NetCommand, NetHandle},
};

pub(super) fn search_bar(
    ui: &mut egui::Ui,
    s: &mut UiState,
    net: &NetHandle,
    dm_id: &str,
    _h: f32,
) {
    Frame::NONE
        .inner_margin(egui::Margin::symmetric(12, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let resp = ui.add_sized(
                    egui::vec2(ui.available_width(), 20.0),
                    TextEdit::singleline(&mut s.dm_search_query)
                        .font(FontId::proportional(12.0))
                        .text_color(t::TEXT_PRIMARY)
                        .hint_text(RichText::new("search…").color(t::TEXT_DISABLED))
                        .frame(true),
                );
                if resp.changed() {
                    let query = s.dm_search_query.trim().to_string();
                    let net = net.clone();
                    let id = dm_id.to_string();
                    tokio::spawn(async move {
                        net.send(NetCommand::DmSearch { dm_id: id, query }).await;
                    });
                }
            });
        });
}

pub(super) fn show(
    ui: &mut egui::Ui,
    s: &mut UiState,
    net: &NetHandle,
    identity: &Identity,
    dm_id: &str,
    max_h: f32,
) {
    let msgs = s.dm_messages.get(dm_id).cloned().unwrap_or_default();
    ScrollArea::vertical()
        .max_height(max_h)
        .auto_shrink([false; 2])
        .stick_to_bottom(s.scroll_bottom)
        .show(ui, |ui| {
            ui.add_space(12.0);
            if msgs.is_empty() {
                ui.label(
                    RichText::new("no messages yet")
                        .font(FontId::proportional(12.0))
                        .color(t::TEXT_DISABLED),
                );
            } else {
                ui.label(
                    RichText::new("today")
                        .font(FontId::proportional(11.0))
                        .color(t::TEXT_DISABLED),
                );
                ui.add_space(8.0);
                let mut prev: Option<String> = None;
                for msg in &msgs {
                    let group_start = prev.as_deref() != Some(&msg.sender_id);
                    message_row(ui, s, net, identity, msg, dm_id, group_start);
                    prev = Some(msg.sender_id.clone());
                }
            }
            ui.add_space(8.0);
        });
    s.scroll_bottom = false;
}

pub(super) fn input_bar(
    ui: &mut egui::Ui,
    ctx: &Context,
    s: &mut UiState,
    net: &NetHandle,
    _identity: &Identity,
    dm_id: &str,
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
            ui.horizontal(|ui| {
                let input = egui::TextEdit::singleline(&mut s.chat_input)
                    .font(FontId::proportional(13.0))
                    .text_color(t::TEXT_PRIMARY)
                    .hint_text(RichText::new("message…").color(t::TEXT_DISABLED))
                    .frame(false)
                    .desired_width(ui.available_width() - 44.0);

                let enter = Frame::NONE
                    .fill(t::BG_INTERACTIVE)
                    .corner_radius(t::R_MD)
                    .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .show(ui, |ui| ui.add(input))
                    .inner;

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
                    send_dm(s, net, dm_id);
                    ctx.request_repaint();
                }
            });
        });
}

fn send_dm(s: &mut UiState, net: &NetHandle, dm_id: &str) {
    let content = s.chat_input.trim().to_string();
    if content.is_empty() {
        return;
    }
    s.chat_input.clear();

    s.dm_messages
        .entry(dm_id.to_string())
        .or_default()
        .push(ChatMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender_id: String::new(),
            content: content.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
            edited: false,
            reactions: HashMap::new(),
            reply_to: None,
        });
    s.scroll_bottom = true;

    let net = net.clone();
    let id = dm_id.to_string();
    tokio::spawn(async move {
        net.send(NetCommand::DmSend { dm_id: id, content }).await;
    });
}
