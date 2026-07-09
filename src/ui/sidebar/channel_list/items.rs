use eframe::egui::{self, Frame};

use crate::theme as t;
use vnox_client::net::{NetCommand, NetHandle};

/// Channel row with optional context menu for deletion.
/// When `net` is provided and the channel is not a default one, a "Delete channel"
/// entry appears in the right-click context menu.
pub(super) fn ch_row_with_menu(
    ui: &mut egui::Ui,
    icon: &str,
    name: &str,
    active: bool,
    badge: Option<&str>,
    live: bool,
    net: Option<&NetHandle>,
) -> bool {
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
                ui.label(
                    egui::RichText::new(icon)
                        .color(if active { t::ACCENT } else { t::TEXT_MUTED })
                        .size(13.0),
                );
                ui.label(egui::RichText::new(name).color(text_c).size(13.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if live {
                        let (r, _) =
                            ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                        ui.painter().circle_filled(r.center(), 3.0, t::SUCCESS);
                    } else if let Some(b) = badge {
                        ui.label(
                            egui::RichText::new(b)
                                .font(egui::FontId::proportional(10.0))
                                .color(t::TEXT_DISABLED),
                        );
                    }
                });
            });
        })
        .response
        .interact(egui::Sense::click());

    // Right-click context menu — offer deletion for non-default channels.
    if let Some(net) = net
        && name != "general"
        && name != "voice"
    {
        resp.context_menu(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Delete channel")
                            .font(egui::FontId::proportional(12.0))
                            .color(t::ERROR),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .frame(false),
                )
                .clicked()
            {
                let net = net.clone();
                let ch_id = name.to_string();
                tokio::spawn(async move {
                    net.send(NetCommand::ChannelDelete { channel_id: ch_id })
                        .await;
                });
                ui.close();
            }
        });
    }

    resp.clicked()
}

pub(super) fn member_row(ui: &mut egui::Ui, nick: &str) {
    Frame::NONE
        .inner_margin(egui::Margin {
            left: 28,
            right: 8,
            top: 4,
            bottom: 4,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let init = nick
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                let (r, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(r.center(), 8.0, nick_color(nick));
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &init,
                    egui::FontId::proportional(8.0),
                    egui::Color32::from_rgb(15, 15, 15),
                );
                ui.label(
                    egui::RichText::new(nick)
                        .font(egui::FontId::proportional(11.5))
                        .color(t::TEXT_MUTED),
                );
            });
        });
}

fn nick_color(nick: &str) -> egui::Color32 {
    let colors = [
        t::ACCENT,
        t::SUCCESS,
        t::WARNING,
        t::INFO,
        egui::Color32::from_rgb(200, 100, 180),
    ];
    colors[nick.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len()]
}
