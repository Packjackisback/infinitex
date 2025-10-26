use crate::models::{DrawObject, SelectionHandle};
use uuid::Uuid;

pub fn get_selection_bounds(objects: &[DrawObject], selected_objects: &[Uuid]) -> Option<([f32; 2], [f32; 2])> {
    if selected_objects.is_empty() {
        return None;
    }

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for obj_id in selected_objects {
        if let Some(obj) = objects.iter().find(|o| o.id() == *obj_id) {
            let (obj_min, obj_max) = obj.bounds();
            min_x = min_x.min(obj_min[0]);
            min_y = min_y.min(obj_min[1]);
            max_x = max_x.max(obj_max[0]);
            max_y = max_y.max(obj_max[1]);
        }
    }

    Some(([min_x, min_y], [max_x, max_y]))
}

pub fn get_handle_at_pos(canvas_pos: [f32; 2], bounds: ([f32; 2], [f32; 2]), canvas_zoom: f32) -> Option<SelectionHandle> {
    let (min, max) = bounds;
    let handle_size = 10.0 / canvas_zoom;
    
    let mid_x = (min[0] + max[0]) / 2.0;
    let mid_y = (min[1] + max[1]) / 2.0;

    if (canvas_pos[0] - min[0]).abs() < handle_size && (canvas_pos[1] - min[1]).abs() < handle_size {
        return Some(SelectionHandle::TopLeft);
    }
    if (canvas_pos[0] - max[0]).abs() < handle_size && (canvas_pos[1] - min[1]).abs() < handle_size {
        return Some(SelectionHandle::TopRight);
    }
    if (canvas_pos[0] - min[0]).abs() < handle_size && (canvas_pos[1] - max[1]).abs() < handle_size {
        return Some(SelectionHandle::BottomLeft);
    }
    if (canvas_pos[0] - max[0]).abs() < handle_size && (canvas_pos[1] - max[1]).abs() < handle_size {
        return Some(SelectionHandle::BottomRight);
    }

    if (canvas_pos[0] - mid_x).abs() < handle_size && (canvas_pos[1] - min[1]).abs() < handle_size {
        return Some(SelectionHandle::Top);
    }
    if (canvas_pos[0] - mid_x).abs() < handle_size && (canvas_pos[1] - max[1]).abs() < handle_size {
        return Some(SelectionHandle::Bottom);
    }
    if (canvas_pos[0] - min[0]).abs() < handle_size && (canvas_pos[1] - mid_y).abs() < handle_size {
        return Some(SelectionHandle::Left);
    }
    if (canvas_pos[0] - max[0]).abs() < handle_size && (canvas_pos[1] - mid_y).abs() < handle_size {
        return Some(SelectionHandle::Right);
    }

    let rotate_y = min[1] - 30.0 / canvas_zoom;
    if (canvas_pos[0] - mid_x).abs() < handle_size && (canvas_pos[1] - rotate_y).abs() < handle_size {
        return Some(SelectionHandle::Rotate);
    }

    None
}

pub fn transform_objects(objects: &mut [DrawObject], selected_objects: &[Uuid], scale: [f32; 2], rotation: f32, translation: [f32; 2], center: [f32; 2]) {
    for obj_id in selected_objects {
        if let Some(obj) = objects.iter_mut().find(|o| o.id() == *obj_id) {
            match obj {
                DrawObject::Stroke { points, .. } => {
                    for point in points {
                        let mut x = point.pos[0] - center[0];
                        let mut y = point.pos[1] - center[1];
                        
                        if rotation != 0.0 {
                            let cos_r = rotation.cos();
                            let sin_r = rotation.sin();
                            let new_x = x * cos_r - y * sin_r;
                            let new_y = x * sin_r + y * cos_r;
                            x = new_x;
                            y = new_y;
                        }
                        
                        x *= scale[0];
                        y *= scale[1];
                        
                        point.pos[0] = x + center[0] + translation[0];
                        point.pos[1] = y + center[1] + translation[1];
                    }
                }
                DrawObject::Line { start, end, .. } => {
                    for pos in [start, end] {
                        let mut x = pos[0] - center[0];
                        let mut y = pos[1] - center[1];
                        
                        if rotation != 0.0 {
                            let cos_r = rotation.cos();
                            let sin_r = rotation.sin();
                            let new_x = x * cos_r - y * sin_r;
                            let new_y = x * sin_r + y * cos_r;
                            x = new_x;
                            y = new_y;
                        }
                        
                        x *= scale[0];
                        y *= scale[1];
                        
                        pos[0] = x + center[0] + translation[0];
                        pos[1] = y + center[1] + translation[1];
                    }
                }
                DrawObject::Circle { center: circle_center, radius, .. } => {
                    let mut x = circle_center[0] - center[0];
                    let mut y = circle_center[1] - center[1];
                    
                    if rotation != 0.0 {
                        let cos_r = rotation.cos();
                        let sin_r = rotation.sin();
                        let new_x = x * cos_r - y * sin_r;
                        let new_y = x * sin_r + y * cos_r;
                        x = new_x;
                        y = new_y;
                    }
                    
                    x *= scale[0];
                    y *= scale[1];
                    
                    circle_center[0] = x + center[0] + translation[0];
                    circle_center[1] = y + center[1] + translation[1];
                    *radius *= scale[0].max(scale[1]);
                }
                DrawObject::Rectangle { min, max, .. } => {
                    for pos in [min, max] {
                        let mut x = pos[0] - center[0];
                        let mut y = pos[1] - center[1];
                        
                        if rotation != 0.0 {
                            let cos_r = rotation.cos();
                            let sin_r = rotation.sin();
                            let new_x = x * cos_r - y * sin_r;
                            let new_y = x * sin_r + y * cos_r;
                            x = new_x;
                            y = new_y;
                        }
                        
                        x *= scale[0];
                        y *= scale[1];
                        
                        pos[0] = x + center[0] + translation[0];
                        pos[1] = y + center[1] + translation[1];
                    }
                }
                DrawObject::LatexFormula { pos, .. } => {
                    let mut x = pos[0] - center[0];
                    let mut y = pos[1] - center[1];
                    
                    if rotation != 0.0 {
                        let cos_r = rotation.cos();
                        let sin_r = rotation.sin();
                        let new_x = x * cos_r - y * sin_r;
                        let new_y = x * sin_r + y * cos_r;
                        x = new_x;
                        y = new_y;
                    }
                    
                    x *= scale[0];
                    y *= scale[1];
                    
                    pos[0] = x + center[0] + translation[0];
                    pos[1] = y + center[1] + translation[1];
                }
            }
        }
    }
}
