use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Tool {
    Brush,
    Line,
    Circle,
    Square,
    Eraser,
    Select,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    None,
    Selecting,
    Moving,
    Scaling,
    Rotating,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Rotate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokePoint {
    pub pos: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrawObject {
    Stroke {
        id: Uuid,
        points: Vec<StrokePoint>,
        color: [u8; 4],
        width: f32,
    },
    Line {
        id: Uuid,
        start: [f32; 2],
        end: [f32; 2],
        color: [u8; 4],
        width: f32,
    },
    Circle {
        id: Uuid,
        center: [f32; 2],
        radius: f32,
        color: [u8; 4],
        width: f32,
        filled: bool,
    },
    Rectangle {
        id: Uuid,
        min: [f32; 2],
        max: [f32; 2],
        color: [u8; 4],
        width: f32,
        filled: bool,
    },
    LatexFormula {
        id: Uuid,
        pos: [f32; 2],
        formula: String,
        color: [u8; 4],
        #[serde(skip)]
        cached_size: Option<[f32; 2]>,
    },
}

impl DrawObject {
    pub fn id(&self) -> Uuid {
        match self {
            DrawObject::Stroke { id, .. } => *id,
            DrawObject::Line { id, .. } => *id,
            DrawObject::Circle { id, .. } => *id,
            DrawObject::Rectangle { id, .. } => *id,
            DrawObject::LatexFormula { id, .. } => *id,
        }
    }

    pub fn bounds(&self) -> ([f32; 2], [f32; 2]) {
        match self {
            DrawObject::Stroke { points, width, .. } => {
                if points.is_empty() {
                    return ([0.0, 0.0], [0.0, 0.0]);
                }
                let mut min_x = points[0].pos[0];
                let mut min_y = points[0].pos[1];
                let mut max_x = points[0].pos[0];
                let mut max_y = points[0].pos[1];
                for p in points {
                    min_x = min_x.min(p.pos[0]);
                    min_y = min_y.min(p.pos[1]);
                    max_x = max_x.max(p.pos[0]);
                    max_y = max_y.max(p.pos[1]);
                }
                let half_width = width / 2.0;
                ([min_x - half_width, min_y - half_width], [max_x + half_width, max_y + half_width])
            }
            DrawObject::Line { start, end, width, .. } => {
                let half_width = width / 2.0;
                let min_x = start[0].min(end[0]) - half_width;
                let min_y = start[1].min(end[1]) - half_width;
                let max_x = start[0].max(end[0]) + half_width;
                let max_y = start[1].max(end[1]) + half_width;
                ([min_x, min_y], [max_x, max_y])
            }
            DrawObject::Circle { center, radius, width, .. } => {
                let r = radius + width / 2.0;
                ([center[0] - r, center[1] - r], [center[0] + r, center[1] + r])
            }
            DrawObject::Rectangle { min, max, width, .. } => {
                let half_width = width / 2.0;
                ([min[0] - half_width, min[1] - half_width], [max[0] + half_width, max[1] + half_width])
            }
            DrawObject::LatexFormula { pos, cached_size, .. } => {
                let size = cached_size.unwrap_or([100.0, 40.0]);
                (*pos, [pos[0] + size[0], pos[1] + size[1]])
            }
        }
    }

    pub fn contains_point(&self, point: [f32; 2]) -> bool {
        let (min, max) = self.bounds();
        point[0] >= min[0] && point[0] <= max[0] && point[1] >= min[1] && point[1] <= max[1]
    }
}

#[derive(Serialize, Deserialize)]
pub struct WhiteboardState {
    pub objects: Vec<DrawObject>,
}
