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
    /// In one step: delete `replacingBefore` characters before the caret and
    /// show `text` as marked text. Some text fields (Chromium) cannot do this
    /// in two steps.
    func setMarked(_ text: String, cursor: Int, replacingBefore: Int)
    /// True when the client can say where the caret is. Without it, Buuz
    /// cannot delete text before the caret.
    var canReplace: Bool { get }
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
        // Backspace reopens a word only if Buuz can delete the old word. If the
        // caret is unknown, forget the word and let the application delete.
        if key.kind == KeyMap.backspace, !engine.isComposing, !client.canReplace {
            engine.cancel()
            return false
        }
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
        // Reopen a word: replace the old text with the marked text in one call.
        if (r.commit ?? "").isEmpty, r.deleteBefore > 0, engine.isComposing {
            client.setMarked(engine.preedit, cursor: engine.cursor, replacingBefore: r.deleteBefore)
            hasMarked = true
            return
        }
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
