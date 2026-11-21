// SPDX-License-Identifier: MIT OR Apache-2.0
// The tutorial: the rail, the page with its footer, the five setup steps, the
// eight lessons and Done (design/macos-app/spec.md section 7). Lessons pass
// only on the real Cyrillic text from the input method, never on Latin keys.

import SwiftUI

// MARK: Metrics

/// Short windows use smaller gaps so every step fits at 880 x 600.
struct TourMetrics {
    var gap: CGFloat = 20
    var rowHeight: CGFloat = 64
    var titleSize: CGFloat = 28
    var fieldHeight: CGFloat = 84
    var fieldFont: CGFloat = 26
    var buddy: CGFloat = 120
    var welcomeTitle: CGFloat = 34

    init(bodyHeight: CGFloat) {
        if bodyHeight < 570 { gap = 16; rowHeight = 56 }
        if bodyHeight < 520 {
            gap = 14; titleSize = 24; fieldHeight = 68; fieldFont = 22; buddy = 96; welcomeTitle = 24
        }
    }
}

private struct TourMetricsKey: EnvironmentKey { static let defaultValue = TourMetrics(bodyHeight: 1000) }
extension EnvironmentValues {
    var tour: TourMetrics {
        get { self[TourMetricsKey.self] }
        set { self[TourMetricsKey.self] = newValue }
    }
}

// MARK: Rail

struct TourRail: View {
    @EnvironmentObject var m: AppModel

    private var index: Int { TourStep.allCases.firstIndex(of: m.step) ?? 0 }

    private func row(_ s: TourStep, _ title: String, small: Bool) -> some View {
        let i = TourStep.allCases.firstIndex(of: s) ?? 0
        let isDone: Bool = {
            if let n = s.lessonNumber { return m.passed.contains(n) && s != m.step }
            return TourFlow.isDone(s, current: m.step, enabled: m.enabled)
        }()
        let current = s == m.step
        return Button { m.step = s } label: {
            HStack(spacing: 9) {
                CheckCircle(state: isDone ? .done : (current ? .current : .todo), size: small ? 14 : 16)
                Text(title).font(.system(size: small ? 12.5 : 13, weight: current ? .semibold : .regular))
                    .foregroundColor(Tok.text).lineLimit(1).truncationMode(.tail)
                Spacer(minLength: 0)
            }
            .padding(.horizontal, 10).frame(height: small ? 26 : 28)
            .background(RoundedRectangle(cornerRadius: 8, style: .continuous).fill(current ? Tok.selected : .clear))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(!(isDone || current))
        .accessibilityValue(isDone ? m.t("common.done") : "")
        .accessibilityAddTraits(current ? .isSelected : [])
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            LogoRow()
            Eyebrow(m.t("tour.setup")).padding(.horizontal, 10).padding(.bottom, 4)
            ForEach([TourStep.welcome, .privacy, .add, .key, .switchTo], id: \.self) { s in
                row(s, m.t("tour.step." + (s == .switchTo ? "switch" : s.rawValue)), small: false)
            }
            Eyebrow(m.t("tour.lessons")).padding(.horizontal, 10).padding(.top, 10).padding(.bottom, 4)
            ForEach(1...8, id: \.self) { n in
                row(TourStep.lesson(n) ?? .lesson1, m.t("lesson\(n).title"), small: true)
            }
            row(.done, m.t("tour.step.done"), small: true)
            Spacer(minLength: 6)
            Rectangle().fill(Tok.sep).frame(height: 1).padding(.horizontal, 10).padding(.bottom, 6)
            SidebarItem(symbol: "xmark", title: m.t("tour.skipAll")) { m.skipAll() }
        }
        .padding(.top, 48).padding(.leading, 12).padding(.trailing, 8).padding(.bottom, 12)
    }
}

// MARK: Page and footer

struct TourPage: View {
    @EnvironmentObject var m: AppModel

