import AppKit
import CoreGraphics

enum TextInjector {
    /// Writes text to pasteboard and synthesizes ⌘V to the frontmost app.
    static func paste(_ text: String) {
        copy(text)
        usleep(60_000) // 60ms — let pasteboard settle
        synthesizeCmdV()
    }

    static func copy(_ text: String) {
        let pb = NSPasteboard.general
        pb.clearContents()
        pb.setString(text, forType: .string)
    }

    private static func synthesizeCmdV() {
        let src = CGEventSource(stateID: .hidSystemState)
        let vKey: CGKeyCode = 9 // ANSI "V"
        guard let down = CGEvent(keyboardEventSource: src, virtualKey: vKey, keyDown: true),
              let up = CGEvent(keyboardEventSource: src, virtualKey: vKey, keyDown: false) else {
            return
        }
        down.flags = .maskCommand
        up.flags = .maskCommand
        down.post(tap: .cghidEventTap)
        up.post(tap: .cghidEventTap)
    }
}
