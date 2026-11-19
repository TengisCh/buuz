// SPDX-License-Identifier: MIT OR Apache-2.0
// The Buuz switch key: a key code and modifiers. Pure logic, no Carbon calls.

import Foundation

struct HotkeyMods: OptionSet, Hashable {
    let rawValue: Int
    static let control = HotkeyMods(rawValue: 1)
    static let option = HotkeyMods(rawValue: 2)
    static let shift = HotkeyMods(rawValue: 4)
    static let command = HotkeyMods(rawValue: 8)

    /// The Carbon modifier mask for `RegisterEventHotKey`.
    var carbon: UInt32 {
        var m: UInt32 = 0
        if contains(.command) { m |= 256 }
        if contains(.shift) { m |= 512 }
        if contains(.option) { m |= 2048 }
        if contains(.control) { m |= 4096 }
        return m
    }
}

extension HotkeyMods {
    /// From the modifier bits of `NSEvent.ModifierFlags` (also used by the
    /// symbolic hotkeys plist).
    init(cocoaFlags f: Int) {
        var m = HotkeyMods()
        if f & 131_072 != 0 { m.insert(.shift) }
        if f & 262_144 != 0 { m.insert(.control) }
        if f & 524_288 != 0 { m.insert(.option) }
        if f & 1_048_576 != 0 { m.insert(.command) }
        self = m
    }
}

struct Hotkey: Equatable, Hashable {
    var keyCode: Int
    var mods: HotkeyMods

    static let space = 49
    /// The default switch key, used only when it has no conflict. It is
    /// Control+Option+Return. Control+Option+Space is a macOS default (the
    /// "next input source" shortcut), so most Macs would refuse it. Control+
    /// Option+Return has no macOS default (checked in the tests).
    static let `default` = Hotkey(keyCode: 36, mods: [.control, .option])

    /// Key codes of letters, digits and punctuation: the keys that print a
    /// character. Option (and Option+Shift) with one of them types another
    /// character in many layouts, so such a hotkey would break typing.
    static let printingKeyCodes: Set<Int> = [
        0, 11, 8, 2, 14, 3, 5, 4, 34, 38, 40, 37, 46, 45, 31, 35, 12, 15, 1, 17, 32, 9, 13, 7, 16, 6,
        18, 19, 20, 21, 23, 22, 26, 28, 25, 29, 27, 24, 33, 30, 41, 39, 42, 43, 47, 44, 50,
    ]

    /// A hotkey needs Control, Option or Command. Bare keys and Shift-only
    /// keys would break normal typing. Option or Option+Shift with a printing
    /// key is refused too, unless Control or Command is also held.
    var isValid: Bool {
        guard keyCode >= 0, !mods.isDisjoint(with: [.control, .option, .command]) else { return false }
        if mods.isDisjoint(with: [.control, .command]), Hotkey.printingKeyCodes.contains(keyCode) { return false }
        return true
    }

    private static let keyNames: [Int: String] = [
        49: "Space", 36: "Return", 48: "Tab", 51: "Delete", 53: "Esc", 76: "Enter",
        123: "←", 124: "→", 125: "↓", 126: "↑",
        122: "F1", 120: "F2", 99: "F3", 118: "F4", 96: "F5", 97: "F6", 98: "F7",
        100: "F8", 101: "F9", 109: "F10", 103: "F11", 111: "F12",
        0: "A", 11: "B", 8: "C", 2: "D", 14: "E", 3: "F", 5: "G", 4: "H", 34: "I",
        38: "J", 40: "K", 37: "L", 46: "M", 45: "N", 31: "O", 35: "P", 12: "Q",
        15: "R", 1: "S", 17: "T", 32: "U", 9: "V", 13: "W", 7: "X", 16: "Y", 6: "Z",
        18: "1", 19: "2", 20: "3", 21: "4", 23: "5", 22: "6", 26: "7", 28: "8",
        25: "9", 29: "0", 27: "-", 24: "=", 33: "[", 30: "]", 41: ";", 39: "'",
        42: "\\", 43: ",", 47: ".", 44: "/", 50: "`",
    ]

    /// For example "⌃⌥Space". Modifiers go in the macOS order.
    var display: String {
        var s = ""
        if mods.contains(.control) { s += "⌃" }
        if mods.contains(.option) { s += "⌥" }
        if mods.contains(.shift) { s += "⇧" }
        if mods.contains(.command) { s += "⌘" }
        return s + (Hotkey.keyNames[keyCode] ?? "Key \(keyCode)")
    }

