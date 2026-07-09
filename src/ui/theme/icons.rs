//! Hand-drawn vector icons rendered via `egui::Painter`.
//!
//! Each icon is a function `draw_<name>(painter, rect, color)` that fills
//! the given `egui::Rect` with a monochrome glyph of the given `Color32`.
//!
//! All glyphs are designed on a 24×24 grid normalized to the rect.
//! Replace emoji usage throughout the UI with these for a cleaner, more
//! professional look that doesn't depend on system emoji fonts.

use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, Vec2};

/// Draw an icon centered in the given rect, scaled to fit.
pub fn draw_icon(
    ui: &mut egui::Ui,
    size: f32,
    draw_fn: impl FnOnce(&egui::Painter, Rect, Color32),
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let s = rect.size().min_elem();
        let center = rect.center();
        let half = s / 2.0;
        let scaled =
            Rect::from_min_size(Pos2::new(center.x - half, center.y - half), Vec2::splat(s));
        draw_fn(painter, scaled, super::TEXT_MUTED);
    }
    resp
}

/// Helper to convert a 0..24 coordinate into the scaled rect's coordinate.
fn p(rect: Rect, x: f32, y: f32) -> Pos2 {
    let sx = rect.width() / 24.0;
    let sy = rect.height() / 24.0;
    Pos2::new(rect.min.x + x * sx, rect.min.y + y * sy)
}

/// Microphone icon.
pub fn microphone(painter: &egui::Painter, rect: Rect, color: Color32) {
    // Mic body (rounded capsule).
    let body = Rect::from_center_size(
        p(rect, 12.0, 9.0),
        Vec2::new(rect.width() * 0.18, rect.height() * 0.42),
    );
    painter.rect_filled(body, egui::CornerRadius::same(4), color);
    // Arc below (the cradle).
    let arc_center = p(rect, 12.0, 14.0);
    let radius = rect.width() * 0.22;
    painter.add(Shape::circle_stroke(
        arc_center,
        radius,
        Stroke::new(rect.width() * 0.05, color),
    ));
    // Stand.
    painter.line_segment(
        [p(rect, 12.0, 16.0), p(rect, 12.0, 20.0)],
        Stroke::new(rect.width() * 0.06, color),
    );
    painter.line_segment(
        [p(rect, 9.0, 20.0), p(rect, 15.0, 20.0)],
        Stroke::new(rect.width() * 0.06, color),
    );
}

/// Microphone-slash icon (mic with a diagonal line through it).
pub fn microphone_off(painter: &egui::Painter, rect: Rect, color: Color32) {
    microphone(painter, rect, Color32::from_rgb(80, 80, 80));
    painter.line_segment(
        [p(rect, 5.0, 5.0), p(rect, 19.0, 19.0)],
        Stroke::new(rect.width() * 0.08, color),
    );
}

/// Headphones icon (for deafen).
pub fn headphones(painter: &egui::Painter, rect: Rect, color: Color32) {
    // Top arc.
    painter.add(Shape::circle_stroke(
        p(rect, 12.0, 13.0),
        rect.width() * 0.30,
        Stroke::new(rect.width() * 0.06, color),
    ));
    // Cover bottom half of the arc so only top shows.
    painter.rect_filled(
        Rect::from_min_max(p(rect, 4.0, 13.0), p(rect, 20.0, 21.0)),
        egui::CornerRadius::same(0),
        super::BG_ELEVATED,
    );
    // Left ear cup.
    painter.rect_filled(
        Rect::from_center_size(
            p(rect, 5.5, 13.0),
            Vec2::new(rect.width() * 0.18, rect.height() * 0.28),
        ),
        egui::CornerRadius::same(2),
        color,
    );
    // Right ear cup.
    painter.rect_filled(
        Rect::from_center_size(
            p(rect, 18.5, 13.0),
            Vec2::new(rect.width() * 0.18, rect.height() * 0.28),
        ),
        egui::CornerRadius::same(2),
        color,
    );
}

/// Headphones-off (with slash).
pub fn headphones_off(painter: &egui::Painter, rect: Rect, color: Color32) {
    headphones(painter, rect, Color32::from_rgb(80, 80, 80));
    painter.line_segment(
        [p(rect, 5.0, 5.0), p(rect, 19.0, 19.0)],
        Stroke::new(rect.width() * 0.08, color),
    );
}

