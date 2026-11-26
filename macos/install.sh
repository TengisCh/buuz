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
# macOS does not always start a new input method by itself. Start it now, so
# that selecting Buuz works at once.
open -g "$DEST/Buuz.app" || true

# The welcome and settings app.
if [ -d build/Settings.app ]; then
  # Quit a running copy first. An old copy keeps running after its files are
  # replaced, and it can show old state, for example "not added" in step 3.
  was_running=false
  if pgrep -x BuuzSettings >/dev/null; then
    was_running=true
    pkill -x BuuzSettings || true
    for _ in 1 2 3 4 5 6 7 8 9 10; do
      pgrep -x BuuzSettings >/dev/null || break
      sleep 0.5
    done
  fi
  rm -rf "$HOME/Applications/Buuz.app"
  mkdir -p "$HOME/Applications"
  cp -R build/Settings.app "$HOME/Applications/Buuz.app"
  echo "Installed $HOME/Applications/Buuz.app (open it for the welcome steps)"
  # Open the new copy again if an old copy was open.
  if [ "$was_running" = true ]; then
    open "$HOME/Applications/Buuz.app" || true
  fi
fi
