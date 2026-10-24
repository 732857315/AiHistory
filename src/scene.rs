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

fn paper(c: &Painter<'_>, t: f64) {
    c.rect([0.0, 0.0, 1200.0, 675.0], 0.0, PAPER, None);
    c.faded(0.15, || {
        for i in 0..95 {
            let x = noise(i) * 1200.0;
            let y = noise(i + 191) * 675.0;
            c.line(
                &[(x, y), (x + 1.0 + noise(i + 641) * 5.0, y + 0.7)],
                PRINT,
                0.7,
            );
        }
        c.ring((610.0, 306.0), 235.0, PRINT, 0.8);
        c.ring((610.0, 306.0), 241.0, PRINT, 0.5);
        for x in [42.0, 1158.0] {
            c.line(&[(x, 55.0), (x, 585.0)], PRINT, 0.7);
        }
        for y in [54.0, 61.0, 582.0, 588.0] {
            c.line(&[(60.0, y), (1140.0, y)], PRINT, 0.7);
        }
    });
    let path = [
        (60.0, 548.0),
        (380.0, 592.0),
        (800.0, 515.0),
        (1140.0, 554.0),
    ];
    c.faded(0.35, || c.curve(path, ACCENT, 1.4));
    c.signal(path, t / 17.0, ACCENT, 3.4);
}