/// Gear / settings icon (cog).
pub fn gear(painter: &egui::Painter, rect: Rect, color: Color32) {
    let center = p(rect, 12.0, 12.0);
    let outer_r = rect.width() * 0.40;
    let inner_r = rect.width() * 0.28;
    let hole_r = rect.width() * 0.12;
    // Draw 8 teeth as small rounded rects around the circle.
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let tooth_center = Pos2::new(
            center.x + angle.cos() * (outer_r - rect.width() * 0.04),
            center.y + angle.sin() * (outer_r - rect.width() * 0.04),
        );
        painter.rect_filled(
            Rect::from_center_size(tooth_center, Vec2::splat(rect.width() * 0.10)),
            egui::CornerRadius::same(1),
            color,
        );
    }
    // Outer ring.
    painter.add(Shape::circle_stroke(
        center,
        inner_r,
        Stroke::new(rect.width() * 0.07, color),
    ));
    // Center hole.
    painter.circle_filled(center, hole_r, color);
}

/// Plus icon.
pub fn plus(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.08;
    let h = rect.height() * 0.6;
    let v = rect.width() * 0.6;
    let hv = rect.height() * 0.08;
    painter.rect_filled(
        Rect::from_center_size(rect.center(), Vec2::new(w, h)),
        egui::CornerRadius::same(1),
        color,
    );
    painter.rect_filled(
        Rect::from_center_size(rect.center(), Vec2::new(v, hv)),
        egui::CornerRadius::same(1),
        color,
    );
}

/// X / close icon.
pub fn close(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(rect.width() * 0.08, color);
    painter.line_segment([p(rect, 6.0, 6.0), p(rect, 18.0, 18.0)], stroke);
    painter.line_segment([p(rect, 18.0, 6.0), p(rect, 6.0, 18.0)], stroke);
}

/// Trash icon.
pub fn trash(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Lid.
    painter.line_segment(
        [p(rect, 5.0, 7.0), p(rect, 19.0, 7.0)],
        Stroke::new(w, color),
    );
    // Handle.
    painter.line_segment(
        [p(rect, 10.0, 5.0), p(rect, 14.0, 5.0)],
        Stroke::new(w, color),
    );
    // Body.
    painter.line_segment(
        [p(rect, 7.0, 7.0), p(rect, 8.5, 19.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 17.0, 7.0), p(rect, 15.5, 19.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 8.5, 19.0), p(rect, 15.5, 19.0)],
        Stroke::new(w, color),
    );
    // Ribs.
    painter.line_segment(
        [p(rect, 11.0, 9.0), p(rect, 11.0, 17.0)],
        Stroke::new(w * 0.7, color),
    );
    painter.line_segment(
        [p(rect, 13.0, 9.0), p(rect, 13.0, 17.0)],
        Stroke::new(w * 0.7, color),
    );
}

/// Lock icon (closed padlock).
pub fn lock(painter: &egui::Painter, rect: Rect, color: Color32) {
    // Shackle (arc).
    painter.add(Shape::circle_stroke(
        p(rect, 12.0, 9.0),
        rect.width() * 0.18,
        Stroke::new(rect.width() * 0.07, color),
    ));
    // Cover the bottom of the shackle arc.
    painter.rect_filled(
        Rect::from_min_max(p(rect, 9.0, 9.0), p(rect, 15.0, 13.0)),
        egui::CornerRadius::same(0),
        super::BG_ELEVATED,
    );
    // Body.
    painter.rect_filled(
        Rect::from_center_size(
            p(rect, 12.0, 15.0),
            Vec2::new(rect.width() * 0.42, rect.height() * 0.28),
        ),
        egui::CornerRadius::same(2),
        color,
    );
}

/// Unlock icon (open padlock).
pub fn unlock(painter: &egui::Painter, rect: Rect, color: Color32) {
    // Shackle (open — arc shifted right).
    painter.add(Shape::circle_stroke(
        p(rect, 14.0, 9.0),
        rect.width() * 0.18,
        Stroke::new(rect.width() * 0.07, color),
    ));
    // Cover bottom of shackle.
    painter.rect_filled(
        Rect::from_min_max(p(rect, 11.0, 9.0), p(rect, 17.0, 13.0)),
        egui::CornerRadius::same(0),
        super::BG_ELEVATED,
    );
    // Body.
    painter.rect_filled(
        Rect::from_center_size(
            p(rect, 10.0, 15.0),
            Vec2::new(rect.width() * 0.42, rect.height() * 0.28),
        ),
        egui::CornerRadius::same(2),
        color,
    );
}

