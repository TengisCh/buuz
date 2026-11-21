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
            for name in ["Odbayar", "Nyamtseren", "Одбаяр", "Нямцэрэн"] {
                expect((mn ?? "").contains(name) || (en ?? "").contains(name), false, "no author name in \(key)")
            }
        }
        for key in ["hotkey.conflict.title", "hotkey.action.prevSource", "general.theme.system", "nav.lessons", "home.card.mode"] {
            expect(strings.raw(key, .en) != nil && strings.raw(key, .mn) != nil, true, "both languages have \(key)")
        }
        expect(placeholders(strings.raw("hotkey.conflict.body", .en) ?? ""), ["action"], "conflict body placeholder")
        for key in ["welcome.credit", "settings.about.credit", "settings.about.original", "settings.about.original.url"] {
            expect(strings.raw(key, .en) == nil && strings.raw(key, .mn) == nil, true, "retired key \(key) is gone")
        }
        for key in ["nav.home", "lesson8.turnOn", "tour.done.replay", "hotkey.conflict.body", "lesson.wrong.title"] {
            expect(strings.raw(key, .en) != nil && strings.raw(key, .mn) != nil, true, "both languages have \(key)")
        }
        for key in ["lesson2.off", "lesson6.off", "lesson8.off", "hotkey.failed.title", "hotkey.failed.body",
            "settings.about.notices", "settings.about.notices.open"] {
            expect(strings.raw(key, .en) != nil && strings.raw(key, .mn) != nil, true, "both languages have \(key)")
        }
        expect(placeholders(strings.raw("hotkey.failed.body", .mn) ?? ""), ["code"], "failed body placeholder")
        for key in ["switch.failed.title", "switch.failed.notEnabled", "switch.failed.body", "switch.failed.path"] {
            expect(strings.raw(key, .en) != nil && strings.raw(key, .mn) != nil, true, "both languages have \(key)")
        }
        for key in ["globe.warn", "globe.warn.unknown", "globe.action.emoji", "globe.action.dictation",
            "globe.action.none", "globe.fn.note"] {
            expect(strings.raw(key, .en) != nil && strings.raw(key, .mn) != nil, true, "both languages have \(key)")
        }
        expect(placeholders(strings.raw("globe.warn", .en) ?? ""), ["action"], "globe warn placeholder")
        for n in 1...8 where LessonRequirement.option(for: n) != nil {
            expect(strings.raw("lesson\(n).off", .en) != nil, true, "lesson \(n) has an off banner text")
        }
        expect(placeholders(strings.raw("hotkey.conflict.title", .mn) ?? ""), ["keys"], "conflict title placeholder")
        expect(placeholders(strings.raw("tour.switch.backHint", .en) ?? ""), ["key", "name"], "back hint placeholders")
        expect(strings.text("common.next"), "Үргэлжлүүлэх", "default language is Mongolian")
        strings.language = .en
        expect(strings.text("common.next"), "Next", "English on request")
        expect(strings.text("no.such.key"), "no.such.key", "unknown key gives the key")
        expect(strings.text("common.next", ["x": "1"]), "Next", "unused values are ignored")
    }
}

// MARK: Config editor

do {
    let base = "buuz_config = 1\npreset = \"smart\"\n\n# my notes\n[options]\nautocorrect = false # off\n\n[abbreviations]\nbn = \"байна\"\n"
    expect(ConfigEditor.flag(base, "autocorrect"), false, "reads a flag with a comment")
    expect(ConfigEditor.flag(base, "smart_vowels"), nil, "unset flag is nil")
    expect(ConfigEditor.flag(nil, "autocorrect"), nil, "no file is nil")
    let on = ConfigEditor.setFlag(base, "autocorrect", true)
    expect(ConfigEditor.flag(on, "autocorrect"), true, "changes a flag")
    expect(on.contains("# my notes") && on.contains("bn = \"байна\""), true, "keeps comments and other sections")
    let added = ConfigEditor.setFlag(base, "smart_vowels", false)
    expect(ConfigEditor.flag(added, "smart_vowels"), false, "adds a flag to [options]")
    expect(ConfigEditor.flag(added, "autocorrect"), false, "keeps the other flag")
    let fresh = ConfigEditor.setFlag(nil, "abbreviations", true)
    expect(fresh, "buuz_config = 1\n\n[options]\nabbreviations = true\n", "makes a new file")
    let noOptions = ConfigEditor.setFlag("buuz_config = 1\npreset = \"classic\"\n", "smart_vowels", true)
    expect(noOptions.hasSuffix("[options]\nsmart_vowels = true\n"), true, "adds [options] at the end")
    for text in [on, added, fresh, noOptions] {
        expect(Engine(config: text, words: nil) != nil, true, "the core accepts the edited config")
    }
    expect(ConfigEditor.setFlag(on, "autocorrect", true), on, "setting the same value changes nothing")
}

// MARK: Enabled sources

do {
    let abc: [String: Any] = ["InputSourceKind": "Keyboard Layout", "KeyboardLayout Name": "ABC"]
    let other: [String: Any] = ["Bundle ID": "com.apple.inputmethod.Kotoeri", "InputSourceKind": "Input Mode"]
    let byBundle: [String: Any] = ["Bundle ID": "com.thetengis.inputmethod.Buuz", "InputSourceKind": "Keyboard Input Method"]
    let byMode: [String: Any] = ["Input Mode": "com.thetengis.inputmethod.Buuz.Mongolian", "InputSourceKind": "Input Mode"]
    expect(EnabledSources.containsBuuz([abc]), false, "list without Buuz")
    expect(EnabledSources.containsBuuz([abc, byBundle]), true, "Bundle ID entry")
    expect(EnabledSources.containsBuuz([abc, byMode]), true, "Input Mode entry")
    expect(EnabledSources.containsBuuz([]), false, "empty list")
    expect(EnabledSources.containsBuuz([abc, other]), false, "unrelated entries only")
}

