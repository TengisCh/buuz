// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

//! WebAssembly wrapper: exposes `buuz-core` to the website so the page gives
//! exactly the same output as the Windows and macOS input methods.

use wasm_bindgen::prelude::*;

/// Transliterate Latin text to Mongolian Cyrillic ("sain baina uu" -> "сайн байна уу"),
/// with smart vowel harmony on.
#[wasm_bindgen]
pub fn convert(input: &str) -> String {
    buuz_core::convert(input)
}

/// Like `convert`, with smart vowel harmony switched on or off
/// (`false` is the classic left-to-right behaviour of the original Buuz).
#[wasm_bindgen]
pub fn convert_with(input: &str, smart_vowels: bool) -> String {
    buuz_core::convert_with(input, &buuz_core::Options { smart_vowels })
}

#[cfg(test)]
mod tests {
    #[test]
    fn matches_core() {
        assert_eq!(super::convert("sain baina uu"), "сайн байна уу");
    }

    #[test]
    fn convert_with_switches_smart_vowels() {
        assert_eq!(super::convert_with("unee", true), "үнээ");
        assert_eq!(super::convert_with("unee", false), "унээ");
        assert_eq!(super::convert_with("irsen uu", true), "ирсэн үү");
        assert_eq!(super::convert_with("irsen uu", false), "ирсэн уу");
    }

    /// Every typewriter phrase on the page must convert exactly as written.
    #[test]
    fn site_phrases_convert_as_written() {
        let json = include_str!("../../site/src/phrases.json");
        // phrases.json is a flat list of string pairs; read its strings (with \" and \\).
        let mut strings = Vec::new();
        let mut cur: Option<String> = None;
        let mut chars = json.chars();
        while let Some(c) = chars.next() {
            match (&mut cur, c) {
                (None, '"') => cur = Some(String::new()),
                (None, _) => {}
                (Some(_), '"') => strings.push(cur.take().unwrap()),
                (Some(s), '\\') => s.extend(chars.next()),
                (Some(s), c) => s.push(c),
            }
        }
        assert!(strings.len() >= 2 && strings.len() % 2 == 0);
        for pair in strings.chunks(2) {
            assert_eq!(super::convert(&pair[0]), pair[1], "phrase {:?}", pair[0]);
        }
    }
}
