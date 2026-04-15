# VibeTalk

> Hold a key to talk, release and your speech gets transcribed and pasted into whatever input you're focused on.

A cross-platform menu-bar utility (macOS + Windows) using ByteDance OpenSpeech for real-time transcription.

[中文](./README.md)

## Features

- **Push-to-talk**: hold the hotkey to record, release to transcribe and auto-paste into any text field
- **Menu-bar resident**: click the tray icon for a translucent popover with your history; auto-hides on blur
- **History**: keeps the last 500 items; click to copy to clipboard
- **Configurable**: set your own ByteDance OpenSpeech credentials in the Settings window
- **Lightweight**: native Tauri + Rust, not Electron

## Hotkey

| Platform | Hold |
|---|---|
| macOS | **Fn** |
| Windows | **Right Alt** |

## Install

### macOS

1. Grab the latest `VibeTalk_*.dmg` from [Releases](https://github.com/Zeus233/vibetalk/releases)
2. Open the dmg, drag VibeTalk to Applications
3. Launch from Launchpad. On first run you'll need to grant:
   - **Microphone**: accept when prompted
   - **Accessibility**: System Settings → Privacy & Security → Accessibility → enable VibeTalk (required to listen for the Fn key and synthesize Cmd+V)
4. Not notarized — first launch may be blocked by Gatekeeper. Right-click the app and choose "Open" once to bypass.

### Windows

1. Grab the latest `VibeTalk_*.msi` from [Releases](https://github.com/Zeus233/vibetalk/releases)
2. Run the installer, launch from the Start menu
3. Grant microphone access on first recording

## API configuration

You'll need your own ByteDance OpenSpeech credentials:

1. Sign up at [Volcano Engine Speech Services](https://console.volcengine.com/speech/service/10011) and enable the large-model speech recognition service
2. Note your `App ID`, `Access Token`, and `Resource ID`
3. Right-click the VibeTalk tray icon → Settings, enter the three values and save

## Usage

Hold the hotkey → speak → release → wait 1–3 seconds; the text pastes itself into the focused input. The tray icon turns orange while recording.

Left-click the tray icon to expand the history popover:
- Click any row to copy it to the clipboard
- Hover an item and click × to delete
- Bottom row: Clear / Settings / Quit

## Build from source

Requires Node 18+, Rust stable, npm.

```bash
cd app
npm install
npm run tauri dev      # dev mode
npm run tauri build    # bundle, output in src-tauri/target/release/bundle/
```

## Stack

- [Tauri 2](https://tauri.app/) — app framework
- Rust — backend (audio capture, HTTP, hotkey, clipboard)
- TypeScript + Vite — frontend (popover / settings)
- ByteDance OpenSpeech — transcription API

~77% of code is shared across platforms; only the global hotkey listener and paste-key synthesis need platform-specific branches.

## License

MIT
