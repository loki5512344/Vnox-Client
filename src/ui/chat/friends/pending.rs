use eframe::egui::{self, FontId, Frame, RichText};
use vnox_client::net::{NetCommand, NetHandle};

use crate::state::FriendState;
use crate::theme as t;

pub(super) fn pending_row(ui: &mut egui::Ui, f: &FriendState, net: &NetHandle) {
    Frame::NONE
        .fill(t::BG_SURFACE)
        .corner_radius(t::R_MD)
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let init = f
                    .nickname
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                t::avatar(ui, &init, t::WARNING, 28.0);
                ui.add_space(8.0);
                ui.label(
                    RichText::new(&f.nickname)
                        .font(FontId::proportional(13.0))
                        .color(t::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if t::icon_btn_drawn(
                        ui,
                        crate::theme::icons::check,
                        "accept",
                        Some(t::SUCCESS),
                        18.0,
                    )
                    .clicked()
                    {
                        let net = net.clone();
                        let uid = f.user_id.clone();
                        tokio::spawn(async move {
                            net.send(NetCommand::FriendAccept { user_id: uid }).await;
                        });
                    }
                    if t::icon_btn_drawn(
                        ui,
                        crate::theme::icons::close,
                        "decline",
                        Some(t::ERROR),
                        18.0,
                    )
                    .clicked()
                    {
                        let net = net.clone();
                        let uid = f.user_id.clone();
                        tokio::spawn(async move {
                            net.send(NetCommand::FriendDecline { user_id: uid }).await;
                        });
                    }
                });
            });
        });
}
