import Foundation

enum TranscribeError: LocalizedError {
    case submitFailed(String)
    case queryFailed(String)
    case timeout
    case emptyResult

    var errorDescription: String? {
        switch self {
        case .submitFailed(let m): return "提交失败: \(m)"
        case .queryFailed(let m): return "查询失败: \(m)"
        case .timeout: return "识别超时"
        case .emptyResult: return "识别结果为空"
        }
    }
}

enum Transcriber {
    /// Submit audio bytes and poll for result. Throws on error. Runs on caller's queue (use Task).
    static func transcribe(wav: Data) async throws -> String {
        let requestID = UUID().uuidString
        let audioB64 = wav.base64EncodedString()

        let payload: [String: Any] = [
            "user": ["uid": Config.appID],
            "audio": [
                "data": audioB64,
                "format": "wav",
                "language": Config.language,
            ],
            "request": [
                "model_name": "bigmodel",
                "enable_itn": true,
                "enable_punc": true,
                "show_utterances": false,
                "enable_speaker_info": false,
            ],
        ]

        // Submit
        var submit = URLRequest(url: Config.submitURL)
        submit.httpMethod = "POST"
        applyHeaders(&submit, requestID: requestID)
        submit.httpBody = try JSONSerialization.data(withJSONObject: payload)
        let (submitBody, submitResp) = try await URLSession.shared.data(for: submit)
        let submitCode = headerString(submitResp, "X-Api-Status-Code") ?? ""
        if submitCode != "20000000" {
            let msg = headerString(submitResp, "X-Api-Message") ?? ""
            let body = String(data: submitBody, encoding: .utf8) ?? ""
            throw TranscribeError.submitFailed("code=\(submitCode) msg=\(msg) body=\(body.prefix(200))")
        }

        // Poll
        for _ in 0..<120 {
            try await Task.sleep(nanoseconds: 1_000_000_000)
            var query = URLRequest(url: Config.queryURL)
            query.httpMethod = "POST"
            applyHeaders(&query, requestID: requestID)
            query.httpBody = "{}".data(using: .utf8)
            let (body, resp) = try await URLSession.shared.data(for: query)
            let hCode = headerString(resp, "X-Api-Status-Code") ?? ""
            let json = (try? JSONSerialization.jsonObject(with: body)) as? [String: Any] ?? [:]
            let bodyCode = (json["header"] as? [String: Any])?["code"].flatMap { "\($0)" }
            let code = bodyCode ?? hCode

            switch code {
            case "20000000":
                let result = json["result"] as? [String: Any]
                let text = result?["text"] as? String ?? ""
                if !text.isEmpty { return text }
                if bodyCode != nil { throw TranscribeError.emptyResult }
            case "20000001", "20000002":
                continue
            default:
                let msg = headerString(resp, "X-Api-Message") ?? ""
                throw TranscribeError.queryFailed("code=\(code) msg=\(msg)")
            }
        }
        throw TranscribeError.timeout
    }

    private static func applyHeaders(_ req: inout URLRequest, requestID: String) {
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.setValue(Config.appID, forHTTPHeaderField: "X-Api-App-Key")
        req.setValue(Config.accessToken, forHTTPHeaderField: "X-Api-Access-Key")
        req.setValue(Config.resourceID, forHTTPHeaderField: "X-Api-Resource-Id")
        req.setValue(requestID, forHTTPHeaderField: "X-Api-Request-Id")
        req.setValue("-1", forHTTPHeaderField: "X-Api-Sequence")
    }

    private static func headerString(_ resp: URLResponse, _ key: String) -> String? {
        guard let http = resp as? HTTPURLResponse else { return nil }
        return http.value(forHTTPHeaderField: key)
    }
}
