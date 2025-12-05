#!/bin/bash
set -e

APP_NAME="Macro"
APP_DIR="$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "Building binaries..."
cargo build --release

echo "Creating app bundle structure..."
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

echo "Copying binaries..."
cp target/release/macro "$MACOS_DIR/"

echo "Creating Info.plist..."
cat > "$CONTENTS_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>macro</string>
    <key>CFBundleIdentifier</key>
    <string>com.event-replay.macro</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF



echo "App bundle created at $APP_DIR"

echo "Creating DMG..."
DMG_NAME="$APP_NAME.dmg"
# npx create-dmg will overwrite if we pass the flag, but let's be safe
rm -f "$DMG_NAME"

# Create DMG using create-dmg for better UX (Applications link, icons)
# usage: create-dmg <app> [destination_dir]
npx -y create-dmg "$APP_DIR" || true

# Rename the generated DMG (likely "Macro 1.0.dmg") to Macro.dmg
mv "$APP_NAME"*.dmg "$DMG_NAME" || echo "Warning: Could not rename DMG, check output."

echo "DMG created at $DMG_NAME"
