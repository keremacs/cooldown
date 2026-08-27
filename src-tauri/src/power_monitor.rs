//! Screen lock detection — Windows desktop API, macOS ioreg session flag.

use std::sync::Arc;
use std::time::Duration;

use crate::db::Database;

pub fn start(db: Arc<Database>) {
    std::thread::Builder::new()
        .name("cooldown-power".into())
        .spawn(move || lock_poll_loop(db))
        .ok();
}

fn lock_poll_loop(db: Arc<Database>) {
    let mut locked = false;
    let mut open_break: Option<i64> = None;

    loop {
        std::thread::sleep(Duration::from_secs(3));

        let is_locked = screen_is_locked();
        let now = chrono::Utc::now().timestamp();

        if is_locked && !locked {
            locked = true;
            open_break = Some(db.start_break(now, "lock"));
        } else if !is_locked && locked {
            locked = false;
            if let Some(id) = open_break.take() {
                db.end_break(id, now);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn screen_is_locked() -> bool {
    use windows::Win32::System::StationsAndDesktops::{
        OpenInputDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS,
    };

    unsafe { OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_READOBJECTS).is_err() }
}

#[cfg(target_os = "macos")]
fn screen_is_locked() -> bool {
    use std::process::Command;

    Command::new("ioreg")
        .args(["-n", "Root", "-d1"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout).contains("\"CGSSessionScreenIsLocked\"=Yes")
        })
        .unwrap_or(false)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn screen_is_locked() -> bool {
    false
}
