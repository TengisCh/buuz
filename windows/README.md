<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Buuz for Windows

A Text Services Framework (TSF) input method for Mongolian. It is a COM DLL
in C++17 that Windows loads into each app. The typing rules come from the
Rust core, through the C library in [`../ffi`](../ffi).

Status: first version, see issue #76. It types and shows the underlined
preedit. CI builds x64 and ARM64. There is no installer, settings app or onboarding yet.

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

## Automated end-to-end test

`tests/e2e_typing.cpp` makes a real window with a Rich Edit control and
turns the text service on for it. It sends the real keys `sain baina` with
`SendInput`. Then it reads the text from the control and expects `сайн байна`.

The `windows` workflow runs it after `regsvr32` on two free GitHub
runners: x64 (`windows-latest`) and ARM64 (`windows-11-arm`). It runs on every
PR that changes `windows/`, `ffi/` or the core. This replaces manual testing
for typing; a human check of the real feel is still worth doing before a release. It
needs the runner's interactive desktop and `ctfmon.exe`. The step has
`continue-on-error: true` until it has passed on several runs. Then remove
that line, so a failure blocks the PR.

## Test on a Mac (Windows 11 ARM in a VM)

An Apple silicon Mac runs Windows 11 ARM in a virtual machine. You do not
need a Windows PC.

1. Install a VM app. Use Parallels Desktop (it downloads Windows 11 for you)
   or UTM (free; you need a Windows 11 ARM ISO).
2. Make a Windows 11 ARM VM and sign in to Windows.
3. Open the latest run of the `windows` workflow on GitHub, in the Actions
   tab of `TengisCh/buuz`.
4. Download the artifact `buuz-windows-arm64`. Unzip it on the Mac.
5. Copy the unzipped files into the VM (drag and drop, or a shared folder).
6. In the VM, open `Terminal (Admin)` and go to the folder.
7. Run `regsvr32 buuz_tsf.dll`. A message says the registration succeeded.
8. Open Settings, Time and language, Language and region. Make sure
   Mongolian has the keyboard "Buuz 2.0". Add Mongolian first if it is not
   in the list.
9. Open Notepad. Press Win+Space until "Buuz 2.0" is active.
10. Type `sain baina uu`. The text must become `сайн байна уу`, with an
    underline under the word you are still typing.

To remove it, run `regsvr32 /u buuz_tsf.dll`.

Use the `arm64` DLL in this VM. Windows loads it into ARM64 apps only
(Notepad, Edge, Terminal and most Windows apps). An x64 app running under
emulation does not load it. One DLL for both kinds of app needs ARM64X,
which is a later step.

If something fails, write down the step number and what the screen shows.

## Test on a Windows PC

Use the artifact `buuz-windows-x64`, and the same steps. The
`e2e_typing.exe` file in it repeats the automatic test on your PC.

## Not done yet

- Installer, signing (SignPath or Microsoft Store), winget.
- ARM64X (one DLL for ARM64 and emulated x64 apps) and x86 builds.
- Settings app and onboarding (`docs/desktop-onboarding.md`), in Mongolian
  and English. Mongolian text needs a native check by T.
- Ctrl+Space Buuz on/off key, per-app memory, password fields.
- Key-to-glyph timing on real hardware. Measure it before the first release.
