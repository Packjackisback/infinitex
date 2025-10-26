use eframe::egui;
use uuid::Uuid;

use crate::models::{Tool, DrawObject, StrokePoint, SelectionMode, SelectionHandle, WhiteboardState};
use crate::canvas;
use crate::latex::LatexRenderer;
use crate::selection;
use crate::file_io;

pub struct WhiteboardApp {
    pub objects: Vec<DrawObject>,
    pub undo_stack: Vec<Vec<DrawObject>>,
    pub current_tool: Tool,
    pub brush_size: f32,
    pub current_color: egui::Color32,
    
    pub canvas_offset: egui::Vec2,
    pub canvas_zoom: f32,
    pub background_color: egui::Color32,
    pub show_grid: bool,
    
    pub is_drawing: bool,
    pub current_stroke: Vec<StrokePoint>,
    pub draw_start_pos: Option<[f32; 2]>,
    
    pub selected_objects: Vec<Uuid>,
    pub selection_start: Option<[f32; 2]>,
    pub selection_rect: Option<([f32; 2], [f32; 2])>,
    pub selection_mode: SelectionMode,
    pub selection_drag_start: Option<[f32; 2]>,
    pub selection_original_bounds: Option<([f32; 2], [f32; 2])>,
    pub selection_handle: Option<SelectionHandle>,
    pub selection_saved_objects: Vec<DrawObject>,
    
    pub editing_text: Option<Uuid>,
    pub text_input: String,
    pub text_cursor_pos: usize,
    
    pub latex_renderer: LatexRenderer,
    
    pub show_latex_dialog: bool,
    pub latex_input: String,
    pub latex_placement_pos: [f32; 2],
    pub show_toolbar: bool,
    
    pub save_path: String,
    pub load_path: String,
    
    pub needs_repaint: bool,
}

impl Default for WhiteboardApp {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            undo_stack: Vec::new(),
            current_tool: Tool::Brush,
            brush_size: 2.0,
            current_color: egui::Color32::BLACK,
            canvas_offset: egui::Vec2::ZERO,
            canvas_zoom: 1.0,
            background_color: egui::Color32::WHITE,
            show_grid: true,
            is_drawing: false,
            current_stroke: Vec::new(),
            draw_start_pos: None,
            selected_objects: Vec::new(),
            selection_start: None,
            selection_rect: None,
            selection_mode: SelectionMode::None,
            selection_drag_start: None,
            selection_original_bounds: None,
            selection_handle: None,
            selection_saved_objects: Vec::new(),
            editing_text: None,
            text_input: String::new(),
            text_cursor_pos: 0,
            latex_renderer: LatexRenderer::new(),
            show_latex_dialog: false,
            latex_input: String::new(),
            latex_placement_pos: [100.0, 100.0],
            show_toolbar: true,
            save_path: "whiteboard.json".to_string(),
            load_path: "whiteboard.json".to_string(),
            needs_repaint: true,
        }
    }
}

