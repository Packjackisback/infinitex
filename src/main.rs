use eframe::egui;

mod models;
mod canvas;
mod latex;
mod selection;
mod file_io;
mod app;

use app::WhiteboardApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Math Workspace - Infinite Whiteboard",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(WhiteboardApp::default()))
        }),
    )
}
