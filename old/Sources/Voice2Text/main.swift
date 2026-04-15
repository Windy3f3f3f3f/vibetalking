import AppKit
import SwiftUI

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let controller = StatusBarController()
app.run()

final class StatusBarController: NSObject {
    private let item: NSStatusItem
    private let menu = NSMenu()
    private let recorder = Recorder()
    private let hotkey = HotkeyMonitor()
    private let history = HistoryStore.shared
    private var settingsWindow: NSWindow?
    private var historyWindow: NSWindow?
    private var maxDurationTimer: Timer?

    override init() {
        item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        super.init()

        configureButton(recording: false)

        let historyItem = NSMenuItem(title: "转录历史", action: #selector(showHistory), keyEquivalent: "h")
        historyItem.target = self
        let settingsItem = NSMenuItem(title: "设置...", action: #selector(showSettings), keyEquivalent: ",")
        settingsItem.target = self
        let quitItem = NSMenuItem(title: "退出 Voice2Text", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        menu.addItem(historyItem)
        menu.addItem(settingsItem)
        menu.addItem(.separator())
        menu.addItem(quitItem)

        hotkey.onPress = { [weak self] in self?.startRecording() }
        hotkey.onRelease = { [weak self] in self?.stopRecordingAndTranscribe() }

        // First launch: nudge permissions
        if !HotkeyMonitor.isTrusted(prompt: false) {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                _ = HotkeyMonitor.isTrusted(prompt: true)
            }
        }
        MicPermission.request { _ in }

        hotkey.start()
    }

    // MARK: - Menu bar button

    private func configureButton(recording: Bool) {
        guard let button = item.button else { return }
        // Use SF Symbol for a crisp template icon.
        let name = recording ? "mic.fill" : "mic"
        let config = NSImage.SymbolConfiguration(pointSize: 14, weight: .regular)
        if let img = NSImage(systemSymbolName: name, accessibilityDescription: "Voice2Text")?.withSymbolConfiguration(config) {
            img.isTemplate = !recording
            button.image = img
        } else {
            button.title = recording ? "●" : "🎙"
        }
        button.target = self
        button.action = #selector(clicked(_:))
        button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        if recording {
            button.contentTintColor = .systemRed
        } else {
            button.contentTintColor = nil
        }
    }

    @objc private func clicked(_ sender: NSStatusBarButton) {
        let event = NSApp.currentEvent
        if event?.type == .rightMouseUp || (event?.modifierFlags.contains(.control) ?? false) {
            item.menu = menu
            item.button?.performClick(nil)
            item.menu = nil
        } else {
            // Left click: show the menu too (standard menu-bar UX for small apps)
            item.menu = menu
            item.button?.performClick(nil)
            item.menu = nil
        }
    }

    // MARK: - Windows

    @objc private func showSettings() {
        if let w = settingsWindow {
            w.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true); return
        }
        let w = NSWindow(contentViewController: NSHostingController(rootView: SettingsView()))
        w.title = "Voice2Text 设置"
        w.styleMask = [.titled, .closable]
        w.center()
        w.isReleasedWhenClosed = false
        w.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        settingsWindow = w
    }

    @objc private func showHistory() {
        if let w = historyWindow {
            w.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true); return
        }
        let w = NSWindow(contentViewController: NSHostingController(rootView: HistoryView()))
        w.title = "Voice2Text 历史"
        w.styleMask = [.titled, .closable, .resizable]
        w.center()
        w.isReleasedWhenClosed = false
        w.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        historyWindow = w
    }

    // MARK: - Recording flow

    private func startRecording() {
        guard !recorder.isRunning else { return }
        do {
            try recorder.start()
            configureButton(recording: true)
            maxDurationTimer?.invalidate()
            maxDurationTimer = Timer.scheduledTimer(withTimeInterval: Config.maxRecordSeconds, repeats: false) { [weak self] _ in
                self?.stopRecordingAndTranscribe()
            }
        } catch {
            NSLog("Voice2Text: start recording failed: \(error)")
            configureButton(recording: false)
        }
    }

    private func stopRecordingAndTranscribe() {
        maxDurationTimer?.invalidate()
        maxDurationTimer = nil
        guard recorder.isRunning else { return }
        configureButton(recording: false)

        let result: (Data, Int)
        do {
            result = try recorder.stop()
        } catch {
            NSLog("Voice2Text: stop failed: \(error)")
            return
        }
        let (wav, durationMs) = result
        if wav.count < 4_000 {
            NSLog("Voice2Text: recording too short (\(wav.count) bytes), skipping")
            return
        }

        Task { [history] in
            do {
                let text = try await Transcriber.transcribe(wav: wav)
                await MainActor.run {
                    TextInjector.paste(text)
                    history.add(HistoryItem(
                        id: UUID(),
                        timestamp: Date(),
                        text: text,
                        durationMs: durationMs,
                        error: nil
                    ))
                }
            } catch {
                await MainActor.run {
                    history.add(HistoryItem(
                        id: UUID(),
                        timestamp: Date(),
                        text: "",
                        durationMs: durationMs,
                        error: error.localizedDescription
                    ))
                }
            }
        }
    }
}
