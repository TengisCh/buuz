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
| `src/edit_plan.*` | Edit to text-field steps in UTF-16 units | no |
| `src/text_service.*` | `ITfKeyEventSink`, edit sessions, composition | yes |
| `src/display_attributes.*` | Underline for the preedit | yes |
| `src/registration.cpp` | `DllMain`, class factory, `regsvr32` | yes |
| `tests/engine_test.cpp`, `tests/edit_plan_test.cpp` | Tests for the files marked "no" | no |

## How a key moves

1. TSF calls `OnKeyDown`. The shell reads the modifiers and maps the key
   as US QWERTY, whatever layout is installed. Numpad digits and operators
   and the ISO extra key (`<` and `>`) reach the core as symbols.
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

CI runs these tests on Linux (job `logic-linux`, gcc) before it builds
for Windows. If this job fails, the Windows jobs do not start.

## Settings and onboarding app

`buuz_settings.exe` is a plain Win32 program (no extra runtime) that runs
apart from the DLL. It shows the onboarding in `docs/desktop-onboarding.md`
(welcome, privacy, add Buuz, switch key, try it, vowel tip, done), then the
settings window. The text is in Mongolian and English (`app/strings_*.tsv`).

| File | What |
| --- | --- |
| `app/main.cpp` | Window, pages, "Add Buuz" button |
| `app/app_state.*` | Language, theme, onboarding step, typing switches |
| `app/conf.*` | Edits a config file and keeps its comments |
| `app/strings.*` | UI text lookup |

- The app stores its own state in `%APPDATA%\Buuz\app.conf`.
- It writes the typing switches to `[options]` in `%APPDATA%\Buuz\settings.conf`.
  The DLL reads that file when an app starts, so a change applies in apps you open afterwards.
- "Add Buuz" calls `InstallLayoutOrTip` and then checks the profile every second.
- The tests for the logic run in `ctest` (`app_test`). The window itself needs a manual check (#84).

## Word list

The word list `mn.txt` (from `data/wordlist/`) sits next to the DLL. The build copies it beside the DLL, and the installer puts it in the install folder.

- The text service reads it on the first typing key in a process, not at activation. It parses it once for each process.
- The list turns on word choice: ий over ы (#66) and the other list rules. Without it, Buuz types without word choice.
- A missing or bad `mn.txt` is not an error. The text service types without a list.
- Parse time on a Mac is about 22 ms (release build of the core). It happens once, on the first key.
- `e2e_typing` also types `surguuliin` and expects `сургуулийн`. It skips this step when `mn.txt` is not next to the DLL.

## Installer

`installer/buuz.iss` is an [Inno Setup](https://jrsoftware.org/isinfo.php) script. CI builds one installer for each CPU:
`Buuz-<version>-windows-x64.exe` and `Buuz-<version>-windows-arm64.exe`. Both are in the `buuz-windows-<arch>` artifact.

- It needs an administrator, because the text service goes into `HKLM`.
- It copies `buuz_tsf.dll`, `buuz_settings.exe` and `mn.txt` to `C:\Program Files\Buuz`.
- It registers the DLL (`regserver`) and adds a Start menu entry. The last page opens the settings app.
- If an app still holds the old DLL, Windows replaces it at the next restart.
- Uninstall (Settings, Apps) unregisters the DLL and removes the files.

CI installs the x64 installer silently, checks the registration and the files, uninstalls it, and checks again.
Run the same test by hand with `Buuz-<version>-windows-x64.exe /VERYSILENT`.

To build it yourself, install Inno Setup 6, then run:

```sh
ISCC /DArch=x64 /DSourceDir=windows\build\Release windows\installer\buuz.iss
```

## ARM64X (ARM64 and emulated x64 apps)

On Windows ARM64, an ARM64 DLL loads only into ARM64 apps. Many apps still run as x64 under emulation. They need an x64 DLL.
The ARM64 installer therefore carries three DLLs:

| File | What |
| --- | --- |
| `buuz_tsf.dll` | ARM64X forwarder, no code. This is the registered path. |
| `buuz_tsf_arm64.dll` | The text service for ARM64 apps |
| `buuz_tsf_x64.dll` | The text service for x64 apps |

- `windows/arm64x/build.ps1` links the forwarder with `link /machine:arm64x` and two `.def` files (`forward_arm64.def`, `forward_x64.def`).
- The ARM64 CI job builds the x64 DLL too (Rust target `x86_64-pc-windows-msvc`, CMake `-A x64`, `-DBUUZ_DLL_SUFFIX=_x64`).
- The suffixed DLLs register the forwarder path, not their own (`registered_path` in `src/registration.cpp`).
- If a step fails, CI builds the plain ARM64 installer, and the ARM64X steps show red but do not block.
- Check by hand on a Windows 11 ARM VM: type in an ARM64 app (Notepad) and in an x64 app (for example an x64 build of Notepad++).

## Automated end-to-end test

`tests/e2e_typing.cpp` makes a real window with a Rich Edit control and
turns the text service on for it. It sends the real keys `sain baina` with
`SendInput`. Then it reads the text from the control and expects `сайн байна`.

The `windows` workflow runs it after `regsvr32` on two free GitHub
runners: x64 (`windows-latest`) and ARM64 (`windows-11-arm`). It runs on every
PR that changes `windows/`, `ffi/` or the core. This replaces manual testing
for typing; a human check of the real feel is still worth doing before a release. It
needs the runner's interactive desktop and `ctfmon.exe`. A failure on x64 blocks the PR. The ARM64 run does not block: on the
`windows-11-arm` runner the test window never becomes the foreground window,
so the keys go to another window and the control stays empty. The DLL
builds, registers and loads on ARM64 there, and the shell logic tests pass.
Typing on ARM64 is checked by hand in a Windows 11 ARM VM (steps below and
issue #84). The ARM64 step shows as passed in the job list, but its log
says `FAIL`. Read the log until this works.

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

- Code signing (SignPath or Microsoft Store) and winget. The installer is unsigned, so SmartScreen warns.
- x86 (32-bit) build.
- Settings app: import and export, abbreviation and key editors, per-app
  memory, update check, dark theme polish. Mongolian text needs a native
  check by T.
- Ctrl+Space Buuz on/off key, per-app memory, password fields.
- Key-to-glyph timing on real hardware. Measure it before the first release.
