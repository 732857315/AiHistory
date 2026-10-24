//! Deterministic live engravings and a score-driven galaxy, drawn by Rust/WASM.
//! There are no images, encoded media, frame sequences, or random state.
use std::f64::consts::{PI, TAU};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

type Point = (f64, f64);
type Curve = [Point; 4];
const PAPER: &str = "#f1e8d6";
const INK: &str = "#302d28";
const PRINT: &str = "#786d5c";
const PALE: &str = "#d9cdb5";
const ACCENT: &str = "#d87935";
const CYAN: &str = "#5de5e7";
const PURPLE: &str = "#ba9bff";
const GOLD: &str = "#ffd489";
const NIGHT: &str = "#080f24";

/// Positions, rankings and scores come from the application's benchmark engine.
pub struct Racer<'a> {
    pub name: &'a str,
    pub organization: &'a str,
    pub color: &'a str,
    pub score: Option<f64>,
    pub position: f64,
    pub rank: usize,
    pub appeared: f64,
}

fn noise(seed: usize) -> f64 {
    let mut value = (seed as u32)
        .wrapping_mul(747_796_405)
        .wrapping_add(2_891_336_453);
    value = ((value >> ((value >> 28) + 4)) ^ value).wrapping_mul(277_803_737);
    f64::from((value >> 22) ^ value) / f64::from(u32::MAX)
}

fn bezier([a, b, c, d]: Curve, t: f64) -> Point {
    let q = 1.0 - t;
    (
        q.powi(3) * a.0 + 3.0 * q * q * t * b.0 + 3.0 * q * t * t * c.0 + t.powi(3) * d.0,
        q.powi(3) * a.1 + 3.0 * q * q * t * b.1 + 3.0 * q * t * t * c.1 + t.powi(3) * d.1,
    )
}

struct Painter<'a>(&'a CanvasRenderingContext2d);
impl Painter<'_> {
    fn finish(&self, fill: Option<&str>, stroke: Option<&str>, width: f64) {
        if let Some(color) = fill {
            self.0.set_fill_style_str(color);
            self.0.fill();
        }
        if let Some(color) = stroke {
            self.0.set_stroke_style_str(color);
            self.0.set_line_width(width);
            self.0.stroke();
        }
    }
    fn ellipse(&self, p: Point, r: Point, fill: Option<&str>, stroke: Option<&str>, width: f64) {
        let (x, y) = p;
        let (rx, ry) = r;
        let k = 0.552_284_749_8;
        self.0.begin_path();
        self.0.move_to(x + rx, y);
        self.0
            .bezier_curve_to(x + rx, y + ry * k, x + rx * k, y + ry, x, y + ry);
        self.0
            .bezier_curve_to(x - rx * k, y + ry, x - rx, y + ry * k, x - rx, y);
        self.0
            .bezier_curve_to(x - rx, y - ry * k, x - rx * k, y - ry, x, y - ry);
        self.0
            .bezier_curve_to(x + rx * k, y - ry, x + rx, y - ry * k, x + rx, y);
        self.0.close_path();
        self.finish(fill, stroke, width);
    }
    fn dot(&self, p: Point, r: f64, color: &str) {
        self.ellipse(p, (r, r), Some(color), None, 0.0);
    }
    fn ring(&self, p: Point, r: f64, color: &str, width: f64) {
        self.ellipse(p, (r, r), None, Some(color), width);
    }
    fn rect(&self, b: [f64; 4], radius: f64, color: &str, stroke: Option<&str>) {
        let [x, y, right, bottom] = b;
        let r = radius
            .min((right - x) / 2.0)
            .min((bottom - y) / 2.0)
            .max(0.0);
        self.0.begin_path();
        self.0.move_to(x + r, y);
        self.0.line_to(right - r, y);
        self.0.quadratic_curve_to(right, y, right, y + r);
        self.0.line_to(right, bottom - r);
        self.0.quadratic_curve_to(right, bottom, right - r, bottom);
        self.0.line_to(x + r, bottom);
        self.0.quadratic_curve_to(x, bottom, x, bottom - r);
        self.0.line_to(x, y + r);
        self.0.quadratic_curve_to(x, y, x + r, y);
        self.0.close_path();
        self.finish(Some(color), stroke, 2.0);
    }
    fn line(&self, points: &[Point], color: &str, width: f64) {
        if let Some(p) = points.first() {
            self.0.begin_path();
            self.0.move_to(p.0, p.1);
            for p in &points[1..] {
                self.0.line_to(p.0, p.1);
            }
            self.finish(None, Some(color), width);
        }
    }
    fn polygon(&self, points: &[Point], color: &str, stroke: Option<&str>) {
        if let Some(p) = points.first() {
            self.0.begin_path();
            self.0.move_to(p.0, p.1);
            for p in &points[1..] {
                self.0.line_to(p.0, p.1);
            }
            self.0.close_path();
            self.finish(Some(color), stroke, 2.0);
        }
    }
    fn curve(&self, [a, b, c, d]: Curve, color: &str, width: f64) {
        self.0.begin_path();
        self.0.move_to(a.0, a.1);
        self.0.bezier_curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
        self.finish(None, Some(color), width);
    }
    fn faded(&self, opacity: f64, paint: impl FnOnce()) {
        self.0.save();
        self.0
            .set_global_alpha(self.0.global_alpha() * opacity.clamp(0.0, 1.0));
        paint();
        self.0.restore();
    }
    fn spark(&self, p: Point, radius: f64, color: &str, angle: f64) {
        let points: Vec<_> = (0..8)
            .map(|i| {
                let a = f64::from(i) * PI / 4.0 + angle;
                let r = if i % 2 == 0 { radius } else { radius * 0.22 };
                (p.0 + a.cos() * r, p.1 + a.sin() * r)
            })
            .collect();
        self.polygon(&points, color, None);
    }
    fn signal(&self, path: Curve, phase: f64, color: &str, radius: f64) {
        let p = bezier(path, phase.rem_euclid(1.0));
        self.faded(0.14, || self.dot(p, radius + 8.0, color));
        self.dot(p, radius, color);
    }
    fn hatch(&self, b: [f64; 4], spacing: f64, color: &str) {
        let [x, y, right, bottom] = b;
        let mut yy = y;
        while yy < bottom {
            self.line(&[(x, yy), (right, (yy - 12.0).max(y))], color, 0.7);
            yy += spacing;
        }
    }
    fn text(
        &self,
        value: &str,
        p: Point,
        size: f64,
        color: &str,
        max_width: f64,
    ) -> Result<(), JsValue> {
        self.0.set_font(&format!(
            "500 {size}px system-ui, -apple-system, sans-serif"
        ));
        self.0.set_fill_style_str(color);
        self.0.set_text_align("left");
        self.0.set_text_baseline("middle");
        self.0.fill_text_with_max_width(value, p.0, p.1, max_width)
    }
}

// Engraving primitives deliberately use visible marks, rather than bitmap grain.
fn plate(c: &Painter<'_>, b: [f64; 4], angle: f64, paint: impl FnOnce()) {
    let [x, y, right, bottom] = b;
    c.0.save();
    let _ = c.0.translate((x + right) * 0.5, (y + bottom) * 0.5);
    let _ = c.0.rotate(angle);
    let _ = c.0.translate(-(x + right) * 0.5, -(y + bottom) * 0.5);
    c.0.set_shadow_color("rgba(45,31,18,.15)");
    c.0.set_shadow_blur(18.0);
    c.0.set_shadow_offset_y(9.0);
    c.rect(b, 1.0, "#f4ebd8", None);
    c.0.set_shadow_color("transparent");
    c.faded(0.25, || {
        c.rect(
            [x + 8.0, y + 8.0, right - 8.0, bottom - 8.0],
            0.0,
            "rgba(255,255,255,0)",
            Some(PRINT),
        );
        for row in 0..7 {
            let yy = bottom - 25.0 + f64::from(row) * 1.6;
            c.line(
                &[
                    (x + 18.0, yy),
                    (right - 22.0 - f64::from(row % 3) * 9.0, yy),
                ],
                PRINT,
                0.4,
            );
        }
    });
    paint();
    c.0.restore();
}

fn ornament(c: &Painter<'_>, p: Point, radius: f64, color: &str) {
    c.ring(p, radius, color, 0.8);
    for i in 0..24 {
        let a = f64::from(i) * TAU / 24.0;
        let inner = radius - if i % 3 == 0 { 8.0 } else { 3.0 };
        c.line(
            &[
                (p.0 + a.cos() * inner, p.1 + a.sin() * inner),
                (p.0 + a.cos() * radius, p.1 + a.sin() * radius),
            ],
            color,
            0.8,
        );
    }
    for i in 0..4 {
        let a = f64::from(i) * PI / 2.0;
        c.line(
            &[
                (
                    p.0 + a.cos() * (radius + 4.0),
                    p.1 + a.sin() * (radius + 4.0),
                ),
                (
                    p.0 + a.cos() * (radius + 13.0),
                    p.1 + a.sin() * (radius + 13.0),
                ),
            ],
            color,
            0.8,
        );
    }
}