fn tools(c: &Painter<'_>, t: f64) {
    c.faded(0.10, || {
        c.ellipse((628.0, 506.0), (290.0, 23.0), Some(INK), None, 0.0)
    });
    c.polygon(
        &[
            (180.0, 339.0),
            (298.0, 302.0),
            (366.0, 243.0),
            (405.0, 236.0),
            (451.0, 274.0),
            (468.0, 329.0),
            (437.0, 367.0),
            (337.0, 381.0),
            (234.0, 421.0),
        ],
        PALE,
        Some(INK),
    );
    for i in 0..4 {
        let y = 270.0 + f64::from(i) * 24.0;
        c.curve(
            [
                (330.0, y),
                (374.0, y - 25.0),
                (426.0, y + 4.0),
                (445.0, y + 14.0),
            ],
            PRINT,
            1.5,
        );
    }
    c.hatch([220.0, 345.0, 315.0, 384.0], 6.0, PRINT);
    c.polygon(
        &[
            (582.0, 123.0),
            (651.0, 158.0),
            (703.0, 249.0),
            (713.0, 352.0),
            (661.0, 464.0),
            (558.0, 496.0),
            (481.0, 428.0),
            (477.0, 319.0),
            (520.0, 211.0),
        ],
        INK,
        Some(INK),
    );
    for (points, color) in [
        (
            vec![(582.0, 123.0), (605.0, 247.0), (520.0, 211.0)],
            "#a6977d",
        ),
        (
            vec![
                (605.0, 247.0),
                (703.0, 249.0),
                (652.0, 353.0),
                (562.0, 343.0),
            ],
            PRINT,
        ),
        (
            vec![(481.0, 428.0), (562.0, 343.0), (558.0, 496.0)],
            "#b1a184",
        ),
        (
            vec![(651.0, 158.0), (605.0, 247.0), (703.0, 249.0)],
            "#554d40",
        ),
        (
            vec![
                (652.0, 353.0),
                (713.0, 352.0),
                (661.0, 464.0),
                (558.0, 496.0),
            ],
            "#8a7c65",
        ),
    ] {
        c.polygon(&points, color, None);
    }
    for i in 0..10 {
        let x = 537.0 + f64::from(i) * 9.0;
        c.line(&[(x, 390.0), (x - 26.0, 443.0)], PALE, 0.6);
    }
    c.line(
        &[
            (582.0, 123.0),
            (520.0, 211.0),
            (477.0, 319.0),
            (481.0, 428.0),
        ],
        ACCENT,
        3.2,
    );
    let p = bezier(
        [
            (582.0, 125.0),
            (528.0, 190.0),
            (472.0, 313.0),
            (483.0, 422.0),
        ],
        (t * 0.4).rem_euclid(1.0),
    );
    c.spark(p, 8.0 + 3.0 * (t * 3.0).sin(), ACCENT, 0.0);
    for i in 0..5 {
        let n = f64::from(i);
        let x = 793.0 + n * 33.0;
        let y = 323.0 + (n * 1.4 + t * 0.4).sin() * 36.0;
        c.polygon(
            &[(x, y), (x + 22.0, y - 10.0), (x + 13.0, y + 21.0)],
            if i % 2 == 0 { PRINT } else { PALE },
            Some(INK),
        );
    }
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
    c.rect([228.0, 492.0, 968.0, 510.0], 3.0, INK, None);
    for x in [268.0, 915.0] {
        c.rect([x, 509.0, x + 16.0, 551.0], 0.0, PRINT, None);
    }
    c.polygon(
        &[
            (416.0, 253.0),
            (532.0, 252.0),
            (565.0, 401.0),
            (391.0, 401.0),
        ],
        PALE,
        Some(INK),
    );
    c.ellipse((475.0, 191.0), (49.0, 58.0), Some(INK), None, 0.0);
    c.polygon(&[(507.0, 183.0), (535.0, 205.0), (508.0, 212.0)], INK, None);
    c.rect([449.0, 237.0, 497.0, 258.0], 0.0, PRINT, Some(INK));
    c.dot((506.0, 184.0), 3.0, PAPER);
    c.line(&[(413.0, 400.0), (414.0, 482.0), (449.0, 482.0)], INK, 23.0);
    c.line(&[(531.0, 400.0), (548.0, 482.0), (580.0, 482.0)], INK, 23.0);
    gear(c, (471.0, 331.0), 54.0, t * 0.3);
    gear(c, (382.0, 395.0), 43.0, -t * 0.38);
    c.line(
        &[(418.0, 279.0), (347.0, 331.0), (382.0, 395.0)],
        PRINT,
        15.0,
    );
    for p in [(418.0, 279.0), (347.0, 331.0)] {
        c.dot(p, 11.0, INK);
        c.dot(p, 4.0, PAPER);
    }
    c.polygon(
        &[
            (663.0, 412.0),
            (915.0, 412.0),
            (1000.0, 491.0),
            (706.0, 491.0),
        ],
        "#fbf4e5",
        Some(PRINT),
    );
    let pen = (
        815.0 + 49.0 * (t * 0.75).sin(),
        449.0 + 13.0 * (t * 1.5).sin(),
    );
    let elbow = (641.0 + 11.0 * (t * 0.75).sin(), 344.0);
    let wrist = (pen.0 - 28.0, pen.1 - 45.0);
    c.line(&[(530.0, 280.0), elbow, wrist], INK, 21.0);
    c.line(&[(532.0, 277.0), elbow, wrist], PALE, 12.0);
    for p in [(530.0, 280.0), elbow, wrist] {
        c.dot(p, 12.0, INK);
        c.dot(p, 5.0, PAPER);
    }
    c.line(&[(wrist.0 + 2.0, wrist.1 - 15.0), pen], INK, 4.0);
    c.curve(
        [
            (746.0, 459.0),
            (794.0, 422.0),
            (835.0, 489.0),
            (892.0, 448.0),
        ],
        ACCENT,
        2.8,
    );
    c.dot(pen, 3.8, ACCENT);
}

