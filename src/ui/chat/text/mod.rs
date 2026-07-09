mod input;
mod messages;

use eframe::egui::{self, Context, FontId, Frame, RichText};
use vnox_client::{identity::Identity, net::NetHandle};

use crate::state::UiState;
use crate::theme as t;

pub(super) use messages::message_row;

pub(crate) fn text_chat(
    ui: &mut egui::Ui,
    ctx: &Context,
    s: &mut UiState,
    net: &NetHandle,
    identity: &Identity,
    ch_id: &str,
    ch_name: &str,
) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        Frame::NONE
            .fill(t::BG_SURFACE)
            .inner_margin(egui::Margin::symmetric(16, 0))
            .show(ui, |ui| {
                ui.set_height(40.0);
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new("#").color(t::TEXT_MUTED).size(16.0));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(ch_name)
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

        if let Some(elapsed) = s.voice_elapsed() {
            voice_banner(ui, s, &elapsed);
        }

        let input_h = 52.0;
        messages::show(ui, s, net, identity, ch_id, ui.available_height() - input_h);
        input::chat_input_bar(ui, ctx, s, net, ch_id, identity);
    });
}

fn voice_banner(ui: &mut egui::Ui, s: &UiState, elapsed: &str) {
    Frame::NONE
        .fill(t::BG_ELEVATED)
        .inner_margin(egui::Margin::symmetric(16, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (dot, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover());
                ui.painter().circle_filled(dot.center(), 3.5, t::SUCCESS);
                ui.label(RichText::new("voice").color(t::SUCCESS).size(12.0));
                ui.label(RichText::new("·").color(t::TEXT_DISABLED).size(12.0));
                if let Some(ch) = s.active_voice_channel() {
                    ui.label(RichText::new(&ch.name).color(t::TEXT_SECONDARY).size(12.0));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(elapsed)
                            .font(FontId::monospace(11.0))
                            .color(t::TEXT_DISABLED),
                    );
                });
            });
        });
}