fn paper(c: &Painter<'_>, t: f64) {
    c.rect([0.0, 0.0, 1200.0, 675.0], 0.0, "#e8dcc4", None);
    if let Ok(g) =
        c.0.create_radial_gradient(650.0, 270.0, 30.0, 650.0, 300.0, 760.0)
    {
        let _ = g.add_color_stop(0.0, "#fbf4e5");
        let _ = g.add_color_stop(0.65, "#efe2c9");
        let _ = g.add_color_stop(1.0, "#d5bc93");
        c.0.set_fill_style_canvas_gradient(&g);
        c.0.fill_rect(0.0, 0.0, 1200.0, 675.0);
    }
    // Old survey contours and fine metal-type marks make a quiet historical field.
    c.faded(0.12, || {
        for ring in 0..17 {
            let r = 136.0 + f64::from(ring) * 17.0;
            let points: Vec<_> = (0..100)
                .map(|i| {
                    let a = f64::from(i) * TAU / 99.0;
                    let v =
                        r + 12.0 * (a * 5.0 + f64::from(ring) * 0.19).sin() + 7.0 * (a * 9.0).cos();
                    (806.0 + a.cos() * v, 333.0 + a.sin() * v * 0.70)
                })
                .collect();
            c.line(&points, PRINT, 0.5);
        }
        for col in 0..7 {
            for row in 0..15 {
                let x = 466.0 + f64::from(col) * 90.0;
                let y = 91.0 + f64::from(row) * 4.5;
                c.line(
                    &[
                        (x, y),
                        (x + 65.0 + noise((col * 21 + row) as usize) * 14.0, y),
                    ],
                    PRINT,
                    0.45,
                );
            }
        }
    });
    c.faded(0.22, || {
        for i in 0..450 {
            let p = (noise(i + 73) * 1200.0, noise(i + 919) * 675.0);
            c.line(
                &[p, (p.0 + 0.6 + noise(i + 641) * 2.5, p.1 + 0.25)],
                PRINT,
                0.4,
            );
        }
    });
    c.faded(0.22, || {
        for r in [219.0, 225.0, 248.0] {
            c.ring((793.0, 324.0), r, PRINT, 0.6);
        }
        for i in 0..100 {
            let a = f64::from(i) * TAU / 100.0;
            let r = if i % 5 == 0 { 237.0 } else { 243.0 };
            c.line(
                &[
                    (793.0 + a.cos() * r, 324.0 + a.sin() * r),
                    (793.0 + a.cos() * 248.0, 324.0 + a.sin() * 248.0),
                ],
                PRINT,
                0.7,
            );
        }
        ornament(c, (1091.0, 153.0), 28.0, PRINT);
        c.line(&[(430.0, 557.0), (1135.0, 557.0)], PRINT, 0.7);
        for i in 0..42 {
            let x = 430.0 + f64::from(i) * 17.0;
            c.line(
                &[(x, 557.0), (x, 557.0 + if i % 5 == 0 { 9.0 } else { 4.0 })],
                PRINT,
                0.7,
            );
        }
    });
    // This line persists between eras: a visual thread rather than a cut to black.
    let path = [
        (410.0, 542.0),
        (610.0, 582.0),
        (922.0, 516.0),
        (1135.0, 542.0),
    ];
    c.faded(0.22, || c.curve(path, ACCENT, 1.1));
    c.faded(0.6, || c.signal(path, t / 31.0, ACCENT, 2.0));
    // Protect the title area without putting the illustration into a card.
    let g = c.0.create_linear_gradient(20.0, 0.0, 540.0, 0.0);
    let _ = g.add_color_stop(0.0, "rgba(243,233,212,.92)");
    let _ = g.add_color_stop(0.72, "rgba(243,233,212,.72)");
    let _ = g.add_color_stop(1.0, "rgba(243,233,212,0)");
    c.0.set_fill_style_canvas_gradient(&g);
    c.0.fill_rect(0.0, 70.0, 540.0, 510.0);
}

fn tools(c: &Painter<'_>, t: f64) {
    plate(c, [511.0, 190.0, 915.0, 487.0], -0.12, || {
        c.faded(0.42, || {
            for line in 0..14 {
                let y = 219.0 + f64::from(line) * 15.0;
                c.curve(
                    [
                        (537.0, y),
                        (596.0, y - 31.0),
                        (728.0, y + 45.0),
                        (889.0, y - 8.0),
                    ],
                    PRINT,
                    0.6,
                );
            }
            // A drawn stratigraphic section, not a fabricated artifact label.
            c.line(
                &[
                    (546.0, 249.0),
                    (599.0, 225.0),
                    (642.0, 258.0),
                    (699.0, 227.0),
                    (749.0, 252.0),
                    (883.0, 233.0),
                ],
                INK,
                1.1,
            );
        });
    });
    plate(c, [904.0, 322.0, 1104.0, 513.0], 0.13, || {
        c.faded(0.6, || {
            ornament(c, (999.0, 407.0), 53.0, PRINT);
            let shape = [
                (975.0, 365.0),
                (1032.0, 381.0),
                (1028.0, 429.0),
                (991.0, 449.0),
                (968.0, 415.0),
            ];
            c.polygon(&shape, PALE, Some(PRINT));
            c.line(
                &[(975.0, 365.0), (1000.0, 404.0), (1028.0, 429.0)],
                PRINT,
                1.0,
            );
            c.line(&[(1000.0, 404.0), (968.0, 415.0)], PRINT, 0.8);
        });
    });
    c.faded(0.11, || {
        c.ellipse((780.0, 513.0), (170.0, 19.0), Some(INK), None, 0.0)
    });
    let lift = (t * 0.65).sin() * 5.0;
    c.0.save();
    let _ = c.0.translate(786.0, 317.0 + lift);
    let _ = c.0.rotate(-0.075 + (t * 0.22).sin() * 0.018);
    let outline = [
        (-106.0, -90.0),
        (-82.0, -138.0),
        (-39.0, -155.0),
        (5.0, -142.0),
        (48.0, -151.0),
        (92.0, -112.0),
        (101.0, -77.0),
        (131.0, -42.0),
        (116.0, -4.0),
        (134.0, 32.0),
        (112.0, 72.0),
        (96.0, 116.0),
        (47.0, 143.0),
        (3.0, 160.0),
        (-47.0, 145.0),
        (-87.0, 117.0),
        (-108.0, 69.0),
        (-124.0, 25.0),
        (-112.0, -18.0),
        (-126.0, -55.0),
    ];
    c.0.set_shadow_color("rgba(48,32,18,.32)");
    c.0.set_shadow_blur(29.0);
    c.0.set_shadow_offset_x(13.0);
    c.0.set_shadow_offset_y(20.0);
    c.polygon(&outline, "#504d43", Some("#3a3931"));
    c.0.set_shadow_color("transparent");
    let centers = [(-28.0, -49.0), (39.0, 12.0), (-39.0, 67.0)];
    let stone = [
        "#777465", "#aaa58d", "#5b5d53", "#90917d", "#b0aa91", "#6a6d60", "#414a43",
    ];
    for i in 0..outline.len() {
        let a = outline[i];
        let b = outline[(i + 1) % outline.len()];
        let center = centers[if i < 7 {
            0
        } else if i < 13 {
            1
        } else {
            2
        }];
        let points = [a, b, center];
        c.polygon(&points, stone[(i * 3 + 1) % stone.len()], None);
        c.0.save();
        c.0.clip();
        c.faded(0.35, || {
            // Conchoidal ripples and fine knapping striations follow each scar.
            for ring in 0..13 {
                let r = 10.0 + f64::from(ring) * 12.0;
                c.ellipse(
                    (a.0 + 12.0, a.1 + 13.0),
                    (r, r * 0.70),
                    None,
                    Some("#ded8bc"),
                    0.8,
                );
                c.ellipse(
                    (a.0 + 11.0, a.1 + 15.0),
                    (r + 2.0, r * 0.70),
                    None,
                    Some("#292f29"),
                    0.5,
                );
            }
        });
        c.0.restore();
        c.faded(0.48, || c.line(&[a, center, b], "#d2cdb5", 0.8));
    }
    c.polygon(&centers, "#858675", Some("#b0ad92"));
    c.0.save();
    c.polygon(&outline, "rgba(0,0,0,0)", None);
    c.0.clip();
    for i in 0..260 {
        let p = (
            -132.0 + noise(i + 3160) * 270.0,
            -160.0 + noise(i + 3619) * 325.0,
        );
        c.faded(0.15 + noise(i + 83) * 0.3, || {
            c.line(
                &[p, (p.0 + noise(i + 166) * 3.0, p.1 - 0.6)],
                if i % 3 == 0 { "#e2dbbd" } else { "#30392e" },
                0.6,
            )
        });
    }
    c.0.restore();
    c.faded(0.7, || c.line(&outline[..6], "#dcd2b2", 1.4));
    c.0.restore();
    // Small detached flakes make the transformation from stone to tool legible.
    for (i, (x, y, r)) in [
        (567.0, 457.0, 20.0),
        (963.0, 241.0, 16.0),
        (1036.0, 287.0, 10.0),
    ]
    .into_iter()
    .enumerate()
    {
        let dy = (t * 0.75 + i as f64).sin() * 3.0;
        c.polygon(
            &[
                (x - r, y + dy),
                (x + r * 0.5, y - r + dy),
                (x + r, y + r * 0.7 + dy),
            ],
            "#8b8a73",
            Some(PRINT),
        );
        c.line(
            &[
                (x - r, y + dy),
                (x + r * 0.2, y + dy),
                (x + r * 0.5, y - r + dy),
            ],
            "#cfc9ac",
            0.8,
        );
    }
    c.faded(0.65, || {
        c.line(
            &[(909.0, 190.0), (1009.0, 177.0), (1070.0, 177.0)],
            PRINT,
            0.7,
        );
        c.dot((909.0, 190.0), 2.0, ACCENT);
        c.line(
            &[(652.0, 410.0), (573.0, 508.0), (509.0, 508.0)],
            PRINT,
            0.7,
        );
        c.dot((652.0, 410.0), 2.0, ACCENT);
    });
}