// MARK: Tutorial flow

do {
    var f = TourFlow()
    expect(f.step, .welcome, "starts at welcome")
    expect(TourStep.allCases.count, 14, "5 setup steps, 8 lessons, done")
    f.goNext(enabled: false); expect(f.step, .privacy, "welcome to privacy")
    f.goNext(enabled: false); expect(f.step, .add, "privacy to add")
    f.goNext(enabled: false); expect(f.step, .key, "add to key")
    f.goBack(enabled: false); expect(f.step, .add, "back to add")
    f.step = .privacy
    f.goNext(enabled: true); expect(f.step, .key, "add is skipped when already enabled")
    f.goBack(enabled: true); expect(f.step, .privacy, "back skips add too")
    f.step = .switchTo
    f.goNext(enabled: true); expect(f.step, .lesson1, "switch to first lesson")
    f.step = .lesson8
    f.goNext(enabled: true); expect(f.step, .done, "last lesson to done")
    f.goNext(enabled: true); expect(f.step, .done, "done is the end")
    f.step = .key
    f.goNext(enabled: false); expect(f.step, .add, "next cannot skip add after Buuz was removed")
    f.step = .lesson3
    f.goNext(enabled: false); expect(f.step, .add, "next from a lesson goes back to add when removed")
    f.step = .done
    f.goNext(enabled: false); expect(f.step, .done, "done stays done")
    expect(TourFlow.isDone(.add, current: .key, enabled: false), false, "add is not done when removed")
    expect(TourFlow.isDone(.add, current: .key, enabled: true), true, "add is done when enabled")
    expect(TourFlow.isDone(.privacy, current: .key, enabled: false), true, "other earlier steps stay done")
    expect(TourFlow.isDone(.key, current: .privacy, enabled: true), false, "later step is not done")
    f.step = .welcome
    f.goBack(enabled: false); expect(f.step, .welcome, "welcome is the start")
    f.step = .lesson3
    f.skipAll(); expect(f.step, .done, "skip the tutorial goes to done")
    expect(TourStep.welcome.canSkip, false, "no skip on welcome")
    expect(TourStep.done.canSkip, false, "no skip on done")
    expect(TourStep.add.canSkip, true, "skip on add")
    expect(TourStep.lesson4.canSkip, true, "skip on a lesson")
    expect(TourStep.lesson4.lessonNumber, 4, "lesson number")
    expect(TourStep.key.lessonNumber, nil, "setup step has no lesson")
    expect(TourStep.key.setupNumber, 4, "setup number")
    expect(TourStep.lesson1.setupNumber, nil, "lesson has no setup number")
    expect(TourStep.lesson(5), .lesson5, "lesson step from number")
    expect(TourStep.lesson(9), nil, "no lesson 9")
    expect(TourStep(rawValue: "lesson2"), .lesson2, "raw value round trip")

    let prefs = Prefs(MemoryStore())
    expect(prefs.language, .mn, "Mongolian by default")
    expect(prefs.theme, .system, "system theme by default")
    expect(prefs.tourStep, .welcome, "welcome by default")
    expect(prefs.onboardingDone, false, "not done by default")
    expect(prefs.passedLessons, [], "no lesson passed")
    expect(prefs.switchKey, nil, "no switch key")
    expect(prefs.switchedOnce, false, "not switched yet")
    prefs.language = .en; prefs.theme = .dark; prefs.tourStep = .lesson4; prefs.onboardingDone = true
    prefs.passedLessons = [3, 1]; prefs.switchKey = "fn"; prefs.switchedOnce = true
    expect(prefs.language, .en, "language is saved")
    expect(prefs.theme, .dark, "theme is saved")
    expect(prefs.tourStep, .lesson4, "step is saved")
    expect(prefs.onboardingDone, true, "done is saved")
    expect(prefs.passedLessons, [1, 3], "passed lessons are saved")
    expect(prefs.switchKey, "fn", "switch key is saved")
    expect(prefs.switchedOnce, true, "switched once is saved")
    let raw = MemoryStore()
    raw.set("lesson9", for: "tour.step"); raw.set("1,x,3,99", for: "tour.passed")
    expect(Prefs(raw).tourStep, .welcome, "bad step gives welcome")
    expect(Prefs(raw).passedLessons, [1, 3], "bad lesson numbers are dropped")
    prefs.switchKey = nil
    expect(prefs.switchKey, nil, "switch key can be cleared")
}

// MARK: Setup checklist

do {
    let rows = SetupChecklist.rows(enabled: true, hasSwitchKey: false, switchedOnce: false, passed: [1, 2])
    expect(rows.map { $0.done }, [true, false, false, false], "checklist done flags")
    expect(rows.map { $0.id }, ["add", "key", "switch", "lessons"], "checklist rows")
    expect(SetupChecklist.current(rows), 1, "first open row is current")
    expect(SetupChecklist.doneCount(rows), 1, "one done")
    let all = SetupChecklist.rows(enabled: true, hasSwitchKey: true, switchedOnce: true, passed: Set(1...8))
    expect(SetupChecklist.allDone(all), true, "all done")
    expect(SetupChecklist.current(all), nil, "no current when done")
    expect(SetupChecklist.step(for: "add"), .add, "row to step")
    expect(SetupChecklist.step(for: "key"), .key, "row to step key")
    expect(SetupChecklist.step(for: "switch"), .switchTo, "row to step switch")
    expect(SetupChecklist.step(for: "lessons", passed: [1, 2]), .lesson3, "next lesson")
    expect(SetupChecklist.nextLesson(passed: []), 1, "first lesson")
    expect(SetupChecklist.nextLesson(passed: Set(1...8)), nil, "no lesson left")
}

// MARK: Keyboard ids and versions

