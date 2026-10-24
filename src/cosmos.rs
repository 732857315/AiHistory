//! A live astronomical stage. The cached dust field is drawn procedurally into an
//! in-memory canvas; it is not an image asset, encoded media, or a recorded frame.
//! Every racer shares the same forward orbit; score positions determine the gaps.

use std::{
    cell::RefCell,
    f64::consts::{PI, TAU},
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::scene::Racer;

type Point = (f64, f64);
const NIGHT: &str = "#050914";
const ICE: &str = "#a0d3df";

thread_local! {
    static DUST: RefCell<Option<HtmlCanvasElement>> = const { RefCell::new(None) };
}

fn noise(seed: usize) -> f64 {
    let mut value = (seed as u32)
        .wrapping_mul(747_796_405)
        .wrapping_add(2_891_336_453);
    value = ((value >> ((value >> 28) + 4)) ^ value).wrapping_mul(277_803_737);
    f64::from((value >> 22) ^ value) / f64::from(u32::MAX)
}

fn identity(name: &str) -> usize {
    name.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    }) as usize
}

fn tint(color: &str, alpha: f64) -> String {
    let hex = color.strip_prefix('#').unwrap_or("aea7dc");
    let rgb = u32::from_str_radix(hex, 16).unwrap_or(0xaea7dc);
    format!(
        "rgba({},{},{},{:.4})",
        (rgb >> 16) & 255,
        (rgb >> 8) & 255,
        rgb & 255,
        alpha.clamp(0.0, 1.0)
    )
}

fn ellipse_point(center: Point, radius: Point, angle: f64, tilt: f64) -> Point {
    let x = angle.cos() * radius.0;
    let y = angle.sin() * radius.1;
    (
        center.0 + x * tilt.cos() - y * tilt.sin(),
        center.1 + x * tilt.sin() + y * tilt.cos(),
    )
}

fn orbit(progress: f64, lane: f64, time: f64, compact: bool) -> Point {
    // The shared phase is transport time, so pausing freezes the entire chase.
    // It never changes a score gap or invents an overtake.
    let angle = 3.05 - progress * 4.08 - (time - 154.0).max(0.0) * 0.037;
    if compact {
        return ellipse_point(
            (300.0, 231.0),
            (218.0 + lane * 0.35, 120.0 + lane * 0.18),
            angle,
            -0.18,
        );
    }
    ellipse_point(
        (649.0, 315.0),
        (457.0 + lane, 165.0 + lane * 0.38),
        angle,
        -0.15,
    )
}

fn line(ctx: &CanvasRenderingContext2d, points: &[Point], color: &str, width: f64) {
    if let Some(first) = points.first() {
        ctx.begin_path();
        ctx.move_to(first.0, first.1);
        for point in &points[1..] {
            ctx.line_to(point.0, point.1);
        }
        ctx.set_stroke_style_str(color);
        ctx.set_line_width(width);
        ctx.stroke();
    }
}

fn dot(ctx: &CanvasRenderingContext2d, p: Point, radius: f64, color: &str) {
    // Cubics keep the primitive infallible for finite radii.
    let k = radius * 0.552_284_749_8;
    let (x, y) = p;
    ctx.begin_path();
    ctx.move_to(x + radius, y);
    ctx.bezier_curve_to(x + radius, y + k, x + k, y + radius, x, y + radius);
    ctx.bezier_curve_to(x - k, y + radius, x - radius, y + k, x - radius, y);
    ctx.bezier_curve_to(x - radius, y - k, x - k, y - radius, x, y - radius);
    ctx.bezier_curve_to(x + k, y - radius, x + radius, y - k, x + radius, y);
    ctx.close_path();
    ctx.set_fill_style_str(color);
    ctx.fill();
}