    var body: some View {
        VStack(spacing: 0) {
            GeometryReader { geo in
                let metrics = TourMetrics(bodyHeight: geo.size.height)
                ScrollView(.vertical) {
                    VStack(spacing: metrics.gap) { stepBody }
                        .environment(\.tour, metrics)
                        .frame(maxWidth: 580)
                        .padding(.horizontal, max(40, geo.size.width * 0.08))
                        .padding(.top, 28).padding(.bottom, 20)
                        .frame(maxWidth: .infinity)
                        .frame(minHeight: geo.size.height)
                }
            }
            TourFooter()
        }
        .background(m.step == .done ? AnyView(Confetti()) : AnyView(Color.clear))
    }

    @ViewBuilder private var stepBody: some View {
        switch m.step {
        case .welcome: WelcomeStep()
        case .privacy: PrivacyStep()
        case .add: AddStep()
        case .key: KeyStep()
        case .switchTo: SwitchStep()
        case .done: DoneStep()
        default: LessonStep(number: m.step.lessonNumber ?? 1)
        }
    }
}

struct TourFooter: View {
    @EnvironmentObject var m: AppModel

    private var count: String? {
        if let n = m.step.lessonNumber { return m.t("tour.lessonEyebrow", ["n": "\(n)", "total": "8"]) }
        if let n = m.step.setupNumber, m.step != .welcome { return m.t("tour.eyebrow", ["n": "\(n)", "total": "5"]) }
        return nil
    }

    private var primary: String {
        switch m.step {
        case .welcome: return m.t("welcome.button") + " →"
        case .privacy: return m.t("privacy.button")
        case .done: return m.t("done.button")
        case .lesson8: return m.t("lesson.finish")
        default:
            return m.step.lessonNumber != nil ? m.t("lesson.next") + " →" : m.t("common.next")
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            Rectangle().fill(Tok.sep).frame(height: 1)
            HStack(spacing: 8) {
                if m.step != .welcome {
                    Button("← " + m.t("common.back")) { m.back() }.buttonStyle(GhostButtonStyle())
                }
                Spacer(minLength: 8)
                if let c = count { Text(c).font(.system(size: 12.5)).foregroundColor(Tok.text3).padding(.trailing, 6) }
                if m.step.canSkip {
                    Button(m.t("tour.skipStep")) { m.next() }.buttonStyle(GhostButtonStyle())
                }
                Button(primary) { m.next() }
                    .buttonStyle(PrimaryButtonStyle())
                    .disabled(!m.stepPassed)
                    .keyboardShortcut(.defaultAction)
            }
            .padding(.horizontal, 20).frame(height: 59)
        }
        .frame(height: 60)
    }
}

// MARK: Header

