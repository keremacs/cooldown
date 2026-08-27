//! Event-driven active window tracking.
//!
//! Windows: `SetWinEventHook` with `EVENT_SYSTEM_FOREGROUND` (zero polling).
//! Other platforms: lightweight 1 s poll via `active-win-pos-rs`.

use std::sync::{mpsc, Arc, OnceLock};
use std::time::Duration;

use crate::state::AppState;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

static FOREGROUND_SIGNAL: OnceLock<mpsc::Sender<()>> = OnceLock::new();

pub fn start(state: Arc<AppState>) {
    std::thread::Builder::new()
        .name("cooldown-window".into())
        .spawn(move || window_loop(state))
        .expect("failed to spawn window tracker");
}

fn window_loop(state: Arc<AppState>) {
    #[cfg(target_os = "windows")]
    {
        if windows_hook_loop(&state).is_err() {
            eprintln!("[cooldown] win event hook unavailable, falling back to polling");
            poll_loop(state);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        poll_loop(state);
    }
}

#[cfg(not(target_os = "windows"))]
fn poll_loop(state: Arc<AppState>) {
    let mut last = (String::new(), String::new());
    loop {
        if let Ok(win) = active_win_pos_rs::get_active_window() {
            let title = win.title.clone();
            let app = normalize_app_name(&win.app_name);
            let key = (title.clone(), app.clone());
            if key != last {
                last = key;
                state.record_window_change(title, app);
            }
        }
        state.tick_screen_time();
        std::thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(not(target_os = "windows"))]
fn normalize_app_name(raw: &str) -> String {
    raw.rsplit(['/', '\\'])
        .next()
        .unwrap_or(raw)
        .trim_end_matches(".app")
        .to_string()
}

#[cfg(target_os = "windows")]
fn poll_loop(state: Arc<AppState>) {
    let mut last = (String::new(), String::new());
    loop {
        if let Some((title, app)) = read_foreground_window() {
            let key = (title.clone(), app.clone());
            if key != last {
                last = key;
                state.record_window_change(title, app);
            }
        }
        state.tick_screen_time();
        std::thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(target_os = "windows")]
fn windows_hook_loop(state: &Arc<AppState>) -> Result<(), ()> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Accessibility::SetWinEventHook;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, PeekMessageW, TranslateMessage, EVENT_SYSTEM_FOREGROUND,
        MSG, PM_REMOVE,
    };

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }

    let (tx, rx) = mpsc::channel();
    FOREGROUND_SIGNAL
        .set(tx)
        .map_err(|_| ())?;

    unsafe extern "system" fn foreground_hook(
        _hook: windows::Win32::UI::Accessibility::HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        _id_object: i32,
        _id_child: i32,
        _thread: u32,
        _time: u32,
    ) {
        if event == EVENT_SYSTEM_FOREGROUND && !hwnd.0.is_null() {
            if let Some(tx) = FOREGROUND_SIGNAL.get() {
                let _ = tx.send(());
            }
        }
    }

    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(foreground_hook),
            0,
            0,
            0,
        )
    };

    if hook.is_invalid() {
        return Err(());
    }

    // Initial reading.
    if let Some((title, app)) = read_foreground_window() {
        state.record_window_change(title, app);
    }

    loop {
        while rx.try_recv().is_ok() {
            if let Some((title, app)) = read_foreground_window() {
                state.record_window_change(title, app);
            }
        }

        state.tick_screen_time();

        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
            // Block briefly so the hook thread stays responsive without busy-waiting.
            let _ = GetMessageW(&mut msg, None, 0, 0);
        }
    }
}

#[cfg(target_os = "windows")]
fn read_foreground_window() -> Option<(String, String)> {
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

        let app = process_name(pid).unwrap_or_else(|| "unknown".to_string());
        Some((title, app))
    }
}

#[cfg(target_os = "windows")]
fn process_name(pid: u32) -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 260];
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
        Some(
            path.rsplit(['\\', '/'])
                .next()
                .unwrap_or("unknown")
                .to_string(),
        )
    }
}
