// SPDX-License-Identifier: MIT OR Apache-2.0
// Reads the portable config file and the word list. This runs when a text
// field gets focus, never per key.

import Foundation

final class ConfigStore {
    let configURL: URL
    let wordListURL: URL?
    private(set) var words: WordList?
    private(set) var configText: String?
    private var stamp: Date?
    /// Counts the reloads. A session compares it to know that it is out of date.
    private(set) var generation = 0

    /// The file the settings app and the web converter both write.
    static var defaultConfigURL: URL {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("Buuz", isDirectory: true)
            .appendingPathComponent("config.toml")
    }

    init(configURL: URL = ConfigStore.defaultConfigURL, wordListURL: URL?) {
        self.configURL = configURL
        self.wordListURL = wordListURL
        if let url = wordListURL, let data = try? Data(contentsOf: url) {
            words = WordList(data: data)
        }
        reload()
    }

    /// Reread the config when the file changed. Gives true when it did.
    @discardableResult
    func refresh() -> Bool {
        let now = modified()
        if now == stamp { return false }
        reload()
        return true
    }

    func makeSession() -> Session {
        Session(engine: Engine.lenient(config: configText, words: words))
    }

    private func reload() {
        stamp = modified()
        configText = try? String(contentsOf: configURL, encoding: .utf8)
        generation += 1
    }

    private func modified() -> Date? {
        let attrs = try? FileManager.default.attributesOfItem(atPath: configURL.path)
        return attrs?[.modificationDate] as? Date
    }
}
