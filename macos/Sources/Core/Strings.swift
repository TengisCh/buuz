// SPDX-License-Identifier: MIT OR Apache-2.0
// UI strings for the onboarding and settings app. Mongolian is the default and
// English is the second language. The app does not follow the system language.

import Foundation

enum Language: String {
    case mn, en
}

final class Strings {
    private let tables: [Language: [String: String]]
    var language: Language

    /// `directory` holds `strings.mn.json` and `strings.en.json`.
    init?(directory: URL, language: Language = .mn) {
        var loaded: [Language: [String: String]] = [:]
        for lang in [Language.mn, Language.en] {
            let url = directory.appendingPathComponent("strings.\(lang.rawValue).json")
            guard let data = try? Data(contentsOf: url),
                let table = try? JSONDecoder().decode([String: String].self, from: data)
            else { return nil }
            loaded[lang] = table
        }
        tables = loaded
        self.language = language
    }

    var keys: Set<String> { Set(tables[.en]?.keys ?? [:].keys) }

    func raw(_ key: String, _ lang: Language) -> String? { tables[lang]?[key] }

    /// The text for `key`. Falls back to English, then to the key. Each
    /// `{name}` is replaced from `values`.
    func text(_ key: String, _ values: [String: String] = [:]) -> String {
        var s = tables[language]?[key] ?? tables[.en]?[key] ?? key
        for (name, value) in values { s = s.replacingOccurrences(of: "{\(name)}", with: value) }
        return s
    }
}

/// The `{name}` placeholders of a string.
func placeholders(_ s: String) -> Set<String> {
    var found = Set<String>()
    var rest = s[...]
    while let open = rest.firstIndex(of: "{"), let close = rest[open...].firstIndex(of: "}") {
        found.insert(String(rest[rest.index(after: open)..<close]))
        rest = rest[rest.index(after: close)...]
    }
    return found
}
