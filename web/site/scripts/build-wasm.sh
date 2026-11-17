#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Builds ../wasm (buuz-core via wasm-bindgen) into src/wasm/.
# Installs Rust and a matching wasm-bindgen CLI if they are missing (Vercel, CI).
set -euo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  export PATH="$HOME/.cargo/bin:$PATH"
fi
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

(cd ../wasm && cargo build --release --target wasm32-unknown-unknown --locked)

# The CLI version must equal the wasm-bindgen crate version in Cargo.lock.
ver=$(awk '/name = "wasm-bindgen"$/{getline; gsub(/[^0-9.]/,"",$0); print; exit}' ../wasm/Cargo.lock)
if ! command -v wasm-bindgen >/dev/null || [ "$(wasm-bindgen --version | awk '{print $2}')" != "$ver" ]; then
  tmp=$(mktemp -d)
  curl -sSfL "https://github.com/rustwasm/wasm-bindgen/releases/download/$ver/wasm-bindgen-$ver-x86_64-unknown-linux-musl.tar.gz" | tar xz -C "$tmp"
  export PATH="$tmp/wasm-bindgen-$ver-x86_64-unknown-linux-musl:$PATH"
fi

rm -rf src/wasm
wasm-bindgen ../wasm/target/wasm32-unknown-unknown/release/buuz_wasm.wasm \
  --target web --out-dir src/wasm --out-name buuz
ls -l src/wasm