/// An explanatory cutaway, not a claim to reconstruct Heron's exact machine.
fn mechanical_theatre(c: &Painter<'_>, t: f64) {
    c.faded(0.08, || {
        c.ellipse((604.0, 545.0), (400.0, 17.0), Some(INK), None, 0.0);
    });
    c.rect([268.0, 151.0, 934.0, 428.0], 0.0, "#e7dcc6", Some(INK));
    c.faded(0.45, || c.hatch([286.0, 176.0, 913.0, 402.0], 16.0, PALE));
    c.line(&[(290.0, 385.0), (912.0, 385.0)], PRINT, 1.0);
    // The actors have visible pivots and repeat a mechanically arranged gesture.
    for (i, x) in [426.0, 609.0, 791.0].into_iter().enumerate() {
        let angle = t * 1.7 + i as f64 * 1.2;
        let sway = angle.sin() * 8.0;
        let shoulder = (x + sway, 288.0);
        c.ellipse((x + sway, 248.0), (21.0, 27.0), Some(INK), None, 0.0);
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
    c.rect([242.0, 128.0, 960.0, 153.0], 2.0, INK, None);
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
    let x = 588.0 + 176.0 * (t * 1.1).sin();
    c.polygon(
        &[(x - 58.0, 341.0), (x, 325.0), (x + 58.0, 341.0), (x, 355.0)],
        INK,
        Some(INK),
    );
    c.line(&[(x - 30.0, 341.0), (x + 30.0, 341.0)], ACCENT, 4.0);
    c.rect([886.0, 187.0, 1041.0, 345.0], 3.0, "#e5d8bd", Some(PRINT));
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
    let nodes: Vec<Point> = if layered {
        [(278.0, 3), (490.0, 4), (701.0, 3), (920.0, 1)]
            .iter()
            .flat_map(|(x, count)| {
                (0..*count).map(move |i| {
                    (
                        *x,
                        320.0 + (f64::from(i) - f64::from(*count - 1) / 2.0) * 86.0,
                    )
                })
            })
            .collect()
    } else {
        vec![
            (286.0, 298.0),
            (405.0, 163.0),
            (417.0, 423.0),
            (564.0, 132.0),
            (582.0, 297.0),
            (573.0, 469.0),
            (730.0, 180.0),
            (765.0, 397.0),
            (886.0, 291.0),
        ]
    };
    for (i, a) in nodes.iter().enumerate() {
        for (j, b) in nodes.iter().enumerate().skip(i + 1) {
            let dx = b.0 - a.0;
            if dx > 75.0 && dx < 270.0 && (layered || (b.1 - a.1).abs() < 240.0) {
                let path = [*a, (a.0 + 90.0, a.1), (b.0 - 90.0, b.1), *b];
                c.faded(0.5, || c.curve(path, PRINT, 1.3));
                if (i + j) % 3 == 0 {
                    c.signal(path, t * 0.22 + i as f64 * 0.16, ACCENT, 3.8);
                }
            }
        }
    }
    for (i, p) in nodes.iter().enumerate() {
        let r = if i == 4 && !layered { 34.0 } else { 22.0 };
        c.faded(0.1, || {
            c.dot(*p, r + 12.0 + 3.0 * (t * 1.6 - i as f64).sin(), ACCENT)
        });
        c.dot(*p, r, INK);
        c.ring(*p, r + 5.0, PRINT, 1.0);
        c.dot((p.0 - 5.0, p.1 - 5.0), 4.8, PAPER);
    }
}

fn vision(c: &Painter<'_>, t: f64, electric: bool) {
    let colors = if electric {
        [CYAN, PURPLE, GOLD]
    } else {
        [INK, PRINT, ACCENT]
    };
    let (x, y) = (156.0, 220.0);
    c.rect(
        [x - 12.0, y - 12.0, x + 222.0, y + 222.0],
        2.0,
        if electric { "#e4e9df" } else { "#e9ddc7" },
        Some(INK),
    );
    let pixels = [
        0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0,
    ];
    for (i, value) in pixels.iter().enumerate() {
        let xx = x + (i % 5) as f64 * 42.0;
        let yy = y + (i / 5) as f64 * 42.0;
        c.rect(
            [xx + 2.0, yy + 2.0, xx + 39.0, yy + 39.0],
            0.0,
            if *value == 1 { INK } else { PAPER },
            None,
        );
    }
    let scan = ((t * 0.24).rem_euclid(1.0) * 9.0).floor() as usize;
    let sx = x + (scan % 3) as f64 * 42.0;
    let sy = y + (scan / 3) as f64 * 42.0;
    c.faded(0.18, || {
        c.rect([sx, sy, sx + 126.0, sy + 126.0], 0.0, colors[2], None)
    });
    c.line(
        &[
            (sx, sy),
            (sx + 126.0, sy),
            (sx + 126.0, sy + 126.0),
            (sx, sy + 126.0),
            (sx, sy),
        ],
        colors[2],
        2.2,
    );
    let count = if electric { 5 } else { 3 };
    for layer in 0..count {
        let lx = 480.0 + f64::from(layer) * 67.0;
        let ly = 225.0 - f64::from(layer) * 13.0;
        c.polygon(
            &[
                (lx, ly),
                (lx + 84.0, ly - 48.0),
                (lx + 84.0, ly + 142.0),
                (lx, ly + 190.0),
            ],
            if electric {
                colors[(layer % 3) as usize]
            } else {
                PALE
            },
            Some(INK),
        );
        for row in 0..4 {
            for col in 0..3 {
                let xx = lx + 12.0 + f64::from(col) * 25.0;
                let yy = ly + 22.0 + f64::from(row) * 39.0 - f64::from(col) * 14.0;
                c.dot(
                    (xx, yy),
                    3.4,
                    if (row + col + layer + scan as i32) % 4 == 0 {
                        colors[2]
                    } else {
                        PRINT
                    },
                );
            }
        }
    }
    let path = [
        (383.0, 322.0),
        (427.0, 322.0),
        (433.0, 313.0),
        (480.0, 313.0),
    ];
    c.curve(path, PRINT, 2.0);
    c.signal(path, t * 0.6, colors[2], 4.5);
    c.dot((1000.0, 318.0), 57.0, INK);
    c.ring((1000.0, 318.0), 65.0, colors[1], 1.2);
    c.line(
        &[(974.0, 320.0), (994.0, 340.0), (1030.0, 296.0)],
        if electric { CYAN } else { PAPER },
        7.0,
    );
    let path = [
        (if electric { 832.0 } else { 704.0 }, 309.0),
        (875.0, 291.0),
        (900.0, 317.0),
        (938.0, 318.0),
    ];
    c.faded(0.6, || c.curve(path, colors[1], 1.4));
    c.signal(path, t * 0.4, colors[2], 4.0);
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
        c.spark(p, 13.0, color, i as f64 * PI / 4.0);
        let path = [
            (p.0, 360.0),
            (p.0, 410.0),
            (p.0 + 20.0, 439.0),
            (p.0 + 20.0, 476.0),
        ];
        c.faded(0.3, || c.curve(path, color, 1.0));
        c.signal(path, t * 0.19 + i as f64 * 0.1, color, 3.0);
        c.faded(0.20, || {
            c.rect([p.0 - 22.0, 476.0, p.0 + 62.0, 493.0], 5.0, color, None)
        });
    }
}

