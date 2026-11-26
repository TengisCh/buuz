// SPDX-License-Identifier: MIT OR Apache-2.0
// The tutorial: its steps, the setup checklist and the home greeting. Pure
// logic, no AppKit (design/macos-app/spec.md section 7).

import Foundation

enum TourStep: String, CaseIterable {
    case welcome, privacy, add, key, switchTo
    case lesson1, lesson2, lesson3, lesson4, lesson5, lesson6, lesson7, lesson8
    case done

    /// 1 to 8 for a lesson step, else nil.
    var lessonNumber: Int? {
        guard rawValue.hasPrefix("lesson") else { return nil }
        return Int(rawValue.dropFirst(6))
    }

    /// 1 to 5 for a setup step, else nil.
    var setupNumber: Int? {
        switch self {
        case .welcome: return 1
        case .privacy: return 2
        case .add: return 3
        case .key: return 4
        case .switchTo: return 5
        default: return nil
        }
    }

    /// Every step has a Skip button except Welcome and Done.
    var canSkip: Bool { self != .welcome && self != .done }

    static func lesson(_ n: Int) -> TourStep? { TourStep(rawValue: "lesson\(n)") }
}

/// Moves between steps. The "add" step is skipped when Buuz is already in the
/// keyboards (detect, do not ask). Skip is the same move as next: it does not
/// mark the step as passed.
struct TourFlow: Equatable {
    var step: TourStep = .welcome

    private var all: [TourStep] { TourStep.allCases }

    func next(enabled: Bool) -> TourStep {
        guard let i = all.firstIndex(of: step), i + 1 < all.count else { return step }
        // Buuz was removed after the add step: Next cannot skip the add step.
        if !enabled, let a = all.firstIndex(of: .add), i > a, step != .done { return .add }
        let n = all[i + 1]
        return (n == .add && enabled) ? .key : n
    }

    func back(enabled: Bool) -> TourStep {
        guard let i = all.firstIndex(of: step), i > 0 else { return step }
        let p = all[i - 1]
        return (p == .add && enabled) ? .privacy : p
    }

    /// True when the rail shows `s` as done while `current` is open. The add
    /// step is done only while Buuz is in the keyboards.
    static func isDone(_ s: TourStep, current: TourStep, enabled: Bool) -> Bool {
        let all = TourStep.allCases
        guard let i = all.firstIndex(of: s), let c = all.firstIndex(of: current), i < c else { return false }
        return s == .add ? enabled : true
    }

    mutating func goNext(enabled: Bool) { step = next(enabled: enabled) }
    mutating func goBack(enabled: Bool) { step = back(enabled: enabled) }
    mutating func skipAll() { step = .done }
}

/// The four rows of the setup card, and what they open.
enum SetupChecklist {
    struct Row: Equatable {
        var id: String
        var done: Bool
    }

    static func rows(enabled: Bool, hasSwitchKey: Bool, switchedOnce: Bool, passed: Set<Int>) -> [Row] {
        [
            Row(id: "add", done: enabled),
            Row(id: "key", done: hasSwitchKey),
            Row(id: "switch", done: switchedOnce),
            Row(id: "lessons", done: passed.isSuperset(of: Set(1...8))),
        ]
    }

    static func doneCount(_ rows: [Row]) -> Int { rows.filter { $0.done }.count }
    static func allDone(_ rows: [Row]) -> Bool { rows.allSatisfy { $0.done } }

    /// The index of the first row that is not done.
    static func current(_ rows: [Row]) -> Int? { rows.firstIndex { !$0.done } }

    static func nextLesson(passed: Set<Int>) -> Int? { (1...8).first { !passed.contains($0) } }

    static func step(for id: String, passed: Set<Int> = []) -> TourStep {
        switch id {
        case "add": return .add
        case "key": return .key
        case "switch": return .switchTo
        default: return TourStep.lesson(nextLesson(passed: passed) ?? 1) ?? .lesson1
        }
    }
}

enum Greeting {
    /// Before 12 morning, before 18 afternoon, else evening.
    static func key(hour: Int) -> String {
        if hour < 12 { return "home.greeting.morning" }
        if hour < 18 { return "home.greeting.afternoon" }
        return "home.greeting.evening"
    }
}

enum BuuzSource {
    static let idPrefix = "com.thetengis.inputmethod.Buuz"
    static func isBuuz(id: String?) -> Bool { id?.hasPrefix(idPrefix) ?? false }
}

enum VersionCompare {
    /// True when `a` is a higher version than `b`. Reads "2.1.0" and "v2.1.0".
    /// Bad text is never newer.
    static func isNewer(_ a: String, than b: String) -> Bool {
        func parts(_ s: String) -> [Int]? {
            let t = s.hasPrefix("v") ? String(s.dropFirst()) : s
            let nums = t.split(separator: ".").map { Int($0) }
            return nums.contains(where: { $0 == nil }) || nums.isEmpty ? nil : nums.compactMap { $0 }
        }
        guard let x = parts(a), let y = parts(b) else { return false }
        for i in 0..<max(x.count, y.count) {
            let l = i < x.count ? x[i] : 0, r = i < y.count ? y[i] : 0
            if l != r { return l > r }
        }
        return false
    }
}
