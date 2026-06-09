#!/bin/bash
# Assemble ccgotchi.app — the menu-bar tray GUI + bundled CLI + icon.
# Prereq:  cargo build --release --workspace   (produces target/release/{ccgotchi-app,ccgotchi})
set -e
cd "$(dirname "$0")"

APP="build/ccgotchi.app"
GUI="${GUI_BIN:-target/release/ccgotchi-app}"
CLI="${CLI_BIN:-target/release/ccgotchi}"

[ -x "$GUI" ] || { echo "missing $GUI — run: cargo build --release"; exit 1; }
[ -x "$CLI" ] || { echo "missing $CLI — run: cargo build --release"; exit 1; }

rm -rf build
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp "$GUI" "$APP/Contents/MacOS/ccgotchi-app"
cp "$CLI" "$APP/Contents/MacOS/ccgotchi"   # sibling — used as the statusline command

# icon.png -> icon.icns
ICONSET="build/icon.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z $s $s gui/icons/icon.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  d=$((s * 2))
  sips -z $d $d gui/icons/icon.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/icon.icns"
rm -rf "$ICONSET"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>ccgotchi</string>
  <key>CFBundleDisplayName</key><string>ccgotchi</string>
  <key>CFBundleIdentifier</key><string>com.ccgotchi.app</string>
  <key>CFBundleVersion</key><string>0.1.0</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleExecutable</key><string>ccgotchi-app</string>
  <key>CFBundleIconFile</key><string>icon</string>
  <key>LSMinimumSystemVersion</key><string>10.15</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# strip quarantine for local builds
xattr -cr "$APP" 2>/dev/null || true

echo "✅ built $APP"
echo "   launch: open $APP"
