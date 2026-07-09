use crate::theme as t;
use eframe::egui::{self, Color32, FontId, Frame, Margin, RichText, Sense, Stroke, Vec2};

pub fn toggle(ui: &mut egui::Ui, value: &mut bool) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(30.0, 16.0), Sense::click());
    if resp.clicked() {
        *value = !*value;
    }
    let painter = ui.painter();
    let bg = if *value {
        Color32::from_rgba_premultiplied(255, 107, 53, 38)
    } else {
        t::BG_ELEVATED
    };
    let bdr = if *value {
        Color32::from_rgb(100, 50, 25)
    } else {
        t::BORDER_SUBTLE
    };
    painter.rect(
        rect,
        8u8,
        bg,
        Stroke::new(1.0, bdr),
        egui::StrokeKind::Inside,
    );
    let kx = if *value {
        rect.max.x - 7.0
    } else {
        rect.min.x + 7.0
    };
    let kc = if *value {
        t::ACCENT
    } else {
        Color32::from_rgb(45, 45, 45)
    };
    painter.circle_filled(egui::pos2(kx, rect.center().y), 5.0, kc);
}

pub fn small_select_labels(ui: &mut egui::Ui, id: &str, value: &mut usize, options: &[String]) {
    let fallback = "?".to_string();
    let selected = options.get(*value).unwrap_or(&fallback);
    egui::ComboBox::from_id_salt(id)
        .selected_text(
            RichText::new(selected.as_str())
                .font(FontId::monospace(10.0))
                .color(t::TEXT_MUTED),
        )
        .width(220.0)
        .show_ui(ui, |ui| {
            for (i, opt) in options.iter().enumerate() {
                ui.selectable_value(
                    value,
                    i,
                    RichText::new(opt.as_str())
                        .font(FontId::monospace(10.0))
                        .color(t::TEXT_SECONDARY),
                );
            }
        });
}

pub fn small_select(ui: &mut egui::Ui, id: &str, value: &mut usize, options: &[&str]) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(
            RichText::new(options.get(*value).copied().unwrap_or("?"))
                .font(FontId::monospace(10.0))
                .color(t::TEXT_MUTED),
        )
        .width(110.0)
        .show_ui(ui, |ui| {
            for (i, opt) in options.iter().enumerate() {
                ui.selectable_value(
                    value,
                    i,
                    RichText::new(*opt)
                        .font(FontId::monospace(10.0))
                        .color(t::TEXT_SECONDARY),
                );
            }
        });
}

pub fn slider_row(
    ui: &mut egui::Ui,
    val: &mut f32,
    min: f32,
    max: f32,
    fmt: impl Fn(f32) -> String,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().slider_width = 90.0;
        ui.add(
            egui::Slider::new(val, min..=max)
                .show_value(false)
                .trailing_fill(true),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(fmt(*val))
                .font(FontId::monospace(10.0))
                .color(t::TEXT_DIM),
        );
    });
}

pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    Frame::new()
        .fill(Color32::from_rgba_premultiplied(
            color.r(),
            color.g(),
            color.b(),
            25,
        ))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 50),
        ))
        .corner_radius(3u8)
        .inner_margin(Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .font(FontId::monospace(9.0))
                    .color(color),
            );
        });
}
