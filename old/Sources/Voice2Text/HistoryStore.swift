import Foundation
import Combine

struct HistoryItem: Codable, Identifiable, Equatable {
    let id: UUID
    let timestamp: Date
    let text: String
    let durationMs: Int
    let error: String?
}

final class HistoryStore: ObservableObject {
    static let shared = HistoryStore()

    @Published private(set) var items: [HistoryItem] = []

    private let fileURL: URL

    private init() {
        fileURL = Config.supportDir.appendingPathComponent("history.json")
        load()
    }

    func add(_ item: HistoryItem) {
        items.insert(item, at: 0)
        if items.count > Config.historyMax {
            items.removeLast(items.count - Config.historyMax)
        }
        persist()
    }

    func delete(_ id: UUID) {
        items.removeAll { $0.id == id }
        persist()
    }

    func clear() {
        items.removeAll()
        persist()
    }

    private func load() {
        guard let data = try? Data(contentsOf: fileURL) else { return }
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        items = (try? decoder.decode([HistoryItem].self, from: data)) ?? []
    }

    private func persist() {
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        encoder.outputFormatting = [.prettyPrinted]
        if let data = try? encoder.encode(items) {
            try? data.write(to: fileURL, options: .atomic)
        }
    }
}
