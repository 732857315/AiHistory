//! Sample-free playback of the original score in `assets/music-score.json`.
//! Oscillators are scheduled a fraction of a second ahead and released promptly.

use wasm_bindgen::JsValue;
use web_sys::{AudioContext, AudioContextState, GainNode, OscillatorNode, OscillatorType};

#[derive(Clone, Copy)]
struct Bar {
    chord: usize,
    melody: [u8; 4],
}

// MIDI pitches, in the same order as the editable original score. Zero is rest.
const SCORE: [Bar; 48] = [
    Bar {
        chord: 0,
        melody: [76, 0, 79, 74],
    },
    Bar {
        chord: 1,
        melody: [69, 76, 0, 72],
    },
    Bar {
        chord: 2,
        melody: [79, 76, 72, 0],
    },
    Bar {
        chord: 3,
        melody: [74, 0, 67, 69],
    },
    Bar {
        chord: 0,
        melody: [72, 79, 76, 0],
    },
    Bar {
        chord: 1,
        melody: [81, 79, 0, 76],
    },
    Bar {
        chord: 2,
        melody: [72, 74, 69, 76],
    },
    Bar {
        chord: 3,
        melody: [79, 0, 74, 67],
    },
    Bar {
        chord: 1,
        melody: [76, 81, 79, 72],
    },
    Bar {
        chord: 2,
        melody: [69, 74, 76, 0],
    },
    Bar {
        chord: 0,
        melody: [79, 72, 0, 76],
    },
    Bar {
        chord: 3,
        melody: [74, 69, 79, 0],
    },
    Bar {
        chord: 0,
        melody: [67, 76, 74, 79],
    },
    Bar {
        chord: 2,
        melody: [81, 0, 72, 76],
    },
    Bar {
        chord: 1,
        melody: [69, 79, 76, 72],
    },
    Bar {
        chord: 3,
        melody: [74, 67, 0, 74],
    },
    Bar {
        chord: 2,
        melody: [72, 69, 0, 74],
    },
    Bar {
        chord: 0,
        melody: [76, 79, 72, 0],
    },
    Bar {
        chord: 4,
        melody: [81, 76, 74, 69],
    },
    Bar {
        chord: 3,
        melody: [79, 0, 74, 76],
    },
    Bar {
        chord: 1,
        melody: [69, 76, 79, 81],
    },
    Bar {
        chord: 2,
        melody: [76, 72, 74, 0],
    },
    Bar {
        chord: 0,
        melody: [79, 76, 84, 79],
    },
    Bar {
        chord: 3,
        melody: [74, 79, 81, 74],
    },
    Bar {
        chord: 0,
        melody: [76, 84, 79, 74],
    },
    Bar {
        chord: 1,
        melody: [81, 76, 72, 79],
    },
    Bar {
        chord: 2,
        melody: [84, 81, 0, 76],
    },
    Bar {
        chord: 3,
        melody: [79, 74, 69, 79],
    },
    Bar {
        chord: 2,
        melody: [81, 72, 76, 79],
    },
    Bar {
        chord: 3,
        melody: [74, 79, 86, 81],
    },
    Bar {
        chord: 1,
        melody: [84, 76, 79, 0],
    },
    Bar {
        chord: 0,
        melody: [76, 74, 72, 79],
    },
    Bar {
        chord: 1,
        melody: [76, 79, 69, 84],
    },
    Bar {
        chord: 2,
        melody: [81, 76, 72, 74],
    },
    Bar {
        chord: 0,
        melody: [79, 0, 76, 84],
    },
    Bar {
        chord: 3,
        melody: [86, 81, 79, 74],
    },
    Bar {
        chord: 4,
        melody: [74, 81, 76, 0],
    },
    Bar {
        chord: 2,
        melody: [84, 79, 81, 76],
    },
    Bar {
        chord: 0,
        melody: [79, 74, 72, 76],
    },
    Bar {
        chord: 3,
        melody: [69, 79, 0, 74],
    },
    Bar {
        chord: 0,
        melody: [84, 76, 79, 74],
    },
    Bar {
        chord: 1,
        melody: [81, 72, 76, 79],
    },
    Bar {
        chord: 2,
        melody: [69, 74, 84, 81],
    },
    Bar {
        chord: 3,
        melody: [79, 86, 76, 74],
    },
    Bar {
        chord: 2,
        melody: [81, 76, 72, 0],
    },
    Bar {
        chord: 3,
        melody: [74, 79, 0, 69],
    },
    Bar {
        chord: 0,
        melody: [76, 74, 67, 0],
    },
    Bar {
        chord: 0,
        melody: [72, 0, 0, 0],
    },
];

const CHORDS: [[u8; 3]; 5] = [
    [60, 64, 67],
    [57, 60, 64],
    [53, 57, 60],
    [55, 59, 62],
    [62, 65, 69],
];
const BASS: [u8; 5] = [36, 45, 41, 43, 38];
const LEAD_OFFSETS: [f64; 4] = [0.125, 0.5, 1.125, 1.5];
const DURATION: f64 = 96.0;
const BAR_SECONDS: f64 = 2.0;
const LOOKAHEAD: f64 = 0.12;

struct Voice {
    oscillator: OscillatorNode,
    gain: GainNode,
    end: f64,
}

/// Browser-generated music; no encoded audio, samples, or network requests.
pub struct Music {
    context: AudioContext,
    master: GainNode,
    voices: Vec<Voice>,
    scheduled_through: f64,
    last_time: Option<f64>,
    restart_bed: bool,
}

