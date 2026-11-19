#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Build macos/build/strings-editor.html: one file, no network needed to use it.
#
#   macos/tools/strings-editor/build.sh
#   open macos/build/strings-editor.html
#
# Needs Node. It builds the Buuz converter (web/wasm) with the web site's
# script when web/site/src/wasm is missing. Set REBUILD_WASM=1 to build it again.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
wasm="$root/web/site/src/wasm"
export PATH="$HOME/.cargo/bin:$PATH"

command -v node >/dev/null || { echo "strings-editor: Node is needed" >&2; exit 1; }
if [ -n "${REBUILD_WASM:-}" ] || [ ! -f "$wasm/buuz.js" ] || [ ! -f "$wasm/buuz_bg.wasm" ]; then
  "$root/web/site/scripts/build-wasm.sh"
fi
node "$here/build.mjs" "$root/macos/build/strings-editor.html"
echo "Open it with: open $root/macos/build/strings-editor.html"
