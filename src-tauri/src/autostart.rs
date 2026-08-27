//! OS startup registration — launch Cooldown in the background on boot.

use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

pub const BACKGROUND_ARG: &str = "--background";

pub fn launched_in_background() -> bool {
    std::env::args().any(|arg| arg == BACKGROUND_ARG)
}

pub fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// Apply the user's autostart preference to the OS (release builds only).
pub fn sync_autostart(app: &AppHandle, enabled: bool) {
    if cfg!(debug_assertions) {
        return;
    }

    let mgr = app.autolaunch();
    let os_enabled = mgr.is_enabled().unwrap_or(false);
    let result = match (enabled, os_enabled) {
        (true, false) => mgr.enable(),
        (false, true) => mgr.disable(),
        _ => Ok(()),
    };

    if let Err(err) = result {
        eprintln!("[cooldown] autostart sync failed: {err}");
    }
}