impl Music {
    pub fn new() -> Result<Self, JsValue> {
        let context = AudioContext::new()?;
        let master = context.create_gain()?;
        master.gain().set_value(0.7);
        master.connect_with_audio_node(&context.destination())?;
        Ok(Self {
            context,
            master,
            voices: Vec::new(),
            scheduled_through: -0.001,
            last_time: None,
            restart_bed: false,
        })
    }

    /// Await this Promise after a user gesture when the browser blocks autoplay.
    pub fn resume(&self) -> Result<js_sys::Promise, JsValue> {
        self.context.resume()
    }

    pub fn pause(&self) {
        let _ = self.context.suspend();
    }

    pub fn is_running(&self) -> bool {
        self.context.state() == AudioContextState::Running
    }

    pub fn set_volume(&self, volume: f64) {
        let value = if volume.is_finite() {
            volume.clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        let _ = self
            .master
            .gain()
            .set_target_at_time(value, self.context.current_time(), 0.03);
    }

    /// Stop old notes before resynchronizing with a new animation position.
    pub fn seek(&mut self, time: f64) {
        self.stop_voices();
        let time = valid_time(time);
        self.scheduled_through = time - 0.001;
        self.last_time = Some(time);
        self.restart_bed = time % BAR_SECONDS > 0.001 && time < DURATION;
    }

    /// Call while the transport is playing, with its current 0–96 second time.
    pub fn tick(&mut self, time: f64) {
        let time = valid_time(time);
        if time >= DURATION {
            self.stop_voices();
            self.last_time = Some(time);
            return;
        }
        if self
            .last_time
            .is_some_and(|last| time < last - 0.05 || time - last > 0.5)
        {
            self.seek(time);
        }
        self.last_time = Some(time);
        let now = self.context.current_time();
        self.voices.retain(|voice| {
            if voice.end <= now {
                let _ = voice.oscillator.disconnect();
                let _ = voice.gain.disconnect();
                false
            } else {
                true
            }
        });
        if self.restart_bed {
            let bar_index = (time / BAR_SECONDS) as usize;
            let remaining = ((bar_index + 1) as f64 * BAR_SECONDS - time).max(0.05);
            self.pad(SCORE[bar_index].chord, now + 0.005, remaining, fade(time));
            self.restart_bed = false;
        }
        let lower = self.scheduled_through.max(time - 0.04);
        let upper = (time + LOOKAHEAD).min(DURATION);
        let first = (lower.max(0.0) / BAR_SECONDS) as usize;
        let last = ((upper / BAR_SECONDS) as usize).min(SCORE.len() - 1);
        for (index, bar) in SCORE.iter().copied().enumerate().take(last + 1).skip(first) {
            let start = index as f64 * BAR_SECONDS;
            if start > lower && start <= upper {
                let at = now + (start - time).max(0.0) + 0.005;
                self.pad(bar.chord, at, 1.98, fade(start));
            }
            for offset in [0.0, 1.0] {
                let position = start + offset;
                if position > lower && position <= upper {
                    let at = now + (position - time).max(0.0) + 0.005;
                    self.tone(BASS[bar.chord], at, 0.72, 0.055 * fade(position), 0.035);
                }
            }
            for (note, offset) in bar.melody.into_iter().zip(LEAD_OFFSETS) {
                let position = start + offset;
                if note != 0 && position > lower && position <= upper {
                    let at = now + (position - time).max(0.0) + 0.005;
                    let level = fade(position);
                    self.tone(note, at, 0.49, 0.075 * level, 0.012);
                    self.tone(note + 12, at, 0.28, 0.012 * level, 0.012);
                }
            }
        }
        self.scheduled_through = upper;
    }

    fn pad(&mut self, chord: usize, start: f64, duration: f64, level: f32) {
        for note in CHORDS[chord] {
            self.tone(note, start, duration, 0.026 * level, 0.20);
        }
    }

    fn tone(&mut self, note: u8, start: f64, duration: f64, level: f32, attack: f64) {
        let result = (|| -> Result<Voice, JsValue> {
            let oscillator = self.context.create_oscillator()?;
            oscillator.set_type(OscillatorType::Sine);
            oscillator
                .frequency()
                .set_value((440.0 * 2.0_f64.powf((f64::from(note) - 69.0) / 12.0)) as f32);
            let gain = self.context.create_gain()?;
            let parameter = gain.gain();
            let end = start + duration;
            let attack = attack.min(duration / 3.0);
            parameter.set_value_at_time(0.0, start)?;
            parameter.linear_ramp_to_value_at_time(level.max(0.0001), start + attack)?;
            parameter.exponential_ramp_to_value_at_time(0.0001, end - 0.01)?;
            parameter.linear_ramp_to_value_at_time(0.0, end)?;
            oscillator.connect_with_audio_node(&gain)?;
            gain.connect_with_audio_node(&self.master)?;
            oscillator.start_with_when(start)?;
            oscillator.stop_with_when(end)?;
            Ok(Voice {
                oscillator,
                gain,
                end,
            })
        })();
        if let Ok(voice) = result {
            self.voices.push(voice);
        }
    }

    fn stop_voices(&mut self) {
        for voice in self.voices.drain(..) {
            let _ = voice.oscillator.stop();
            let _ = voice.oscillator.disconnect();
            let _ = voice.gain.disconnect();
        }
    }
}

impl Drop for Music {
    fn drop(&mut self) {
        self.stop_voices();
        let _ = self.context.close();
    }
}

fn valid_time(time: f64) -> f64 {
    if time.is_finite() {
        time.clamp(0.0, DURATION)
    } else {
        0.0
    }
}

fn fade(time: f64) -> f32 {
    let position = (time / 1.25).min((DURATION - time) / 3.0).clamp(0.0, 1.0);
    (position * std::f64::consts::FRAC_PI_2).sin().powi(2) as f32
}
