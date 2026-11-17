// Copyright 2009 Odbayar Nyamtseren <odbayar.n@gmail.com> (original Buuz rules)
// Rust port: Copyright 2026 the Buuz contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The conversion rule table, ported from `Composer::Composer()` in the
//! original Buuz `Composer.cpp`, plus the case-expansion logic of
//! `Composer::addRule()`.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Word-state flags. A "word" is one composition; it starts male.
pub(crate) const X_AC: u8 = 0x01; // allow case conversion
pub(crate) const X_M: u8 = 0x02; // usable in male words
pub(crate) const X_F: u8 = 0x04; // usable in female words
pub(crate) const X_MM: u8 = 0x08; // makes the word male
pub(crate) const X_MF: u8 = 0x10; // makes the word female

const ALL: u8 = X_M | X_F | X_AC;

/// Source rules exactly as listed in the original, in original order.
/// Patterns are written in upper case; `X_AC` rules are expanded to every
/// upper/lower case combination of the alphabetic pattern characters.
#[rustfmt::skip]
const SOURCE_RULES: &[(&str, &str, u8)] = &[
    // Cyrillic input: used when the composition buffer has been
    // re-seeded from Cyrillic output (after cursor movement).
    ("А",   "А",  ALL | X_MM),
    ("АI",  "АЙ", ALL | X_MM),
    ("О",   "О",  ALL | X_MM),
    ("ОI",  "ОЙ", ALL | X_MM),
    ("У",   "У",  ALL | X_MM),
    ("УI",  "УЙ", ALL | X_MM),
    ("Э",   "Э",  ALL | X_MF),
    ("ЭI",  "ЭЙ", ALL | X_MF),
    ("Ө",   "Ө",  ALL | X_MF),
    ("ӨI",  "ӨЙ", ALL | X_MF),
    ("Ү",   "Ү",  ALL | X_MF),
    ("ҮI",  "ҮЙ", ALL | X_MF),
    ("ИI",  "ИЙ", ALL),

    ("A",   "А",  ALL | X_MM),
    ("AI",  "АЙ", ALL | X_MM),
    ("B",   "Б",  ALL),
    ("C",   "Ц",  ALL),
    ("CH",  "Ч",  ALL),
    ("D",   "Д",  ALL),
    ("E",   "Э",  ALL | X_MF),
    ("EI",  "ЭЙ", ALL | X_MF),
    ("F",   "Ф",  ALL),
    ("G",   "Г",  ALL),
    ("H",   "Х",  ALL),
    ("I",   "И",  ALL),
    ("II",  "ИЙ", ALL),
    ("III", "Ы",  ALL),
    ("J",   "Ж",  ALL),
    ("K",   "К",  ALL),
    ("KH",  "Х",  ALL),
    ("L",   "Л",  ALL),
    ("M",   "М",  ALL),
    ("N",   "Н",  ALL),

    // O: plain 'o' follows the word's vowel-harmony state.
    ("O",    "О",  X_M | X_AC),
    ("OI",   "ОЙ", X_M | X_AC),
    ("O\"",  "О",  ALL | X_MM),
    ("O\"I", "ОЙ", ALL | X_MM),
    ("\"O",  "О",  ALL | X_MM),
    ("\"OI", "ОЙ", ALL | X_MM),

    ("O",    "Ө",  X_F | X_AC),
    ("OI",   "ӨЙ", X_F | X_AC),
    ("Q",    "Ө",  ALL | X_MF),
    ("QI",   "ӨЙ", ALL | X_MF),
    ("O'",   "Ө",  ALL | X_MF),
    ("O'I",  "ӨЙ", ALL | X_MF),
    ("'O",   "Ө",  ALL | X_MF),
    ("'OI",  "ӨЙ", ALL | X_MF),

    ("P",   "П",  ALL),
    ("R",   "Р",  ALL),
    ("S",   "С",  ALL),
    ("SH",  "Ш",  ALL),
    ("SXC", "Щ",  ALL),
    ("T",   "Т",  ALL),

    ("U",    "У",  X_M | X_AC),
    ("UI",   "УЙ", X_M | X_AC),
    ("U\"",  "У",  ALL | X_MM),
    ("U\"I", "УЙ", ALL | X_MM),
    ("\"U",  "У",  ALL | X_MM),
    ("\"UI", "УЙ", ALL | X_MM),

    ("U",    "Ү",  X_F | X_AC),
    ("UI",   "ҮЙ", X_F | X_AC),
    ("W",    "Ү",  ALL | X_MF),
    ("WI",   "ҮЙ", ALL | X_MF),
    ("U'",   "Ү",  ALL | X_MF),
    ("U'I",  "ҮЙ", ALL | X_MF),
    ("'U",   "Ү",  ALL | X_MF),
    ("'UI",  "ҮЙ", ALL | X_MF),

    ("V",   "В",  ALL),
    ("X",   "Х",  ALL),
    ("Y",   "Ы",  ALL),
    ("YA",  "Я",  ALL | X_MM),
    ("YE",  "Е",  ALL | X_MF),
    ("YO",  "Ё",  ALL | X_MM),
    ("YU",  "Ю",  ALL | X_MM),
    ("Z",   "З",  ALL),

    // Hard / soft signs: no case conversion.
    ("\"",   "ъ", X_M | X_F),
    ("\"\"", "Ъ", X_M | X_F),
    ("'",    "ь", X_M | X_F),
    ("''",   "Ь", X_M | X_F),
];

