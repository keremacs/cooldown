mod analytics;
mod benchmark;
mod db;
mod fatigue;
mod focus;
mod http_server;
mod keyboard_tracker;
mod models;
mod notifications;
mod plugins;
mod power_monitor;
mod privacy;
mod state;
mod tray;
mod window_tracker;

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, State};

use db::Database;
use models::{DashboardState, JournalEntry, TrendPeriod};
use plugins::PluginRegistry;
use state::AppState;

#[tauri::command]
fn get_dashboard(state: State<'_, Arc<AppState>>) -> DashboardState {
    state.dashboard()
}

#[tauri::command]
fn dismiss_notification(state: State<'_, Arc<AppState>>) {
    state.dismiss_notification();
}

#[tauri::command]
fn snooze_notification(state: State<'_, Arc<AppState>>, minutes: u32) {
    state.snooze(minutes);
}

#[tauri::command]
fn save_journal(state: State<'_, Arc<AppState>>, text: String) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("Journal entry cannot be empty".into());
    }
    state.save_journal(text.trim());
    Ok(())
}

#[tauri::command]
fn get_journal(state: State<'_, Arc<AppState>>, limit: u32) -> Vec<JournalEntry> {
    state.db.journal_entries(limit)
}

#[tauri::command]
fn set_focus_mode(state: State<'_, Arc<AppState>>, active: bool, duration_min: u32) {
    state.set_focus_mode(active, duration_min);
}

#[tauri::command]
fn set_theme(state: State<'_, Arc<AppState>>, theme: String) {
    state.db.set_theme(&theme);
}

#[tauri::command]
fn set_retention_days(state: State<'_, Arc<AppState>>, days: u32) {
    state.db.set_retention_days(days);
    state.db.apply_retention_policy();
}

#[tauri::command]
fn get_trend(state: State<'_, Arc<AppState>>, period: String) -> Vec<models::TrendBucket> {
    let p = if period == "monthly" {
        TrendPeriod::Monthly
    } else {
        TrendPeriod::Weekly
    };
    state.db.trend(p)
}

#[tauri::command]
fn toggle_widget(app: AppHandle) {
    if let Some(w) = app.get_webview_window("widget") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
        }
    }
}

fn start_background_services(app: &AppHandle, state: Arc<AppState>) {
    http_server::start(state.clone(), app.clone());
    window_tracker::start(state.clone());
    keyboard_tracker::start(state.clone());
    power_monitor::start(state.db.clone());

    let app_handle = app.clone();
    std::thread::Builder::new()
        .name("cooldown-emit".into())
        .spawn(move || {
            let mut baseline_tick = 0u32;
            loop {
                let dashboard = state.dashboard();
                let _ = app_handle.emit("fatigue-update", &dashboard);
                tray::update_tray(&app_handle, dashboard.fatigue_score);

                if let Some(notification) = state.check_notification() {
                    let _ = app_handle.emit("break-notification", &notification);
                } else if let Some(hint) = state.check_hint_toast() {
                    let _ = app_handle.emit("alert-hint", &hint);
                }

                baseline_tick += 1;
                if baseline_tick % 1800 == 0 {
                    state.maybe_update_baseline();
                    state.db.apply_retention_policy();
                }

                std::thread::sleep(Duration::from_secs(2));
            }
        })
        .expect("failed to spawn emit loop");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            let db = Database::open(app.handle());
            db.apply_retention_policy();
            if db.compute_baseline_from_history().is_some() {
                if let Some(b) = db.compute_baseline_from_history() {
                    db.update_baseline(&b);
                }
            }

            let plugins = Arc::new(PluginRegistry::new());
            let app_state = AppState::new(db, plugins);
            app.manage(app_state.clone());

            tray::setup(app.handle())?;
            start_background_services(app.handle(), app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard,
            dismiss_notification,
            snooze_notification,
            save_journal,
            get_journal,
            set_focus_mode,
            set_theme,
            set_retention_days,
            get_trend,
            toggle_widget,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
