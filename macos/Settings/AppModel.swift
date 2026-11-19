// SPDX-License-Identifier: MIT OR Apache-2.0
// State of the Buuz app: pages, the tutorial, the keyboard that is current now,
// the switch key and the typing settings. No key the user types is read here.

import AppKit
import Foundation
import ServiceManagement
import SwiftUI

enum AppPage { case home, lessons, help }

enum SettingsPage: String, CaseIterable, Identifiable {
    case general, typing, shortcuts, privacy, about
    var id: String { rawValue }
}

/// What the hotkey recorder shows.
enum RecorderState: Equatable {
    case idle
    case recording(held: HotkeyMods)
    case conflict(Hotkey, keys: String, action: String)
    case saved(Hotkey)
}

enum UpdateState: Equatable {
    case idle, checking, upToDate, newer(String), failed
}

final class AppModel: ObservableObject {
    let strings: Strings
    private let prefs = Prefs(DefaultsStore())
    private let configURL = ConfigStore.defaultConfigURL
    private let hotkeyStore = HotkeyStore()
    private var observers: [NSObjectProtocol] = []
    private var monitor: Any?

    static let version = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "2.0.0"
    static let repo = "https://github.com/TengisCh/buuz"
    static let keyboardSettings = "x-apple.systempreferences:com.apple.Keyboard-Settings.extension"

    // Look
    @Published var language: Language { didSet { prefs.language = language; strings.language = language } }
    @Published var theme: Theme { didSet { prefs.theme = theme; applyTheme() } }

    // Pages
    @Published var page: AppPage = .home { didSet { if page != oldValue { cancelRecording() } } }
    /// Closing the sheet or changing its page stops the key recorder, so its
    /// event monitor never outlives the page that shows it.
    @Published var sheet: SettingsPage? { didSet { if sheet != oldValue { cancelRecording() } } }
    private var lastSettings: SettingsPage = .general
    @Published var heroHidden = false
    @Published var setupFolded: Bool { didSet { prefs.setupFolded = setupFolded } }

    // Tutorial
    @Published var tourActive: Bool
    @Published var step: TourStep { didSet { stepChanged() } }
    @Published var passed: Set<Int> { didSet { prefs.passedLessons = passed } }
    @Published var checker = LessonChecker(lesson: 1)
    @Published var fieldReset = 0
    private var tourReturn: AppPage?
    private var tourSingle = false

    // The keyboard
    @Published var enabled: Bool
    @Published var addFailed = false
    /// Why "Switch to Buuz" did not work. Nil when it worked or never ran.
    @Published var switchFailure: SwitchResult?
    @Published var currentKeyboard: (id: String, name: String)?
    @Published var switchedOnce: Bool
    @Published var switchProgress = SwitchProgress()
    /// Buuz, another keyboard, Buuz again, seen since the app started. The
    /// Shortcuts page uses it for "It works".
    @Published var roundTrip = SwitchProgress()

    // The switch key
    @Published var selectedKey: SwitchKey?
    @Published var customHotkey: Hotkey?
    @Published var recorder: RecorderState = .idle
    /// The OSStatus when the input method could not register the chosen key.
    /// Nil when it worked or when the input method has not reported yet.
    @Published var hotkeyFailure: Int?

    // Typing
    @Published var configText: String?

    // General
    @Published var update: UpdateState = .idle
    @Published var launchAtLogin = false
    /// The "ready when my computer starts" switch on the Done step.
    @Published var loginAtFinish = true

