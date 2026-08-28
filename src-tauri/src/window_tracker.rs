//! Active window tracking — 1 s poll on all platforms (reliable screen-time accrual).

use std::sync::Arc;
use std::time::Duration;

use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::state::AppState;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

pub fn start(state: Arc<AppState>) {
    std::thread::Builder::new()
        .name("cooldown-window".into())
        .spawn(move || poll_loop(state))
        .expect("failed to spawn window tracker");
}

fn poll_loop(state: Arc<AppState>) {
    let mut sys = System::new();
    loop {
        if let Some((title, app)) = read_foreground_window(&mut sys) {
            state.record_window_change(title, app);
        } else {
            state.pause_self_tracking();
        }
        state.tick_screen_time();
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn normalize_app_name(raw: &str) -> String {
    raw.rsplit(['/', '\\'])
        .next()
        .unwrap_or(raw)
        .trim()
        .to_string()
}

fn is_self_process(stem: &str) -> bool {
    stem.eq_ignore_ascii_case("cooldown")
}

#[cfg(target_os = "windows")]
fn read_foreground_window(sys: &mut System) -> Option<(String, String)> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        let title = String::from_utf16_lossy(&buffer[..len as usize]);

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let app = resolve_process_name(sys, pid)?;
        if is_self_process(&process_stem(&app)) {
            return None;
        }

        Some((title, app))
    }
}

#[cfg(not(target_os = "windows"))]
fn read_foreground_window(_sys: &mut System) -> Option<(String, String)> {
    let win = active_win_pos_rs::get_active_window().ok()?;
    let app = normalize_app_name(&win.app_name);
    if is_self_process(&process_stem(&app)) {
        return None;
    }
    Some((win.title, app))
}

fn process_stem(app_name: &str) -> String {
    let base = app_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(app_name)
        .trim()
        .to_lowercase();
    base.strip_suffix(".exe")
        .or_else(|| base.strip_suffix(".app"))
        .unwrap_or(&base)
        .to_string()
}

fn resolve_process_name(sys: &mut System, pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }

    #[cfg(target_os = "windows")]
    if let Some(name) = process_name_winapi(pid) {
        return Some(normalize_app_name(&name));
    }

    let pid = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    sys.process(pid)
        .map(|p| normalize_app_name(&p.name().to_string_lossy()))
}

#[cfg(target_os = "windows")]
fn process_name_winapi(pid: u32) -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 512];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        if ok.is_err() {
            return None;
        }
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        path.rsplit(['\\', '/'])
            .next()
            .map(|s| s.to_string())
    }
}