fn racer(
    c: &Painter<'_>,
    t: f64,
    racer: &Racer<'_>,
    index: usize,
    gap: f64,
    top: f64,
) -> Result<(), JsValue> {
    let position = if racer.position.is_finite() {
        racer.position.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let x = 260.0 + position * 790.0;
    let y = top + index as f64 * gap;
    c.faded(0.14, || {
        if racer.rank == 0 {
            for tick in 0..32 {
                let start = 160.0 + f64::from(tick) * 30.0;
                c.line(&[(start, y), (start + 12.0, y)], racer.color, 0.7);
            }
        } else {
            c.line(&[(160.0, y), (1120.0, y)], racer.color, 0.7);
        }
    });
    // Animated propulsion never moves the model away from its supplied score.
    for segment in 0..8 {
        let n = f64::from(segment);
        let distance = 18.0 + n * 16.0;
        let bend = (n * 0.47 + t * 1.8 + index as f64).sin() * 2.4;
        c.faded((1.0 - n / 9.0) * 0.31, || {
            c.line(
                &[(x - distance - 14.0, y + bend), (x - distance, y + bend)],
                racer.color,
                4.6 - n * 0.39,
            )
        });
    }
    c.faded(0.06, || c.dot((x, y), 36.0, racer.color));
    c.faded(0.15, || c.dot((x, y), 23.0, racer.color));
    c.dot((x, y), 11.5, racer.color);
    c.dot((x - 2.0, y - 2.0), 4.0, "#ffffff");
    if racer.rank == 1 {
        c.ring((x, y), 17.5, GOLD, 1.1);
    } else if racer.rank == 0 {
        c.ring((x, y), 17.5, "#8497b6", 0.8);
    }
    let orbit = t * 0.8 + index as f64;
    c.faded(0.55, || {
        c.dot(
            (x + 20.0 * orbit.cos(), y + 9.0 * orbit.sin()),
            1.8,
            racer.color,
        )
    });
    let label_x = (x - 76.0).clamp(156.0, 870.0);
    c.rect(
        [label_x - 9.0, y - 52.0, label_x + 253.0, y - 9.0],
        6.0,
        "rgba(8,15,36,0.88)",
        None,
    );
    c.text(racer.name, (label_x, y - 35.0), 17.0, "#f3f6ff", 245.0)?;
    c.text(
        racer.organization,
        (label_x, y - 17.0),
        11.0,
        "#9cadc8",
        190.0,
    )?;
    if let Some(score) = racer.score.filter(|v| v.is_finite()) {
        c.0.set_text_align("right");
        c.0.set_font("500 12px system-ui, sans-serif");
        c.0.set_fill_style_str(racer.color);
        c.0.fill_text(&format!("{score:.1}"), label_x + 244.0, y - 17.0)?;
    }
    c.text(
        &if racer.rank > 0 {
            format!("{:02}", racer.rank)
        } else {
            "—".to_owned()
        },
        (111.0, y),
        13.0,
        if racer.rank == 1 { GOLD } else { "#8b9fbd" },
        35.0,
    )?;
    Ok(())
}

fn race(c: &Painter<'_>, t: f64, racers: &[Racer<'_>]) -> Result<(), JsValue> {
    let count = racers.len().min(8);
    let gap = if count > 6 { 58.0 } else { 72.0 };
    let top = if count > 6 { 126.0 } else { 156.0 };
    c.faded(0.11, || {
        for i in 0..6 {
            let x = 260.0 + f64::from(i) * 158.0;
            c.line(&[(x, 95.0), (x, 604.0)], "#91b7d3", 0.7);
        }
    });
    for (i, item) in racers.iter().take(8).enumerate() {
        let opacity = if item.appeared.is_finite() {
            item.appeared.clamp(0.0, 1.0)
        } else {
            0.0
        };
        if opacity <= 0.0 {
            continue;
        }
        c.0.save();
        c.0.set_global_alpha(c.0.global_alpha() * opacity);
        let result = racer(c, t, item, i, gap, top);
        c.0.restore();
        result?;
    }
    Ok(())
}

