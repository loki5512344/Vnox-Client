use crate::state::UiState;
use crate::theme as t;
use crate::ui::settings::widgets::{
    badge, device_meta_row, group_label, page_header, s_row, slider_row, small_select,
    small_select_labels, toggle,
};
use eframe::egui::{self, FontId, RichText, Stroke, Vec2};
use vnox_client::audio::devices::DeviceLabel;
fn selected_meta(index: usize, metas: &[DeviceLabel]) -> Option<&DeviceLabel> {
    if index == 0 {
        return None;
    }
    metas.get(index.saturating_sub(1))
}

pub fn voice(ui: &mut egui::Ui, s: &mut UiState) {
    page_header(ui, "voice", "input device, encoding, push-to-talk");
    group_label(ui, "input device");
    s_row(ui, "microphone", None, |ui| {
        small_select_labels(ui, "mic_dev", &mut s.mic_device, &s.input_device_labels);
    });
    device_meta_row(ui, selected_meta(s.mic_device, &s.input_device_metas));
    s_row(ui, "input volume", None, |ui| {
        slider_row(ui, &mut s.input_volume, 0.0, 100.0, |v| {
            format!("{}%", v as u32)
        });
    });
    s_row(ui, "noise suppression", Some("rnnoise · cpu ~2%"), |ui| {
        toggle(ui, &mut s.noise_suppress);
    });
    s_row(ui, "echo cancellation", None, |ui| {
        toggle(ui, &mut s.echo_cancel);
    });
    ui.add_space(8.0);
    group_label(ui, "activation");
    s_row(ui, "mode", None, |ui| {
        small_select(
            ui,
            "vad_mode",
            &mut s.vad_mode,
            &["push-to-talk", "voice activity", "always on"],
        );
    });
    if s.vad_mode == 1 {
        s_row(
            ui,
            "vad threshold",
            Some("voice activity sensitivity"),
            |ui| {
                slider_row(ui, &mut s.vad_threshold, 0.0, 100.0, |v| {
                    format!("{}%", v as u32)
                });
            },
        );
    }
    ui.add_space(8.0);
    group_label(ui, "codec");
    s_row(ui, "encoder", Some("opus · low latency mode"), |ui| {
        badge(ui, "opus", t::ACCENT_SOFT);
    });
    s_row(ui, "bitrate", Some("opus 8–128 kbps"), |ui| {
        ui.horizontal(|ui| {
            slider_row(ui, &mut s.voice_bitrate, 8.0, 128.0, |v| {
                format!("{}k", v as u32)
            });
            let presets = [32, 64, 96, 128usize];
            for &p in &presets {
                let pf = p as f32;
                let selected = (s.voice_bitrate - pf).abs() < 1.0;
                let resp = ui.add_sized(
                    Vec2::new(28.0, 20.0),
                    egui::Button::new(
                        RichText::new(format!("{}", p))
                            .font(FontId::monospace(8.0))
                            .color(if selected { t::ACCENT } else { t::TEXT_MUTED }),
                    )
                    .fill(if selected {
                        egui::Color32::from_rgba_premultiplied(
                            t::ACCENT.r(),
                            t::ACCENT.g(),
                            t::ACCENT.b(),
                            25,
                        )
                    } else {
                        egui::Color32::from_rgb(23, 23, 23)
                    })
                    .stroke(Stroke::new(1.0, t::BORDER_SUBTLE))
                    .corner_radius(3u8),
                );
                if resp.clicked() {
                    s.voice_bitrate = pf;
                }
            }
        });
        let effective = (s.voice_bitrate as u32).clamp(8, 128);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("effective: {} kbps", effective))
                    .font(FontId::monospace(9.0))
                    .color(t::TEXT_DEAD),
            );
        });
    });
    s_row(
        ui,
        "packet interval",
        Some("lower = less latency, more cpu"),
        |ui| {
            small_select(
                ui,
                "pkt_interval",
                &mut s.packet_interval,
                &["10ms", "20ms", "40ms"],
            );
        },
    );
}

pub fn output(ui: &mut egui::Ui, s: &mut UiState) {
    page_header(ui, "audio output", "output device · volume · jitter buffer");
    group_label(ui, "output device");
    s_row(ui, "device", None, |ui| {
        small_select_labels(ui, "out_dev", &mut s.output_device, &s.output_device_labels);
    });
    device_meta_row(ui, selected_meta(s.output_device, &s.output_device_metas));
    s_row(ui, "output volume", None, |ui| {
        slider_row(ui, &mut s.output_volume, 0.0, 100.0, |v| {
            format!("{}%", v as u32)
        });
    });
    ui.add_space(8.0);
    group_label(ui, "jitter buffer");
    s_row(
        ui,
        "buffer size",
        Some("lower = less delay, more glitches"),
        |ui| {
            small_select(ui, "jitter", &mut s.jitter_size, &["20ms", "40ms", "80ms"]);
        },
    );
    s_row(
        ui,
        "adaptive buffer",
        Some("auto-adjust on packet loss"),
        |ui| {
            toggle(ui, &mut s.adaptive_buffer);
        },
    );
}
