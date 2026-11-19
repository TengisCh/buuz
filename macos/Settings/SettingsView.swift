// SPDX-License-Identifier: MIT OR Apache-2.0
// The settings window (docs/desktop-onboarding.md section 4). Import and
// export stays hidden in v1, until the portable config import ships.

import SwiftUI

enum SettingsSection: String, CaseIterable, Identifiable {
    case home, typing, shortcuts, privacy, general, feedback, about
    var id: String { rawValue }
}

struct SettingsView: View {
    @EnvironmentObject var m: AppModel
    @State private var section: SettingsSection = .home
    @State private var tryText = ""

    var body: some View {
        HStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 4) {
                HStack { Buddy(size: 28); Text(m.t("settings.windowTitle")).font(.headline) }
                    .padding(.bottom, 8)
                ForEach(SettingsSection.allCases) { s in
                    Button { section = s } label: {
                        Text(m.t("settings.\(s.rawValue)"))
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.vertical, 6).padding(.horizontal, 8)
                            .background(section == s ? Brand.blue.opacity(0.15) : Color.clear)
                            .cornerRadius(8)
                    }.buttonStyle(.plain)
                }
                Spacer()
            }
            .padding(16).frame(width: 200)
            Divider()
            ScrollView { content.padding(24).frame(maxWidth: .infinity, alignment: .leading) }
        }
    }

    @ViewBuilder private var content: some View {
        switch section {
        case .home: home
        case .typing: typing
        case .shortcuts: shortcuts
        case .privacy: privacy
        case .general: general
        case .feedback: feedback
        case .about: about
        }
    }

    private func heading(_ key: String) -> some View {
        Text(m.t(key)).font(.system(size: 24, weight: .bold)).padding(.bottom, 8)
    }

    private var home: some View {
        VStack(alignment: .leading, spacing: 14) {
            heading("settings.home.greeting")
            Card {
                HStack {
                    Text(m.enabled ? m.t("settings.home.statusOn") : m.t("settings.home.statusOff"))
                    Spacer()
                    if !m.enabled { Button(m.t("settings.home.fix")) { m.addBuuz() } }
                }
            }
            TextField(m.t("settings.home.tryBox"), text: $tryText).textFieldStyle(.roundedBorder)
            Card {
                VStack(alignment: .leading, spacing: 4) {
                    Text(m.t("settings.home.cheatSheet")).font(.headline)
                    Text(m.t("tip.force.keys")).font(.system(.body, design: .monospaced))
                    Text(m.t("tip.harmony.example2"))
                }
            }
            Text(m.t("settings.home.noStats")).font(.caption).foregroundColor(.secondary)
        }
    }

    private func flagRow(_ title: String, _ hint: String, key: String, def: Bool) -> some View {
        Toggle(isOn: Binding(get: { m.flag(key, default: def) }, set: { m.setFlag(key, $0) })) {
            VStack(alignment: .leading) {
                Text(m.t(title))
                Text(m.t(hint)).font(.caption).foregroundColor(.secondary)
            }
        }
    }

    private func soonRow(_ title: String, _ hint: String) -> some View {
        HStack {
            VStack(alignment: .leading) {
                Text(m.t(title))
                Text(m.t(hint)).font(.caption).foregroundColor(.secondary)
            }
            Spacer()
            SoonPill(text: m.t("common.comingSoon"))
        }.opacity(0.6)
    }

    private var typing: some View {
        VStack(alignment: .leading, spacing: 14) {
            heading("settings.typing.title")
            flagRow("settings.typing.smartVowels", "settings.typing.smartVowels.hint", key: "smart_vowels", def: true)
            flagRow("settings.typing.abbreviations", "settings.typing.abbreviations.hint", key: "abbreviations", def: true)
            soonRow("settings.typing.autoCapitals", "settings.typing.autoCapitals.hint")
            soonRow("settings.typing.customKeys", "settings.typing.customKeys.hint")
            soonRow("settings.typing.perApp", "settings.typing.perApp.hint")
        }
    }

    private var shortcuts: some View {
        VStack(alignment: .leading, spacing: 14) {
            heading("settings.shortcuts.title")
            Text(m.t("settings.shortcuts.switchKey")).font(.headline)
            Text(m.t("settings.shortcuts.switchKey.hint")).foregroundColor(.secondary)
            Button(m.t("settings.shortcuts.openSystem")) { m.open(AppModel.keyboardSettings) }
            Button(m.t("settings.shortcuts.rerunTutorial")) { m.restartOnboarding() }
        }
    }

    private var privacy: some View {
        VStack(alignment: .leading, spacing: 14) {
            heading("settings.privacy.title")
            Text(m.t("settings.privacy.promises")).font(.headline)
            ForEach(["offline", "noLogging", "noAccount", "passwords"], id: \.self) {
                Text(m.t("privacy.card.\($0).title"))
            }
            Text(m.t("settings.privacy.noNetwork"))
            Button(m.t("settings.privacy.source")) { m.open("https://github.com/TengisCh/buuz") }
        }
    }

    private var general: some View {
        VStack(alignment: .leading, spacing: 14) {
            heading("settings.general.title")
            Text(m.t("settings.general.language")).font(.headline)
            LanguagePill()
            Text(m.t("settings.general.theme")).font(.headline)
            Picker("", selection: $m.dark) {
                Text(m.t("settings.general.theme.light")).tag(false)
                Text(m.t("settings.general.theme.dark")).tag(true)
            }.pickerStyle(.segmented).frame(width: 220).labelsHidden()
            Button(m.t("settings.general.showOnboarding")) { m.restartOnboarding() }
        }
    }

    private var feedback: some View {
        VStack(alignment: .leading, spacing: 14) {
            heading("settings.feedback.title")
            Text(m.t("settings.feedback.body"))
            Button(m.t("settings.feedback.button")) { m.open("https://github.com/TengisCh/buuz/issues/new") }
        }
    }

    private var about: some View {
        let version = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""
        return VStack(alignment: .leading, spacing: 14) {
            heading("settings.about.title")
            Buddy(size: 64)
            Text(m.t("settings.about.version", ["version": version]))
            Text(m.t("settings.about.credit"))
            Button(m.t("settings.about.original")) { m.open(m.t("settings.about.original.url")) }
            Text(m.t("settings.about.license")).foregroundColor(.secondary)
            Text(m.t("settings.about.licenseCore")).foregroundColor(.secondary)
        }
    }
}
