// SPDX-License-Identifier: MIT OR Apache-2.0
// Reads the list of enabled keyboards that macOS keeps in the preference
// `AppleEnabledInputSources` (domain com.apple.HIToolbox). The TIS flag can be
// stale in a long-lived process, so the app trusts this list.

import Foundation

enum EnabledSources {
    static let bundleID = "com.thetengis.inputmethod.Buuz"
    static let inputModeID = "com.thetengis.inputmethod.Buuz.Mongolian"

    /// True when one entry belongs to Buuz: its "Bundle ID" is the Buuz app, or
    /// its "Input Mode" is the Buuz mode (or starts with the Buuz id).
    static func containsBuuz(_ entries: [[String: Any]]) -> Bool {
        entries.contains { e in
            if (e["Bundle ID"] as? String) == bundleID { return true }
            if let mode = e["Input Mode"] as? String {
                return mode == inputModeID || mode.hasPrefix(bundleID)
            }
            return false
        }
    }
}
