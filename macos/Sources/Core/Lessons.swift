// SPDX-License-Identifier: MIT OR Apache-2.0
// Checks the lessons of the tutorial from the real text in the lesson field.
// Pure logic, no AppKit. The checks use only Cyrillic output, never the Latin
// keys. The text is never saved or logged (design/macos-app/spec.md 7.3, 8.3).

import Foundation

/// What the lesson field holds: all text, and the range that is still marked.
struct LessonText: Equatable {
    var full: String
    var marked: NSRange?

    private var validMarked: NSRange? {
        guard let r = marked, r.location >= 0, r.length >= 0,
            r.location + r.length <= (full as NSString).length
        else { return nil }
        return r
    }

    /// The text without the marked range.
    var committed: String {
        guard let r = validMarked else { return full }
        return (full as NSString).replacingCharacters(in: r, with: "")
    }

    var markedText: String? {
        validMarked.map { (full as NSString).substring(with: $0) }
    }

    var hasMarked: Bool { validMarked.map { $0.length > 0 } ?? false }
}

enum LessonState: Equatable {
    case success, wrongKeyboard, latinOnly, waiting
    /// `own` means the lesson has its own `lessonN.close` hint.
    case almost(got: String, own: Bool)
}

/// Which lesson needs which typing option. A lesson with a needed option
/// cannot pass in classic mode or with that option off.
enum LessonRequirement {
    static func option(for lesson: Int) -> String? {
        switch lesson {
        case 2: return "smart_vowels"
        case 6: return "word_choice"
        case 8: return "abbreviations"
        default: return nil
        }
    }

    /// True when the lesson needs an option that is off. `flag` is the value
    /// in the config file (nil: not set, so on in smart mode).
    static func isMissing(lesson: Int, classic: Bool, flag: Bool?) -> Bool {
        guard option(for: lesson) != nil else { return false }
        return classic || !(flag ?? true)
    }
}

struct LessonChecker {
    let lesson: Int
    private(set) var passed = false
    private var last = LessonText(full: "", marked: nil)
    private var returnPending = false
    private var step1 = false
    private var step2 = false
    /// The committed text when the marked зоом was last seen. Lesson 7 passes
    /// only when Esc turns that marked text into zoom in place.
    private var zoomBase: String?

    init(lesson: Int) { self.lesson = lesson }

    /// The Cyrillic text of each lesson, for the word count.
    static let targets = [
        "сайн байна уу", "эрүүл мэнд", "өндөр үг", "морь хонь",
        "ном", "тов", "zoom", "би зүгээр байна",
    ]

    private static let required: [Int: [String]] = [
        1: ["сайн байна уу"], 2: ["эрүүл", "мэнд"], 3: ["өндөр", "үг"],
        4: ["морь", "хонь"], 8: ["би зүгээр байна"],
    ]

    /// Lower case, trimmed, runs of spaces made into one.
    static func normalize(_ s: String) -> String {
        s.lowercased().split(whereSeparator: { $0.isWhitespace }).joined(separator: " ")
    }

    /// The sub-checks of lessons 6 and 7. Empty for the other lessons.
    var subChecks: [Bool] { (lesson == 6 || lesson == 7) ? [step1, step2] : [] }

    mutating func reset() { self = LessonChecker(lesson: lesson) }

    /// The field text changed.
    mutating func feed(_ text: LessonText) {
        last = text
        evaluate()
        returnPending = false
    }

    /// Return was pressed in the field (after Buuz committed the word).
    mutating func submit() {
        returnPending = true
        evaluate()
    }

    private mutating func evaluate() {
        if passed { return }
        let committed = LessonChecker.normalize(last.committed)
        let full = LessonChecker.normalize(last.full)
        switch lesson {
        case 5:
            if returnPending && committed.contains("ном") && !last.hasMarked { passed = true }
        case 6:
            if step1 && !step2 && full.contains("тов") && !full.contains("төв") { step2 = true }
            if committed.contains("төв") { step1 = true }
            passed = step1 && step2
        case 7:
            if let m = last.markedText, LessonChecker.normalize(m) == "зоом" {
                step1 = true
                zoomBase = last.committed
            } else if let base = zoomBase {
                if last.hasMarked {
                    zoomBase = nil
                } else if last.full == base {
                    // The field cleared the marked text. The Latin text follows.
                } else if LessonChecker.normalize(last.full) == LessonChecker.normalize(base + "zoom") {
                    step2 = true
                } else {
                    zoomBase = nil
                }
            }
            passed = step1 && step2
        default:
            if let words = LessonChecker.required[lesson], words.allSatisfy({ committed.contains($0) }) {
                passed = true
            }
        }
    }

    func state(isBuuzCurrent: Bool) -> LessonState {
        if passed { return .success }
        if !isBuuzCurrent { return .wrongKeyboard }
        let full = last.full
        if lesson != 7, !containsCyrillic(full), full.unicodeScalars.filter({ $0.isASCII && CharacterSet.letters.contains($0) }).count >= 3 {
            return .latinOnly
        }
        let committed = LessonChecker.normalize(last.committed)
        switch lesson {
        case 5 where last.hasMarked && LessonChecker.normalize(full).contains("ном"):
            return .almost(got: "", own: true)
        case 6 where step1:
            return .almost(got: "", own: true)
        case 7 where committed.contains("зоом"):
            return .almost(got: "", own: true)
        default: break
        }
        let shown = last.committed.split(whereSeparator: { $0.isWhitespace }).joined(separator: " ")
        let targetWords = LessonChecker.targets[min(max(lesson, 1), 8) - 1].split(separator: " ").count
        if containsCyrillic(shown), shown.split(separator: " ").count >= targetWords {
            return .almost(got: shown, own: false)
        }
        return .waiting
    }

    /// Each letter of `got`, and whether the target has it. The UI shows the
    /// wrong letters in red.
    static func mismatch(got: String, target: String) -> [(char: Character, ok: Bool)] {
        let t = Set(target.lowercased())
        return got.map { c in (c, c == " " || t.contains(Character(c.lowercased()))) }
    }
}

/// Tutorial step 5: Buuz, then another keyboard, then Buuz again.
struct SwitchProgress {
    private(set) var toBuuz = false
    private(set) var back = false
    private(set) var done = false

    mutating func observe(isBuuz: Bool) {
        if !toBuuz {
            if isBuuz { toBuuz = true }
        } else if !back {
            if !isBuuz { back = true }
        } else if isBuuz {
            done = true
        }
    }
}
