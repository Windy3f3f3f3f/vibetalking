# VibeTalk

> 按住一个键说话,松开自动把语音转成文字粘到当前输入框。

跨平台菜单栏小工具 (macOS + Windows),使用字节 OpenSpeech 实时转写。

[English](./README.en.md)

## 特性

- **Push-to-talk**:按住快捷键录音,松开转写并自动粘贴到任何输入框
- **菜单栏常驻**:点击图标弹出历史 popover,半透明毛玻璃风格,失焦自动收起
- **历史记录**:保留最近 500 条,点一下复制到剪贴板
- **可配置**:设置窗口里填自己的字节 OpenSpeech API 凭证
- **轻量**:原生 Tauri + Rust,不跑 Electron,内存占用小

## 快捷键

| 平台 | 按住键 |
|---|---|
| macOS | **Fn** |
| Windows | **Right Alt** |

## 安装

### macOS

1. 去 [Releases](https://github.com/Windy3f3f3f3f/vibetalk/releases) 下最新 `VibeTalk_*.dmg`
2. 打开 dmg → 把 VibeTalk 拖到 Applications
3. 从 Launchpad 启动。首次会弹三个授权:
   - **麦克风**:允许
   - **辅助功能**:系统设置 → 隐私与安全 → 辅助功能 → 勾选 VibeTalk(用于监听 Fn 键 + 合成粘贴)
4. 因为没做公证,首次启动可能被 Gatekeeper 拦,右键 App 选"打开"绕过一次即可

### Windows

1. 去 [Releases](https://github.com/Windy3f3f3f3f/vibetalk/releases) 下最新 `VibeTalk_*.msi`
2. 双击安装 → 从开始菜单启动
3. 首次录音时授权麦克风即可

## 配置 API

使用前需要自己的字节 OpenSpeech 凭证:

1. 去 [字节火山引擎语音服务](https://console.volcengine.com/speech/service/10011) 开通"录音文件识别大模型"
2. 记下 `App ID`、`Access Token`、`Resource ID`
3. 在 VibeTalk 托盘右键 → 设置,把三个值填进去保存

## 使用

按住快捷键 → 说话 → 松开 → 等 1~3 秒,文字自动粘到光标所在输入框。托盘图标会在录音时变橙。

菜单栏左键展开历史 popover:
- 单击条目复制到剪贴板
- 右上 × 删除单条
- 底部"清空 / 设置 / 退出"

## 从源码构建

需要 Node 18+、Rust stable、npm。

```bash
cd app
npm install
npm run tauri dev      # 开发模式
npm run tauri build    # 出安装包,产物在 src-tauri/target/release/bundle/
```

## 技术栈

- [Tauri 2](https://tauri.app/) — 应用框架
- Rust — 后端(录音、HTTP、热键、剪贴板)
- TypeScript + Vite — 前端(popover / 设置页)
- 字节 OpenSpeech — 语音识别 API

跨平台共享 ~77% 代码,只有全局热键监听和粘贴键合成需要平台分支。

## License

MIT