/// Draw in 1200×675 logical coordinates. Caller owns canvas scaling and playback.
/// Unchanged time and data produce the same frame, including in reduced motion.
pub fn draw(
    ctx: &CanvasRenderingContext2d,
    time: f64,
    racers: &[Racer<'_>],
    early_phase: Option<usize>,
) -> Result<(), JsValue> {
    if !time.is_finite() {
        return Err(JsValue::from_str("Animation time must be finite"));
    }
    let t = time.max(0.0);
    ctx.save();
    ctx.set_global_alpha(1.0);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    let c = Painter(ctx);
    let result = (|| {
        match early_phase {
            Some(phase) if phase != 9 => {
                paper(&c, t);
                match phase {
                    0 => tools(&c, t),
                    1 => waterclock(&c, t),
                    2 => automaton(&c, t),
                    3 => loom(&c, t),
                    4 => instructions(&c, t),
                    5 => neural(&c, t, false),
                    6 => neural(&c, t, true),
                    7 => vision(&c, t, false),
                    10 => mechanical_theatre(&c, t),
                    11 => musical_boat(&c, t),
                    _ => vision(&c, t, true),
                }
            }
            Some(_) => {
                galaxy(&c, t)?;
                attention(&c, t);
            }
            None => {
                galaxy(&c, t)?;
                race(&c, t, racers)?;
            }
        }
        Ok(())
    })();
    ctx.restore();
    result
}
