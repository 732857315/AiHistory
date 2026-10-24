pub mod history;
pub mod models;

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod audio;
#[cfg(target_arch = "wasm32")]
mod scene;

pub const LANGUAGES: [&str; 4] = ["zh", "en", "ja", "es"];
pub const RACE_SECONDS: f64 = 180.0;
pub const RACE_START: f64 = 2017.45;
pub const SNAPSHOT_DATE: &str = "2026-10-24";
pub const SNAPSHOT_SECONDS: f64 = 24.0;

pub fn language_index(language: &str) -> usize {
    let prefix = language.split(['-', '_']).next().unwrap_or("zh");
    LANGUAGES
        .iter()
        .position(|code| code.eq_ignore_ascii_case(prefix))
        .unwrap_or(0)
}

/// A monotonic date coordinate. Months have equal visual width on the timeline.
pub fn date_value(date: &str) -> f64 {
    let parts: Vec<f64> = date.split('-').filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 3 {
        return 0.0;
    }
    parts[0] + (parts[1] - 1.0) / 12.0 + (parts[2] - 1.0) / 372.0
}

pub fn history_duration() -> f64 {
    history::HISTORY.iter().map(|h| h.duration).sum()
}
pub fn total_duration() -> f64 {
    history_duration() + RACE_SECONDS + SNAPSHOT_SECONDS
}
pub fn race_end() -> f64 {
    date_value(SNAPSHOT_DATE)
}
pub fn race_date(time: f64) -> f64 {
    let progress = ((time - history_duration()) / RACE_SECONDS).clamp(0.0, 1.0);
    RACE_START + progress * (race_end() - RACE_START)
}
pub fn model_time(date: &str) -> f64 {
    history_duration()
        + ((date_value(date) - RACE_START) / (race_end() - RACE_START)).clamp(0.0, 1.0)
            * RACE_SECONDS
}
pub fn history_start(index: usize) -> f64 {
    history::HISTORY
        .iter()
        .take(index)
        .map(|h| h.duration)
        .sum()
}
pub fn history_at(time: f64) -> Option<usize> {
    let time = if time.is_finite() { time.max(0.0) } else { 0.0 };
    let mut elapsed = 0.0;
    for (i, event) in history::HISTORY.iter().enumerate() {
        elapsed += event.duration;
        if time < elapsed {
            return Some(i);
        }
    }
    None
}

/// A model cannot acquire a score before that result was published.
pub fn ranking(time: f64, benchmark: &str, protocol: &str) -> Vec<usize> {
    if time < history_duration() {
        return Vec::new();
    }
    let date = race_date(time);
    let mut ranked: Vec<usize> = models::MODELS
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            m.benchmark == benchmark
                && m.protocol == protocol
                && m.score.is_some_and(f64::is_finite)
                && date_value(m.released) <= date
                && date_value(m.reported) <= date
        })
        .map(|(i, _)| i)
        .collect();
    ranked.sort_by(|a, b| {
        models::MODELS[*b]
            .score
            .unwrap_or(0.0)
            .total_cmp(&models::MODELS[*a].score.unwrap_or(0.0))
            .then_with(|| models::MODELS[*a].released.cmp(models::MODELS[*b].released))
    });
    ranked
}

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn localized_language_variants_have_safe_defaults() {
        assert_eq!(language_index("en-GB"), 1);
        assert_eq!(language_index("JA_jp"), 2);
        assert_eq!(language_index("es-MX"), 3);
        assert_eq!(language_index("fr"), 0);
    }
    #[test]
    fn timing_honors_variable_length_scenes() {
        assert_eq!(history_at(0.0), Some(0));
        assert_eq!(history_at(history::HISTORY[0].duration), Some(1));
        assert_eq!(history_at(history_duration()), None);
        assert_eq!(history_at(f64::NAN), Some(0));
        assert!((race_date(total_duration()) - race_end()).abs() < 1e-8);
    }
    #[test]
    fn scores_do_not_appear_before_publication() {
        assert!(ranking(0.0, "MMLU", "5-shot / no-CoT").is_empty());
        for (index, model) in models::MODELS
            .iter()
            .enumerate()
            .filter(|(_, m)| m.score.is_some())
        {
            let published = model_time(model.reported);
            assert!(!ranking(published - 0.01, model.benchmark, model.protocol).contains(&index));
            assert!(
                ranking(
                    (published + 0.01).min(total_duration()),
                    model.benchmark,
                    model.protocol
                )
                .contains(&index)
            );
        }
    }
    #[test]
    fn final_rankings_are_sorted_and_never_mix_benchmarks() {
        for (metric, protocol) in [
            ("MMLU", "5-shot / no-CoT"),
            ("MMLU", "0-shot CoT / simple-evals"),
            ("MMLU-Pro", "5-shot CoT / MMLU-Pro"),
            ("AdvancedIF", "Mercor / single-shot / highest effort"),
        ] {
            let ranked = ranking(total_duration(), metric, protocol);
            for pair in ranked.windows(2) {
                assert!(models::MODELS[pair[0]].score >= models::MODELS[pair[1]].score);
            }
            for index in ranked {
                assert_eq!(models::MODELS[index].benchmark, metric);
                assert_eq!(models::MODELS[index].protocol, protocol);
            }
        }
    }
    #[test]
    fn content_has_sources_languages_and_valid_dates() {
        for event in history::HISTORY {
            assert!(event.duration > 0.0 && event.duration <= 30.0);
            assert!(event.source_url.starts_with("https://"));
            for l in 0..4 {
                assert!(!event.title[l].is_empty() && !event.simple[l].is_empty());
                assert!(!event.capability[l].is_empty() && !event.limitation[l].is_empty());
            }
        }
        for model in models::MODELS {
            assert!(model.source_url.starts_with("https://"));
            assert!(date_value(model.released) >= RACE_START);
            if let Some(score) = model.score {
                assert!((0.0..=100.0).contains(&score));
                assert!(date_value(model.reported) >= date_value(model.released));
                assert!(model.score_source.starts_with("https://"));
            }
        }
    }
    #[test]
    fn markup_escapes_user_controlled_text() {
        assert_eq!(
            escape_html("<a \"x\">&'"),
            "&lt;a &quot;x&quot;&gt;&amp;&#39;"
        );
    }
}
