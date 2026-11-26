// SPDX-License-Identifier: MIT OR Apache-2.0
// End-to-end test on a real Mac. It needs no Accessibility permission.
//
// It makes a window with a standard Cocoa text view, turns on the installed
// Buuz input method, selects it for that text view, and sends key events
// through NSTextInputContext. The events go through macOS to the real Buuz
// process (InputMethodKit), and the text view shows what Buuz gives back. The
// test compares that text with the expected Cyrillic.
//
// Run through macos/e2e/run.sh after macos/build.sh and macos/install.sh.

import AppKit
import Foundation

setvbuf(stdout, nil, _IOLBF, 0)

func log(_ s: String) { print("e2e: \(s)") }

/// Run the main run loop for `seconds` so IMK replies can arrive.
func pump(_ seconds: Double) { RunLoop.main.run(until: Date(timeIntervalSinceNow: seconds)) }

func waitFor(_ what: String, timeout: Double, _ ok: () -> Bool) -> Bool {
    let end = Date(timeIntervalSinceNow: timeout)
    while Date() < end {
        if ok() { return true }
        pump(0.25)
    }
    log("timeout: \(what)")
    return false
}

// US QWERTY key codes for the keys the test types.
let keyCodes: [Character: UInt16] = [
    "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9, "b": 11,
    "q": 12, "w": 13, "e": 14, "r": 15, "y": 16, "t": 17, "o": 31, "u": 32, "i": 34, "p": 35,
    "l": 37, "j": 38, "k": 40, "n": 45, "m": 46, " ": 49, "⌫": 51,
]

let app = NSApplication.shared
app.setActivationPolicy(.regular)

let textView = NSTextView(frame: NSRect(x: 0, y: 0, width: 400, height: 200))
let window = NSWindow(
    contentRect: NSRect(x: 100, y: 100, width: 400, height: 200),
    styleMask: [.titled], backing: .buffered, defer: false)
window.contentView = textView
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
window.makeFirstResponder(textView)
pump(0.5)

// 1. Turn Buuz on.
log("registering and enabling \(InputSource.id)")
if !InputSource.isEnabled { InputSource.enable() }
guard waitFor("Buuz in the enabled keyboards", timeout: 30, { InputSource.isEnabled }) else { exit(2) }
log("Buuz is enabled")

// 2. Select it for the text view.
guard let context = textView.inputContext else { log("no input context"); exit(3) }
log("keyboards offered to this text view: \(context.keyboardInputSources ?? [])")
context.selectedKeyboardInputSource = InputSource.id
pump(0.5)
log("current keyboard after the text view selected Buuz: \(InputSource.currentID ?? "none")")
if InputSource.currentID != InputSource.id {
    // Select it for the whole login session instead.
    log("select capable: \(InputSource.isSelectCapable)")
    log("TISSelectInputSource status: \(String(describing: InputSource.select()))")
    _ = waitFor("Buuz is the current keyboard", timeout: 20, { InputSource.currentID == InputSource.id })
}
log("current keyboard: \(InputSource.currentID ?? "none")")
log("text view keyboard: \(context.selectedKeyboardInputSource ?? "none")")
pump(1.0)  // macOS starts the input method process

// 3. Type. A key Buuz does not use goes to the text view, like in a real app.
func press(_ ch: Character) {
    guard let code = keyCodes[ch] else { fatalError("no key code for \(ch)") }
    let s = ch == "⌫" ? "\u{7f}" : String(ch)
    let event = NSEvent.keyEvent(
        with: .keyDown, location: .zero, modifierFlags: [],
        timestamp: ProcessInfo.processInfo.systemUptime,
        windowNumber: window.windowNumber, context: nil,
        characters: s, charactersIgnoringModifiers: s, isARepeat: false, keyCode: code)!
    if !context.handleEvent(event) {
        textView.insertText(s, replacementRange: NSRange(location: NSNotFound, length: 0))
    }
    pump(0.03)
}

func typeAndRead(_ keys: String) -> String {
    textView.string = ""
    textView.unmarkText()
    for ch in keys { press(ch) }
    pump(0.3)
    return textView.string
}

let cases: [(String, String)] = [
    ("sain ", "сайн "),
    ("geree ", "гэрээ "),
    ("sain baina uu ", "сайн байна уу "),
]

var failed = false
for (keys, want) in cases {
    var got = typeAndRead(keys)
    if got != want {
        // The input method may need a moment after it starts. Try once more.
        pump(2.0)
        got = typeAndRead(keys)
    }
    if got == want {
        log("ok   \(keys.debugDescription) -> \(got.debugDescription)")
    } else {
        log("FAIL \(keys.debugDescription): got \(got.debugDescription), want \(want.debugDescription)")
        failed = true
    }
}

// While a word is open, the text view must show marked text.
_ = typeAndRead("sai")
if textView.hasMarkedText() && textView.string == "сай" {
    log("ok   marked text while typing")
} else {
    log("FAIL marked text: hasMarkedText=\(textView.hasMarkedText()) string=\(textView.string.debugDescription)")
    failed = true
}

// Backspace after a space opens the word again: the text view deletes the
// committed word and shows it as marked text.
_ = typeAndRead("tov ⌫")
if textView.string == "тов" && textView.hasMarkedText() {
    log("ok   Backspace after a space reopens the word")
} else {
    log("FAIL reopen: hasMarkedText=\(textView.hasMarkedText()) string=\(textView.string.debugDescription)")
    failed = true
}

exit(failed ? 1 : 0)
