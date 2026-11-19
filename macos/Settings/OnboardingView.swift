// SPDX-License-Identifier: MIT OR Apache-2.0
// The welcome steps (docs/desktop-onboarding.md section 3).

import SwiftUI

struct OnboardingView: View {
    @EnvironmentObject var m: AppModel
    @State private var tryText = ""
    @State private var switchKey = "fn"

    var body: some View {
        VStack(spacing: 0) {
            topBar
            Spacer(minLength: 0)
            screen.padding(.horizontal, 48)
            Spacer(minLength: 0)
            bottomBar
        }
        .animation(.easeInOut(duration: 0.2), value: m.flow.step)
    }

    // MARK: Bars

    private var topBar: some View {
        HStack {
            if let p = m.flow.progress {
                HStack(spacing: 6) {
                    ForEach(1...p.total, id: \.self) { i in
                        Circle().fill(i <= p.index ? Brand.blue : Color.secondary.opacity(0.3))
                            .frame(width: 8, height: 8)
                    }
                }
                .accessibilityLabel(m.t("common.progress", ["step": "\(p.index)", "total": "\(p.total)"]))
            }
            Spacer()
            LanguagePill()
        }
        .padding(16)
    }

    private var bottomBar: some View {
        HStack {
            if m.flow.step != .welcome && m.flow.step != .done {
                Button(m.t("common.back")) { m.back() }.buttonStyle(.plain)
            }
            Spacer()
            if m.flow.step == .enable && !m.enabled {
                Button(m.t("common.skip")) { m.next() }.buttonStyle(.plain)
            }
            primary
        }
        .padding(20)
    }

    @ViewBuilder private var primary: some View {
        switch m.flow.step {
        case .welcome: Button(m.t("welcome.button")) { m.next() }.buttonStyle(PrimaryButtonStyle())
        case .privacy: Button(m.t("privacy.button")) { m.next() }.buttonStyle(PrimaryButtonStyle())
        case .enable:
            if m.enabled { Button(m.t("common.next")) { m.next() }.buttonStyle(PrimaryButtonStyle()) }
        case .shortcut, .tryIt, .tip:
            Button(m.t("common.next")) { m.next() }.buttonStyle(PrimaryButtonStyle())
        case .done: Button(m.t("done.button")) { m.finishOnboarding() }.buttonStyle(PrimaryButtonStyle())
        }
    }

    // MARK: Screens

    @ViewBuilder private var screen: some View {
        switch m.flow.step {
        case .welcome: welcome
        case .privacy: privacy
        case .enable: enable
        case .shortcut: shortcut
        case .tryIt: tryIt
        case .tip: tip
        case .done: done
        }
    }

    private func title(_ key: String, _ sub: String? = nil) -> some View {
        VStack(spacing: 6) {
            Text(m.t(key)).font(.system(size: 28, weight: .bold)).multilineTextAlignment(.center)
            if let sub = sub { Text(m.t(sub)).foregroundColor(.secondary) }
        }
    }

    private var welcome: some View {
        VStack(spacing: 18) {
            Buddy(size: 120).accessibilityLabel(m.t("welcome.mascotLabel"))
            title("welcome.title", "welcome.subtitle")
            HStack(spacing: 10) {
                Text(m.t("welcome.demo.latin")).font(.system(.body, design: .monospaced))
                Text("→")
                Text(m.t("welcome.demo.cyrillic")).font(.system(size: 20, weight: .bold))
            }
            Text(m.t("welcome.credit")).font(.caption).foregroundColor(.secondary)
        }
    }

