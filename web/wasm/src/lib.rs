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

// --- Portable config (see docs/decisions/0005-portable-config-format.md) -----
//
// The page keeps the config as text. Errors and warnings go to JavaScript as
// machine-readable strings so the page can show them in its own language:
// `code<TAB>number<TAB>detail`. `number` is a line number (0 when there is
// none); `detail` is a key or section name. For `too_new` the number is the
// version found and the detail is the version supported. For `too_large` the
// detail is the limit.

fn tag(code: &str, number: usize, detail: &str) -> String {
    format!(
        "{code}\t{number}\t{}",
        detail.replace(['\t', '\n', '\r'], " ")
    )
}

fn error_tag(e: &buuz_core::ConfigError) -> String {
    use buuz_core::{AbbrevError, ConfigError as E};
    match e {
        E::MissingVersion { line } => tag("missing_version", *line, ""),
        E::TooNew { found, supported } => tag("too_new", *found as usize, &supported.to_string()),
        E::Syntax { line } => tag("syntax", *line, ""),
        E::UnknownKey { line, key } => tag("unknown_key", *line, key),
        E::BadValue { line, key } => tag("bad_value", *line, key),
        E::BadEscape { line } => tag("bad_escape", *line, ""),
        E::SingleQuotes { line } => tag("single_quotes", *line, ""),
        E::DuplicateKey { line, key } => tag("duplicate_key", *line, key),
        E::DuplicateSection { line, name } => tag("duplicate_section", *line, name),
        E::Abbreviation(AbbrevError::BadKey { line, key })
        | E::Abbreviation(AbbrevError::BadValue { line, key }) => {
            tag("bad_abbreviation", *line, key)
        }
        E::Abbreviation(AbbrevError::MissingEquals { line }) => tag("syntax", *line, ""),
        E::BadKeys { line, key } => tag("bad_keys", *line, key),
        E::TooLarge { line: 0, .. } => tag("too_large", 0, "1 MiB"),
        E::TooLarge { line, limit } => tag("too_many_entries", *line, &limit.to_string()),
        other => tag("other", 0, &other.to_string()),
    }
}

fn warning_tag(w: &buuz_core::ConfigWarning) -> String {
    use buuz_core::ConfigWarning as W;
    match w {
        W::UnknownOption { line, key } => tag("unknown_option", *line, key),
        W::UnknownSection { line, name } => tag("unknown_section", *line, name),
        other => tag("other", 0, &other.to_string()),
    }
}

fn parse_config(text: &str) -> Result<buuz_core::Parsed, String> {
    buuz_core::Config::parse(text).map_err(|e| error_tag(&e))
}

/// A config that was read and written back in canonical form.
#[wasm_bindgen]
pub struct ConfigReport {
    text: String,
    warnings: Vec<String>,
    classic: bool,
    smart_vowels: bool,
    abbreviations: bool,
}

#[wasm_bindgen]
impl ConfigReport {
    /// The canonical config text.
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }
    /// What the parser ignored, as `code<TAB>line<TAB>name` strings.
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
    /// The preset is classic.
    #[wasm_bindgen(getter)]
    pub fn classic(&self) -> bool {
        self.classic
    }
    /// Smart vowels are on after the preset and overrides.
    #[wasm_bindgen(getter)]
    pub fn smart_vowels(&self) -> bool {
        self.smart_vowels
    }
    /// Abbreviation expansion is on after the preset and overrides.
    #[wasm_bindgen(getter)]
    pub fn abbreviations(&self) -> bool {
        self.abbreviations
    }
}

/// The default config (smart preset, no overrides) as text.
#[wasm_bindgen]
pub fn default_config_text() -> String {
    buuz_core::Config::default().to_text()
}

/// Read a config text and give back its canonical form, the warnings and the
/// effective switches. The error is a tagged string (see above).
#[wasm_bindgen]
pub fn config_normalize(text: &str) -> Result<ConfigReport, String> {
    let parsed = parse_config(text)?;
    let o = parsed.config.options();
    Ok(ConfigReport {
        text: parsed.config.to_text(),
        warnings: parsed.warnings.iter().map(warning_tag).collect(),
        classic: parsed.config.preset() == buuz_core::Preset::Classic,
        smart_vowels: o.smart_vowels,
        abbreviations: o.abbreviations.is_some(),
    })
}

/// Set the preset of a config and return the new canonical text. Overrides,
/// abbreviations and keys stay.
#[wasm_bindgen]
pub fn config_set_classic(text: &str, classic: bool) -> Result<String, String> {
    let mut config = parse_config(text)?.config;
    config.set_preset(if classic {
        buuz_core::Preset::Classic
    } else {
        buuz_core::Preset::Smart
    });
    Ok(config.to_text())
}

