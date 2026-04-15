import SwiftUI

struct HistoryView: View {
    @ObservedObject var store: HistoryStore = .shared
    @State private var toast: String?

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("转录历史").font(.headline)
                Spacer()
                if !store.items.isEmpty {
                    Button("清空") { store.clear() }
                }
            }
            .padding()

            Divider()

            if store.items.isEmpty {
                VStack {
                    Spacer()
                    Text("还没有转录记录").foregroundColor(.secondary)
                    Text("按住 Fn 键开始说话").font(.caption).foregroundColor(.secondary)
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 6) {
                        ForEach(store.items) { item in
                            HistoryRow(item: item, onCopy: {
                                copy(item)
                            }, onDelete: {
                                store.delete(item.id)
                            })
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                }
            }

            if let toast {
                Text(toast)
                    .font(.caption)
                    .padding(.horizontal, 12).padding(.vertical, 6)
                    .background(Color.accentColor.opacity(0.85))
                    .foregroundColor(.white)
                    .cornerRadius(6)
                    .padding(.bottom, 10)
                    .transition(.opacity)
            }
        }
        .frame(width: 440, height: 560)
    }

    private func copy(_ item: HistoryItem) {
        guard item.error == nil, !item.text.isEmpty else { return }
        TextInjector.copy(item.text)
        withAnimation { toast = "已复制到剪贴板" }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
            withAnimation { toast = nil }
        }
    }
}

struct HistoryRow: View {
    let item: HistoryItem
    let onCopy: () -> Void
    let onDelete: () -> Void
    @State private var hover = false

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(formatDate(item.timestamp))
                    .font(.caption).foregroundColor(.secondary)
                Spacer()
                Text(String(format: "%.1fs", Double(item.durationMs) / 1000.0))
                    .font(.caption).foregroundColor(.secondary)
                if hover {
                    Button(action: onDelete) {
                        Image(systemName: "trash").font(.caption)
                    }
                    .buttonStyle(.borderless)
                    .foregroundColor(.secondary)
                }
            }
            if let err = item.error {
                Text("[失败] \(err)")
                    .foregroundColor(.red)
                    .font(.system(.body))
            } else {
                Text(item.text)
                    .font(.system(.body))
                    .multilineTextAlignment(.leading)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(hover ? Color.gray.opacity(0.15) : Color.gray.opacity(0.08))
        .cornerRadius(6)
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .stroke(item.error != nil ? Color.red.opacity(0.4) : Color.clear, lineWidth: 1)
        )
        .contentShape(Rectangle())
        .onTapGesture { onCopy() }
        .onHover { hover = $0 }
    }

    private func formatDate(_ d: Date) -> String {
        let f = DateFormatter()
        f.dateFormat = "MM-dd HH:mm:ss"
        return f.string(from: d)
    }
}