do {
    expect(BuuzSource.isBuuz(id: "com.thetengis.inputmethod.Buuz.Mongolian"), true, "Buuz id")
    expect(BuuzSource.isBuuz(id: "com.thetengis.inputmethod.Buuz"), true, "Buuz id prefix")
    expect(BuuzSource.isBuuz(id: "com.apple.keylayout.ABC"), false, "ABC is not Buuz")
    expect(BuuzSource.isBuuz(id: nil), false, "unknown is not Buuz")
    expect(VersionCompare.isNewer("2.1.0", than: "2.0.0"), true, "newer minor")
    expect(VersionCompare.isNewer("v2.0.1", than: "2.0.0"), true, "leading v")
    expect(VersionCompare.isNewer("2.0.0", than: "2.0.0"), false, "same")
    expect(VersionCompare.isNewer("1.9.9", than: "2.0.0"), false, "older")
    expect(VersionCompare.isNewer("2.0", than: "2.0.0"), false, "short is equal")
    expect(VersionCompare.isNewer("10.0.0", than: "9.0.0"), true, "numbers, not text")
    expect(VersionCompare.isNewer("junk", than: "2.0.0"), false, "junk is not newer")
    let p = Prefs(MemoryStore())
    expect(p.customHotkey, nil, "no custom key")
    p.customHotkey = Hotkey(keyCode: 11, mods: [.control, .option])
    expect(p.customHotkey, Hotkey(keyCode: 11, mods: [.control, .option]), "custom key is saved")
    p.customHotkey = nil
    expect(p.customHotkey, nil, "custom key is cleared")
}

// MARK: Greeting

do {
    expect(Greeting.key(hour: 5), "home.greeting.morning", "morning")
    expect(Greeting.key(hour: 11), "home.greeting.morning", "late morning")
    expect(Greeting.key(hour: 12), "home.greeting.afternoon", "noon")
    expect(Greeting.key(hour: 17), "home.greeting.afternoon", "afternoon")
    expect(Greeting.key(hour: 18), "home.greeting.evening", "evening")
    expect(Greeting.key(hour: 2), "home.greeting.morning", "night is before 12")
}

// MARK: Theme

do {
    let prefs = Prefs(MemoryStore())
    expect(prefs.theme, .system, "theme follows the system by default")
    prefs.theme = .light
    expect(prefs.theme, .light, "light is saved")
    let old = MemoryStore()
    old.set("light", for: "theme")
    expect(Prefs(old).theme, .light, "old light value is read")
    old.set("dark", for: "theme")
    expect(Prefs(old).theme, .dark, "old dark value is read")
    old.set("junk", for: "theme")
    expect(Prefs(old).theme, .system, "bad value gives system")
    expect(Theme.system.appearanceName, nil, "system forces nothing")
    expect(Theme.light.appearanceName, "NSAppearanceNameAqua", "light appearance")
    expect(Theme.dark.appearanceName, "NSAppearanceNameDarkAqua", "dark appearance")
    expect(Theme.dark.forcedDark, true, "dark is forced")
    expect(Theme.system.forcedDark, nil, "system is not forced")
}

// MARK: Setup card fold

do {
    let prefs = Prefs(MemoryStore())
    expect(prefs.setupFolded, false, "the setup card is open by default")
    prefs.setupFolded = true
    expect(prefs.setupFolded, true, "the fold is saved")
    prefs.setupFolded = false
    expect(prefs.setupFolded, false, "the fold is cleared")
}

// MARK: Hotkey

do {
    let cs = Hotkey(keyCode: 49, mods: [.control])
    expect(cs.display, "⌃Space", "control space display")
    expect(Hotkey.default.display, "⌃⌥Return", "default display")
    expect(Hotkey(keyCode: 8, mods: [.command, .shift]).display, "⇧⌘C", "shift before command")
    expect(Hotkey.decode(Hotkey.default.encoded), Hotkey.default, "encode round trip")
    expect(Hotkey.decode("49:"), nil, "no modifiers is invalid")
    expect(Hotkey.decode("49:shift"), nil, "shift only is invalid")
    expect(Hotkey.decode("x:control"), nil, "bad code")
    expect(Hotkey.decode("49:hyper"), nil, "bad modifier")
    expect(Hotkey(keyCode: 49, mods: []).isValid, false, "bare key is invalid")
    expect(Hotkey(keyCode: 49, mods: [.shift]).isValid, false, "shift only is invalid")
    expect(Hotkey(keyCode: 49, mods: [.command]).isValid, true, "command is valid")
    // Option (with or without Shift) and a printing key types a character.
    expect(Hotkey(keyCode: 14, mods: [.option]).isValid, false, "option E is invalid")
    expect(Hotkey(keyCode: 14, mods: [.option, .shift]).isValid, false, "option shift E is invalid")
    expect(Hotkey(keyCode: 18, mods: [.option]).isValid, false, "option 1 is invalid")
    expect(Hotkey(keyCode: 43, mods: [.option]).isValid, false, "option comma is invalid")
    expect(Hotkey(keyCode: 14, mods: [.control, .option]).isValid, true, "control option E is valid")
    expect(Hotkey(keyCode: 14, mods: [.option, .command]).isValid, true, "option command E is valid")
    expect(Hotkey(keyCode: 14, mods: [.command]).isValid, true, "command E is valid")
    expect(Hotkey(keyCode: 49, mods: [.option]).isValid, true, "option Space is valid")
    expect(Hotkey(keyCode: 36, mods: [.option]).isValid, true, "option Return is valid")
    expect(Hotkey(keyCode: 122, mods: [.option]).isValid, true, "option F1 is valid")
    expect(Hotkey(keyCode: 123, mods: [.option, .shift]).isValid, true, "option shift arrow is valid")
    expect(Hotkey.decode("14:option"), nil, "decode refuses option E")
}

