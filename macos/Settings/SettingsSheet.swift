// SPDX-License-Identifier: MIT OR Apache-2.0
// The settings sheet: a modal sheet over the window with its own list
// (design/macos-app/spec.md section 6.4). It opens from Settings in the
// sidebar, from the status tiles and with Command-comma.

import SwiftUI

struct SettingsSheet: View {
    @EnvironmentObject var m: AppModel

    var body: some View {
        GeometryReader { geo in
            let w = min(760, geo.size.width - 64)
            let h = min(560, geo.size.height - 56)
            ZStack {
                Tok.scrim.onTapGesture { m.closeSettings() }
                HStack(spacing: 0) {
                    list.frame(width: 184)
                    ZStack(alignment: .topTrailing) {
                        PageColumn(maxWidth: .infinity, top: 32, bottom: 40, minSide: 36, sideFraction: 0, gap: 28) {
                            switch m.sheet ?? .general {
                            case .general: GeneralPage()
                            case .typing: TypingPage()
                            case .shortcuts: ShortcutsPage()
                            case .privacy: PrivacyPage()
                            case .about: AboutPage()
                            }
                        }
                        .id(m.sheet)
                        IconButton(symbol: "xmark", label: m.t("common.close")) { m.closeSettings() }
                            .padding(.top, 14).padding(.trailing, 14)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(Tok.panel)
                }
                .frame(width: max(w, 0), height: max(h, 0))
                .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
                .shadow(color: .black.opacity(0.3), radius: 30, y: 12)
                Button("") { m.closeSettings() }.keyboardShortcut(.cancelAction).opacity(0).frame(width: 0, height: 0)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    private var list: some View {
        let items: [(SettingsPage, String, String)] = [
            (.general, "slider.horizontal.3", "settings.general.title"),
            (.typing, "pencil.line", "nav.typing"),
            (.shortcuts, "command", "nav.shortcuts"),
            (.privacy, "lock", "nav.privacy"),
            (.about, "info.circle", "nav.about"),
        ]
        return VStack(alignment: .leading, spacing: 2) {
            Eyebrow(m.t("nav.settings")).padding(.horizontal, 10).padding(.bottom, 10)
            ForEach(items, id: \.0) { item in
                SidebarItem(symbol: item.1, title: m.t(item.2), selected: m.sheet == item.0) {
                    m.openSettings(item.0)
                }
            }
            Spacer(minLength: 0)
            Text(m.t("settings.about.version", ["version": AppModel.version]))
                .font(.system(size: 12)).foregroundColor(Tok.text3).padding(.horizontal, 10)
        }
        .padding(.vertical, 20).padding(.horizontal, 10)
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .background(Tok.frame)
    }
}

// MARK: General

struct GeneralPage: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        PageHeader(m.t("settings.general.title"), size: 22)
        SettingsGroup {
            SettingsRow(m.t("settings.general.language")) {
                Segmented(
                    options: [(Language.mn, m.t("settings.general.language.mn")), (.en, m.t("settings.general.language.en"))],
                    selection: Binding(get: { m.language }, set: { m.language = $0 }))
            }
            SettingsRow(m.t("settings.general.theme")) {
                Segmented(
                    options: [
                        (Theme.light, noEmoji(m.t("settings.general.theme.light"))),
                        (.dark, noEmoji(m.t("settings.general.theme.dark"))),
                        (.system, m.t("general.theme.system")),
                    ],
                    selection: Binding(get: { m.theme }, set: { m.theme = $0 }))
            }
        }
        SettingsGroup {
            if AppModel.loginAvailable {
                SettingsRow(m.t("settings.general.openAtLogin")) {
                    BuuzSwitch(isOn: Binding(get: { m.launchAtLogin }, set: { m.setLaunchAtLogin($0) }))
                }
            }
            SettingsRow(m.t("settings.general.checkUpdates"), hint: m.t("general.updates.hint")) {
                VStack(alignment: .trailing, spacing: 6) {
                    Button(m.t("settings.general.checkUpdates")) { m.checkUpdates() }
                        .buttonStyle(SecondaryButtonStyle())
                    UpdateResult()
                }
            }
            SettingsRow(m.t("general.tutorial"), hint: m.t("general.tutorial.hint")) {
                Button(m.t("settings.shortcuts.rerunTutorial")) { m.startTour() }
                    .buttonStyle(SecondaryButtonStyle())
            }
        }
    }
}

// MARK: Typing

struct TypingPage: View {
    @EnvironmentObject var m: AppModel

    private func flagRow(_ key: String, _ cfg: String, on: Bool) -> some View {
        let classic = m.isClassic
        return SettingsRow(m.t(key), hint: m.t(key + ".hint")) {
            BuuzSwitch(
                isOn: Binding(get: { m.flag(cfg, default: on) }, set: { m.setFlag(cfg, $0) }),
                dimmed: classic
            ).disabled(classic)
        }
    }

    var body: some View {
        PageHeader(m.t("settings.typing.title"), m.t("typing.subtitle"), size: 22)
        VStack(alignment: .leading, spacing: 12) {
            SettingsGroup {
                SettingsRow(m.t("typing.mode"), hint: m.t("typing.mode.hint")) {
                    Segmented(
                        options: [(false, m.t("typing.mode.smart")), (true, m.t("typing.mode.classic"))],
                        selection: Binding(get: { m.isClassic }, set: { m.setClassic($0) }))
                }
            }
            if m.isClassic { StatusBanner(.info, m.t("typing.classicNote")) }
        }
        TitledSection(m.t("typing.group.vowels")) {
            SettingsGroup {
                flagRow("typing.smartVowels", "smart_vowels", on: true)
                flagRow("typing.autocorrect", "autocorrect", on: false)
            }
        }
        TitledSection(m.t("typing.group.keys")) {
            SettingsGroup {
                flagRow("typing.doubleV", "double_v", on: true)
                flagRow("typing.habitKeys", "habit_keys", on: true)
                flagRow("typing.abbreviations", "abbreviations", on: true)
            }
        }
        TitledSection(m.t("typing.group.latin")) {
            SettingsGroup {
                flagRow("typing.keepLatin", "keep_latin", on: true)
                flagRow("typing.autoLatin", "auto_latin", on: true)
            }
        }
        TitledSection(m.t("typing.preview"), hint: m.t("typing.preview.hint")) {
            SettingsGroup {
                previewRow("gertoo", m.preview("gertoo"))
                previewRow("svv", m.preview("svv"))
                previewRow("tsai", m.preview("tsai"))
                previewRow("bn", m.preview("bn"))
                previewRow("zoom ⎋", m.preview("zoom", escape: true))
            }
        }
        TitledSection(m.t("typing.group.soon")) {
            SettingsGroup {
                ForEach(["settings.typing.autoCapitals", "settings.typing.customKeys", "settings.typing.perApp"], id: \.self) { k in
                    SettingsRow(noEmoji(m.t(k)), hint: m.t(k + ".hint"), soon: m.t("common.comingSoon"))
                }
            }
        }
    }

    private func previewRow(_ latin: String, _ result: String) -> some View {
        VStack(spacing: 0) {
            Rectangle().fill(Tok.sep).frame(height: 1)
            HStack {
                Text(latin).font(.mono(13.5, .medium)).foregroundColor(Tok.text2)
                Spacer(minLength: 12)
                Text(result).font(.system(size: 14, weight: .bold)).foregroundColor(Tok.text).lineLimit(1)
            }
            .frame(minHeight: 43)
        }
    }
}

// MARK: Shortcuts

struct ShortcutsPage: View {
    @EnvironmentObject var m: AppModel

    var body: some View {
        PageHeader(m.t("settings.shortcuts.title"), m.t("shortcuts.switchKey.hint"), size: 22)
        TitledSection(m.t("settings.shortcuts.switchKey")) { SwitchKeyPicker() }
        SettingsGroup {
            SettingsRow(m.t("shortcuts.test.title"), hint: m.t("shortcuts.test.body", ["key": m.switchKeyLabel])) {
                HStack(spacing: 8) {
                    KeyboardChipView(buuz: false, name: m.otherKeyboardName, live: !m.isBuuzCurrent)
                    Image(systemName: "arrow.left.arrow.right").font(.system(size: 12)).foregroundColor(Tok.text3)
                    KeyboardChipView(buuz: true, name: "Buuz", live: m.isBuuzCurrent)
                }
            }
            if m.roundTrip.done {
                SettingsRow(noEmoji(m.t("shortcut.works"))) {
                    Image(systemName: "checkmark.circle.fill").foregroundColor(Tok.success)
                }
            }
        }
        TitledSection(m.t("shortcuts.typingKeys")) {
            SettingsGroup {
                SettingsRow(m.t("shortcuts.key.commit"), minHeight: 52) { Keycaps(keys: ["Space", "⏎", "."]) }
                SettingsRow(m.t("shortcuts.key.esc"), minHeight: 52) { Keycaps(keys: ["esc"]) }
                SettingsRow(m.t("shortcuts.key.backspace"), minHeight: 52) { Keycaps(keys: ["⌫"]) }
            }
        }
    }
}

// MARK: Privacy

struct PrivacyPromises: View {
    @EnvironmentObject var m: AppModel
    var minHeight: CGFloat = 60
    var body: some View {
        SettingsGroup {
            ForEach(["offline", "noLogging", "noAccount", "passwords"], id: \.self) { c in
                SettingsRow(
                    noEmoji(m.t("privacy.card.\(c).title")), hint: m.t("privacy.card.\(c).body"),
                    minHeight: minHeight)
            }
        }
    }
}

struct PrivacyPage: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        PageHeader(m.t("settings.privacy.title"), m.t("privacy.subtitle"), size: 22)
        StatusBanner(
            .ok, symbol: "wifi.slash", title: m.t("settings.privacy.noNetwork"), "",
            actions: AnyView(LinkButton(title: m.t("settings.privacy.source") + " →") { m.open(AppModel.repo) }))
        TitledSection(m.t("settings.privacy.promises")) { PrivacyPromises() }
        TitledSection(m.t("privacy.keeps.title")) {
            SettingsGroup {
                SettingsRow(m.t("privacy.keeps.config"), hint: m.configPath, monoHint: true) {
                    Button(m.t("privacy.keeps.reveal")) { m.revealConfig() }.buttonStyle(SecondaryButtonStyle())
                }
                SettingsRow(m.t("privacy.keeps.prefs"), minHeight: 52)
                SettingsRow(m.t("privacy.keeps.never"), minHeight: 52)
            }
        }
    }
}

// MARK: About

struct AboutPage: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        VStack(spacing: 8) {
            Wordmark(height: 44)
            Text(m.t("settings.about.version", ["version": AppModel.version]))
                .font(.system(size: 12.5)).foregroundColor(Tok.text2)
        }
        .frame(maxWidth: .infinity).padding(.vertical, 4)
        SettingsGroup {
            SettingsRow(m.t("about.website"), hint: m.t("settings.about.website")) {
                LinkButton(title: m.t("setup.open") + " →") { m.open("https://" + m.t("settings.about.website")) }
            }
            SettingsRow(m.t("settings.about.license"), hint: m.t("settings.about.licenseCore"))
            SettingsRow(m.t("settings.about.notices")) {
                Button(m.t("settings.about.notices.open")) { m.openNotices() }
                    .buttonStyle(SecondaryButtonStyle())
            }
            SettingsRow(m.t("settings.feedback.title"), hint: m.t("settings.feedback.body")) {
                Button(m.t("settings.feedback.button")) { m.open(AppModel.repo + "/issues/new") }
                    .buttonStyle(SecondaryButtonStyle())
            }
        }
    }
}
