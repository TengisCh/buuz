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

/// Like `convert`, with smart vowel harmony and abbreviation expansion switched
/// on or off (`bn` -> `байна` needs `abbreviations`). Links and emails stay
/// Latin here whatever the flags are; `false, false` is not the classic
/// behaviour. Use `convert_classic` for the original Buuz.
#[wasm_bindgen]
pub fn convert_with(input: &str, smart_vowels: bool, abbreviations: bool) -> String {
    let opts = buuz_core::Options {
        smart_vowels,
        abbreviations: abbreviations.then(buuz_core::Abbreviations::builtin),
        ..Default::default()
    };
    buuz_core::convert_with(input, &opts)
}

/// Classic mode: the original Buuz rules with every smart feature off,
/// including any added later (autocorrect, completion). The "Classic" switch
/// on the site uses this rather than `convert_with(.., false, false)`.
#[wasm_bindgen]
pub fn convert_classic(input: &str) -> String {
    buuz_core::convert_with(input, &buuz_core::Options::classic())
}

/// The built-in abbreviation list as text, one `abbreviation = expansion` per line.
#[wasm_bindgen]
pub fn abbreviation_list() -> String {
    buuz_core::Abbreviations::builtin().to_text()
}

#[cfg(test)]
mod tests {
    #[test]
    fn matches_core() {
        assert_eq!(super::convert("sain baina uu"), "сайн байна уу");
    }

    #[test]
    fn links_and_emails_stay_latin_except_in_classic() {
        assert_eq!(super::convert("github.com"), "github.com");
        assert_eq!(super::convert("bat@gmail.com"), "bat@gmail.com");
        assert_ne!(super::convert_classic("github.com"), "github.com");
    }

    #[test]
    fn convert_with_switches_smart_vowels() {
        assert_eq!(super::convert_with("unee", true, false), "үнээ");
        assert_eq!(super::convert_with("unee", false, false), "унээ");
        assert_eq!(super::convert_with("irsen uu", true, false), "ирсэн үү");
        assert_eq!(super::convert_with("irsen uu", false, false), "ирсэн уу");
    }

    #[test]
    fn convert_classic_is_the_original_buuz() {
        assert_eq!(super::convert_classic("unee"), "унээ");
        assert_eq!(super::convert_classic("irsen uu"), "ирсэн уу");
        assert_eq!(super::convert_classic("bn uu "), "бн уу ");
        assert_eq!(super::convert_classic("sain baina"), "сайн байна");
    }

    #[test]
    fn convert_with_switches_abbreviations() {
        assert_eq!(super::convert_with("bn uu", true, true), "байна уу");
        // Only a typed Space or punctuation expands; the end of the text does not.
        assert_eq!(super::convert_with("bn ", true, true), "байна ");
        assert_eq!(super::convert_with("bn.", true, true), "байна.");
        assert_eq!(super::convert_with("bn", true, true), "бн");
        assert_eq!(super::convert_with("bn ", true, false), "бн ");
        assert!(super::abbreviation_list().contains("bn = байна\n"));
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