    init() {
        let dir = Bundle.main.resourceURL?.appendingPathComponent("Strings")
            ?? URL(fileURLWithPath: "Strings")
        guard let loaded = Strings(directory: dir, language: prefs.language) else {
            fatalError("The string files are missing from the app bundle")
        }
        strings = loaded
        language = prefs.language
        theme = prefs.theme
        setupFolded = prefs.setupFolded
        tourActive = !prefs.onboardingDone
        step = prefs.tourStep
        passed = prefs.passedLessons
        enabled = InputSource.isEnabled
        currentKeyboard = InputSource.current
        switchedOnce = prefs.switchedOnce
        // Show the key that the input method registers (HotkeyStore.resolve),
        // also when the user never chose one.
        let initial = SwitchKeyState.initial(
            saved: prefs.switchKey.flatMap(SwitchKey.init(rawValue:)), custom: prefs.customHotkey,
            storeHasChoice: hotkeyStore.hasChoice, resolved: hotkeyStore.resolve())
        selectedKey = initial.key
        customHotkey = initial.hotkey
        if selectedKey == .custom, let h = customHotkey { recorder = .saved(h) }
        refreshConfig()
        refreshHotkeyStatus()
        launchAtLogin = AppModel.loginEnabled
        applyTheme()
        checker = LessonChecker(lesson: step.lessonNumber ?? 1)
        observers.append(InputSource.observe { [weak self] in self?.refreshEnabled() })
        observers.append(InputSource.observeSelected { [weak self] in self?.refreshKeyboard() })
        observers.append(
            NotificationCenter.default.addObserver(
                forName: NSApplication.didBecomeActiveNotification, object: nil, queue: .main
            ) { [weak self] _ in
                self?.refreshKeyboard()
                self?.refreshConfig()
                self?.refreshHotkeyStatus()
            })
        observers.append(
            DistributedNotificationCenter.default().addObserver(
                forName: Notification.Name(HotkeyToggle.statusNotification), object: nil, queue: .main
            ) { [weak self] _ in self?.refreshHotkeyStatus() })
        refreshKeyboard()
    }

    /// Text for `key` in the chosen language. Reading `language` makes the
    /// views update when it changes.
    func t(_ key: String, _ values: [String: String] = [:]) -> String {
        _ = language
        return strings.text(key, values)
    }

    // MARK: Look

    /// Sets the window appearance, so the dynamic colours and the traffic
    /// lights follow the choice. `nil` follows the system.
    /// Only `NSApp.appearance` is used. A SwiftUI colour scheme that was
    /// forced once can keep a window dark after "System", so there is none.
    /// Windows never keep an own appearance.
    func applyTheme() {
        let name = theme.appearanceName.map { NSAppearance.Name(rawValue: $0) }
        NSApplication.shared.appearance = name.flatMap { NSAppearance(named: $0) }
        for window in NSApplication.shared.windows { window.appearance = nil }
    }

    // MARK: Keyboard

    func refreshEnabled() { enabled = InputSource.isEnabled }

    /// Reads the current keyboard again. Called when the user picks another
    /// keyboard and when the window gets focus.
    func refreshKeyboard() {
        refreshEnabled()
        currentKeyboard = InputSource.current
        let buuz = isBuuzCurrent
        if buuz && !switchedOnce {
            switchedOnce = true
            prefs.switchedOnce = true
        }
        roundTrip.observe(isBuuz: buuz)
        if step == .switchTo { switchProgress.observe(isBuuz: buuz) }
    }

    var isBuuzCurrent: Bool { BuuzSource.isBuuz(id: currentKeyboard?.id) }

    /// The name of the other keyboard, for the texts. ABC when unknown.
    var otherKeyboardName: String {
        if let k = currentKeyboard, !BuuzSource.isBuuz(id: k.id) { return k.name }
        return "ABC"
    }

    func addBuuz() {
        addFailed = !InputSource.enable()
        if !addFailed { InputSource.launchInstalled() }
        refreshEnabled()
    }

    /// Makes Buuz the current keyboard. Used by "Switch to Buuz". It never
    /// fails silently: `switchFailure` tells the screens to show a banner.
    func switchToBuuz() {
        switchFailure = nil
        SwitchToBuuz.run(
            isEnabled: { InputSource.isEnabled },
            enable: { InputSource.enable() },
            select: { InputSource.select().map { Int($0) } },
            launch: { InputSource.launchInstalled() },
            after: { work in DispatchQueue.main.asyncAfter(deadline: .now() + 1.0, execute: work) },
            done: { [weak self] result in
                self?.switchFailure = result.isFailure ? result : nil
                self?.refreshKeyboard()
            })
    }

    // MARK: Pages and sheet

