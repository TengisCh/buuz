// SPDX-License-Identifier: MIT OR Apache-2.0
// Swift view of the buuz-ffi C ABI (ffi/include/buuz.h). No AppKit here, so
// the tests can run it without a window server.

import Foundation

/// The result of one key. The shell reads it in this order: delete
/// `deleteBefore` characters before the caret, insert `commit`, then show the
/// preedit.
struct KeyResult {
    var consumed = false
    var rejected = false
    var corrected = false
    var deleteBefore = 0
    var commit: String?
}

/// A parsed word list (`data/wordlist/mn.txt`). Share one for the whole app.
final class WordList {
    fileprivate let handle: OpaquePointer

    init?(data: Data) {
        let made: OpaquePointer? = data.withUnsafeBytes { raw in
            let base = raw.bindMemory(to: UInt8.self).baseAddress
            return buuz_word_list_parse(base, raw.count)
        }
        guard let made = made else { return nil }
        handle = made
    }

    convenience init?(path: String) {
        guard let data = FileManager.default.contents(atPath: path) else { return nil }
        self.init(data: data)
    }

    deinit { buuz_word_list_free(handle) }
}

/// One composer. Use it from one thread at a time (the main thread).
final class Engine {
    private let composer: OpaquePointer
    private var outcome = BuuzOutcome()

    /// Gives nil when `config` is not a valid portable config.
    init?(config: String?, words: WordList?) {
        var made: OpaquePointer?
        if let config = config, !config.isEmpty {
            let bytes = Array(config.utf8)
            made = bytes.withUnsafeBufferPointer {
                buuz_composer_new($0.baseAddress, $0.count, words?.handle)
            }
        } else {
            made = buuz_composer_new(nil, 0, words?.handle)
        }
        guard let made = made else { return nil }
        composer = made
    }

    /// Never fails: a bad config falls back to the smart defaults, so a typo in
    /// the config file cannot stop people from typing.
    static func lenient(config: String?, words: WordList?) -> Engine {
        if let engine = Engine(config: config, words: words) { return engine }
        if let engine = Engine(config: nil, words: words) { return engine }
        // The defaults cannot fail unless memory is gone.
        fatalError("buuz_composer_new failed for the default config")
    }

    deinit { buuz_composer_free(composer) }

    static var version: String { String(cString: buuz_version()) }

    func process(kind: UInt32, ch: UInt32 = 0) -> KeyResult {
        var result = KeyResult()
        guard buuz_process_key(composer, kind, ch, &outcome) else { return result }
        result.consumed = outcome.consumed
        result.rejected = outcome.rejected
        result.corrected = outcome.corrected
        result.deleteBefore = Int(outcome.delete_before)
        if let p = outcome.commit {
            result.commit = Engine.string(p, outcome.commit_len)
        }
        return result
    }

    var isComposing: Bool { buuz_is_composing(composer) }

    /// The cursor inside the preedit, in UTF-16 units.
    var cursor: Int { Int(buuz_cursor(composer)) }

    var preedit: String {
        var len = 0
        guard let p = buuz_preedit(composer, &len) else { return "" }
        return Engine.string(p, len)
    }

    /// Take the preedit as text and end the composition. Nil when empty.
    func commit() -> String? {
        var len = 0
        guard let p = buuz_commit(composer, &len) else { return nil }
        return Engine.string(p, len)
    }

    func cancel() { buuz_cancel(composer) }

    private static func string(_ p: UnsafePointer<CChar>, _ len: Int) -> String {
        p.withMemoryRebound(to: UInt8.self, capacity: len) {
            String(decoding: UnsafeBufferPointer(start: $0, count: len), as: UTF8.self)
        }
    }
}
