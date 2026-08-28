//! Graduated alerts and contextual break suggestions.

use crate::models::{AlertLevel, AlertToast, BreakNotification, BreakSuggestion, DailySummary, PomodoroPhase};

pub fn break_suggestion_for(level: AlertLevel, fatigue: f64, errors: u32) -> BreakSuggestion {
    match level {
        AlertLevel::Hint => BreakSuggestion {
            title: "Light reset".into(),
            detail: "Stand up, stretch your shoulders, and look at something 20 feet away for 20 seconds.".into(),
            duration_min: 2,
        },
        AlertLevel::Warning => {
            if errors > 3 {
                BreakSuggestion {
                    title: "Debug break".into(),
                    detail: "Step away from errors for 5 minutes. Return with a fresh approach or rubber-duck the problem.".into(),
                    duration_min: 5,
                }
            } else {
                BreakSuggestion {
                    title: "Focus recovery".into(),
                    detail: "Close Slack and browser tabs. Take a 5-minute walk or get water.".into(),
                    duration_min: 5,
                }
            }
        }
        AlertLevel::Critical => BreakSuggestion {
            title: "Mandatory break".into(),
            detail: if fatigue >= 95.0 {
                "Stop all work for 10 minutes. Your fatigue is critical — rest eyes, hydrate, breathe.".into()
            } else {
                "Take a 10-minute break away from the screen. Do not dismiss without resting.".into()
            },
            duration_min: 10,
        },
    }
}

pub fn build_notification(
    fatigue: f64,
    insight: String,
    errors: u32,
) -> Option<BreakNotification> {
    let level = AlertLevel::from_fatigue(fatigue)?;
    if level == AlertLevel::Hint {
        return None; // hints use toast, not full popup
    }
    let break_suggestion = break_suggestion_for(level, fatigue, errors);
    Some(BreakNotification {
        fatigue_score: fatigue,
        insight,
        level,
        break_suggestion,
    })
}

pub fn build_hint_toast(fatigue: f64) -> Option<AlertToast> {
    if fatigue >= 60.0 && fatigue < 75.0 {
        Some(AlertToast {
            level: AlertLevel::Hint,
            message: "Cognitive load rising — consider a 2-minute micro-break before pushing further.".into(),
        })
    } else {
        None
    }
}

pub fn build_pomodoro_toast(
    message: String,
    phase: PomodoroPhase,
    cycle: u32,
) -> Option<crate::models::PomodoroNotification> {
    Some(crate::models::PomodoroNotification {
        message,
        phase,
        cycle,
    })
}

pub fn build_daily_summary_toast(summary: &DailySummary) -> AlertToast {
    AlertToast {
        level: AlertLevel::Hint,
        message: summary.message.clone(),
    }
}
