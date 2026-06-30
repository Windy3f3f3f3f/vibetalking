# VibeTalking

> Hold a key to talk, release and your speech gets transcribed and pasted into whatever input you're focused on.

A cross-platform menu-bar utility (macOS + Windows). It transcribes speech with **Qwen3.5 Omni** (Alibaba Bailian / DashScope) by default, then optionally runs the text through an **LLM refine** pass (any OpenAI-compatible endpoint) to fix homophones, term spelling and punctuation.

[中文](./README.md)

![VibeTalking](./assets/image.png)

## Pipeline

```
Hold hotkey → record → ASR transcribe (Qwen Omni) → optional LLM refine → auto-paste at cursor
```

1. **Transcribe (ASR)**: send the recording to the selected backend (Qwen3.5 Omni Flash by default), get raw text.
2. **Refine (optional)**: send the raw text to an OpenAI-compatible chat endpoint for post-processing — fix homophones / mis-recognitions, keep the original spelling and casing of English terms in mixed Chinese-English speech, normalize numbers and version strings, restore punctuation, drop filler words ("um", "uh", "you know") — **without summarizing or rewriting**. Falls back to the raw text on error.
3. **Paste**: synthesize Cmd/Ctrl+V to paste the final text into the focused input.

## Features

- **Push-to-talk**: hold the hotkey to record, release to transcribe and auto-paste into any text field
- **Swappable transcribe backend**: Qwen3.5 Omni Flash / Omni Plus / Qwen3 ASR Flash / ByteDance OpenSpeech, switch in Settings
- **Optional LLM refine**: any OpenAI-compatible endpoint (you supply base URL + model + key) for cleaner output
- **Menu-bar resident**: click the tray icon for a translucent popover with your history; auto-hides on blur
- **History**: keeps the last 500 items, storing both the raw and refined text; click to copy to clipboard
- **Lightweight**: native Tauri + Rust, not Electron

## Hotkey

| Platform | Hold |
|---|---|
| macOS | **Fn** |
| Windows | **Right Alt** |

## Install

### macOS

1. Grab the latest `VibeTalking_*.dmg` from [Releases](https://github.com/Windy3f3f3f3f/vibetalking/releases)
2. Open the dmg, drag VibeTalking to Applications
3. Launch from Launchpad. On first run you'll need to grant:
   - **Microphone**: accept when prompted
   - **Accessibility**: System Settings → Privacy & Security → Accessibility → enable VibeTalking (required to listen for the Fn key and synthesize Cmd+V)
4. Not notarized — first launch may be blocked by Gatekeeper. Right-click the app and choose "Open" once to bypass.

### Windows

1. Grab the latest `VibeTalking_*.msi` from [Releases](https://github.com/Windy3f3f3f3f/vibetalking/releases)
2. Run the installer, launch from the Start menu
3. Grant microphone access on first recording

## Configuration

Right-click the tray icon → Settings.

### 1. Transcribe backend (ASR)

Pick a model under "转写后端" (Transcribe backend):

| Option | Notes |
|---|---|
| **Qwen3.5 Omni Flash** (default) | Bailian multimodal, fast, good at mixed Chinese-English |
| **Qwen3.5 Omni Plus** | Stronger tier of the same family |
| **Qwen3 ASR Flash** | Pure ASR model |
| **ByteDance OpenSpeech** | Legacy Volcano Engine recording-recognition (kept) |

For the Qwen backends, fill the **API Key** in the "DashScope / Qwen" card ([get one from Alibaba Bailian](https://bailian.console.aliyun.com/)). Base URL defaults to `https://dashscope.aliyuncs.com/compatible-mode/v1`; you can also leave the key blank and let it read the `DASHSCOPE_API_KEY` environment variable. "Omni Prompt" is the instruction passed to the model at transcription time — edit as needed.

### 2. LLM refine (optional)

The "Refine (LLM 后处理)" card treats **any** OpenAI-compatible chat completions endpoint as the post-processor — **which provider and which model is entirely up to you**:

- **Enable refine**: only runs the LLM pass when checked
- **Base URL / Model / API Key**: point it at your own endpoint (OpenAI, a self-hosted vLLM, any aggregator gateway, …). The default shipped in the repo is AIHUBMIX's `gpt-5.4-mini` — just a placeholder, swap it freely
- If the API Key is left blank, it falls back to the `AIHUBMIX_API_KEY` or `OPENAI_API_KEY` environment variable
- **Refine Prompt**: the post-processing prompt; editable, with a one-click reset to default

### ByteDance OpenSpeech (optional)

If you use the ByteDance backend, fill App ID / Access Token / Resource ID / Language in its card. Sign up at [Volcano Engine Speech Services](https://console.volcengine.com/speech/service/10011) and enable the large-model recording-recognition service to get credentials.

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
- Rust — backend (audio capture, HTTP, hotkey, clipboard, ASR + refine calls)
- TypeScript + Vite — frontend (popover / settings)
- Transcription: Qwen3.5 Omni / Qwen3 ASR (Alibaba Bailian DashScope) or ByteDance OpenSpeech
- Refine: any OpenAI-compatible LLM

Most of the code is shared across platforms; only the global hotkey listener and paste-key synthesis need platform-specific branches.

## License

MIT
