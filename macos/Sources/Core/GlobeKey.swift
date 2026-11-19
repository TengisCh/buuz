// SPDX-License-Identifier: MIT OR Apache-2.0
// What the Globe (fn) key does on this Mac. macOS keeps the choice in
// System Settings > Keyboard > "Press 🌐 key to". Buuz only reads it.

import Foundation

enum GlobeKeyState: Equatable {
    /// "Change Input Source": the Globe key can be the switch key.
    case switchesSource
    /// Another action: "none", "emoji" or "dictation".
    case other(String)
    case unknown

    /// `usageType` is the `AppleFnUsageType` value.
    init(usageType: Int?) {
        switch usageType {
        case 1: self = .switchesSource
        case 0: self = .other("none")
        case 2: self = .other("emoji")
        case 3: self = .other("dictation")
        default: self = .unknown
        }
    }

    var works: Bool { self == .switchesSource }

    static func read() -> GlobeKeyState {
        let value = CFPreferencesCopyAppValue(
            "AppleFnUsageType" as CFString, "com.apple.HIToolbox" as CFString)
        return GlobeKeyState(usageType: (value as? NSNumber)?.intValue)
    }
}
