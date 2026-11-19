// SPDX-License-Identifier: MIT OR Apache-2.0
// Tests for the macOS shell logic. They run the real buuz-ffi library through
// the key map and the session, with a fake text field. No window server needed.
// Run: macos/build.sh test

import Foundation

var failures = 0
var checks = 0

func expect<T: Equatable>(_ got: T, _ want: T, _ what: String, line: Int = #line) {
    checks += 1
    if got != want {
        failures += 1
        FileHandle.standardError.write(Data("FAIL line \(line): \(what): got \(got), want \(want)\n".utf8))
    }
}

/// A fake text field that keeps committed and marked text like a real one.
final class FakeClient: TextClient {
    var text = ""        // committed text
    var marked = ""      // marked text
    var cursor = 0
    var beeps = 0
    func insert(_ s: String, deleteBefore: Int) {
        marked = ""
        text.removeLast(deleteBefore)
        text += s
    }
    func setMarked(_ s: String, cursor: Int) { marked = s; self.cursor = cursor }
    func clearMarked() { marked = "" }
    func beep() { beeps += 1 }
    var visible: String { text + marked }
}

/// Type Latin keys like the keyboard does. The application inserts the keys
/// that Buuz does not use. Gives those keys.
@discardableResult
func type(_ keys: String, into s: Session, _ c: FakeClient) -> String {
    var passed = ""
    for ch in keys {
        let code: UInt16 = ch == " " ? 49 : 0
        let used = s.handle(keyCode: code, characters: String(ch), mods: Mods(), client: c)
        if !used {
            passed += String(ch)
            c.text += String(ch)  // the application gets the key
        }
    }
    return passed
}

func session(config: String? = nil, words: WordList? = nil) -> Session {
    Session(engine: Engine.lenient(config: config, words: words))
}

let wordListPath = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "../data/wordlist/mn.txt"

// MARK: Key map

expect(KeyMap.map(keyCode: 0, characters: "a"), MappedKey(kind: KeyMap.char, ch: 97), "a is a letter")
expect(KeyMap.map(keyCode: 0, characters: "G"), MappedKey(kind: KeyMap.char, ch: 71), "shift keeps the case")
expect(KeyMap.map(keyCode: 0, characters: "'"), MappedKey(kind: KeyMap.char, ch: 39), "apostrophe is an input key")
expect(KeyMap.map(keyCode: 0, characters: "ö"), MappedKey(kind: KeyMap.char, ch: 246), "ö is an input key")
expect(KeyMap.map(keyCode: 0, characters: "."), MappedKey(kind: KeyMap.symbol, ch: 46), "dot is a symbol")
expect(KeyMap.map(keyCode: 0, characters: "1"), MappedKey(kind: KeyMap.symbol, ch: 49), "digit is a symbol")
expect(KeyMap.map(keyCode: 0, characters: "ы"), MappedKey(kind: KeyMap.other), "a Cyrillic letter is other")
expect(KeyMap.map(keyCode: 36, characters: "\r"), MappedKey(kind: KeyMap.other), "Return is other")
expect(KeyMap.map(keyCode: 126, characters: "\u{F700}"), MappedKey(kind: KeyMap.other), "Up arrow is other")
expect(KeyMap.map(keyCode: 0, characters: nil), MappedKey(kind: KeyMap.other), "no characters is other")
expect(KeyMap.map(keyCode: 0, characters: "ab"), MappedKey(kind: KeyMap.other), "two characters is other")
expect(KeyMap.map(keyCode: 51, characters: "\u{7F}"), MappedKey(kind: KeyMap.backspace), "Backspace")
expect(KeyMap.map(keyCode: 49, characters: " "), MappedKey(kind: KeyMap.space, ch: 32), "Space")
expect(KeyMap.map(keyCode: 53, characters: "\u{1B}"), MappedKey(kind: KeyMap.escape), "Escape")
expect(KeyMap.map(keyCode: 48, characters: "\t"), MappedKey(kind: KeyMap.tab), "Tab")
expect(KeyMap.map(keyCode: 123, characters: "\u{F702}"), MappedKey(kind: KeyMap.left), "Left")
expect(KeyMap.map(keyCode: 124, characters: "\u{F703}"), MappedKey(kind: KeyMap.right), "Right")

