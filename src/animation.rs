//! 🎬 Advanced animation system with easing, tweening, and particle effects
//! Replaces scattered sin() calls with declarative, composable animations

use std::f64::consts::TAU;

/// Easing function collection for smooth, natural motion
#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

impl Easing {
    /// Apply easing to normalized time [0, 1] → eased value [0, 1]
    pub fn apply(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,

            // Quad (power of 2)
            Easing::EaseIn | Easing::EaseInQuad => t * t,
            Easing::EaseOut | Easing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut | Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }

            // Cubic (power of 3)
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }

            // Quart (power of 4)
            Easing::EaseInQuart => t * t * t * t,
            Easing::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Easing::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }

            // Back (overshoot)
            Easing::EaseInBack => {
                const C: f64 = 1.70158;
                (C + 1.0) * t * t * t - C * t * t
            }
            Easing::EaseOutBack => {
                const C: f64 = 1.70158;
                1.0 + (C + 1.0) * (t - 1.0).powi(3) + C * (t - 1.0).powi(2)
            }
            Easing::EaseInOutBack => {
                const C1: f64 = 1.70158;
                const C2: f64 = C1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
                }
            }

            // Elastic (spring-like)
            Easing::EaseInElastic => {
                if t == 0.0 || t == 1.0 {
                    return t;
                }
                const C: f64 = TAU / 3.0;
                -(2.0_f64.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * C).sin()
            }
            Easing::EaseOutElastic => {
                if t == 0.0 || t == 1.0 {
                    return t;
                }
                const C: f64 = TAU / 3.0;
                2.0_f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C).sin() + 1.0
            }
            Easing::EaseInOutElastic => {
                if t == 0.0 || t == 1.0 {
                    return t;
                }
                const C: f64 = TAU / 4.5;
                if t < 0.5 {
                    -(2.0_f64.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C).sin()) / 2.0
                } else {
                    (2.0_f64.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C).sin()) / 2.0 + 1.0
                }
            }

            // Bounce
            Easing::EaseInBounce => 1.0 - Easing::EaseOutBounce.apply(1.0 - t),
            Easing::EaseOutBounce => {
                const N1: f64 = 7.5625;
                const D1: f64 = 2.75;
                if t < 1.0 / D1 {
                    N1 * t * t
                } else if t < 2.0 / D1 {
                    N1 * (t - 1.5 / D1) * (t - 1.5 / D1) + 0.75
                } else if t < 2.5 / D1 {
                    N1 * (t - 2.25 / D1) * (t - 2.25 / D1) + 0.9375
                } else {
                    N1 * (t - 2.625 / D1) * (t - 2.625 / D1) + 0.984375
                }
            }
            Easing::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - Easing::EaseOutBounce.apply(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + Easing::EaseOutBounce.apply(2.0 * t - 1.0)) / 2.0
                }
            }
        }
    }
}

/// A tween animates a value from start to end over duration using an easing function
#[derive(Debug, Clone)]
pub struct Tween {
    pub start: f64,
    pub end: f64,
    pub duration: f64,
    pub easing: Easing,
}

impl Tween {
    pub fn new(start: f64, end: f64, duration: f64) -> Self {
        Self {
            start,
            end,
            duration,
            easing: Easing::EaseOutCubic,
        }
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Get value at time t (0 = start, duration = end)
    pub fn at(&self, time: f64) -> f64 {
        let progress = (time / self.duration).clamp(0.0, 1.0);
        let eased = self.easing.apply(progress);
        self.start + (self.end - self.start) * eased
    }

    /// Check if animation is complete
    pub fn is_complete(&self, time: f64) -> bool {
        time >= self.duration
    }
}

/// Oscillate smoothly between min and max
pub fn oscillate(time: f64, period: f64, min: f64, max: f64) -> f64 {
    let phase = (time / period * TAU).sin();
    min + (max - min) * (phase + 1.0) / 2.0
}

/// Pulse effect (0 → 1 → 0) over period
pub fn pulse(time: f64, period: f64) -> f64 {
    let t = (time % period) / period;
    (t * TAU).sin().abs()
}

/// Smooth step function (0 below edge0, 1 above edge1, smooth between)
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Extra smooth step (even smoother than smoothstep)
pub fn smootherstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Exponential ease in/out
pub fn exp_ease_in_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0_f64.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0_f64.powf(-20.0 * t + 10.0)) / 2.0
    }
}

/// Spring physics simulation
#[derive(Debug, Clone)]
pub struct Spring {
    pub value: f64,
    pub velocity: f64,
    pub target: f64,
    pub stiffness: f64,
    pub damping: f64,
}

impl Spring {
    pub fn new(initial: f64, stiffness: f64, damping: f64) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            stiffness,
            damping,
        }
    }

    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }

    pub fn update(&mut self, delta: f64) {
        let force = (self.target - self.value) * self.stiffness;
        let damping_force = self.velocity * self.damping;
        let acceleration = force - damping_force;

        self.velocity += acceleration * delta;
        self.value += self.velocity * delta;
    }

    pub fn is_resting(&self, threshold: f64) -> bool {
        (self.value - self.target).abs() < threshold && self.velocity.abs() < threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_at_edges() {
        for easing in [
            Easing::Linear,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::EaseOutCubic,
            Easing::EaseOutElastic,
        ] {
            assert_eq!(easing.apply(0.0), 0.0);
            assert_eq!(easing.apply(1.0), 1.0);
        }
    }

    #[test]
    fn tween_interpolates() {
        let tween = Tween::new(0.0, 100.0, 2.0);
        assert_eq!(tween.at(0.0), 0.0);
        assert!(tween.at(1.0) > 0.0 && tween.at(1.0) < 100.0);
        assert_eq!(tween.at(2.0), 100.0);
    }

    #[test]
    fn spring_converges() {
        let mut spring = Spring::new(0.0, 0.5, 0.8);
        spring.set_target(100.0);

        for _ in 0..100 {
            spring.update(0.016); // ~60fps
        }

        assert!((spring.value - 100.0).abs() < 1.0);
    }
}
