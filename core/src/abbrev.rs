// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0.

//! Abbreviation expansion (issue #5): a committed word such as `bn` becomes
//! the full Cyrillic word `байна`. The table is plain data passed to the core;
//! the core itself does no I/O.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, OnceLock};

use crate::composer::{is_input_char, MAX_COMP_LEN};
use crate::rules::to_lower;

/// Why an abbreviation table could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbbrevError {
    /// The key is empty, too long, or has a character that cannot be typed.
    /// Carries the 1-based line number (0 when not parsing text) and the key.
    BadKey { line: usize, key: String },
    /// The expansion is empty or contains a line break.
    BadValue { line: usize, key: String },
    /// A text line that is not `key = value`.
    MissingEquals { line: usize },
}

impl fmt::Display for AbbrevError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AbbrevError::BadKey { line: 0, key } => write!(f, "invalid abbreviation {key:?}"),
            AbbrevError::BadKey { line, key } => {
                write!(f, "line {line}: invalid abbreviation {key:?}")
            }
            AbbrevError::BadValue { line: 0, key } => {
                write!(f, "empty or multi-line expansion for {key:?}")
            }
            AbbrevError::BadValue { line, key } => {
                write!(f, "line {line}: empty or multi-line expansion for {key:?}")
            }
            AbbrevError::MissingEquals { line } => {
                write!(f, "line {line}: expected `abbreviation = expansion`")
            }
        }
    }
}

impl std::error::Error for AbbrevError {}

/// The built-in list: very common words that people already abbreviate in
/// Latin chat. Note that each key's literal conversion (`bn` -> бн) is not a
/// real word, so an entry never hides a word typed in full.
const BUILTIN: &[(&str, &str)] = &[
    ("bn", "байна"),
    ("bnu", "байна уу"),
    ("bga", "байгаа"),
    ("bgaa", "байгаа"),
    ("bhgui", "байхгүй"),
    ("bhgwi", "байхгүй"),
    ("ym", "юм"),
    ("zgr", "зүгээр"),
    ("mdku", "мэдэхгүй"),
    ("sn", "сайн"),
    ("sbnu", "сайн байна уу"),
    ("tgd", "тэгээд"),
    // Chat shortcuts from real typing samples (2026-11-19).
    ("bna", "байна"),
    ("snu", "сайн уу"),
    ("sainu", "сайн уу"),
    ("yu", "юу"),
    ("mar", "маргааш"),
    ("uzi", "үзье"),
    ("yri", "ярья"),
    ("baiy", "байя"),
    ("shde", "ш дээ"),
    // The particle нь. `ни` is not in the word list, so `ni` never hides a word.
    ("ni", "нь"),
];

/// A table of abbreviations: lowercase Latin keys and their Cyrillic text.
#[derive(Debug, Clone, Default)]
pub struct Abbreviations {
    map: HashMap<Box<str>, Box<str>>,
}

fn lower_key(key: &str) -> String {
    key.chars().map(to_lower).collect()
}

pub(crate) fn check_key(key: &str, line: usize) -> Result<String, AbbrevError> {
    let n = key.chars().count();
    if n == 0 || n > MAX_COMP_LEN || !key.chars().all(is_input_char) {
        return Err(AbbrevError::BadKey {
            line,
            key: key.to_string(),
        });
    }
    Ok(lower_key(key))
}

pub(crate) fn check_value(key: &str, value: &str, line: usize) -> Result<(), AbbrevError> {
    if value.is_empty() || value.contains(['\n', '\r']) {
        return Err(AbbrevError::BadValue {
            line,
            key: key.to_string(),
        });
    }
    Ok(())
}

impl Abbreviations {
    /// The built-in conservative list (shared, built once).
    pub fn builtin() -> Arc<Abbreviations> {
        static CELL: OnceLock<Arc<Abbreviations>> = OnceLock::new();
        CELL.get_or_init(|| {
            Arc::new(Abbreviations::from_pairs(BUILTIN.iter().copied()).expect("valid built-in"))
        })
        .clone()
    }

    /// The built-in list with user changes on top: a pair with a non-empty
    /// value adds or replaces an entry, a pair with an empty value removes
    /// one. Keys must already be checked and lowercase (see `Config::parse`).
    pub(crate) fn builtin_with(changes: &[(String, String)]) -> Self {
        let mut map = Self::builtin().map.clone();
        for (k, v) in changes {
            if v.is_empty() {
                map.remove(k.as_str());
            } else {
                map.insert(k.as_str().into(), v.as_str().into());
            }
        }
        Abbreviations { map }
    }

    /// Build a table from `(key, expansion)` pairs. Keys are matched without
    /// regard to case; a later duplicate replaces an earlier one.
    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self, AbbrevError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            let (k, v) = (k.as_ref().trim(), v.as_ref().trim());
            let key = check_key(k, 0)?;
            check_value(k, v, 0)?;
            map.insert(key.into_boxed_str(), v.into());
        }
        Ok(Abbreviations { map })
    }

    /// Parse the plain-text form: one `abbreviation = expansion` per line.
    /// Blank lines and lines starting with `#` are skipped; the last of
    /// several duplicates wins.
    pub fn parse(text: &str) -> Result<Self, AbbrevError> {
        let mut map = HashMap::new();
        for (i, raw) in text.lines().enumerate() {
            let line = i + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (k, v) = raw
                .split_once('=')
                .ok_or(AbbrevError::MissingEquals { line })?;
            let (k, v) = (k.trim(), v.trim());
            let key = check_key(k, line)?;
            check_value(k, v, line)?;
            map.insert(key.into_boxed_str(), v.into());
        }
        Ok(Abbreviations { map })
    }

    /// The plain-text form, sorted by key; `parse` reads it back.
    pub fn to_text(&self) -> String {
        let mut entries: Vec<_> = self.map.iter().collect();
        entries.sort();
        let mut s = String::new();
        for (k, v) in entries {
            s.push_str(k);
            s.push_str(" = ");
            s.push_str(v);
            s.push('\n');
        }
        s
    }

    /// Expansion for a raw (typed) word, with the typed case carried over:
    /// `BN` gives all capitals, `Bn` a capital first letter.
    pub(crate) fn lookup(&self, raw: &[char]) -> Option<String> {
        if self.map.is_empty() || raw.is_empty() {
            return None;
        }
        let key: String = raw.iter().map(|&c| to_lower(c)).collect();
        let text = self.map.get(key.as_str())?;
        let letters = raw.iter().filter(|c| c.is_alphabetic());
        let upper = |c: &char| c.is_uppercase();
        if letters.clone().count() >= 2 && letters.clone().all(upper) {
            return Some(text.to_uppercase());
        }
        if raw.first().is_some_and(upper) {
            let mut it = text.chars();
            let first = it.next()?;
            return Some(first.to_uppercase().chain(it).collect());
        }
        Some(text.to_string())
    }
}
