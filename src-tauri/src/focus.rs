//! Focus mode — suppress non-critical alerts, track IDE-only sessions.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::FocusModeState;

pub struct FocusController {
    state: FocusModeState,
}

impl Default for FocusController {
    fn default() -> Self {
        Self {
            state: FocusModeState {
                active: false,
                until: None,
                session_secs: 0,
            },
        }
    }
}

impl FocusController {
    pub fn state(&self) -> FocusModeState {
        self.state.clone()
    }

    pub fn enable(&mut self, duration_min: u32) {
        let until = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + duration_min as i64 * 60;
        self.state.active = true;
        self.state.until = Some(until);
    }

    pub fn disable(&mut self) {
        self.state.active = false;
        self.state.until = None;
    }

    pub fn tick(&mut self, elapsed_secs: u64, in_ide: bool) {
        if !self.state.active {
            return;
        }
        if let Some(until) = self.state.until {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            if now >= until {
                self.disable();
                return;
            }
        }
        if in_ide {
            self.state.session_secs += elapsed_secs;
        }
    }

    pub fn is_active(&self) -> bool {
        if !self.state.active {
            return false;
        }
        if let Some(until) = self.state.until {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            if now >= until {
                return false;
            }
        }
        true
    }

    /// In focus mode, only critical alerts (>=90) should surface.
    pub fn should_suppress_alert(&self, fatigue: f64) -> bool {
        self.is_active() && fatigue < 90.0
    }
}
