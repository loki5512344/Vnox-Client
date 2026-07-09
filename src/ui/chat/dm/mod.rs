mod messages;

use eframe::egui::{self, Context, FontId, Frame, RichText};
use vnox_client::{identity::Identity, net::NetHandle};

use crate::state::UiState;
use crate::theme as t;

pub(crate) fn dm_chat(
    ui: &mut egui::Ui,
    ctx: &Context,
    s: &mut UiState,
    identity: &Identity,
    net: &NetHandle,
    dm_id: &str,
) {
    let conv = s
        .dm_conversations
        .iter()
        .find(|c| c.dm_id == dm_id)
        .cloned();

    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        Frame::NONE
            .fill(t::BG_SURFACE)
            .inner_margin(egui::Margin::symmetric(16, 0))
            .show(ui, |ui| {
                ui.set_height(40.0);
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new("@").color(t::TEXT_MUTED).size(16.0));
                    ui.add_space(4.0);
                    let name = conv
                        .as_ref()
                        .map(|c| c.other_nickname.as_str())
                        .unwrap_or("?");
                    ui.label(
                        RichText::new(name)
                            .font(FontId::proportional(14.0))
                            .color(t::TEXT_PRIMARY)
                            .strong(),
                    );
                });
            });

        let r = ui.max_rect();
        ui.painter().hline(
            r.x_range(),
            r.top() + 40.0,
            egui::Stroke::new(1.0, t::BORDER_SUBTLE),
        );

        let search_h = 32.0;
        let input_h = 52.0;
        messages::search_bar(ui, s, net, dm_id, search_h);
        ui.painter().hline(
            r.x_range(),
            r.top() + 40.0 + search_h,
            egui::Stroke::new(1.0, t::BORDER_SUBTLE),
        );

        messages::show(
            ui,
            s,
            net,
            identity,
            dm_id,
            ui.available_height() - input_h - search_h,
        );
        messages::input_bar(ui, ctx, s, net, identity, dm_id);
    });
}