fn waterclock(c: &Painter<'_>, t: f64) {
    c.faded(0.09, || {
        c.ellipse((609.0, 510.0), (254.0, 20.0), Some(INK), None, 0.0)
    });
    c.polygon(
        &[
            (415.0, 137.0),
            (794.0, 137.0),
            (737.0, 448.0),
            (472.0, 448.0),
        ],
        PALE,
        Some(INK),
    );
    c.ellipse((604.0, 137.0), (189.0, 35.0), Some(PAPER), Some(INK), 2.5);
    c.ellipse(
        (604.0, 146.0),
        (169.0, 21.0),
        Some("#c5b99f"),
        Some(PRINT),
        1.0,
    );
    c.curve(
        [
            (472.0, 448.0),
            (510.0, 476.0),
            (704.0, 476.0),
            (737.0, 448.0),
        ],
        INK,
        3.0,
    );
    let level = 225.0 + 19.0 * (t * 0.22).sin();
    c.ellipse(
        (605.0, level),
        (160.0 - (level - 190.0) * 0.13, 25.0),
        None,
        Some(ACCENT),
        3.0,
    );
    for i in 0..9 {
        let y = 193.0 + f64::from(i) * 25.0;
        c.line(
            &[
                (747.0 - (y - 193.0) * 0.17, y),
                (768.0 - (y - 193.0) * 0.17, y),
            ],
            INK,
            1.5,
        );
    }
    c.faded(0.5, || {
        for i in 0..30 {
            let x = 487.0 + f64::from(i) * 8.0;
            c.curve(
                [
                    (x, 185.0),
                    (x - 10.0, 240.0),
                    (x + 7.0, 372.0),
                    (x + 13.0, 441.0),
                ],
                PRINT,
                0.6,
            );
        }
        for y in [280.0, 288.0, 359.0, 367.0, 420.0] {
            c.curve(
                [(485.0, y), (535.0, y + 21.0), (682.0, y + 21.0), (726.0, y)],
                INK,
                1.0,
            );
        }
        for i in 0..11 {
            let x = 504.0 + f64::from(i) * 18.0;
            c.polygon(
                &[
                    (x, 308.0),
                    (x + 7.0, 297.0),
                    (x + 14.0, 309.0),
                    (x + 7.0, 336.0),
                ],
                "#b4a17b",
                Some(PRINT),
            );
            c.line(&[(x + 7.0, 305.0), (x + 7.0, 341.0)], INK, 0.7);
        }
    });
    c.hatch([493.0, 294.0, 524.0, 416.0], 7.0, PRINT);
    c.hatch([677.0, 294.0, 707.0, 416.0], 7.0, PRINT);
    c.dot((607.0, 456.0), 5.0, INK);
    for i in 0..3 {
        let phase = (t * 0.8 + f64::from(i) / 3.0).rem_euclid(1.0);
        c.ellipse(
            (607.0, 468.0 + phase * 65.0),
            (3.0, 5.0),
            Some(ACCENT),
            None,
            0.0,
        );
    }
    c.curve(
        [
            (553.0, 528.0),
            (579.0, 546.0),
            (637.0, 545.0),
            (664.0, 528.0),
        ],
        INK,
        2.0,
    );
    c.faded(0.55, || {
        c.ellipse(
            (608.0, 532.0),
            (25.0 + 9.0 * (t * 2.0).sin(), 4.0),
            None,
            Some(ACCENT),
            1.0,
        )
    });
    for i in 0..5 {
        let y = 226.0 + f64::from(i) * 46.0;
        c.line(&[(320.0, y), (371.0, y)], PRINT, 1.0);
        c.dot((320.0, y), 2.2, PRINT);
    }
}

fn gear(c: &Painter<'_>, p: Point, r: f64, angle: f64) {
    let points: Vec<_> = (0..64)
        .map(|i| {
            let a = f64::from(i) * TAU / 64.0 + angle;
            let radius = if i % 4 < 2 { r } else { r * 0.87 };
            (p.0 + a.cos() * radius, p.1 + a.sin() * radius)
        })
        .collect();
    c.polygon(&points, INK, None);
    c.dot(p, r * 0.62, PAPER);
    c.ring(p, r * 0.68, PRINT, 1.0);
    for i in 0..5 {
        let a = f64::from(i) * TAU / 5.0 + angle;
        c.line(
            &[p, (p.0 + a.cos() * r * 0.59, p.1 + a.sin() * r * 0.59)],
            INK,
            r * 0.13,
        );
    }
    c.dot(p, r * 0.18, INK);
    c.dot(p, r * 0.07, PALE);
}

