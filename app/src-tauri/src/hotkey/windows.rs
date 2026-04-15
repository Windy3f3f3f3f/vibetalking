use super::HotkeyEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tokio::sync::mpsc::UnboundedSender;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RMENU;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static PRESSED: AtomicBool = AtomicBool::new(false);
static SENDER: OnceLock<UnboundedSender<HotkeyEvent>> = OnceLock::new();

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        if kbd.vkCode == VK_RMENU.0 as u32 {
            let w = wparam.0 as u32;
            let is_down = w == WM_KEYDOWN || w == WM_SYSKEYDOWN;
            let is_up = w == WM_KEYUP || w == WM_SYSKEYUP;
            if is_down {
                let was = PRESSED.swap(true, Ordering::SeqCst);
                if !was {
                    if let Some(tx) = SENDER.get() {
                        let _ = tx.send(HotkeyEvent::Pressed);
                    }
                }
            } else if is_up {
                let was = PRESSED.swap(false, Ordering::SeqCst);
                if was {
                    if let Some(tx) = SENDER.get() {
                        let _ = tx.send(HotkeyEvent::Released);
                    }
                }
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

pub fn spawn_listener(tx: UnboundedSender<HotkeyEvent>) {
    let _ = SENDER.set(tx);
    std::thread::spawn(move || unsafe {
        let hmod = match GetModuleHandleW(None) {
            Ok(h) => h,
            Err(e) => {
                log::error!("GetModuleHandleW failed: {:?}", e);
                return;
            }
        };
        let hinst = HINSTANCE(hmod.0);
        let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hinst, 0) {
            Ok(h) => h,
            Err(e) => {
                log::error!("SetWindowsHookExW failed: {:?}", e);
                return;
            }
        };
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let _ = UnhookWindowsHookEx(hook);
    });
}
