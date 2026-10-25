//! 🌟 Particle system for visual effects: stars, sparks, trails, explosions
//! Efficient pooled particle management with multiple emitter types

use std::f64::consts::TAU;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::animation::Easing;

const MAX_PARTICLES: usize = 2000;

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: f64,
    pub max_life: f64,
    pub size: f64,
    pub color: ParticleColor,
    pub particle_type: ParticleType,
}

#[derive(Debug, Clone, Copy)]
pub enum ParticleColor {
    Cyan,
    Gold,
    Emerald,
    Purple,
    Orange,
    White,
    Custom(u8, u8, u8), // RGB
}

impl ParticleColor {
    pub fn to_rgba(&self, alpha: f64) -> String {
        let (r, g, b) = match self {
            ParticleColor::Cyan => (56, 189, 248),
            ParticleColor::Gold => (255, 181, 71),
            ParticleColor::Emerald => (110, 231, 183),
            ParticleColor::Purple => (186, 155, 255),
            ParticleColor::Orange => (255, 138, 91),
            ParticleColor::White => (240, 241, 243),
            ParticleColor::Custom(r, g, b) => (*r, *g, *b),
        };
        format!("rgba({r}, {g}, {b}, {alpha:.3})")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleType {
    Dot,           // Simple circular particle
    Star,          // 4-pointed star
    Spark,         // Elongated streak
    Trail,         // Motion trail
    Glow,          // Soft glow blob
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    gravity: f64,
    wind: f64,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
            gravity: 0.0,
            wind: 0.0,
        }
    }

    pub fn with_gravity(mut self, gravity: f64) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn with_wind(mut self, wind: f64) -> Self {
        self.wind = wind;
        self
    }

    /// Add a single particle with custom parameters
    pub fn add(
        &mut self,
        x: f64,
        y: f64,
        velocity: (f64, f64),
        life: f64,
        color: ParticleColor,
        particle_type: ParticleType,
    ) {
        if self.particles.len() >= MAX_PARTICLES {
            return;
        }

        self.particles.push(Particle {
            x,
            y,
            vx: velocity.0,
            vy: velocity.1,
            life,
            max_life: life,
            size: 2.0 + noise(self.particles.len()) * 2.0,
            color,
            particle_type,
        });
    }

