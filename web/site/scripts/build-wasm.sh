#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Builds ../wasm (buuz-core via wasm-bindgen) into src/wasm/.
#
# Installs Rust and a matching wasm-bindgen CLI if they are missing (Vercel,
# CI, a fresh Mac). Every download is checked against a SHA-256 pinned in this
# file before it runs; nothing is piped into a shell.
#
# Bumping wasm-bindgen in web/wasm/Cargo.lock: add the new version's hashes to
# wasm_bindgen_sha256 below (from the release page's .sha256sum files, checked
# against the GitHub API "digest" field). Until then the script falls back to
# `cargo install wasm-bindgen-cli --locked`, which is verified by crates.io
# but takes a few minutes to compile.
set -euo pipefail
cd "$(dirname "$0")/.."

die() { echo "build-wasm: $*" >&2; exit 1; }

sha256_of() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | awk '{print $1}'
  else shasum -a 256 "$1" | awk '{print $1}'; fi
}

# fetch URL FILE SHA256: download to FILE and refuse it unless the hash matches.
fetch() {
  curl --proto '=https' --tlsv1.2 -sSfL -o "$2" "$1"
  local got
  got=$(sha256_of "$2")
  if [ "$got" != "$3" ]; then
    rm -f "$2"
    die "checksum mismatch for $1
  expected $3
  got      $got"
  fi
}

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) arch=x86_64-unknown-linux;;
  Linux-aarch64 | Linux-arm64) arch=aarch64-unknown-linux;;
  Darwin-arm64) arch=aarch64-apple-darwin;;
  Darwin-x86_64) arch=x86_64-apple-darwin;;
  *) die "unsupported host $(uname -s)-$(uname -m); install Rust and wasm-bindgen-cli yourself";;
esac

# --- Rust --------------------------------------------------------------------

RUSTUP_VERSION=1.29.3
rustup_init_sha256() {
  case "$1" in
    x86_64-unknown-linux-gnu)  echo b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55;;
    aarch64-unknown-linux-gnu) echo 9f608c142e0e38a78957d0661ba1dfaf0b6fb37184cfaa4b9a0aa57af6184282;;
    aarch64-apple-darwin)      echo 63752a1919a3a4793173ec70f8f212621a7ce38ba6a246095984482f4de20f9f;;
    x86_64-apple-darwin)       echo bc107f1fb0228de7fa32413c89c1b305cea9cd256e94b01150cd054128145429;;
  esac
}

export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
if ! command -v cargo >/dev/null; then
  case "$arch" in *-linux) triple=$arch-gnu;; *) triple=$arch;; esac
  echo "build-wasm: cargo not found, installing Rust via rustup-init $RUSTUP_VERSION ($triple)"
  tmp=$(mktemp -d)
  fetch "https://static.rust-lang.org/rustup/archive/$RUSTUP_VERSION/$triple/rustup-init" \
    "$tmp/rustup-init" "$(rustup_init_sha256 "$triple")"
  chmod +x "$tmp/rustup-init"
  "$tmp/rustup-init" -y --profile minimal --no-modify-path
  rm -rf "$tmp"
fi
if command -v rustup >/dev/null; then
  rustup target add wasm32-unknown-unknown >/dev/null
fi

(cd ../wasm && cargo build --release --target wasm32-unknown-unknown --locked)

# --- wasm-bindgen CLI ----------------------------------------------------------

# Release tarball hashes, keyed by "<version> <target>". Linux builds are musl
# (static), so they run on any glibc too.
wasm_bindgen_sha256() {
  case "$1" in
    "0.2.130 x86_64-unknown-linux-musl")  echo 9687f3bb551b9f66f37ed32a40d7913a00f7962db653054e56857bc306c3227f;;
    "0.2.130 aarch64-unknown-linux-musl") echo 99ffc93ce80081b91632e2b539b782821a8f3439ab0fddc645287462b2f55d3d;;
    "0.2.130 aarch64-apple-darwin")       echo b90ddb9357f584b1af897d58ad77e4cce2b48fc17e2a967bc249a87f72809444;;
    "0.2.130 x86_64-apple-darwin")        echo eb209c115b7cb95832423c1aa3f4d06d6c53f63ef74fc4b6fb79ef0b2eea5f10;;
  esac
}

# The CLI version must equal the wasm-bindgen crate version in Cargo.lock.
ver=$(awk '/name = "wasm-bindgen"$/{getline; gsub(/[^0-9.]/,"",$0); print; exit}' ../wasm/Cargo.lock)
[ -n "$ver" ] || die "wasm-bindgen not found in ../wasm/Cargo.lock"

if ! command -v wasm-bindgen >/dev/null || [ "$(wasm-bindgen --version | awk '{print $2}')" != "$ver" ]; then
  case "$arch" in *-linux) target=$arch-musl;; *) target=$arch;; esac
  sum=$(wasm_bindgen_sha256 "$ver $target")
  if [ -n "$sum" ]; then
    name="wasm-bindgen-$ver-$target"
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    fetch "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$ver/$name.tar.gz" \
      "$tmp/$name.tar.gz" "$sum"
    tar xzf "$tmp/$name.tar.gz" -C "$tmp"
    export PATH="$tmp/$name:$PATH"
  else
    echo "build-wasm: no pinned hash for wasm-bindgen $ver ($target); building it with cargo install." >&2
    echo "build-wasm: add the release hashes to scripts/build-wasm.sh to skip this." >&2
    root="${CARGO_HOME:-$HOME/.cargo}/buuz-wasm-bindgen-$ver"
    cargo install wasm-bindgen-cli --version "=$ver" --locked --root "$root"
    export PATH="$root/bin:$PATH"
  fi
fi

rm -rf src/wasm
wasm-bindgen ../wasm/target/wasm32-unknown-unknown/release/buuz_wasm.wasm \
  --target web --out-dir src/wasm --out-name buuz
ls -l src/wasm
