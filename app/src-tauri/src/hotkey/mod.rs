#[derive(Debug, Clone, Copy)]
pub enum HotkeyEvent {
    Pressed,
    Released,
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{check_accessibility_trusted, spawn_listener};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::spawn_listener;

#[cfg(target_os = "windows")]
pub fn check_accessibility_trusted(_prompt: bool) -> bool {
    true
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn spawn_listener(_tx: tokio::sync::mpsc::UnboundedSender<HotkeyEvent>) {}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn check_accessibility_trusted(_prompt: bool) -> bool {
    true
}
