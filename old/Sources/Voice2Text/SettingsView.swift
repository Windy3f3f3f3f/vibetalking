import SwiftUI

struct SettingsView: View {
    @State private var axTrusted = HotkeyMonitor.isTrusted(prompt: false)
    @State private var micStatus = MicPermission.status()

    var body: some View {
        Form {
            Section("快捷键") {
                HStack {
                    Text("Push-to-Talk")
                    Spacer()
                    Text("按住 Fn").foregroundColor(.secondary)
                }
                Text("按住 Fn 说话，松开后识别并自动粘贴到当前输入框。")
                    .font(.caption).foregroundColor(.secondary)
            }

            Section("API (字节 OpenSpeech)") {
                HStack { Text("App ID"); Spacer(); Text(Config.appID).font(.system(.body, design: .monospaced)).foregroundColor(.secondary) }
                HStack { Text("Access Token"); Spacer(); Text(maskedToken).font(.system(.body, design: .monospaced)).foregroundColor(.secondary) }
                HStack { Text("语言"); Spacer(); Text(Config.language).foregroundColor(.secondary) }
            }

            Section("权限") {
                HStack {
                    Text("辅助功能")
                    Spacer()
                    Text(axTrusted ? "✅ 已授权" : "❌ 未授权")
                        .foregroundColor(axTrusted ? .green : .red)
                }
                HStack {
                    Text("麦克风")
                    Spacer()
                    Text(micLabel).foregroundColor(micStatus == .authorized ? .green : .red)
                }
                HStack {
                    Button("请求辅助功能") {
                        _ = HotkeyMonitor.isTrusted(prompt: true)
                        axTrusted = HotkeyMonitor.isTrusted(prompt: false)
                    }
                    Button("打开系统设置") {
                        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                            NSWorkspace.shared.open(url)
                        }
                    }
                }
            }
        }
        .padding()
        .frame(width: 420, height: 380)
        .onAppear {
            axTrusted = HotkeyMonitor.isTrusted(prompt: false)
            micStatus = MicPermission.status()
        }
    }

    private var maskedToken: String {
        let t = Config.accessToken
        guard t.count > 8 else { return "***" }
        return "\(t.prefix(4))…\(t.suffix(4))"
    }

    private var micLabel: String {
        switch micStatus {
        case .authorized: return "✅ 已授权"
        case .denied: return "❌ 已拒绝"
        case .notDetermined: return "⏳ 待请求"
        case .restricted: return "⚠️ 受限"
        @unknown default: return "?"
        }
    }
}

import AVFoundation

enum MicPermission {
    static func status() -> AVAuthorizationStatus {
        AVCaptureDevice.authorizationStatus(for: .audio)
    }
    static func request(_ completion: @escaping (Bool) -> Void) {
        AVCaptureDevice.requestAccess(for: .audio) { ok in
            DispatchQueue.main.async { completion(ok) }
        }
    }
}
