// SPDX-License-Identifier: MIT OR Apache-2.0
// The typing flow of one text field: key in, marked text and commit out. It
// talks to the text field through `TextClient`, so the tests use a fake one.

import Foundation

protocol TextClient: AnyObject {
    /// Replace the marked text (if any) with `text`. First delete
    /// `deleteBefore` characters before the caret.
    func insert(_ text: String, deleteBefore: Int)
    /// Show `text` as underlined marked text with the caret at `cursor`.
    func setMarked(_ text: String, cursor: Int)
    func clearMarked()
    func beep()
}

final class Session {
    let engine: Engine
    private var hasMarked = false

    init(engine: Engine) { self.engine = engine }

    /// Handle one key down. Gives true when Buuz used the key. Gives false
    /// when the application must also get it (after any commit).
    func handle(keyCode: UInt16, characters: String?, mods: Mods, client: TextClient) -> Bool {
        var key = KeyMap.map(keyCode: keyCode, characters: characters)
        // A shortcut ends the word and goes to the application.
        if mods.isShortcut { key = MappedKey(kind: KeyMap.other) }
        let result = engine.process(kind: key.kind, ch: key.ch)
        if result.rejected { client.beep() }
        present(result, client)
        return result.consumed
    }

    /// The application asks to end the composition (focus change, click).
    func commitComposition(client: TextClient) {
        if let text = engine.commit() {
            client.insert(text, deleteBefore: 0)
            hasMarked = false
        } else if hasMarked {
            client.clearMarked()
            hasMarked = false
        }
    }

    private func present(_ r: KeyResult, _ client: TextClient) {
        if r.commit != nil || r.deleteBefore > 0 {
            client.insert(r.commit ?? "", deleteBefore: r.deleteBefore)
            hasMarked = false
        }
        if engine.isComposing {
            client.setMarked(engine.preedit, cursor: engine.cursor)
            hasMarked = true
        } else if hasMarked {
            client.clearMarked()
            hasMarked = false
        }
    }
}
