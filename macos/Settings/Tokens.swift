// SPDX-License-Identifier: MIT OR Apache-2.0
// Colours, fonts and sizes of the Buuz app (design/macos-app/spec.md section 3).
// Colours are dynamic `NSColor` values, so they follow the window appearance.

import AppKit
import SwiftUI

private func hex(_ v: UInt32, _ a: CGFloat = 1) -> NSColor {
    NSColor(
        srgbRed: CGFloat((v >> 16) & 0xFF) / 255, green: CGFloat((v >> 8) & 0xFF) / 255,
        blue: CGFloat(v & 0xFF) / 255, alpha: a)
}

private func dyn(_ light: NSColor, _ dark: NSColor) -> Color {
    Color(nsColor: NSColor(name: nil) { appearance in
        appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua ? dark : light
    })
}

private func same(_ v: UInt32, _ a: CGFloat = 1) -> Color { Color(nsColor: hex(v, a)) }

enum Tok {
    static let frame = dyn(hex(0xF4EFE8), hex(0x121019))
    static let panel = dyn(hex(0xFFFFFF), hex(0x1B1825))
    static let panelLine = dyn(hex(0xEAE2D7), NSColor(white: 1, alpha: 0.07))
    static let text = dyn(hex(0x1E1838), hex(0xF3F0F9))
    static let text2 = dyn(hex(0x645D72), hex(0xA8A2B7))
    static let text3 = dyn(hex(0x9A93A6), hex(0x6F6981))
    static let fill = dyn(hex(0xF7F3EE), hex(0x221E2E))
    static let fill2 = dyn(hex(0xEFE8DF), hex(0x2B2639))
    static let hover = dyn(hex(0x1E1838, 0.05), NSColor(white: 1, alpha: 0.05))
    static let selected = dyn(hex(0x1E1838, 0.075), NSColor(white: 1, alpha: 0.085))
    static let sep = dyn(hex(0xEAE2D7), NSColor(white: 1, alpha: 0.06))
    static let track = dyn(hex(0xE6DED3), hex(0x2E2940))
    static let accent = dyn(hex(0x2E6BFF), hex(0x4D82FF))
    static let accentEdge = dyn(hex(0x1D4FD1), hex(0x2D5FD6))
    static let accentSoft = dyn(hex(0xEAF0FF), hex(0x4D82FF, 0.15))
    static let ember = same(0xFF6A3D)
    static let amber = same(0xFFB648)
    static let success = dyn(hex(0x12805A), hex(0x43D196))
    /// The checkmark on a `success` disc. At least 4.5:1 in both themes.
    static let onSuccess = dyn(hex(0xFFFFFF), hex(0x1B1825))
    static let successBg = dyn(hex(0xE5F6EE), hex(0x43D196, 0.12))
    static let warn = dyn(hex(0xA85400), hex(0xFFC266))
    static let warnBg = dyn(hex(0xFFF2DF), hex(0xFFB648, 0.11))
    static let danger = dyn(hex(0xD23B3B), hex(0xFF7B7B))
    static let dangerBg = dyn(hex(0xD23B3B, 0.12), hex(0xFF7B7B, 0.12))
    static let hero = dyn(hex(0x1E1838), hex(0x0D0B14))
    static let keyBg = dyn(hex(0xFFFFFF), hex(0x2A2537))
    static let keyLine = dyn(hex(0xD8CFC2), hex(0x3B3550))
    static let keyEdge = dyn(hex(0xCFC5B8), hex(0x15121D))

    static let scrim = dyn(hex(0x1E1838, 0.30), NSColor(white: 0, alpha: 0.55))
    static let btn2 = dyn(hex(0xFFFFFF), hex(0x2E2940))
    static let btn2Line = dyn(hex(0xE3DACE), NSColor.clear)
}

extension Font {
    static func rounded(_ size: CGFloat, _ weight: Font.Weight = .bold) -> Font {
        .system(size: size, weight: weight, design: .rounded)
    }
    static func mono(_ size: CGFloat, _ weight: Font.Weight = .semibold) -> Font {
        .system(size: size, weight: weight, design: .monospaced)
    }
    static let pageTitle = Font.rounded(24)
    static let tourTitle = Font.rounded(28, .heavy)
    static let heroTitle = Font.rounded(21, .heavy)
    static let tileValue = Font.rounded(17)
    static let lessonOutput = Font.rounded(22)
    static let practice = Font.system(size: 26)
    static let body13 = Font.system(size: 13.5)
    static let rowTitle = Font.system(size: 13.5, weight: .semibold)
    static let hint = Font.system(size: 12.5)
    static let sectionTitle = Font.system(size: 13, weight: .semibold)
}

/// The width of the page column, set by `PageColumn`. Rows use it to hide
/// optional text in a narrow column.
private struct ColumnWidthKey: EnvironmentKey { static let defaultValue: CGFloat = 646 }

extension EnvironmentValues {
    var columnWidth: CGFloat {
        get { self[ColumnWidthKey.self] }
        set { self[ColumnWidthKey.self] = newValue }
    }
}

var reduceMotion: Bool { NSWorkspace.shared.accessibilityDisplayShouldReduceMotion }