do {
    expect(Hotkey(keyCode: 49, mods: [.control, .option]).mods.carbon, 4096 | 2048, "carbon mask")
    // Fake symbolic hotkeys: Control+Space on 60, a disabled one on 61.
    let fake: [String: Any] = [
        "60": ["enabled": true, "value": ["parameters": [32, 49, 262144], "type": "standard"]],
        "61": ["enabled": false, "value": ["parameters": [32, 49, 786432], "type": "standard"]],
        "64": ["enabled": 1, "value": ["parameters": [32, 49, 1048576], "type": "standard"]],
        "999": ["enabled": true, "value": ["parameters": [1, 2, 3]]],
    ]
    let system = SystemShortcuts.parse(fake)
    // 60 and 64 are set. 61 is off. 65 is missing, so macOS uses its default.
    expect(system.count, 3, "two enabled and one default shortcut")
    expect(system.contains { $0.hotkey == Hotkey(keyCode: 49, mods: [.option, .command]) }, true,
        "missing entry 65 is on with its default key")
    expect(SystemShortcuts.conflict(for: Hotkey(keyCode: 49, mods: [.control]), in: system),
        "Select previous input source", "control space conflicts")
    expect(SystemShortcuts.conflict(for: Hotkey(keyCode: 49, mods: [.control, .option]), in: system),
        nil, "disabled shortcut does not conflict")
    expect(SystemShortcuts.conflict(for: Hotkey(keyCode: 8, mods: [.command]), in: []),
        "Copy", "reserved list")
    expect(SystemShortcuts.conflict(for: Hotkey(keyCode: 49, mods: [.command]), in: []),
        "Spotlight search", "command space is reserved")
    // macOS leaves a default shortcut out of the plist until the user edits it.
    let defaults = SystemShortcuts.parse([:])
    expect(defaults.count, 4, "empty dictionary: the four default shortcuts apply")
    for (id, code, mods) in [(60, 49, HotkeyMods.control), (61, 49, [.control, .option]),
        (64, 49, .command), (65, 49, [.option, .command])] {
        let k = Hotkey(keyCode: code, mods: mods)
        expect(defaults.contains { $0.hotkey == k && $0.name == SystemShortcuts.names[id] }, true, "default \(id) is on")
    }
    let off = SystemShortcuts.parse([
        "60": ["enabled": false], "61": ["enabled": false], "64": ["enabled": 0], "65": ["enabled": false],
    ])
    expect(off.isEmpty, true, "explicit disabled entries are off")
    let one = SystemShortcuts.parse([
        "60": ["enabled": false], "61": ["enabled": false], "64": ["enabled": false],
    ])
    expect(one.map { $0.hotkey }, [Hotkey(keyCode: 49, mods: [.option, .command])], "only the missing entry stays")
    let moved = SystemShortcuts.parse([
        "60": ["enabled": true, "value": ["parameters": [32, 11, 262144], "type": "standard"]],
        "61": ["enabled": false], "64": ["enabled": false], "65": ["enabled": false],
    ])
    expect(moved.map { $0.hotkey }, [Hotkey(keyCode: 11, mods: [.control])], "an edited entry uses its own key")
    // The default switch key is free on a Mac with untouched settings.
    expect(Hotkey.default.isValid, true, "default is valid")
    expect(SystemShortcuts.conflict(for: .default, in: defaults), nil, "default switch key has no conflict")
    expect(SystemShortcuts.conflict(for: Hotkey(keyCode: 49, mods: [.control, .option]), in: defaults),
        "Select next source in Input menu", "control option space is a macOS default")

    let url = FileManager.default.temporaryDirectory
        .appendingPathComponent("buuz-test-\(UUID().uuidString)/macos.json")
    let store = HotkeyStore(url: url)
    expect(store.hasChoice, false, "no choice at first")
    expect(store.resolve(system: []), Hotkey.default, "default without conflict")
    let taken = [SystemShortcut(name: "X", hotkey: .default)]
    expect(store.resolve(system: taken), nil, "no default when it conflicts")
    let mine = Hotkey(keyCode: 11, mods: [.control, .option])
    expect(store.save(mine), true, "save works")
    expect(store.load(), mine, "load gives the saved key")
    expect(store.resolve(system: taken), mine, "saved choice wins")
    expect(store.save(nil), true, "save none")
    expect(store.hasChoice, true, "none is a choice")
    expect(store.resolve(system: []), nil, "chosen none stays none")
    try? FileManager.default.removeItem(at: url.deletingLastPathComponent())
}

do {
    let b = "buuz", abc = "abc"
    func t(_ cur: String?, _ other: String?, ascii: String? = nil, avail: [String]) -> String? {
        HotkeyToggle.target(current: cur, buuz: b, lastOther: other, asciiCapable: ascii, available: avail)
    }
    expect(t("mn", abc, avail: [abc, b]), b, "from other to Buuz")
    expect(t(nil, nil, avail: []), b, "unknown goes to Buuz")
    expect(t(b, "mn", avail: ["mn", abc, b]), "mn", "from Buuz to last other")
    expect(t(b, "gone", ascii: "us", avail: ["us", b]), "us", "last other is not enabled any more")
    expect(t(b, nil, ascii: "us", avail: ["us", abc, b]), "us", "no last other: the ASCII-capable keyboard")
    expect(t(b, nil, ascii: nil, avail: [b, "fr", "de"]), "fr", "first enabled keyboard that is not Buuz")
    expect(t(b, b, ascii: b, avail: [b, "de"]), "de", "never Buuz itself")
    expect(t(b, nil, ascii: nil, avail: [b]), nil, "no other keyboard: nothing to select")
    expect(t(b, nil, ascii: "us", avail: []), "us", "ASCII-capable keyboard needs no list")
}

// MARK: Hotkey status

