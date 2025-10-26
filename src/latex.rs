use eframe::egui;
use std::collections::HashMap;
use std::sync::Arc;

pub struct LatexRenderer {
    cache: HashMap<String, Arc<egui::ColorImage>>,
    textures: HashMap<String, egui::TextureHandle>,
}

impl LatexRenderer {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn render_to_image(&mut self, formula: &str, color: [u8; 4]) -> Result<Arc<egui::ColorImage>, String> {
        let cache_key = format!("{}_{}_{}_{}", formula, color[0], color[1], color[2]);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let mut svg_string = match mathjax_svg::convert_to_svg(formula) {
            Ok(svg) => svg,
            Err(e) => return Err(format!("Failed to render LaTeX: {}", e)),
        };
        
        let color_hex = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        svg_string = svg_string.replace("currentColor", &color_hex);
        svg_string = svg_string.replace("fill=\"#000\"", &format!("fill=\"{}\"", color_hex));
        svg_string = svg_string.replace("fill=\"#000000\"", &format!("fill=\"{}\"", color_hex));
        svg_string = svg_string.replace("fill=\"black\"", &format!("fill=\"{}\"", color_hex));
        svg_string = svg_string.replace("stroke=\"#000\"", &format!("stroke=\"{}\"", color_hex));
        svg_string = svg_string.replace("stroke=\"#000000\"", &format!("stroke=\"{}\"", color_hex));
        svg_string = svg_string.replace("stroke=\"black\"", &format!("stroke=\"{}\"", color_hex));

        let opt = usvg::Options::default();
        let tree = match usvg::Tree::from_str(&svg_string, &opt) {
            Ok(tree) => tree,
            Err(e) => return Err(format!("Failed to parse SVG: {}", e)),
        };

        let size = tree.size();
        let scale_factor = 3.0;
        let width = (size.width() * scale_factor) as u32;
        let height = (size.height() * scale_factor) as u32;

        if width == 0 || height == 0 {
            return Err("Invalid image dimensions".to_string());
        }

        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or("Failed to create pixmap")?;

        let transform = tiny_skia::Transform::from_scale(scale_factor, scale_factor);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let mut image_data = Vec::with_capacity((width * height) as usize);
        for pixel in pixmap.pixels() {
            image_data.push(egui::Color32::from_rgba_premultiplied(
                pixel.red(),
                pixel.green(),
                pixel.blue(),
                pixel.alpha(),
            ));
        }

        let color_image = Arc::new(egui::ColorImage {
            size: [width as usize, height as usize],
            pixels: image_data,
        });

        self.cache.insert(cache_key, color_image.clone());
        Ok(color_image)
    }

    pub fn get_or_create_texture(&mut self, ctx: &egui::Context, formula: &str, color: [u8; 4]) -> Option<egui::TextureHandle> {
        let texture_key = format!("{}_{}_{}_{}", formula, color[0], color[1], color[2]);
        
        if let Some(texture) = self.textures.get(&texture_key) {
            return Some(texture.clone());
        }

        let image = match self.render_to_image(formula, color) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Error rendering LaTeX '{}': {}", formula, e);
                return None;
            }
        };

        let texture = ctx.load_texture(
            format!("latex_{}_{}_{}_{}", formula, color[0], color[1], color[2]),
            image.as_ref().clone(),
            egui::TextureOptions::LINEAR,
        );

        self.textures.insert(texture_key, texture.clone());
        Some(texture)
    }
}
