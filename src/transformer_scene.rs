//! Transformer revolution scene: The arrival of true understanding
//! This is THE PRESENT MOMENT — make it feel like we're living in the explosion

use crate::animation::{Easing, Tween};
use crate::particles::{ParticleColor, ParticleSystem, ParticleType};
use crate::scene::{Painter, ACCENT, CYAN, INK, PAPER, PRINT, PURPLE};

/// Transformer revolution: 6-act structure showing attention emergence
/// Duration: 20 seconds (the current revolution deserves full treatment)
///
/// Act 1 (0-4s): Attention awakening - words looking at each other
/// Act 2 (4-8s): Language emerges from chaos
/// Act 3 (8-12s): Scale explosion GPT-1 → GPT-2 → GPT-3
/// Act 4 (12-16s): Capability shock moments
/// Act 5 (16-18s): Recursive understanding (meta-cognition)
/// Act 6 (18-20s): Convergence of all AI history into this moment
pub fn transformer_revolution(c: &Painter<'_>, t: f64, particles: &mut ParticleSystem) {
    let local_t = t % 20.0;

    // Act 1: Attention Awakening (0-4s)
    if local_t < 4.0 {
        draw_attention_awakening(c, local_t);
    }

    // Act 2: Language Emergence (4-8s)
    if local_t >= 4.0 && local_t < 8.0 {
        draw_language_emergence(c, local_t - 4.0);
    }

    // Act 3: Scale Explosion (8-12s)
    if local_t >= 8.0 && local_t < 12.0 {
        draw_scale_explosion(c, particles, local_t - 8.0);
    }

    // Act 4: Capability Shock (12-16s)
    if local_t >= 12.0 && local_t < 16.0 {
        draw_capability_shock(c, local_t - 12.0);
    }

    // Act 5: Recursive Understanding (16-18s)
    if local_t >= 16.0 && local_t < 18.0 {
        draw_recursive_understanding(c, local_t - 16.0);
    }

    // Act 6: Historical Convergence (18-20s)
    if local_t >= 18.0 {
        draw_historical_convergence(c, particles, local_t - 18.0);
    }
}

fn draw_attention_awakening(c: &Painter, t: f64) {
    let words = ["The", "cat", "sat", "on", "the", "mat"];
    let y = 300.0;
    let spacing = 150.0;
    let start_x = 150.0;

    // Draw words
    for (i, word) in words.iter().enumerate() {
        let x = start_x + f64::from(i as i32) * spacing;
        c.text(word, (x, y), "#eee", 24.0, "center");

        // Word focus indicator (cycles through)
        let focused = (t * 2.0).floor() as usize % words.len();
        if i == focused {
            c.ring((x, y), 35.0, ACCENT, 2.5);
        }
    }

    // Attention connections (every word to every other word)
    let connection_progress = (t / 4.0).min(1.0);
    for i in 0..words.len() {
        for j in (i + 1)..words.len() {
            let x1 = start_x + f64::from(i as i32) * spacing;
            let x2 = start_x + f64::from(j as i32) * spacing;

            // Attention weight varies by time
            let attention_phase = t * 0.5 + f64::from(i as i32) * 0.3;
            let weight = 0.5 + attention_phase.sin() * 0.5;
            let thickness = 1.0 + weight * 3.0;

            c.faded(connection_progress * (0.3 + weight * 0.3), || {
                // Bezier arc connecting words
                c.curve(
                    [
                        (x1, y + 10.0),
                        (x1, y - 50.0),
                        (x2, y - 50.0),
                        (x2, y + 10.0),
                    ],
                    CYAN,
                    thickness,
                );
            });
        }
    }

    // Attention matrix grid overlay
    if t > 2.0 {
        let grid_alpha = ((t - 2.0) / 1.0).min(1.0) * 0.2;
        c.faded(grid_alpha, || {
            // 6x6 grid showing attention scores
            let grid_x = 50.0;
            let grid_y = 450.0;
            let cell_size = 30.0;

            for i in 0..6 {
                for j in 0..6 {
                    let intensity = if i == j {
                        1.0
                    } else {
                        0.3 + ((t + f64::from(i * j)) * 0.5).sin() * 0.3
                    };

                    c.faded(intensity, || {
                        c.ellipse(
                            (
                                grid_x + f64::from(i) * cell_size,
                                grid_y + f64::from(j) * cell_size,
                            ),
                            (10.0, 10.0),
                            Some(CYAN),
                            None,
                            0.0,
                        );
                    });
                }
            }
        });
    }

    // Text label
    if t > 1.0 {
        c.faded((t - 1.0).min(1.0), || {
            c.text("ATTENTION", (600.0, 550.0), PURPLE, 28.0, "center");
        });
    }
}