fn automaton(c: &Painter<'_>, t: f64) {
    c.faded(0.12, || {
        c.ellipse((636.0, 530.0), (340.0, 20.0), Some(INK), None, 0.0)
    });
    // Turned chair and writing desk, drawn as an engraved museum illustration.
    for x in [364.0, 543.0] {
        c.line(&[(x, 260.0), (x, 515.0)], "#51483b", 15.0);
        for y in [275.0, 293.0, 430.0, 487.0] {
            c.ellipse((x, y), (12.0, 6.0), Some(PRINT), Some(INK), 1.0);
        }
    }
    c.rect([354.0, 255.0, 557.0, 281.0], 5.0, INK, None);
    c.rect([350.0, 403.0, 568.0, 425.0], 5.0, "#5c5141", Some(INK));
    for x in [416.0, 510.0] {
        c.line(
            &[(x, 388.0), (x + 25.0, 462.0), (x + 45.0, 494.0)],
            INK,
            25.0,
        );
        c.line(&[(x + 15.0, 441.0), (x + 30.0, 477.0)], PAPER, 12.0);
        c.ellipse((x + 51.0, 499.0), (32.0, 10.0), Some(INK), None, 0.0);
    }
    c.0.begin_path();
    c.0.move_to(414.0, 251.0);
    c.0.bezier_curve_to(383.0, 276.0, 395.0, 340.0, 377.0, 395.0);
    c.0.bezier_curve_to(422.0, 418.0, 509.0, 420.0, 555.0, 390.0);
    c.0.bezier_curve_to(537.0, 339.0, 549.0, 279.0, 515.0, 251.0);
    c.0.close_path();
    c.finish(Some("#465454"), Some(INK), 2.0);
    c.polygon(
        &[
            (444.0, 254.0),
            (491.0, 254.0),
            (511.0, 390.0),
            (427.0, 390.0),
        ],
        "#c2ae82",
        Some(INK),
    );
    for i in 0..8 {
        let y = 274.0 + f64::from(i) * 14.0;
        c.dot((477.0, y), 3.5, INK);
        c.dot((476.0, y - 1.0), 1.2, PAPER);
        c.curve(
            [
                (407.0, y),
                (417.0, y + 7.0),
                (423.0, y + 8.0),
                (433.0, y + 2.0),
            ],
            "#a39c7e",
            0.8,
        );
        c.curve(
            [
                (505.0, y),
                (514.0, y + 8.0),
                (526.0, y + 9.0),
                (536.0, y + 2.0),
            ],
            "#a39c7e",
            0.8,
        );
    }
    c.polygon(
        &[
            (433.0, 244.0),
            (470.0, 274.0),
            (496.0, 242.0),
            (479.0, 231.0),
            (449.0, 231.0),
        ],
        PAPER,
        Some(PRINT),
    );
    c.polygon(
        &[
            (460.0, 259.0),
            (478.0, 259.0),
            (488.0, 297.0),
            (468.0, 285.0),
            (450.0, 298.0),
        ],
        "#e6dac0",
        Some(PRINT),
    );
    // Curled hair, a modeled face, and small etched facial features.
    c.ellipse(
        (465.0, 191.0),
        (49.0, 59.0),
        Some("#e2c7a0"),
        Some(INK),
        1.6,
    );
    c.ellipse((450.0, 184.0), (42.0, 53.0), Some("#655749"), None, 0.0);
    c.ellipse((482.0, 194.0), (30.0, 42.0), Some("#efdab8"), None, 0.0);
    c.polygon(
        &[(507.0, 185.0), (522.0, 201.0), (507.0, 206.0)],
        "#e2c7a0",
        Some(PRINT),
    );
    for i in 0..15 {
        let a = 1.6 + f64::from(i) * PI / 15.0;
        let p = (461.0 + a.cos() * 43.0, 185.0 + a.sin() * 49.0);
        c.dot(p, 9.0, "#706151");
        c.curve(
            [
                (p.0 - 5.0, p.1),
                (p.0 - 6.0, p.1 - 8.0),
                (p.0 + 8.0, p.1 - 5.0),
                (p.0 + 5.0, p.1 + 3.0),
            ],
            "#b19d7f",
            1.1,
        );
    }
    c.curve(
        [
            (488.0, 184.0),
            (497.0, 181.0),
            (502.0, 182.0),
            (507.0, 185.0),
        ],
        INK,
        1.5,
    );
    c.dot((500.0, 188.0), 2.2, INK);
    c.curve(
        [
            (493.0, 216.0),
            (500.0, 218.0),
            (507.0, 216.0),
            (511.0, 213.0),
        ],
        PRINT,
        1.0,
    );
    c.faded(0.4, || c.hatch([481.0, 221.0, 498.0, 234.0], 2.5, PRINT));
    // A cutaway suggests the cam mechanism without claiming literal anatomy.
    gear(c, (373.0, 338.0), 54.0, t * 0.32);
    gear(c, (327.0, 390.0), 31.0, -t * 0.55);
    c.line(
        &[(373.0, 338.0), (404.0, 305.0), (426.0, 305.0)],
        PRINT,
        4.0,
    );
    c.rect([577.0, 429.0, 1009.0, 450.0], 2.0, "#64513d", Some(INK));
    c.rect([595.0, 450.0, 995.0, 469.0], 2.0, "#8c7150", Some(INK));
    for x in [614.0, 967.0] {
        c.line(&[(x, 464.0), (x - 7.0, 532.0)], INK, 15.0);
        c.ellipse((x - 3.0, 482.0), (11.0, 6.0), Some(PRINT), Some(INK), 1.0);
    }
    c.polygon(
        &[
            (635.0, 373.0),
            (874.0, 372.0),
            (966.0, 427.0),
            (668.0, 427.0),
        ],
        "#fff6e1",
        Some(PRINT),
    );
    let pen = (
        755.0 + 28.0 * (t * 0.65).sin(),
        402.0 + 8.0 * (t * 1.3).sin(),
    );
    let elbow = (620.0 + 5.0 * (t * 0.65).sin(), 339.0);
    let hand = (pen.0 - 24.0, pen.1 - 33.0);
    c.line(&[(521.0, 276.0), elbow, (hand.0 - 22.0, hand.1)], INK, 30.0);
    c.line(
        &[(521.0, 273.0), elbow, (hand.0 - 22.0, hand.1 - 2.0)],
        "#506060",
        23.0,
    );
    c.line(
        &[(hand.0 - 35.0, hand.1 - 7.0), (hand.0 - 16.0, hand.1 + 1.0)],
        PAPER,
        25.0,
    );
    c.ellipse(hand, (18.0, 11.0), Some("#e4c9a1"), Some(PRINT), 1.0);
    for i in 0..3 {
        c.line(
            &[
                (hand.0 - 3.0, hand.1 - 4.0 + f64::from(i) * 4.0),
                (hand.0 + 13.0, hand.1 + f64::from(i) * 3.0),
            ],
            PRINT,
            0.7,
        );
    }
    c.line(&[(pen.0 - 34.0, pen.1 - 83.0), pen], INK, 2.0);
    c.curve(
        [
            (pen.0 - 34.0, pen.1 - 83.0),
            (pen.0 - 77.0, pen.1 - 113.0),
            (pen.0 - 35.0, pen.1 - 132.0),
            (pen.0 - 25.0, pen.1 - 100.0),
        ],
        "#bdb99e",
        13.0,
    );
    c.line(
        &[(pen.0 - 54.0, pen.1 - 117.0), (pen.0 - 29.0, pen.1 - 79.0)],
        INK,
        0.8,
    );
    for i in 0..4 {
        let x = 698.0 + f64::from(i) * 44.0;
        c.curve(
            [
                (x, 407.0),
                (x + 8.0, 383.0),
                (x + 29.0, 427.0),
                (x + 41.0, 401.0),
            ],
            PRINT,
            1.0,
        );
    }
    c.ellipse((922.0, 395.0), (14.0, 7.0), Some(INK), Some(PRINT), 1.0);
    c.rect([909.0, 377.0, 935.0, 395.0], 3.0, INK, Some(PRINT));
    c.ellipse((922.0, 377.0), (13.0, 5.0), Some(PAPER), Some(INK), 1.0);
}

/// An explanatory cutaway, not a claim to reconstruct Heron's exact machine.
fn mechanical_theatre(c: &Painter<'_>, t: f64) {
    c.faded(0.08, || {
        c.ellipse((604.0, 545.0), (400.0, 17.0), Some(INK), None, 0.0);
    });
    c.rect([268.0, 151.0, 934.0, 428.0], 0.0, "#e7dcc6", Some(INK));
    c.faded(0.45, || c.hatch([286.0, 176.0, 913.0, 402.0], 16.0, PALE));
    c.faded(0.30, || {
        for row in 0..9 {
            let y = 329.0 + f64::from(row) * 7.0;
            c.curve(
                [
                    (297.0, y),
                    (424.0, y - 88.0),
                    (638.0, y + 45.0),
                    (914.0, y - 45.0),
                ],
                PRINT,
                0.8,
            );
        }
        c.ellipse((604.0, 221.0), (44.0, 44.0), None, Some(PRINT), 0.8);
        for x in [311.0, 883.0] {
            c.rect([x, 190.0, x + 22.0, 379.0], 0.0, PALE, Some(PRINT));
            for col in 0..4 {
                let xx = x + 3.0 + f64::from(col) * 5.0;
                c.line(&[(xx, 198.0), (xx, 370.0)], PRINT, 0.8);
            }
            c.rect([x - 7.0, 182.0, x + 29.0, 193.0], 0.0, PAPER, Some(PRINT));
        }
    });
    c.line(&[(290.0, 385.0), (912.0, 385.0)], PRINT, 1.0);
    // The actors have visible pivots and repeat a mechanically arranged gesture.
    for (i, x) in [426.0, 609.0, 791.0].into_iter().enumerate() {
        let angle = t * 1.7 + i as f64 * 1.2;
        let sway = angle.sin() * 8.0;
        let shoulder = (x + sway, 288.0);
        c.ellipse((x + sway, 248.0), (21.0, 27.0), Some(INK), None, 0.0);
        c.ellipse(
            (x + 6.0 + sway, 249.0),
            (13.0, 20.0),
            Some("#d5c7a9"),
            None,
            0.0,
        );
        c.dot((x + 12.0 + sway, 246.0), 1.8, INK);
        c.line(
            &[
                (x + 15.0 + sway, 250.0),
                (x + 20.0 + sway, 254.0),
                (x + 14.0 + sway, 256.0),
            ],
            PRINT,
            0.8,
        );
        c.curve(
            [
                (x - 19.0 + sway, 239.0),
                (x - 10.0 + sway, 215.0),
                (x + 14.0 + sway, 221.0),
                (x + 21.0 + sway, 238.0),
            ],
            PRINT,
            3.0,
        );
        c.polygon(
            &[
                (x - 24.0 + sway, 280.0),
                (x + 25.0 + sway, 280.0),
                (x + 38.0, 353.0),
                (x - 35.0, 353.0),
            ],
            if i == 1 { PRINT } else { PALE },
            Some(INK),
        );
        for fold in 0..6 {
            let dx = f64::from(fold) * 8.0 - 20.0;
            c.curve(
                [
                    (x + dx * 0.5 + sway, 290.0),
                    (x + dx + sway, 307.0),
                    (x + dx * 0.9, 333.0),
                    (x + dx * 1.4, 348.0),
                ],
                PRINT,
                0.9,
            );
        }
        c.line(
            &[(x - 23.0 + sway, 312.0), (x + 28.0 + sway, 312.0)],
            INK,
            3.0,
        );
        for side in [-1.0, 1.0] {
            let knee = (x + side * 23.0 + sway, 379.0);
            let foot = (x + side * 33.0 + sway * 1.8, 414.0);
            c.line(&[(x + side * 16.0, 350.0), knee, foot], INK, 9.0);
            c.line(&[foot, (foot.0 + 15.0, foot.1)], INK, 7.0);
            let elbow = (shoulder.0 + side * 44.0, 300.0 + side * angle.sin() * 19.0);
            let hand = (shoulder.0 + side * 74.0, 267.0 + side * angle.sin() * 27.0);
            c.line(
                &[(shoulder.0 + side * 19.0, shoulder.1), elbow, hand],
                INK,
                11.0,
            );
            c.line(
                &[(shoulder.0 + side * 19.0, shoulder.1), elbow, hand],
                PALE,
                5.0,
            );
            for point in [elbow, knee] {
                c.dot(point, 5.0, INK);
                c.dot(point, 2.0, PAPER);
            }
        }
        c.line(&[(x, 418.0), (x, 472.0)], PRINT, 2.0);
        let pin = (x + 14.0 * angle.cos(), 495.0 + 14.0 * angle.sin());
        c.ellipse(
            (x + 5.0 * angle.cos(), 495.0 + 5.0 * angle.sin()),
            (24.0, 18.0),
            Some(PALE),
            Some(INK),
            2.0,
        );
        c.line(&[pin, (x, 472.0)], ACCENT, 3.0);
        c.dot(pin, 3.0, INK);
    }
    // Curtains open and settle while the exposed shaft drives the figures.
    let opening = 18.0 + 16.0 * (t * 0.65).sin();
    for right in [false, true] {
        let edge = if right { 934.0 } else { 268.0 };
        let sign = if right { -1.0 } else { 1.0 };
        c.polygon(
            &[
                (edge, 151.0),
                (edge + sign * (117.0 - opening), 151.0),
                (edge + sign * 73.0, 237.0),
                (edge + sign * (62.0 - opening * 0.5), 341.0),
                (edge + sign * 52.0, 423.0),
                (edge, 423.0),
            ],
            INK,
            None,
        );
        for fold in 1..4 {
            let x = edge + sign * f64::from(fold) * 15.0;
            c.curve(
                [
                    (x, 163.0),
                    (x + sign * 13.0, 225.0),
                    (x - sign * 9.0, 349.0),
                    (x, 412.0),
                ],
                PRINT,
                1.3,
            );
        }
    }
    c.polygon(
        &[(255.0, 127.0), (601.0, 87.0), (948.0, 127.0)],
        PALE,
        Some(INK),
    );
    c.faded(0.65, || c.hatch([339.0, 111.0, 849.0, 125.0], 3.0, PRINT));
    c.ellipse((601.0, 112.0), (25.0, 13.0), Some(INK), Some(PRINT), 1.0);
    c.spark((601.0, 112.0), 10.0, PALE, 0.0);
    c.rect([242.0, 128.0, 960.0, 153.0], 2.0, INK, None);
    for i in 0..34 {
        let x = 252.0 + f64::from(i) * 21.0;
        c.rect([x, 135.0, x + 9.0, 145.0], 0.0, PALE, None);
    }
    for x in [246.0, 947.0] {
        c.line(&[(x, 153.0), (x, 425.0)], INK, 17.0);
        c.line(&[(x - 3.0, 156.0), (x - 3.0, 422.0)], PRINT, 2.0);
        ornament(c, (x, 169.0), 11.0, PALE);
    }
    c.rect([242.0, 427.0, 960.0, 443.0], 2.0, INK, None);
    c.rect([253.0, 446.0, 269.0, 534.0], 0.0, PRINT, None);
    c.rect([933.0, 446.0, 949.0, 534.0], 0.0, PRINT, None);
    c.rect([242.0, 532.0, 960.0, 543.0], 1.0, INK, None);
    c.line(&[(298.0, 495.0), (917.0, 495.0)], INK, 5.0);
    gear(c, (334.0, 495.0), 28.0, t * 1.7);
    // A falling weight and rope make the source of motion legible.
    let weight_y = 273.0 + 30.0 * (t * 0.45).sin();
    c.ring((192.0, 189.0), 22.0, INK, 3.0);
    c.dot((192.0, 189.0), 5.0, INK);
    c.line(
        &[
            (170.0, weight_y),
            (170.0, 189.0),
            (170.0, 167.0),
            (214.0, 167.0),
            (214.0, 495.0),
            (306.0, 495.0),
        ],
        PRINT,
        2.0,
    );
    c.rect([153.0, weight_y, 187.0, weight_y + 53.0], 3.0, INK, None);
    c.hatch([158.0, weight_y + 8.0, 181.0, weight_y + 44.0], 6.0, PRINT);
}