    /// A string to save, for example "49:control,option".
    var encoded: String {
        var names: [String] = []
        if mods.contains(.control) { names.append("control") }
        if mods.contains(.option) { names.append("option") }
        if mods.contains(.shift) { names.append("shift") }
        if mods.contains(.command) { names.append("command") }
        return "\(keyCode):" + names.joined(separator: ",")
    }

    /// Reads `encoded`. Gives nil for bad text or an invalid hotkey.
    static func decode(_ text: String) -> Hotkey? {
        let parts = text.split(separator: ":", maxSplits: 1, omittingEmptySubsequences: false)
        guard parts.count == 2, let code = Int(parts[0]) else { return nil }
        var mods = HotkeyMods()
        for name in parts[1].split(separator: ",") {
            switch name {
            case "control": mods.insert(.control)
            case "option": mods.insert(.option)
            case "shift": mods.insert(.shift)
            case "command": mods.insert(.command)
            default: return nil
            }
        }
        let h = Hotkey(keyCode: code, mods: mods)
        return h.isValid ? h : nil
    }
}

/// A macOS shortcut that Buuz must not take.
struct SystemShortcut: Equatable {
    var name: String
    var hotkey: Hotkey
}

enum SystemShortcuts {
    /// The symbolic hotkey ids we care about.
    static let names: [Int: String] = [
        60: "Select previous input source",
        61: "Select next source in Input menu",
        64: "Spotlight search",
        65: "Finder search window",
        32: "Mission Control",
        33: "Application windows",
        34: "Mission Control",
        35: "Application windows",
        36: "Show Desktop",
    ]

    /// Shortcuts that apps and macOS always use.
    static let reserved: [SystemShortcut] = {
        let c: [(String, Int)] = [
            ("Spotlight search", 49), ("Switch apps", 48), ("Quit", 12), ("Close window", 13),
            ("Copy", 8), ("Paste", 9), ("Cut", 7), ("Undo", 6), ("Select all", 0),
            ("Save", 1), ("Open", 31), ("New", 45), ("Print", 35), ("Find", 3),
            ("Hide app", 4), ("Minimise window", 46), ("Settings", 43),
        ]
        return c.map { SystemShortcut(name: $0.0, hotkey: Hotkey(keyCode: $0.1, mods: [.command])) }
    }()

    /// macOS leaves a default shortcut out of the plist until the user edits
    /// it, but it still applies. A missing entry means "on, default key".
    static let defaults: [Int: Hotkey] = [
        60: Hotkey(keyCode: 49, mods: [.control]),
        61: Hotkey(keyCode: 49, mods: [.control, .option]),
        64: Hotkey(keyCode: 49, mods: [.command]),
        65: Hotkey(keyCode: 49, mods: [.option, .command]),
    ]

    /// Reads the `AppleSymbolicHotKeys` dictionary. An entry with `enabled`
    /// false is off. A missing entry with a default key (60, 61, 64, 65) is
    /// on. Other entries without a key are skipped.
    static func parse(_ symbolic: [String: Any]) -> [SystemShortcut] {
        var found: [SystemShortcut] = []
        for (id, name) in names.sorted(by: { $0.key < $1.key }) {
            if symbolic[String(id)] == nil {
                if let key = defaults[id] { found.append(SystemShortcut(name: name, hotkey: key)) }
                continue
            }
            guard let entry = symbolic[String(id)] as? [String: Any],
                (entry["enabled"] as? NSNumber)?.boolValue ?? (entry["enabled"] as? Bool) ?? false,
                let value = entry["value"] as? [String: Any],
                let params = value["parameters"] as? [Any], params.count >= 3,
                let code = (params[1] as? NSNumber)?.intValue,
                let flags = (params[2] as? NSNumber)?.intValue,
                code >= 0, code < 65535
            else { continue }
            found.append(SystemShortcut(name: name, hotkey: Hotkey(keyCode: code, mods: HotkeyMods(cocoaFlags: flags))))
        }
        return found
    }

    /// The shortcuts that are on in System Settings now.
    static func read() -> [SystemShortcut] {
        let value = CFPreferencesCopyAppValue(
            "AppleSymbolicHotKeys" as CFString, "com.apple.symbolichotkeys" as CFString)
        return parse((value as? [String: Any]) ?? [:])
    }