struct TourHeader: View {
    let eyebrow: String
    let title: String
    var subtitle: String?
    @Environment(\.tour) private var metrics
    var body: some View {
        VStack(spacing: 6) {
            Eyebrow(eyebrow)
            Text(title).font(.rounded(metrics.titleSize, .heavy)).foregroundColor(Tok.text)
                .multilineTextAlignment(.center).fixedSize(horizontal: false, vertical: true)
            if let s = subtitle {
                Text(s).font(.system(size: 14.5)).foregroundColor(Tok.text2)
                    .multilineTextAlignment(.center).frame(maxWidth: 480)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .frame(maxWidth: .infinity)
    }
}

// MARK: Setup steps

struct WelcomeStep: View {
    @EnvironmentObject var m: AppModel
    @Environment(\.tour) private var metrics
    @State private var hop = false

    var body: some View {
        Buddy(size: metrics.buddy).offset(y: hop ? -14 : 0)
            .accessibilityElement(children: .ignore)
            .accessibilityLabel(m.t("welcome.mascotLabel"))
            .onAppear {
                guard !reduceMotion else { return }
                withAnimation(.easeOut(duration: 0.25)) { hop = true }
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) {
                    withAnimation(.easeIn(duration: 0.25)) { hop = false }
                }
            }
        Text(m.t("welcome.title")).font(.rounded(metrics.welcomeTitle, .heavy)).foregroundColor(Tok.text)
            .multilineTextAlignment(.center).fixedSize(horizontal: false, vertical: true)
        Text(m.t("welcome.subtitle")).font(.system(size: 14.5)).foregroundColor(Tok.text2)
        TypingDemo()
        Text(m.t("tour.welcome.time")).font(.hint).foregroundColor(Tok.text3)
    }
}

/// Types `sain baina uu` and its Cyrillic over and over. The two texts have
/// the same length. It uses fixed text only.
struct TypingDemo: View {
    private let latin = Array("sain baina uu"), cyr = Array("сайн байна уу")
    var body: some View {
        TimelineView(.periodic(from: Date(), by: 0.14)) { ctx in
            let n = reduceMotion ? latin.count : Int(ctx.date.timeIntervalSinceReferenceDate / 0.14) % (latin.count + 10)
            let c = min(n, latin.count)
            HStack(spacing: 12) {
                Text(String(latin.prefix(c))).font(.mono(15, .medium)).foregroundColor(Tok.text2)
                    .frame(width: 130, alignment: .trailing)
                Text("→").foregroundColor(Tok.text3)
                Text(String(cyr.prefix(c))).font(.rounded(20)).foregroundColor(Tok.text)
                    .frame(width: 150, alignment: .leading)
            }
            .padding(.horizontal, 20).frame(height: 52)
            .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.fill))
        }
    }
}

struct PrivacyStep: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        TourHeader(eyebrow: m.t("tour.eyebrow", ["n": "2", "total": "5"]), title: m.t("privacy.title"), subtitle: m.t("privacy.subtitle"))
        PrivacyPromises(minHeight: 52)
        StatusBanner(
            .ok, symbol: "wifi.slash", title: m.t("settings.privacy.noNetwork"), "",
            actions: AnyView(LinkButton(title: m.t("privacy.githubPill") + " →") { m.open(AppModel.repo) }))
    }
}

struct AddStep: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        let added = m.enabled
        TourHeader(
            eyebrow: m.t("tour.eyebrow", ["n": "3", "total": "5"]),
            title: added ? m.t("enable.success") : m.t("enable.title"), subtitle: m.t("enable.subtitle"))
        SettingsGroup {
            HStack(spacing: 14) {
                Buddy(size: 36, radius: 9)
                VStack(alignment: .leading, spacing: 2) {
                    Text(m.t("enable.status.name")).font(.rowTitle).foregroundColor(Tok.text)
                    Text(m.t("enable.status.kind") + " · " + m.t("tour.add.live")).font(.hint).foregroundColor(Tok.text2)
                        .fixedSize(horizontal: false, vertical: true)
                }.frame(maxWidth: .infinity, alignment: .leading)
                Pill(
                    text: noEmoji(m.t(added ? "enable.status.added" : "enable.status.waiting")),
                    kind: added ? .green : .warn)
                if !added {
                    Button(m.t("enable.button")) { m.addBuuz() }.buttonStyle(PrimaryButtonStyle())
                }
            }
            .padding(.vertical, 12).frame(minHeight: 60)
        }
        if added {
            ZStack {
                Circle().fill(Tok.successBg).frame(width: 64, height: 64)
                Image(systemName: "checkmark").font(.system(size: 26, weight: .bold)).foregroundColor(Tok.success)
            }
        } else {
            if m.addFailed {
                StatusBanner(.danger, m.t("enable.error"))
            } else {
                Text(m.t("enable.note")).font(.hint).foregroundColor(Tok.text2).multilineTextAlignment(.center)
            }
            Card {
                VStack(alignment: .leading, spacing: 10) {
                    Eyebrow(m.t("enable.manual.title"))
                    Crumbs()
                    Button(m.t("enable.manual.button")) { m.open(AppModel.keyboardSettings) }
                        .buttonStyle(m.addFailed ? AnyButtonStyle(PrimaryButtonStyle()) : AnyButtonStyle(SecondaryButtonStyle()))
                }
            }
        }
    }
}

