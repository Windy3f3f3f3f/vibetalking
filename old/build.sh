#!/bin/bash
set -e

echo "Building Voice2Text..."
swift build -c release

APP_DIR="Voice2Text.app/Contents/MacOS"
mkdir -p "$APP_DIR"
cp .build/release/Voice2Text "$APP_DIR/Voice2Text"
cp Resources/Info.plist Voice2Text.app/Contents/Info.plist

echo "Done! Run with: open Voice2Text.app"
