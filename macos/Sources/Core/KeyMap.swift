// SPDX-License-Identifier: MIT OR Apache-2.0
// Turns a macOS key event into a Buuz key kind. Pure functions, no AppKit.
// The rules match `convert_with` in core/src/composer.rs.

import Foundation

struct MappedKey: Equatable {
    var kind: UInt32
    var ch: UInt32 = 0
}

/// The modifier keys that make a key a shortcut. Shift and Caps Lock are not
/// here: they change the character, and the character is already in the event.
struct Mods {
    var command = false
    var control = false
    var option = false

    var isShortcut: Bool { command || control || option }
}

enum KeyMap {
    static let char = UInt32(BUUZ_KEY_CHAR)
    static let backspace = UInt32(BUUZ_KEY_BACKSPACE)
    static let delete = UInt32(BUUZ_KEY_DELETE)
    static let left = UInt32(BUUZ_KEY_LEFT)
    static let right = UInt32(BUUZ_KEY_RIGHT)
    static let home = UInt32(BUUZ_KEY_HOME)
    static let end = UInt32(BUUZ_KEY_END)
    static let space = UInt32(BUUZ_KEY_SPACE)
    static let escape = UInt32(BUUZ_KEY_ESCAPE)
    static let symbol = UInt32(BUUZ_KEY_SYMBOL)
    static let tab = UInt32(BUUZ_KEY_TAB)
    static let other = UInt32(BUUZ_KEY_OTHER)

    /// Map one key down. `keyCode` is the hardware key code and `characters`
    /// the text the key gives with Shift and Caps Lock applied.
    static func map(keyCode: UInt16, characters: String?) -> MappedKey {
        switch keyCode {
        case 51: return MappedKey(kind: backspace)
        case 117: return MappedKey(kind: delete)
        case 123: return MappedKey(kind: left)
        case 124: return MappedKey(kind: right)
        case 115: return MappedKey(kind: home)
        case 119: return MappedKey(kind: end)
        case 49: return MappedKey(kind: space, ch: 32)
        case 53: return MappedKey(kind: escape)
        case 48: return MappedKey(kind: tab)
        default: break
        }
        guard let text = characters else { return MappedKey(kind: other) }
        var scalars = text.unicodeScalars.makeIterator()
        guard let first = scalars.next(), scalars.next() == nil else {
            return MappedKey(kind: other)
        }
        return map(scalar: first)
    }

    static func map(scalar c: Unicode.Scalar) -> MappedKey {
        let v = c.value
        if isInputChar(c) { return MappedKey(kind: char, ch: v) }
        if v == 32 { return MappedKey(kind: space, ch: 32) }
        // Arrow keys, F keys and the like come as private use characters.
        if (0xF700...0xF8FF).contains(v) { return MappedKey(kind: other) }
        if c.properties.isAlphabetic || c.properties.generalCategory == .control
            || c.properties.isWhitespace {
            return MappedKey(kind: other)
        }
        return MappedKey(kind: symbol, ch: v)
    }

    /// `is_input_char` in core/src/composer.rs.
    static func isInputChar(_ c: Unicode.Scalar) -> Bool {
        switch c {
        case "'", "\"", "ö", "ü", "Ö", "Ü": return true
        case "a"..."z", "A"..."Z": return true
        default: return false
        }
    }
}