/// Four musicians in a boat; visible linkages are schematic teaching devices.
fn musical_boat(c: &Painter<'_>, t: f64) {
    let bob = 2.0 * (t * 0.8).sin();
    c.0.save();
    let _ = c.0.translate(0.0, bob);
    for (i, x) in [362.0, 520.0, 681.0, 837.0].into_iter().enumerate() {
        let phase = t * 2.3 + i as f64 * 0.9;
        let sway = 4.0 * phase.sin();
        c.rect([x - 33.0, 336.0, x + 34.0, 346.0], 1.0, INK, None);
        c.line(&[(x - 24.0, 344.0), (x - 24.0, 399.0)], INK, 5.0);
        c.line(&[(x + 25.0, 344.0), (x + 25.0, 399.0)], INK, 5.0);
        c.line(
            &[(x - 9.0, 324.0), (x + 35.0, 351.0), (x + 39.0, 389.0)],
            INK,
            14.0,
        );
        c.line(
            &[(x + 13.0, 323.0), (x + 48.0, 347.0), (x + 56.0, 389.0)],
            PRINT,
            10.0,
        );
        c.polygon(
            &[
                (x - 24.0 + sway, 246.0),
                (x + 26.0 + sway, 246.0),
                (x + 38.0, 330.0),
                (x - 31.0, 330.0),
            ],
            if i % 2 == 0 { PALE } else { PRINT },
            Some(INK),
        );
        c.ellipse((x + sway, 218.0), (23.0, 29.0), Some(PAPER), Some(INK), 2.0);
        c.ellipse(
            (x - 3.0 + sway, 195.0),
            (29.0, 13.0),
            Some(PALE),
            Some(INK),
            2.0,
        );
        c.line(
            &[(x - 24.0 + sway, 191.0), (x + 17.0 + sway, 202.0)],
            PRINT,
            1.2,
        );
        c.dot((x + 12.0 + sway, 216.0), 2.2, INK);
        c.curve(
            [
                (x + 7.0 + sway, 230.0),
                (x + 26.0 + sway, 229.0),
                (x + 17.0 + sway, 246.0),
                (x + 3.0 + sway, 238.0),
            ],
            INK,
            4.0,
        );
        for fold in 0..5 {
            let xx = x - 17.0 + f64::from(fold) * 9.0;
            c.curve(
                [
                    (xx + sway, 254.0),
                    (xx + 11.0, 285.0),
                    (xx - 4.0, 304.0),
                    (xx + 4.0, 326.0),
                ],
                PRINT,
                0.9,
            );
        }
        c.line(&[(x - 23.0, 304.0), (x + 33.0, 304.0)], INK, 3.0);
        for seam in 0..6 {
            c.dot((x + 2.0 + sway, 257.0 + f64::from(seam) * 7.0), 1.3, PAPER);
        }
        // A lute, a barrel drum, a flute, and a frame drum give distinct poses.
        let (left_hand, right_hand) = match i {
            0 => {
                c.ellipse((x + 32.0, 295.0), (27.0, 35.0), Some(PALE), Some(INK), 2.5);
                c.line(&[(x + 36.0, 280.0), (x + 73.0, 229.0)], INK, 10.0);
                for string in 0..3 {
                    let dx = f64::from(string) * 3.0;
                    c.line(
                        &[(x + 23.0 + dx, 317.0), (x + 70.0 + dx, 230.0)],
                        PRINT,
                        0.8,
                    );
                }
                c.dot((x + 32.0, 292.0), 7.0, INK);
                ((x + 26.0, 294.0 + phase.sin() * 8.0), (x + 58.0, 249.0))
            }
            1 => {
                c.rect([x + 18.0, 289.0, x + 82.0, 336.0], 3.0, PALE, Some(INK));
                c.ellipse((x + 50.0, 289.0), (33.0, 8.0), Some(PAPER), Some(INK), 2.0);
                for string in 0..5 {
                    let xx = x + 23.0 + f64::from(string) * 12.0;
                    c.line(&[(xx, 297.0), (xx + 7.0, 331.0)], PRINT, 1.0);
                }
                let a = (x + 28.0, 262.0 + 13.0 * phase.sin());
                let b = (x + 70.0, 264.0 - 13.0 * phase.sin());
                c.line(&[a, (a.0 + 13.0, a.1 + 18.0)], INK, 3.0);
                c.line(&[b, (b.0 - 10.0, b.1 + 18.0)], INK, 3.0);
                (a, b)
            }
            2 => {
                c.line(&[(x + 18.0 + sway, 225.0), (x + 83.0, 270.0)], INK, 5.0);
                ((x + 41.0, 241.0 + phase.sin() * 2.0), (x + 65.0, 258.0))
            }
            _ => {
                c.ellipse((x + 60.0, 285.0), (24.0, 30.0), Some(PAPER), Some(INK), 4.0);
                c.ellipse((x + 60.0, 285.0), (18.0, 24.0), None, Some(PRINT), 1.0);
                ((x + 42.0, 285.0 + phase.sin() * 11.0), (x + 71.0, 309.0))
            }
        };
        for (side, hand) in [(-1.0, left_hand), (1.0, right_hand)] {
            let shoulder = (x + side * 22.0 + sway, 257.0);
            let elbow = (x + side * 33.0 + 15.0, 286.0 + phase.sin() * 6.0);
            c.line(&[shoulder, elbow, hand], INK, 10.0);
            c.line(&[shoulder, elbow, hand], PALE, 4.5);
            c.dot(elbow, 5.0, INK);
            c.dot(elbow, 2.0, PAPER);
            c.dot(hand, 4.0, INK);
        }
        c.faded(0.55, || c.line(&[(x, 322.0), (x, 397.0)], PRINT, 1.3));
    }
    c.polygon(
        &[
            (188.0, 362.0),
            (280.0, 390.0),
            (945.0, 390.0),
            (1034.0, 362.0),
            (949.0, 452.0),
            (882.0, 478.0),
            (337.0, 478.0),
            (269.0, 452.0),
        ],
        PALE,
        Some(INK),
    );
    c.line(
        &[
            (208.0, 383.0),
            (296.0, 408.0),
            (923.0, 408.0),
            (1013.0, 383.0),
        ],
        INK,
        3.0,
    );
    c.rect([322.0, 412.0, 899.0, 462.0], 3.0, PAPER, Some(PRINT));
    c.line(&[(304.0, 437.0), (923.0, 437.0)], INK, 5.0);
    for (i, x) in [362.0, 520.0, 681.0, 837.0].into_iter().enumerate() {
        let phase = t * 2.3 + i as f64 * 0.9;
        let pin = (x + phase.cos() * 12.0, 437.0 + phase.sin() * 12.0);
        c.ellipse(
            (x + phase.cos() * 4.0, 437.0 + phase.sin() * 4.0),
            (20.0, 15.0),
            Some(PALE),
            Some(INK),
            2.0,
        );
        c.line(&[pin, (x, 396.0)], ACCENT, 2.5);
        c.dot(pin, 3.0, INK);
    }
    gear(c, (291.0, 434.0), 28.0, -t * 2.3);
    c.hatch([362.0, 467.0, 853.0, 475.0], 4.0, PRINT);
    for x in [280.0, 931.0] {
        for i in 0..4 {
            let y = 412.0 + f64::from(i) * 10.0;
            c.curve(
                [
                    (x - 18.0, y),
                    (x - 4.0, y + 7.0),
                    (x + 15.0, y - 3.0),
                    (x + 22.0, y + 6.0),
                ],
                PRINT,
                0.7,
            );
        }
    }
    for i in 0..26 {
        let x = 310.0 + f64::from(i) * 23.0;
        c.polygon(
            &[
                (x, 395.0),
                (x + 7.0, 390.0),
                (x + 14.0, 395.0),
                (x + 7.0, 401.0),
            ],
            PRINT,
            None,
        );
    }
    c.curve(
        [
            (207.0, 382.0),
            (149.0, 324.0),
            (200.0, 317.0),
            (226.0, 359.0),
        ],
        INK,
        5.0,
    );
    c.curve(
        [
            (1006.0, 382.0),
            (1064.0, 324.0),
            (1013.0, 317.0),
            (987.0, 359.0),
        ],
        INK,
        5.0,
    );
    c.0.restore();
    for row in 0..3 {
        let y = 501.0 + f64::from(row) * 13.0;
        let wave: Vec<_> = (0..37)
            .map(|i| {
                let x = 187.0 + f64::from(i) * 24.0;
                (
                    x,
                    y + 3.0 * (f64::from(i) * 0.8 - t * 1.2 + f64::from(row)).sin(),
                )
            })
            .collect();
        c.faded(0.6 - f64::from(row) * 0.15, || c.line(&wave, PRINT, 1.3));
    }
}

