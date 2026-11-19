// SPDX-License-Identifier: MIT OR Apache-2.0
// Registers the Buuz switch key with Carbon. This needs no permission. The
// handler never reads or logs keys: it only knows that the hotkey fired.
//
// This code runs inside the input method process. macOS starts that process
// only when the user selects Buuz (or after login, when Buuz was the last
// keyboard). Until then the switch key is not registered. The user may need
// to select Buuz once after each login. The input method has no reliable,
// permission-free way to start itself, and Buuz adds no login item for it.

import Carbon
import Cocoa

final class HotkeyManager {
    static let shared = HotkeyManager()
    private var ref: EventHotKeyRef?
    private var handler: EventHandlerRef?
    private var lastOther: String?
    /// The OSStatus when `InstallEventHandler` failed. No key can work then.
    private var installFailure: Int?
    private let store = HotkeyStore()
    private let buuzID = "com.thetengis.inputmethod.Buuz.Mongolian"

    func start() {
        var spec = EventTypeSpec(
            eventClass: OSType(kEventClassKeyboard), eventKind: UInt32(kEventHotKeyPressed))
        let installed = InstallEventHandler(
            GetApplicationEventTarget(),
            { _, _, _ in
                HotkeyManager.shared.toggle()
                return noErr
            }, 1, &spec, nil, &handler)
        installFailure = installed == noErr ? nil : Int(installed)
        DistributedNotificationCenter.default().addObserver(
            forName: Notification.Name(HotkeyToggle.notification), object: nil, queue: .main
        ) { [weak self] _ in self?.register() }
        // Remember the last other keyboard.
        DistributedNotificationCenter.default().addObserver(
            forName: Notification.Name(kTISNotifySelectedKeyboardInputSourceChanged as String),
            object: nil, queue: .main
        ) { [weak self] _ in self?.noteSource() }
        noteSource()
        register()
    }

    /// Reads the hotkey again and registers it.
    func register() {
        if let r = ref { UnregisterEventHotKey(r); ref = nil }
        guard let key = store.resolve(), key.isValid else { return }
        let id = EventHotKeyID(signature: OSType(0x4255_555A), id: 1)
        var code = installFailure ?? 0
        if code == 0 {
            let status = RegisterEventHotKey(
                UInt32(key.keyCode), key.mods.carbon, id, GetApplicationEventTarget(), 0, &ref)
            code = Int(status)
            if status != noErr { ref = nil }
        }
        // The settings app reads this to show a warning. It holds no typing.
        store.writeStatus(HotkeyStatus(key: key.encoded, registered: code == 0, code: code))
        DistributedNotificationCenter.default().postNotificationName(
            Notification.Name(HotkeyToggle.statusNotification), object: nil, userInfo: nil,
            deliverImmediately: true)
    }

    private static func currentID() -> String? {
        guard let src = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
            let p = TISGetInputSourceProperty(src, kTISPropertyInputSourceID)
        else { return nil }
        return Unmanaged<CFString>.fromOpaque(p).takeUnretainedValue() as String
    }

    private func noteSource() {
        if let id = HotkeyManager.currentID(), id != buuzID { lastOther = id }
    }

    /// The enabled keyboards that a program may select, as ids.
    private static func availableIDs() -> [String] {
        let filter = [kTISPropertyInputSourceCategory as String: kTISCategoryKeyboardInputSource as String,
            kTISPropertyInputSourceIsSelectCapable as String: true] as CFDictionary
        guard let list = TISCreateInputSourceList(filter, false)?.takeRetainedValue() as? [TISInputSource]
        else { return [] }
        return list.compactMap { src in
            guard let p = TISGetInputSourceProperty(src, kTISPropertyInputSourceID) else { return nil }
            return Unmanaged<CFString>.fromOpaque(p).takeUnretainedValue() as String
        }
    }

    private static func asciiCapableID() -> String? {
        guard let src = TISCopyCurrentASCIICapableKeyboardLayoutInputSource()?.takeRetainedValue(),
            let p = TISGetInputSourceProperty(src, kTISPropertyInputSourceID)
        else { return nil }
        return Unmanaged<CFString>.fromOpaque(p).takeUnretainedValue() as String
    }

    private func toggle() {
        guard let target = HotkeyToggle.target(
            current: HotkeyManager.currentID(), buuz: buuzID, lastOther: lastOther,
            asciiCapable: HotkeyManager.asciiCapableID(), available: HotkeyManager.availableIDs())
        else { return }
        let filter = [kTISPropertyInputSourceID as String: target] as CFDictionary
        guard let list = TISCreateInputSourceList(filter, false)?.takeRetainedValue() as? [TISInputSource],
            let source = list.first
        else { return }
        TISSelectInputSource(source)
    }
}