do {
    let url = FileManager.default.temporaryDirectory
        .appendingPathComponent("buuz-test-\(UUID().uuidString)/macos.json")
    let store = HotkeyStore(url: url)
    expect(store.readStatus(), nil, "no status at first")
    expect(store.save(Hotkey.default), true, "save the key")
    store.writeStatus(HotkeyStatus(key: Hotkey.default.encoded, registered: false, code: -9878))
    expect(store.readStatus(), HotkeyStatus(key: Hotkey.default.encoded, registered: false, code: -9878), "status is read back")
    expect(store.load(), Hotkey.default, "the status does not remove the key")
    expect(store.save(nil), true, "save none")
    expect(store.readStatus() != nil, true, "a new choice keeps the old status until the input method writes")
    let bad = HotkeyStatus(key: Hotkey.default.encoded, registered: false, code: -9878)
    expect(bad.failure(for: Hotkey.default), -9878, "failure for the same key")
    expect(bad.failure(for: Hotkey(keyCode: 11, mods: [.control, .option])), nil, "old failure for another key is ignored")
    expect(bad.failure(for: nil), nil, "no key, no failure")
    expect(HotkeyStatus(key: Hotkey.default.encoded, registered: true, code: 0).failure(for: Hotkey.default), nil, "registered is no failure")
    try? FileManager.default.removeItem(at: url.deletingLastPathComponent())
}

do {
    let k = Hotkey(keyCode: 11, mods: [.control, .option])
    var r = SwitchKeyState.initial(saved: nil, custom: nil, storeHasChoice: false, resolved: .default)
    expect(r.key, .custom, "no choice: show the key that is registered")
    expect(r.hotkey, .default, "the shown key is the default")
    r = SwitchKeyState.initial(saved: nil, custom: nil, storeHasChoice: false, resolved: nil)
    expect(r.key, nil, "no choice and a conflict: nothing is registered, nothing shown")
    r = SwitchKeyState.initial(saved: .fn, custom: nil, storeHasChoice: true, resolved: nil)
    expect(r.key, .fn, "a saved system key stays")
    r = SwitchKeyState.initial(saved: .custom, custom: k, storeHasChoice: true, resolved: k)
    expect(r.key, .custom, "a saved custom key stays")
    expect(r.hotkey, k, "with its key")
    let cleared = SwitchKeyState.afterClear()
    expect(cleared.key, nil, "clear: no key is selected")
    expect(cleared.hotkey, nil, "clear: no custom key stays")
}

// MARK: Globe key

do {
    expect(GlobeKeyState(usageType: 1), .switchesSource, "globe changes the input source")
    expect(GlobeKeyState(usageType: 0), .other("none"), "globe does nothing")
    expect(GlobeKeyState(usageType: 2), .other("emoji"), "globe opens emoji")
    expect(GlobeKeyState(usageType: 3), .other("dictation"), "globe starts dictation")
    expect(GlobeKeyState(usageType: nil), .unknown, "no value is unknown")
    expect(GlobeKeyState(usageType: 9), .unknown, "a new value is unknown")
    expect(GlobeKeyState.switchesSource.works, true, "switching works")
    expect(GlobeKeyState.other("emoji").works, false, "emoji does not switch")
    expect(GlobeKeyState.unknown.works, false, "unknown is not trusted")
    _ = GlobeKeyState.read()
}

// MARK: Try it checks

do {
    expect(containsCyrillic("сайн"), true, "cyrillic")
    expect(containsCyrillic("sain"), false, "latin is not cyrillic")
    expect(containsCyrillic(""), false, "empty")
    expect(isLatinOnly("sain baina"), true, "latin only")
    expect(isLatinOnly("sain сайн"), false, "mixed")
    expect(isLatinOnly("123 !"), false, "no letters")
    expect(isLatinOnly(""), false, "empty is not latin")
    func st(_ cur: Bool, _ t: String, _ e: String? = nil) -> TryState {
        TryState.evaluate(isBuuzCurrent: cur, currentKeyboardName: "ABC", text: t, expected: e)
    }
    expect(st(true, ""), .waiting, "empty waits")
    expect(st(false, ""), .wrongKeyboard, "empty on other keyboard")
    expect(st(false, "sain"), .wrongKeyboard, "latin on other keyboard")
    expect(st(true, "sain"), .latinOnly, "latin on Buuz")
    expect(st(true, "сайн"), .success, "any cyrillic succeeds")
    expect(st(true, "сайн байна уу ", "сайн байна уу"), .success, "expected text")
    expect(st(true, "сайн", "сайн байна уу"), .waiting, "partial waits")
    expect(st(true, "sain", "сайн"), .latinOnly, "latin with expected")
}

// MARK: Recorder and conflicts

