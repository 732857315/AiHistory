//! Neural awakening explosion - AlexNet moment, deep learning breakthrough

use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

const CYAN: &str = "#38bdf8";
const BLUE: &str = "#3b82f6";
const TEAL: &str = "#14b8a6";

pub fn draw_neural_awakening(ctx: &CanvasRenderingContext2d, t: f64) -> Result<(), JsValue> {
    let t = t.clamp(0.0, 1.0);

    ctx.save();

    // Neural grid awakening (0.0-0.6)
    if t < 0.6 {
        let phase = t / 0.6;
        ctx.set_global_composite_operation("lighter")?;

        // Grid lines pulsing
        for i in 0..15 {
            for j in 0..10 {
                let x = 100.0 + i as f64 * 80.0;
                let y = 80.0 + j as f64 * 60.0;
                let delay = (i + j) as f64 * 0.05;
                let node_t = ((phase - delay) / 0.3).clamp(0.0, 1.0);

                if node_t > 0.0 {
                    let pulse = (node_t * std::f64::consts::PI).sin();
                    let alpha = pulse * (1.0 - node_t * 0.5);

                    // Node
                    ctx.set_fill_style_str(&format!("rgba(56, 189, 248, {})", alpha * 0.8));
                    ctx.begin_path();
                    ctx.arc(x, y, 3.0 + pulse * 5.0, 0.0, std::f64::consts::TAU)?;
                    ctx.fill();

                    // Connections
                    if i < 14 {
                        ctx.set_stroke_style_str(&format!("rgba(20, 184, 166, {})", alpha * 0.4));
                        ctx.set_line_width(1.0 + pulse * 2.0);
                        ctx.begin_path();
                        ctx.move_to(x, y);
                        ctx.line_to(x + 80.0, y);
                        ctx.stroke();
                    }
                    if j < 9 {
                        ctx.set_stroke_style_str(&format!("rgba(59, 130, 246, {})", alpha * 0.3));
                        ctx.set_line_width(1.0 + pulse * 1.5);
                        ctx.begin_path();
                        ctx.move_to(x, y);
                        ctx.line_to(x, y + 60.0);
                        ctx.stroke();
                    }
                }
            }
        }

        ctx.set_global_composite_operation("source-over")?;
    }

    // Energy wave explosion (0.4-1.0)
    if t > 0.4 {
        let wave_t = ((t - 0.4) / 0.6).clamp(0.0, 1.0);
        let cx = 600.0;
        let cy = 337.0;

        for ring in 0..5 {
            let delay = ring as f64 * 0.15;
            let ring_t = ((wave_t - delay) / 0.5).clamp(0.0, 1.0);

            if ring_t > 0.0 {
                let radius = ring_t * 450.0;
                let alpha = (1.0 - ring_t).powi(3);

                ctx.set_stroke_style_str(&format!("rgba(56, 189, 248, {})", alpha * 0.7));
                ctx.set_line_width(4.0 * (1.0 - ring_t));
                ctx.begin_path();
                ctx.arc(cx, cy, radius, 0.0, std::f64::consts::TAU)?;
                ctx.stroke();
            }
        }
    }

    // Text fade in (0.7-1.0)
    if t > 0.7 {
        let text_alpha = ((t - 0.7) / 0.3).clamp(0.0, 1.0);
        ctx.set_global_alpha(text_alpha);
        ctx.set_font("bold 42px -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif");
        ctx.set_fill_style_str(CYAN);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let _ = ctx.fill_text("Neural Networks Awaken", 600.0, 550.0);
    }

    ctx.restore();
    Ok(())
}
