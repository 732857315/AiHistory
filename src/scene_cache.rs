//! 🎨 Scene layer caching system
//! Cache static scene elements to offscreen canvases, only redraw dynamic parts

use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Manages offscreen canvas layers for efficient rendering
pub struct SceneCache {
    layers: HashMap<String, HtmlCanvasElement>,
    width: u32,
    height: u32,
}

impl SceneCache {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            layers: HashMap::new(),
            width,
            height,
        }
    }

    /// Get or create a cached layer, calling draw_fn once if not cached
    pub fn get_or_create<F>(
        &mut self,
        key: &str,
        draw_fn: F,
    ) -> Option<&HtmlCanvasElement>
    where
        F: FnOnce(&CanvasRenderingContext2d),
    {
        if !self.layers.contains_key(key) {
            let canvas = create_offscreen_canvas(self.width, self.height)?;
            let ctx = get_2d_context(&canvas)?;

            // Set up high-quality rendering
            ctx.set_line_cap("round");
            ctx.set_line_join("round");
            ctx.set_image_smoothing_enabled(true);

            // Draw to offscreen canvas
            draw_fn(&ctx);

            self.layers.insert(key.to_string(), canvas);
        }

        self.layers.get(key)
    }

    /// Invalidate a cached layer, forcing redraw on next get_or_create
    pub fn invalidate(&mut self, key: &str) {
        self.layers.remove(key);
    }

    /// Clear all cached layers
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Resize all canvases (clears cache)
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.clear();
    }

    /// Get number of cached layers
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Check if a layer is cached
    pub fn has_layer(&self, key: &str) -> bool {
        self.layers.contains_key(key)
    }
}

/// Create an offscreen canvas for caching
fn create_offscreen_canvas(width: u32, height: u32) -> Option<HtmlCanvasElement> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let canvas: HtmlCanvasElement = document.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(width);
    canvas.set_height(height);
    Some(canvas)
}

/// Get 2D context from canvas
fn get_2d_context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()??
        .dyn_into()
        .ok()
}

/// Composite multiple layers onto target context
pub fn composite_layers(
    target: &CanvasRenderingContext2d,
    layers: &[(&HtmlCanvasElement, f64, f64, f64)], // (canvas, x, y, alpha)
) -> Result<(), wasm_bindgen::JsValue> {
    target.save();

    for (canvas, x, y, alpha) in layers {
        if *alpha < 1.0 {
            target.set_global_alpha(*alpha);
        }
        target.draw_image_with_html_canvas_element(canvas, *x, *y)?;
        if *alpha < 1.0 {
            target.set_global_alpha(1.0);
        }
    }

    target.restore();
    Ok(())
}

/// Layer blending modes
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
}

impl BlendMode {
    pub fn to_str(&self) -> &'static str {
        match self {
            BlendMode::Normal => "source-over",
            BlendMode::Multiply => "multiply",
            BlendMode::Screen => "screen",
            BlendMode::Overlay => "overlay",
            BlendMode::Add => "lighter",
        }
    }
}

/// Composite with blend mode
pub fn composite_with_blend(
    target: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    x: f64,
    y: f64,
    alpha: f64,
    blend: BlendMode,
) -> Result<(), wasm_bindgen::JsValue> {
    target.save();
    target.set_global_alpha(alpha);
    target.set_global_composite_operation(blend.to_str())?;
    target.draw_image_with_html_canvas_element(canvas, x, y)?;
    target.restore();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_creates_layers_once() {
        let mut cache = SceneCache::new(1200, 675);

        // First call should create
        let mut draw_count = 0;
        cache.get_or_create("test", |_ctx| {
            draw_count += 1;
        });
        assert_eq!(draw_count, 1);

        // Second call should not draw again
        cache.get_or_create("test", |_ctx| {
            draw_count += 1;
        });
        assert_eq!(draw_count, 1);

        assert_eq!(cache.layer_count(), 1);
    }

    #[test]
    fn cache_invalidation_forces_redraw() {
        let mut cache = SceneCache::new(1200, 675);

        let mut draw_count = 0;
        cache.get_or_create("test", |_ctx| {
            draw_count += 1;
        });

        cache.invalidate("test");

        cache.get_or_create("test", |_ctx| {
            draw_count += 1;
        });

        assert_eq!(draw_count, 2);
    }
}
