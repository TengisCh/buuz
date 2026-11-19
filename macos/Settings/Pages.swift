// SPDX-License-Identifier: MIT OR Apache-2.0
// The three pages of the main window: Home, Lessons and Help
// (design/macos-app/spec.md section 6). Each is a `PageColumn`.

import SwiftUI

/// The fixed data of the eight lessons: the keys to press and the result.
struct LessonInfo {
    struct Key { var text: String; var isKey: Bool }
    var number: Int
    var keys: [Key]
    var out: String

    private static func w(_ s: String) -> Key { Key(text: s, isKey: false) }
    private static func k(_ s: String) -> Key { Key(text: s, isKey: true) }

    static let all: [LessonInfo] = [
        LessonInfo(number: 1, keys: [w("sain"), w("baina"), w("uu")], out: "сайн байна уу"),
        LessonInfo(number: 2, keys: [w("gertoo"), w("teruu")], out: "гэртөө тэрүү"),
        LessonInfo(number: 3, keys: [w("qndqr"), w("wg")], out: "өндөр үг"),
        LessonInfo(number: 4, keys: [w("mor'"), w("hon'")], out: "морь хонь"),
        LessonInfo(number: 5, keys: [w("nom"), k("⏎ Enter")], out: "ном"),
        LessonInfo(number: 6, keys: [w("tov"), k("␣ Space"), k("⌫")], out: "тов"),
        LessonInfo(number: 7, keys: [w("zoom"), k("esc"), k("␣ Space")], out: "zoom"),
        LessonInfo(number: 8, keys: [w("bi"), w("zgr"), w("bn")], out: "би зүгээр байна"),
    ]
}

// MARK: Home

struct HeroCard: View {
    let title: String
    let message: String
    let button: String
    var warn = false
    var onClose: (() -> Void)?
    let action: () -> Void
    @EnvironmentObject var m: AppModel
    @Environment(\.columnWidth) private var columnWidth

    var body: some View {
        let showBuddy = columnWidth >= 500
        let shape = RoundedRectangle(cornerRadius: 14, style: .continuous)
        ZStack(alignment: .topTrailing) {
            Tok.hero
            RadialGradient(
                colors: [(warn ? Tok.amber : Tok.ember).opacity(0.7), .clear],
                center: .topTrailing, startRadius: 0, endRadius: 300)
            RadialGradient(
                colors: [(warn ? Tok.ember : Tok.amber).opacity(0.5), .clear],
                center: .bottomTrailing, startRadius: 0, endRadius: 220)
            if showBuddy {
                Buddy(size: 122, radius: 24).rotationEffect(.degrees(-6))
                    .offset(x: -22, y: 40)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
            }
            VStack(alignment: .leading, spacing: 6) {
                Text(title).font(.heroTitle).foregroundColor(.white).fixedSize(horizontal: false, vertical: true)
                Text(message).font(.system(size: 13.5)).foregroundColor(Color.white.opacity(0.78))
                    .fixedSize(horizontal: false, vertical: true)
                Button(action: action) { Text(button) }
                    .buttonStyle(PrimaryButtonStyle(large: true)).padding(.top, 8)
            }
            .padding(.trailing, showBuddy ? 150 : 0)
            .padding(.horizontal, 28)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
            if let close = onClose {
                Button(action: close) {
                    Image(systemName: "xmark").font(.system(size: 10, weight: .bold)).foregroundColor(.white)
                        .frame(width: 24, height: 24).background(Circle().fill(Color.white.opacity(0.14)))
                }
                .buttonStyle(.plain).padding(12)
                .accessibilityLabel(m.t("common.close")).help(m.t("common.close"))
            }
        }
        .frame(height: 152)
        .clipShape(shape)
    }
}

struct StatusTile<Value: View>: View {
    let label: String
    let action: () -> Void
    @ViewBuilder var value: Value
    @State private var hover = false
    var body: some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 2) {
                value.frame(minHeight: 24, alignment: .leading)
                Text(label).font(.hint).foregroundColor(Tok.text2).lineLimit(1)
            }
            .padding(.vertical, 14).padding(.horizontal, 16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(hover ? Tok.fill2 : Tok.fill))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain).onHover { hover = $0 }
    }
}

struct CheatEntry {
    var keys: [String]
    var latin: String
    var out: String
    var desc: String
    /// The config switch that turns this row on. `nil` for always.
    var flag: String?
    var smart: Bool
}

struct HomeView: View {
    @EnvironmentObject var m: AppModel
    @State private var searching = false
    @State private var query = ""

