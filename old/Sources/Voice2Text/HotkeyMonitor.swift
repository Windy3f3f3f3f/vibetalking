import AppKit
import ApplicationServices
import CoreGraphics
import Foundation

/// Monitors the Fn (secondary function) modifier via a global CGEventTap.
/// Fires `onPress` on Fn down-edge, `onRelease` on up-edge.
final class HotkeyMonitor {
    var onPress: (() -> Void)?
    var onRelease: (() -> Void)?

    private var tap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var lastPressed = false

    func start() {
        guard tap == nil else { return }
        let mask = (1 << CGEventType.flagsChanged.rawValue)
        let selfPtr = Unmanaged.passUnretained(self).toOpaque()

        let callback: CGEventTapCallBack = { _, type, event, refcon in
            guard let refcon = refcon else { return Unmanaged.passUnretained(event) }
            let monitor = Unmanaged<HotkeyMonitor>.fromOpaque(refcon).takeUnretainedValue()
            if type == .flagsChanged {
                monitor.handleFlags(event)
            } else if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
                if let tap = monitor.tap {
                    CGEvent.tapEnable(tap: tap, enable: true)
                }
            }
            return Unmanaged.passUnretained(event)
        }

        guard let eventTap = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .headInsertEventTap,
            options: .listenOnly,
            eventsOfInterest: CGEventMask(mask),
            callback: callback,
            userInfo: selfPtr
        ) else {
            NSLog("Voice2Text: CGEvent.tapCreate failed — need Accessibility permission")
            return
        }
        self.tap = eventTap
        let source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0)
        self.runLoopSource = source
        CFRunLoopAddSource(CFRunLoopGetMain(), source, .commonModes)
        CGEvent.tapEnable(tap: eventTap, enable: true)
    }

    func stop() {
        if let source = runLoopSource {
            CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes)
            runLoopSource = nil
        }
        if let tap = tap {
            CGEvent.tapEnable(tap: tap, enable: false)
            self.tap = nil
        }
    }

    private func handleFlags(_ event: CGEvent) {
        let flags = event.flags
        let pressed = flags.contains(.maskSecondaryFn)
        if pressed != lastPressed {
            lastPressed = pressed
            DispatchQueue.main.async {
                if pressed { self.onPress?() } else { self.onRelease?() }
            }
        }
    }

    static func isTrusted(prompt: Bool) -> Bool {
        let key = kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String
        let options = [key: prompt] as CFDictionary
        return AXIsProcessTrustedWithOptions(options)
    }
}