/// Lets one `Button` use one of two styles.
struct AnyButtonStyle: ButtonStyle {
    private let make: (Configuration) -> AnyView
    init<S: ButtonStyle>(_ style: S) { make = { AnyView(style.makeBody(configuration: $0)) } }
    func makeBody(configuration: Configuration) -> some View { make(configuration) }
}

struct KeyStep: View {
    @EnvironmentObject var m: AppModel
    @Environment(\.tour) private var metrics
    var body: some View {
        TourHeader(eyebrow: m.t("tour.eyebrow", ["n": "4", "total": "5"]), title: m.t("tour.key.title"), subtitle: m.t("shortcut.subtitle"))
        SwitchKeyPicker(rowHeight: metrics.rowHeight)
    }
}

struct KeyboardTile: View {
    let buuz: Bool
    let name: String
    let now: Bool
    let nowText: String
    var body: some View {
        VStack(spacing: 8) {
            GlyphTile(buuz: buuz, name: name, size: 32, round: false)
            Text(name).font(.system(size: 13, weight: .semibold)).foregroundColor(Tok.text).lineLimit(1)
        }
        .frame(width: 140, height: 96)
        .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(now ? Tok.panel : Color.clear))
        .overlay(RoundedRectangle(cornerRadius: 12, style: .continuous).stroke(now ? Tok.accent : Tok.track, lineWidth: now ? 2 : 1))
        .overlay(alignment: .top) {
            if now {
                Text(nowText.uppercased()).font(.system(size: 9.5, weight: .heavy)).kerning(0.6)
                    .foregroundColor(.white).padding(.horizontal, 7).padding(.vertical, 2)
                    .background(Capsule().fill(Tok.accent)).offset(y: -8)
            }
        }
    }
}

struct SwitchStep: View {
    @EnvironmentObject var m: AppModel

    private func check(_ text: String, done: Bool, waiting: Bool) -> some View {
        HStack(spacing: 8) {
            ZStack {
                if done { CheckCircle(state: .done, size: 18) }
                else if waiting { ProgressView().controlSize(.small).scaleEffect(0.6) }
                else { CheckCircle(state: .todo, size: 18) }
            }.frame(width: 18, height: 18)
            Text(text).font(.system(size: 13, weight: .semibold)).foregroundColor(Tok.text)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 14).padding(.vertical, 10)
        .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(done ? Tok.successBg : Tok.fill))
    }

    var body: some View {
        let p = m.switchProgress
        let key = m.switchKeyLabel
        let other = m.otherKeyboardName
        let sub = (p.toBuuz && !p.done)
            ? m.t("tour.switch.backHint", ["key": key, "name": other])
            : m.t("tour.switch.subtitle", ["key": key])
        TourHeader(
            eyebrow: m.t("tour.eyebrow", ["n": "5", "total": "5"]),
            title: p.done ? m.t("tour.switch.done") : m.t("tour.switch.title"), subtitle: sub)
        HStack(spacing: 18) {
            KeyboardTile(buuz: false, name: other, now: !m.isBuuzCurrent, nowText: m.t("tour.switch.now"))
            VStack(spacing: 6) {
                Image(systemName: "arrow.left.arrow.right").font(.system(size: 14)).foregroundColor(Tok.text3)
                Keycaps(keys: m.switchKeyCaps)
            }
            KeyboardTile(buuz: true, name: "Buuz", now: m.isBuuzCurrent, nowText: m.t("tour.switch.now"))
        }
        .frame(maxWidth: .infinity).padding(.vertical, 18)
        .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.fill))
        HStack(spacing: 12) {
            check(m.t("tour.switch.toBuuz"), done: p.toBuuz, waiting: !p.toBuuz)
            check(m.t("tour.switch.back", ["name": other]), done: p.done, waiting: p.toBuuz && !p.done)
        }
        GlobeWarning()
        if let f = m.switchFailure { SwitchFailureBanner(failure: f) }
        if !p.toBuuz {
            StatusBanner(
                .neutral, m.t("tour.switch.menu"),
                actions: AnyView(
                    Button(m.t("tour.switch.toBuuz")) { m.switchToBuuz() }.buttonStyle(SecondaryButtonStyle())))
        }
    }
}

