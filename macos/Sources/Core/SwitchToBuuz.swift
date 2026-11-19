// SPDX-License-Identifier: MIT OR Apache-2.0
// The plan for "Switch to Buuz". macOS may not have started the input method
// process yet. Then `TISSelectInputSource` gives -50 until the process runs.
// The plan launches the input method and tries once more. Pure logic: the
// caller gives closures for each action.

import Foundation

enum SwitchResult: Equatable {
    case switched
    /// Buuz is not in the user's keyboards and could not be added.
    case notEnabled
    /// `select` failed, also after the launch and one retry.
    case failed(status: Int)

    var isFailure: Bool { self != .switched }
}

enum SwitchToBuuz {
    /// `select` gives the OSStatus, or nil when macOS does not list Buuz.
    /// `after` runs its closure about one second later, without blocking the
    /// main thread. `done` is called once.
    static func run(
        isEnabled: () -> Bool, enable: () -> Bool, select: @escaping () -> Int?,
        launch: () -> Void, after: (@escaping () -> Void) -> Void, done: @escaping (SwitchResult) -> Void
    ) {
        if !isEnabled() {
            guard enable() else { done(.notEnabled); return }
            launch()
            after { done(finish(select())) }
            return
        }
        let first = select()
        if first == 0 { done(.switched); return }
        launch()
        after { done(finish(select())) }
    }

    private static func finish(_ status: Int?) -> SwitchResult {
        status == 0 ? .switched : .failed(status: status ?? -1)
    }
}
