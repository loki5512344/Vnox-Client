use eframe::egui::{self, Context, FontId, RichText};

use crate::state::UiState;
use crate::theme as t;
use vnox_client::{identity::Identity, net::NetHandle};

mod dm;
mod friends;
mod text;
mod voice;

pub use friends::show as friends_panel;

pub fn show(
    ui: &mut egui::Ui,
    ctx: &Context,
    s: &mut UiState,
    identity: &Identity,
    net: &NetHandle,
) {
    // If a DM conversation is active, show the DM chat view
    if let Some(dm_id) = s.active_dm_id.clone() {
        // Clear any active channel when showing DM
        s.active_channel = None;
        dm::dm_chat(ui, ctx, s, identity, net, &dm_id);
        return;
    }

    // Home screen — show Friends panel if no guild and no DM
    if s.active_guild_id.is_none() {
        friends::show(ui, s, net);
        return;
    }

    let ch_id = match s.active_channel.clone() {
        Some(id) => id,
        None => {
            empty_state(ui, "pick a channel");
            return;
        }
    };

    let ch = match s.channels.iter().find(|c| c.id == ch_id) {
        Some(c) => c.clone(),
        None => {
            empty_state(ui, "channel not found");
            return;
        }
    };

    if ch.kind == "voice" {
        voice::voice_panel(ui, s, &ch);
    } else {
        text::text_chat(ui, ctx, s, net, identity, &ch_id, &ch.name);
    }
}

fn empty_state(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(text)
                .font(FontId::monospace(12.0))
                .color(t::TEXT_DEAD),
        );
    });
}