// MARK: Lessons

struct LessonStep: View {
    @EnvironmentObject var m: AppModel
    @Environment(\.tour) private var metrics
    let number: Int

    private var info: LessonInfo { LessonInfo.all[min(max(number, 1), 8) - 1] }

    /// The lesson needs an option that is off (classic mode or the switch).
    private var optionOff: Bool {
        guard let option = LessonRequirement.option(for: number) else { return false }
        return LessonRequirement.isMissing(lesson: number, classic: m.isClassic, flag: m.flag(option))
    }

    var body: some View {
        let state = m.checker.state(isBuuzCurrent: m.isBuuzCurrent)
        TourHeader(
            eyebrow: m.t("tour.lessonEyebrow", ["n": "\(number)", "total": "8"]),
            title: m.t("lesson\(number).title"), subtitle: m.t("lesson\(number).body"))
        task
        Text(m.t("lesson\(number).why")).font(.system(size: 12.5)).foregroundColor(Tok.text3)
            .multilineTextAlignment(.center).fixedSize(horizontal: false, vertical: true)
            .padding(.top, -8)
        LessonField(
            placeholder: m.t("lesson.field") + "…", fontSize: metrics.fieldFont, minHeight: metrics.fieldHeight,
            autofocus: true, stateColor: border(state),
            onChange: { m.lessonFeed($0) },
            onSubmit: { if m.checker.passed { m.next() } else { m.lessonSubmit() } }
        )
        .id(m.fieldReset)
        .padding(.top, -4)
        tools
        banner(state)
    }

    private func border(_ s: LessonState) -> Color? {
        switch s {
        case .success: return Tok.success
        case .wrongKeyboard, .latinOnly: return Tok.warn
        case .almost: return Tok.accent
        case .waiting: return nil
        }
    }

