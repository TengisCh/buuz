// SPDX-License-Identifier: MIT OR Apache-2.0
// The theme choice and what the UI needs from it.

import Foundation

enum Theme: String, CaseIterable {
    case system, light, dark

    /// The `NSAppearance.Name` raw value to force, or nil to follow the system.
    var appearanceName: String? {
        switch self {
        case .system: return nil
        case .light: return "NSAppearanceNameAqua"
        case .dark: return "NSAppearanceNameDarkAqua"
        }
    }

    /// True or false to force a colour scheme, nil to follow the system.
    var forcedDark: Bool? {
        switch self {
        case .system: return nil
        case .light: return false
        case .dark: return true
        }
    }
}