    /// The name of the shortcut that uses `hotkey`, or nil. Checks `system`
    /// first, then the reserved list.
    static func conflict(for hotkey: Hotkey, in system: [SystemShortcut] = read()) -> String? {
        (system + reserved).first { $0.hotkey == hotkey }?.name
    }
}

/// Where the settings app and the input method keep the hotkey. A separate
/// file, because the portable config rejects unknown top-level keys.
final class HotkeyStore {
    let url: URL
    init(url: URL = HotkeyStore.defaultURL) { self.url = url }

    static var defaultURL: URL {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("Buuz", isDirectory: true)
            .appendingPathComponent("macos.json")
    }

    /// nil when the user never chose. An empty text means "no hotkey".
    private func readChoice() -> String? {
        guard let data = try? Data(contentsOf: url),
            let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return nil }
        return object["hotkey"] as? String
    }

    /// True when the user chose a hotkey, or chose none.
    var hasChoice: Bool { readChoice() != nil }

    /// The saved hotkey. Gives nil for none or for a bad value.
    func load() -> Hotkey? { readChoice().flatMap(Hotkey.decode) }

    /// Saves `hotkey`. nil saves "no hotkey".
    @discardableResult
    func save(_ hotkey: Hotkey?) -> Bool {
        var object: [String: Any] = [:]
        if let data = try? Data(contentsOf: url),
            let old = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        { object = old }
        object["hotkey"] = hotkey?.encoded ?? ""
        guard let data = try? JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
        else { return false }
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        return (try? data.write(to: url, options: .atomic)) != nil
    }

    /// The hotkey to use now. A saved choice wins. With no choice, the
    /// default key is used only when it has no conflict.
    func resolve(system: [SystemShortcut] = SystemShortcuts.read()) -> Hotkey? {
        if hasChoice { return load() }
        return SystemShortcuts.conflict(for: .default, in: system) == nil ? .default : nil
    }
}

/// What the input method says about the last `RegisterEventHotKey` call.
struct HotkeyStatus: Equatable {
    /// `Hotkey.encoded` of the key that the input method tried to register.
    var key: String
    var registered: Bool
    /// The `OSStatus` of the first failing call. 0 when it worked.
    var code: Int

    /// The failure code when the status is about `hotkey` and it failed.
    func failure(for hotkey: Hotkey?) -> Int? {
        guard let h = hotkey, h.encoded == key, !registered else { return nil }
        return code
    }
}

extension HotkeyStore {
    /// Written by the input method after it registers the key. A separate
    /// key in the same file, so `save` keeps it.
    func readStatus() -> HotkeyStatus? {
        guard let data = try? Data(contentsOf: url),
            let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
            let s = object["status"] as? [String: Any],
            let key = s["key"] as? String, let ok = s["registered"] as? Bool
        else { return nil }
        return HotkeyStatus(key: key, registered: ok, code: (s["code"] as? Int) ?? 0)
    }

    @discardableResult
    func writeStatus(_ status: HotkeyStatus) -> Bool {
        var object: [String: Any] = [:]
        if let data = try? Data(contentsOf: url),
            let old = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        { object = old }
        object["status"] = ["key": status.key, "registered": status.registered, "code": status.code] as [String: Any]
        guard let data = try? JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
        else { return false }
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        return (try? data.write(to: url, options: .atomic)) != nil
    }
}

enum HotkeyToggle {
    static let fallback = "com.apple.keylayout.ABC"
    static let notification = "com.thetengis.buuz.hotkeyChanged"
    /// Posted by the input method after it tried to register the key.
    static let statusNotification = "com.thetengis.buuz.hotkeyStatus"

    /// The input source to select when the switch key fires. From another
    /// keyboard, go to Buuz. From Buuz, go to the first of: the last other
    /// keyboard (if still enabled), the ASCII-capable keyboard of the user,
    /// the first enabled keyboard that is not Buuz. Nil when there is none.
    /// `available` lists the enabled keyboards that can be selected.
    static func target(
        current: String?, buuz: String, lastOther: String?, asciiCapable: String?, available: [String]
    ) -> String? {
        guard current == buuz else { return buuz }
        if let other = lastOther, other != buuz, available.contains(other) { return other }
        if let ascii = asciiCapable, ascii != buuz { return ascii }
        return available.first { $0 != buuz }
    }
}
