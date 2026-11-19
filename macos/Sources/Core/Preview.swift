// SPDX-License-Identifier: MIT OR Apache-2.0
// Runs sample keys through the real engine, for the Typing preview card. It
// uses only the fixed sample keys of the UI, never what the user types.

import Foundation

enum Preview {
    /// The Cyrillic result for `keys`, typed as letters. With
    /// `escapeBeforeSpace`, Esc is pressed before the final Space.
    static func result(keys: String, escapeBeforeSpace: Bool, config: String?, words: WordList?) -> String {
        let engine = Engine.lenient(config: config, words: words)
        var out = ""
        for u in keys.unicodeScalars {
            let r = engine.process(kind: KeyMap.char, ch: u.value)
            if let c = r.commit { out += c }
        }
        if escapeBeforeSpace {
            let r = engine.process(kind: KeyMap.escape)
            if let c = r.commit { out += c }
        }
        let r = engine.process(kind: KeyMap.space, ch: 32)
        if let c = r.commit { out += c }
        if let rest = engine.commit() { out += rest }
        return out.trimmingCharacters(in: .whitespaces)
    }
}