fn glow(
    ctx: &CanvasRenderingContext2d,
    p: Point,
    radius: Point,
    color: &str,
    opacity: f64,
) -> Result<(), JsValue> {
    ctx.save();
    let result: Result<(), JsValue> = (|| {
        ctx.translate(p.0, p.1)?;
        ctx.scale(radius.0, radius.1)?;
        let gradient = ctx.create_radial_gradient(0.0, 0.0, 0.0, 0.0, 0.0, 1.0)?;
        gradient.add_color_stop(0.0, &tint(color, opacity))?;
        gradient.add_color_stop(0.26, &tint(color, opacity * 0.50))?;
        gradient.add_color_stop(0.62, &tint(color, opacity * 0.13))?;
        gradient.add_color_stop(1.0, &tint(color, 0.0))?;
        ctx.set_fill_style_canvas_gradient(&gradient);
        ctx.fill_rect(-1.0, -1.0, 2.0, 2.0);
        Ok(())
    })();
    ctx.restore();
    result
}

fn dust_field() -> Result<HtmlCanvasElement, JsValue> {
    DUST.with(|stored| {
        if let Some(canvas) = stored.borrow().as_ref() {
            return Ok(canvas.clone());
        }
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("Browser document unavailable"))?;
        let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
        canvas.set_width(1700);
        canvas.set_height(950);
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Canvas 2D unavailable"))?
            .dyn_into()?;
        ctx.set_fill_style_str(NIGHT);
        ctx.fill_rect(0.0, 0.0, 1700.0, 950.0);
        glow(&ctx, (910.0, 465.0), (740.0, 410.0), "#55518d", 0.44)?;
        glow(&ctx, (480.0, 670.0), (570.0, 260.0), "#264a66", 0.24)?;
        glow(&ctx, (1320.0, 230.0), (490.0, 290.0), "#4d416f", 0.18)?;

        // Dust clouds follow logarithmic arms, rather than outlining a flat disk.
        for arm in 0..3 {
            for step in 0..21 {
                let radius = 38.0 + f64::from(step) * 28.0;
                let angle = 1.62 * (radius / 24.0).ln() + f64::from(arm) * TAU / 3.0;
                let p = ellipse_point((900.0, 465.0), (radius, radius * 0.36), angle, -0.19);
                let color = if arm == 1 { "#718ba2" } else { "#8982ac" };
                glow(
                    &ctx,
                    p,
                    (65.0 + radius * 0.14, 28.0 + radius * 0.035),
                    color,
                    0.085 + (1.0 - radius / 650.0) * 0.08,
                )?;
            }
        }
        // Individual points supply photographic grain without an external texture.
        for i in 0..5900 {
            let arm = (i % 3) as f64;
            let radius = 18.0 + noise(i + 101).powf(0.68) * 650.0;
            let spread = (noise(i + 8801) + noise(i + 4901) - 1.0) * 0.43;
            let angle = 1.62 * (radius / 24.0).ln() + arm * TAU / 3.0 + spread;
            let p = ellipse_point((900.0, 465.0), (radius, radius * 0.36), angle, -0.19);
            let size = 0.35 + noise(i + 7603).powi(6) * 1.6;
            let density = 0.035 + noise(i + 201).powi(4) * 0.55;
            let color = if i % 11 == 0 {
                "#efdac3"
            } else if i % 5 == 0 {
                "#abd7e8"
            } else {
                "#c9c6e0"
            };
            ctx.set_fill_style_str(&tint(color, density));
            ctx.fill_rect(p.0, p.1, size, size);
        }
        // Narrow, dark dust lanes break up the luminous bands organically.
        for arm in 0..3 {
            let points: Vec<_> = (0..110)
                .map(|step| {
                    let radius = 90.0 + f64::from(step) * 4.6;
                    let angle = 1.62 * (radius / 24.0).ln() + f64::from(arm) * TAU / 3.0 + 0.14;
                    ellipse_point((900.0, 465.0), (radius, radius * 0.36), angle, -0.19)
                })
                .collect();
            line(&ctx, &points, "rgba(4,8,18,0.16)", 10.0);
            line(&ctx, &points, "rgba(4,8,18,0.13)", 3.0);
        }
        glow(&ctx, (900.0, 465.0), (210.0, 88.0), "#c6bed8", 0.26)?;
        glow(&ctx, (900.0, 465.0), (88.0, 34.0), "#eee0d7", 0.46)?;
        glow(&ctx, (900.0, 465.0), (24.0, 10.0), "#fff3e3", 0.64)?;
        for i in 0..640 {
            let p = (noise(i + 10001) * 1700.0, noise(i + 12001) * 950.0);
            let radius = 0.45 + noise(i + 9001).powi(9) * 1.65;
            let alpha = 0.08 + noise(i + 13001).powi(3) * 0.61;
            ctx.set_fill_style_str(&tint("#becbe0", alpha));
            ctx.fill_rect(p.0, p.1, radius, radius);
            if radius > 1.5 {
                glow(&ctx, p, (8.0, 8.0), ICE, 0.16)?;
            }
        }
        *stored.borrow_mut() = Some(canvas.clone());
        Ok(canvas)
    })
}

