# 0001: Shared Rust core, native shells, WASM for web

Status: Accepted
Date: 2026-11-17

## Context

Buuz converts phonetic Latin typing into Mongolian Cyrillic on Windows, macOS,
the web, and later mobile. The typing rules must behave identically everywhere,
and typing must feel instant.

## Decision

- One shared core in Rust (`core/`) holds the conversion rules.
- Each platform gets a thin native shell: C++ on Text Services Framework for
  Windows, Swift on InputMethodKit for macOS.
- The web converter uses the same core compiled to WebAssembly.

See [architecture.md](../architecture.md) for details.

## Consequences

- A rule fix lands once and reaches every platform.
- Shells stay small and contain no typing logic.
- Contributors need Rust for rule changes, and platform toolchains only for shells.
