//! Attention storm explosion - Transformer breakthrough, attention is all you need

use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

const PURPLE: &str = "#a78bfa";
const _PINK: &str = "#f472b6";
const _INDIGO: &str = "#6366f1";

pub fn draw_attention_storm(ctx: &CanvasRenderingContext2d, t: f64) -> Result<(), JsValue> {
    let t = t.clamp(0.0, 1.0);
    
    ctx.save();
    ctx.set_fill_style_str("#05070c");
    ctx.fill_rect(0.0, 0.0, 1200.0, 675.0);
    
    let cx = 600.0;
    let cy = 337.0;
    
    // Attention beam burst (0.0-0.5)
    if t < 0.5 {
        let phase = t / 0.5;
        ctx.set_global_composite_operation("lighter")?;
        
        for i in 0..24 {
            let angle = (i as f64 / 24.0) * std::f64::consts::TAU + phase * 0.3;
            let length = phase * 500.0;
            let alpha = (1.0 - phase).powi(2);
            
            let gradient = ctx.create_linear_gradient(
                cx,
                cy,
                cx + angle.cos() * length,
                cy + angle.sin() * length,
            );
            let _ = gradient.add_color_stop(0.0, &format!("rgba(167, 139, 250, {})", alpha * 0.9));
            let _ = gradient.add_color_stop(0.5, &format!("rgba(244, 114, 182, {})", alpha * 0.6));
            let _ = gradient.add_color_stop(1.0, "rgba(99, 102, 241, 0)");
            
            ctx.set_stroke_style(&gradient);
            ctx.set_line_width(3.0 + (i as f64 * 1.3).sin().abs() * 4.0);
            ctx.begin_path();
            ctx.move_to(cx, cy);
            ctx.line_to(cx + angle.cos() * length, cy + angle.sin() * length);
            ctx.stroke();
        }
        
        ctx.set_global_composite_operation("source-over")?;
    }
    
    // Attention nodes connecting (0.3-0.8)
    if t > 0.3 && t < 0.8 {
        let node_t = ((t - 0.3) / 0.5).clamp(0.0, 1.0);
        
        for i in 0..12 {
            let angle = (i as f64 / 12.0) * std::f64::consts::TAU;
            let radius = 200.0;
            let x = cx + angle.cos() * radius;
            let y = cy + angle.sin() * radius;
            
            for j in (i + 1)..12 {
                let angle2 = (j as f64 / 12.0) * std::f64::consts::TAU;
                let x2 = cx + angle2.cos() * radius;
                let y2 = cy + angle2.sin() * radius;
                
                let alpha = node_t * (1.0 - node_t * 0.5);
                ctx.set_stroke_style_str(&format!("rgba(167, 139, 250, {})", alpha * 0.3));
                ctx.set_line_width(1.0);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x2, y2);
                ctx.stroke();
            }
            
            let alpha = node_t;
            ctx.set_fill_style_str(&format!("rgba(244, 114, 182, {})", alpha * 0.9));
            ctx.begin_path();
            ctx.arc(x, y, 6.0, 0.0, std::f64::consts::TAU)?;
            ctx.fill();
        }
    }
    
    // Central explosion (0.6-1.0)
    if t > 0.6 {
        let burst_t = ((t - 0.6) / 0.4).clamp(0.0, 1.0);
        let radius = burst_t * 350.0;
        let alpha = (1.0 - burst_t).powi(2);
        
        let grad = ctx.create_radial_gradient(cx, cy, 0.0, cx, cy, radius)?;
        let _ = grad.add_color_stop(0.0, &format!("rgba(167, 139, 250, {})", alpha * 0.8));
        let _ = grad.add_color_stop(0.6, &format!("rgba(99, 102, 241, {})", alpha * 0.4));
        let _ = grad.add_color_stop(1.0, "rgba(167, 139, 250, 0)");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0.0, 0.0, 1200.0, 675.0);
    }
    
    // Text (0.8-1.0)
    if t > 0.8 {
        let text_alpha = ((t - 0.8) / 0.2).clamp(0.0, 1.0);
        ctx.set_global_alpha(text_alpha);
        ctx.set_font("bold 52px -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif");
        ctx.set_fill_style_str(PURPLE);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let _ = ctx.fill_text("Attention Is All You Need", cx, cy + 200.0);
    }
    
    ctx.restore();
    Ok(())
}
