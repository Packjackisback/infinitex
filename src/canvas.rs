use eframe::egui;
use crate::models::{DrawObject, StrokePoint};

pub fn smooth_stroke(points: &[StrokePoint]) -> Vec<StrokePoint> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut smoothed = Vec::new();
    smoothed.push(points[0].clone());

    for i in 0..points.len() - 1 {
        let p0 = if i == 0 { points[0].pos } else { points[i - 1].pos };
        let p1 = points[i].pos;
        let p2 = points[i + 1].pos;
        let p3 = if i + 2 < points.len() { points[i + 2].pos } else { points[i + 1].pos };

        let segments = 5;
        for t in 0..segments {
            let t = t as f32 / segments as f32;
            let t2 = t * t;
            let t3 = t2 * t;

            let x = 0.5 * (
                (2.0 * p1[0]) +
                (-p0[0] + p2[0]) * t +
                (2.0 * p0[0] - 5.0 * p1[0] + 4.0 * p2[0] - p3[0]) * t2 +
                (-p0[0] + 3.0 * p1[0] - 3.0 * p2[0] + p3[0]) * t3
            );
            let y = 0.5 * (
                (2.0 * p1[1]) +
                (-p0[1] + p2[1]) * t +
                (2.0 * p0[1] - 5.0 * p1[1] + 4.0 * p2[1] - p3[1]) * t2 +
                (-p0[1] + 3.0 * p1[1] - 3.0 * p2[1] + p3[1]) * t3
            );

            smoothed.push(StrokePoint { pos: [x, y] });
        }
    }

    smoothed.push(points[points.len() - 1].clone());
    smoothed
}

pub fn screen_to_canvas(screen_pos: egui::Pos2, canvas_offset: egui::Vec2, canvas_zoom: f32) -> [f32; 2] {
    let canvas_pos = (screen_pos.to_vec2() - canvas_offset) / canvas_zoom;
    [canvas_pos.x, canvas_pos.y]
}

pub fn canvas_to_screen(canvas_pos: [f32; 2], canvas_offset: egui::Vec2, canvas_zoom: f32) -> egui::Pos2 {
    let screen_vec = egui::Vec2::new(canvas_pos[0], canvas_pos[1]) * canvas_zoom + canvas_offset;
    egui::Pos2::new(screen_vec.x, screen_vec.y)
}

pub fn render_object(painter: &egui::Painter, obj: &DrawObject, canvas_offset: egui::Vec2, canvas_zoom: f32) {
    match obj {
        DrawObject::Stroke { points, color, width, .. } => {
            if points.len() < 2 {
                return;
            }
            let color = egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
            for i in 0..points.len() - 1 {
                let start = canvas_to_screen(points[i].pos, canvas_offset, canvas_zoom);
                let end = canvas_to_screen(points[i + 1].pos, canvas_offset, canvas_zoom);
                painter.line_segment(
                    [start, end],
                    egui::Stroke::new(*width * canvas_zoom, color),
                );
            }
        }
        DrawObject::Line { start, end, color, width, .. } => {
            let color = egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
            let screen_start = canvas_to_screen(*start, canvas_offset, canvas_zoom);
            let screen_end = canvas_to_screen(*end, canvas_offset, canvas_zoom);
            painter.line_segment(
                [screen_start, screen_end],
                egui::Stroke::new(*width * canvas_zoom, color),
            );
        }
        DrawObject::Circle { center, radius, color, width, filled, .. } => {
            let color = egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
            let screen_center = canvas_to_screen(*center, canvas_offset, canvas_zoom);
            let screen_radius = radius * canvas_zoom;
            if *filled {
                painter.circle_filled(screen_center, screen_radius, color);
            } else {
                painter.circle_stroke(
                    screen_center,
                    screen_radius,
                    egui::Stroke::new(*width * canvas_zoom, color),
                );
            }
        }
        DrawObject::Rectangle { min, max, color, width, filled, .. } => {
            let color = egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
            let screen_min = canvas_to_screen(*min, canvas_offset, canvas_zoom);
            let screen_max = canvas_to_screen(*max, canvas_offset, canvas_zoom);
            let rect = egui::Rect::from_two_pos(screen_min, screen_max);
            if *filled {
                painter.rect_filled(rect, 0.0, color);
            } else {
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(*width * canvas_zoom, color),
                );
            }
        }
        DrawObject::LatexFormula { .. } => {
            //hi future me don't delete this
        }
    }
}

pub fn find_object_at(objects: &[DrawObject], canvas_pos: [f32; 2]) -> Option<uuid::Uuid> {
    for obj in objects.iter().rev() {
        if obj.contains_point(canvas_pos) {
            return Some(obj.id());
        }
    }
    None
}
