// SPDX-License-Identifier: MIT OR Apache-2.0
// The InputMethodKit controller. macOS makes one for each text field that
// uses Buuz. It only moves keys and text between IMK and `Session`.

import Cocoa
import InputMethodKit

/// Adapts the IMK client (the text field) to `TextClient`.
private final class IMKClientAdapter: TextClient {
    var client: IMKTextInput?
    private static let none = NSRange(location: NSNotFound, length: NSNotFound)
    private static let markAttributes: [NSAttributedString.Key: Any] = [
        .underlineStyle: NSUnderlineStyle.single.rawValue,
    ]

    func insert(_ text: String, deleteBefore: Int) {
        guard let client = client else { return }
        var range = IMKClientAdapter.none
        if deleteBefore > 0 {
            // Some text fields do not report the caret. Then the old text stays
            // and only the new text goes in.
            let caret = client.selectedRange()
            if caret.location != NSNotFound, caret.length == 0, caret.location >= deleteBefore {
                range = NSRange(location: caret.location - deleteBefore, length: deleteBefore)
            }
        }
        client.insertText(text, replacementRange: range)
    }

    func setMarked(_ text: String, cursor: Int) {
        guard let client = client else { return }
        let marked = NSAttributedString(string: text, attributes: IMKClientAdapter.markAttributes)
        client.setMarkedText(
            marked,
            selectionRange: NSRange(location: cursor, length: 0),
            replacementRange: IMKClientAdapter.none)
    }

    func clearMarked() {
        client?.setMarkedText(
            "", selectionRange: NSRange(location: 0, length: 0),
            replacementRange: IMKClientAdapter.none)
    }

    func beep() { NSSound.beep() }
}

@objc(BuuzController)
final class BuuzController: IMKInputController {
    private var session: Session?
    private var generation = -1
    private let adapter = IMKClientAdapter()

    /// The session for this text field. It restarts when the config changed.
    private func currentSession() -> Session {
        let store = SharedStore.store
        if let s = session, generation == store.generation { return s }
        let s = store.makeSession()
        session = s
        generation = store.generation
        return s
    }

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        // Keys must reach Buuz as plain Latin, whatever the keyboard layout is.
        (sender as? IMKTextInput)?.overrideKeyboard(withKeyboardNamed: "com.apple.keylayout.ABC")
        SharedStore.store.refresh()
        _ = currentSession()
    }

    override func deactivateServer(_ sender: Any!) {
        commitComposition(sender)
        super.deactivateServer(sender)
    }

    override func commitComposition(_ sender: Any!) {
        guard let client = sender as? IMKTextInput else { return }
        adapter.client = client
        currentSession().commitComposition(client: adapter)
    }

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask.keyDown.rawValue)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event, event.type == .keyDown,
            let client = sender as? IMKTextInput
        else { return false }
        adapter.client = client
        let flags = event.modifierFlags
        let mods = Mods(
            command: flags.contains(.command),
            control: flags.contains(.control),
            option: flags.contains(.option))
        return currentSession().handle(
            keyCode: event.keyCode, characters: event.characters, mods: mods, client: adapter)
    }
}
