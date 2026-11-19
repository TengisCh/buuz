// SPDX-License-Identifier: MIT OR Apache-2.0
// Entry point of the Buuz input method. macOS starts this app when a text
// field uses Buuz. It stays small: it starts the IMK server and waits.

import Cocoa
import InputMethodKit

let connectionName = Bundle.main.object(forInfoDictionaryKey: "InputMethodConnectionName") as? String
    ?? "com.thetengis.inputmethod.Buuz_Connection"

// The word list is in the app bundle. The shared store loads it once.
SharedStore.configure(wordListURL: Bundle.main.url(forResource: "mn", withExtension: "txt"))

let server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)
if server == nil {
    NSLog("Buuz: could not start the input method server")
    exit(1)
}

NSApplication.shared.run()
