// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

//! WebAssembly wrapper: exposes `buuz-core` to the website so the page gives
//! exactly the same output as the Windows and macOS input methods.

use wasm_bindgen::prelude::*;

/// Transliterate Latin text to Mongolian Cyrillic ("sain baina uu" -> "сайн байна уу").
#[wasm_bindgen]
pub fn convert(input: &str) -> String {
    buuz_core::convert(input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn matches_core() {
        assert_eq!(super::convert("sain baina uu"), "сайн байна уу");
    }
}