    private var task: some View {
        HStack(spacing: 8) {
            ForEach(Array(info.keys.enumerated()), id: \.offset) { _, k in
                if k.isKey { Keycap(text: k.text) } else { LatinChip(text: k.text, onPanel: true, size: 16) }
            }
            Image(systemName: "arrow.right").font(.system(size: 13)).foregroundColor(Tok.text3).padding(.horizontal, 4)
            Text(info.out).font(.lessonOutput).foregroundColor(Tok.text).lineLimit(1)
        }
        .padding(.vertical, 16).padding(.horizontal, 20)
        .frame(maxWidth: .infinity)
        .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.fill))
        .accessibilityElement(children: .combine)
    }

    private var tools: some View {
        HStack(spacing: 8) {
            KeyboardChip()
            if number == 6 || number == 7 {
                let subs = m.checker.subChecks
                ForEach(0..<2, id: \.self) { i in
                    SubCheck(text: m.t("lesson\(number).step\(i + 1)"), done: i < subs.count && subs[i])
                }
            }
            Spacer(minLength: 8)
            Button(m.t("lesson.clear")) { m.lessonClear() }.buttonStyle(GhostButtonStyle())
        }
        .padding(.top, -6)
    }

    @ViewBuilder private func banner(_ s: LessonState) -> some View {
        if let f = m.switchFailure, s == .wrongKeyboard {
            SwitchFailureBanner(failure: f)
        } else if optionOff && s != .success, let option = LessonRequirement.option(for: number) {
            StatusBanner(
                .warn, m.t("lesson\(number).off"),
                actions: AnyView(
                    Button(m.t("lesson8.turnOn")) {
                        if m.isClassic { m.setClassic(false) }
                        m.setFlag(option, true)
                    }.buttonStyle(SecondaryButtonStyle())))
        } else {
            switch s {
            case .success:
                StatusBanner(
                    .ok, title: m.t("lesson.success"), m.t("lesson\(number).success"),
                    trailing: AnyView(HoppingBuddy()))
            case .wrongKeyboard:
                StatusBanner(
                    .warn, title: m.t("lesson.wrong.title", ["name": m.currentKeyboard?.name ?? "—"]),
                    m.t("lesson.wrong.body", ["key": m.switchKeyLabel]),
                    trailing: AnyView(Keycaps(keys: m.switchKeyCaps)),
                    actions: AnyView(
                        Button(m.t("tour.switch.toBuuz")) { m.switchToBuuz() }.buttonStyle(SecondaryButtonStyle())))
                GlobeWarning()
            case .latinOnly:
                StatusBanner(.info, title: m.t("lesson.latin.title"), m.t("lesson.latin.body"))
            case .almost(let got, let own):
                if own {
                    StatusBanner(.info, symbol: "lightbulb", title: m.t("lesson.close.title"), m.t("lesson\(number)." + "close"))
                } else {
                    CloseBanner(got: got, target: info.out)
                }
            case .waiting:
                StatusBanner(.neutral, m.t("lesson.waiting"))
            }
        }
    }
}

/// Shows why "Switch to Buuz" failed and the manual steps.
struct SwitchFailureBanner: View {
    @EnvironmentObject var m: AppModel
    let failure: SwitchResult
    var body: some View {
        let reason: String
        switch failure {
        case .failed(let status): reason = m.t("switch.failed.body", ["code": "\(status)"])
        default: reason = m.t("switch.failed.notEnabled")
        }
        return StatusBanner(
            .warn, title: m.t("switch.failed.title"), reason + " " + m.t("switch.failed.path"),
            actions: AnyView(
                Button(m.t("enable.manual.button")) { m.open(AppModel.keyboardSettings) }
                    .buttonStyle(SecondaryButtonStyle())))
    }
}

/// "Almost". It shows what the user got, with the wrong letters in red and
/// underlined.
struct CloseBanner: View {
    @EnvironmentObject var m: AppModel
    let got: String
    let target: String
    var body: some View {
        let parts = m.t("lesson.close.body", ["got": "\u{0001}"]).components(separatedBy: "\u{0001}")
        var gotText = Text("")
        for item in LessonChecker.mismatch(got: got, target: target) {
            let piece = Text(String(item.char))
            gotText = gotText + (item.ok ? piece.bold() : piece.bold().underline().foregroundColor(Tok.danger))
        }
        let line = Text(parts.first ?? "") + gotText + Text(parts.count > 1 ? parts[1] : "")
        return HStack(alignment: .top, spacing: 12) {
            Image(systemName: "lightbulb").font(.system(size: 18, weight: .semibold)).foregroundColor(Tok.accent)
                .frame(width: 22, height: 22)
            VStack(alignment: .leading, spacing: 2) {
                Text(m.t("lesson.close.title")).font(.system(size: 13.5, weight: .bold)).foregroundColor(Tok.accent)
                line.font(.system(size: 13)).foregroundColor(Tok.text).fixedSize(horizontal: false, vertical: true)
            }
            .frame(maxWidth: .infinity, alignment: .leading).padding(.top, 2)
        }
        .padding(.vertical, 14).padding(.horizontal, 16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.accentSoft))
    }
}