/// One expanded conversion rule.
#[derive(Debug, Clone)]
pub(crate) struct Rule {
    pub to: Vec<char>,
    pub flags: u8,
}

pub(crate) struct RuleSet {
    map: HashMap<Vec<char>, Vec<Rule>>,
    /// Distinct pattern lengths, longest first.
    pub lengths: Vec<usize>,
}

impl RuleSet {
    pub fn get(&self, from: &[char]) -> Option<&[Rule]> {
        self.map.get(from).map(|v| v.as_slice())
    }
}

pub(crate) fn to_upper(ch: char) -> char {
    if ch.is_ascii() {
        ch.to_ascii_uppercase()
    } else if (ch as u32) >> 8 == 4 {
        ch.to_uppercase().next().unwrap_or(ch)
    } else {
        ch
    }
}

pub(crate) fn to_lower(ch: char) -> char {
    if ch.is_ascii() {
        ch.to_ascii_lowercase()
    } else if (ch as u32) >> 8 == 4 {
        ch.to_lowercase().next().unwrap_or(ch)
    } else {
        ch
    }
}

fn is_alpha(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ((ch as u32) >> 8 == 4 && ch.is_alphabetic())
}

fn insert_rule(map: &mut HashMap<Vec<char>, Vec<Rule>>, from: Vec<char>, rule: Rule) {
    map.entry(from).or_default().push(rule);
}

/// Port of `Composer::addRule`: when `X_AC` is set, the k-th alphabetic
/// character of `from` controls the case of the k-th character of `to`
/// (alphabetic characters beyond the length of `to` only affect `from`).
fn add_rule(map: &mut HashMap<Vec<char>, Vec<Rule>>, from: &str, to: &str, flags: u8) {
    let from: Vec<char> = from.chars().collect();
    let to: Vec<char> = to.chars().collect();
    if flags & X_AC == 0 {
        insert_rule(map, from, Rule { to, flags });
        return;
    }
    let mut to_pos = 0usize;
    let pos_map: Vec<Option<usize>> = from
        .iter()
        .map(|&c| {
            if is_alpha(c) && to_pos < to.len() {
                to_pos += 1;
                Some(to_pos - 1)
            } else {
                None
            }
        })
        .collect();
    let alpha: Vec<usize> = (0..from.len()).filter(|&i| is_alpha(from[i])).collect();
    for mask in 0u32..(1u32 << alpha.len()) {
        let mut f = from.clone();
        let mut t = to.clone();
        for (bit, &i) in alpha.iter().enumerate() {
            let upper = mask & (1 << bit) == 0;
            f[i] = if upper {
                to_upper(f[i])
            } else {
                to_lower(f[i])
            };
            if let Some(tp) = pos_map[i] {
                t[tp] = if upper {
                    to_upper(t[tp])
                } else {
                    to_lower(t[tp])
                };
            }
        }
        insert_rule(map, f, Rule { to: t, flags });
    }
}

pub(crate) fn rules() -> &'static RuleSet {
    static RULES: OnceLock<RuleSet> = OnceLock::new();
    RULES.get_or_init(|| {
        let mut map = HashMap::new();
        for &(from, to, flags) in SOURCE_RULES {
            add_rule(&mut map, from, to, flags);
        }
        let mut lengths: Vec<usize> = map.keys().map(|k| k.len()).collect();
        lengths.sort_unstable_by(|a, b| b.cmp(a));
        lengths.dedup();
        RuleSet { map, lengths }
    })
}

/// Human-readable dump of the expanded table (`from = to`), sorted; useful
/// for docs and for diffing against the original IME's debug export.
pub fn export_rules() -> Vec<(String, String, bool, bool)> {
    let mut out = Vec::new();
    for (from, rs) in &rules().map {
        for r in rs {
            out.push((
                from.iter().collect::<String>(),
                r.to.iter().collect::<String>(),
                r.flags & X_M != 0,
                r.flags & X_F != 0,
            ));
        }
    }
    out.sort();
    out
}
