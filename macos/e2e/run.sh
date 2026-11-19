#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# End-to-end test of the installed input method. Run after macos/build.sh and
# macos/install.sh, on a Mac with a logged-in user (a GitHub macOS runner has one).
set -euo pipefail

cd "$(dirname "$0")/.."
OUT=build
mkdir -p "$OUT"

swiftc -O e2e/main.swift Settings/InputSource.swift -o "$OUT/e2e"

status=0
"$OUT/e2e" || status=$?

if [ "$status" -ne 0 ]; then
  echo "--- e2e failed with status $status. Diagnostics:"
  pgrep -lf Buuz || echo "no Buuz process is running"
  ls -la "$HOME/Library/Input Methods" || true
  log show --last 3m --style compact --predicate 'process == "Buuz" OR eventMessage CONTAINS "Buuz"' 2>/dev/null | tail -60 || true
fi
exit "$status"