    func openSettings(_ p: SettingsPage? = nil) {
        sheet = p ?? lastSettings
        if let s = sheet { lastSettings = s }
    }

    func closeSettings() {
        cancelRecording()
        sheet = nil
    }

    // MARK: Setup card

    var setupRows: [SetupChecklist.Row] {
        SetupChecklist.rows(
            enabled: enabled, hasSwitchKey: switchKeyReady,
            switchedOnce: switchedOnce, passed: passed)
    }

    var setupDone: Bool { SetupChecklist.allDone(setupRows) }

    // MARK: Tutorial

    var nextLesson: Int? { SetupChecklist.nextLesson(passed: passed) }

    func next() {
        if step == .done { finishTour(); return }
        if tourSingle, step.lessonNumber != nil {
            leaveTour()
            return
        }
        var f = TourFlow(step: step)
        f.goNext(enabled: enabled)
        step = f.step
    }

    func back() {
        var f = TourFlow(step: step)
        f.goBack(enabled: enabled)
        step = f.step
    }

    func skipAll() {
        var f = TourFlow(step: step)
        f.skipAll()
        step = f.step
    }

    /// Start the tutorial. A single lesson returns to `returnTo` when it is
    /// done. Without `single`, the tutorial goes on to the end.
    func startTour(at s: TourStep = .welcome, returnTo: AppPage? = nil, single: Bool = false) {
        tourReturn = returnTo
        tourSingle = single
        sheet = nil
        tourActive = true
        if returnTo == nil && !single { prefs.onboardingDone = false }
        step = s
        stepChanged()
    }

    func replayLesson(_ n: Int) {
        startTour(at: TourStep.lesson(n) ?? .lesson1, returnTo: .lessons, single: true)
    }

    private func leaveTour() {
        tourActive = false
        page = tourReturn ?? .home
        tourReturn = nil
        tourSingle = false
    }

    func finishTour() {
        if loginAtFinish && AppModel.loginAvailable && !launchAtLogin { setLaunchAtLogin(true) }
        prefs.onboardingDone = true
        prefs.tourStep = .welcome
        leaveTour()
    }

    private func stepChanged() {
        if tourReturn == nil && !tourSingle && tourActive { prefs.tourStep = step }
        checker = LessonChecker(lesson: step.lessonNumber ?? 1)
        fieldReset += 1
        if step == .switchTo {
            switchProgress = SwitchProgress()
            switchProgress.observe(isBuuz: isBuuzCurrent)
        }
        if step != .key { cancelRecording() }
    }

    func lessonFeed(_ text: LessonText) {
        checker.feed(text)
        notePassed()
    }

    func lessonSubmit() {
        checker.submit()
        notePassed()
    }

    func lessonClear() {
        checker.reset()
        fieldReset += 1
    }

    private func notePassed() {
        if checker.passed, let n = step.lessonNumber, !passed.contains(n) { passed.insert(n) }
    }

    /// True when the current step has passed, so Next is on.
    var stepPassed: Bool {
        switch step {
        case .add: return enabled
        case .key: return switchKeyReady
        case .switchTo: return switchProgress.done
        case .lesson1, .lesson2, .lesson3, .lesson4, .lesson5, .lesson6, .lesson7, .lesson8:
            return checker.passed
        default: return true
        }
    }

    // MARK: Switch key

    /// The keys of the chosen switch key, as keycap texts.
    var switchKeyCaps: [String] {
        switch selectedKey ?? .fn {
        case .fn: return ["🌐 fn"]
        case .controlSpace: return ["⌃", "Space"]
        case .capsLock: return ["⇪ caps"]
        case .custom: return customHotkey.map { Self.caps($0) } ?? []
        }
    }

    /// One line for the texts, for example "🌐 fn".
    var switchKeyLabel: String {
        switch selectedKey ?? .fn {
        case .fn: return "🌐 fn"
        case .controlSpace: return "⌃ Space"
        case .capsLock: return "⇪ Caps Lock"
        case .custom: return customHotkey?.display ?? "—"
        }
    }

