// SPDX-License-Identifier: MIT OR Apache-2.0
// Shared views of the Buuz app: Buddy, buttons, cards, keycaps, banners,
// keyboard chip, settings rows and the page column
// (design/macos-app/spec.md sections 4.1 and 5).

import SwiftUI

struct Buddy: View {
    var size: CGFloat = 96
    var radius: CGFloat = 0
    var body: some View {
        if let url = Bundle.main.url(forResource: "buuz-buddy-256", withExtension: "png"),
            let image = NSImage(contentsOf: url)
        {
            Image(nsImage: image).resizable().interpolation(.high)
                .frame(width: size, height: size)
                .clipShape(RoundedRectangle(cornerRadius: radius, style: .continuous))
                .accessibilityHidden(true)
        } else {
            Text("🥟").font(.system(size: size * 0.7)).accessibilityHidden(true)
        }
    }
}

struct Wordmark: View {
    @Environment(\.colorScheme) private var scheme
    var height: CGFloat = 44
    var body: some View {
        let name = scheme == .dark ? "buuz-wordmark-light" : "buuz-wordmark"
        if let url = Bundle.main.url(forResource: name, withExtension: "png"), let image = NSImage(contentsOf: url) {
            Image(nsImage: image).resizable().interpolation(.high).scaledToFit().frame(height: height)
        } else {
            Text("Buuz").font(.rounded(28, .heavy))
        }
    }
}

// MARK: Page column (spec 4.1)

/// Every app page and every settings page is built with this. One vertical
/// scroll view, one centred column, no side cards.
struct PageColumn<Content: View>: View {
    var maxWidth: CGFloat = 720
    var top: CGFloat = 40
    var bottom: CGFloat = 56
    var minSide: CGFloat = 48
    var sideFraction: CGFloat = 0.09
    var gap: CGFloat = 32
    @ViewBuilder var content: Content

    var body: some View {
        GeometryReader { geo in
            let side = max(minSide, geo.size.width * sideFraction)
            let column = min(maxWidth, max(0, geo.size.width - side * 2))
            ScrollView(.vertical) {
                VStack(alignment: .leading, spacing: gap) { content }
                    .environment(\.columnWidth, column)
                    .frame(maxWidth: maxWidth)
                    .padding(.horizontal, side)
                    .padding(.top, top).padding(.bottom, bottom)
                    .frame(maxWidth: .infinity)
            }
        }
    }
}

// MARK: Buttons

/// Khadag Blue with a 2 pt pressed edge. One for each screen.
struct PrimaryButtonStyle: ButtonStyle {
    var large = false
    @Environment(\.isEnabled) private var enabled
    func makeBody(configuration: Configuration) -> some View {
        let h: CGFloat = large ? 38 : 32
        configuration.label
            .font(.system(size: large ? 14 : 13, weight: .semibold))
            .foregroundColor(enabled ? .white : Tok.text3)
            .padding(.horizontal, large ? 18 : 14)
            .frame(height: h)
            .background(RoundedRectangle(cornerRadius: 8, style: .continuous).fill(enabled ? Tok.accent : Tok.fill2))
            .background(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .fill(enabled ? Tok.accentEdge : Tok.track)
                    .offset(y: configuration.isPressed ? 0 : 2))
            .offset(y: configuration.isPressed ? 2 : 0)
            .padding(.bottom, 2)
            .contentShape(Rectangle())
    }
}

struct SecondaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 13, weight: .semibold))
            .foregroundColor(Tok.text)
            .padding(.horizontal, 14).frame(height: 32)
            .background(RoundedRectangle(cornerRadius: 8, style: .continuous).fill(Tok.btn2))
            .overlay(RoundedRectangle(cornerRadius: 8, style: .continuous).stroke(Tok.btn2Line, lineWidth: 1))
            .opacity(configuration.isPressed ? 0.7 : 1)
            .contentShape(Rectangle())
    }
}

struct GhostButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        GhostBody(configuration: configuration)
    }
    private struct GhostBody: View {
        let configuration: Configuration
        @State private var hover = false
        var body: some View {
            configuration.label
                .font(.system(size: 13, weight: .semibold))
                .foregroundColor(hover ? Tok.text : Tok.text2)
                .padding(.horizontal, 10).frame(height: 32)
                .background(RoundedRectangle(cornerRadius: 8, style: .continuous).fill(hover ? Tok.hover : .clear))
                .contentShape(Rectangle())
                .onHover { hover = $0 }
                .opacity(configuration.isPressed ? 0.7 : 1)
        }
    }
}