/// Refresh / rotate icon (circular arrow).
pub fn refresh(painter: &egui::Painter, rect: Rect, color: Color32) {
    let center = p(rect, 12.0, 12.0);
    let r = rect.width() * 0.32;
    let stroke = Stroke::new(rect.width() * 0.07, color);
    // Arc (3/4 circle).
    let steps = 24;
    let start = -std::f32::consts::FRAC_PI_2;
    let end = start + std::f32::consts::TAU * 0.75;
    let mut prev: Option<Pos2> = None;
    for i in 0..=steps {
        let t = start + (end - start) * (i as f32 / steps as f32);
        let pt = Pos2::new(center.x + t.cos() * r, center.y + t.sin() * r);
        if let Some(p0) = prev {
            painter.line_segment([p0, pt], stroke);
        }
        prev = Some(pt);
    }
    // Arrowhead at the end.
    let arrow_end = prev.unwrap();
    let perp = end + std::f32::consts::FRAC_PI_2;
    let ah1 = Pos2::new(
        arrow_end.x + perp.cos() * rect.width() * 0.10,
        arrow_end.y + perp.sin() * rect.height() * 0.10,
    );
    let ah2 = Pos2::new(
        arrow_end.x - perp.cos() * rect.width() * 0.10,
        arrow_end.y - perp.sin() * rect.height() * 0.10,
    );
    painter.line_segment([arrow_end, ah1], stroke);
    painter.line_segment([arrow_end, ah2], stroke);
}

/// Clipboard / list icon (for audit log).
pub fn clipboard(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Outer rect.
    painter.rect_stroke(
        Rect::from_min_max(p(rect, 5.0, 4.0), p(rect, 19.0, 20.0)),
        egui::CornerRadius::same(2),
        Stroke::new(w, color),
        egui::StrokeKind::Middle,
    );
    // Clip at top.
    painter.rect_filled(
        Rect::from_min_max(p(rect, 9.0, 3.0), p(rect, 15.0, 6.0)),
        egui::CornerRadius::same(1),
        color,
    );
    // Lines.
    painter.line_segment(
        [p(rect, 8.0, 10.0), p(rect, 16.0, 10.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 8.0, 13.0), p(rect, 16.0, 13.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 8.0, 16.0), p(rect, 14.0, 16.0)],
        Stroke::new(w, color),
    );
}

/// People / members icon (two overlapping circles + shoulders).
pub fn people(painter: &egui::Painter, rect: Rect, color: Color32) {
    let r1 = rect.width() * 0.16;
    let r2 = rect.width() * 0.13;
    // Two heads.
    painter.circle_filled(p(rect, 9.0, 8.0), r1, color);
    painter.circle_filled(p(rect, 16.0, 9.0), r2, color);
    // Shoulders.
    painter.rect_filled(
        Rect::from_min_max(p(rect, 4.0, 13.0), p(rect, 14.0, 21.0)),
        egui::CornerRadius::same(4),
        color,
    );
    painter.rect_filled(
        Rect::from_min_max(p(rect, 12.0, 14.0), p(rect, 20.0, 21.0)),
        egui::CornerRadius::same(4),
        color,
    );
}

/// Crown icon (for owner).
pub fn crown(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Base bar.
    painter.line_segment(
        [p(rect, 5.0, 18.0), p(rect, 19.0, 18.0)],
        Stroke::new(w * 1.5, color),
    );
    // Three peaks.
    painter.line_segment(
        [p(rect, 5.0, 18.0), p(rect, 7.0, 8.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 7.0, 8.0), p(rect, 12.0, 14.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 12.0, 14.0), p(rect, 17.0, 8.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 17.0, 8.0), p(rect, 19.0, 18.0)],
        Stroke::new(w, color),
    );
    // Gems.
    painter.circle_filled(p(rect, 7.0, 7.0), rect.width() * 0.04, color);
    painter.circle_filled(p(rect, 17.0, 7.0), rect.width() * 0.04, color);
    painter.circle_filled(p(rect, 12.0, 13.0), rect.width() * 0.04, color);
}

/// Boot icon (for kick).
pub fn boot(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.07;
    // Boot outline.
    painter.line_segment(
        [p(rect, 8.0, 5.0), p(rect, 8.0, 14.0)],
        Stroke::new(w * 1.5, color),
    );
    painter.line_segment(
        [p(rect, 8.0, 14.0), p(rect, 8.0, 17.0)],
        Stroke::new(w * 1.5, color),
    );
    painter.line_segment(
        [p(rect, 7.0, 17.0), p(rect, 18.0, 17.0)],
        Stroke::new(w * 1.5, color),
    );
    painter.line_segment(
        [p(rect, 18.0, 17.0), p(rect, 18.0, 14.0)],
        Stroke::new(w * 1.5, color),
    );
    painter.line_segment(
        [p(rect, 18.0, 14.0), p(rect, 12.0, 14.0)],
        Stroke::new(w * 1.5, color),
    );
    painter.line_segment(
        [p(rect, 12.0, 14.0), p(rect, 12.0, 5.0)],
        Stroke::new(w * 1.5, color),
    );
    // Top opening.
    painter.line_segment(
        [p(rect, 8.0, 5.0), p(rect, 12.0, 5.0)],
        Stroke::new(w, color),
    );
}

