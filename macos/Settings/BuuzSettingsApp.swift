// SPDX-License-Identifier: MIT OR Apache-2.0
// The Buuz app: tutorial, lessons and settings. It is separate from the input
// method process, which stays tiny.

import SwiftUI

@main
struct BuuzSettingsApp: App {
    @StateObject private var model = AppModel()

    var body: some Scene {
        WindowGroup {
            AppShell()
                .environmentObject(model)
                // 568 + the 32 pt title bar area gives a 600 pt window at the smallest size.
                .frame(minWidth: 880, maxWidth: .infinity, minHeight: 568, maxHeight: .infinity)
                .onAppear { model.applyTheme() }
        }
        .windowStyle(.hiddenTitleBar)
        .commands {
            CommandGroup(replacing: .appSettings) {
                Button(model.t("nav.settings") + "…") { model.openSettings() }
                    .keyboardShortcut(",", modifiers: .command)
                    .disabled(model.tourActive)
            }
        }
    }
}