/// A plain icon button, at least 28 pt for the tap target.
struct IconButton: View {
    let symbol: String
    let label: String
    var size: CGFloat = 30
    let action: () -> Void
    @State private var hover = false
    var body: some View {
        Button(action: action) {
            Image(systemName: symbol).font(.system(size: 16, weight: .regular)).foregroundColor(Tok.text2)
                .frame(width: size, height: size)
                .background(RoundedRectangle(cornerRadius: 8, style: .continuous).fill(hover ? Tok.hover : .clear))
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain).onHover { hover = $0 }
        .help(label).accessibilityLabel(label)
    }
}

struct LinkButton: View {
    let title: String
    let action: () -> Void
    var body: some View {
        Button(action: action) {
            Text(title).font(.system(size: 12.5, weight: .semibold)).foregroundColor(Tok.accent)
        }.buttonStyle(.plain)
    }
}

// MARK: Surfaces

/// `fill`, radius 12, padding 20, no border and no shadow.
struct Card<Content: View>: View {
    var padding: CGFloat = 20
    let content: Content
    init(padding: CGFloat = 20, @ViewBuilder _ content: () -> Content) {
        self.padding = padding
        self.content = content()
    }
    var body: some View {
        content
            .padding(padding)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.fill))
    }
}

struct Eyebrow: View {
    let text: String
    var color: Color = Tok.text3
    init(_ text: String, color: Color = Tok.text3) { self.text = text; self.color = color }
    var body: some View {
        Text(text.uppercased()).font(.system(size: 11, weight: .bold)).kerning(0.8).foregroundColor(color)
    }
}

struct SectionTitle: View {
    let text: String
    init(_ text: String) { self.text = text }
    var body: some View {
        Text(text).font(.sectionTitle).foregroundColor(Tok.text2).fixedSize(horizontal: false, vertical: true)
    }
}

/// A section: its title, 12 pt of space, then the content.
struct TitledSection<Content: View>: View {
    let title: String
    var hint: String?
    let content: Content
    init(_ title: String, hint: String? = nil, @ViewBuilder _ content: () -> Content) {
        self.title = title; self.hint = hint; self.content = content()
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            SectionTitle(title)
            if let h = hint { Text(h).font(.hint).foregroundColor(Tok.text2).padding(.top, -6) }
            content
        }
    }
}

struct Pill: View {
    enum Kind { case plain, blue, green, warn }
    let text: String
    var kind: Kind = .plain
    var body: some View {
        let (fg, bg): (Color, Color) = {
            switch kind {
            case .plain: return (Tok.text2, Tok.fill2)
            case .blue: return (Tok.accent, Tok.accentSoft)
            case .green: return (Tok.success, Tok.successBg)
            case .warn: return (Tok.warn, Tok.warnBg)
            }
        }()
        Text(text).font(.system(size: 11, weight: .bold)).foregroundColor(fg)
            .padding(.horizontal, 8).padding(.vertical, 2)
            .background(Capsule().fill(bg))
            .fixedSize()
    }
}

struct Keycap: View {
    let text: String
    var lit = false
    var large = false
    var body: some View {
        let r: CGFloat = 6
        Text(text)
            .font(.mono(large ? 13.5 : 12))
            .foregroundColor(lit ? .white : Tok.text)
            .padding(.horizontal, large ? 10 : 7)
            .frame(minWidth: large ? 32 : 24, minHeight: large ? 32 : 24)
            .background(
                ZStack {
                    RoundedRectangle(cornerRadius: r, style: .continuous)
                        .fill(lit ? Tok.accentEdge : Tok.keyEdge).offset(y: 2.5)
                    RoundedRectangle(cornerRadius: r, style: .continuous)
                        .fill(lit ? Tok.accent : Tok.keyBg)
                    RoundedRectangle(cornerRadius: r, style: .continuous)
                        .stroke(lit ? Tok.accentEdge : Tok.keyLine, lineWidth: 1)
                }
            )
            .padding(.bottom, 2.5)
            .fixedSize()
    }
}

/// Monospaced Latin input such as `qndqr`.
struct LatinChip: View {
    let text: String
    var onPanel = false
    var size: CGFloat = 13.5
    var body: some View {
        Text(text).font(.mono(size)).foregroundColor(Tok.text)
            .padding(.horizontal, 8).padding(.vertical, 3)
            .background(RoundedRectangle(cornerRadius: 6, style: .continuous).fill(onPanel ? Tok.panel : Tok.fill2))
            .fixedSize()
    }
}