    private var privacy: some View {
        VStack(spacing: 18) {
            title("privacy.title", "privacy.subtitle")
            let cards = ["offline", "noLogging", "noAccount", "passwords"]
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                ForEach(cards, id: \.self) { c in
                    Card {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(m.t("privacy.card.\(c).title")).font(.headline)
                            Text(m.t("privacy.card.\(c).body")).foregroundColor(.secondary)
                        }
                    }
                }
            }
            Button(m.t("privacy.githubPill")) { m.open("https://github.com/TengisCh/buuz") }
        }
    }

    private var enable: some View {
        VStack(spacing: 16) {
            title("enable.title", "enable.subtitle")
            Card {
                HStack {
                    Buddy(size: 36)
                    VStack(alignment: .leading) {
                        Text(m.t("enable.status.name")).font(.headline)
                        Text(m.t("enable.status.kind")).foregroundColor(.secondary)
                    }
                    Spacer()
                    Text(m.enabled ? m.t("enable.status.added") : m.t("enable.status.waiting"))
                }
            }
            if m.enabled {
                Text(m.t("enable.success")).font(.title2)
            } else {
                Button(m.t("enable.button")) { m.addBuuz() }.buttonStyle(PrimaryButtonStyle())
                Text(m.t("enable.note")).font(.caption).foregroundColor(.secondary)
                if m.addFailed { Text(m.t("enable.error")).foregroundColor(.red) }
                Card {
                    VStack(alignment: .leading, spacing: 6) {
                        Text(m.t("enable.manual.title")).font(.headline)
                        Text(m.t("enable.manual.path")).font(.callout)
                        Button(m.t("enable.manual.button")) { m.open(AppModel.keyboardSettings) }
                    }
                }
            }
        }
    }

    private var shortcut: some View {
        VStack(spacing: 14) {
            title("shortcut.title", "shortcut.subtitle")
            ForEach(["fn", "controlSpace", "capsLock"], id: \.self) { k in
                Button { switchKey = k } label: {
                    Card {
                        HStack {
                            Image(systemName: switchKey == k ? "largecircle.fill.circle" : "circle")
                            VStack(alignment: .leading) {
                                Text(m.t("shortcut.\(k).title")).font(.headline)
                                Text(m.t("shortcut.\(k).note")).foregroundColor(.secondary)
                            }
                            Spacer()
                            if k == "fn" { SoonPill(text: "⭐ " + m.t("common.recommended")) }
                        }
                    }
                }.buttonStyle(.plain)
            }
            Text(m.t("shortcut.manual.note")).font(.caption).foregroundColor(.secondary)
            Button(m.t("shortcut.manual.button")) { m.open(AppModel.keyboardSettings) }
        }
    }

    private let chips: [(String, String)] = [
        ("try.chip1", "try.chip1.keys"), ("try.chip2", "try.chip2.keys"), ("try.chip3", "try.chip3.keys"),
    ]

    private var tryIt: some View {
        VStack(spacing: 14) {
            title("try.title", "try.subtitle")
            Card {
                VStack(alignment: .leading, spacing: 8) {
                    Text(m.t("try.chat.name")).font(.caption).foregroundColor(.secondary)
                    Text(m.t("try.chat.message"))
                    // A real text field, so typing goes through the input method.
                    TextField(m.t("try.placeholder"), text: $tryText).textFieldStyle(.roundedBorder)
                }
            }
            Text(m.t("try.tasks")).font(.headline)
            HStack {
                ForEach(chips, id: \.0) { chip, keys in
                    let done = tryText.contains(m.t(chip))
                    VStack {
                        Text((done ? "✅ " : "") + m.t(chip)).font(.headline)
                        Text(m.t(keys)).font(.system(.caption, design: .monospaced)).foregroundColor(.secondary)
                    }
                    .padding(8).background(done ? Brand.dough.opacity(0.4) : Color.secondary.opacity(0.1))
                    .cornerRadius(10)
                }
            }
            if tryText.contains("унуудур") { Text(m.t("try.trap")) }
        }
    }

    private var tip: some View {
        VStack(spacing: 14) {
            title("tip.title", "tip.subtitle")
            Card {
                VStack(alignment: .leading, spacing: 4) {
                    Text(m.t("tip.harmony.title")).font(.headline)
                    Text(m.t("tip.harmony.body")).foregroundColor(.secondary)
                    ForEach(1...3, id: \.self) { Text(m.t("tip.harmony.example\($0)")) }
                }
            }
            Card {
                VStack(alignment: .leading, spacing: 4) {
                    Text(m.t("tip.force.title")).font(.headline)
                    Text(m.t("tip.force.body")).foregroundColor(.secondary)
                    Text(m.t("tip.force.keys")).font(.system(.body, design: .monospaced))
                    Text(m.t("tip.force.example"))
                }
            }
        }
    }

    private var done: some View {
        VStack(spacing: 16) {
            Buddy(size: 96)
            title("done.title", "done.subtitle")
            Card {
                VStack(alignment: .leading, spacing: 4) {
                    Text(m.t("done.recap.title")).font(.headline)
                    Text(m.t("done.recap.switchKey", ["key": m.t("shortcut.\(switchKey).title")]))
                    Text(m.t("done.recap.q"))
                    Text(m.t("done.recap.w"))
                }
            }
        }
    }
}

struct LanguagePill: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        Picker("", selection: $m.language) {
            Text(m.t("settings.general.language.mn")).tag(Language.mn)
            Text(m.t("settings.general.language.en")).tag(Language.en)
        }
        .pickerStyle(.segmented).frame(width: 170).labelsHidden()
    }
}
