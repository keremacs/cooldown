//! Analytics: deep work score, anomaly detection, proactive suggestions.

use chrono::{Local, Timelike};
use std::collections::HashMap;

use crate::models::{
    AnomalyReport, AppCategory, BaselineMetrics, ProactiveSuggestion, TrendBucket,
};

/// Tracks uninterrupted IDE focus blocks.
pub struct DeepWorkTracker {
    current_block_secs: u64,
    longest_block_secs: u64,
    last_category: AppCategory,
    block_started: bool,
}

impl Default for DeepWorkTracker {
    fn default() -> Self {
        Self {
            current_block_secs: 0,
            longest_block_secs: 0,
            last_category: AppCategory::Other,
            block_started: false,
        }
    }
}

impl DeepWorkTracker {
    pub fn tick(&mut self, category: AppCategory, elapsed_secs: u64) {
        if category == AppCategory::Ide {
            self.current_block_secs += elapsed_secs;
            self.block_started = true;
            if self.current_block_secs > self.longest_block_secs {
                self.longest_block_secs = self.current_block_secs;
            }
        } else if self.block_started {
            self.current_block_secs = 0;
            self.block_started = false;
        }
        self.last_category = category;
    }

    /// Score 0–100 based on longest uninterrupted IDE block today.
    pub fn score(&self) -> f64 {
        // 25 min uninterrupted = 50, 50 min = 80, 90+ min = 100
        let mins = self.longest_block_secs as f64 / 60.0;
        if mins >= 90.0 {
            100.0
        } else if mins >= 50.0 {
            80.0 + (mins - 50.0) * 0.5
        } else if mins >= 25.0 {
            50.0 + (mins - 25.0) * 1.2
        } else {
            (mins / 25.0) * 50.0
        }
        .clamp(0.0, 100.0)
    }
}

pub fn detect_anomalies(
    fatigue: f64,
    switches: f64,
    errors: f64,
    cpm: f64,
    deep_work: f64,
    baseline: Option<&BaselineMetrics>,
) -> Vec<AnomalyReport> {
    let Some(b) = baseline else {
        return vec![];
    };

    let mut out = vec![];
    let checks = [
        ("fatigue", fatigue, b.avg_fatigue),
        ("context_switches", switches, b.avg_switches),
        ("errors", errors, b.avg_errors),
        ("typing_cpm", cpm, b.avg_keystrokes_per_min),
        ("deep_work", deep_work, b.avg_deep_work),
    ];

    for (metric, current, base) in checks {
        if base <= 0.0 {
            continue;
        }
        let deviation = ((current - base) / base) * 100.0;
        if deviation.abs() >= 40.0 {
            let message = if deviation > 0.0 {
                format!(
                    "Today's {metric} is {:.0}% above your personal baseline ({current:.1} vs {base:.1}).",
                    deviation
                )
            } else {
                format!(
                    "Today's {metric} is {:.0}% below your personal baseline — unusually light day.",
                    deviation.abs()
                )
            };
            out.push(AnomalyReport {
                metric: metric.to_string(),
                current,
                baseline: base,
                deviation_pct: deviation,
                message,
            });
        }
    }
    out
}

/// Suggest break times based on historical hourly fatigue peaks (last 14 days of buckets).
pub fn proactive_suggestions(hourly_fatigue: &HashMap<u32, Vec<f64>>) -> Vec<ProactiveSuggestion> {
    let mut avgs: Vec<(u32, f64)> = hourly_fatigue
        .iter()
        .map(|(h, vals)| {
            let avg = vals.iter().sum::<f64>() / vals.len() as f64;
            (*h, avg)
        })
        .collect();
    avgs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    avgs.into_iter()
        .take(3)
        .filter(|(_, avg)| *avg >= 55.0)
        .map(|(hour, avg)| {
            ProactiveSuggestion {
                hour: format!("{:02}:00", hour),
                message: format!(
                    "Historical data shows elevated load around {:02}:00 (avg fatigue {:.0}). Schedule a break before then.",
                    hour, avg
                ),
            }
        })
        .collect()
}

pub fn peak_hours_from_trends(trends: &[TrendBucket]) -> HashMap<u32, Vec<f64>> {
    let mut map: HashMap<u32, Vec<f64>> = HashMap::new();
    for bucket in trends {
        // Parse hour from label if possible
        if let Ok(h) = bucket.label.split(':').next().unwrap_or("12").parse::<u32>() {
            map.entry(h).or_default().push(bucket.avg_fatigue);
        }
    }
    // Also seed from current hour for cold start
    let now_h = Local::now().hour();
    map.entry(now_h).or_default();
    map
}
