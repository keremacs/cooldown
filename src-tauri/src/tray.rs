//! System tray — app icon with fatigue-colored status ring.

use std::path::PathBuf;
use std::sync::OnceLock;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

const TRAY_ID: &str = "cooldown-tray";

pub fn setup<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Cooldown", true, None::<&str>)?;
    let widget = MenuItem::with_id(app, "widget", "Toggle Widget", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &widget, &quit])?;

    let icon = tray_icon_for_fatigue(0.0);
    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Cooldown — Fatigue: 0")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "widget" => toggle_widget(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn update_tray<R: Runtime>(app: &AppHandle<R>, fatigue: f64) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_icon(Some(tray_icon_for_fatigue(fatigue)));
        let _ = tray.set_tooltip(Some(format!(
            "Cooldown — Fatigue: {:.0} ({})",
            fatigue,
            zone_label(fatigue)
        )));
    }
}

fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn toggle_widget<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("widget") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
        }
    }
}

fn zone_label(score: f64) -> &'static str {
    if score >= 75.0 {
        "Burnout"
    } else if score >= 40.0 {
        "Distraction"
    } else {
        "Flow"
    }
}

fn status_color(fatigue: f64) -> (u8, u8, u8) {
    if fatigue >= 75.0 {
        (239, 68, 68)
    } else if fatigue >= 40.0 {
        (245, 158, 11)
    } else {
        (34, 197, 94)
    }
}

/// Branded tray icon: app logo center + colored status ring by fatigue zone.
fn tray_icon_for_fatigue(fatigue: f64) -> Image<'static> {
    static BASE: OnceLock<Vec<u8>> = OnceLock::new();
    let base = BASE.get_or_init(|| {
        load_png_rgba(icon_path("tray-icon.png"))
            .or_else(|| load_png_rgba(icon_path("32x32.png")))
            .unwrap_or_else(generate_fallback_logo)
    });

    let (sr, sg, sb) = status_color(fatigue);
    let size = 32usize;
    let mut rgba = base.clone();
    if rgba.len() >= size * size * 4 {
        draw_status_ring(&mut rgba, size, sr, sg, sb);
    }
    let leaked: &'static [u8] = Box::leak(rgba.into_boxed_slice());
    Image::new(leaked, size as u32, size as u32)
}

fn icon_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons").join(name)
}

fn load_png_rgba(path: PathBuf) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    Some(img.to_rgba8().into_raw())
}

fn draw_status_ring(rgba: &mut [u8], size: usize, r: u8, g: u8, b: u8) {
    let cx = size as f32 / 2.0 - 0.5;
    let cy = size as f32 / 2.0 - 0.5;
    let outer = size as f32 / 2.0 - 0.5;
    let inner = outer - 2.5;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= outer && d >= inner {
                let i = (y * size + x) * 4;
                rgba[i] = r;
                rgba[i + 1] = g;
                rgba[i + 2] = b;
                rgba[i + 3] = 255;
            }
        }
    }
}

fn generate_fallback_logo() -> Vec<u8> {
    let size = 32usize;
    let mut rgba = vec![0u8; size * size * 4];
    let cx = 15.5f32;
    let cy = 15.5f32;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let i = (y * size + x) * 4;
            if d <= 13.0 {
                rgba[i] = 79;
                rgba[i + 1] = 70;
                rgba[i + 2] = 229;
                rgba[i + 3] = 255;
            } else if d <= 14.5 {
                rgba[i + 3] = 255;
            }
        }
    }
    rgba
}
