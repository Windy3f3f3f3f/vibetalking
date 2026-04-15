use anyhow::{anyhow, Result};
use arboard::Clipboard;

pub fn copy_text(text: &str) -> Result<()> {
    let mut cb = Clipboard::new().map_err(|e| anyhow!("clipboard: {}", e))?;
    cb.set_text(text.to_string())
        .map_err(|e| anyhow!("clipboard set: {}", e))?;
    Ok(())
}

pub fn paste_text(text: &str) -> Result<()> {
    copy_text(text)?;
    std::thread::sleep(std::time::Duration::from_millis(60));
    synthesize_paste()
}

#[cfg(target_os = "macos")]
fn synthesize_paste() -> Result<()> {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    const KEY_V: CGKeyCode = 9;
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| anyhow!("CGEventSource::new failed"))?;
    let down = CGEvent::new_keyboard_event(source.clone(), KEY_V, true)
        .map_err(|_| anyhow!("v down"))?;
    down.set_flags(CGEventFlags::CGEventFlagCommand);
    down.post(CGEventTapLocation::HID);
    let up = CGEvent::new_keyboard_event(source, KEY_V, false).map_err(|_| anyhow!("v up"))?;
    up.set_flags(CGEventFlags::CGEventFlagCommand);
    up.post(CGEventTapLocation::HID);
    Ok(())
}

#[cfg(target_os = "windows")]
fn synthesize_paste() -> Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        VIRTUAL_KEY, VK_CONTROL, VK_V,
    };

    fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
    let inputs = [
        key(VK_CONTROL, false),
        key(VK_V, false),
        key(VK_V, true),
        key(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        return Err(anyhow!("SendInput sent {}/{}", sent, inputs.len()));
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn synthesize_paste() -> Result<()> {
    Err(anyhow!("paste not supported on this platform"))
}