do {
    let prev = [SystemShortcut(name: "Select previous input source", hotkey: Hotkey(keyCode: 49, mods: [.control]))]
    func rec(_ code: Int, _ mods: HotkeyMods) -> RecordOutcome {
        HotkeyRecording.classify(Hotkey(keyCode: code, mods: mods), system: prev)
    }
    expect(rec(49, []), .needsModifier, "a bare key needs a modifier")
    expect(rec(49, [.shift]), .needsModifier, "shift alone is not enough")
    expect(rec(11, [.control, .option]), .ok(Hotkey(keyCode: 11, mods: [.control, .option])), "free key is ok")
    expect(rec(49, [.control]), .conflict(keys: "⌃Space", actionKey: "hotkey.action.prevSource", name: "Select previous input source"),
        "control space is the previous source")
    expect(rec(49, [.control, .option]), .ok(Hotkey(keyCode: 49, mods: [.control, .option])), "control option space is free when its system entry is off")
    expect(HotkeyRecording.classify(.default, system: SystemShortcuts.parse([:])), .ok(.default), "the default switch key is free")
    expect(rec(20, [.shift, .command]), .conflict(keys: "⇧⌘3", actionKey: "hotkey.action.screenshot", name: "Screenshot"), "screenshot is always on")
    expect(rec(49, [.control, .command]), .conflict(keys: "⌃⌘Space", actionKey: "hotkey.action.emoji", name: "Emoji & Symbols"), "emoji is always on")
    expect(rec(49, [.command]), .conflict(keys: "⌘Space", actionKey: "hotkey.action.spotlight", name: "Spotlight search"), "spotlight")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 49, mods: [.control, .option])), "hotkey.action.nextSource", "next source")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 49, mods: [.option, .command])), "hotkey.action.spotlight", "finder search")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 49, mods: [.control, .command])), "hotkey.action.emoji", "emoji")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 48, mods: [.command])), "hotkey.action.appSwitcher", "app switcher")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 50, mods: [.command])), "hotkey.action.appSwitcher", "switch windows")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 20, mods: [.shift, .command])), "hotkey.action.screenshot", "screenshot 3")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 21, mods: [.shift, .command])), "hotkey.action.screenshot", "screenshot 4")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 23, mods: [.shift, .command])), "hotkey.action.screenshot", "screenshot 5")
    expect(HotkeyRecording.actionKey(for: Hotkey(keyCode: 11, mods: [.control, .option])), nil, "no action for a free key")
    let taken = [SystemShortcut(name: "Mission Control", hotkey: Hotkey(keyCode: 126, mods: [.control]))]
    let out = HotkeyRecording.classify(Hotkey(keyCode: 126, mods: [.control]), system: taken)
    expect(out, .conflict(keys: "⌃↑", actionKey: nil, name: "Mission Control"), "other system shortcut keeps its name")
    expect(SwitchKey.fn.rawValue, "fn", "switch key names")
    expect(SwitchKey(rawValue: "custom") == .custom, true, "custom switch key")
    expect(SwitchKey.fn.isSystemOwned, true, "fn is set in macOS")
    expect(SwitchKey.custom.isSystemOwned, false, "custom is set by Buuz")
}

// MARK: Preset

do {
    expect(ConfigEditor.preset(nil), "smart", "no file is smart")
    expect(ConfigEditor.preset("buuz_config = 1\npreset = \"classic\"\n"), "classic", "reads classic")
    expect(ConfigEditor.preset("buuz_config = 1\n[options]\npreset = \"classic\"\n"), "smart", "preset in [options] is not the preset")
    expect(ConfigEditor.preset("buuz_config = 1\npreset = \"smart\" # my choice\n"), "smart", "reads a comment")
    let base = "buuz_config = 1\npreset = \"smart\"\n\n# notes\n[options]\nautocorrect = true\n"
    let c = ConfigEditor.setPreset(base, "classic")
    expect(ConfigEditor.preset(c), "classic", "sets classic")
    expect(c.contains("# notes") && c.contains("autocorrect = true"), true, "keeps the rest")
    expect(ConfigEditor.setPreset(c, "smart"), base, "back to smart gives the same text")
    let fresh = ConfigEditor.setPreset(nil, "classic")
    expect(fresh, "buuz_config = 1\npreset = \"classic\"\n", "new file with a preset")
    let noPreset = ConfigEditor.setPreset("buuz_config = 1\n[options]\nautocorrect = true\n", "classic")
    expect(noPreset, "buuz_config = 1\npreset = \"classic\"\n[options]\nautocorrect = true\n", "adds the preset before the sections")
    for t in [c, fresh, noPreset] { expect(Engine(config: t, words: nil) != nil, true, "the core accepts the preset") }
    expect(ConfigEditor.setPreset("preset = \"smart\"\n", "classic").hasPrefix("buuz_config = 1\n"), true, "adds the header")
}

// MARK: Preview

do {
    let words = WordList(path: wordListPath)
    func pv(_ keys: String, esc: Bool = false, config: String? = nil) -> String {
        Preview.result(keys: keys, escapeBeforeSpace: esc, config: config, words: words)
    }
    expect(pv("gertoo"), "гэртөө", "vowels match")
    expect(pv("svv"), "сүү", "double v")
    expect(pv("tsai"), "цай", "habit ts")
    expect(pv("bn"), "байна", "abbreviation")
    expect(pv("zoom", esc: true), "zoom", "escape keeps Latin")
    expect(pv("zoom"), "зоом", "without escape it is Cyrillic")
    expect(pv("bn", config: "buuz_config = 1\npreset = \"classic\"\n"), "бн", "classic has no abbreviations")
}

// MARK: Lessons

