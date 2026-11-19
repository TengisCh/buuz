#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copy build/Buuz.app to ~/Library/Input Methods and start it.
# Then add Buuz in System Settings > Keyboard > Input Sources > Edit > + > Mongolian.
# Log out and in once if macOS does not list Buuz yet.
set -euo pipefail

cd "$(dirname "$0")"
APP=build/Buuz.app
DEST="$HOME/Library/Input Methods"
[ -d "$APP" ] || { echo "Run macos/build.sh first." >&2; exit 1; }

mkdir -p "$DEST"
pkill -x Buuz 2>/dev/null || true
rm -rf "$DEST/Buuz.app"
cp -R "$APP" "$DEST/Buuz.app"
echo "Installed $DEST/Buuz.app"

# The welcome and settings app.
if [ -d build/Settings.app ]; then
  rm -rf "$HOME/Applications/Buuz.app"
  mkdir -p "$HOME/Applications"
  cp -R build/Settings.app "$HOME/Applications/Buuz.app"
  echo "Installed $HOME/Applications/Buuz.app (open it for the welcome steps)"
fi