    /// Emit a burst of particles from a point
    pub fn emit_burst(
        &mut self,
        x: f64,
        y: f64,
        count: usize,
        speed: f64,
        color: ParticleColor,
        particle_type: ParticleType,
    ) {
        if self.particles.len() + count > MAX_PARTICLES {
            return;
        }

        for i in 0..count {
            let angle = (i as f64 / count as f64) * TAU;
            let speed_var = speed * (0.7 + noise(i) * 0.6);

            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed_var,
                vy: angle.sin() * speed_var,
                life: 1.0,
                max_life: 1.0,
                size: 2.0 + noise(i + 100) * 3.0,
                color,
                particle_type,
            });
        }
    }

    /// Emit particles continuously from a source
    #[allow(clippy::too_many_arguments)]
    pub fn emit_stream(
        &mut self,
        x: f64,
        y: f64,
        count: usize,
        angle: f64,
        spread: f64,
        speed: f64,
        color: ParticleColor,
        particle_type: ParticleType,
    ) {
        if self.particles.len() + count > MAX_PARTICLES {
            return;
        }

        for i in 0..count {
            let offset_angle = angle + (noise(i) - 0.5) * spread;
            let speed_var = speed * (0.8 + noise(i + 50) * 0.4);

            self.particles.push(Particle {
                x: x + (noise(i + 200) - 0.5) * 10.0,
                y: y + (noise(i + 300) - 0.5) * 10.0,
                vx: offset_angle.cos() * speed_var,
                vy: offset_angle.sin() * speed_var,
                life: 0.8 + noise(i + 150) * 0.4,
                max_life: 1.0,
                size: 1.5 + noise(i + 250) * 2.0,
                color,
                particle_type,
            });
        }
    }

    /// Create floating ambient particles (like stars or dust)
    pub fn emit_ambient(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        count: usize,
        color: ParticleColor,
    ) {
        if self.particles.len() + count > MAX_PARTICLES {
            return;
        }

        for i in 0..count {
            self.particles.push(Particle {
                x: x + noise(i) * width,
                y: y + noise(i + 100) * height,
                vx: (noise(i + 200) - 0.5) * 20.0,
                vy: (noise(i + 300) - 0.5) * 20.0,
                life: 1.0,
                max_life: 1.0,
                size: 1.0 + noise(i + 400) * 2.5,
                color,
                particle_type: ParticleType::Star,
            });
        }
    }

    /// Emit trail particles (following an object)
    pub fn emit_trail(
        &mut self,
        x: f64,
        y: f64,
        vx: f64,
        vy: f64,
        count: usize,
        color: ParticleColor,
    ) {
        if self.particles.len() + count > MAX_PARTICLES {
            return;
        }

        for i in 0..count {
            let offset = (i as f64 / count as f64) * 0.3;

            self.particles.push(Particle {
                x: x - vx * offset + (noise(i) - 0.5) * 5.0,
                y: y - vy * offset + (noise(i + 100) - 0.5) * 5.0,
                vx: vx * 0.3,
                vy: vy * 0.3,
                life: 0.6,
                max_life: 0.6,
                size: 3.0 + noise(i + 200) * 2.0,
                color,
                particle_type: ParticleType::Trail,
            });
        }
    }

    /// Update all particles (call every frame)
    pub fn update(&mut self, delta: f64) {
        for particle in &mut self.particles {
            // Apply forces
            particle.vy += self.gravity * delta;
            particle.vx += self.wind * delta;

            // Apply drag
            let drag = 0.98;
            particle.vx *= drag;
            particle.vy *= drag;

            // Update position
            particle.x += particle.vx * delta * 60.0;
            particle.y += particle.vy * delta * 60.0;

            // Decay life
            particle.life -= delta * 0.5;
        }

        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0);
    }

    /// Draw all particles
    pub fn draw(&self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        ctx.save();

        for particle in &self.particles {
            let alpha = Easing::EaseOut.apply(particle.life / particle.max_life);
            let color = particle.color.to_rgba(alpha);

            match particle.particle_type {
                ParticleType::Dot => {
                    ctx.set_fill_style_str(&color);
                    ctx.begin_path();
                    ctx.arc(particle.x, particle.y, particle.size, 0.0, TAU)?;
                    ctx.fill();
                }

                ParticleType::Star => {
                    ctx.set_fill_style_str(&color);
                    ctx.begin_path();
                    for i in 0..8 {
                        let angle = (i as f64) * TAU / 8.0;
                        let r = if i % 2 == 0 {
                            particle.size
                        } else {
                            particle.size * 0.3
                        };
                        let x = particle.x + angle.cos() * r;
                        let y = particle.y + angle.sin() * r;
                        if i == 0 {
                            ctx.move_to(x, y);
                        } else {
                            ctx.line_to(x, y);
                        }
                    }
                    ctx.close_path();
                    ctx.fill();
                }

                ParticleType::Spark => {
                    let angle = particle.vy.atan2(particle.vx);
                    let length = particle.size * 2.0;
                    ctx.set_stroke_style_str(&color);
                    ctx.set_line_width(particle.size * 0.5);
                    ctx.set_line_cap("round");
                    ctx.begin_path();
                    ctx.move_to(particle.x, particle.y);
                    ctx.line_to(
                        particle.x - angle.cos() * length,
                        particle.y - angle.sin() * length,
                    );
                    ctx.stroke();
                }

                ParticleType::Trail => {
                    ctx.set_global_alpha(alpha * 0.6);
                    ctx.set_fill_style_str(&particle.color.to_rgba(1.0));
                    ctx.begin_path();
                    ctx.arc(particle.x, particle.y, particle.size, 0.0, TAU)?;
                    ctx.fill();
                    ctx.set_global_alpha(1.0);
                }

                ParticleType::Glow => {
                    // Create radial gradient for soft glow
                    if let Ok(gradient) =
                        ctx.create_radial_gradient(particle.x, particle.y, 0.0, particle.x, particle.y, particle.size * 2.0)
                    {
                        let _ = gradient.add_color_stop(0.0, &color);
                        let _ = gradient.add_color_stop(1.0, &particle.color.to_rgba(0.0));
                        ctx.set_fill_style_canvas_gradient(&gradient);
                        ctx.begin_path();
                        ctx.arc(particle.x, particle.y, particle.size * 2.0, 0.0, TAU)?;
                        ctx.fill();
                    }
                }
            }
        }

        ctx.restore();
        Ok(())
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple deterministic noise based on seed (same as scene.rs)
fn noise(seed: usize) -> f64 {
    let mut value = (seed as u32)
        .wrapping_mul(747_796_405)
        .wrapping_add(2_891_336_453);
    value = ((value >> ((value >> 28) + 4)) ^ value).wrapping_mul(277_803_737);
    f64::from((value >> 22) ^ value) / f64::from(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_system_respects_max_count() {
        let mut system = ParticleSystem::new();
        system.emit_burst(0.0, 0.0, 3000, 100.0, ParticleColor::Cyan, ParticleType::Dot);
        assert!(system.count() <= MAX_PARTICLES);
    }

    #[test]
    fn particles_decay_over_time() {
        let mut system = ParticleSystem::new();
        system.emit_burst(0.0, 0.0, 100, 50.0, ParticleColor::Gold, ParticleType::Dot);
        let initial = system.count();

        for _ in 0..100 {
            system.update(0.1);
        }

        assert!(system.count() < initial);
    }
}
