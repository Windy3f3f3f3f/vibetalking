import Foundation

enum Config {
    static let appID = "7236214542"
    static let accessToken = "MMTCwjoy_KAOIaYTY64ZpwPyEP0gV0N5"
    static let resourceID = "volc.seedasr.auc"
    static let language = "zh-CN"

    static let submitURL = URL(string: "https://openspeech.bytedance.com/api/v3/auc/bigmodel/submit")!
    static let queryURL = URL(string: "https://openspeech.bytedance.com/api/v3/auc/bigmodel/query")!

    static let maxRecordSeconds: TimeInterval = 300
    static let historyMax = 500

    static var supportDir: URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            .appendingPathComponent("com.voice2text.app")
        try? FileManager.default.createDirectory(at: base, withIntermediateDirectories: true)
        return base
    }
}
