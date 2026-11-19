#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# End-to-end test of the installed input method. Run after macos/build.sh and
# macos/install.sh, on a Mac with a logged-in user (a GitHub macOS runner has one).
set -euo pipefail

cd "$(dirname "$0")/.."
OUT=build
mkdir -p "$OUT"

swiftc -O e2e/main.swift Settings/InputSource.swift -o "$OUT/e2e"

# Tell LaunchServices about the input method, and start it once so that it
# shows up in the process list. macOS starts it by itself when it is selected.
IM="$HOME/Library/Input Methods/Buuz.app"
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$IM" || true
open -g "$IM" || true
sleep 2
pgrep -lf "Buuz" || echo "e2e: Buuz is not running after open"

status=0
"$OUT/e2e" || status=$?

if [ "$status" -ne 0 ]; then
  echo "--- e2e failed with status $status. Diagnostics:"
  pgrep -lf Buuz || echo "no Buuz process is running"
  ls -la "$HOME/Library/Input Methods" || true
  log show --last 3m --style compact --predicate '(process == "Buuz" OR process == "imklaunchagent" OR process == "TextInputMenuAgent" OR process == "e2e" OR eventMessage CONTAINS "inputmethod.Buuz") AND NOT subsystem == "com.apple.networkextension"' 2>/dev/null | tail -60 || true
fi
exit "$status"
