// SPDX-License-Identifier: MIT OR Apache-2.0
// Finds, adds and detects the Buuz input source (docs/desktop-onboarding.md
// section 3.3). Needs no permission.

import Carbon
import Foundation

enum InputSource {
    static let id = "com.thetengis.inputmethod.Buuz.Mongolian"

    /// The places where the input method app can be installed.
    static var bundleURLs: [URL] {
        let home = FileManager.default.homeDirectoryForCurrentUser
        return [
            home.appendingPathComponent("Library/Input Methods/Buuz.app"),
            URL(fileURLWithPath: "/Library/Input Methods/Buuz.app"),
        ]
    }

    private static func find() -> TISInputSource? {
        let filter = [kTISPropertyInputSourceID as String: id] as CFDictionary
        guard let list = TISCreateInputSourceList(filter, true)?.takeRetainedValue(),
            CFArrayGetCount(list) > 0
        else { return nil }
        let raw = CFArrayGetValueAtIndex(list, 0)
        return Unmanaged<TISInputSource>.fromOpaque(raw!).takeUnretainedValue()
    }

    /// True when Buuz is in the user's keyboards.
    static var isEnabled: Bool {
        guard let source = find(),
            let p = TISGetInputSourceProperty(source, kTISPropertyInputSourceIsEnabled)
        else { return false }
        return CFBooleanGetValue(Unmanaged<CFBoolean>.fromOpaque(p).takeUnretainedValue())
    }

    /// Register the installed input method and turn it on. Gives false when
    /// the app is not installed or macOS refuses. The user can then follow the
    /// manual steps.
    @discardableResult
    static func enable() -> Bool {
        for url in bundleURLs where FileManager.default.fileExists(atPath: url.path) {
            _ = TISRegisterInputSource(url as CFURL)
        }
        guard let source = find() else { return false }
        return TISEnableInputSource(source) == noErr
    }

    /// Call `changed` when the list of enabled keyboards changes.
    static func observe(_ changed: @escaping () -> Void) -> NSObjectProtocol {
        DistributedNotificationCenter.default().addObserver(
            forName: Notification.Name("com.apple.Carbon.TISNotifyEnabledKeyboardInputSourcesChanged"),
            object: nil, queue: .main
        ) { _ in changed() }
    }
}
