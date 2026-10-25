//! Stone impact explosion - the first tool, the first spark of intelligence

use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

const ORANGE: &str = "#ff8844";
const YELLOW: &str = "#ffdd66";
const BROWN: &str = "#8b6b47";

pub fn draw_stone_impact(ctx: &CanvasRenderingContext2d, t: f64) -> Result<(), JsValue> {
    let t = t.clamp(0.0, 1.0);

    ctx.save();
    ctx.set_fill_style_str("#0a0b0d");
    ctx.fill_rect(0.0, 0.0, 1200.0, 675.0);

    // Impact center
    let cx = 600.0;
    let cy = 337.0;

    // Explosion phase (0.0-0.4)
    if t < 0.4 {
        let phase = t / 0.4;
        let radius = phase * 400.0;
        let alpha = (1.0 - phase).powi(2);

        // Flash
        let grad = ctx.create_radial_gradient(cx, cy, 0.0, cx, cy, radius)?;
        grad.add_color_stop(0.0, &format!("rgba(255, 220, 120, {})", alpha * 0.9))?;
        grad.add_color_stop(0.5, &format!("rgba(255, 136, 68, {})", alpha * 0.6))?;
        grad.add_color_stop(1.0, "rgba(139, 107, 71, 0)")?;
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0.0, 0.0, 1200.0, 675.0);
    }

    // Particles (0.2-1.0)
    if t > 0.2 {
        let particle_t = ((t - 0.2) / 0.8).clamp(0.0, 1.0);
        ctx.set_global_composite_operation("lighter")?;

        for i in 0..60 {
            let angle = (i as f64 / 60.0) * std::f64::consts::TAU;
            let speed = 200.0 + (i as f64 * 7.3).sin() * 150.0;
            let dist = particle_t * speed;
            let x = cx + angle.cos() * dist;
            let y = cy + angle.sin() * dist;

            let size = 3.0 + (i as f64 * 3.7).cos().abs() * 4.0;
            let alpha = (1.0 - particle_t).powi(2);

            let color = if i % 3 == 0 { YELLOW } else if i % 3 == 1 { ORANGE } else { BROWN };
            ctx.set_fill_style_str(&format!("{}99", color));
            ctx.set_global_alpha(alpha);
            ctx.fill_rect(x - size / 2.0, y - size / 2.0, size, size);
        }

        ctx.set_global_composite_operation("source-over")?;
        ctx.set_global_alpha(1.0);
    }

    // Fade in text (0.5-1.0)
    if t > 0.5 {
        let text_alpha = ((t - 0.5) / 0.5).clamp(0.0, 1.0);
        ctx.set_global_alpha(text_alpha);
        ctx.set_font("bold 48px -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif");
        ctx.set_fill_style_str("#f1e8d6");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let _ = ctx.fill_text("The First Tool", cx, cy + 150.0);
    }

    ctx.restore();
    Ok(())
}