fn loom(c: &Painter<'_>, t: f64) {
    for x in [323.0, 843.0] {
        c.rect([x, 122.0, x + 24.0, 518.0], 1.0, INK, None);
        c.rect([x - 18.0, 509.0, x + 48.0, 525.0], 1.0, INK, None);
    }
    for x in [323.0, 843.0] {
        for line in 0..5 {
            let xx = x + 3.0 + f64::from(line) * 4.0;
            c.curve(
                [
                    (xx, 130.0),
                    (xx + 3.0, 260.0),
                    (xx - 3.0, 390.0),
                    (xx, 508.0),
                ],
                PRINT,
                0.7,
            );
        }
        for y in [154.0, 218.0, 474.0] {
            c.dot((x + 12.0, y), 4.0, PALE);
            c.dot((x + 12.0, y), 1.5, INK);
        }
    }
    for y in [142.0, 205.0, 461.0] {
        c.rect([326.0, y, 862.0, y + 19.0], 2.0, INK, None);
    }
    for i in 0..24 {
        let x = 362.0 + f64::from(i) * 19.0;
        c.line(
            &[
                (x, 166.0),
                (x + 4.0 * (t * 1.8 + f64::from(i)).sin(), 453.0),
            ],
            if i % 4 == 0 { ACCENT } else { PRINT },
            if i % 4 == 0 { 1.8 } else { 0.8 },
        );
    }
    c.rect([370.0, 381.0, 822.0, 456.0], 0.0, PALE, None);
    for row in 0..9 {
        let y = 386.0 + f64::from(row) * 8.0;
        c.line(&[(371.0, y), (820.0, y)], PRINT, 0.7);
        for col in 0..7 {
            if (row + col) % 3 == 0 {
                let x = 385.0 + f64::from(col) * 59.0;
                c.line(&[(x, y), (x + 31.0, y)], INK, 4.0);
            }
        }
    }
    for row in 0..5 {
        for col in 0..9 {
            let x = 392.0 + f64::from(col) * 47.0;
            let y = 390.0 + f64::from(row) * 13.0;
            c.curve(
                [
                    (x - 13.0, y),
                    (x - 5.0, y - 9.0),
                    (x + 5.0, y + 9.0),
                    (x + 13.0, y),
                ],
                "#a2794a",
                1.4,
            );
        }
    }
    let x = 588.0 + 176.0 * (t * 1.1).sin();
    c.polygon(
        &[(x - 58.0, 341.0), (x, 325.0), (x + 58.0, 341.0), (x, 355.0)],
        INK,
        Some(INK),
    );
    c.line(&[(x - 30.0, 341.0), (x + 30.0, 341.0)], ACCENT, 4.0);
    for offset in [16.0, 8.0, 0.0] {
        c.rect(
            [
                886.0 + offset,
                187.0 - offset,
                1041.0 + offset,
                345.0 - offset,
            ],
            2.0,
            "#e5d8bd",
            Some(PRINT),
        );
    }
    for y in [194.0, 335.0] {
        for col in 0..11 {
            let x = 892.0 + f64::from(col) * 13.0;
            c.dot((x, y), 1.5, PRINT);
        }
    }
    for row in 0..5 {
        for col in 0..5 {
            if (row * 3 + col) % 3 != 1 {
                let x = 900.0 + f64::from(col) * 26.0;
                let y = 205.0 + f64::from(row) * 26.0;
                c.rect([x, y, x + 9.0, y + 9.0], 0.0, INK, None);
            }
        }
    }
    c.curve(
        [
            (886.0, 266.0),
            (864.0, 268.0),
            (870.0, 205.0),
            (844.0, 210.0),
        ],
        ACCENT,
        2.0,
    );
}

fn instructions(c: &Painter<'_>, t: f64) {
    c.polygon(
        &[
            (278.0, 137.0),
            (567.0, 137.0),
            (607.0, 178.0),
            (607.0, 480.0),
            (278.0, 480.0),
        ],
        "#f9f0de",
        Some(INK),
    );
    c.polygon(
        &[(567.0, 137.0), (567.0, 178.0), (607.0, 178.0)],
        PALE,
        Some(INK),
    );
    for i in 0..8 {
        let y = 207.0 + f64::from(i) * 29.0;
        c.dot((305.0, y), 2.2, PRINT);
        c.line(
            &[(321.0, y), (390.0 + f64::from(i % 4) * 39.0, y)],
            if i % 3 == 0 { INK } else { PRINT },
            2.0,
        );
        if i % 3 == 0 {
            c.line(&[(335.0, y + 10.0), (423.0, y + 10.0)], PRINT, 1.0);
        }
    }
    let scan = ((t * 0.4).rem_euclid(1.0) * 7.0).floor() * 29.0;
    c.faded(0.14, || {
        c.rect(
            [294.0, 192.0 + scan, 571.0, 218.0 + scan],
            0.0,
            ACCENT,
            None,
        )
    });
    for (i, y) in [212.0, 320.0, 428.0].iter().enumerate() {
        let path = [
            (609.0, 230.0 + i as f64 * 75.0),
            (654.0, 230.0 + i as f64 * 75.0),
            (677.0, *y),
            (719.0, *y),
        ];
        c.curve(path, PRINT, 1.5);
        c.signal(path, t * 0.3 + i as f64 * 0.3, ACCENT, 4.0);
        c.polygon(
            &[
                (783.0, *y - 36.0),
                (847.0, *y),
                (783.0, *y + 36.0),
                (719.0, *y),
            ],
            PALE,
            Some(INK),
        );
        c.ring((783.0, *y), 9.0, INK, 2.0);
        if i < 2 {
            c.line(&[(783.0, *y + 38.0), (783.0, *y + 69.0)], INK, 2.0);
        }
    }
    let path = [
        (846.0, 320.0),
        (940.0, 320.0),
        (941.0, 143.0),
        (780.0, 145.0),
    ];
    c.curve(path, PRINT, 1.5);
    c.signal(path, t * 0.17, ACCENT, 4.0);
}

