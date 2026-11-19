// SPDX-License-Identifier: MIT OR Apache-2.0
// The Buuz app: first-run welcome steps and the settings window. It is
// separate from the input method process, which stays tiny.

import SwiftUI

@main
struct BuuzSettingsApp: App {
    @StateObject private var model = AppModel()

    var body: some Scene {
        WindowGroup {
            Group {
                if model.showOnboarding { OnboardingView() } else { SettingsView() }
            }
            .environmentObject(model)
            .frame(width: 760, height: 540)
            .preferredColorScheme(model.dark ? .dark : .light)
        }
    }
}
