//! Fatigue score algorithm: combines context switches, typing cadence, and error events
//! into a 0–100 real-time score. Higher = more cognitive load / burnout risk.

use crate::models::CognitiveZone;

/// Raw metrics collected from trackers over rolling windows.
#[derive(Debug, Clone, Default)]
pub struct FatigueInputs {
    /// Context switches in the last 30 minutes.
    pub switches_last_30min: u32,
    /// Developer error events in the last hour (VS Code diagnostics, terminal failures).
    pub errors_last_hour: u32,
    /// Keystrokes per minute averaged over the last 5 minutes.
    pub keystrokes_per_min: f64,
}

/// Compute fatigue score (0–100) from current metrics.
///
/// Weighting rationale:
/// - Context switching (max 35): rapid app hopping fragments focus.
/// - Error events (max 35): repeated build/test failures increase stress.
/// - Typing intensity (max 30): sustained high cadence without pauses signals overload.
pub fn compute_fatigue_score(inputs: &FatigueInputs) -> f64 {
    let switch_rate = inputs.switches_last_30min as f64 / 30.0;
    let switch_component = (switch_rate * 4.0).min(35.0);

    let error_component = (inputs.errors_last_hour as f64 * 7.0).min(35.0);

    let cpm = inputs.keystrokes_per_min;
    let typing_component = if cpm > 140.0 {
        30.0
    } else if cpm > 90.0 {
        15.0 + (cpm - 90.0) / 3.3
    } else if cpm > 50.0 {
        (cpm - 50.0) / 2.7
    } else {
        0.0
    };

    (switch_component + error_component + typing_component)
        .clamp(0.0, 100.0)
}

pub fn zone_from_score(score: f64) -> CognitiveZone {
    CognitiveZone::from_fatigue(score)
}

/// Generate a one-sentence actionable insight from recent activity patterns.
pub fn generate_insight(
    score: f64,
    zone: CognitiveZone,
    switches: u32,
    errors: u32,
    peak_hour: Option<&str>,
) -> String {
    match zone {
        CognitiveZone::Flow => {
            if score < 20.0 {
                "You're in a deep focus zone — protect this block from interruptions.".to_string()
            } else {
                "Steady flow detected. Consider a short break before the next complex task.".to_string()
            }
        }
        CognitiveZone::Distraction => {
            if switches > 15 {
                format!(
                    "High context-switching ({} switches/30min) is fragmenting focus — batch similar tasks.",
                    switches
                )
            } else if errors > 3 {
                format!(
                    "Repeated errors ({} this hour) are adding cognitive load — step back and debug systematically.",
                    errors
                )
            } else if let Some(hour) = peak_hour {
                format!(
                    "Elevated cognitive load around {} — consider a 5-minute reset.",
                    hour
                )
            } else {
                "Moderate distraction detected — close non-essential apps for the next 25 minutes.".to_string()
            }
        }
        CognitiveZone::Burnout => {
            if errors > 5 {
                if let Some(hour) = peak_hour {
                    format!(
                        "High cognitive load detected around {} due to repeated terminal/build errors.",
                        hour
                    )
                } else {
                    format!(
                        "Burnout risk: {} errors this hour. Stop debugging and take a 10-minute break.",
                        errors
                    )
                }
            } else if switches > 25 {
                "Burnout zone: excessive context switching. Close Slack/browser and focus on one task.".to_string()
            } else {
                "Fatigue score critical — sustained typing without rest. Step away from the screen now.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_activity_yields_low_score() {
        let inputs = FatigueInputs {
            switches_last_30min: 2,
            errors_last_hour: 0,
            keystrokes_per_min: 20.0,
        };
        assert!(compute_fatigue_score(&inputs) < 20.0);
    }

    #[test]
    fn high_stress_yields_high_score() {
        let inputs = FatigueInputs {
            switches_last_30min: 30,
            errors_last_hour: 8,
            keystrokes_per_min: 150.0,
        };
        assert!(compute_fatigue_score(&inputs) > 75.0);
    }
}