/// A key or a list of keys, for example "o u" or "esc".
struct Keycaps: View {
    let keys: [String]
    var lit = false
    var large = false
    var body: some View {
        HStack(spacing: 4) {
            ForEach(Array(keys.enumerated()), id: \.offset) { Keycap(text: $0.element, lit: lit, large: large) }
        }
    }
}

// MARK: Switch and segmented control

struct BuuzSwitch: View {
    @Binding var isOn: Bool
    var dimmed = false
    var body: some View {
        Button { isOn.toggle() } label: {
            ZStack(alignment: isOn ? .trailing : .leading) {
                Capsule().fill(isOn ? Tok.accent : Tok.track).frame(width: 36, height: 20)
                Circle().fill(Color.white).frame(width: 16, height: 16).padding(2)
                    .shadow(color: .black.opacity(0.2), radius: 1, y: 1)
            }
            .animation(reduceMotion ? nil : .easeOut(duration: 0.12), value: isOn)
            .opacity(dimmed ? 0.45 : 1)
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(.isButton)
        .accessibilityValue(isOn ? "1" : "0")
    }
}

struct Segmented<T: Hashable>: View {
    let options: [(T, String)]
    @Binding var selection: T
    var body: some View {
        HStack(spacing: 2) {
            ForEach(Array(options.enumerated()), id: \.offset) { _, opt in
                let on = opt.0 == selection
                Button { selection = opt.0 } label: {
                    Text(opt.1).font(.system(size: 12.5, weight: .semibold))
                        .foregroundColor(on ? Tok.text : Tok.text2)
                        .padding(.horizontal, 12).frame(height: 26)
                        .background(RoundedRectangle(cornerRadius: 6, style: .continuous).fill(on ? Tok.panel : .clear))
                        .contentShape(Rectangle())
                }.buttonStyle(.plain)
            }
        }
        .padding(2)
        .background(RoundedRectangle(cornerRadius: 8, style: .continuous).fill(Tok.track))
        .fixedSize()
    }
}

// MARK: Banner

struct StatusBanner: View {
    enum Kind { case neutral, info, ok, warn, danger }
    let kind: Kind
    var symbol: String?
    var title: String?
    let message: String
    var trailing: AnyView?
    var actions: AnyView?

    init(_ kind: Kind, symbol: String? = nil, title: String? = nil, _ message: String,
         trailing: AnyView? = nil, actions: AnyView? = nil) {
        self.kind = kind; self.symbol = symbol; self.title = title; self.message = message
        self.trailing = trailing; self.actions = actions
    }

    static func defaultSymbol(_ k: Kind) -> String {
        switch k {
        case .neutral, .info: return "info.circle"
        case .ok: return "checkmark"
        case .warn, .danger: return "exclamationmark.triangle"
        }
    }

    private var colors: (fg: Color, bg: Color) {
        switch kind {
        case .neutral: return (Tok.text2, Tok.fill)
        case .info: return (Tok.accent, Tok.accentSoft)
        case .ok: return (Tok.success, Tok.successBg)
        case .warn: return (Tok.warn, Tok.warnBg)
        case .danger: return (Tok.danger, Tok.dangerBg)
        }
    }

    var body: some View {
        let c = colors
        HStack(alignment: .top, spacing: 12) {
            Image(systemName: symbol ?? StatusBanner.defaultSymbol(kind))
                .font(.system(size: 18, weight: .semibold)).foregroundColor(c.fg)
                .frame(width: 22, height: 22)
            VStack(alignment: .leading, spacing: 2) {
                if let t = title {
                    Text(t).font(.system(size: 13.5, weight: .bold)).foregroundColor(c.fg)
                        .fixedSize(horizontal: false, vertical: true)
                }
                if !message.isEmpty {
                    Text(message).font(.system(size: 13)).foregroundColor(Tok.text)
                        .fixedSize(horizontal: false, vertical: true)
                }
                if let a = actions { a.padding(.top, 8) }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, 2)
            if let t = trailing { t.fixedSize() }
        }
        .padding(.vertical, 14).padding(.horizontal, 16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(c.bg))
    }
}

// MARK: Keyboard chip

struct KeyboardChip: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        let buuz = m.isBuuzCurrent
        KeyboardChipView(buuz: buuz, name: buuz ? "Buuz" : (m.currentKeyboard?.name ?? "—"), live: true)
    }
}

