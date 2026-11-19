#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Build the macOS input method from source. No Xcode project is needed.
#
#   macos/build.sh          build build/Buuz.app (universal, unsigned)
#   macos/build.sh test     build and run the Swift tests for this Mac
#   ARCHS=arm64 macos/build.sh    build for one architecture
set -euo pipefail

cd "$(dirname "$0")"
ROOT=..
OUT=build
MIN=12.0
HEADER=$ROOT/ffi/include/buuz.h
CORE_SWIFT=(Sources/Core/*.swift)
APP_SWIFT=(Sources/App/*.swift)

triple_for() {
  case "$1" in
    arm64) echo aarch64-apple-darwin ;;
    x86_64) echo x86_64-apple-darwin ;;
    *) echo "unknown architecture: $1" >&2; exit 1 ;;
  esac
}

# Build libbuuz_ffi.a for one architecture and print its path.
ffi_lib() {
  local triple
  triple=$(triple_for "$1")
  if command -v rustup >/dev/null 2>&1; then
    rustup target list --installed | grep -qx "$triple" || rustup target add "$triple" >&2
  fi
  (cd "$ROOT/ffi" && MACOSX_DEPLOYMENT_TARGET=$MIN cargo build --release --locked --target "$triple" >&2)
  echo "$ROOT/ffi/target/$triple/release/libbuuz_ffi.a"
}

host_arch() { uname -m | sed 's/arm64/arm64/'; }

if [ "${1:-}" = "test" ]; then
  mkdir -p "$OUT"
  arch=$(host_arch)
  lib=$(ffi_lib "$arch")
  swiftc -O -target "$arch-apple-macos$MIN" -import-objc-header "$HEADER" \
    "${CORE_SWIFT[@]}" Tests/main.swift "$lib" -o "$OUT/tests"
  "$OUT/tests" "$ROOT/data/wordlist/mn.txt"
  exit 0
fi

ARCHS=${ARCHS:-"arm64 x86_64"}
APP=$OUT/Buuz.app
rm -rf "$APP" "$OUT/slices"
mkdir -p "$OUT/slices" "$APP/Contents/MacOS" "$APP/Contents/Resources"

slices=()
for arch in $ARCHS; do
  lib=$(ffi_lib "$arch")
  swiftc -O -wmo -target "$arch-apple-macos$MIN" -import-objc-header "$HEADER" \
    -module-name Buuz "${CORE_SWIFT[@]}" "${APP_SWIFT[@]}" "$lib" \
    -o "$OUT/slices/Buuz-$arch"
  slices+=("$OUT/slices/Buuz-$arch")
done
lipo -create "${slices[@]}" -output "$APP/Contents/MacOS/Buuz"

cp Info.plist "$APP/Contents/Info.plist"
cp Resources/*.png "$APP/Contents/Resources/"
cp "$ROOT/data/wordlist/mn.txt" "$APP/Contents/Resources/mn.txt"

# The app icon: Buuz Buddy, from the exported PNGs in brand/export.
iconset=$OUT/Buuz.iconset
rm -rf "$iconset"
mkdir -p "$iconset"
b=$ROOT/brand/export/buuz-buddy
cp "$b-16.png" "$iconset/icon_16x16.png"
cp "$b-32.png" "$iconset/icon_16x16@2x.png"
cp "$b-32.png" "$iconset/icon_32x32.png"
cp "$b-64.png" "$iconset/icon_32x32@2x.png"
cp "$b-128.png" "$iconset/icon_128x128.png"
cp "$b-256.png" "$iconset/icon_128x128@2x.png"
cp "$b-256.png" "$iconset/icon_256x256.png"
cp "$b-512.png" "$iconset/icon_256x256@2x.png"
cp "$b-512.png" "$iconset/icon_512x512.png"
cp "$b-1024.png" "$iconset/icon_512x512@2x.png"
iconutil -c icns "$iconset" -o "$APP/Contents/Resources/Buuz.icns"

# The Buuz app: welcome steps and settings (SwiftUI). It shares the Core
# Swift files. The input method process does the typing.
SET=$OUT/Settings.app
rm -rf "$SET" "$OUT/settings-slices"
mkdir -p "$OUT/settings-slices" "$SET/Contents/MacOS" "$SET/Contents/Resources"
set_slices=()
for arch in $ARCHS; do
  lib=$(ffi_lib "$arch")
  swiftc -O -wmo -parse-as-library -target "$arch-apple-macos$MIN" -import-objc-header "$HEADER" \
    -module-name BuuzSettings "${CORE_SWIFT[@]}" Settings/*.swift "$lib" \
    -o "$OUT/settings-slices/BuuzSettings-$arch"
  set_slices+=("$OUT/settings-slices/BuuzSettings-$arch")
done
lipo -create "${set_slices[@]}" -output "$SET/Contents/MacOS/BuuzSettings"
cp Settings/Info.plist "$SET/Contents/Info.plist"
cp -R App/Strings "$SET/Contents/Resources/Strings"
cp "$b-256.png" "$SET/Contents/Resources/buuz-buddy-256.png"
cp "$APP/Contents/Resources/Buuz.icns" "$SET/Contents/Resources/Buuz.icns"
codesign --force --sign - "$SET"

# Unsigned for now (decision: sign at launch). An ad-hoc signature lets the
# app run on Apple silicon.
codesign --force --sign - "$APP"
echo "Built $APP and $SET"
lipo -info "$APP/Contents/MacOS/Buuz"
