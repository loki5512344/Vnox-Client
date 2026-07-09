use crate::state::{TYPING_TIMEOUT, UiState};
use crate::theme as t;
use eframe::egui::{self, FontId, RichText, ScrollArea};
use std::time::Instant;
use vnox_client::{identity::Identity, net::NetHandle};

use super::row::message_row;

pub(crate) fn show(
    ui: &mut egui::Ui,
    s: &mut UiState,
    net: &NetHandle,
    identity: &Identity,
    ch_id: &str,
    max_h: f32,
) {
    let msgs = s.messages.get(ch_id).cloned().unwrap_or_default();
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
                    message_row(ui, s, net, identity, msg, ch_id, group_start);
                    prev = Some(msg.sender_id.clone());
                }
            }
            typing_indicator(ui, s, ch_id);
            ui.add_space(8.0);
        });
    s.scroll_bottom = false;
}

fn typing_indicator(ui: &mut egui::Ui, s: &mut UiState, ch_id: &str) {
    let now = Instant::now();
    if let Some(typers) = s.typing_users.get_mut(ch_id) {
        typers.retain(|(_, t)| now.duration_since(*t) < TYPING_TIMEOUT);
        if typers.is_empty() {
            s.typing_users.remove(ch_id);
            return;
        }
        let names: Vec<&str> = typers.iter().map(|(n, _)| n.as_str()).collect();
        let label = if names.len() == 1 {
            format!("{} is typing...", names[0])
        } else {
            format!(
                "{} and {} are typing...",
                names[..names.len() - 1].join(", "),
                names[names.len() - 1]
            )
        };
        ui.add_space(4.0);
        ui.label(
            RichText::new(label)
                .font(FontId::proportional(11.0))
                .color(t::TEXT_DISABLED),
        );
    }
}
