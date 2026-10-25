//! Explosive, cinematic scene animations that make history come alive.
//! Each scene is a visual transformation showing capability breakthroughs.

use crate::animation::{Easing, Tween};
use crate::particles::{ParticleColor, ParticleSystem, ParticleType};
use crate::scene::{Painter, ACCENT, CYAN, INK, PALE, PAPER, PRINT, PURPLE};

/// Stone tool impact scene: From bare hands to power unleashed
/// Duration: 6 seconds
/// Visual beats:
/// - 0-0.8s: Hammer stone descends (buildup)
/// - 0.8s: IMPACT explosion with particles and shockwaves
/// - 0.9-3s: Before/After split screen reveals transformation
/// - 3-6s: Tool capability demonstration
pub fn stone_tool_impact(c: &Painter<'_>, t: f64, particles: &mut ParticleSystem) {
    let local_t = t % 6.0;

    // Hammer stone descent (0-0.8s)
    let hammer_y = if local_t < 0.8 {
        Tween::new(150.0, 320.0, 0.8)
            .with_easing(Easing::EaseInQuad)
            .at(local_t)
    } else {
        320.0
    };

    // Impact moment (0.8-0.9s)
    if local_t >= 0.8 && local_t < 1.2 {
        let impact_t = local_t - 0.8;

        // Particle burst
        if impact_t < 0.05 {
            for _ in 0..30 {
                particles.add(
                    600.0,
                    320.0,
                    (rand_range(-1.0, 1.0) * 200.0, rand_range(-2.0, 0.5) * 150.0),
                    rand_range(0.5, 1.5),
                    ParticleColor::Custom(139, 138, 115),
                    ParticleType::Spark,
                );
            }
        }

        // Shockwave rings
        for ring in 0..3 {
            let radius = impact_t * 400.0 + f64::from(ring) * 30.0;
            let alpha = 1.0 - impact_t / 0.4;
            if alpha > 0.0 {
                c.faded(alpha * 0.7, || {
                    c.ring((600.0, 320.0), radius, "#d87935", 3.0);
                });
            }
        }

        // Flash at impact point
        if impact_t < 0.1 {
            c.faded(1.0 - impact_t / 0.1, || {
                c.ellipse((600.0, 320.0), (15.0, 15.0), Some("#ffcc66"), None, 0.0);
            });
        }
    }

    // Draw hammer stone with motion blur
    if local_t < 0.8 {
        // Trail positions for motion blur
        for i in 0..3 {
            let trail_y = hammer_y - f64::from(i) * 25.0;
            let trail_alpha = 0.15 * (1.0 - f64::from(i) / 3.0);
            c.faded(trail_alpha, || {
                draw_hammer_stone(c, 600.0, trail_y);
            });
        }
    }

    // Draw target stone
    draw_target_stone(c, 600.0, 340.0);

    // Before/After split screen (0.9-6s)
    if local_t > 0.9 {
        let split_progress = ((local_t - 0.9) / 2.0).min(1.0);
        let split_x = Tween::new(200.0, 600.0, 1.0)
            .with_easing(Easing::EaseOutCubic)
            .at(split_progress);

        // Left side: Bare hands failing
        c.0.save();
        c.0.begin_path();
        c.0.rect(0.0, 0.0, split_x, 675.0);
        c.0.clip();
        draw_bare_hands_helpless(c, 300.0, 400.0);
        c.0.restore();

        // Right side: Tool power
        c.0.save();
        c.0.begin_path();
        c.0.rect(split_x, 0.0, 1200.0 - split_x, 675.0);
        c.0.clip();
        draw_tool_destroying(c, 800.0, 400.0, local_t);
        c.0.restore();

        // Split line glow
        c.faded(0.8, || {
            for i in 0..3 {
                c.line(
                    &[(split_x, 0.0), (split_x, 675.0)],
                    ACCENT,
                    5.0 - f64::from(i) * 1.5,
                );
            }
        });
    }

    // Capability text overlay (3-6s)
    if local_t > 3.0 {
        let text_alpha = ((local_t - 3.0) / 0.5).min(1.0);
        c.faded(text_alpha, || {
            c.text(
                "POWER UNLEASHED",
                (600.0, 550.0),
                ACCENT,
                32.0,
                "center",
            );
        });
    }
}