    static func caps(_ h: Hotkey) -> [String] {
        var out: [String] = []
        if h.mods.contains(.control) { out.append("⌃") }
        if h.mods.contains(.option) { out.append("⌥") }
        if h.mods.contains(.shift) { out.append("⇧") }
        if h.mods.contains(.command) { out.append("⌘") }
        out.append(String(h.display.drop(while: { "⌃⌥⇧⌘".contains($0) })))
        return out
    }

    /// A switch key is chosen, and no recording or conflict is open.
    var switchKeyReady: Bool {
        guard let k = selectedKey else { return false }
        if k == .custom {
            if case .saved = recorder { return true }
            return false
        }
        return true
    }

    func chooseSwitchKey(_ k: SwitchKey) {
        cancelRecording()
        selectedKey = k
        if k == .custom {
            if let h = customHotkey, SystemShortcuts.conflict(for: h) == nil {
                commitCustom(h)
            } else {
                recorder = .idle
                startRecording()
            }
        } else {
            recorder = .idle
            prefs.switchKey = k.rawValue
            _ = hotkeyStore.save(nil)
            postHotkeyChanged()
        }
    }

    private func commitCustom(_ h: Hotkey) {
        customHotkey = h
        prefs.customHotkey = h
        prefs.switchKey = SwitchKey.custom.rawValue
        selectedKey = .custom
        recorder = .saved(h)
        _ = hotkeyStore.save(h)
        postHotkeyChanged()
        refreshHotkeyStatus()
    }

    /// Reads what the input method said about the key it registered.
    func refreshHotkeyStatus() {
        let shown = selectedKey == .custom ? customHotkey : nil
        hotkeyFailure = hotkeyStore.readStatus()?.failure(for: shown)
    }

    /// The input method process registers the hotkey. It listens for this.
    private func postHotkeyChanged() {
        DistributedNotificationCenter.default().postNotificationName(
            Notification.Name(HotkeyToggle.notification), object: nil, userInfo: nil,
            deliverImmediately: true)
    }

