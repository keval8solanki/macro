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

echo "Processing icon..."
if [ -f "assets/icon.png" ]; then
    ICONSET_DIR="AppIcon.iconset"
    mkdir -p "$ICONSET_DIR"
    
    # Generate icons of different sizes
    sips -z 16 16     -s format png assets/icon.png --out "${ICONSET_DIR}/icon_16x16.png"
    sips -z 32 32     -s format png assets/icon.png --out "${ICONSET_DIR}/icon_16x16@2x.png"
    sips -z 32 32     -s format png assets/icon.png --out "${ICONSET_DIR}/icon_32x32.png"
    sips -z 64 64     -s format png assets/icon.png --out "${ICONSET_DIR}/icon_32x32@2x.png"
    sips -z 128 128   -s format png assets/icon.png --out "${ICONSET_DIR}/icon_128x128.png"
    sips -z 256 256   -s format png assets/icon.png --out "${ICONSET_DIR}/icon_128x128@2x.png"
    sips -z 256 256   -s format png assets/icon.png --out "${ICONSET_DIR}/icon_256x256.png"
    sips -z 512 512   -s format png assets/icon.png --out "${ICONSET_DIR}/icon_256x256@2x.png"
    sips -z 512 512   -s format png assets/icon.png --out "${ICONSET_DIR}/icon_512x512.png"
    sips -z 1024 1024 -s format png assets/icon.png --out "${ICONSET_DIR}/icon_512x512@2x.png"
    
    iconutil -c icns "$ICONSET_DIR"
    cp AppIcon.icns "$RESOURCES_DIR/"
    rm -rf "$ICONSET_DIR" AppIcon.icns
else
    echo "Warning: assets/icon.png not found, skipping icon generation"
fi

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
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF



echo "App bundle created at $APP_DIR"

echo "Signing app..."
# Ad-hoc signing to prevent "Damaged" error
if [ -z "$SIGNING_IDENTITY" ]; then
    # Try to auto-detect identity (e.g. Mac OS Certificate)
    # Using '|| true' to prevent script exit if grep fails
    SIGNING_IDENTITY=$(security find-identity -v -p codesigning | grep '"' | head -1 | awk -F '"' '{print $2}' || true)
    if [ -n "$SIGNING_IDENTITY" ]; then
        echo "Auto-detected Identity: $SIGNING_IDENTITY"
    else
         echo "No signing identity found. Using ad-hoc signing."
    fi
fi

if [ -n "$SIGNING_IDENTITY" ]; then
    echo "Signing with identity: $SIGNING_IDENTITY"
    codesign --force --deep --options runtime --sign "$SIGNING_IDENTITY" "$APP_DIR" || { echo "Signing failed"; exit 1; }
else
    echo "Warning: No signing identity found, using ad-hoc signing."
    echo "       (Apps signed ad-hoc will lose permissions on update)"
    codesign --force --deep --sign - "$APP_DIR"
fi

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