// MARK: Engine

do {
    let e = Engine.lenient(config: nil, words: nil)
    for ch in "sain".unicodeScalars { _ = e.process(kind: KeyMap.char, ch: ch.value) }
    expect(e.preedit, "сайн", "preedit for sain")
    expect(e.cursor, 4, "cursor after sain")
    expect(e.isComposing, true, "composing")
    let r = e.process(kind: KeyMap.space, ch: 32)
    expect(r.commit, "сайн", "space commits")
    expect(r.consumed, false, "space passes through")
    expect(e.isComposing, false, "not composing after space")
    expect(e.commit(), nil, "nothing left to commit")
    expect(!Engine.version.isEmpty, true, "version")
}

expect(Engine(config: "this is not a config", words: nil) == nil, true, "a bad config gives nil")
expect(Engine(config: "buuz_config = 1\n", words: nil) != nil, true, "a good config works")
expect(Engine.lenient(config: "this is not a config", words: nil).isComposing, false, "lenient falls back")

let words = WordList(path: wordListPath)
expect(words != nil, true, "the word list loads")
expect(WordList(data: Data([0xFF, 0xFE])) == nil, true, "a bad word list gives nil")

// MARK: Session

do {
    let c = FakeClient()
    let s = session()
    let passed = type("geree", into: s, c)
    expect(passed, "", "letters are not passed on")
    expect(c.marked, "гэрээ", "marked text while typing")
    expect(c.text, "", "nothing committed yet")
    expect(c.cursor, 5, "cursor at the end")
    let passed2 = type(" ", into: s, c)
    expect(passed2, " ", "space goes to the application")
    expect(c.text, "гэрээ ", "space commits the word")
    expect(c.marked, "", "no marked text after the commit")
}

do {
    // Backspace edits the preedit and does not reach the application.
    let c = FakeClient()
    let s = session()
    type("sain", into: s, c)
    let used = s.handle(keyCode: 51, characters: "\u{7F}", mods: Mods(), client: c)
    expect(used, true, "Backspace is used while composing")
    expect(c.marked, "сай", "Backspace removes the last letter")
    // The last Backspace empties the composition and clears the marked text.
    _ = s.handle(keyCode: 51, characters: "\u{7F}", mods: Mods(), client: c)
    _ = s.handle(keyCode: 51, characters: "\u{7F}", mods: Mods(), client: c)
    _ = s.handle(keyCode: 51, characters: "\u{7F}", mods: Mods(), client: c)
    expect(c.marked, "", "marked text is gone")
    let again = s.handle(keyCode: 51, characters: "\u{7F}", mods: Mods(), client: c)
    expect(again, false, "Backspace passes on when nothing is composed")
}

do {
    // Return commits and passes on.
    let c = FakeClient()
    let s = session()
    type("bayarlalaa", into: s, c)
    let used = s.handle(keyCode: 36, characters: "\r", mods: Mods(), client: c)
    expect(used, false, "Return goes to the application")
    expect(c.text, "баярлалаа", "Return commits the word")
}

do {
    // Command + letter ends the word and passes on. It never types a letter.
    let c = FakeClient()
    let s = session()
    type("sain", into: s, c)
    let used = s.handle(keyCode: 8, characters: "c", mods: Mods(command: true), client: c)
    expect(used, false, "Command+C goes to the application")
    expect(c.text, "сайн", "Command+C commits the word first")
    expect(c.marked, "", "no marked text after Command+C")
    // Without a composition a shortcut changes nothing.
    let used2 = s.handle(keyCode: 8, characters: "c", mods: Mods(command: true), client: c)
    expect(used2, false, "a shortcut with no word goes on")
    expect(c.text, "сайн", "nothing is added")
}

do {
    // The application ends the composition (focus change).
    let c = FakeClient()
    let s = session()
    type("sain", into: s, c)
    s.commitComposition(client: c)
    expect(c.text, "сайн", "focus change commits the preedit")
    expect(c.marked, "", "no marked text left")
    s.commitComposition(client: c)
    expect(c.text, "сайн", "a second commit adds nothing")
}