    func startRecording() {
        selectedKey = .custom
        recorder = .recording(held: [])
        removeMonitor()
        // Only while recording, and only for this window. No permission needed.
        monitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .flagsChanged]) { [weak self] event in
            self?.recordEvent(event) ?? event
        }
    }

    /// Stops the key recorder. Safe to call at any time and more than once.
    func cancelRecording() {
        removeMonitor()
        guard case .recording = recorder else { return }
        if let h = customHotkey, prefs.switchKey == SwitchKey.custom.rawValue {
            recorder = .saved(h)
        } else {
            recorder = .idle
        }
    }

    private func removeMonitor() {
        if let m = monitor { NSEvent.removeMonitor(m) }
        monitor = nil
    }

    private func recordEvent(_ event: NSEvent) -> NSEvent? {
        guard case .recording = recorder else { return event }
        let mods = HotkeyMods(cocoaFlags: Int(event.modifierFlags.rawValue))
        if event.type == .flagsChanged {
            recorder = .recording(held: mods)
            return nil
        }
        if event.keyCode == 53 && mods.isEmpty {
            cancelRecording()
            return nil
        }
        let h = Hotkey(keyCode: Int(event.keyCode), mods: mods)
        switch HotkeyRecording.classify(h, system: SystemShortcuts.read()) {
        case .needsModifier:
            NSSound.beep()
        case .conflict(let keys, let actionKey, let name):
            removeMonitor()
            recorder = .conflict(h, keys: keys, action: actionKey.map { t($0) } ?? name)
        case .ok(let ok):
            removeMonitor()
            commitCustom(ok)
        }
        return nil
    }

    func useConflictAnyway() {
        if case .conflict(let h, _, _) = recorder { commitCustom(h) }
    }

    func clearHotkey() {
        removeMonitor()
        customHotkey = nil
        prefs.customHotkey = nil
        prefs.switchKey = nil
        recorder = .idle
        _ = hotkeyStore.save(nil)
        postHotkeyChanged()
    }

    // MARK: Typing settings

    /// A switch in the portable config. The input method reads the file the next
    /// time a text field gets focus.
    func flag(_ key: String, default value: Bool) -> Bool {
        ConfigEditor.flag(configText, key) ?? value
    }

    /// The value in the config file, or nil when the file does not set it.
    func flag(_ key: String) -> Bool? { ConfigEditor.flag(configText, key) }

    func setFlag(_ key: String, _ value: Bool) { writeConfig { ConfigEditor.setFlag($0, key, value) } }

    var isClassic: Bool { ConfigEditor.preset(configText) == "classic" }

    func setClassic(_ classic: Bool) {
        writeConfig { ConfigEditor.setPreset($0, classic ? "classic" : "smart") }
    }

    /// Reads the file again, so a change in the user's own editor shows here.
    func refreshConfig() {
        configText = try? String(contentsOf: configURL, encoding: .utf8)
    }

    /// Reads the file, edits it and writes it. It never edits an old copy.
    private func writeConfig(_ edit: (String?) -> String) {
        if let text = ConfigEditor.modify(at: configURL, edit) {
            configText = text
        } else {
            NSLog("Buuz: could not write the config file")
            refreshConfig()
        }
    }

    var configPath: String {
        let p = configURL.path
        let home = NSHomeDirectory()
        return p.hasPrefix(home) ? "~" + p.dropFirst(home.count) : p
    }

    func revealConfig() {
        if FileManager.default.fileExists(atPath: configURL.path) {
            NSWorkspace.shared.activateFileViewerSelecting([configURL])
        } else {
            NSWorkspace.shared.open(configURL.deletingLastPathComponent())
        }
    }

    /// The Cyrillic result of fixed sample keys, with the current settings.
    func preview(_ keys: String, escape: Bool = false) -> String {
        Preview.result(keys: keys, escapeBeforeSpace: escape, config: configText, words: SharedWords.list)
    }

    // MARK: General

    static var loginEnabled: Bool {
        if #available(macOS 13.0, *) { return SMAppService.mainApp.status == .enabled }
        return false
    }

    static var loginAvailable: Bool {
        if #available(macOS 13.0, *) { return true }
        return false
    }

    func setLaunchAtLogin(_ on: Bool) {
        if #available(macOS 13.0, *) {
            do {
                if on { try SMAppService.mainApp.register() } else { try SMAppService.mainApp.unregister() }
            } catch { NSLog("Buuz: could not change open at login") }
        }
        launchAtLogin = AppModel.loginEnabled
    }

    /// Asks GitHub for the newest release, only when the user clicks. It sends
    /// nothing about typing.
    func checkUpdates() {
        update = .checking
        guard let url = URL(string: "https://api.github.com/repos/TengisCh/buuz/releases/latest") else { return }
        var req = URLRequest(url: url, timeoutInterval: 10)
        req.setValue("application/vnd.github+json", forHTTPHeaderField: "Accept")
        URLSession.shared.dataTask(with: req) { [weak self] data, _, _ in
            let tag = data.flatMap { try? JSONSerialization.jsonObject(with: $0) as? [String: Any] }
                .flatMap { $0["tag_name"] as? String }
            DispatchQueue.main.async {
                guard let self = self else { return }
                guard let tag = tag else { self.update = .failed; return }
                self.update = VersionCompare.isNewer(tag, than: AppModel.version) ? .newer(tag) : .upToDate
            }
        }.resume()
    }

    /// Opens the NOTICE file in the app bundle (text editor, or the default app).
    func openNotices() {
        guard let url = Bundle.main.url(forResource: "NOTICE", withExtension: nil) else { return }
        if let editor = NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.apple.TextEdit") {
            NSWorkspace.shared.open([url], withApplicationAt: editor, configuration: NSWorkspace.OpenConfiguration())
        } else {
            NSWorkspace.shared.open(url)
        }
    }

    func open(_ url: String) {
        if let u = URL(string: url) { NSWorkspace.shared.open(u) }
    }
}

/// The word list for the Typing preview. It loads once from the app bundle.
enum SharedWords {
    static let list: WordList? = {
        Bundle.main.url(forResource: "mn", withExtension: "txt").flatMap { WordList(path: $0.path) }
    }()
}
