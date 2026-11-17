#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# PostToolUse hook: rustfmt a Rust file right after Claude edits it, so
# `cargo fmt --check` in CI never fails on agent edits. Reads the hook JSON on
# stdin; never blocks the edit (always exits 0).
set -u
file=$(jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0
case "$file" in
  *.rs) ;;
  *) exit 0 ;;
esac
[ -f "$file" ] || exit 0
rustfmt_bin=$(command -v rustfmt || echo "$HOME/.cargo/bin/rustfmt")
[ -x "$rustfmt_bin" ] || exit 0
"$rustfmt_bin" --edition 2021 "$file" >/dev/null 2>&1 || true
exit 0