/// Neural network awakening: City lights up in cascade
/// Duration: 20 seconds (the breakthrough deserves time)
/// Visual beats:
/// - 0-3s: Dormant grid, occasional flickers
/// - 3-7s: Cascading activation wave from bottom-left
/// - 7-11s: Capability montage (image/language/game/speed)
/// - 11-13s: Scale zoom-out (1 → 60M neurons)
/// - 13-16s: ImageNet freeze-frame moment
/// - 16-20s: Data particle storm, understanding flows
pub fn network_awakening(c: &Painter<'_>, t: f64, particles: &mut ParticleSystem) {
    let local_t = t % 20.0;
    let width = 1200.0;
    let height = 675.0;

    // Replace paper texture with deep space background
    c.0.set_fill_style_str("#0a1628");
    c.0.fill_rect(0.0, 0.0, width, height);

    // Phase 1: Dormant grid (0-3s)
    if local_t < 3.0 {
        draw_dormant_neural_grid(c, width, height, local_t);
    }

    // Phase 2: Cascading activation (3-7s)
    if local_t >= 3.0 && local_t < 7.0 {
        let cascade_t = (local_t - 3.0) / 4.0;
        let origin = (100.0, height - 100.0);

        // Draw neuron grid with cascade
        let cols = 50;
        let rows = 20;
        for i in 0..(cols * rows) {
            let x = f64::from(i % cols) * 24.0;
            let y = f64::from(i / cols) * 34.0;

            // Distance from origin determines activation time
            let dist = ((x - origin.0).powi(2) + (y - origin.1).powi(2)).sqrt();
            let light_time = dist / 1200.0; // Wave speed

            if cascade_t > light_time {
                let activation = smoothstep(light_time, light_time + 0.1, cascade_t);

                // Neuron dot
                c.faded(activation, || {
                    let size = 2.0 + activation * 2.0;
                    c.dot((x, y), size, CYAN);

                    // Glow ring when freshly activated
                    if activation > 0.8 {
                        let ring_size = 8.0 * (1.0 + (cascade_t - light_time) * 5.0);
                        c.faded(0.5 * (1.0 - (cascade_t - light_time) * 2.0).max(0.0), || {
                            c.ring((x, y), ring_size, CYAN, 1.5);
                        });
                    }
                });

                // Connection lines to neighbors (sparse)
                if i % 7 == 0 && i + cols < cols * rows {
                    let x2 = f64::from((i + cols) % cols) * 24.0;
                    let y2 = f64::from((i + cols) / cols) * 34.0;
                    c.faded(activation * 0.3, || {
                        c.line(&[(x, y), (x2, y2)], CYAN, 0.5);
                    });
                }
            }
        }

        // Emit awakening particles
        if (local_t * 10.0) as i32 % 5 == 0 {
            particles.add(
                origin.0,
                origin.1,
                (rand_range(-1.0, 1.0) * 100.0, rand_range(-1.0, 1.0) * 100.0),
                1.5,
                ParticleColor::Cyan,
                ParticleType::Glow,
            );
        }
    }

    // Phase 3: Capability montage (7-11s)
    if local_t >= 7.0 && local_t < 11.0 {
        let demo_t = local_t - 7.0;
        let demo_index = (demo_t / 1.0).floor() as usize % 4;

        match demo_index {
            0 => draw_image_recognition_demo(c, demo_t % 1.0),
            1 => draw_language_translation_demo(c, demo_t % 1.0),
            2 => draw_game_mastery_demo(c, demo_t % 1.0),
            3 => draw_speed_comparison(c, demo_t % 1.0),
            _ => {}
        }
    }

    // Phase 4: Scale shock zoom-out (11-13s)
    if local_t >= 11.0 && local_t < 13.0 {
        draw_scale_zoom_out(c, local_t - 11.0);
    }

    // Phase 5: ImageNet freeze frame (13-16s)
    if local_t >= 13.0 && local_t < 16.0 {
        draw_imagenet_freeze_frame(c, local_t - 13.0);
    }

    // Phase 6: Particle storm finale (16-20s)
    if local_t >= 16.0 {
        draw_data_particle_storm(c, particles, local_t - 16.0);
    }
}

// Helper functions for stone tool scene

fn draw_hammer_stone(c: &Painter, x: f64, y: f64) {
    let points = [
        (x - 30.0, y - 15.0),
        (x + 30.0, y - 20.0),
        (x + 35.0, y + 10.0),
        (x - 25.0, y + 15.0),
    ];
    c.polygon(&points, "#6b6a58", Some("#4a4938"));
}

fn draw_target_stone(c: &Painter, x: f64, y: f64) {
    let points = [
        (x - 50.0, y - 40.0),
        (x + 50.0, y - 35.0),
        (x + 55.0, y + 40.0),
        (x - 45.0, y + 45.0),
    ];
    c.polygon(&points, "#8b8a73", Some("#5a5848"));

    // Knapping striations
    c.faded(0.4, || {
        for i in 0..8 {
            let ly = y - 30.0 + f64::from(i) * 10.0;
            c.line(
                &[(x - 40.0, ly), (x + 40.0, ly + 5.0)],
                "#cfc9ac",
                0.6,
            );
        }
    });
}

