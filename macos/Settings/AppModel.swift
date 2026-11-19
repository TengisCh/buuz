// SPDX-License-Identifier: MIT OR Apache-2.0
// State of the onboarding and settings app.

import AppKit
import Foundation
import SwiftUI

final class AppModel: ObservableObject {
    let strings: Strings
    private let prefs = Prefs(DefaultsStore())
    private let configURL = ConfigStore.defaultConfigURL
    private var observer: NSObjectProtocol?

    @Published var language: Language { didSet { prefs.language = language; strings.language = language } }
    @Published var dark: Bool { didSet { prefs.dark = dark } }
    @Published var flow: OnboardingFlow { didSet { prefs.step = flow.step } }
    @Published var showOnboarding: Bool
    @Published var enabled: Bool
    @Published var addFailed = false
    @Published var configText: String?

    init() {
        let dir = Bundle.main.resourceURL?.appendingPathComponent("Strings")
            ?? URL(fileURLWithPath: "Strings")
        guard let loaded = Strings(directory: dir, language: prefs.language) else {
            fatalError("The string files are missing from the app bundle")
        }
        strings = loaded
        language = prefs.language
        dark = prefs.dark
        flow = OnboardingFlow(step: prefs.step)
        showOnboarding = !prefs.onboardingDone
        enabled = InputSource.isEnabled
        configText = try? String(contentsOf: configURL, encoding: .utf8)
        observer = InputSource.observe { [weak self] in self?.refreshEnabled() }
    }

    /// Text for `key` in the chosen language. Reading `language` makes the
    /// views update when it changes.
    func t(_ key: String, _ values: [String: String] = [:]) -> String {
        _ = language
        return strings.text(key, values)
    }

    func refreshEnabled() { enabled = InputSource.isEnabled }

    func addBuuz() {
        addFailed = !InputSource.enable()
        refreshEnabled()
    }

    func next() { flow.goNext(enabled: enabled) }
    func back() { flow.goBack(enabled: enabled) }

    func finishOnboarding() {
        prefs.onboardingDone = true
        showOnboarding = false
    }

    func restartOnboarding() {
        flow = OnboardingFlow(step: .welcome)
        prefs.onboardingDone = false
        showOnboarding = true
    }

    /// A switch in the portable config. The input method reads the file the next
    /// time a text field gets focus.
    func flag(_ key: String, default value: Bool) -> Bool {
        ConfigEditor.flag(configText, key) ?? value
    }

    func setFlag(_ key: String, _ value: Bool) {
        let text = ConfigEditor.setFlag(configText, key, value)
        do {
            try FileManager.default.createDirectory(
                at: configURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            try text.write(to: configURL, atomically: true, encoding: .utf8)
            configText = text
        } catch {
            NSLog("Buuz: could not write the config file")
        }
    }

    func open(_ url: String) {
        if let u = URL(string: url) { NSWorkspace.shared.open(u) }
    }

    static let keyboardSettings = "x-apple.systempreferences:com.apple.Keyboard-Settings.extension"
}
