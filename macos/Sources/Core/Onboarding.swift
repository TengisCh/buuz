// SPDX-License-Identifier: MIT OR Apache-2.0
// The order of the onboarding screens, and what the app remembers
// (docs/desktop-onboarding.md section 3).

import Foundation

enum Step: Int, CaseIterable {
    case welcome, privacy, enable, shortcut, tryIt, tip, done

    /// Screens 2 to 6 show progress dots. Welcome and Done do not.
    var showsProgress: Bool { self != .welcome && self != .done }
}

/// Moves between screens. The "Add Buuz" screen is skipped when Buuz is
/// already in the keyboards (Detect, don't ask).
struct OnboardingFlow: Equatable {
    var step: Step = .welcome

    func next(enabled: Bool) -> Step {
        guard let n = Step(rawValue: step.rawValue + 1) else { return step }
        return (n == .enable && enabled) ? .shortcut : n
    }

    func back(enabled: Bool) -> Step {
        guard let p = Step(rawValue: step.rawValue - 1) else { return step }
        return (p == .enable && enabled) ? .privacy : p
    }

    mutating func goNext(enabled: Bool) { step = next(enabled: enabled) }
    mutating func goBack(enabled: Bool) { step = back(enabled: enabled) }

    /// "Step n of total" over the screens that show progress.
    var progress: (index: Int, total: Int)? {
        guard step.showsProgress else { return nil }
        let shown = Step.allCases.filter { $0.showsProgress }
        return (step.rawValue, shown.count)
    }
}

/// Where the app keeps its small settings. `UserDefaults` in the app, a
/// dictionary in the tests.
protocol KeyValueStore: AnyObject {
    func string(_ key: String) -> String?
    func set(_ value: String?, for key: String)
}

final class MemoryStore: KeyValueStore {
    private var values: [String: String] = [:]
    func string(_ key: String) -> String? { values[key] }
    func set(_ value: String?, for key: String) { values[key] = value }
}

final class DefaultsStore: KeyValueStore {
    private let defaults: UserDefaults
    init(_ defaults: UserDefaults = .standard) { self.defaults = defaults }
    func string(_ key: String) -> String? { defaults.string(forKey: key) }
    func set(_ value: String?, for key: String) {
        if let v = value { defaults.set(v, forKey: key) } else { defaults.removeObject(forKey: key) }
    }
}

/// Language, theme and the onboarding position. Mongolian and the light theme
/// are the defaults. The app does not follow the system language or theme.
final class Prefs {
    private let store: KeyValueStore
    init(_ store: KeyValueStore) { self.store = store }

    var language: Language {
        get { store.string("language").flatMap(Language.init(rawValue:)) ?? .mn }
        set { store.set(newValue.rawValue, for: "language") }
    }

    var dark: Bool {
        get { store.string("theme") == "dark" }
        set { store.set(newValue ? "dark" : "light", for: "theme") }
    }

    /// The screen to open on. Quitting in the middle resumes here.
    var step: Step {
        get { store.string("onboarding.step").flatMap(Int.init).flatMap(Step.init(rawValue:)) ?? .welcome }
        set { store.set(String(newValue.rawValue), for: "onboarding.step") }
    }

    var onboardingDone: Bool {
        get { store.string("onboarding.done") == "1" }
        set { store.set(newValue ? "1" : nil, for: "onboarding.done") }
    }
}
