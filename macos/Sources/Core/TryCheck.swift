// SPDX-License-Identifier: MIT OR Apache-2.0
// Live checks for the "Try it" screen. Pure functions: the UI gives the state.

import Foundation

func containsCyrillic(_ text: String) -> Bool {
    text.unicodeScalars.contains { (0x0400...0x04FF).contains($0.value) }
}

/// True when the text has Latin letters and no letter of another script.
/// Digits, spaces and punctuation do not count.
func isLatinOnly(_ text: String) -> Bool {
    var sawLatin = false
    for u in text.unicodeScalars where CharacterSet.letters.contains(u) {
        guard u.isASCII else { return false }
        sawLatin = true
    }
    return sawLatin
}

enum TryState: Equatable {
    case waiting, wrongKeyboard, latinOnly, success

    /// `expected` is the Cyrillic text the user should get. With nil, any
    /// Cyrillic text is a success. `currentKeyboardName` is for the hint text.
    static func evaluate(
        isBuuzCurrent: Bool, currentKeyboardName: String?, text: String, expected: String?
    ) -> TryState {
        let t = text.trimmingCharacters(in: .whitespacesAndNewlines)
        if !t.isEmpty {
            if let e = expected, !e.isEmpty {
                if t.contains(e) { return .success }
            } else if containsCyrillic(t) {
                return .success
            }
        }
        if !isBuuzCurrent { return .wrongKeyboard }
        if isLatinOnly(t) { return .latinOnly }
        return .waiting
    }
}