fn neural(c: &Painter<'_>, t: f64, layered: bool) {
    plate(c, [468.0, 151.0, 1111.0, 512.0], -0.018, || {
        c.faded(0.17, || {
            for i in 0..18 {
                let y = 176.0 + f64::from(i) * 17.0;
                c.line(&[(488.0, y), (1092.0, y)], PRINT, 0.5);
            }
            for i in 0..30 {
                let x = 488.0 + f64::from(i) * 20.0;
                c.line(&[(x, 170.0), (x, 480.0)], PRINT, 0.4);
            }
        });
        if !layered {
            // A botanical-style neuron engraving becomes a mathematical switch.
            let p = (643.0, 319.0);
            for branch in 0..12 {
                let a = f64::from(branch) * TAU / 12.0;
                let tip = (p.0 + a.cos() * 115.0, p.1 + a.sin() * 117.0);
                let middle = (p.0 + a.cos() * 63.0, p.1 + a.sin() * 54.0);
                c.curve(
                    [p, (p.0 + a.cos() * 35.0, p.1 + a.sin() * 40.0), middle, tip],
                    INK,
                    2.0,
                );
                for twig in [-1.0, 1.0] {
                    let b = a + twig * 0.4;
                    let end = (middle.0 + b.cos() * 53.0, middle.1 + b.sin() * 51.0);
                    c.curve(
                        [
                            middle,
                            (middle.0 + 4.0, middle.1),
                            (end.0 - 8.0, end.1),
                            end,
                        ],
                        PRINT,
                        1.1,
                    );
                    c.line(
                        &[
                            end,
                            (
                                end.0 + (b - 0.4).cos() * 13.0,
                                end.1 + (b - 0.4).sin() * 13.0,
                            ),
                        ],
                        PRINT,
                        0.7,
                    );
                }
                c.faded(0.65, || {
                    c.signal(
                        [tip, middle, middle, p],
                        t * 0.22 + f64::from(branch) / 12.0,
                        ACCENT,
                        2.0,
                    )
                });
            }
            let body: Vec<_> = (0..36)
                .map(|i| {
                    let a = f64::from(i) * TAU / 36.0;
                    let r = 25.0 + 4.0 * (a * 6.0).sin();
                    (p.0 + a.cos() * r, p.1 + a.sin() * r)
                })
                .collect();
            c.polygon(&body, "#b3ad8c", Some(INK));
            c.ellipse(p, (14.0, 18.0), Some("#6b7160"), Some(INK), 1.0);
            c.dot((639.0, 314.0), 4.0, PAPER);
            let axon = [
                (672.0, 321.0),
                (750.0, 284.0),
                (805.0, 366.0),
                (864.0, 317.0),
            ];
            c.curve(axon, INK, 3.0);
            for i in 0..8 {
                let v = f64::from(i) / 8.0;
                let q = bezier(axon, v);
                c.ellipse(q, (10.0, 6.0), Some(PALE), Some(PRINT), 1.0);
            }
            c.signal(axon, t * 0.35, ACCENT, 4.0);
            c.ring((923.0, 317.0), 43.0, INK, 2.0);
            c.ring((923.0, 317.0), 48.0, PRINT, 0.6);
            let _ = c.text("Σ", (905.0, 317.0), 38.0, INK, 50.0);
            c.line(&[(967.0, 317.0), (1048.0, 317.0)], PRINT, 1.6);
            let on = (t * 1.4).sin() > 0.0;
            c.dot((1052.0, 317.0), 14.0, if on { ACCENT } else { PALE });
            for i in 0..5 {
                let y = 412.0 + f64::from(i) * 10.0;
                c.line(&[(835.0, y), (1034.0, y)], PRINT, 0.6);
            }
            c.line(
                &[
                    (843.0, 452.0),
                    (895.0, 452.0),
                    (895.0, 417.0),
                    (1033.0, 417.0),
                ],
                ACCENT,
                2.0,
            );
        } else {
            // Input pixels, weighted edges and a threshold output form a perceptron.
            for row in 0..5 {
                for col in 0..5 {
                    let x = 505.0 + f64::from(col) * 16.0;
                    let y = 274.0 + f64::from(row) * 16.0;
                    c.rect(
                        [x, y, x + 12.0, y + 12.0],
                        0.0,
                        if row == 2 || col == 2 { INK } else { PALE },
                        None,
                    );
                }
            }
            // One weighted sum and threshold, not an anachronistic hidden layer.
            let layers = [(657.0, 4), (818.0, 1), (975.0, 1)];
            for layer in 0..2 {
                let (x, count) = layers[layer];
                let (nx, ncount) = layers[layer + 1];
                for i in 0..count {
                    for j in 0..ncount {
                        let a = (
                            x,
                            320.0 + (f64::from(i) - f64::from(count - 1) / 2.0) * 65.0,
                        );
                        let b = (
                            nx,
                            320.0 + (f64::from(j) - f64::from(ncount - 1) / 2.0) * 65.0,
                        );
                        let path = [a, (x + 60.0, a.1), (nx - 60.0, b.1), b];
                        c.faded(0.25 + f64::from((i + j) % 3) * 0.18, || {
                            c.curve(path, INK, 0.8 + f64::from((i + j) % 3) * 0.7)
                        });
                        c.signal(path, t * 0.3 + f64::from(i + j) * 0.13, ACCENT, 2.6);
                    }
                }
            }
            for (x, count) in layers {
                for i in 0..count {
                    let y = 320.0 + (f64::from(i) - f64::from(count - 1) / 2.0) * 65.0;
                    c.dot((x + 3.0, y + 4.0), 19.0, "#c9b994");
                    c.dot((x, y), 17.0, INK);
                    c.ring((x, y), 22.0, PRINT, 0.7);
                    c.dot((x - 4.0, y - 4.0), 4.0, PALE);
                }
            }
            c.line(&[(599.0, 314.0), (628.0, 314.0)], PRINT, 1.2);
            let _ = c.text("Σ", (808.0, 369.0), 22.0, INK, 40.0);
            let _ = c.text("0 / 1", (951.0, 369.0), 20.0, INK, 100.0);
            c.faded(0.7, || {
                c.line(&[(648.0, 464.0), (969.0, 464.0)], PRINT, 0.8);
                for i in 0..22 {
                    let x = 652.0 + f64::from(i) * 14.0;
                    let y = 450.0 - 32.0 * (-f64::from(i) * 0.14).exp();
                    c.dot((x, y), 1.7, ACCENT);
                }
            });
        }
    });
}

fn digit_pixel(col: i32, row: i32) -> bool {
    ((2..=4).contains(&row) && (3..=12).contains(&col))
        || ((4..=14).contains(&row)
            && (f64::from(col) - (12.0 - f64::from(row - 4) * 0.58)).abs() < 1.15)
}

fn vision(c: &Painter<'_>, t: f64, electric: bool) {
    let accent = if electric { "#338b92" } else { ACCENT };
    let violet = if electric { "#7469a7" } else { PRINT };
    plate(c, [456.0, 214.0, 651.0, 444.0], -0.07, || {
        c.faded(0.25, || {
            for row in 0..12 {
                let y = 232.0 + f64::from(row) * 15.0;
                c.line(&[(471.0, y), (636.0, y)], PRINT, 0.5);
            }
        });
        let (x, y) = (474.0, 239.0);
        for row in 0..16 {
            for col in 0..16 {
                let xx = x + f64::from(col) * 9.8;
                let yy = y + f64::from(row) * 9.8;
                c.rect(
                    [xx, yy, xx + 8.9, yy + 8.9],
                    0.0,
                    if digit_pixel(col, row) {
                        INK
                    } else {
                        "#e7dec8"
                    },
                    None,
                );
            }
        }
        let scan = (t * 3.5) as i32 % 100;
        let sx = x + f64::from(scan % 10) * 9.8;
        let sy = y + f64::from(scan / 10) * 9.8;
        c.faded(0.18, || {
            c.rect([sx, sy, sx + 49.0, sy + 49.0], 0.0, accent, None)
        });
        c.line(
            &[
                (sx, sy),
                (sx + 49.0, sy),
                (sx + 49.0, sy + 49.0),
                (sx, sy + 49.0),
                (sx, sy),
            ],
            accent,
            1.8,
        );
        c.0.set_font("italic 13px Georgia, serif");
        c.0.set_fill_style_str(PRINT);
        let _ = c.0.fill_text("0 1 2 3 4 5 6 7 8 9", 474.0, 424.0);
    });
    let count = if electric { 5 } else { 3 };
    for layer in 0..count {
        let x = 701.0 + f64::from(layer) * 41.0;
        let y = 244.0 - f64::from(layer) * 10.0;
        let fill = if electric {
            ["#c1d9cf", "#adbfcd", "#c7bad8", "#d7bbba", "#b9cec1"][layer as usize]
        } else {
            "#ddd0b2"
        };
        c.0.set_shadow_color("rgba(49,40,26,.13)");
        c.0.set_shadow_blur(10.0);
        c.0.set_shadow_offset_y(7.0);
        c.polygon(
            &[
                (x, y),
                (x + 64.0, y - 31.0),
                (x + 64.0, y + 146.0),
                (x, y + 177.0),
            ],
            fill,
            Some(PRINT),
        );
        c.0.set_shadow_color("transparent");
        for row in 0..10 {
            for col in 0..4 {
                let xx = x + 8.0 + f64::from(col) * 14.0;
                let yy = y + 13.0 + f64::from(row) * 15.0 - f64::from(col) * 7.0;
                let edge = digit_pixel(col * 3 + 2, row + 3) != digit_pixel(col * 3 + 3, row + 3);
                let active = edge || ((row + col + layer) % 7 == 0);
                c.polygon(
                    &[
                        (xx, yy),
                        (xx + 10.0, yy - 5.0),
                        (xx + 10.0, yy + 6.0),
                        (xx, yy + 11.0),
                    ],
                    if active { accent } else { fill },
                    None,
                );
            }
        }
        c.line(&[(x, y + 183.0), (x + 64.0, y + 152.0)], PRINT, 0.7);
    }
    let input = [
        (650.0, 320.0),
        (671.0, 320.0),
        (680.0, 318.0),
        (700.0, 318.0),
    ];
    c.curve(input, PRINT, 1.0);
    c.signal(input, t * 0.5, accent, 3.0);
    let end = 701.0 + f64::from(count - 1) * 41.0 + 65.0;
    for i in 0..5 {
        let p = (989.0, 264.0 + f64::from(i) * 27.0);
        let path = [(end, 310.0), (end + 32.0, 310.0), (962.0, p.1), p];
        c.faded(0.6, || c.curve(path, violet, 0.8));
        c.signal(path, t * 0.34 + f64::from(i) * 0.12, accent, 2.2);
        c.dot(p, 5.0, violet);
        c.line(&[p, (1035.0, 318.0)], PRINT, 0.7);
    }
    c.dot((1070.0, 318.0), 40.0, INK);
    c.ring((1070.0, 318.0), 46.0, accent, 1.5);
    c.0.set_font("italic 55px Georgia, serif");
    c.0.set_fill_style_str("#f7edda");
    let _ = c.0.fill_text("7", 1053.0, 337.0);
    // Output is an illustrative class, never a fabricated benchmark score.
    for (i, height) in [9.0, 14.0, 7.0, 18.0, 6.0, 11.0, 8.0, 48.0, 9.0, 13.0]
        .into_iter()
        .enumerate()
    {
        let x = 980.0 + i as f64 * 13.0;
        c.rect(
            [x, 470.0 - height, x + 7.0, 470.0],
            0.0,
            if i == 7 { accent } else { "#c6bba2" },
            None,
        );
    }
    c.line(&[(977.0, 472.0), (1111.0, 472.0)], PRINT, 0.7);
    if electric {
        c.faded(0.22, || {
            let points: Vec<_> = (0..100)
                .map(|i| {
                    let a = f64::from(i) * TAU / 99.0;
                    (811.0 + a.cos() * 296.0, 329.0 + a.sin() * 185.0)
                })
                .collect();
            c.line(&points, violet, 1.0);
        });
    }
}