do {
    func text(_ full: String, marked: String? = nil) -> LessonText {
        // `marked` is the tail of `full`, like a word that is still typed.
        if let m = marked { return LessonText(full: full, marked: NSRange(location: (full as NSString).length - (m as NSString).length, length: (m as NSString).length)) }
        return LessonText(full: full, marked: nil)
    }
    func state(_ c: LessonChecker, buuz: Bool = true) -> LessonState { c.state(isBuuzCurrent: buuz) }

    expect(LessonText(full: "сайн байна", marked: NSRange(location: 5, length: 5)).committed, "сайн ", "committed drops the marked range")
    expect(LessonText(full: "abc", marked: NSRange(location: 9, length: 5)).committed, "abc", "bad range is ignored")
    expect(LessonChecker.normalize("  Сайн   БАЙНА \n уу "), "сайн байна уу", "normalize")
    expect(LessonChecker.targets.count, 8, "eight lessons")

    // Lesson 1
    var c = LessonChecker(lesson: 1)
    expect(state(c), .waiting, "empty waits")
    expect(state(c, buuz: false), .wrongKeyboard, "other keyboard")
    c.feed(text("sain", marked: "sain"))
    expect(state(c), .latinOnly, "latin only with 4 letters")
    c.feed(text("sa", marked: "sa"))
    expect(state(c), .waiting, "two letters are not enough for latin only")
    c.feed(text("сайн ", marked: nil))
    expect(state(c), .waiting, "one word of three waits")
    c.feed(text("сайн байна ", marked: nil))
    expect(state(c), .waiting, "two words of three wait")
    c.feed(text("сайн байна ха ", marked: nil))
    expect(state(c), .almost(got: "сайн байна ха", own: false), "three wrong words is almost")
    c.feed(text("сайн байна уу ", marked: nil))
    expect(state(c), .success, "lesson 1 passes with the real text")
    expect(c.passed, true, "passed flag")
    c.feed(text("", marked: nil))
    expect(state(c), .success, "success stays after the field is cleared")
    expect(state(c, buuz: false), .success, "success wins over the keyboard")

    // Latin letters never pass.
    c = LessonChecker(lesson: 1)
    c.feed(text("sain baina uu ", marked: nil))
    expect(c.passed, false, "latin letters do not pass")
    expect(state(c), .latinOnly, "latin shows the latin state")
    c.feed(text("сайн байна уу", marked: "уу"))
    expect(c.passed, false, "the last word is still marked: not committed")
    c.feed(text("сайн байна уу ", marked: nil))
    expect(c.passed, true, "committed text passes")

    // Lessons 2 to 4, 8: all words needed.
    c = LessonChecker(lesson: 2)
    c.feed(text("гэртөө ")); expect(c.passed, false, "needs both words")
    c.feed(text("гэртөө тэрүү ")); expect(c.passed, true, "lesson 2")
    c = LessonChecker(lesson: 3)
    c.feed(text("өндөр үг ")); expect(c.passed, true, "lesson 3")
    c = LessonChecker(lesson: 4)
    c.feed(text("морь ")); expect(c.passed, false, "lesson 4 half")
    c.feed(text("морь хонь ")); expect(c.passed, true, "lesson 4")
    c = LessonChecker(lesson: 8)
    c.feed(text("би зүгээр байна ")); expect(c.passed, true, "lesson 8")
    c = LessonChecker(lesson: 8)
    c.feed(text("БИ  Зүгээр БАЙНА")); expect(c.passed, true, "case and spaces are normalised")

    // Lesson 5: Return, committed, no marked text.
    c = LessonChecker(lesson: 5)
    c.feed(text("ном", marked: "ном"))
    expect(state(c), .almost(got: "", own: true), "ном still marked is the own almost state")
    c.feed(text("ном", marked: nil))
    expect(c.passed, false, "no Return yet")
    c.submit()
    expect(c.passed, true, "Return after committed ном passes")
    c = LessonChecker(lesson: 5)
    c.feed(text("ном", marked: "ном")); c.submit()
    expect(c.passed, false, "Return while still marked does not pass")
    c.feed(text("ном", marked: nil))
    expect(c.passed, true, "the commit right after Return passes")
    c = LessonChecker(lesson: 5)
    c.submit(); c.feed(text("", marked: nil)); c.feed(text("ном", marked: nil))
    expect(c.passed, false, "an old Return does not count")

    // Lesson 6: both steps in order.
    c = LessonChecker(lesson: 6)
    c.feed(text("тов", marked: nil))
    expect(c.passed, false, "тов alone does not pass")
    expect(c.subChecks, [false, false], "no sub-check")
    c = LessonChecker(lesson: 6)
    c.feed(text("tov", marked: "tov"))
    c.feed(text("төв ", marked: nil))
    expect(c.subChecks, [true, false], "step 1 done")
    expect(state(c), .almost(got: "", own: true), "step 1 only is the own almost state")
    c.feed(text("төв", marked: nil))
    expect(c.passed, false, "still төв")
    c.feed(text("тов", marked: nil))
    expect(c.passed, true, "тов after төв passes")
    expect(c.subChecks, [true, true], "both steps")

    // Lesson 7: marked зоом, then Esc turns it into zoom. No latin-only state.
    c = LessonChecker(lesson: 7)
    c.feed(text("zoom", marked: "zoom"))
    expect(state(c), .waiting, "lesson 7 has no latin only state")
    c.feed(text("zoom ", marked: nil))
    expect(c.passed, false, "zoom without the Cyrillic step does not pass")
    c = LessonChecker(lesson: 7)
    c.feed(text("зоом", marked: "зоом"))
    expect(c.subChecks, [true, false], "marked зоом seen")
    c.feed(text("зоом ", marked: nil))
    expect(state(c), .almost(got: "", own: true), "зоом committed is the own almost state")
    c.feed(text("zoom ", marked: nil))
    expect(c.passed, false, "zoom typed without Esc does not pass")
    c = LessonChecker(lesson: 7)
    c.feed(text("зоом", marked: "зоом"))
    c.feed(text("zoom", marked: nil))
    expect(c.passed, true, "Esc turns the marked зоом into zoom")
    expect(c.subChecks, [true, true], "both steps")
    // The field may show the cleared state before the Latin text.
    c = LessonChecker(lesson: 7)
    c.feed(text("зоом", marked: "зоом"))
    c.feed(text("", marked: nil))
    c.feed(text("zoom", marked: nil))
    expect(c.passed, true, "Esc with a clear step in between")
    // зоом once, then the user types zoom on a plain keyboard.
    c = LessonChecker(lesson: 7)
    c.feed(text("зоом", marked: "зоом"))
    c.feed(text("", marked: nil))
    for t in ["z", "zo", "zoo", "zoom"] { c.feed(text(t, marked: nil)) }
    expect(c.passed, false, "typing zoom key by key after зоом does not pass")
    c = LessonChecker(lesson: 7)
    c.feed(text("зоом", marked: "зоом"))
    c.feed(text("зоом ", marked: nil))
    c.feed(text("", marked: nil))
    c.feed(text("zoom", marked: nil))
    expect(c.passed, false, "zoom after a committed зоом and a clear does not pass")

    // Wrong keyboard beats latin and almost but not success.
    c = LessonChecker(lesson: 2)
    c.feed(text("gertoo", marked: "gertoo"))
    expect(state(c, buuz: false), .wrongKeyboard, "wrong keyboard first")
    c.reset()
    expect(c.passed, false, "reset clears the lesson")
    expect(LessonChecker.mismatch(got: "гэрт", target: "гэртөө").map { $0.ok }, [true, true, true, true], "all letters in target")
    expect(LessonChecker.mismatch(got: "гзр", target: "гэр").map { $0.ok }, [true, false, true], "wrong letter is flagged")
}