fn sky(ctx: &CanvasRenderingContext2d, time: f64) -> Result<(), JsValue> {
    let field = dust_field()?;
    ctx.set_fill_style_str(NIGHT);
    ctx.fill_rect(0.0, 0.0, 1200.0, 675.0);
    ctx.save();
    let result: Result<(), JsValue> = (|| {
        ctx.translate(
            600.0 + 9.0 * (time * 0.021).sin(),
            337.0 + 5.0 * (time * 0.018).cos(),
        )?;
        ctx.rotate(0.008 * (time * 0.015).sin())?;
        let zoom = 1.025 + 0.008 * (time * 0.022).sin();
        ctx.scale(zoom, zoom)?;
        ctx.draw_image_with_html_canvas_element(&field, -900.0, -465.0)?;
        Ok(())
    })();
    ctx.restore();
    result?;

    // Slow near-field particles make the camera feel present inside the galaxy.
    for i in 0..58 {
        let phase = (noise(i + 1703) + time * (0.002 + noise(i + 3201) * 0.002)).fract();
        let distance = 130.0 + phase.powf(1.5) * 700.0;
        let angle = noise(i + 7301) * TAU + time * 0.002;
        let p = ellipse_point((620.0, 330.0), (distance, distance * 0.65), angle, -0.15);
        let opacity = (phase * PI).sin().powi(2) * (0.25 + noise(i + 11) * 0.48);
        let color = tint(if i % 6 == 0 { "#e5cdad" } else { "#bad7ef" }, opacity);
        dot(ctx, p, 0.6 + phase * 1.1, &color);
        if phase > 0.72 {
            let direction = ((p.0 - 620.0) * 0.012, (p.1 - 330.0) * 0.012);
            line(
                ctx,
                &[(p.0 - direction.0, p.1 - direction.1), p],
                &tint(ICE, opacity * 0.28),
                0.65,
            );
        }
    }
    // These broad, translucent orbital traces share the racers' geometric frame.
    for lane in [-23.0, 0.0, 23.0] {
        let points: Vec<_> = (0..120)
            .map(|i| orbit(f64::from(i) / 119.0 * TAU / 4.08, lane, 154.0, false))
            .collect();
        line(ctx, &points, "rgba(176,176,216,0.045)", 4.5);
        line(ctx, &points, "rgba(184,189,225,0.10)", 0.65);
    }
    // Protect the text HUD without framing the scene in a visible box.
    let top = ctx.create_linear_gradient(0.0, 0.0, 0.0, 165.0);
    top.add_color_stop(0.0, "rgba(4,8,18,0.81)")?;
    top.add_color_stop(1.0, "rgba(4,8,18,0)")?;
    ctx.set_fill_style_canvas_gradient(&top);
    ctx.fill_rect(0.0, 0.0, 1200.0, 165.0);
    let bottom = ctx.create_linear_gradient(0.0, 490.0, 0.0, 675.0);
    bottom.add_color_stop(0.0, "rgba(4,8,18,0)")?;
    bottom.add_color_stop(1.0, "rgba(4,8,18,0.94)")?;
    ctx.set_fill_style_canvas_gradient(&bottom);
    ctx.fill_rect(0.0, 490.0, 1200.0, 185.0);
    Ok(())
}