fn draw_language_emergence(c: &Painter, t: f64) {
    let emergence_progress = (t / 4.0).min(1.0);

    // Character rain turning into coherent text
    for i in 0..80 {
        let x = f64::from(i % 16) * 75.0 + 50.0;
        let y_offset = (t * 60.0 + f64::from(i) * 20.0) % 700.0;

        if emergence_progress < 0.3 {
            // Chaos: random characters
            let chaos_char = match i % 4 {
                0 => "█",
                1 => "▓",
                2 => "░",
                _ => "▒",
            };
            c.faded(0.4, || {
                c.text(chaos_char, (x, y_offset), "#666", 16.0, "center");
            });
        } else if emergence_progress < 0.6 {
            // Word fragments forming
            let fragments = ["the", "cat", "is", "AI", "can", "now"];
            let frag = fragments[i % fragments.len()];
            c.faded(emergence_progress - 0.3, || {
                c.text(frag, (x, y_offset), CYAN, 18.0, "center");
            });
        } else {
            // Coherent sentences
            if i % 16 < 12 {
                let sentences = [
                    "Language",
                    "becomes",
                    "native",
                    "element",
                ];
                if i / 16 < sentences.len() {
                    let word = sentences[i / 16];
                    let word_x = 300.0 + f64::from((i % 16) as i32) * 60.0;
                    let word_y = 250.0 + f64::from((i / 16) as i32) * 60.0;

                    c.faded(emergence_progress, || {
                        let size = 20.0 + emergence_progress * 10.0;
                        let color = if emergence_progress > 0.8 {
                            ACCENT
                        } else {
                            CYAN
                        };
                        c.text(word, (word_x, word_y), color, size, "center");
                    });
                }
            }
        }
    }

    // Luminosity increase as understanding deepens
    if emergence_progress > 0.5 {
        let glow_alpha = (emergence_progress - 0.5) * 0.3;
        c.faded(glow_alpha, || {
            for i in 0..5 {
                let radius = 200.0 + f64::from(i) * 80.0;
                c.ring((600.0, 337.0), radius, PURPLE, 1.0);
            }
        });
    }

    // Progress label
    let stage = if emergence_progress < 0.3 {
        "CHAOS"
    } else if emergence_progress < 0.6 {
        "WORDS"
    } else {
        "UNDERSTANDING"
    };

    c.text(stage, (600.0, 600.0), ACCENT, 32.0, "center");
}

