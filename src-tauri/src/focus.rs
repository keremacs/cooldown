//! Focus & Pomodoro — timed sessions with work/break cycles.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{FocusModeState, PomodoroPhase};

const DEFAULT_WORK_MIN: u32 = 25;
const DEFAULT_BREAK_MIN: u32 = 5;
const DEFAULT_LONG_BREAK_MIN: u32 = 15;
const CYCLES_BEFORE_LONG: u32 = 4;

pub struct FocusController {
    state: FocusModeState,
    pomodoro_enabled: bool,
    phase: PomodoroPhase,
    completed_cycles: u32,
    work_min: u32,
    break_min: u32,
    long_break_min: u32,
    /// Set when a phase transition occurs — consumed by state to emit notification.
    pub pending_phase_message: Option<String>,
}

impl Default for FocusController {
    fn default() -> Self {
        Self {
            state: FocusModeState {
                active: false,
                until: None,
                session_secs: 0,
                pomodoro: false,
                phase: PomodoroPhase::Idle,
                cycle: 0,
            },
            pomodoro_enabled: false,
            phase: PomodoroPhase::Idle,
            completed_cycles: 0,
            work_min: DEFAULT_WORK_MIN,
            break_min: DEFAULT_BREAK_MIN,
            long_break_min: DEFAULT_LONG_BREAK_MIN,
            pending_phase_message: None,
        }
    }
}

impl FocusController {
    pub fn state(&self) -> FocusModeState {
        let mut s = self.state.clone();
        s.active = self.is_active();
        s.pomodoro = self.pomodoro_enabled;
        s.phase = self.phase;
        s.cycle = self.completed_cycles;
        s
    }

    /// Simple focus block (non-Pomodoro).
    pub fn enable(&mut self, duration_min: u32) {
        self.pomodoro_enabled = false;
        self.phase = PomodoroPhase::Work;
        self.set_timer(duration_min);
        self.state.session_secs = 0;
        self.sync_state();
    }

    /// Start Pomodoro: 25 min work → 5 min break (15 min long break every 4 cycles).
    pub fn start_pomodoro(&mut self) {
        self.pomodoro_enabled = true;
        self.completed_cycles = 0;
        self.phase = PomodoroPhase::Work;
        self.set_timer(self.work_min);
        self.state.session_secs = 0;
        self.pending_phase_message = Some(format!(
            "Pomodoro started — {} min focus. Alerts suppressed below 90.",
            self.work_min
        ));
        self.sync_state();
    }

    pub fn disable(&mut self) {
        self.state.active = false;
        self.state.until = None;
        self.state.session_secs = 0;
        self.pomodoro_enabled = false;
        self.phase = PomodoroPhase::Idle;
        self.completed_cycles = 0;
        self.sync_state();
    }

    pub fn tick(&mut self, elapsed_secs: u64, in_ide: bool) {
        if !self.state.active {
            return;
        }
        self.check_expiry();
        if self.phase == PomodoroPhase::Work && in_ide {
            self.state.session_secs += elapsed_secs;
        }
    }

    pub fn sync_expiry(&mut self) {
        self.check_expiry();
    }

    fn check_expiry(&mut self) {
        if !self.state.active {
            return;
        }
        let Some(until) = self.state.until else {
            return;
        };
        let now = unix_now();
        if now < until {
            return;
        }

        if self.pomodoro_enabled {
            self.advance_pomodoro_phase();
        } else {
            self.pending_phase_message =
                Some("Focus session complete. Time for a break!".into());
            self.disable();
        }
    }

    fn advance_pomodoro_phase(&mut self) {
        match self.phase {
            PomodoroPhase::Work => {
                self.completed_cycles += 1;
                if self.completed_cycles % CYCLES_BEFORE_LONG == 0 {
                    self.phase = PomodoroPhase::LongBreak;
                    self.set_timer(self.long_break_min);
                    self.pending_phase_message = Some(format!(
                        "Work block done! Take a {} min long break (cycle {}).",
                        self.long_break_min, self.completed_cycles
                    ));
                } else {
                    self.phase = PomodoroPhase::Break;
                    self.set_timer(self.break_min);
                    self.pending_phase_message = Some(format!(
                        "Work block done! Take a {} min break.",
                        self.break_min
                    ));
                }
            }
            PomodoroPhase::Break | PomodoroPhase::LongBreak => {
                self.phase = PomodoroPhase::Work;
                self.set_timer(self.work_min);
                self.state.session_secs = 0;
                self.pending_phase_message = Some(format!(
                    "Break over — next {} min focus block (cycle {}).",
                    self.work_min,
                    self.completed_cycles + 1
                ));
            }
            PomodoroPhase::Idle => {}
        }
        self.sync_state();
    }

    fn set_timer(&mut self, duration_min: u32) {
        self.state.active = true;
        self.state.until = Some(unix_now() + duration_min as i64 * 60);
    }

    fn sync_state(&mut self) {
        self.state.pomodoro = self.pomodoro_enabled;
        self.state.phase = self.phase;
        self.state.cycle = self.completed_cycles;
    }

    pub fn is_active(&self) -> bool {
        if !self.state.active {
            return false;
        }
        if let Some(until) = self.state.until {
            if unix_now() >= until {
                return false;
            }
        }
        true
    }

    pub fn take_phase_message(&mut self) -> Option<String> {
        self.pending_phase_message.take()
    }

    pub fn should_suppress_alert(&self, fatigue: f64) -> bool {
        if !self.is_active() || fatigue >= 90.0 {
            return false;
        }
        !self.pomodoro_enabled || self.phase == PomodoroPhase::Work
    }

    pub fn should_suppress_hint(&self) -> bool {
        self.is_active()
            && (!self.pomodoro_enabled || self.phase == PomodoroPhase::Work)
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