fn galaxy(c: &Painter<'_>, t: f64) -> Result<(), JsValue> {
    c.rect([0.0, 0.0, 1200.0, 675.0], 0.0, NIGHT, None);
    for (p, r, color) in [
        ((780.0, 250.0), 610.0, "rgba(71,50,146,0.25)"),
        ((310.0, 450.0), 450.0, "rgba(15,113,133,0.16)"),
        ((1060.0, 490.0), 370.0, "rgba(97,47,139,0.16)"),
    ] {
        let gradient = c.0.create_radial_gradient(p.0, p.1, 0.0, p.0, p.1, r)?;
        gradient.add_color_stop(0.0, color)?;
        gradient.add_color_stop(1.0, "rgba(8,15,36,0)")?;
        c.0.set_fill_style_canvas_gradient(&gradient);
        c.0.fill_rect(0.0, 0.0, 1200.0, 675.0);
    }
    c.faded(0.17, || {
        for arm in 0..3 {
            let points: Vec<_> = (0..100)
                .map(|i| {
                    let v = f64::from(i) / 99.0;
                    let a = v * TAU * 1.22 + f64::from(arm) * TAU / 3.0 + t * 0.012;
                    let r = 50.0 + v * 580.0;
                    (820.0 + a.cos() * r, 251.0 + a.sin() * r * 0.36)
                })
                .collect();
            c.line(&points, if arm == 0 { CYAN } else { PURPLE }, 0.8);
        }
    });
    for i in 0..100 {
        let p = (noise(i + 2000) * 1200.0, noise(i + 3700) * 675.0);
        let bright = 0.21 + noise(i + 4000) * 0.44 + 0.1 * (t * 0.55 + i as f64).sin();
        c.faded(bright, || {
            c.dot(
                p,
                0.5 + noise(i + 5000) * 1.2,
                if i % 7 == 0 { GOLD } else { "#d5e6ff" },
            )
        });
    }
    for i in 0..5 {
        let p = (
            120.0 + noise(i + 7200) * 960.0,
            75.0 + noise(i + 7800) * 500.0,
        );
        c.faded(0.18 + 0.08 * (t * 0.4 + i as f64).sin(), || {
            c.spark(p, 4.0, CYAN, 0.0)
        });
    }
    Ok(())
}

fn attention(c: &Painter<'_>, t: f64) {
    let colors = [CYAN, PURPLE, GOLD, "#ff9ca9", CYAN, PURPLE];
    let focus = (t * 0.25) as usize % 6;
    for i in 0..6 {
        for j in i + 1..6 {
            let a = (220.0 + i as f64 * 150.0, 327.0);
            let b = (220.0 + j as f64 * 150.0, 327.0);
            let height = 38.0 + (j - i) as f64 * 50.0;
            let path = [a, (a.0, 327.0 - height), (b.0, 327.0 - height), b];
            let active = i == focus || j == focus;
            c.faded(if active { 0.7 } else { 0.16 }, || {
                c.curve(path, colors[i], if active { 1.8 } else { 1.0 })
            });
            if active {
                c.signal(path, t * 0.22 + i as f64 * 0.13, colors[j], 4.2);
            }
        }
    }
    for (i, color) in colors.iter().enumerate() {
        let p = (220.0 + i as f64 * 150.0, 327.0);
        c.faded(0.08, || c.dot(p, 51.0, color));
        c.dot(p, 27.0, NIGHT);
        c.ring(p, 27.0, color, 2.0);
        c.0.set_font("400 24px Georgia, serif");
        c.0.set_text_align("center");
        c.0.set_text_baseline("middle");
        c.0.set_fill_style_str(color);
        let _ = c.0.fill_text(&(i + 1).to_string(), p.0, p.1 + 1.0);
        let path = [
            (p.0, 360.0),
            (p.0, 410.0),
            (p.0 + 20.0, 439.0),
            (p.0 + 20.0, 476.0),
        ];
        c.faded(0.3, || c.curve(path, color, 1.0));
        c.signal(path, t * 0.19 + i as f64 * 0.1, color, 3.0);
        // A small mixed-color vector makes the result of combining tokens
        // visible. These illustrative values are not benchmark measurements.
        for row in 0..4 {
            let y = 466.0 + f64::from(row) * 10.0;
            c.faded(0.18, || {
                c.rect([p.0 - 28.0, y, p.0 + 58.0, y + 6.0], 2.0, color, None);
            });
            let weight = 0.35 + 0.52 * (t * 0.35 + i as f64 * 0.4 + f64::from(row)).sin().abs();
            c.faded(0.45 + 0.2 * weight, || {
                c.rect(
                    [p.0 - 28.0, y, p.0 - 28.0 + 86.0 * weight, y + 6.0],
                    2.0,
                    colors[(i + row as usize) % colors.len()],
                    None,
                );
            });
        }
    }
}

/// Historical plates leave the editorial title area quiet on the left.
/// Local progress is 0..1 within this milestone, never wall-clock time.
pub fn draw_history(
    ctx: &CanvasRenderingContext2d,
    time: f64,
    phase: usize,
    local_progress: f64,
) -> Result<(), JsValue> {
    if !time.is_finite() || !local_progress.is_finite() {
        return Err(JsValue::from_str("Animation time must be finite"));
    }
    let t = time.max(0.0);
    ctx.save();
    ctx.set_global_alpha(1.0);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    let c = Painter(ctx);
    let result = (|| {
        if phase == 9 {
            galaxy(&c, t)?;
            ctx.save();
            let _ = ctx.translate(377.0, 51.0);
            let _ = ctx.scale(0.66, 0.88);
            attention(&c, t);
            ctx.restore();
        } else {
            paper(&c, t);
            if phase == 8 {
                let g = ctx.create_linear_gradient(420.0, 140.0, 1120.0, 480.0);
                let _ = g.add_color_stop(0.0, "rgba(66,134,139,0)");
                let _ = g.add_color_stop(0.5, "rgba(66,134,139,.07)");
                let _ = g.add_color_stop(1.0, "rgba(122,91,166,.12)");
                ctx.set_fill_style_canvas_gradient(&g);
                ctx.fill_rect(420.0, 80.0, 780.0, 490.0);
            }
            let reveal = (local_progress * 12.0).clamp(0.0, 1.0);
            let reveal = reveal * reveal * (3.0 - 2.0 * reveal);
            ctx.save();
            ctx.set_global_alpha(0.86 + 0.14 * reveal);
            let _ = ctx.translate(0.0, (1.0 - reveal) * 6.0);
            match phase {
                0 => tools(&c, t),
                5 => neural(&c, t, false),
                6 => neural(&c, t, true),
                7 => vision(&c, t, false),
                8 => vision(&c, t, true),
                _ => {
                    let _ = ctx.translate(354.0, 62.0);
                    let _ = ctx.scale(0.76, 0.86);
                    match phase {
                        1 => waterclock(&c, t),
                        2 => automaton(&c, t),
                        3 => loom(&c, t),
                        4 => instructions(&c, t),
                        10 => mechanical_theatre(&c, t),
                        11 => musical_boat(&c, t),
                        _ => {}
                    }
                }
            }
            ctx.restore();
        }
        Ok(())
    })();
    ctx.restore();
    result
}