    private var entries: [CheatEntry] {
        [
            CheatEntry(keys: ["q"], latin: "qndqr", out: "өндөр", desc: "cheat.q", flag: nil, smart: false),
            CheatEntry(keys: ["w"], latin: "wg", out: "үг", desc: "cheat.w", flag: nil, smart: false),
            CheatEntry(keys: ["'"], latin: "mor'", out: "морь", desc: "cheat.soft", flag: nil, smart: false),
            CheatEntry(keys: ["\""], latin: "\"", out: "ъ", desc: "cheat.hard", flag: nil, smart: false),
            CheatEntry(keys: ["o", "u"], latin: "gertoo", out: "гэртөө", desc: "cheat.harmony", flag: "smart_vowels", smart: true),
            CheatEntry(keys: ["v", "v"], latin: "svv", out: "сүү", desc: "cheat.vv", flag: "double_v", smart: true),
            CheatEntry(keys: ["t", "s"], latin: "tsai", out: "цай", desc: "cheat.ts", flag: "habit_keys", smart: true),
            CheatEntry(keys: ["z", "h"], latin: "zhims", out: "жимс", desc: "cheat.zh", flag: "habit_keys", smart: true),
            CheatEntry(keys: ["esc"], latin: "zoom ⎋", out: "zoom", desc: "cheat.esc", flag: "keep_latin", smart: true),
            CheatEntry(keys: ["⌫"], latin: "tov ␣ ⌫", out: "тов", desc: "cheat.backspace", flag: nil, smart: true),
            CheatEntry(keys: ["b", "n"], latin: "bn", out: "байна", desc: "cheat.abbr", flag: "abbreviations", smart: true),
        ]
    }

    private var shown: [CheatEntry] {
        let classic = m.isClassic
        let q = query.lowercased().trimmingCharacters(in: .whitespaces)
        return entries.filter { e in
            if e.smart && classic { return false }
            if let f = e.flag, e.smart, !m.flag(f, default: f != "autocorrect") { return false }
            if q.isEmpty { return true }
            return e.keys.joined(separator: " ").lowercased().contains(q) || e.latin.lowercased().contains(q)
                || e.out.lowercased().contains(q) || m.t(e.desc).lowercased().contains(q)
        }
    }

    var body: some View {
        PageColumn {
            let hour = Calendar.current.component(.hour, from: Date())
            PageHeader(m.t(Greeting.key(hour: hour)) + " 👋", m.t("home.subtitle"))
            VStack(alignment: .leading, spacing: 12) {
                hero
                tiles
            }
            TitledSection(m.t("home.try.title")) {
                LessonField(placeholder: m.t("home.try.placeholder"), fontSize: 17, minHeight: 48, radius: 12)
            }
            .overlay(alignment: .topTrailing) { KeyboardChip().offset(y: -5) }
            TitledSection(m.t("home.cheat.title")) { cheat }
                .overlay(alignment: .topTrailing) {
                    IconButton(symbol: "magnifyingglass", label: m.t("home.cheat.find"), size: 28) {
                        searching.toggle()
                        if !searching { query = "" }
                    }.offset(y: -6)
                }
        }
    }

    @ViewBuilder private var hero: some View {
        if !m.enabled {
            VStack(spacing: 12) {
                HeroCard(
                    title: m.t("home.hero.off.title"), message: m.t("home.hero.off.body"),
                    button: m.t("enable.button"), warn: true, onClose: nil) { m.addBuuz() }
                if m.addFailed { StatusBanner(.danger, m.t("enable.error")) }
            }
        } else if !m.heroHidden {
            if m.nextLesson != nil {
                HeroCard(
                    title: m.t("home.hero.lessons.title"), message: m.t("home.hero.lessons.body"),
                    button: m.t("home.hero.lessons.continue", ["done": "\(m.passed.count)", "total": "8"]),
                    onClose: { m.heroHidden = true }
                ) { m.startTour(at: TourStep.lesson(m.nextLesson ?? 1) ?? .lesson1, returnTo: .home) }
            } else {
                HeroCard(
                    title: m.t("home.hero.tip.title"), message: m.t("home.hero.tip.body"),
                    button: m.t("home.hero.tip.button"), onClose: { m.heroHidden = true }
                ) { m.replayLesson(7) }
            }
        }
    }

    private var tiles: some View {
        let cols = Array(repeating: GridItem(.flexible(), spacing: 12), count: 3)
        return LazyVGrid(columns: cols, spacing: 12) {
            StatusTile(label: m.t("home.card.keyboard")) { m.openSettings(.shortcuts) } value: {
                let buuz = m.isBuuzCurrent
                let name = buuz ? "Buuz" : (m.currentKeyboard?.name ?? m.t("home.card.notAdded"))
                HStack(spacing: 8) {
                    GlyphTile(buuz: buuz, name: name, size: 22, round: false)
                    Text(name).font(.tileValue).foregroundColor(Tok.text).lineLimit(1)
                }
            }
            StatusTile(label: m.t("home.card.switchKey")) { m.openSettings(.shortcuts) } value: {
                Text(m.switchKeyLabel).font(.tileValue).foregroundColor(Tok.text).lineLimit(1)
            }
            StatusTile(label: m.t("home.card.mode")) { m.openSettings(.typing) } value: {
                Text(m.t(m.isClassic ? "typing.mode.classic" : "typing.mode.smart"))
                    .font(.tileValue).foregroundColor(Tok.text).lineLimit(1)
            }
        }
    }

