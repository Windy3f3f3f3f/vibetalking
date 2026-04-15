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
        CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
        CGEventType, CallbackResult,
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let prev = Arc::new(AtomicBool::new(false));
    let prev_cb = prev.clone();
    let tx_cb = tx.clone();

    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        vec![CGEventType::FlagsChanged],
        move |_proxy, _etype, event| {
            let flags = event.get_flags();
            let fn_pressed = flags.contains(CGEventFlags::CGEventFlagSecondaryFn);
            let was = prev_cb.swap(fn_pressed, Ordering::SeqCst);
            if fn_pressed && !was {
                let _ = tx_cb.send(HotkeyEvent::Pressed);
            } else if !fn_pressed && was {
                let _ = tx_cb.send(HotkeyEvent::Released);
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
