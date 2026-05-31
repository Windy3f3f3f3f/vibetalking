use super::HotkeyEvent;
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_listener(tx: UnboundedSender<HotkeyEvent>) {
    std::thread::spawn(move || {
        if let Err(e) = run_tap(tx) {
            log::error!("fn listener crashed: {}", e);
        }
    });
}

fn run_tap(tx: UnboundedSender<HotkeyEvent>) -> anyhow::Result<()> {
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
    use core_graphics::event::{
        CallbackResult, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
        CGEventTapPlacement, CGEventType, EventField, KeyCode,
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let prev = Arc::new(AtomicBool::new(false));
    let prev_cb = prev.clone();
    let tx_cb = tx.clone();

    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![
            CGEventType::FlagsChanged,
            CGEventType::KeyDown,
            CGEventType::KeyUp,
        ],
        move |_proxy, etype, event| {
            let flags = event.get_flags();
            let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            // 注意：macOS 的方向键 / PageUp / PageDown / Home / End 等键
            // 自身就带有 CGEventFlagSecondaryFn flag（Apple 历史设计，与是否按 Fn 无关）。
            // 因此 KeyDown/KeyUp 的 flag 不能用来反推 Fn 状态，否则按一下 ↑ 就会
            // 误判为 Fn 按下，松开 ↑ 时 flag 还在又判不出释放，导致录音一直进行。
            // 只有 FlagsChanged 事件才真实反映 Fn 自身的按下/释放。
            let fn_pressed = match etype {
                CGEventType::KeyDown if keycode == KeyCode::FUNCTION as i64 => true,
                CGEventType::KeyUp if keycode == KeyCode::FUNCTION as i64 => false,
                CGEventType::FlagsChanged => {
                    flags.contains(CGEventFlags::CGEventFlagSecondaryFn)
                }
                _ => prev_cb.load(Ordering::SeqCst),
            };
            let was = prev_cb.swap(fn_pressed, Ordering::SeqCst);
            if fn_pressed && !was {
                let _ = tx_cb.send(HotkeyEvent::Pressed);
            } else if !fn_pressed && was {
                let _ = tx_cb.send(HotkeyEvent::Released);
            }
            if keycode == KeyCode::FUNCTION as i64 || fn_pressed || was {
                return CallbackResult::Drop;
            }
            CallbackResult::Keep
        },
    )
    .map_err(|_| anyhow::anyhow!("CGEventTap::new failed — enable Accessibility permission"))?;

    let loop_source = tap
        .mach_port()
        .create_runloop_source(0)
        .map_err(|_| anyhow::anyhow!("create_runloop_source failed"))?;
    let run_loop = CFRunLoop::get_current();
    unsafe {
        run_loop.add_source(&loop_source, kCFRunLoopCommonModes);
    }
    tap.enable();
    CFRunLoop::run_current();
    Ok(())
}

pub fn check_accessibility_trusted(prompt: bool) -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::string::CFString;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    }
    let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
    let value = if prompt {
        CFBoolean::true_value()
    } else {
        CFBoolean::false_value()
    };
    let dict = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef()) }
}
