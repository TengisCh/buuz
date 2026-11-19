// SPDX-License-Identifier: MIT OR Apache-2.0
// The window: frame, sidebar, top icons and the content panel
// (design/macos-app/spec.md section 4 and 5).

import AppKit
import SwiftUI

/// Sets the window style once: hidden title, full-size content, minimum size.
struct WindowConfigurator: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView { ConfigView() }
    func updateNSView(_ nsView: NSView, context: Context) {}

    private final class ConfigView: NSView {
        private var done = false
        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            guard !done, let w = window else { return }
            done = true
            w.titleVisibility = .hidden
            w.titlebarAppearsTransparent = true
            w.styleMask.insert(.fullSizeContentView)
            w.isMovableByWindowBackground = true
            w.minSize = NSSize(width: 880, height: 600)
            let hadFrame = w.setFrameAutosaveName("BuuzMain")
            // The first launch opens at 1000 x 680. Later launches restore the
            // saved frame. This works on macOS 12 and later.
            if !hadFrame {
                w.setFrame(NSRect(x: 0, y: 0, width: 1000, height: 680), display: true)
                w.center()
            }
        }
    }
}

struct AppShell: View {
    @EnvironmentObject var m: AppModel

    var body: some View {
        ZStack {
            Tok.frame
            HStack(spacing: 0) {
                if m.tourActive { TourRail().frame(width: 200) } else { Sidebar().frame(width: 200) }
                ContentPanel {
                    if m.tourActive {
                        TourPage()
                    } else {
                        switch m.page {
                        case .home: HomeView()
                        case .lessons: LessonsView()
                        case .help: HelpView()
                        }
                    }
                }
            }
            TopIcons()
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
                .padding(.top, 7).padding(.trailing, 12)
            if m.sheet != nil && !m.tourActive { SettingsSheet() }
        }
        .ignoresSafeArea()
        .background(WindowConfigurator())
    }
}

/// One large rounded panel. It fills all the room that is left.
struct ContentPanel<Content: View>: View {
    @ViewBuilder var content: Content
    var body: some View {
        let shape = RoundedRectangle(cornerRadius: 14, style: .continuous)
        content
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(shape.fill(Tok.panel))
            .clipShape(shape)
            .overlay(shape.stroke(Tok.panelLine, lineWidth: 1))
            .padding(.top, 44).padding(.trailing, 12).padding(.bottom, 12)
    }
}

// MARK: Sidebar

struct SidebarItem: View {
    let symbol: String
    let title: String
    var selected = false
    let action: () -> Void
    @State private var hover = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                Image(systemName: symbol).font(.system(size: 14.5)).frame(width: 18)
                    .foregroundColor(selected ? Tok.text : Tok.text2)
                Text(title).font(.system(size: 13.5, weight: selected ? .semibold : .medium))
                    .foregroundColor(Tok.text).lineLimit(1)
                Spacer(minLength: 0)
            }
            .padding(.horizontal, 10).frame(height: 32)
            .background(RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(selected ? Tok.selected : (hover ? Tok.hover : .clear)))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain).onHover { hover = $0 }
        .accessibilityAddTraits(selected ? .isSelected : [])
    }
}

struct LogoRow: View {
    var body: some View {
        HStack(spacing: 8) {
            Buddy(size: 26, radius: 7)
            Text("Buuz").font(.rounded(18, .heavy)).foregroundColor(Tok.text)
            Text("2.0").font(.system(size: 10.5, weight: .bold)).foregroundColor(Tok.text3)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 10).padding(.bottom, 14)
    }
}

struct Sidebar: View {
    @EnvironmentObject var m: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            LogoRow()
            VStack(spacing: 2) {
                SidebarItem(symbol: "house", title: m.t("nav.home"), selected: m.page == .home && m.sheet == nil) {
                    m.closeSettings(); m.page = .home
                }
                SidebarItem(symbol: "book", title: m.t("nav.lessons"), selected: m.page == .lessons && m.sheet == nil) {
                    m.closeSettings(); m.page = .lessons
                }
            }
            Spacer(minLength: 8)
            if !m.setupDone { SetupCard().padding(.bottom, 10) }
            Rectangle().fill(Tok.sep).frame(height: 1).padding(.horizontal, 10).padding(.bottom, 6)
            VStack(spacing: 2) {
                SidebarItem(symbol: "gearshape", title: m.t("nav.settings"), selected: m.sheet != nil) {
                    m.openSettings()
                }
                SidebarItem(symbol: "questionmark.circle", title: m.t("nav.help"), selected: m.page == .help && m.sheet == nil) {
                    m.closeSettings(); m.page = .help
                }
            }
        }
        .padding(.top, 48).padding(.leading, 12).padding(.trailing, 8).padding(.bottom, 12)
    }
}

// MARK: Setup card

struct CheckCircle: View {
    enum State { case done, current, todo }
    let state: State
    var size: CGFloat = 14
    var body: some View {
        ZStack {
            switch state {
            case .done:
                Circle().fill(Tok.success)
                Image(systemName: "checkmark").font(.system(size: size * 0.55, weight: .heavy)).foregroundColor(Tok.onSuccess)
            case .current:
                Circle().stroke(Tok.accent, lineWidth: 1.6)
                Circle().fill(Tok.accent).frame(width: size * 0.42, height: size * 0.42)
            case .todo:
                Circle().stroke(Tok.text3, lineWidth: 1.4)
            }
        }
        .frame(width: size, height: size)
    }
}

struct SetupCard: View {
    @EnvironmentObject var m: AppModel
    @Environment(\.colorScheme) private var scheme

