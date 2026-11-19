<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Buuz for macOS

A thin Swift InputMethodKit shell over the shared Rust core. It calls the core
through the C library `buuz-ffi` (`ffi/include/buuz.h`).

Status: first version of the input method (issue #77). The onboarding and
settings app are a separate piece (`docs/desktop-onboarding.md`).

## Layout

| Path | What |
| --- | --- |
| `Sources/Core/` | Logic with no AppKit: `Engine` (the C ABI), `KeyMap`, `Session`, `ConfigStore` |
| `Sources/App/` | The IMK app: `main.swift`, `BuuzController` |
| `Tests/main.swift` | Tests that run the real FFI with a fake text field |
| `Info.plist` | Input method keys (Mongolian, Cyrillic) |
| `Resources/` | Menu icon `Buuz-menu.png` (made by `tools/make-menu-icon.py`) |
| `build.sh`, `install.sh` | Build and install without Xcode |

`BuuzController` only moves keys and text between IMK and `Session`. All
behaviour that tests can reach lives in `Sources/Core/`.

## Build and test

You need Xcode command line tools and Rust (`rustup`).

```sh
macos/build.sh test   # Swift tests for this Mac
macos/build.sh        # build/Buuz.app, universal (arm64 and x86_64), ad-hoc signed
macos/install.sh      # copy to ~/Library/Input Methods
```

Then open System Settings › Keyboard › Input Sources › Edit › ＋ › Mongolian ›
Buuz. Log out and in once if macOS does not list Buuz.

## How typing works

1. IMK gives each key down to `BuuzController.handle`.
2. `KeyMap` turns the key into a Buuz key kind. A key with Command, Control or
   Option ends the word and goes to the application.
3. `Engine.process` calls `buuz_process_key`.
4. `Session` shows the result: `insertText` for a commit (with a replacement
   range when the core asks to delete characters before the caret), and
   `setMarkedText` for the underlined preedit.

Space, Return and symbols commit the word and then go to the application.
Backspace edits the preedit. Esc follows the core rules.

## Config and word list

- The word list `mn.txt` is in the app bundle. It loads once when the app starts.
- The config is the portable file `~/Library/Application Support/Buuz/config.toml`
  (decision 0005). The input method rereads it when a text field gets focus and
  the file changed. A bad file falls back to the smart defaults, so a typo never
  stops typing.

## Speed and privacy

- The typing path makes no network call and writes no log. It allocates only the
  Swift strings that IMK needs (the preedit and the commit).
- The Swift test checks the time per key. The core budget is 10 µs.
- The app is unsigned for now (ad-hoc signature). Signing and notarization
  come at launch.

## Known limits

- Undo of a correction (the core asks to delete characters before the caret) needs
  a text field that reports the caret with `selectedRange`. Other fields keep
  the old word and add the new one.
- No menu items, per-app off or personal dictionary storage yet.
