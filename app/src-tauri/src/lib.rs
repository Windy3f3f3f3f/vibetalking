mod config;
mod history;
mod hotkey;
mod inject;
mod recorder;
mod settings;
mod transcribe;

use chrono::Utc;
use history::{HistoryItem, HistoryStore};
use hotkey::HotkeyEvent;
use recorder::Recorder;
use settings::{Settings, SettingsStore};
use std::sync::Arc;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::mpsc;

struct AppState {
    history: Arc<HistoryStore>,
    settings: Arc<SettingsStore>,
}

#[tauri::command]
fn list_history(state: tauri::State<'_, AppState>) -> Vec<HistoryItem> {
    state.history.list()
}

#[tauri::command]
fn copy_history_item(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let item = state
        .history
        .get(&id)
        .ok_or_else(|| "item not found".to_string())?;
    if item.error.is_some() || item.text.is_empty() {
        return Err("item is empty".to_string());
    }
    inject::copy_text(&item.text).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_history_item(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.history.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.history.clear().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Settings {
    state.settings.get()
}

#[tauri::command]
fn save_settings(new: Settings, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.settings.save(new).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_meta(state: tauri::State<'_, AppState>) -> serde_json::Value {
    let s = state.settings.get();
    serde_json::json!({
        "hotkey": config::HOTKEY_LABEL,
        "platform": std::env::consts::OS,
        "accessibility_ok": hotkey::check_accessibility_trusted(false),
        "settings": s,
    })
}

#[tauri::command]
fn request_accessibility() -> bool {
    hotkey::check_accessibility_trusted(true)
}

#[tauri::command]
fn open_settings_window(app: AppHandle) {
    open_settings(&app);
}

#[tauri::command]
fn hide_popover(app: AppHandle) {
    if let Some(w) = app.get_webview_window("popover") {
        let _ = w.hide();
    }
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let history = Arc::new(HistoryStore::load().expect("history store init"));
    let settings = Arc::new(SettingsStore::load().expect("settings store init"));
    let (tx, rx) = mpsc::unbounded_channel::<HotkeyEvent>();
    let mut rx_opt = Some(rx);

    tauri::Builder::default()
        .manage(AppState {
            history: history.clone(),
            settings: settings.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            list_history,
            copy_history_item,
            delete_history_item,
            clear_history,
            get_settings,
            save_settings,
            get_meta,
            request_accessibility,
            open_settings_window,
            hide_popover,
            quit_app,
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            set_accessory_activation_policy();

            build_popover_window(app.handle())?;

            // Right-click menu fallback
            let settings_item =
                MenuItem::with_id(app, "settings", "设置…", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出 Voice2Text", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            let icon = make_tray_icon(false);
            TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, ev| match ev.id.as_ref() {
                    "quit" => app.exit(0),
                    "settings" => open_settings(app),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    log::info!("tray event: {:?}", event);
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        rect,
                        ..
                    } = event
                    {
                        let app = tray.app_handle().clone();
                        let scale = tray
                            .app_handle()
                            .get_webview_window("popover")
                            .and_then(|w| w.scale_factor().ok())
                            .unwrap_or(1.0);
                        let pos = rect.position.to_physical::<f64>(scale);
                        let sz = rect.size.to_physical::<f64>(scale);
                        toggle_popover(&app, pos.x, pos.y, sz.width, scale);
                    }
                })
                .build(app)?;

            if let Some(rx) = rx_opt.take() {
                spawn_event_loop(app.handle().clone(), history.clone(), settings.clone(), rx);
            }
            hotkey::spawn_listener(tx.clone());

            #[cfg(target_os = "macos")]
            if !hotkey::check_accessibility_trusted(false) {
                let _ = hotkey::check_accessibility_trusted(true);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("tauri run");
}

const POPOVER_W: f64 = 360.0;
const POPOVER_H: f64 = 480.0;

fn build_popover_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("popover").is_some() {
        return Ok(());
    }
    let _w = WebviewWindowBuilder::new(app, "popover", WebviewUrl::App("popover.html".into()))
        .title("VibeTalking")
        .inner_size(POPOVER_W, POPOVER_H)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .shadow(true)
        .build()?;
    Ok(())
}

fn toggle_popover(app: &AppHandle, tray_px: f64, tray_py: f64, tray_pw: f64, scale: f64) {
    let Some(w) = app.get_webview_window("popover") else {
        log::warn!("popover window missing");
        return;
    };
    let visible = w.is_visible().unwrap_or(false);
    log::info!(
        "toggle_popover: visible={} tray_physical=({},{},w={}) scale={}",
        visible, tray_px, tray_py, tray_pw, scale
    );
    if visible {
        let _ = w.hide();
        return;
    }
    let pop_pw = POPOVER_W * scale;
    let px = tray_px + tray_pw / 2.0 - pop_pw / 2.0;
    let py = tray_py + tray_pw.max(60.0); // below the tray icon in physical px
    let _ = w.set_size(LogicalSize::new(POPOVER_W, POPOVER_H));
    let _ = w.set_position(PhysicalPosition::new(px, py));
    let _ = w.show();
    let _ = w.set_focus();
    let _ = app.emit("popover-opened", ());
}

fn spawn_event_loop(
    app: AppHandle,
    history: Arc<HistoryStore>,
    settings: Arc<SettingsStore>,
    mut rx: mpsc::UnboundedReceiver<HotkeyEvent>,
) {
    std::thread::spawn(move || {
        let recorder = Recorder::new();
        while let Some(ev) = rx.blocking_recv() {
            match ev {
                HotkeyEvent::Pressed => {
                    if let Err(e) = recorder.start() {
                        log::error!("start recording failed: {}", e);
                        continue;
                    }
                    set_tray_state(&app, true);
                    let _ = app.emit("recording-state", true);
                }
                HotkeyEvent::Released => {
                    if !recorder.is_running() {
                        continue;
                    }
                    set_tray_state(&app, false);
                    let _ = app.emit("recording-state", false);
                    let (wav, duration_ms) = match recorder.stop() {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("stop: {}", e);
                            continue;
                        }
                    };
                    if wav.len() < 4_000 {
                        log::warn!("recording too short ({} bytes), skipping", wav.len());
                        continue;
                    }
                    let hist = history.clone();
                    let app_h = app.clone();
                    let snap = settings.get();
                    tauri::async_runtime::spawn(async move {
                        match transcribe::transcribe(&wav, snap).await {
                            Ok(text) => {
                                log::info!("transcribed {} chars", text.chars().count());
                                if let Err(e) = inject::paste_text(&text) {
                                    log::error!("paste failed: {}", e);
                                }
                                let _ = hist.add(HistoryItem {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    timestamp: Utc::now(),
                                    text,
                                    duration_ms,
                                    error: None,
                                });
                                let _ = app_h.emit("history-updated", ());
                            }
                            Err(e) => {
                                log::error!("transcribe failed: {}", e);
                                let _ = hist.add(HistoryItem {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    timestamp: Utc::now(),
                                    text: String::new(),
                                    duration_ms,
                                    error: Some(e.to_string()),
                                });
                                let _ = app_h.emit("history-updated", ());
                            }
                        }
                    });
                }
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn set_accessory_activation_policy() {
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    use objc2_foundation::MainThreadMarker;
    let Some(mtm) = MainThreadMarker::new() else { return };
    let ns_app = NSApplication::sharedApplication(mtm);
    ns_app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

fn set_tray_state(app: &AppHandle, recording: bool) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_icon(Some(make_tray_icon(recording)));
        let _ = tray.set_icon_as_template(!recording);
    }
}

fn make_tray_icon(recording: bool) -> Image<'static> {
    let bytes: &'static [u8] = if recording {
        include_bytes!("../icons/tray-recording.png")
    } else {
        include_bytes!("../icons/tray-idle.png")
    };
    Image::from_bytes(bytes).expect("tray icon decode")
}

fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Voice2Text 设置")
        .inner_size(520.0, 460.0)
        .resizable(false)
        .build();
}
