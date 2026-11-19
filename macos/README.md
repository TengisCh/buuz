<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Buuz for macOS

A thin Swift InputMethodKit shell over the shared Rust core. It calls the core
through the C library `buuz-ffi` (`ffi/include/buuz.h`).

Status: first version. The input method is issue #77. The Buuz app (tutorial,
lessons and settings) follows `docs/desktop-onboarding.md` and
`design/macos-app/spec.md`.

**Not released.** Do not publish, sign, notarize or link a download of this app
until the manual test in issue #84 is done and the issue is closed. No
workflow publishes this app.

## Layout

| Path | What |
| --- | --- |
| `Sources/Core/` | Logic with no AppKit: `Engine` (the C ABI), `KeyMap`, `Session`, `ConfigStore` |
| `Sources/App/` | The IMK app: `main.swift`, `BuuzController` |
| `Settings/` | The Buuz app in SwiftUI: tutorial, lessons and settings (`Settings.app`) |
| `App/Strings/` | Mongolian and English UI strings |
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
macos/install.sh      # copy to ~/Library/Input Methods and ~/Applications/Buuz.app
```

Open `~/Applications/Buuz.app` for the tutorial. The app adds Buuz to the
keyboards with one click (`TISRegisterInputSource` and `TISEnableInputSource`).
It sees by itself when Buuz is added. The settings sheet writes the switches
to the portable config file.

If the button does not work, open System Settings › Keyboard › Input Sources › Edit › ＋ › Mongolian ›
Buuz. Log out and in once if macOS does not list Buuz.

## End-to-end test

`macos/e2e/run.sh` tests the installed app on a real Mac. CI runs it on the
macOS runner after each change in `macos/`, `ffi/` or `core/src/`.

1. It turns on the installed Buuz input method (`TISRegisterInputSource`,
   `TISEnableInputSource`).
2. It makes a window with a standard Cocoa text view and selects Buuz for it.
3. It sends key events through `NSTextInputContext`. macOS routes them to the
   real Buuz process, as it does for any app.
4. It compares the text in the text view with the expected Cyrillic.

The test needs no Accessibility permission. It does not use TextEdit: reading
TextEdit needs an Automation permission that a runner cannot grant.

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

## The Buuz app

The app is a resizable window. The default size is 1000 × 680. The smallest
size is 880 × 600. The title bar is hidden. The window has a sidebar and one
large rounded panel. Each page is one scroll view with one centred column
(`PageColumn` in `Settings/Components.swift`). Nothing overlaps or clips at
any size.

| Part | What it shows |
| --- | --- |
| Sidebar | Home, Lessons, Settings and Help. A small setup card shows four checks. |
| Home | A hero card, three status tiles, a try box and a cheat sheet |
| Lessons | The eight lessons. You can replay each one. |
| Help | How to add Buuz by hand, feedback and the tutorial |
| Settings | A sheet with General, Typing, Shortcuts, Privacy and About |
| Top icons | Appearance (Light, Dark, System), updates and language |

The theme is System by default. The app sets `NSApp.appearance` at launch
and when you change it.

### Tutorial

The tutorial has five setup steps, eight lessons and a Done step. Every
step except Welcome and Done has a Skip button. The app saves the step, so
quitting resumes in the same place. The checks are real:

- **Add the keyboard.** Next turns on when macOS lists Buuz. The app
  watches the enabled keyboards.
- **Switch key.** You choose Globe, Control + Space, Caps Lock or your own
  switch key. The recorder refuses keys that macOS already uses and says
  which macOS action owns them. Your own switch key is saved in
  `macos.json`. The input method process registers it, not this app.
- **Switch to Buuz.** The app reads the current keyboard live. It watches
  the notification `com.apple.Carbon.TISNotifySelectedKeyboardInputSourceChanged`
  and window activation. Next turns on after Buuz, another keyboard, then
  Buuz again.
- **Lessons.** The practice field goes through the real input method. A
  lesson passes only when the field holds the expected Cyrillic text.
  Latin letters never pass. A banner tells you when the current keyboard is
  not Buuz, with a "Switch to Buuz" button. The checks are in
  `Sources/Core/Lessons.swift` (`LessonChecker`).

The app never saves or logs the text of the practice field.

## Switch key, theme and live checks

The switch key toggles between Buuz and your last other keyboard.
The input method process registers it with Carbon `RegisterEventHotKey`.
This needs no permission. The process never reads or logs other keys.

- The default switch key is Control+Option+Return. Control+Option+Space is a
  macOS default ("next input source"), so Buuz does not use it. Buuz uses
  the default only when no macOS shortcut and no common app shortcut uses
  it.
- With no saved choice, the settings app shows the key that the input
  method registers (`HotkeyStore.resolve`). What you see is what is
  registered.
- If the default switch key is taken, no key is set. The settings app asks
  you to record a key.
- A switch key needs Control, Option or Command. Bare keys and Shift-only
  keys are not valid. Option or Option+Shift with a letter, digit or
  punctuation key is not valid either, because it types a character in many
  layouts.
- `SystemShortcuts` reads the macOS shortcuts from
  `com.apple.symbolichotkeys`. It checks input source switching, Spotlight,
  Finder search and Mission Control. macOS leaves a default shortcut out of
  that file until the user edits it. Buuz treats a missing entry for
  Control+Space, Control+Option+Space, Command+Space and Option+Command+Space
  as on. Only an entry with `enabled` false is off.
- The key is saved in `~/Library/Application Support/Buuz/macos.json`. The
  portable `config.toml` stays the same, because its parser rejects unknown
  top-level keys.
- After a change, the settings app posts the distributed notification
  `com.thetengis.buuz.hotkeyChanged`. The input method reads the key again.
- The input method writes the result of `RegisterEventHotKey` to the
  `status` entry of `macos.json` and posts `com.thetengis.buuz.hotkeyStatus`.
  The Shortcuts page shows a warning when the registration failed.
- When you press the switch key in Buuz, Buuz selects your last other
  keyboard. If there is none, it selects your ASCII-capable keyboard, then
  the first enabled keyboard that is not Buuz.
- The input method process runs only after macOS starts Buuz. Select Buuz
  once after each login, or the switch key does nothing. Buuz adds no login
  item for this, and there is no reliable way to start the process without
  one.

The theme is `system` (default), `light` or `dark`. The old saved values
`light` and `dark` still work.

`LessonChecker` gives `waiting`, `wrongKeyboard`, `latinOnly`, `almost` or
`success` from the current keyboard and the text of the lesson field. The
checks run on that field only. `TryState` (`TryCheck.swift`) is a simpler
check for one text. Only the tests use it now; the Home try box shows no
banner.

Some lessons need a typing option: lesson 2 needs `smart_vowels`, lesson 6
needs `word_choice` and lesson 8 needs `abbreviations`
(`LessonRequirement`). The lesson shows a banner with a "Turn on" button
when the option is off or Buuz is in classic mode.

## Notices

`build.sh` copies the repository `NOTICE` file into `Contents/Resources` of
both apps. Settings › About › Open-source notices opens it.

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