/// Switch `smart_vowels` or `abbreviations` and return the new canonical
/// text. A value equal to the preset's own is not written, so the default
/// config stays free of overrides.
#[wasm_bindgen]
pub fn config_set_option(text: &str, name: &str, value: bool) -> Result<String, String> {
    let mut config = parse_config(text)?.config;
    let base = if config.preset() == buuz_core::Preset::Classic {
        buuz_core::Options::classic()
    } else {
        buuz_core::Options::smart()
    };
    match name {
        "smart_vowels" => config.set_smart_vowels((value != base.smart_vowels).then_some(value)),
        "abbreviations" => {
            config.set_abbreviations((value != base.abbreviations.is_some()).then_some(value))
        }
        _ => return Err(tag("unknown_option", 0, name)),
    }
    Ok(config.to_text())
}

thread_local! {
    // The options of the last two config texts (the page's config and its
    // typewriter-demo config), so typing does not parse either again.
    static RECENT: std::cell::RefCell<Vec<(String, buuz_core::Options)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Transliterate with the options of a config text. The error is a tagged
/// string (see above).
#[wasm_bindgen]
pub fn convert_with_config(input: &str, config_text: &str) -> Result<String, String> {
    RECENT.with(|recent| {
        let mut recent = recent.borrow_mut();
        let at = match recent.iter().position(|(t, _)| t == config_text) {
            Some(i) => i,
            None => {
                let opts = parse_config(config_text)?.config.options();
                if recent.len() >= 2 {
                    recent.remove(0);
                }
                recent.push((config_text.to_string(), opts));
                recent.len() - 1
            }
        };
        Ok(buuz_core::convert_with(input, &recent[at].1))
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_config_is_smart_with_abbreviations() {
        let cfg = super::default_config_text();
        for input in ["unee", "irsen uu", "bn uu ", "aaviin", "github.com"] {
            assert_eq!(
                super::convert_with_config(input, &cfg).unwrap(),
                super::convert_with(input, true, true),
                "{input}"
            );
        }
    }

    #[test]
    fn config_classic_and_overrides_change_output() {
        let classic = super::config_set_classic(&super::default_config_text(), true).unwrap();
        assert_eq!(
            super::convert_with_config("unee", &classic).unwrap(),
            "унээ"
        );
        assert_eq!(super::convert_with_config("bn ", &classic).unwrap(), "бн ");
        let t = super::config_set_option(&super::default_config_text(), "abbreviations", false)
            .unwrap();
        assert!(t.contains("abbreviations = false"));
        assert_eq!(super::convert_with_config("bn ", &t).unwrap(), "бн ");
        // Back to the preset value: the override goes away.
        let t = super::config_set_option(&t, "abbreviations", true).unwrap();
        assert_eq!(t, super::default_config_text());
        assert!(super::config_set_option(&t, "nope", true).is_err());
    }

    #[test]
    fn normalize_gives_canonical_text_and_warnings() {
        let r = super::config_normalize(
            "# mine\nbuuz_config = 1\n[options]\nsmart_vowels = false\nfuture = true\n",
        )
        .unwrap();
        assert_eq!(
            r.text,
            "buuz_config = 1\npreset = \"smart\"\n\n[options]\nsmart_vowels = false\n"
        );
        assert_eq!(r.warnings, vec!["unknown_option\t5\tfuture".to_string()]);
        assert!(!r.classic && !r.smart_vowels && r.abbreviations);
        assert_eq!(super::config_normalize(&r.text).unwrap().text, r.text);
    }

    #[test]
    fn two_configs_alternate_with_the_right_output() {
        let a = super::default_config_text();
        let b = super::config_set_classic(&a, true).unwrap();
        for _ in 0..3 {
            assert_eq!(super::convert_with_config("unee", &a).unwrap(), "үнээ");
            assert_eq!(super::convert_with_config("unee", &b).unwrap(), "унээ");
        }
        let c = super::config_set_option(&a, "smart_vowels", false).unwrap();
        assert_eq!(super::convert_with_config("unee", &c).unwrap(), "унээ");
        assert_eq!(super::convert_with_config("unee", &a).unwrap(), "үнээ");
    }

    #[test]
    fn too_large_tags() {
        let big = "#".repeat(1024 * 1024 + 1);
        assert_eq!(
            super::config_normalize(&big).err().unwrap(),
            "too_large\t0\t1 MiB"
        );
        let mut many = String::from("buuz_config = 1\n[abbreviations]\n");
        for i in 0..10_001 {
            let k: String = [i / 676, i / 26 % 26, i % 26]
                .iter()
                .map(|d| (b'a' + *d as u8) as char)
                .collect();
            many.push_str(&format!("{k} = \"x\"\n"));
        }
        assert!(super::config_normalize(&many)
            .err()
            .unwrap()
            .starts_with("too_many_entries\t"));
    }

    #[test]
    fn bad_config_gives_tagged_errors() {
        assert_eq!(
            super::config_normalize("hello").err().unwrap(),
            "missing_version\t1\t"
        );
        assert_eq!(
            super::config_normalize("buuz_config = 9").err().unwrap(),
            "too_new\t9\t1"
        );
        assert!(super::convert_with_config("x", "buuz_config = 1\nzzz = 1").is_err());
        assert!(super::config_set_classic("", true).is_err());
    }

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
