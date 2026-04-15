import AVFoundation
import Foundation

/// Records to a 16kHz mono 16-bit WAV file (ByteDance API accepts format=wav).
final class Recorder {
    private var recorder: AVAudioRecorder?
    private var startedAt: Date?
    private(set) var outputURL: URL?

    var isRunning: Bool { recorder?.isRecording ?? false }
    var elapsed: TimeInterval {
        guard let s = startedAt else { return 0 }
        return Date().timeIntervalSince(s)
    }

    func start() throws {
        if isRunning { return }
        let url = Config.supportDir.appendingPathComponent("record-\(UUID().uuidString).wav")
        let settings: [String: Any] = [
            AVFormatIDKey: kAudioFormatLinearPCM,
            AVSampleRateKey: 16000,
            AVNumberOfChannelsKey: 1,
            AVLinearPCMBitDepthKey: 16,
            AVLinearPCMIsFloatKey: false,
            AVLinearPCMIsBigEndianKey: false,
        ]
        let rec = try AVAudioRecorder(url: url, settings: settings)
        guard rec.record() else {
            throw NSError(domain: "Voice2Text", code: 1, userInfo: [NSLocalizedDescriptionKey: "AVAudioRecorder.record() returned false"])
        }
        recorder = rec
        outputURL = url
        startedAt = Date()
    }

    /// Stops and returns (wav data, duration in ms).
    func stop() throws -> (Data, Int) {
        guard let rec = recorder, let url = outputURL, let started = startedAt else {
            throw NSError(domain: "Voice2Text", code: 2, userInfo: [NSLocalizedDescriptionKey: "not recording"])
        }
        rec.stop()
        let duration = Int(Date().timeIntervalSince(started) * 1000)
        recorder = nil
        startedAt = nil
        outputURL = nil
        let data = try Data(contentsOf: url)
        try? FileManager.default.removeItem(at: url)
        return (data, duration)
    }

    func cancel() {
        recorder?.stop()
        if let url = outputURL { try? FileManager.default.removeItem(at: url) }
        recorder = nil
        startedAt = nil
        outputURL = nil
    }
}