fn draw_bare_hands_helpless(c: &Painter, x: f64, y: f64) {
    // Two hands reaching, unable to grip effectively
    c.faded(0.6, || {
        // Left hand
        for i in 0..5 {
            let fx = x - 40.0 + f64::from(i) * 10.0;
            c.line(
                &[(fx, y - 30.0), (fx + 3.0, y + 10.0)],
                INK,
                3.0,
            );
        }
        // Right hand
        for i in 0..5 {
            let fx = x + 10.0 + f64::from(i) * 10.0;
            c.line(
                &[(fx, y - 25.0), (fx + 2.0, y + 15.0)],
                INK,
                3.0,
            );
        }

        // "X" mark showing failure
        c.line(&[(x - 60.0, y - 60.0), (x + 60.0, y + 60.0)], "#cc3333", 4.0);
        c.line(&[(x + 60.0, y - 60.0), (x - 60.0, y + 60.0)], "#cc3333", 4.0);
    });
}

fn draw_tool_destroying(c: &Painter, x: f64, y: f64, t: f64) {
    // Tool striking, with impact lines showing force
    let strike_phase = (t * 2.0).sin();
    let tool_angle = -0.3 + strike_phase * 0.2;

    c.0.save();
    let _ = c.0.translate(x, y);
    let _ = c.0.rotate(tool_angle);

    // Tool blade
    c.polygon(
        &[(-50.0, -10.0), (50.0, -5.0), (55.0, 5.0), (-45.0, 10.0)],
        "#8b8a73",
        Some(INK),
    );

    // Impact lines (force visualization)
    c.faded(0.8, || {
        for i in 0..5 {
            let lx = 40.0 + f64::from(i) * 15.0;
            c.line(&[(lx, 0.0), (lx + 30.0, 0.0)], ACCENT, 2.0);
        }
    });

    c.0.restore();

    // "✓" checkmark showing success
    c.line(&[(x - 30.0, y + 80.0), (x - 10.0, y + 100.0)], "#44cc44", 5.0);
    c.line(&[(x - 10.0, y + 100.0), (x + 40.0, y + 50.0)], "#44cc44", 5.0);
}

// Helper functions for neural network scene

fn draw_dormant_neural_grid(c: &Painter, w: f64, h: f64, t: f64) {
    c.faded(0.15, || {
        for x in (0..50).map(|i| f64::from(i) * 24.0) {
            for y in (0..20).map(|i| f64::from(i) * 34.0) {
                // Occasional random flicker
                let seed = (x + y * 1000.0) as usize;
                let flicker = if noise(seed) < 0.05 {
                    0.3 + ((t * 3.0 + x / 100.0).sin() * 0.3).abs()
                } else {
                    0.2
                };

                c.faded(flicker, || {
                    c.dot((x, y), 1.5, "#444455");
                });
            }
        }
    });
}

fn draw_image_recognition_demo(c: &Painter, t: f64) {
    let x = 300.0;
    let y = 300.0;

    // Blur → Clarity transition
    if t < 0.5 {
        // Blurred mess with "?" marks
        c.faded(0.6, || {
            for i in 0..20 {
                c.dot(
                    (x + rand_at(i) * 100.0 - 50.0, y + rand_at(i + 20) * 100.0 - 50.0),
                    rand_at(i + 40) * 5.0 + 2.0,
                    PALE,
                );
            }
            c.text("?", (x, y), "#888", 48.0, "center");
        });
    } else {
        // Sharp classification label
        let alpha = (t - 0.5) / 0.5;
        c.faded(alpha, || {
            c.text("CAT", (x, y - 20.0), CYAN, 36.0, "center");
            c.text("94.7%", (x, y + 20.0), ACCENT, 28.0, "center");
        });
    }
}

fn draw_language_translation_demo(c: &Painter, t: f64) {
    let pairs = [
        ("Hello", "你好"),
        ("World", "世界"),
    ];

    let y_start = 250.0;
    for (i, (en, zh)) in pairs.iter().enumerate() {
        let y = y_start + f64::from(i as i32) * 60.0;
        let connection_alpha = t;

        c.text(en, (200.0, y), "#ccc", 24.0, "left");
        c.text(zh, (1000.0, y), CYAN, 24.0, "right");

        // Connection arc
        c.faded(connection_alpha, || {
            c.curve(
                [
                    (320.0, y),
                    (600.0, y - 30.0),
                    (600.0, y - 30.0),
                    (880.0, y),
                ],
                ACCENT,
                2.0,
            );
        });
    }
}