fn draw_scale_explosion(c: &Painter, particles: &mut ParticleSystem, t: f64) {
    let center = (600.0, 337.0);

    // Three concentric model rings: GPT-1, GPT-2, GPT-3
    let models = [
        ("GPT-1", 40.0, 80.0, "117M", PRINT, 0.0),
        ("GPT-2", 120.0, 200.0, "1.5B", PURPLE, 1.0),
        ("GPT-3", 200.0, 400.0, "175B", CYAN, 2.0),
    ];

    for (name, min_r, max_r, params, color, delay) in models.iter() {
        let model_t = (t - delay).max(0.0);
        if model_t > 0.0 {
            // Ring expansion
            let expansion = (model_t / 1.2).min(1.0);
            let radius = Tween::new(*min_r, *max_r, 1.2)
                .with_easing(Easing::EaseOutElastic)
                .at(model_t);

            // Draw ring
            c.faded(expansion, || {
                c.ring(center, radius, color, 6.0 + expansion * 4.0);
            });

            // Parameter count label
            if expansion > 0.5 {
                let label_alpha = (expansion - 0.5) * 2.0;
                c.faded(label_alpha, || {
                    let label_x = center.0 + radius * 0.707;
                    let label_y = center.1 - radius * 0.707;
                    c.text(params, (label_x, label_y), color, 24.0, "center");
                });
            }

            // GPT-3 explosion particles
            if name == &"GPT-3" && model_t > 0.8 && model_t < 2.0 {
                if (model_t * 20.0) as i32 % 3 == 0 {
                    for i in 0..8 {
                        let angle = f64::from(i) * std::f64::consts::PI / 4.0;
                        particles.add(
                            center.0,
                            center.1,
                            (angle.cos() * 200.0, angle.sin() * 200.0),
                            1.5,
                            ParticleColor::Cyan,
                            ParticleType::Star,
                        );
                    }
                }
            }
        }
    }

    // Scale comparison text
    if t > 2.5 {
        let text_alpha = ((t - 2.5) / 0.5).min(1.0);
        c.faded(text_alpha, || {
            c.text("SCALE", (600.0, 100.0), ACCENT, 48.0, "center");
            c.text("=", (600.0, 150.0), "#fff", 36.0, "center");
            c.text("MAGIC", (600.0, 200.0), ACCENT, 48.0, "center");
        });
    }
}

fn draw_capability_shock(c: &Painter, t: f64) {
    // 4-panel rapid-fire capability showcase
    let demos = [
        ("CODE", "def hello():\n  print('Hi')", 0.0, (200.0, 200.0)),
        ("TRANSLATE", "EN → 中文 → 日本語", 1.0, (1000.0, 200.0)),
        ("MATH", "∑ ∫ ∂ → SOLVED", 2.0, (200.0, 500.0)),
        ("EXAM", "Bar Exam: 90th %ile", 3.0, (1000.0, 500.0)),
    ];

    for (title, content, delay, pos) in demos.iter() {
        let demo_t = (t - delay).max(0.0);
        if demo_t > 0.0 && demo_t < 1.5 {
            let alpha = (demo_t / 0.3).min(1.0);

            c.faded(alpha, || {
                // Panel background
                c.0.set_fill_style_str("rgba(20, 30, 50, 0.8)");
                c.0.fill_rect(pos.0 - 150.0, pos.1 - 80.0, 300.0, 160.0);

                // Title
                c.text(title, (pos.0, pos.1 - 40.0), ACCENT, 24.0, "center");

                // Content
                c.text(content, (pos.0, pos.1 + 20.0), CYAN, 18.0, "center");

                // Shock starburst
                if demo_t < 0.2 {
                    for i in 0..8 {
                        let angle = f64::from(i) * std::f64::consts::PI / 4.0;
                        let len = 50.0 * (1.0 - demo_t / 0.2);
                        c.line(
                            &[
                                (pos.0, pos.1),
                                (pos.0 + angle.cos() * len, pos.1 + angle.sin() * len),
                            ],
                            ACCENT,
                            2.0,
                        );
                    }
                }
            });
        }
    }

    // Central headline
    if t > 2.5 {
        c.faded((t - 2.5).min(1.0), || {
            c.text("IMPOSSIBLE", (600.0, 320.0), "#fff", 36.0, "center");
            c.text("→", (600.0, 360.0), ACCENT, 32.0, "center");
            c.text("ROUTINE", (600.0, 400.0), CYAN, 36.0, "center");
        });
    }
}

