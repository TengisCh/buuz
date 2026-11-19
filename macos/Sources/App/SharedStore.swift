// SPDX-License-Identifier: MIT OR Apache-2.0
// One word list and one config for the whole input method process.

import Foundation

enum SharedStore {
    private(set) static var store = ConfigStore(wordListURL: nil)

    static func configure(wordListURL: URL?) {
        store = ConfigStore(wordListURL: wordListURL)
    }
}
