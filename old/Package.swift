// swift-tools-version: 5.7
import PackageDescription

let package = Package(
    name: "Voice2Text",
    platforms: [.macOS(.v12)],
    targets: [
        .executableTarget(
            name: "Voice2Text",
            path: "Sources/Voice2Text"
        )
    ]
)
