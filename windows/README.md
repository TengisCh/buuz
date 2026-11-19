<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Buuz for Windows

A Text Services Framework (TSF) input method for Mongolian. It is a COM DLL
in C++17 that Windows loads into each app. The typing rules come from the
Rust core, through the C library in [`../ffi`](../ffi).

Status: first version, see issue #76. It types and shows the underlined
preedit. There is no installer, settings app or onboarding yet.

## Layout

| File | Job | Windows headers |
| --- | --- | --- |
| `src/keymap.*` | Virtual key to `BuuzKey`, US QWERTY translation | no |
| `src/engine.*` | One key in, one edit plan out; UTF-8 to UTF-16 | no |
| `src/text_service.*` | `ITfKeyEventSink`, edit sessions, composition | yes |
| `src/display_attributes.*` | Underline for the preedit | yes |
| `src/registration.cpp` | `DllMain`, class factory, `regsvr32` | yes |
| `tests/engine_test.cpp` | Tests for the files marked "no" | no |

## How a key moves

1. TSF calls `OnKeyDown`. The shell reads the modifiers and maps the key
   as US QWERTY, whatever layout is installed.
2. `Engine::process` calls `buuz_process_key` and reads the commit, the
   preedit and the cursor. It converts them to UTF-16.
3. If the text field must change, the shell runs one synchronous edit
   session: delete before the caret, commit, show the new preedit.
4. If the core does not consume the key (Space, Enter, Ctrl+C), the key goes
   on to the app after the commit.

Nothing on this path uses the network, the disk or a log. If the core or an
edit session fails, the shell drops its state and lets the key through.

The shell reads `%APPDATA%\Buuz\settings.conf` once when it starts in an
app (portable config format, `docs/decisions/0005`). A missing file gives the
smart defaults.

## Build

Needs Visual Studio 2022 (C++ tools), CMake 3.20 and Rust.

```sh
# Static C runtime, so the DLL needs no redistributable.
cd ffi && RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --locked && cd ..
cmake -S windows -B windows/build -A x64
cmake --build windows/build --config Release
ctest --test-dir windows/build -C Release --output-on-failure
```

The logic tests also build on Linux and macOS:

```sh
cd ffi && cargo build --locked && cd ..
cmake -S windows -B windows/build -DBUUZ_FFI_LIB=$PWD/ffi/target/debug/libbuuz_ffi.a
cmake --build windows/build && ctest --test-dir windows/build
```

## Try it

Run these in an administrator terminal, in a test machine or VM:

```sh
regsvr32 windows\build\Release\buuz_tsf.dll
```

Then open Settings, Time and language, Language and region, Mongolian,
Options, and check that the keyboard "Buuz 2.0" is there. Switch to it
with Win+Space and type `sain baina uu`. To remove it:

```sh
regsvr32 /u windows\build\Release\buuz_tsf.dll
```

## Not done yet

- Installer, signing (SignPath or Microsoft Store), winget.
- ARM64, ARM64X and x86 builds.
- Settings app and onboarding (`docs/desktop-onboarding.md`), in Mongolian
  and English. Mongolian text needs a native check by T.
- Ctrl+Space Buuz on/off key, per-app memory, password fields.
- Key-to-glyph timing on real hardware. Measure it before the first release.