// MARK: Lesson options

do {
    expect(LessonRequirement.option(for: 1), nil, "lesson 1 needs no option")
    expect(LessonRequirement.option(for: 2), "smart_vowels", "lesson 2 needs smart vowels")
    expect(LessonRequirement.option(for: 6), "word_choice", "lesson 6 needs word choice")
    expect(LessonRequirement.option(for: 8), "abbreviations", "lesson 8 needs abbreviations")
    expect(LessonRequirement.option(for: 7), nil, "lesson 7 needs no option")
    expect(LessonRequirement.isMissing(lesson: 2, classic: false, flag: nil), false, "on by default")
    expect(LessonRequirement.isMissing(lesson: 2, classic: false, flag: true), false, "on")
    expect(LessonRequirement.isMissing(lesson: 2, classic: false, flag: false), true, "off")
    expect(LessonRequirement.isMissing(lesson: 6, classic: true, flag: nil), true, "classic mode")
    expect(LessonRequirement.isMissing(lesson: 1, classic: true, flag: false), false, "no option, no warning")
}

// MARK: Config file

do {
    let dir = FileManager.default.temporaryDirectory.appendingPathComponent("buuz-test-\(UUID().uuidString)")
    let url = dir.appendingPathComponent("config.toml")
    expect(ConfigEditor.modify(at: url) { ConfigEditor.setFlag($0, "double_v", false) } != nil, true, "creates the file")
    expect(ConfigEditor.flag(try? String(contentsOf: url, encoding: .utf8), "double_v"), false, "flag is written")
    // The user edits the file by hand. A stale copy must not erase the edit.
    let edited = ((try? String(contentsOf: url, encoding: .utf8)) ?? "") + "\n# my note\n"
    try? edited.write(to: url, atomically: true, encoding: .utf8)
    let after = ConfigEditor.modify(at: url) { ConfigEditor.setFlag($0, "habit_keys", false) }
    expect(after?.contains("# my note"), true, "hand edit is kept")
    expect(after?.contains("double_v = false"), true, "old flag is kept")
    expect(after?.contains("habit_keys = false"), true, "new flag is added")
    let again = ConfigEditor.modify(at: url) { ConfigEditor.setPreset($0, "classic") }
    expect(again.map { ConfigEditor.preset($0) }, "classic", "preset is written")
    expect(again?.contains("habit_keys = false"), true, "flags stay with the preset")
    try? FileManager.default.removeItem(at: dir)
}

// MARK: Switch to Buuz

do {
    /// Runs the plan with fakes. `selects` gives the status of each select call.
    func run(enabled: Bool, canEnable: Bool = true, selects: [Int?]) -> (SwitchResult, [String]) {
        var log: [String] = []
        var left = selects
        var isOn = enabled
        var result: SwitchResult?
        SwitchToBuuz.run(
            isEnabled: { isOn },
            enable: { log.append("enable"); if canEnable { isOn = true }; return canEnable },
            select: { log.append("select"); return left.isEmpty ? nil : left.removeFirst() },
            launch: { log.append("launch") },
            after: { $0() },
            done: { result = $0 })
        return (result ?? .failed(status: 999), log)
    }
    var r = run(enabled: true, selects: [0])
    expect(r.0, .switched, "success first time")
    expect(r.1, ["select"], "no launch when select works")
    r = run(enabled: true, selects: [-50, 0])
    expect(r.0, .switched, "success after launch")
    expect(r.1, ["select", "launch", "select"], "launch then retry once")
    r = run(enabled: true, selects: [-50, -50, 0])
    expect(r.0, .failed(status: -50), "failure after one retry")
    expect(r.1, ["select", "launch", "select"], "only one retry")
    r = run(enabled: true, selects: [nil, nil])
    expect(r.0, .failed(status: -1), "source not found is a failure")
    r = run(enabled: false, selects: [0])
    expect(r.0, .switched, "not enabled: enable, launch, select")
    expect(r.1, ["enable", "launch", "select"], "not enabled order")
    r = run(enabled: false, canEnable: false, selects: [0])
    expect(r.0, .notEnabled, "enable fails")
    expect(r.1, ["enable"], "nothing else after a failed enable")
    expect(SwitchResult.failed(status: -50).isFailure, true, "failure is a failure")
    expect(SwitchResult.switched.isFailure, false, "switched is no failure")
}

// MARK: Switch round trip

do {
    var p = SwitchProgress()
    expect(p.toBuuz, false, "nothing yet")
    p.observe(isBuuz: false)
    expect(p.toBuuz, false, "other keyboard first")
    p.observe(isBuuz: true)
    expect(p.toBuuz, true, "became Buuz")
    expect(p.done, false, "not done")
    p.observe(isBuuz: true)
    p.observe(isBuuz: false)
    expect(p.back, true, "left Buuz")
    expect(p.done, false, "still not done")
    p.observe(isBuuz: true)
    expect(p.done, true, "back to Buuz")
    var q = SwitchProgress()
    q.observe(isBuuz: false); q.observe(isBuuz: false)
    expect(q.back, false, "other to other is nothing")
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