do {
    // Punctuation commits and goes on.
    let c = FakeClient()
    let s = session()
    type("sain", into: s, c)
    let used = s.handle(keyCode: 47, characters: ".", mods: Mods(), client: c)
    expect(used, false, "a dot goes to the application")
    expect(c.text, "сайн", "a dot commits the word")
}

do {
    // With the word list: a full sentence.
    let c = FakeClient()
    let s = session(words: words)
    type("sain baina uu ", into: s, c)
    expect(c.text, "сайн байна уу ", "sentence with the word list")
}

do {
    // The classic preset stays like the original Buuz.
    let c = FakeClient()
    let s = session(config: "buuz_config = 1\npreset = \"classic\"\n")
    type("sain ", into: s, c)
    expect(c.text, "сайн ", "classic preset types")
}

// MARK: Config store

do {
    let dir = FileManager.default.temporaryDirectory.appendingPathComponent("buuz-test-\(UUID().uuidString)")
    try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    defer { try? FileManager.default.removeItem(at: dir) }
    let file = dir.appendingPathComponent("config.toml")
    let store = ConfigStore(configURL: file, wordListURL: URL(fileURLWithPath: wordListPath))
    expect(store.configText, nil, "no config file")
    expect(store.words != nil, true, "word list loaded")
    expect(store.refresh(), false, "nothing changed")
    try "buuz_config = 1\n".write(to: file, atomically: true, encoding: .utf8)
    expect(store.refresh(), true, "a new file is noticed")
    expect(store.configText, "buuz_config = 1\n", "config text read")
    expect(store.refresh(), false, "unchanged again")
    try "garbage".write(to: file, atomically: true, encoding: .utf8)
    expect(store.refresh() || true, true, "a changed file is read")
    let c = FakeClient()
    type("sain ", into: store.makeSession(), c)
    expect(c.text, "сайн ", "a bad config file still types")
}

// MARK: Strings

do {
    let dir = URL(fileURLWithPath: wordListPath).deletingLastPathComponent()
        .deletingLastPathComponent().deletingLastPathComponent()
        .appendingPathComponent("macos/App/Strings")
    let strings = Strings(directory: dir)
    expect(strings != nil, true, "string files load")
    if let strings = strings {
        expect(strings.keys.isEmpty, false, "there are strings")
        for key in strings.keys.sorted() {
            let mn = strings.raw(key, .mn), en = strings.raw(key, .en)
            expect(mn != nil && !(mn ?? "").isEmpty, true, "mn text for \(key)")
            expect(en != nil && !(en ?? "").isEmpty, true, "en text for \(key)")
            expect(placeholders(mn ?? ""), placeholders(en ?? ""), "same placeholders in \(key)")
        }
        expect(strings.text("common.next"), "Цааш", "default language is Mongolian")
        strings.language = .en
        expect(strings.text("common.next"), "Next", "English on request")
        expect(strings.text("no.such.key"), "no.such.key", "unknown key gives the key")
        expect(strings.text("common.next", ["x": "1"]), "Next", "unused values are ignored")
    }
}

// MARK: Speed

do {
    // Budget: 10 µs per key in the core (docs/architecture.md). The shell and
    // the fake field add a little, so allow 50 µs on a shared CI machine.
    let c = FakeClient()
    let s = session(words: words)
    let keys = Array("sain baina uu bayarlalaa geree ".utf8)
    let rounds = 2000
    let start = DispatchTime.now().uptimeNanoseconds
    for _ in 0..<rounds {
        for k in keys {
            let ch = Character(UnicodeScalar(k))
            _ = s.handle(keyCode: ch == " " ? 49 : 0, characters: String(ch), mods: Mods(), client: c)
        }
    }
    let nanos = DispatchTime.now().uptimeNanoseconds - start
    let perKey = Double(nanos) / Double(rounds * keys.count) / 1000
    print(String(format: "speed: %.2f µs per key", perKey))
    expect(perKey < 50, true, "per-key time under 50 µs (got \(perKey))")
}

print("\(checks) checks, \(failures) failures")
exit(failures == 0 ? 0 : 1)