/// Speaker (sound waves) icon — used for voice activity.
pub fn speaker(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Speaker box.
    painter.line_segment(
        [p(rect, 4.0, 10.0), p(rect, 8.0, 10.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 4.0, 14.0), p(rect, 8.0, 14.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 4.0, 10.0), p(rect, 4.0, 14.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 8.0, 10.0), p(rect, 12.0, 6.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 8.0, 14.0), p(rect, 12.0, 18.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 12.0, 6.0), p(rect, 12.0, 18.0)],
        Stroke::new(w, color),
    );
    // Sound waves.
    painter.add(Shape::circle_stroke(
        p(rect, 15.0, 12.0),
        rect.width() * 0.10,
        Stroke::new(w * 0.8, color),
    ));
    painter.add(Shape::circle_stroke(
        p(rect, 17.0, 12.0),
        rect.width() * 0.16,
        Stroke::new(w * 0.6, color),
    ));
}

/// Speaker-muted icon (speaker with X).
pub fn speaker_off(painter: &egui::Painter, rect: Rect, color: Color32) {
    speaker(painter, rect, Color32::from_rgb(80, 80, 80));
    let w = rect.width() * 0.07;
    painter.line_segment(
        [p(rect, 14.0, 8.0), p(rect, 20.0, 16.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 20.0, 8.0), p(rect, 14.0, 16.0)],
        Stroke::new(w, color),
    );
}

/// Envelope icon (for DM).
pub fn envelope(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Outer rect.
    painter.rect_stroke(
        Rect::from_min_max(p(rect, 4.0, 6.0), p(rect, 20.0, 18.0)),
        egui::CornerRadius::same(1),
        Stroke::new(w, color),
        egui::StrokeKind::Middle,
    );
    // Flap.
    painter.line_segment(
        [p(rect, 4.0, 6.0), p(rect, 12.0, 13.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 20.0, 6.0), p(rect, 12.0, 13.0)],
        Stroke::new(w, color),
    );
}

/// Pencil / edit icon.
pub fn pencil(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Diagonal pencil body.
    painter.line_segment(
        [p(rect, 5.0, 19.0), p(rect, 16.0, 8.0)],
        Stroke::new(w * 1.5, color),
    );
    // Tip.
    painter.line_segment(
        [p(rect, 4.0, 20.0), p(rect, 6.0, 18.0)],
        Stroke::new(w * 1.5, color),
    );
    // Eraser end.
    painter.line_segment(
        [p(rect, 15.0, 7.0), p(rect, 18.0, 10.0)],
        Stroke::new(w * 1.5, color),
    );
    painter.line_segment(
        [p(rect, 16.0, 6.0), p(rect, 19.0, 9.0)],
        Stroke::new(w * 1.5, color),
    );
}

/// Reply arrow icon.
pub fn reply(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.07;
    // Curved arrow (drawn as two segments).
    painter.line_segment(
        [p(rect, 16.0, 8.0), p(rect, 8.0, 8.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 8.0, 8.0), p(rect, 8.0, 16.0)],
        Stroke::new(w, color),
    );
    // Arrowhead.
    painter.line_segment(
        [p(rect, 5.0, 11.0), p(rect, 8.0, 8.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 8.0, 8.0), p(rect, 11.0, 11.0)],
        Stroke::new(w, color),
    );
}

/// Copy icon (two overlapping squares).
pub fn copy(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Back square.
    painter.rect_stroke(
        Rect::from_min_max(p(rect, 8.0, 4.0), p(rect, 20.0, 16.0)),
        egui::CornerRadius::same(1),
        Stroke::new(w, color),
        egui::StrokeKind::Middle,
    );
    // Front square.
    painter.rect_stroke(
        Rect::from_min_max(p(rect, 4.0, 8.0), p(rect, 16.0, 20.0)),
        egui::CornerRadius::same(1),
        Stroke::new(w, color),
        egui::StrokeKind::Middle,
    );
}

