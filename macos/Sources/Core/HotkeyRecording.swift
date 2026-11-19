// SPDX-License-Identifier: MIT OR Apache-2.0
// Decides what the hotkey recorder does with a recorded key. Pure logic.

import Foundation

/// The switch key choices. The first three are macOS settings. Only
/// `custom` is registered by the Buuz input method.
enum SwitchKey: String, CaseIterable {
    case fn, controlSpace, capsLock, custom

    var isSystemOwned: Bool { self != .custom }
}

/// The switch key the Buuz app shows when it starts.
enum SwitchKeyState {
    /// With no saved choice, the input method registers `resolved` (the
    /// default key when it is free). The app shows that same key, so what is
    /// shown is what is registered.
    static func initial(
        saved: SwitchKey?, custom: Hotkey?, storeHasChoice: Bool, resolved: Hotkey?
    ) -> (key: SwitchKey?, hotkey: Hotkey?) {
        if saved == nil, !storeHasChoice, let h = resolved { return (.custom, h) }
        return (saved, custom)
    }

    /// After the user clears the key, nothing is selected and no custom key stays.
    static func afterClear() -> (key: SwitchKey?, hotkey: Hotkey?) { (nil, nil) }
}

enum RecordOutcome: Equatable {
    case needsModifier
    case conflict(keys: String, actionKey: String?, name: String)
    case ok(Hotkey)
}

enum HotkeyRecording {
    private static func key(_ code: Int, _ mods: HotkeyMods) -> Hotkey { Hotkey(keyCode: code, mods: mods) }

    /// macOS actions with a string key, for the conflict message.
    private static let actions: [(Hotkey, String)] = [
        (key(49, [.control]), "hotkey.action.prevSource"),
        (key(49, [.control, .option]), "hotkey.action.nextSource"),
        (key(49, [.command]), "hotkey.action.spotlight"),
        (key(49, [.option, .command]), "hotkey.action.spotlight"),
        (key(49, [.control, .command]), "hotkey.action.emoji"),
        (key(48, [.command]), "hotkey.action.appSwitcher"),
        (key(50, [.command]), "hotkey.action.appSwitcher"),
        (key(20, [.shift, .command]), "hotkey.action.screenshot"),
        (key(21, [.shift, .command]), "hotkey.action.screenshot"),
        (key(23, [.shift, .command]), "hotkey.action.screenshot"),
    ]

    /// Shortcuts that macOS always uses, even when they are not in the
    /// symbolic hotkeys list.
    private static let alwaysOn: [(Hotkey, String)] = [
        (key(49, [.control, .command]), "Emoji & Symbols"),
        (key(50, [.command]), "Switch windows of an app"),
        (key(20, [.shift, .command]), "Screenshot"),
        (key(21, [.shift, .command]), "Screenshot"),
        (key(23, [.shift, .command]), "Screenshot"),
    ]

    static func actionKey(for hotkey: Hotkey) -> String? {
        actions.first { $0.0 == hotkey }?.1
    }

    /// `system` is the list of macOS shortcuts that are on now
    /// (`SystemShortcuts.read()`).
    static func classify(_ hotkey: Hotkey, system: [SystemShortcut]) -> RecordOutcome {
        guard hotkey.isValid else { return .needsModifier }
        let name = SystemShortcuts.conflict(for: hotkey, in: system)
            ?? alwaysOn.first { $0.0 == hotkey }?.1
        guard let found = name else { return .ok(hotkey) }
        return .conflict(keys: hotkey.display, actionKey: actionKey(for: hotkey), name: found)
    }
}
