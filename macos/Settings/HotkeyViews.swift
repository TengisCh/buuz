// SPDX-License-Identifier: MIT OR Apache-2.0
// The switch key picker and the hotkey recorder (design/macos-app/spec.md
// sections 5.11 and 8.2). The input method process registers the own shortcut.
// The recorder refuses shortcuts that macOS already uses.

import SwiftUI

struct SwitchKeyPicker: View {
    @EnvironmentObject var m: AppModel
    /// Option rows are 64 high, or 56 in a short tutorial body.
    var rowHeight: CGFloat = 64

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            SettingsGroup {
                option(
                    .fn, title: m.t("hotkey.fn"),
                    note: m.globe.works ? m.t("hotkey.fn.note") : m.t("globe.fn.note"),
                    pill: m.globe.works ? m.t("common.recommended") : nil)
                option(.controlSpace, title: noEmoji(m.t("shortcut.controlSpace.title")), note: m.t("hotkey.controlSpace.note"))
                option(.capsLock, title: noEmoji(m.t("shortcut.capsLock.title")), note: m.t("hotkey.capsLock.note"))
                option(
                    .custom, title: m.t("hotkey.custom"),
                    note: m.selectedKey == .custom ? m.t("hotkey.custom.note") : m.t("hotkey.record"),
                    pill: m.globe.works ? nil : m.t("common.recommended"))
            }
            below
        }
    }

    private func caps(for k: SwitchKey) -> [String] {
        switch k {
        case .fn: return ["🌐 fn"]
        case .controlSpace: return ["⌃", "Space"]
        case .capsLock: return ["⇪ caps"]
        case .custom:
            switch m.recorder {
            case .saved(let h): return AppModel.caps(h)
            case .conflict(let h, _, _): return AppModel.caps(h)
            default: return []
            }
        }
    }

    private func option(_ k: SwitchKey, title: String, note: String, pill: String? = nil) -> some View {
        let on = m.selectedKey == k
        return Button { m.chooseSwitchKey(k) } label: {
            VStack(spacing: 0) {
                Rectangle().fill(Tok.sep).frame(height: 1)
                HStack(spacing: 14) {
                    ZStack {
                        Circle().stroke(on ? Tok.accent : Tok.text3, lineWidth: 1.5)
                        if on { Circle().fill(Tok.accent) }
                        if on { Circle().fill(Color.white).frame(width: 6, height: 6) }
                    }.frame(width: 18, height: 18)
                    VStack(alignment: .leading, spacing: 2) {
                        HStack(spacing: 8) {
                            Text(title).font(.rowTitle).foregroundColor(Tok.text)
                            if let p = pill { Pill(text: p, kind: .blue) }
                        }
                        Text(note).font(.hint).foregroundColor(Tok.text2)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    Keycaps(keys: caps(for: k))
                }
                .padding(.vertical, 10).frame(minHeight: rowHeight - 1)
                .contentShape(Rectangle())
            }
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(on ? [.isButton, .isSelected] : .isButton)
    }

    @ViewBuilder private var below: some View {
        if let k = m.selectedKey {
            if m.globeWarningActive {
                GlobeWarning()
            } else if k != .custom {
                StatusBanner(
                    .neutral, m.t("hotkey.system"),
                    actions: AnyView(
                        Button(m.t("shortcut.manual.button")) { m.open(AppModel.keyboardSettings) }
                            .buttonStyle(SecondaryButtonStyle())))
            } else {
                HotkeyRecorder()
                if let code = m.hotkeyFailure {
                    StatusBanner(.warn, title: m.t("hotkey.failed.title"), m.t("hotkey.failed.body", ["code": "\(code)"]))
                }
            }
        }
    }
}

/// A warning row: fn is the switch key, but the Globe key does something else.
struct GlobeWarning: View {
    @EnvironmentObject var m: AppModel

    var body: some View {
        if m.globeWarningActive {
            StatusBanner(
                .warn, m.globeWarningText,
                actions: AnyView(
                    Button(m.t("globe.warn.button")) { m.open(AppModel.keyboardSettings) }
                        .buttonStyle(SecondaryButtonStyle())))
        }
    }
}

struct HotkeyRecorder: View {
    @EnvironmentObject var m: AppModel
    @State private var pulse = false

    var body: some View {
        Group { content }
            // Leaving the page must stop the recorder, or its monitor would
            // swallow keys in the whole app.
            .onDisappear { m.cancelRecording() }
    }

    @ViewBuilder private var content: some View {
        switch m.recorder {
        case .idle:
            Button { m.startRecording() } label: {
                Text(m.t("hotkey.record")).font(.system(size: 13.5, weight: .medium)).foregroundColor(Tok.text2)
                    .frame(maxWidth: .infinity, minHeight: 56)
                    .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.fill))
                    .overlay(
                        RoundedRectangle(cornerRadius: 12, style: .continuous)
                            .strokeBorder(Tok.text3, style: StrokeStyle(lineWidth: 1.2, dash: [5, 4])))
                    .contentShape(Rectangle())
            }.buttonStyle(.plain)
        case .recording(let held):
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 10) {
                    Circle().fill(Tok.danger).frame(width: 9, height: 9)
                        .opacity(pulse && !reduceMotion ? 0.35 : 1)
                    Text(m.t("hotkey.recording")).font(.system(size: 13.5, weight: .semibold)).foregroundColor(Tok.text)
                    Spacer(minLength: 8)
                    Keycaps(keys: heldCaps(held), lit: true, large: true)
                    Button(m.t("common.cancel")) { m.cancelRecording() }.buttonStyle(GhostButtonStyle())
                }
                .padding(.horizontal, 16).frame(maxWidth: .infinity, minHeight: 56)
                .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.accentSoft))
                .overlay(RoundedRectangle(cornerRadius: 12, style: .continuous).stroke(Tok.accent, lineWidth: 1.5))
                Text(m.t("hotkey.needsModifier")).font(.hint).foregroundColor(Tok.text2)
            }
            .onAppear {
                guard !reduceMotion else { return }
                withAnimation(.easeInOut(duration: 0.8).repeatForever(autoreverses: true)) { pulse = true }
            }
        case .conflict(_, let keys, let action):
            StatusBanner(
                .warn, title: m.t("hotkey.conflict.title", ["keys": keys]),
                m.t("hotkey.conflict.body", ["action": action]),
                actions: AnyView(
                    HStack(spacing: 10) {
                        Button(m.t("hotkey.conflict.another")) { m.startRecording() }
                            .buttonStyle(PrimaryButtonStyle())
                        Button(m.t("hotkey.conflict.useAnyway")) { m.useConflictAnyway() }
                            .buttonStyle(SecondaryButtonStyle())
                    }))
        case .saved(let h):
            HStack(spacing: 10) {
                Keycaps(keys: AppModel.caps(h), large: true)
                HStack(spacing: 6) {
                    Image(systemName: "checkmark").font(.system(size: 12, weight: .bold))
                    Text(m.t("hotkey.saved")).font(.system(size: 13.5, weight: .semibold))
                }.foregroundColor(Tok.success)
                Spacer(minLength: 8)
                Button(m.t("hotkey.clear")) { m.clearHotkey() }.buttonStyle(GhostButtonStyle())
            }
            .padding(.horizontal, 16).frame(maxWidth: .infinity, minHeight: 56)
            .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.successBg))
            .overlay(RoundedRectangle(cornerRadius: 12, style: .continuous).stroke(Tok.success, lineWidth: 1.5))
        }
    }

    private func heldCaps(_ mods: HotkeyMods) -> [String] {
        var out: [String] = []
        if mods.contains(.control) { out.append("⌃") }
        if mods.contains(.option) { out.append("⌥") }
        if mods.contains(.shift) { out.append("⇧") }
        if mods.contains(.command) { out.append("⌘") }
        return out
    }
}