#[derive(Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn intersects(self, other: Self, padding: f64) -> bool {
        self.x < other.x + other.w + padding
            && self.x + self.w + padding > other.x
            && self.y < other.y + other.h + padding
            && self.y + self.h + padding > other.y
    }
    fn dock(self, p: Point) -> Point {
        (
            p.0.clamp(self.x, self.x + self.w),
            p.1.clamp(self.y + 9.0, self.y + self.h - 8.0),
        )
    }
}

struct Node {
    index: usize,
    point: Point,
    progress: f64,
    lane: f64,
    label: Option<Rect>,
}

fn score_window(racers: &[Racer<'_>]) -> (f64, f64) {
    let scores: Vec<_> = racers
        .iter()
        .filter(|r| r.rank > 0)
        .filter_map(|r| r.score.filter(|score| score.is_finite()))
        .map(|score| score / 100.0)
        .collect();
    if scores.is_empty() {
        return (0.0, 1.0);
    }
    let low = scores.iter().copied().fold(1.0, f64::min);
    let high = scores.iter().copied().fold(0.0, f64::max);
    let padding = ((high - low) * 0.18).max(0.026);
    let a = (low - padding).max(0.0);
    let b = (high + padding).min(1.0);
    (a, (b - a).max(0.052))
}

fn arrival(appearance: f64, lane: f64) -> Point {
    let p = appearance.clamp(0.0, 1.0);
    let smooth = p * p * (3.0 - 2.0 * p);
    (135.0 + 245.0 * smooth, 470.0 + 43.0 * smooth + lane * 0.18)
}

fn place_labels(nodes: &mut [Node], racers: &[Racer<'_>]) {
    let occupied_hud = [
        Rect {
            x: 45.0,
            y: 70.0,
            w: 370.0,
            h: 167.0,
        },
        Rect {
            x: 935.0,
            y: 65.0,
            w: 250.0,
            h: 86.0,
        },
    ];
    let mut used: Vec<Rect> = Vec::new();
    let mut order: Vec<_> = (0..nodes.len()).collect();
    order.sort_by_key(|i| {
        if racers[nodes[*i].index].rank == 0 {
            99
        } else {
            racers[nodes[*i].index].rank
        }
    });
    for index in order.into_iter().take(6) {
        let node = &nodes[index];
        let p = node.point;
        let offsets = [
            (27.0, -50.0),
            (-268.0, -50.0),
            (-105.0, -88.0),
            (-92.0, 35.0),
            (-267.0, 29.0),
            (32.0, 25.0),
        ];
        let mut candidates: Vec<_> = offsets
            .iter()
            .map(|offset| Rect {
                x: (p.0 + offset.0).clamp(65.0, 893.0),
                y: (p.1 + offset.1).clamp(106.0, 520.0),
                w: 242.0,
                h: 49.0,
            })
            .collect();
        candidates.extend(
            [
                (720.0, 124.0),
                (840.0, 460.0),
                (500.0, 510.0),
                (95.0, 397.0),
                (439.0, 145.0),
                (852.0, 304.0),
            ]
            .map(|(x, y)| Rect {
                x,
                y,
                w: 242.0,
                h: 49.0,
            }),
        );
        let fits = |candidate: &Rect| {
            !occupied_hud
                .iter()
                .any(|hud| candidate.intersects(*hud, 5.0))
                && !used.iter().any(|other| candidate.intersects(*other, 13.0))
                && !nodes.iter().any(|other| {
                    candidate.intersects(
                        Rect {
                            x: other.point.0 - 17.0,
                            y: other.point.1 - 17.0,
                            w: 34.0,
                            h: 34.0,
                        },
                        5.0,
                    )
                })
        };
        if let Some(candidate) = candidates
            .iter()
            .filter(|candidate| fits(candidate))
            .min_by(|a, b| {
                let score = |rect: &Rect| {
                    let dock = rect.dock(p);
                    (dock.0 - p.0).powi(2) + (dock.1 - p.1).powi(2)
                };
                score(a).total_cmp(&score(b))
            })
            .copied()
        {
            nodes[index].label = Some(candidate);
            used.push(candidate);
        }
    }
}

fn nodes(racers: &[Racer<'_>], time: f64, compact: bool) -> Vec<Node> {
    let (floor, span) = score_window(racers);
    let mut result: Vec<_> = racers
        .iter()
        .enumerate()
        .take(8)
        .filter(|(_, r)| r.appeared > 0.001)
        .map(|(index, racer)| {
            let seed = identity(racer.name);
            let lane = (noise(seed) - 0.5) * 56.0;
            let position = if racer.position.is_finite() {
                racer.position
            } else {
                0.0
            };
            let progress = ((position - floor) / span).clamp(0.0, 1.0);
            let point = if racer.rank == 0 {
                if compact {
                    let p = arrival(racer.appeared, lane);
                    (p.0 * 0.48 + 50.0, p.1 * 0.44 + 122.0)
                } else {
                    arrival(racer.appeared, lane)
                }
            } else {
                let p = orbit(progress, lane, time, compact);
                // New releases enter from the lower left instead of blinking on.
                let fade = racer.appeared.clamp(0.0, 1.0);
                let entry = (1.0 - fade).powi(3);
                (
                    p.0 - entry * if compact { 160.0 } else { 290.0 },
                    p.1 + entry * 75.0,
                )
            };
            Node {
                index,
                point,
                progress,
                lane,
                label: None,
            }
        })
        .collect();
    if compact {
        place_compact_labels(&mut result, racers);
    } else {
        place_labels(&mut result, racers);
    }
    result
}

fn comet(
    ctx: &CanvasRenderingContext2d,
    node: &Node,
    racer: &Racer<'_>,
    time: f64,
    compact: bool,
) -> Result<(), JsValue> {
    let color = racer.color;
    let p = node.point;
    if racer.rank > 0 {
        let tail_length = 0.145;
        let path: Vec<_> = (0..29)
            .map(|i| {
                orbit(
                    node.progress - tail_length + f64::from(i) / 28.0 * tail_length,
                    node.lane,
                    time,
                    compact,
                )
            })
            .collect();
        line(ctx, &path, &tint(color, 0.025), 13.0);
        line(ctx, &path, &tint(color, 0.055), 6.5);
        for (i, pair) in path.windows(2).enumerate() {
            let q = (i + 1) as f64 / 28.0;
            let shimmer = 0.8 + 0.2 * (q * 20.0 - time * 3.0).sin();
            line(
                ctx,
                pair,
                &tint(color, q.powi(2) * 0.72 * shimmer),
                0.6 + q * 1.8,
            );
        }
        // Small motes stream into the head along the same trajectory.
        for i in 0..3 {
            let q = (time * 0.34 + f64::from(i) / 3.0 + noise(node.index + 701)).fract();
            let dust = orbit(
                node.progress - tail_length * (1.0 - q),
                node.lane,
                time,
                compact,
            );
            dot(ctx, dust, 0.8, &tint(color, q * 0.65));
        }
    } else {
        let path: Vec<_> = (0..35)
            .map(|i| {
                let q = f64::from(i) / 34.0;
                (p.0 - 145.0 + q * 145.0, p.1 + (1.0 - q).powi(2) * 24.0)
            })
            .collect();
        line(ctx, &path, &tint(color, 0.19), 1.0);
    }
    glow(ctx, p, (48.0, 33.0), color, 0.26)?;
    glow(ctx, p, (18.0, 18.0), color, 0.66)?;
    let core = if racer.rank == 1 { 5.1 } else { 4.0 };
    dot(ctx, p, core, color);
    dot(ctx, (p.0 - 0.8, p.1 - 0.6), core * 0.56, "#fffaf3");
    line(
        ctx,
        &[(p.0 - 22.0, p.1), (p.0 + 22.0, p.1)],
        &tint(color, 0.30),
        0.55,
    );
    if racer.rank == 1 {
        line(
            ctx,
            &[(p.0, p.1 - 14.0), (p.0, p.1 + 14.0)],
            "rgba(255,241,223,0.33)",
            0.5,
        );
    }
    Ok(())
}

fn label(ctx: &CanvasRenderingContext2d, node: &Node, racer: &Racer<'_>) -> Result<(), JsValue> {
    let Some(rect) = node.label else {
        return Ok(());
    };
    let dock = rect.dock(node.point);
    let midpoint = ((dock.0 + node.point.0) / 2.0, (dock.1 + node.point.1) / 2.0);
    ctx.begin_path();
    ctx.move_to(node.point.0, node.point.1);
    ctx.bezier_curve_to(midpoint.0, node.point.1, midpoint.0, dock.1, dock.0, dock.1);
    ctx.set_stroke_style_str(&tint(racer.color, 0.28));
    ctx.set_line_width(0.75);
    ctx.stroke();
    dot(ctx, dock, 1.7, &tint(racer.color, 0.70));
    // A soft local shadow grounds typography without rectangular label cards.
    glow(
        ctx,
        (rect.x + 118.0, rect.y + 23.0),
        (155.0, 38.0),
        NIGHT,
        0.85,
    )?;
    ctx.set_text_align("left");
    ctx.set_text_baseline("middle");
    ctx.set_font("500 17px system-ui, -apple-system, sans-serif");
    ctx.set_fill_style_str("#f1f0f8");
    ctx.fill_text_with_max_width(racer.name, rect.x, rect.y + 15.0, 238.0)?;
    ctx.set_font("500 9px system-ui, sans-serif");
    ctx.set_fill_style_str("#a5acc2");
    ctx.fill_text_with_max_width(
        &racer.organization.to_uppercase(),
        rect.x,
        rect.y + 35.0,
        163.0,
    )?;
    if racer.rank > 0 {
        ctx.set_font("500 10px ui-monospace, monospace");
        ctx.set_fill_style_str(&tint(racer.color, 0.82));
        ctx.fill_text(&format!("{:02}", racer.rank), rect.x - 23.0, rect.y + 15.0)?;
        if let Some(score) = racer.score.filter(|score| score.is_finite()) {
            ctx.set_text_align("right");
            ctx.set_font("500 12px ui-monospace, monospace");
            ctx.fill_text(&format!("{score:.1}"), rect.x + 238.0, rect.y + 35.0)?;
        }
    }
    Ok(())
}

fn place_compact_labels(nodes: &mut [Node], racers: &[Racer<'_>]) {
    // Broad, separate annotations retain their size on a phone. The central
    // galaxy stays open; thin leaders connect names to the moving comets.
    let slots = [
        Rect {
            x: 32.0,
            y: 48.0,
            w: 246.0,
            h: 87.0,
        },
        Rect {
            x: 328.0,
            y: 48.0,
            w: 246.0,
            h: 87.0,
        },
        Rect {
            x: 32.0,
            y: 318.0,
            w: 246.0,
            h: 87.0,
        },
        Rect {
            x: 328.0,
            y: 318.0,
            w: 246.0,
            h: 87.0,
        },
        Rect {
            x: 175.0,
            y: 182.0,
            w: 246.0,
            h: 87.0,
        },
    ];
    let mut used = [false; 5];
    let mut order: Vec<_> = (0..nodes.len()).collect();
    order.sort_by_key(|i| {
        let rank = racers[nodes[*i].index].rank;
        if rank == 0 { 99 } else { rank }
    });
    for index in order.into_iter().take(5) {
        let p = nodes[index].point;
        let best = slots
            .iter()
            .enumerate()
            .filter(|(i, _)| !used[*i])
            .min_by(|(_, a), (_, b)| {
                let cost = |rect: &Rect| {
                    let dock = rect.dock(p);
                    let distance = (dock.0 - p.0).powi(2) + (dock.1 - p.1).powi(2);
                    let overlap = nodes
                        .iter()
                        .filter(|node| {
                            rect.intersects(
                                Rect {
                                    x: node.point.0 - 10.0,
                                    y: node.point.1 - 10.0,
                                    w: 20.0,
                                    h: 20.0,
                                },
                                7.0,
                            )
                        })
                        .count();
                    distance + overlap as f64 * 95_000.0
                };
                cost(a).total_cmp(&cost(b))
            })
            .map(|(i, _)| i);
        if let Some(slot) = best {
            used[slot] = true;
            nodes[index].label = Some(slots[slot]);
        }
    }
}

fn compact_name(name: &str) -> Vec<String> {
    // A release suffix remains in the DOM story, while the plotted name keeps
    // its distinguishing model family/version and wraps without tiny text.
    let name = name.split('·').next().unwrap_or(name).trim();
    let chars: Vec<_> = name.chars().collect();
    if chars.len() <= 17 {
        return vec![name.to_owned()];
    }
    let split = chars
        .iter()
        .enumerate()
        .take(18)
        .skip(7)
        .filter(|(_, c)| **c == ' ' || **c == '-')
        .map(|(i, c)| if *c == '-' { i + 1 } else { i })
        .next_back()
        .unwrap_or(17);
    vec![
        chars[..split].iter().collect::<String>().trim().to_owned(),
        chars[split..].iter().collect::<String>().trim().to_owned(),
    ]
}

fn compact_label(
    ctx: &CanvasRenderingContext2d,
    node: &Node,
    racer: &Racer<'_>,
) -> Result<(), JsValue> {
    let Some(rect) = node.label else {
        return Ok(());
    };
    let dock = rect.dock(node.point);
    let middle = (node.point.0 + dock.0) * 0.5;
    ctx.begin_path();
    ctx.move_to(node.point.0, node.point.1);
    ctx.bezier_curve_to(middle, node.point.1, middle, dock.1, dock.0, dock.1);
    ctx.set_stroke_style_str(&tint(racer.color, 0.36));
    ctx.set_line_width(1.0);
    ctx.stroke();
    glow(
        ctx,
        (rect.x + 122.0, rect.y + 42.0),
        (150.0, 64.0),
        NIGHT,
        0.93,
    )?;
    let lines = compact_name(racer.name);
    ctx.set_text_align("left");
    ctx.set_text_baseline("middle");
    ctx.set_font("500 26px system-ui, -apple-system, sans-serif");
    ctx.set_fill_style_str("#f2f0fa");
    for (i, name) in lines.iter().take(2).enumerate() {
        ctx.fill_text_with_max_width(name, rect.x, rect.y + 17.0 + i as f64 * 29.0, 246.0)?;
    }
    let baseline = if lines.len() > 1 { 75.0 } else { 49.0 };
    ctx.set_font("500 18px system-ui, sans-serif");
    ctx.set_fill_style_str("#b6bdd4");
    let organization = racer.organization.split_once('·');
    let subtitle = if racer.rank == 0 {
        organization.map_or(racer.organization, |(_, status)| status)
    } else {
        organization.map_or(racer.organization, |(name, _)| name)
    };
    ctx.fill_text_with_max_width(
        subtitle.trim(),
        rect.x,
        rect.y + baseline,
        if racer.rank == 0 { 246.0 } else { 164.0 },
    )?;
    if racer.rank > 0 {
        ctx.set_font("500 17px ui-monospace, monospace");
        ctx.set_fill_style_str(&tint(racer.color, 0.94));
        ctx.fill_text(&format!("{:02}", racer.rank), rect.x - 25.0, rect.y + 16.0)?;
        if let Some(score) = racer.score.filter(|score| score.is_finite()) {
            ctx.set_text_align("right");
            ctx.set_font("500 22px ui-monospace, monospace");
            ctx.fill_text(&format!("{score:.1}"), rect.x + 246.0, rect.y + baseline)?;
        }
    }
    Ok(())
}

/// A separately composed 600 x 500 plot, with readable type and no cropped racers.
pub fn draw_compact(
    ctx: &CanvasRenderingContext2d,
    time: f64,
    racers: &[Racer<'_>],
) -> Result<(), JsValue> {
    if !time.is_finite() {
        return Err(JsValue::from_str("Animation time must be finite"));
    }
    let time = time.max(0.0);
    ctx.save();
    ctx.set_global_alpha(1.0);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    let result: Result<(), JsValue> = (|| {
        // Reuse only the procedural star texture, not desktop geometry or type.
        ctx.set_fill_style_str(NIGHT);
        ctx.fill_rect(0.0, 0.0, 600.0, 500.0);
        ctx.save();
        let background: Result<(), JsValue> = (|| {
            ctx.translate(300.0 + 4.0 * (time * 0.021).sin(), 235.0)?;
            ctx.rotate(0.009 * (time * 0.018).sin())?;
            ctx.scale(0.57, 0.72)?;
            ctx.draw_image_with_html_canvas_element(&dust_field()?, -900.0, -465.0)?;
            Ok(())
        })();
        ctx.restore();
        background?;
        for lane in [-22.0, 0.0, 22.0] {
            let points: Vec<_> = (0..120)
                .map(|i| orbit(f64::from(i) / 119.0 * TAU / 4.08, lane, 154.0, true))
                .collect();
            line(ctx, &points, "rgba(192,187,229,0.085)", 1.1);
        }
        for i in 0..30 {
            let phase = (noise(i + 1703) + time * (0.003 + noise(i + 3201) * 0.002)).fract();
            let angle = noise(i + 7301) * TAU + time * 0.002;
            let radius = 60.0 + phase.powf(1.5) * 370.0;
            let p = ellipse_point((300.0, 231.0), (radius, radius * 0.68), angle, -0.18);
            dot(
                ctx,
                p,
                0.65 + phase,
                &tint(ICE, (phase * PI).sin().powi(2) * 0.55),
            );
        }
        let nodes = nodes(racers, time, true);
        for node in &nodes {
            let racer = &racers[node.index];
            ctx.set_global_alpha(racer.appeared.clamp(0.0, 1.0));
            comet(ctx, node, racer, time, true)?;
        }
        for node in &nodes {
            let racer = &racers[node.index];
            ctx.set_global_alpha(racer.appeared.clamp(0.0, 1.0));
            compact_label(ctx, node, racer)?;
        }
        Ok(())
    })();
    ctx.restore();
    result
}

/// Immersive 1200×675 space stage, with top 70 and bottom 90 reserved for HUD.
/// A shared focused score window expands close results; no score or rank is inferred.
/// All heads travel forward together; scores determine their relative lead.
pub fn draw(
    ctx: &CanvasRenderingContext2d,
    time: f64,
    racers: &[Racer<'_>],
) -> Result<(), JsValue> {
    if !time.is_finite() {
        return Err(JsValue::from_str("Animation time must be finite"));
    }
    let time = time.max(0.0);
    ctx.save();
    ctx.set_global_alpha(1.0);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    let result: Result<(), JsValue> = (|| {
        sky(ctx, time)?;
        let nodes = nodes(racers, time, false);
        for node in &nodes {
            let racer = &racers[node.index];
            ctx.set_global_alpha(racer.appeared.clamp(0.0, 1.0));
            comet(ctx, node, racer, time, false)?;
        }
        for node in &nodes {
            let racer = &racers[node.index];
            ctx.set_global_alpha(racer.appeared.clamp(0.0, 1.0));
            label(ctx, node, racer)?;
        }
        Ok(())
    })();
    ctx.restore();
    result
}
