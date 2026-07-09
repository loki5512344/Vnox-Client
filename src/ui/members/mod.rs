//! Right-hand member list panel — shows online/offline members
//! of the currently active channel.

use eframe::egui::{
    self, Color32, Context, FontId, Frame, RichText, ScrollArea, SidePanel, Stroke,
};

use crate::state::{Channel, UiState};
use crate::theme as t;

const PANEL_WIDTH: f32 = 220.0;

pub fn show(ctx: &Context, s: &mut UiState) {
    // Only show for guild text channels (not DMs, not voice-only view, not home/friends).
    let ch_id = match &s.active_channel {
        Some(id) => id.clone(),
        None => return,
    };
    if s.active_guild_id.is_none() {
        return;
    }
    if s.active_dm_id.is_some() {
        return;
    }
    let channel = match s.channels.iter().find(|c| c.id == ch_id) {
        Some(c) => c.clone(),
        None => return,
    };
    if channel.kind == "voice" {
        return;
    }

    SidePanel::right("members")
        .exact_width(PANEL_WIDTH)
        .resizable(false)
        .frame(Frame::NONE.fill(t::BG_SURFACE))
        .show_separator_line(false)
        .show(ctx, |ui| {
            let r = ui.max_rect();
            ui.painter()
                .vline(r.left(), r.y_range(), Stroke::new(1.0, t::BORDER_SUBTLE));

            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                ui.add_space(12.0);
                render_section(ui, s, &channel, true);
                ui.add_space(8.0);
                render_section(ui, s, &channel, false);
            });
        });
}

fn render_section(ui: &mut egui::Ui, s: &UiState, channel: &Channel, online: bool) {
    // Members currently in this channel
    let mut members: Vec<&str> = channel.members.iter().map(|m| s.nick_for(m)).collect();

    // Add current user if not present
    let self_nick = "you";
    if !members.contains(&self_nick) {
        members.push(self_nick);
    }

    // Sort alphabetically (case-insensitive)
    members.sort_by_key(|a| a.to_lowercase());

    let visible: Vec<&str> = if online {
        members.clone()
    } else {
        Vec::new() // we don't track offline members per channel yet
    };

    if online && visible.is_empty() {
        return;
    }

    let header = if online {
        format!("ONLINE — {}", visible.len())
    } else {
        "OFFLINE — 0".to_string()
    };

    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(
            RichText::new(header)
                .font(FontId::proportional(10.0))
                .color(t::TEXT_MUTED)
                .strong(),
        );
    });
    ui.add_space(4.0);

    ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for nick in &visible {
                member_row(ui, nick, online);
            }
        });
}

fn member_row(ui: &mut egui::Ui, nick: &str, online: bool) {
    let (dot_color, text_color) = if online {
        (t::SUCCESS, t::TEXT_PRIMARY)
    } else {
        (t::TEXT_DISABLED, t::TEXT_DISABLED)
    };

    Frame::NONE
        .inner_margin(egui::Margin {
            left: 12,
            right: 8,
            top: 2,
            bottom: 2,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let init = nick
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                let avatar_color = if online {
                    nick_avatar_color(nick)
                } else {
                    t::BG_ELEVATED
                };
                let (r, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 12.0, avatar_color);
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &init,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(15, 15, 15),
                );
                // Status dot
                let (sd, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(sd.center(), 4.0, dot_color);

                ui.label(
                    RichText::new(nick)
                        .font(FontId::proportional(12.0))
                        .color(text_color),
                );
            });
        });
}

fn nick_avatar_color(nick: &str) -> egui::Color32 {
    let colors = [
        t::ACCENT,
        t::SUCCESS,
        t::WARNING,
        t::INFO,
        Color32::from_rgb(200, 100, 180),
        Color32::from_rgb(120, 190, 180),
    ];
    colors[nick.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len()]
}