/// User-plus icon (for "add friend").
pub fn user_plus(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Head.
    painter.circle_filled(p(rect, 9.0, 8.0), rect.width() * 0.14, color);
    // Shoulders.
    painter.rect_filled(
        Rect::from_min_max(p(rect, 4.0, 13.0), p(rect, 14.0, 21.0)),
        egui::CornerRadius::same(4),
        color,
    );
    // Plus.
    painter.line_segment(
        [p(rect, 16.0, 11.0), p(rect, 20.0, 11.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 18.0, 9.0), p(rect, 18.0, 13.0)],
        Stroke::new(w, color),
    );
}

/// Home icon.
pub fn home(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Roof.
    painter.line_segment(
        [p(rect, 4.0, 12.0), p(rect, 12.0, 5.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 12.0, 5.0), p(rect, 20.0, 12.0)],
        Stroke::new(w, color),
    );
    // Body.
    painter.line_segment(
        [p(rect, 6.0, 12.0), p(rect, 6.0, 19.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 18.0, 12.0), p(rect, 18.0, 19.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 6.0, 19.0), p(rect, 18.0, 19.0)],
        Stroke::new(w, color),
    );
    // Door.
    painter.line_segment(
        [p(rect, 10.0, 19.0), p(rect, 10.0, 14.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 14.0, 19.0), p(rect, 14.0, 14.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 10.0, 14.0), p(rect, 14.0, 14.0)],
        Stroke::new(w, color),
    );
}

/// Link icon (for invites).
pub fn link(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.07;
    // Left chain.
    painter.rect_stroke(
        Rect::from_min_max(p(rect, 3.0, 8.0), p(rect, 12.0, 16.0)),
        egui::CornerRadius::same(4),
        Stroke::new(w, color),
        egui::StrokeKind::Middle,
    );
    // Right chain.
    painter.rect_stroke(
        Rect::from_min_max(p(rect, 12.0, 8.0), p(rect, 21.0, 16.0)),
        egui::CornerRadius::same(4),
        Stroke::new(w, color),
        egui::StrokeKind::Middle,
    );
    // Connector.
    painter.line_segment(
        [p(rect, 9.0, 12.0), p(rect, 15.0, 12.0)],
        Stroke::new(w, color),
    );
}

/// Tag icon (for roles).
pub fn tag(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.06;
    // Tag body.
    let pts = [
        p(rect, 4.0, 12.0),
        p(rect, 8.0, 6.0),
        p(rect, 19.0, 6.0),
        p(rect, 19.0, 18.0),
        p(rect, 8.0, 18.0),
    ];
    for i in 0..pts.len() - 1 {
        painter.line_segment([pts[i], pts[i + 1]], Stroke::new(w, color));
    }
    painter.line_segment([pts[pts.len() - 1], pts[0]], Stroke::new(w, color));
    // Hole.
    painter.circle_filled(p(rect, 15.0, 12.0), rect.width() * 0.04, color);
}

/// Check / tick icon.
pub fn check(painter: &egui::Painter, rect: Rect, color: Color32) {
    painter.line_segment(
        [p(rect, 5.0, 12.0), p(rect, 10.0, 17.0)],
        Stroke::new(rect.width() * 0.10, color),
    );
    painter.line_segment(
        [p(rect, 10.0, 17.0), p(rect, 19.0, 6.0)],
        Stroke::new(rect.width() * 0.10, color),
    );
}

/// Hash icon (for text channel).
pub fn hash(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.07;
    painter.line_segment(
        [p(rect, 8.0, 4.0), p(rect, 6.0, 20.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 16.0, 4.0), p(rect, 14.0, 20.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 4.0, 9.0), p(rect, 18.0, 9.0)],
        Stroke::new(w, color),
    );
    painter.line_segment(
        [p(rect, 3.0, 15.0), p(rect, 17.0, 15.0)],
        Stroke::new(w, color),
    );
}

/// Waveform icon (for voice channel).
pub fn waveform(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.07;
    let cy = rect.center().y;
    let h = rect.height();
    let bars = [
        (6.0_f32, 0.25_f32),
        (9.0, 0.55),
        (12.0, 0.85),
        (15.0, 0.55),
        (18.0, 0.25),
    ];
    for (x, frac) in bars {
        let bh = h * frac * 0.7;
        painter.line_segment(
            [
                Pos2::new(rect.min.x + x * rect.width() / 24.0, cy - bh / 2.0),
                Pos2::new(rect.min.x + x * rect.width() / 24.0, cy + bh / 2.0),
            ],
            Stroke::new(w, color),
        );
    }
}