/// A chip for one keyboard. The dot pulses slowly when `live`, and not with
/// Reduce Motion.
struct KeyboardChipView: View {
    let buuz: Bool
    let name: String
    var live = true
    @State private var pulse = false
    var body: some View {
        HStack(spacing: 7) {
            GlyphTile(buuz: buuz, name: name, size: 18)
            Text(name).font(.system(size: 12.5, weight: .semibold)).foregroundColor(Tok.text).lineLimit(1)
            Circle().fill(buuz ? Tok.success : Tok.warn).frame(width: 6, height: 6)
                .opacity(live ? (pulse && !reduceMotion ? 0.4 : 1) : 0.3)
        }
        .padding(.leading, 4).padding(.trailing, 10).frame(height: 26)
        .background(Capsule().fill(Tok.fill2))
        .fixedSize()
        .onAppear {
            guard !reduceMotion else { return }
            withAnimation(.easeInOut(duration: 1.1).repeatForever(autoreverses: true)) { pulse = true }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(name)
    }
}

/// The glyph of a keyboard: Б on ember for Buuz, the first letter on gray.
struct GlyphTile: View {
    let buuz: Bool
    let name: String
    var size: CGFloat = 26
    var round = true
    var body: some View {
        let shape = RoundedRectangle(cornerRadius: round ? size / 2 : size * 0.28, style: .continuous)
        Text(buuz ? "Б" : String(name.prefix(1)).uppercased())
            .font(.rounded(size * 0.56, .heavy)).foregroundColor(.white)
            .frame(width: size, height: size)
            .background(shape.fill(buuz ? Tok.ember : Tok.text3))
    }
}

// MARK: Settings rows

/// `fill`, radius 12, padding 0 and 20. A hairline sits between the rows.
struct SettingsGroup<Content: View>: View {
    let content: Content
    init(@ViewBuilder _ content: () -> Content) { self.content = content() }
    var body: some View {
        VStack(spacing: 0) { content }
            .padding(.top, -1)  // hides the hairline above the first row
            .padding(.horizontal, 20)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(Tok.fill))
            .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}

/// Title and hint at the left, the control at the right. The control keeps its
/// natural width. The text takes the rest and wraps.
struct SettingsRow<Trailing: View>: View {
    var title: String
    var hint: String?
    var soon: String?
    var minHeight: CGFloat = 60
    var monoHint = false
    let trailing: Trailing

    init(_ title: String, hint: String? = nil, soon: String? = nil, minHeight: CGFloat = 60,
         monoHint: Bool = false, @ViewBuilder trailing: () -> Trailing) {
        self.title = title; self.hint = hint; self.soon = soon
        self.minHeight = minHeight; self.monoHint = monoHint; self.trailing = trailing()
    }

    var body: some View {
        VStack(spacing: 0) {
            Rectangle().fill(Tok.sep).frame(height: 1)
            HStack(spacing: 16) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(.rowTitle).foregroundColor(Tok.text)
                        .fixedSize(horizontal: false, vertical: true)
                    if let h = hint {
                        if monoHint {
                            Text(h).font(.mono(11.5, .regular)).foregroundColor(Tok.text2)
                                .lineLimit(1).truncationMode(.middle)
                        } else {
                            Text(h).font(.hint).foregroundColor(Tok.text2)
                                .fixedSize(horizontal: false, vertical: true)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                if let s = soon { Pill(text: s) } else { trailing.fixedSize() }
            }
            .padding(.vertical, 12)
            .frame(minHeight: minHeight - 1)
            .opacity(soon != nil ? 0.55 : 1)
        }
    }
}

extension SettingsRow where Trailing == EmptyView {
    init(_ title: String, hint: String? = nil, soon: String? = nil, minHeight: CGFloat = 60) {
        self.init(title, hint: hint, soon: soon, minHeight: minHeight) { EmptyView() }
    }
}

struct PageHeader<Trailing: View>: View {
    let title: String
    let subtitle: String?
    var size: CGFloat = 24
    let trailing: Trailing
    init(_ title: String, _ subtitle: String? = nil, size: CGFloat = 24, @ViewBuilder trailing: () -> Trailing) {
        self.title = title; self.subtitle = subtitle; self.size = size; self.trailing = trailing()
    }
    var body: some View {
        HStack(alignment: .bottom, spacing: 16) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title).font(.rounded(size)).foregroundColor(Tok.text)
                    .fixedSize(horizontal: false, vertical: true)
                if let s = subtitle {
                    Text(s).font(.system(size: 13.5)).foregroundColor(Tok.text2)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            trailing.fixedSize()
        }
    }
}

extension PageHeader where Trailing == EmptyView {
    init(_ title: String, _ subtitle: String? = nil, size: CGFloat = 24) {
        self.init(title, subtitle, size: size) { EmptyView() }
    }
}