fn draw_game_mastery_demo(c: &Painter, t: f64) {
    // Simple board with optimal path lighting up
    let grid_size = 5;
    let cell_size = 40.0;
    let start_x = 400.0;
    let start_y = 200.0;

    for i in 0..grid_size {
        for j in 0..grid_size {
            let x = start_x + f64::from(i) * cell_size;
            let y = start_y + f64::from(j) * cell_size;

            // Optimal path
            let is_optimal = (i == j) || (i + j == grid_size - 1);
            let color = if is_optimal && t > 0.5 {
                ACCENT
            } else {
                "#333355"
            };

            c.ellipse((x, y), (15.0, 15.0), Some(color), None, 0.0);
        }
    }
}

fn draw_speed_comparison(c: &Painter, t: f64) {
    // Top: 1989 slow training
    c.text("1989", (200.0, 200.0), "#888", 20.0, "left");
    let count_1989 = (t * 10.0).floor() as i32;
    c.text(&format!("epoch {}", count_1989), (200.0, 240.0), "#888", 18.0, "left");

    // Bottom: 2012 GPU blur
    c.text("2012 GPU", (200.0, 400.0), CYAN, 20.0, "left");
    let count_2012 = (t * 10000.0).floor() as i32;
    c.text(&format!("epoch {}", count_2012), (200.0, 440.0), CYAN, 18.0, "left");

    // Speed multiplier
    if t > 0.5 {
        c.text("1000x FASTER", (600.0, 320.0), ACCENT, 32.0, "center");
    }
}

fn draw_scale_zoom_out(c: &Painter, t: f64) {
    // Exponential zoom from 1 neuron to 60M
    let scale_levels = [
        (1, "1 neuron"),
        (10, "10"),
        (100, "100"),
        (10000, "10K"),
        (1000000, "1M"),
        (60000000, "60M+"),
    ];

    let level = (t / 0.3).floor() as usize;
    if level < scale_levels.len() {
        let (count, label) = scale_levels[level];

        // Draw that many dots (capped for rendering)
        let render_count = count.min(2000);
        for i in 0..render_count {
            let angle = f64::from(i) * 2.4;
            let radius = (f64::from(i) / f64::from(render_count)) * 500.0;
            let x = 600.0 + angle.cos() * radius;
            let y = 337.0 + angle.sin() * radius;

            c.dot((x, y), 1.5, CYAN);
        }

        // Label
        c.text(label, (600.0, 600.0), ACCENT, 48.0, "center");
    }
}

fn draw_imagenet_freeze_frame(c: &Painter, t: f64) {
    // FREEZE: The ImageNet moment
    c.0.set_fill_style_str("#000000");
    c.0.fill_rect(0.0, 0.0, 1200.0, 675.0);

    // Victory number
    c.text("15.3%", (600.0, 250.0), ACCENT, 96.0, "center");
    c.text("ERROR RATE", (600.0, 320.0), CYAN, 32.0, "center");

    // Year stamp
    c.text("2012", (600.0, 400.0), "#fff", 56.0, "center");

    // Plunging graph line (simplified)
    if t > 0.5 {
        let points: Vec<(f64, f64)> = (0..20)
            .map(|i| {
                let x = 200.0 + f64::from(i) * 40.0;
                let y = if i < 15 {
                    500.0 - f64::from(i) * 5.0
                } else {
                    // Plunge at AlexNet
                    500.0 - 75.0 - f64::from(i - 15) * 40.0
                };
                (x, y)
            })
            .collect();

        c.line(&points, CYAN, 3.0);
    }
}

fn draw_data_particle_storm(c: &Painter, particles: &mut ParticleSystem, t: f64) {
    // Data flows in from left, emerges understood on right
    if (t * 20.0) as i32 % 2 == 0 {
        particles.add(
            50.0,
            rand_range(100.0, 575.0),
            (250.0, rand_range(-20.0, 20.0)),
            2.0,
            ParticleColor::Custom(180, 180, 200),
            ParticleType::Dot,
        );
    }

    // Central network glow
    c.faded(0.6, || {
        for i in 0..10 {
            let radius = 50.0 + f64::from(i) * 30.0;
            c.faded(1.0 / (f64::from(i + 1)), || {
                c.ring((600.0, 337.0), radius, CYAN, 2.0);
            });
        }
    });

    // Text overlay
    c.text("UNDERSTANDING", (600.0, 600.0), ACCENT, 36.0, "center");
    c.text("FLOWS", (600.0, 640.0), CYAN, 28.0, "center");
}

// Utility functions

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn rand_range(min: f64, max: f64) -> f64 {
    min + (max - min) * 0.5 // Deterministic placeholder; replace with actual random
}

fn rand_at(seed: usize) -> f64 {
    ((seed * 2654435761) % 1000) as f64 / 1000.0
}

fn noise(seed: usize) -> f64 {
    ((seed * 1103515245 + 12345) % 1000) as f64 / 1000.0
}