fn draw_recursive_understanding(c: &Painter, t: f64) {
    let center = (600.0, 337.0);

    // Möbius strip / infinity symbol formed by token flow
    let loop_progress = (t / 2.0).min(1.0);

    // Draw infinity symbol path
    for i in 0..100 {
        let phase = f64::from(i) / 100.0 * std::f64::consts::PI * 2.0;
        let scale = 200.0;

        // Infinity curve parametric
        let x = center.0 + scale * (phase.sin() / (1.0 + phase.cos().powi(2)));
        let y = center.1 + scale * (phase.sin() * phase.cos() / (1.0 + phase.cos().powi(2)));

        if f64::from(i) / 100.0 < loop_progress {
            c.dot((x, y), 3.0, PURPLE);

            // Flow direction indicator
            if i % 10 == 0 {
                let next_phase = f64::from(i + 1) / 100.0 * std::f64::consts::PI * 2.0;
                let next_x = center.0 + scale * (next_phase.sin() / (1.0 + next_phase.cos().powi(2)));
                let next_y = center.1 + scale * (next_phase.sin() * next_phase.cos() / (1.0 + next_phase.cos().powi(2)));

                c.line(&[(x, y), (next_x, next_y)], CYAN, 2.0);
            }
        }
    }

    // Labels
    if t > 0.5 {
        c.text("THOUGHT", (400.0, 250.0), PURPLE, 20.0, "center");
        c.text("↓", (400.0, 290.0), "#fff", 24.0, "center");
        c.text("META-THOUGHT", (400.0, 330.0), CYAN, 20.0, "center");
    }

    if t > 1.0 {
        c.text("RECURSION", (600.0, 550.0), ACCENT, 36.0, "center");
    }
}

fn draw_historical_convergence(c: &Painter, particles: &mut ParticleSystem, t: f64) {
    let center = (600.0, 337.0);
    let convergence = (t / 2.0).min(1.0);

    // Streams from different historical eras converging to center
    let streams = [
        ("1958\nPerceptron", (100.0, 100.0), PRINT),
        ("2012\nCNN", (1100.0, 100.0), PURPLE),
        ("1986\nBackprop", (100.0, 574.0), "#8899aa"),
        ("History", (1100.0, 574.0), CYAN),
    ];

    for (label, origin, color) in streams.iter() {
        // Draw stream curve from origin to center
        let ctrl_x = (origin.0 + center.0) / 2.0;
        let ctrl_y = (origin.1 + center.1) / 2.0;

        c.faded(convergence, || {
            c.curve(
                [
                    *origin,
                    (ctrl_x, ctrl_y - 50.0),
                    (ctrl_x, ctrl_y + 50.0),
                    center,
                ],
                color,
                3.0,
            );
        });

        // Origin label
        c.text(label, (origin.0, origin.1 - 30.0), color, 16.0, "center");

        // Flowing particles
        if convergence > 0.5 && (t * 10.0) as i32 % 5 == 0 {
            particles.add(
                origin.0,
                origin.1,
                ((center.0 - origin.0) * 0.5, (center.1 - origin.1) * 0.5),
                1.0,
                ParticleColor::Cyan,
                ParticleType::Dot,
            );
        }
    }

    // Central convergence point - supernova effect
    c.faded(convergence, || {
        for i in 0..8 {
            let radius = 30.0 + f64::from(i) * 25.0;
            let alpha = 1.0 / (f64::from(i + 1));
            c.faded(alpha, || {
                c.ring(center, radius, ACCENT, 3.0);
            });
        }
    });

    // "Attention Is All You Need" paper glow
    if t > 1.0 {
        c.text("Attention Is All", (center.0, center.1 - 40.0), CYAN, 24.0, "center");
        c.text("You Need", (center.0, center.1 - 10.0), CYAN, 24.0, "center");
        c.text("2017", (center.0, center.1 + 30.0), ACCENT, 32.0, "center");
    }

    // Final message
    if t > 1.5 {
        c.text("WE ARE HERE", (center.0, 600.0), ACCENT, 48.0, "center");
    }
}
