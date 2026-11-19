// SPDX-License-Identifier: MIT OR Apache-2.0
// What the app remembers (docs/desktop-onboarding.md section 3). The order of
// the tutorial steps is in Tutorial.swift.

import Foundation

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

/// Language, theme and the onboarding position. Mongolian is the default
/// language. The theme follows the system by default.
final class Prefs {
    private let store: KeyValueStore
    init(_ store: KeyValueStore) { self.store = store }

    var language: Language {
        get { store.string("language").flatMap(Language.init(rawValue:)) ?? .mn }
        set { store.set(newValue.rawValue, for: "language") }
    }

    /// Follows the system by default. The old values "light" and "dark" still work.
    var theme: Theme {
        get { store.string("theme").flatMap(Theme.init(rawValue:)) ?? .system }
        set { store.set(newValue.rawValue, for: "theme") }
    }

    /// The tutorial step to open on. Quitting in the middle resumes here.
    var tourStep: TourStep {
        get { store.string("tour.step").flatMap(TourStep.init(rawValue:)) ?? .welcome }
        set { store.set(newValue.rawValue, for: "tour.step") }
    }

    /// The lessons (1 to 8) that the user passed.
    var passedLessons: Set<Int> {
        get {
            let list = (store.string("tour.passed") ?? "").split(separator: ",").compactMap { Int($0) }
            return Set(list.filter { (1...8).contains($0) })
        }
        set {
            let text = newValue.sorted().map(String.init).joined(separator: ",")
            store.set(text.isEmpty ? nil : text, for: "tour.passed")
        }
    }

    /// `SwitchKey.rawValue` of the chosen switch key. Nil until the user chooses.
    var switchKey: String? {
        get { store.string("switchKey") }
        set { store.set(newValue, for: "switchKey") }
    }

    /// The recorded own shortcut, kept when the user picks another option.
    var customHotkey: Hotkey? {
        get { store.string("switchKey.custom").flatMap(Hotkey.decode) }
        set { store.set(newValue?.encoded, for: "switchKey.custom") }
    }

    /// True after the current keyboard was Buuz once.
    var switchedOnce: Bool {
        get { store.string("switched.once") == "1" }
        set { store.set(newValue ? "1" : nil, for: "switched.once") }
    }

    /// True when the user folded the setup card in the sidebar.
    var setupFolded: Bool {
        get { store.string("setup.folded") == "1" }
        set { store.set(newValue ? "1" : nil, for: "setup.folded") }
    }

    var onboardingDone: Bool {
        get { store.string("onboarding.done") == "1" }
        set { store.set(newValue ? "1" : nil, for: "onboarding.done") }
    }
}
