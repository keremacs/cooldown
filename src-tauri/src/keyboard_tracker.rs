//! Global keyboard listener for typing cadence (minimal footprint via rdev).

use std::sync::Arc;

use rdev::{listen, Event, EventType};

use crate::state::AppState;

pub fn start(state: Arc<AppState>) {
    std::thread::Builder::new()
        .name("cooldown-keyboard".into())
        .spawn(move || {
            if let Err(e) = listen(move |event| on_event(&state, event)) {
                eprintln!("[cooldown] keyboard listener error: {e:?}");
            }
        })
        .expect("failed to spawn keyboard tracker");
}

fn on_event(state: &Arc<AppState>, event: Event) {
    if matches!(event.event_type, EventType::KeyPress(_)) {
        state.record_keystroke();
    }
}