    private var cheat: some View {
        VStack(alignment: .leading, spacing: 12) {
            if searching {
                TextField(m.t("home.cheat.find"), text: $query)
                    .textFieldStyle(.plain).font(.system(size: 13.5))
                    .padding(.horizontal, 12).frame(height: 36)
                    .background(RoundedRectangle(cornerRadius: 8, style: .continuous).fill(Tok.fill))
            }
            SettingsGroup {
                ForEach(Array(shown.enumerated()), id: \.offset) { _, e in CheatRow(entry: e) }
            }
        }
    }
}

struct CheatRow: View {
    @EnvironmentObject var m: AppModel
    @Environment(\.columnWidth) private var columnWidth
    let entry: CheatEntry
    var body: some View {
        VStack(spacing: 0) {
            Rectangle().fill(Tok.sep).frame(height: 1)
            HStack(spacing: 12) {
                Keycaps(keys: entry.keys).frame(width: 64, alignment: .leading)
                HStack(spacing: 8) {
                    LatinChip(text: entry.latin)
                    Image(systemName: "arrow.right").font(.system(size: 11)).foregroundColor(Tok.text3)
                    Text(entry.out).font(.system(size: 14, weight: .bold)).foregroundColor(Tok.text).lineLimit(1)
                }
                Spacer(minLength: 8)
                if columnWidth >= 470 {
                    Text(m.t(entry.desc)).font(.hint).foregroundColor(Tok.text2)
                        .lineLimit(1).truncationMode(.tail)
                }
            }
            .frame(minHeight: 51)
        }
    }
}

// MARK: Lessons

struct LessonsView: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        PageColumn {
            PageHeader(m.t("help.lessons"), m.t("home.hero.lessons.body")) {
                Button { m.startTour(at: .lesson1, returnTo: .lessons) } label: {
                    Label(m.t("help.replayAll"), systemImage: "play.fill")
                }.buttonStyle(PrimaryButtonStyle())
            }
            SettingsGroup {
                ForEach(LessonInfo.all, id: \.number) { l in
                    LessonRow(info: l, done: m.passed.contains(l.number))
                }
            }
        }
    }
}

struct LessonRow: View {
    @EnvironmentObject var m: AppModel
    let info: LessonInfo
    let done: Bool
    var body: some View {
        VStack(spacing: 0) {
            Rectangle().fill(Tok.sep).frame(height: 1)
            HStack(spacing: 14) {
                ZStack {
                    Circle().fill(done ? Tok.success : Tok.fill2)
                    if done {
                        Image(systemName: "checkmark").font(.system(size: 11, weight: .heavy)).foregroundColor(Tok.onSuccess)
                    } else {
                        Text("\(info.number)").font(.system(size: 12, weight: .bold)).foregroundColor(Tok.text2)
                    }
                }.frame(width: 22, height: 22)
                VStack(alignment: .leading, spacing: 2) {
                    Text(m.t("lesson\(info.number).title")).font(.rowTitle).foregroundColor(Tok.text).lineLimit(1)
                    HStack(spacing: 6) {
                        ForEach(Array(info.keys.enumerated()), id: \.offset) { _, k in
                            Text(k.text).font(.mono(12, .medium)).foregroundColor(Tok.text2).lineLimit(1)
                        }
                        Text("→").font(.hint).foregroundColor(Tok.text3)
                        Text(info.out).font(.system(size: 12.5, weight: .semibold)).foregroundColor(Tok.text)
                            .lineLimit(1)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                Button { m.replayLesson(info.number) } label: {
                    Label(m.t("help.replay"), systemImage: "arrow.counterclockwise")
                }.buttonStyle(GhostButtonStyle()).fixedSize()
            }
            .padding(.vertical, 10).frame(minHeight: 59)
        }
    }
}

// MARK: Help

/// The System Settings path to add Buuz by hand.
struct Crumbs: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        // Wraps to a new line when the column is narrow.
        Text(m.t("enable.manual.path")).font(.system(size: 12.5, weight: .medium))
            .foregroundColor(Tok.text2).fixedSize(horizontal: false, vertical: true)
    }
}

struct HelpView: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        PageColumn {
            PageHeader(m.t("help.title"), m.t("help.subtitle"))
            TitledSection(m.t("help.missing")) {
                if m.addFailed && !m.enabled { StatusBanner(.danger, m.t("enable.error")) }
                Card {
                    VStack(alignment: .leading, spacing: 14) {
                        Crumbs()
                        HStack(spacing: 10) {
                            Button(m.t("enable.button")) { m.addBuuz() }.buttonStyle(PrimaryButtonStyle())
                            Button(m.t("enable.manual.button")) { m.open(AppModel.keyboardSettings) }
                                .buttonStyle(SecondaryButtonStyle())
                        }
                    }
                }
            }
            SettingsGroup {
                SettingsRow(m.t("settings.feedback.title"), hint: m.t("settings.feedback.body")) {
                    Button(m.t("settings.feedback.button")) { m.open(AppModel.repo + "/issues/new") }
                        .buttonStyle(SecondaryButtonStyle())
                }
                SettingsRow(m.t("general.tutorial"), hint: m.t("general.tutorial.hint")) {
                    Button(m.t("settings.shortcuts.rerunTutorial")) { m.startTour() }
                        .buttonStyle(SecondaryButtonStyle())
                }
            }
        }
    }
}
