//! Unified left sidebar (design from aiupdate).

mod channel_list;
mod guild_bar;
mod sections;

pub use guild_bar::show as guild_bar;

use eframe::egui::{self, Context, Frame, SidePanel};
use vnox_client::{identity::Identity, net::NetHandle};

use crate::{
    connect,
    state::{ConnState, UiState},
    theme as t,
};

pub fn show(ctx: &Context, s: &mut UiState, identity: &Identity, net: &NetHandle) {
    SidePanel::left("sidebar")
        .exact_width(220.0)
        .resizable(false)
        .frame(Frame::NONE.fill(t::BG_SURFACE))
        .show_separator_line(false)
        .show(ctx, |ui| {
            let r = ui.max_rect();
            ui.painter().vline(
                r.right(),
                r.y_range(),
                egui::Stroke::new(1.0, t::BORDER_SUBTLE),
            );

            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                sections::header::show(ui, s);
                bookmark_row(ui, s, net);

                let userbar_h = 52.0;
                let avail = (ui.available_height() - userbar_h).max(0.0);
                egui::ScrollArea::vertical()
                    .max_height(avail)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(6.0);
                        channel_list::show(ui, s, net);
                        ui.add_space(4.0);
                        sections::dm::show(ui, s, net);
                        ui.add_space(6.0);
                    });

                sections::userbar::show(ui, s, identity, net);
            });
        });
}

fn bookmark_row(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    let busy = matches!(
        s.conn,
        ConnState::Connecting | ConnState::Reconnecting { .. }
    );
    if s.bookmarks.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
        ui.add_space(8.0);
        for (i, bm) in s.bookmarks.clone().into_iter().enumerate() {
            let active = s.selected_bookmark == i;
            let label = bm.label.chars().take(8).collect::<String>();
            let btn = egui::Button::new(
                egui::RichText::new(label)
                    .font(egui::FontId::proportional(10.0))
                    .color(if active {
                        t::ACCENT_SOFT
                    } else {
                        t::TEXT_MUTED
                    }),
            )
            .fill(if active { t::ACCENT_10 } else { t::BG_ELEVATED })
            .stroke(egui::Stroke::new(
                1.0,
                if active {
                    t::BORDER_ACCENT
                } else {
                    t::BORDER_DEFAULT
                },
            ))
            .corner_radius(t::R_SM);
            if ui.add(btn).on_hover_text(&bm.address).clicked() && !busy {
                s.selected_bookmark = i;
                connect::connect_to(s, net, &bm.address);
            }
        }
    });
    ui.add_space(4.0);
}