    var body: some View {
        let rows = m.setupRows
        let done = SetupChecklist.doneCount(rows)
        let current = SetupChecklist.current(rows)
        let ids = ["setup.add", "setup.key", "setup.switch", "setup.lessons"]
        VStack(alignment: .leading, spacing: 0) {
            Button { m.setupFolded.toggle() } label: {
                HStack(spacing: 6) {
                    Image(systemName: m.setupFolded ? "chevron.right" : "chevron.down")
                        .font(.system(size: 9, weight: .bold)).foregroundColor(Tok.text3).frame(width: 13)
                    Text(m.t("setup.title")).font(.system(size: 12.5, weight: .bold)).foregroundColor(Tok.text)
                    Spacer(minLength: 0)
                    Text(m.t("setup.count", ["done": "\(done)", "total": "4"]))
                        .font(.system(size: 11.5, weight: .semibold)).foregroundColor(Tok.text3)
                }
                .frame(height: 22).contentShape(Rectangle())
            }.buttonStyle(.plain)
            ZStack(alignment: .leading) {
                Capsule().fill(Tok.track)
                GeometryReader { g in Capsule().fill(Tok.accent).frame(width: g.size.width * CGFloat(done) / 4) }
            }
            .frame(height: 4).padding(.vertical, 6)
            if !m.setupFolded {
                ForEach(0..<4, id: \.self) { i in
                    Button {
                        m.startTour(at: SetupChecklist.step(for: rows[i].id, passed: m.passed))
                    } label: {
                        HStack(spacing: 8) {
                            CheckCircle(state: rows[i].done ? .done : (current == i ? .current : .todo))
                                .accessibilityHidden(true)
                            Text(m.t(ids[i])).font(.system(size: 12.5)).foregroundColor(Tok.text)
                                .lineLimit(1).truncationMode(.tail)
                            Spacer(minLength: 0)
                            if i == 3 {
                                Text(m.t("setup.count", ["done": "\(m.passed.count)", "total": "8"]))
                                    .font(.system(size: 11.5)).foregroundColor(Tok.text3).lineLimit(1)
                            }
                        }
                        .frame(height: 26).contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .accessibilityValue(rows[i].done ? m.t("common.done") : "")
                    .accessibilityAddTraits(current == i ? .isSelected : [])
                }
            }
        }
        .padding(.horizontal, 12).padding(.top, 8).padding(.bottom, m.setupFolded ? 6 : 6)
        .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(scheme == .dark ? Tok.fill : Tok.panel))
    }
}

// MARK: Top icons

struct TopIcons: View {
    @EnvironmentObject var m: AppModel
    @Environment(\.colorScheme) private var scheme
    @State private var showUpdates = false

    var body: some View {
        HStack(spacing: 4) {
            Menu {
                ForEach([Theme.light, .dark, .system], id: \.self) { th in
                    Toggle(themeName(th), isOn: Binding(get: { m.theme == th }, set: { _ in m.theme = th }))
                }
            } label: {
                Image(systemName: scheme == .dark ? "moon" : "sun.max").font(.system(size: 16))
                    .foregroundColor(Tok.text2).frame(width: 30, height: 30)
            }
            .menuStyle(.borderlessButton).menuIndicator(.hidden).fixedSize()
            .help(m.t("top.theme")).accessibilityLabel(m.t("top.theme"))

            IconButton(symbol: "bell", label: m.t("top.whatsNew")) { showUpdates.toggle() }
                .overlay(alignment: .topTrailing) {
                    if case .newer = m.update { Circle().fill(Tok.ember).frame(width: 7, height: 7).offset(x: -4, y: 4) }
                }
                .popover(isPresented: $showUpdates, arrowEdge: .bottom) { UpdatesPopover() }

            Button { m.language = (m.language == .mn) ? .en : .mn } label: {
                Text(m.language == .mn ? "МН" : "EN")
                    .font(.system(size: 11, weight: .bold)).foregroundColor(Tok.text2)
                    .padding(.horizontal, 8).frame(height: 20)
                    .overlay(Capsule().stroke(Tok.text3, lineWidth: 1))
                    .frame(minWidth: 30, minHeight: 30).contentShape(Rectangle())
            }
            .buttonStyle(.plain).help(m.t("top.language")).accessibilityLabel(m.t("top.language"))
        }
    }

    private func themeName(_ t: Theme) -> String {
        switch t {
        case .light: return noEmoji(m.t("settings.general.theme.light"))
        case .dark: return noEmoji(m.t("settings.general.theme.dark"))
        case .system: return m.t("general.theme.system")
        }
    }
}

/// Drops a leading emoji and the space after it (old string keys have one).
func noEmoji(_ s: String) -> String {
    guard let first = s.unicodeScalars.first, !first.isASCII, let space = s.firstIndex(of: " ") else { return s }
    return String(s[s.index(after: space)...])
}

struct UpdatesPopover: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(m.t("settings.about.version", ["version": AppModel.version]))
                .font(.system(size: 13, weight: .semibold))
            Button(m.t("settings.general.checkUpdates")) { m.checkUpdates() }
                .buttonStyle(SecondaryButtonStyle())
            UpdateResult()
        }
        .padding(16).frame(width: 260, alignment: .leading)
    }
}

/// The result line of the update check.
struct UpdateResult: View {
    @EnvironmentObject var m: AppModel
    var body: some View {
        switch m.update {
        case .idle: EmptyView()
        case .checking: ProgressView().controlSize(.small)
        case .upToDate:
            Text(noEmoji(m.t("settings.general.upToDate"))).font(.hint).foregroundColor(Tok.success)
        case .newer(let tag):
            LinkButton(title: tag + " →") { m.open(AppModel.repo + "/releases/latest") }
        case .failed:
            Text(m.t("settings.general.updateError")).font(.hint).foregroundColor(Tok.text3)
        }
    }
}