struct SubCheck: View {
    let text: String
    let done: Bool
    var body: some View {
        HStack(spacing: 5) {
            CheckCircle(state: done ? .done : .todo, size: 12)
            Text(text).font(.system(size: 11.5, weight: .semibold)).foregroundColor(done ? Tok.success : Tok.text2)
                .lineLimit(1)
        }
        .padding(.horizontal, 8).frame(height: 22)
        .background(Capsule().fill(done ? Tok.successBg : Tok.fill2))
    }
}

/// A 40 pt Buddy that hops twice. Without motion it stays still.
struct HoppingBuddy: View {
    @State private var up = false
    var body: some View {
        Buddy(size: 40, radius: 10).offset(y: up ? -6 : 0)
            .onAppear {
                guard !reduceMotion else { return }
                withAnimation(.easeInOut(duration: 0.22).repeatCount(4, autoreverses: true)) { up = true }
            }
    }
}

// MARK: Done

struct DoneStep: View {
    @EnvironmentObject var m: AppModel
    @Environment(\.tour) private var metrics

    var body: some View {
        Buddy(size: metrics.buddy > 96 ? 96 : metrics.buddy)
        VStack(spacing: 6) {
            Text(m.t("done.title")).font(.rounded(metrics.titleSize, .heavy)).foregroundColor(Tok.text)
            Text(m.t("done.subtitle")).font(.system(size: 14.5)).foregroundColor(Tok.text2)
                .multilineTextAlignment(.center)
        }
        SettingsGroup {
            SettingsRow(m.t("tour.step.key"), hint: "Buuz ⇄ " + m.otherKeyboardName, minHeight: 52) {
                Keycaps(keys: m.switchKeyCaps)
            }
            SettingsRow(m.t("tour.done.tricks"), hint: m.t("cheat.esc"), minHeight: 52) {
                Keycaps(keys: ["q", "w", "'", "esc"])
            }
            SettingsRow(
                m.t("tour.done.lessons", ["done": "\(m.passed.count)", "total": "8"]),
                hint: m.t("tour.done.replay"), minHeight: 52)
        }
        if AppModel.loginAvailable {
            HStack(spacing: 10) {
                BuuzSwitch(isOn: Binding(get: { m.loginAtFinish }, set: { m.loginAtFinish = $0 }))
                Text(m.t("done.login")).font(.system(size: 13.5)).foregroundColor(Tok.text)
            }
        }
    }
}

/// Confetti behind the Done step. None with Reduce Motion.
struct Confetti: View {
    private let colors: [Color] = [Tok.ember, Tok.amber, Tok.accent, Color(red: 1, green: 0.56, blue: 0.56), Tok.success, Color(red: 0.48, green: 0.35, blue: 0.97)]
    var body: some View {
        if reduceMotion {
            EmptyView()
        } else {
            TimelineView(.animation) { ctx in
                GeometryReader { geo in
                    let t = ctx.date.timeIntervalSinceReferenceDate
                    ForEach(0..<40, id: \.self) { i in
                        piece(i, time: t, size: geo.size)
                    }
                }
            }
            .allowsHitTesting(false)
        }
    }

    /// One piece. The maths uses typed steps, so the compiler on the CI
    /// runner can type-check it in reasonable time.
    private func piece(_ i: Int, time t: Double, size: CGSize) -> some View {
        let speed: Double = 60.0 + Double(i % 5) * 18.0
        let span: Double = Double(size.height) + 40.0
        let travelled: Double = t * speed + Double(i) * 53.0
        let y: Double = travelled.truncatingRemainder(dividingBy: span) - 20.0
        let column: CGFloat = CGFloat((i * 37) % 100) / 100.0
        let x: CGFloat = size.width * column
        let angle: Double = Double(i) * 23.0 + t * 90.0
        let color: Color = colors[i % colors.count]
        return Rectangle()
            .fill(color)
            .frame(width: 6, height: 10)
            .rotationEffect(.degrees(angle))
            .position(x: x, y: CGFloat(y))
            .opacity(0.8)
    }
}
