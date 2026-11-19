// SPDX-License-Identifier: MIT OR Apache-2.0
// Edits one switch in the portable config text and keeps everything else:
// comments, abbreviations, keys and the order of the lines. The settings app
// uses it, so the file stays good for the web converter and the input method.

import Foundation

enum ConfigEditor {
    static let header = "buuz_config = 1"

    /// The value of `key` in `[options]`, or nil when the file does not set it.
    static func flag(_ text: String?, _ key: String) -> Bool? {
        guard let text = text else { return nil }
        var inOptions = false
        for raw in text.components(separatedBy: "\n") {
            let line = raw.trimmingCharacters(in: .whitespaces)
            if line.hasPrefix("[") { inOptions = (sectionName(line) == "options"); continue }
            guard inOptions, let (k, v) = pair(line), k == key else { continue }
            return v == "true" ? true : (v == "false" ? false : nil)
        }
        return nil
    }

    /// The config text with `key` set to `value`.
    static func setFlag(_ text: String?, _ key: String, _ value: Bool) -> String {
        var lines = (text ?? "").components(separatedBy: "\n")
        if lines.last == "" { lines.removeLast() }
        if !lines.contains(where: { pair($0.trimmingCharacters(in: .whitespaces))?.0 == "buuz_config" }) {
            lines.insert(header, at: 0)
        }
        let newLine = "\(key) = \(value)"

        var optionsStart: Int?
        var optionsEnd = lines.count
        for (i, raw) in lines.enumerated() {
            let line = raw.trimmingCharacters(in: .whitespaces)
            guard line.hasPrefix("[") else { continue }
            if sectionName(line) == "options" { optionsStart = i }
            else if let s = optionsStart, i > s { optionsEnd = i; break }
        }

        if let start = optionsStart {
            for i in (start + 1)..<optionsEnd {
                if pair(lines[i].trimmingCharacters(in: .whitespaces))?.0 == key {
                    lines[i] = newLine
                    return lines.joined(separator: "\n") + "\n"
                }
            }
            lines.insert(newLine, at: start + 1)
        } else {
            lines.append("")
            lines.append("[options]")
            lines.append(newLine)
        }
        return lines.joined(separator: "\n") + "\n"
    }

    private static func sectionName(_ line: String) -> String {
        line.trimmingCharacters(in: CharacterSet(charactersIn: "[] \t"))
    }

    /// `key = value` without a trailing comment. Nil for other lines.
    private static func pair(_ line: String) -> (String, String)? {
        if line.isEmpty || line.hasPrefix("#") || line.hasPrefix("[") { return nil }
        guard let eq = line.firstIndex(of: "=") else { return nil }
        let key = line[..<eq].trimmingCharacters(in: .whitespaces)
        var value = line[line.index(after: eq)...].trimmingCharacters(in: .whitespaces)
        if let hash = value.firstIndex(of: "#"), !value.hasPrefix("\"") {
            value = value[..<hash].trimmingCharacters(in: .whitespaces)
        }
        return (key, value)
    }
}