impl WhiteboardApp {
    fn push_undo(&mut self) {
        if self.undo_stack.len() >= 50 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.objects.clone());
    }

    fn undo(&mut self) {
        if let Some(previous_state) = self.undo_stack.pop() {
            self.objects = previous_state;
            self.needs_repaint = true;
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if self.editing_text.is_none() {
                if i.key_pressed(egui::Key::B) {
                    self.current_tool = Tool::Brush;
                    self.needs_repaint = true;
                }
                if i.key_pressed(egui::Key::L) {
                    self.current_tool = Tool::Line;
                    self.needs_repaint = true;
                }
                if i.key_pressed(egui::Key::C) {
                    self.current_tool = Tool::Circle;
                    self.needs_repaint = true;
                }
                if i.key_pressed(egui::Key::R) {
                    self.current_tool = Tool::Square;
                    self.needs_repaint = true;
                }
                if i.key_pressed(egui::Key::E) {
                    self.current_tool = Tool::Eraser;
                    self.needs_repaint = true;
                }
                if i.key_pressed(egui::Key::S) && !i.modifiers.ctrl {
                    self.current_tool = Tool::Select;
                    self.needs_repaint = true;
                }
                if i.key_pressed(egui::Key::T) {
                    self.current_tool = Tool::Text;
                    self.needs_repaint = true;
                }
                if i.key_pressed(egui::Key::Z) && i.modifiers.ctrl {
                    self.undo();
                }
                if i.key_pressed(egui::Key::H) {
                    self.show_toolbar = !self.show_toolbar;
                    self.needs_repaint = true;
                }
            }
        });
    }

    fn render_toolbar(&mut self, ctx: &egui::Context) {
        if !self.show_toolbar {
            return;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Tool:");
                
                if ui.selectable_label(self.current_tool == Tool::Brush, "Brush (B)").clicked() {
                    self.current_tool = Tool::Brush;
                    self.needs_repaint = true;
                }
                if ui.selectable_label(self.current_tool == Tool::Line, "Line (L)").clicked() {
                    self.current_tool = Tool::Line;
                    self.needs_repaint = true;
                }
                if ui.selectable_label(self.current_tool == Tool::Circle, "Circle (C)").clicked() {
                    self.current_tool = Tool::Circle;
                    self.needs_repaint = true;
                }
                if ui.selectable_label(self.current_tool == Tool::Square, "Square (R)").clicked() {
                    self.current_tool = Tool::Square;
                    self.needs_repaint = true;
                }
                if ui.selectable_label(self.current_tool == Tool::Eraser, "Eraser (E)").clicked() {
                    self.current_tool = Tool::Eraser;
                    self.needs_repaint = true;
                }
                if ui.selectable_label(self.current_tool == Tool::Select, "Select (S)").clicked() {
                    self.current_tool = Tool::Select;
                    self.needs_repaint = true;
                }
                if ui.selectable_label(self.current_tool == Tool::Text, "Text (T)").clicked() {
                    self.current_tool = Tool::Text;
                    self.needs_repaint = true;
                }
            
                ui.separator();
                
                ui.label("Brush Size:");
                if ui.add(egui::Slider::new(&mut self.brush_size, 1.0..=20.0).text("px")).changed() {
                    self.needs_repaint = true;
                }
                
                ui.separator();
                
                ui.label("Color:");
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut self.current_color,
                    egui::color_picker::Alpha::Opaque,
                ).changed() {
                    self.needs_repaint = true;
                }
                
                ui.separator();
                
                if ui.button("Undo (Ctrl+Z)").clicked() {
                    self.undo();
                }
                
                ui.separator();
                
                if ui.button("Save").clicked() {
                    let state = WhiteboardState {
                        objects: self.objects.clone(),
                    };
                    if let Err(e) = file_io::save_to_file(&state, &self.save_path) {
                        eprintln!("Error saving: {}", e);
                    }
                }
                
                if ui.button("Load").clicked() {
                    if let Ok(state) = file_io::load_from_file(&self.load_path) {
                        self.objects = state.objects;
                        self.needs_repaint = true;
                    } else {
                        eprintln!("Error loading file");
                    }
                }
                
                ui.separator();
                
                ui.label("Background:");
                egui::ComboBox::from_id_salt("bg_preset")
                    .selected_text("Preset")
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(false, "White").clicked() {
                            self.background_color = egui::Color32::WHITE;
                            self.needs_repaint = true;
                        }
                        if ui.selectable_label(false, "Light Gray").clicked() {
                            self.background_color = egui::Color32::from_rgb(240, 240, 240);
                            self.needs_repaint = true;
                        }
                        if ui.selectable_label(false, "Dark Gray").clicked() {
                            self.background_color = egui::Color32::from_rgb(40, 40, 40);
                            self.needs_repaint = true;
                        }
                        if ui.selectable_label(false, "Black").clicked() {
                            self.background_color = egui::Color32::BLACK;
                            self.needs_repaint = true;
                        }
                        if ui.selectable_label(false, "Sepia").clicked() {
                            self.background_color = egui::Color32::from_rgb(255, 245, 230);
                            self.needs_repaint = true;
                        }
                        if ui.selectable_label(false, "Dark Blue").clicked() {
                            self.background_color = egui::Color32::from_rgb(20, 30, 40);
                            self.needs_repaint = true;
                        }
                    });
                
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut self.background_color,
                    egui::color_picker::Alpha::Opaque,
                ).changed() {
                    self.needs_repaint = true;
                }
                
                if ui.checkbox(&mut self.show_grid, "Grid").changed() {
                    self.needs_repaint = true;
                }
                
                ui.separator();
                
                ui.label(format!("Zoom: {:.0}%", self.canvas_zoom * 100.0));
                
                if ui.button("Clear All").clicked() {
                    self.push_undo();
                    self.objects.clear();
                    self.needs_repaint = true;
                }
                
                ui.separator();
                ui.label("Press H to toggle toolbar");
            });
        });
    }

    fn handle_text_editing(&mut self, ctx: &egui::Context) {
        if let Some(editing_id) = self.editing_text {
            ctx.input(|i| {
                for event in &i.events {
                    match event {
                        egui::Event::Text(text) => {
                            self.text_input.insert_str(self.text_cursor_pos, text);
                            self.text_cursor_pos += text.len();
                            self.needs_repaint = true;
                        }
                        egui::Event::Paste(text) => {
                            self.text_input.insert_str(self.text_cursor_pos, text);
                            self.text_cursor_pos += text.len();
                            self.needs_repaint = true;
                        }
                        egui::Event::Key { key, pressed: true, modifiers: _, .. } => {
                            match key {
                                egui::Key::Backspace => {
                                    if self.text_cursor_pos > 0 {
                                        self.text_input.remove(self.text_cursor_pos - 1);
                                        self.text_cursor_pos -= 1;
                                        self.needs_repaint = true;
                                    }
                                }
                                egui::Key::Delete => {
                                    if self.text_cursor_pos < self.text_input.len() {
                                        self.text_input.remove(self.text_cursor_pos);
                                        self.needs_repaint = true;
                                    }
                                }
                                egui::Key::ArrowLeft => {
                                    if self.text_cursor_pos > 0 {
                                        self.text_cursor_pos -= 1;
                                        self.needs_repaint = true;
                                    }
                                }
                                egui::Key::ArrowRight => {
                                    if self.text_cursor_pos < self.text_input.len() {
                                        self.text_cursor_pos += 1;
                                        self.needs_repaint = true;
                                    }
                                }
                                egui::Key::Enter => {
                                    if let Some(DrawObject::LatexFormula { formula, cached_size, .. }) = 
                                        self.objects.iter_mut().find(|o| o.id() == editing_id) {
                                        *formula = self.text_input.clone();
                                        *cached_size = None;
                                    }
                                    self.editing_text = None;
                                    self.text_input.clear();
                                    self.text_cursor_pos = 0;
                                    self.needs_repaint = true;
                                }
                                egui::Key::Escape => {
                                    self.editing_text = None;
                                    self.text_input.clear();
                                    self.text_cursor_pos = 0;
                                    self.needs_repaint = true;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            });
        }
    }

    fn render_latex_dialog(&mut self, ctx: &egui::Context) {
        if self.show_latex_dialog {
            egui::Window::new("Add LaTeX Formula")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Enter LaTeX formula:");
                    ui.text_edit_singleline(&mut self.latex_input);
                    ui.horizontal(|ui| {
                        if ui.button("Add").clicked() {
                            let formula = DrawObject::LatexFormula {
                                id: Uuid::new_v4(),
                                pos: self.latex_placement_pos,
                                formula: self.latex_input.clone(),
                                color: [
                                    self.current_color.r(),
                                    self.current_color.g(),
                                    self.current_color.b(),
                                    self.current_color.a(),
                                ],
                                cached_size: None,
                            };
                            self.objects.push(formula);
                            self.latex_input.clear();
                            self.show_latex_dialog = false;
                            self.needs_repaint = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_latex_dialog = false;
                        }
                    });
                });
        }
    }

    fn render_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        if !self.show_grid {
            return;
        }

        let grid_spacing = 50.0;
        let dot_size = 2.0;
        let dot_opacity = 30;
        
        let bg_brightness = (self.background_color.r() as u32 + 
                            self.background_color.g() as u32 + 
                            self.background_color.b() as u32) / 3;
        let dot_color = if bg_brightness > 128 {
            egui::Color32::from_rgba_premultiplied(100, 100, 100, dot_opacity)
        } else {
            egui::Color32::from_rgba_premultiplied(200, 200, 200, dot_opacity)
        };
        
        let min_canvas = canvas::screen_to_canvas(rect.min, self.canvas_offset, self.canvas_zoom);
        let max_canvas = canvas::screen_to_canvas(rect.max, self.canvas_offset, self.canvas_zoom);
        
        let start_x = (min_canvas[0] / grid_spacing).floor() * grid_spacing;
        let start_y = (min_canvas[1] / grid_spacing).floor() * grid_spacing;
        let end_x = (max_canvas[0] / grid_spacing).ceil() * grid_spacing;
        let end_y = (max_canvas[1] / grid_spacing).ceil() * grid_spacing;
        
        let mut x = start_x;
        while x <= end_x {
            let mut y = start_y;
            while y <= end_y {
                let screen_pos = canvas::canvas_to_screen([x, y], self.canvas_offset, self.canvas_zoom);
                painter.circle_filled(screen_pos, dot_size, dot_color);
                y += grid_spacing;
            }
            x += grid_spacing;
        }
    }

    fn render_objects(&mut self, ctx: &egui::Context, painter: &egui::Painter) {
        let latex_formulas: Vec<(Uuid, [f32; 2], String, [u8; 4])> = self.objects
            .iter()
            .filter_map(|obj| {
                if let DrawObject::LatexFormula { id, pos, formula, color, .. } = obj {
                    if !formula.is_empty() {
                        return Some((*id, *pos, formula.clone(), *color));
                    }
                }
                None
            })
            .collect();

        for obj in &self.objects {
            if !matches!(obj, DrawObject::LatexFormula { .. }) {
                canvas::render_object(painter, obj, self.canvas_offset, self.canvas_zoom);
            }
        }

        for (id, pos, formula, color) in latex_formulas {
            if let Some(texture) = self.latex_renderer.get_or_create_texture(ctx, &formula, color) {
                let screen_pos = canvas::canvas_to_screen(pos, self.canvas_offset, self.canvas_zoom);
                let size = texture.size_vec2() * self.canvas_zoom;
                
                let canvas_size = [size.x / self.canvas_zoom, size.y / self.canvas_zoom];
                if let Some(DrawObject::LatexFormula { cached_size, .. }) = self.objects.iter_mut().find(|o| o.id() == id) {
                    *cached_size = Some(canvas_size);
                }
                
                let rect = egui::Rect::from_min_size(screen_pos, size);
                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }
    }

    fn handle_brush_tool(&mut self, response: &egui::Response, canvas_pos: [f32; 2]) {
        if response.drag_started() {
            self.is_drawing = true;
            self.current_stroke = vec![StrokePoint { pos: canvas_pos }];
            self.needs_repaint = true;
        }

        if self.is_drawing && response.dragged() {
            self.current_stroke.push(StrokePoint { pos: canvas_pos });
            self.needs_repaint = true;
        }

        if response.drag_stopped() && self.is_drawing {
            if self.current_stroke.len() > 1 {
                self.push_undo();
                let color = self.current_color.to_array();
                let smoothed_points = canvas::smooth_stroke(&self.current_stroke);
                let stroke = DrawObject::Stroke {
                    id: Uuid::new_v4(),
                    points: smoothed_points,
                    color,
                    width: self.brush_size,
                };
                self.objects.push(stroke);
            }
            self.is_drawing = false;
            self.current_stroke.clear();
            self.needs_repaint = true;
        }
    }

    fn handle_shape_tool(&mut self, response: &egui::Response, pointer_pos: egui::Pos2, canvas_pos: [f32; 2], painter: &egui::Painter) {
        if response.drag_started() {
            self.draw_start_pos = Some(canvas_pos);
            self.needs_repaint = true;
        }

        if let Some(start_pos) = self.draw_start_pos {
            if response.dragged() {
                self.needs_repaint = true;
                let color = self.current_color;
                match self.current_tool {
                    Tool::Line => {
                        let start = canvas::canvas_to_screen(start_pos, self.canvas_offset, self.canvas_zoom);
                        let end = pointer_pos;
                        painter.line_segment(
                            [start, end],
                            egui::Stroke::new(self.brush_size * self.canvas_zoom, color),
                        );
                    }
                    Tool::Circle => {
                        let start = canvas::canvas_to_screen(start_pos, self.canvas_offset, self.canvas_zoom);
                        let radius = start.distance(pointer_pos);
                        painter.circle_stroke(
                            start,
                            radius,
                            egui::Stroke::new(self.brush_size * self.canvas_zoom, color),
                        );
                    }
                    Tool::Square => {
                        let start = canvas::canvas_to_screen(start_pos, self.canvas_offset, self.canvas_zoom);
                        let rect = egui::Rect::from_two_pos(start, pointer_pos);
                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(self.brush_size * self.canvas_zoom, color),
                        );
                    }
                    _ => {}
                }
            }
        }

        if response.drag_stopped() {
            if let Some(start_pos) = self.draw_start_pos {
                self.push_undo();
                let color_array = self.current_color.to_array();
                match self.current_tool {
                    Tool::Line => {
                        let line = DrawObject::Line {
                            id: Uuid::new_v4(),
                            start: start_pos,
                            end: canvas_pos,
                            color: color_array,
                            width: self.brush_size,
                        };
                        self.objects.push(line);
                    }
                    Tool::Circle => {
                        let dx = canvas_pos[0] - start_pos[0];
                        let dy = canvas_pos[1] - start_pos[1];
                        let radius = (dx * dx + dy * dy).sqrt();
                        let circle = DrawObject::Circle {
                            id: Uuid::new_v4(),
                            center: start_pos,
                            radius,
                            color: color_array,
                            width: self.brush_size,
                            filled: false,
                        };
                        self.objects.push(circle);
                    }
                    Tool::Square => {
                        let min = [
                            start_pos[0].min(canvas_pos[0]),
                            start_pos[1].min(canvas_pos[1]),
                        ];
                        let max = [
                            start_pos[0].max(canvas_pos[0]),
                            start_pos[1].max(canvas_pos[1]),
                        ];
                        let rect = DrawObject::Rectangle {
                            id: Uuid::new_v4(),
                            min,
                            max,
                            color: color_array,
                            width: self.brush_size,
                            filled: false,
                        };
                        self.objects.push(rect);
                    }
                    _ => {}
                }
                self.draw_start_pos = None;
                self.needs_repaint = true;
            }
        }
    }

    fn handle_eraser_tool(&mut self, response: &egui::Response, canvas_pos: [f32; 2]) {
        if response.drag_started() || response.dragged() {
            if let Some(obj_id) = canvas::find_object_at(&self.objects, canvas_pos) {
                self.push_undo();
                self.objects.retain(|obj| obj.id() != obj_id);
                self.needs_repaint = true;
            }
        }
    }

    fn handle_select_tool(&mut self, response: &egui::Response, canvas_pos: [f32; 2]) {
        if response.drag_started() {
            if let Some(bounds) = selection::get_selection_bounds(&self.objects, &self.selected_objects) {
                if let Some(handle) = selection::get_handle_at_pos(canvas_pos, bounds, self.canvas_zoom) {
                    self.selection_handle = Some(handle);
                    self.selection_drag_start = Some(canvas_pos);
                    self.selection_original_bounds = Some(bounds);
                    
                    self.selection_saved_objects = self.selected_objects
                        .iter()
                        .filter_map(|id| self.objects.iter().find(|o| o.id() == *id).cloned())
                        .collect();
                    
                    self.selection_mode = match handle {
                        SelectionHandle::Rotate => SelectionMode::Rotating,
                        _ => SelectionMode::Scaling,
                    };
                    self.needs_repaint = true;
                } else {
                    let (min, max) = bounds;
                    if canvas_pos[0] >= min[0] && canvas_pos[0] <= max[0] &&
                       canvas_pos[1] >= min[1] && canvas_pos[1] <= max[1] {
                        self.selection_mode = SelectionMode::Moving;
                        self.selection_drag_start = Some(canvas_pos);
                        self.needs_repaint = true;
                    } else {
                        self.selected_objects.clear();
                        self.selection_start = Some(canvas_pos);
                        self.selection_mode = SelectionMode::Selecting;
                        self.needs_repaint = true;
                    }
                }
            } else {
                self.selection_start = Some(canvas_pos);
                self.selection_mode = SelectionMode::Selecting;
                self.needs_repaint = true;
            }
        }

        if response.dragged() {
            match self.selection_mode {
                SelectionMode::Selecting => {
                    if let Some(start) = self.selection_start {
                        self.selection_rect = Some((start, canvas_pos));
                        self.needs_repaint = true;
                    }
                }
                SelectionMode::Moving => {
                    if let Some(drag_start) = self.selection_drag_start {
                        let delta = [
                            canvas_pos[0] - drag_start[0],
                            canvas_pos[1] - drag_start[1],
                        ];
                        
                        if let Some(bounds) = selection::get_selection_bounds(&self.objects, &self.selected_objects) {
                            let center = [
                                (bounds.0[0] + bounds.1[0]) / 2.0,
                                (bounds.0[1] + bounds.1[1]) / 2.0,
                            ];
                            selection::transform_objects(&mut self.objects, &self.selected_objects, [1.0, 1.0], 0.0, delta, center);
                        }
                        
                        self.selection_drag_start = Some(canvas_pos);
                        self.needs_repaint = true;
                    }
                }
                SelectionMode::Scaling => {
                    if let (Some(orig_bounds), Some(handle)) = 
                       (self.selection_original_bounds, self.selection_handle) {
                        
                        let center = [
                            (orig_bounds.0[0] + orig_bounds.1[0]) / 2.0,
                            (orig_bounds.0[1] + orig_bounds.1[1]) / 2.0,
                        ];
                        
                        let orig_width = orig_bounds.1[0] - orig_bounds.0[0];
                        let orig_height = orig_bounds.1[1] - orig_bounds.0[1];
                        
                        let mut new_width = orig_width;
                        let mut new_height = orig_height;
                        
                        match handle {
                            SelectionHandle::Left | SelectionHandle::Right => {
                                new_width = if matches!(handle, SelectionHandle::Right) {
                                    (canvas_pos[0] - orig_bounds.0[0]).max(10.0)
                                } else {
                                    (orig_bounds.1[0] - canvas_pos[0]).max(10.0)
                                };
                            }
                            SelectionHandle::Top | SelectionHandle::Bottom => {
                                new_height = if matches!(handle, SelectionHandle::Bottom) {
                                    (canvas_pos[1] - orig_bounds.0[1]).max(10.0)
                                } else {
                                    (orig_bounds.1[1] - canvas_pos[1]).max(10.0)
                                };
                            }
                            SelectionHandle::TopLeft | SelectionHandle::TopRight | 
                            SelectionHandle::BottomLeft | SelectionHandle::BottomRight => {
                                let (ref_x, ref_y) = match handle {
                                    SelectionHandle::TopLeft => (orig_bounds.1[0], orig_bounds.1[1]),
                                    SelectionHandle::TopRight => (orig_bounds.0[0], orig_bounds.1[1]),
                                    SelectionHandle::BottomLeft => (orig_bounds.1[0], orig_bounds.0[1]),
                                    SelectionHandle::BottomRight => (orig_bounds.0[0], orig_bounds.0[1]),
                                    _ => (center[0], center[1]),
                                };
                                new_width = (canvas_pos[0] - ref_x).abs().max(10.0);
                                new_height = (canvas_pos[1] - ref_y).abs().max(10.0);
                                
                                let aspect = orig_width / orig_height;
                                if new_width / new_height > aspect {
                                    new_height = new_width / aspect;
                                } else {
                                    new_width = new_height * aspect;
                                }
                            }
                            _ => {}
                        }
                        
                        let scale_x = new_width / orig_width;
                        let scale_y = new_height / orig_height;
                        
                        for saved_obj in &self.selection_saved_objects {
                            if let Some(current_obj) = self.objects.iter_mut().find(|o| o.id() == saved_obj.id()) {
                                *current_obj = saved_obj.clone();
                            }
                        }
                        
                        selection::transform_objects(&mut self.objects, &self.selected_objects, [scale_x, scale_y], 0.0, [0.0, 0.0], center);
                        self.needs_repaint = true;
                    }
                }
                SelectionMode::Rotating => {
                    if let (Some(drag_start), Some(bounds)) = 
                       (self.selection_drag_start, self.selection_original_bounds) {
                        
                        let center = [
                            (bounds.0[0] + bounds.1[0]) / 2.0,
                            (bounds.0[1] + bounds.1[1]) / 2.0,
                        ];
                        
                        let start_angle = (drag_start[1] - center[1]).atan2(drag_start[0] - center[0]);
                        let current_angle = (canvas_pos[1] - center[1]).atan2(canvas_pos[0] - center[0]);
                        let rotation = current_angle - start_angle;
                        
                        for saved_obj in &self.selection_saved_objects {
                            if let Some(current_obj) = self.objects.iter_mut().find(|o| o.id() == saved_obj.id()) {
                                *current_obj = saved_obj.clone();
                            }
                        }
                        
                        selection::transform_objects(&mut self.objects, &self.selected_objects, [1.0, 1.0], rotation, [0.0, 0.0], center);
                        self.needs_repaint = true;
                    }
                }
                _ => {}
            }
        }

        if response.drag_stopped() {
            if self.selection_mode == SelectionMode::Selecting {
                if let Some((start, end)) = self.selection_rect {
                    let min_x = start[0].min(end[0]);
                    let max_x = start[0].max(end[0]);
                    let min_y = start[1].min(end[1]);
                    let max_y = start[1].max(end[1]);
                    
                    self.selected_objects.clear();
                    for obj in &self.objects {
                        let (obj_min, obj_max) = obj.bounds();
                        if obj_min[0] >= min_x && obj_max[0] <= max_x &&
                           obj_min[1] >= min_y && obj_max[1] <= max_y {
                            self.selected_objects.push(obj.id());
                        }
                    }
                }
                
                self.selection_rect = None;
                self.selection_start = None;
            }
            
            self.selection_mode = SelectionMode::None;
            self.selection_drag_start = None;
            self.selection_handle = None;
            self.selection_saved_objects.clear();
            self.needs_repaint = true;
        }
    }

    fn handle_text_tool(&mut self, response: &egui::Response, canvas_pos: [f32; 2]) {
        if response.clicked() {
            let clicked_existing = if let Some(obj_id) = canvas::find_object_at(&self.objects, canvas_pos) {
                if let Some(DrawObject::LatexFormula { formula, .. }) = self.objects.iter().find(|o| o.id() == obj_id) {
                    self.editing_text = Some(obj_id);
                    self.text_input = formula.clone();
                    self.text_cursor_pos = formula.len();
                    self.needs_repaint = true;
                    true
                } else {
                    false
                }
            } else {
                false
            };
            
            if !clicked_existing {
                self.push_undo();
                let new_id = Uuid::new_v4();
                let formula = DrawObject::LatexFormula {
                    id: new_id,
                    pos: canvas_pos,
                    formula: String::new(),
                    color: [
                        self.current_color.r(),
                        self.current_color.g(),
                        self.current_color.b(),
                        self.current_color.a(),
                    ],
                    cached_size: None,
                };
                self.objects.push(formula);
                self.editing_text = Some(new_id);
                self.text_input.clear();
                self.text_cursor_pos = 0;
                self.needs_repaint = true;
            }
        }
    }

    fn render_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::click_and_drag(),
            );

            painter.rect_filled(response.rect, 0.0, self.background_color);
            
            self.render_grid(&painter, response.rect);

            if response.hovered() {
                let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll_delta != 0.0 {
                    let zoom_factor = 1.0 + scroll_delta * 0.001;
                    let old_zoom = self.canvas_zoom;
                    self.canvas_zoom = (self.canvas_zoom * zoom_factor).clamp(0.1, 10.0);
                    
                    if let Some(hover_pos) = response.hover_pos() {
                        let zoom_ratio = self.canvas_zoom / old_zoom;
                        self.canvas_offset = hover_pos.to_vec2() + (self.canvas_offset - hover_pos.to_vec2()) * zoom_ratio;
                    }
                    self.needs_repaint = true;
                }
            }

            if response.dragged_by(egui::PointerButton::Middle) {
                self.canvas_offset += response.drag_delta();
                self.needs_repaint = true;
            }

            self.render_objects(ctx, &painter);

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = canvas::screen_to_canvas(pointer_pos, self.canvas_offset, self.canvas_zoom);

                match self.current_tool {
                    Tool::Brush => self.handle_brush_tool(&response, canvas_pos),
                    Tool::Line | Tool::Circle | Tool::Square => self.handle_shape_tool(&response, pointer_pos, canvas_pos, &painter),
                    Tool::Eraser => self.handle_eraser_tool(&response, canvas_pos),
                    Tool::Select => self.handle_select_tool(&response, canvas_pos),
                    Tool::Text => self.handle_text_tool(&response, canvas_pos),
                }
            }

            if self.is_drawing && self.current_stroke.len() > 1 {
                for i in 0..self.current_stroke.len() - 1 {
                    let start = canvas::canvas_to_screen(self.current_stroke[i].pos, self.canvas_offset, self.canvas_zoom);
                    let end = canvas::canvas_to_screen(self.current_stroke[i + 1].pos, self.canvas_offset, self.canvas_zoom);
                    painter.line_segment(
                        [start, end],
                        egui::Stroke::new(self.brush_size * self.canvas_zoom, self.current_color),
                    );
                }
            }

            if let Some((start, end)) = self.selection_rect {
                let screen_start = canvas::canvas_to_screen(start, self.canvas_offset, self.canvas_zoom);
                let screen_end = canvas::canvas_to_screen(end, self.canvas_offset, self.canvas_zoom);
                let rect = egui::Rect::from_two_pos(screen_start, screen_end);
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 255)),
                );
                painter.rect_filled(
                    rect,
                    0.0,
                    egui::Color32::from_rgba_premultiplied(100, 150, 255, 20),
                );
            }

            if !self.selected_objects.is_empty() && self.selection_mode != SelectionMode::Selecting {
                if let Some((min, max)) = selection::get_selection_bounds(&self.objects, &self.selected_objects) {
                    let screen_min = canvas::canvas_to_screen(min, self.canvas_offset, self.canvas_zoom);
                    let screen_max = canvas::canvas_to_screen(max, self.canvas_offset, self.canvas_zoom);
                    let rect = egui::Rect::from_two_pos(screen_min, screen_max);
                    
                    painter.rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(50, 100, 255)),
                    );
                    
                    let handle_size = 8.0;
                    let mid_x = (screen_min.x + screen_max.x) / 2.0;
                    let mid_y = (screen_min.y + screen_max.y) / 2.0;
                    
                    let handles = vec![
                        (screen_min.x, screen_min.y),
                        (screen_max.x, screen_min.y),
                        (screen_min.x, screen_max.y),
                        (screen_max.x, screen_max.y),
                        (mid_x, screen_min.y),
                        (mid_x, screen_max.y),
                        (screen_min.x, mid_y),
                        (screen_max.x, mid_y),
                    ];
                    
                    for (x, y) in handles {
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                egui::pos2(x, y),
                                egui::vec2(handle_size, handle_size),
                            ),
                            0.0,
                            egui::Color32::WHITE,
                        );
                        painter.rect_stroke(
                            egui::Rect::from_center_size(
                                egui::pos2(x, y),
                                egui::vec2(handle_size, handle_size),
                            ),
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 100, 255)),
                        );
                    }
                    
                    let rotate_y = screen_min.y - 30.0;
                    painter.circle_filled(
                        egui::pos2(mid_x, rotate_y),
                        5.0,
                        egui::Color32::WHITE,
                    );
                    painter.circle_stroke(
                        egui::pos2(mid_x, rotate_y),
                        5.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 100, 255)),
                    );
                    painter.line_segment(
                        [egui::pos2(mid_x, screen_min.y), egui::pos2(mid_x, rotate_y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 100, 255)),
                    );
                }
            }
            
            if let Some(editing_id) = self.editing_text {
                if let Some(DrawObject::LatexFormula { pos, .. }) = self.objects.iter().find(|o| o.id() == editing_id) {
                    let screen_pos = canvas::canvas_to_screen(*pos, self.canvas_offset, self.canvas_zoom);
                        
                    let text_width = (self.text_input.len().max(10) as f32) * 8.0;
                    let text_height = 30.0;
                    let text_rect = egui::Rect::from_min_size(
                        screen_pos,
                        egui::vec2(text_width, text_height),
                    );
                    painter.rect_filled(
                        text_rect,
                        2.0,
                        egui::Color32::from_rgba_premultiplied(255, 255, 255, 240),
                    );
                    painter.rect_stroke(
                        text_rect,
                        2.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                    );
                    
                    painter.text(
                        screen_pos + egui::vec2(5.0, 5.0),
                        egui::Align2::LEFT_TOP,
                        &self.text_input,
                        egui::FontId::monospace(14.0),
                        egui::Color32::BLACK,
                    );
                    
                    let cursor_x_offset = (self.text_cursor_pos as f32) * 8.0 + 5.0;
                    let time = ctx.input(|i| i.time);
                    if (time * 2.0).fract() < 0.5 {
                        painter.line_segment(
                            [
                                screen_pos + egui::vec2(cursor_x_offset, 5.0),
                                screen_pos + egui::vec2(cursor_x_offset, 23.0),
                            ],
                            egui::Stroke::new(2.0, egui::Color32::BLACK),
                        );
                    }
                }
            }
        });
    }
}

impl eframe::App for WhiteboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard_shortcuts(ctx);
        self.render_toolbar(ctx);
        self.handle_text_editing(ctx);
        self.render_latex_dialog(ctx);
        self.render_canvas(ctx);

        if self.needs_repaint || self.is_drawing || self.draw_start_pos.is_some() || 
           !self.selected_objects.is_empty() || self.selection_mode != SelectionMode::None || 
           self.editing_text.is_some() {
            ctx.request_repaint();
            self.needs_repaint = false;
        }
    }
}
