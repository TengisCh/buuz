// SPDX-License-Identifier: Apache-2.0
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
pub(crate) const X_II: u8 = 0x20; // smart mode may turn this ий into ы
pub(crate) const X_DV: u8 = 0x40; // only with `Options::double_v`
pub(crate) const X_HK: u8 = 0x80; // only with `Options::habit_keys`

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
    ("II",  "ИЙ", ALL | X_II),
    ("III", "Ы",  ALL),
    ("II'", "ИЙ", ALL),
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
    ("Ö",    "Ө",  ALL | X_MF),
    ("ÖI",   "ӨЙ", ALL | X_MF),
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
    ("Ü",    "Ү",  ALL | X_MF),
    ("ÜI",   "ҮЙ", ALL | X_MF),
    ("U'",   "Ү",  ALL | X_MF),
    ("U'I",  "ҮЙ", ALL | X_MF),
    ("'U",   "Ү",  ALL | X_MF),
    ("'UI",  "ҮЙ", ALL | X_MF),

    ("V",   "В",  ALL),
    // `vv` is үү, only with `Options::double_v` (the composer skips it
    // otherwise, so classic output stays the original).
    ("VV",  "ҮҮ", ALL | X_MF | X_DV),
    ("VVI", "ҮҮЙ", ALL | X_MF | X_DV),
    ("X",   "Х",  ALL),
    ("Y",   "Ы",  ALL),
    ("YA",  "Я",  ALL | X_MM),
    ("YE",  "Е",  ALL | X_MF),
    ("YO",  "Ё",  ALL | X_MM),
    ("YU",  "Ю",  ALL | X_MM),
    ("Z",   "З",  ALL),
    // `ts` and `zh` are ц and ж, only with `Options::habit_keys` (the
    // composer skips them otherwise, so classic output stays the original).
    ("TS",  "Ц",  ALL | X_HK),
    ("ZH",  "Ж",  ALL | X_HK),

    // Hard / soft signs: no case conversion.
    ("\"",   "ъ", X_M | X_F),
    ("\"\"", "Ъ", X_M | X_F),
    ("'",    "ь", X_M | X_F),
    ("''",   "Ь", X_M | X_F),
];

/// Consonants after which a masculine word keeps ий instead of ы (smart `ii`):
/// ж ч ш щ ь ъ, the front г, and к, й.
pub(crate) const KEEP_II: &[char] = &['ж', 'ч', 'ш', 'щ', 'ь', 'ъ', 'г', 'к', 'й'];

/// Masculine и-final stems typed as `...ii...`: the и is part of the stem, so
/// the suffix keeps ий (салхийг, таксийг, компанийг).
pub(crate) const II_FINAL_STEMS: &[&str] = &["салх", "такс", "компан"];

/// Stems before г that keep гуй without a word list (smart `ui`): дугуй,
/// шугуй, бугуй, агуй, огторгуй. A stem that ends in н keeps it too.
pub(crate) const GUI_STEMS: &[&str] = &["ду", "шу", "бу", "а", "огтор"];

/// One expanded conversion rule.
#[derive(Debug, Clone)]
pub(crate) struct Rule {
    pub to: Vec<char>,
    pub flags: u8,
}

/// A conversion table: pattern to a group of rules, plus the distinct pattern
/// lengths. Lookups stay one hash lookup; a group is found by its index, so a
/// scratch token can hold the index without borrowing the table.
pub(crate) struct RuleSet {
    map: HashMap<Vec<char>, usize>,
    groups: Vec<Vec<Rule>>,
    /// Distinct pattern lengths, longest first.
    pub lengths: Vec<usize>,
}

impl RuleSet {
    /// The group index and rules of an exact pattern.
    pub fn get(&self, from: &[char]) -> Option<(usize, &[Rule])> {
        let &i = self.map.get(from)?;
        Some((i, self.groups[i].as_slice()))
    }

    /// The rules of a group found by `get`.
    pub fn group(&self, index: usize) -> &[Rule] {
        &self.groups[index]
    }

    fn from_map(table: HashMap<Vec<char>, Vec<Rule>>) -> RuleSet {
        let mut map = HashMap::with_capacity(table.len());
        let mut groups = Vec::with_capacity(table.len());
        for (from, rules) in table {
            map.insert(from, groups.len());
            groups.push(rules);
        }
        let mut lengths: Vec<usize> = map.keys().map(|k| k.len()).collect();
        lengths.sort_unstable_by(|a, b| b.cmp(a));
        lengths.dedup();
        RuleSet {
            map,
            groups,
            lengths,
        }
    }
}

type SourceRule = (&'static str, &'static str, u8);

/// The source rules of the built-in key that a target names, and those of its
/// `i` companion (`w` and `wi`: ү and үй).
#[derive(Debug, Clone, Default)]
pub(crate) struct Donor {
    rules: Vec<SourceRule>,
    companion: Vec<SourceRule>,
}

/// A pattern of the built-in table that a user may type: no Cyrillic input
/// (those rules only re-seed the buffer after the caret moves).
fn is_latin_pattern(from: &str) -> bool {
    from.chars().all(|c| (c as u32) >> 8 != 4)
}

/// Lowercase the output text of a source rule or a target.
fn lower_text(s: &str) -> String {
    s.chars().map(to_lower).collect()
}

fn source_group(from: &str) -> Vec<SourceRule> {
    SOURCE_RULES
        .iter()
        .filter(|r| r.0 == from)
        .copied()
        .collect()
}

/// Find the built-in key that a `[keys]` target names, and give its source
/// rules. A target is a lower case Cyrillic output (`ү`) or outputs joined
/// with `/` (`у/ү`). The key is the first one in table order whose rules give
/// exactly those outputs. `None` when no key does.
pub(crate) fn find_target(target: &str) -> Option<Donor> {
    let mut want: Vec<String> = target.split('/').map(lower_text).collect();
    if want.iter().any(|w| w.is_empty()) {
        return None;
    }
    want.sort();
    want.dedup();
    let mut seen: Vec<&'static str> = Vec::new();
    for &(from, _, flags) in SOURCE_RULES {
        if !is_latin_pattern(from) || seen.contains(&from) || flags & (X_DV | X_HK) != 0 {
            continue;
        }
        seen.push(from);
        let rules = source_group(from);
        let mut have: Vec<String> = rules.iter().map(|r| lower_text(r.1)).collect();
        have.sort();
        have.dedup();
        if have == want {
            let companion = source_group(&format!("{from}I"));
            return Some(Donor { rules, companion });
        }
    }
    None
}

/// What the built-in table gives per letter: `(letter, key)` pairs, with the
/// key in lower case. A rule with one output letter gives that letter from the
/// whole key. A rule that is a vowel plus `й` from a key ending in `i` gives
/// `й` from `i` (`ai` is ай). Used by calibration, once per call.
pub(crate) fn default_letter_chunks() -> Vec<(char, String)> {
    let mut out: Vec<(char, String)> = Vec::new();
    let mut push = |c: char, k: String| {
        if !out.iter().any(|(oc, ok)| *oc == c && *ok == k) {
            out.push((c, k));
        }
    };
    for &(from, to, flags) in SOURCE_RULES {
        // The habit keys give one letter, so push them below with the other
        // defaults; `ts` is not a target for `find_target` either.
        if !is_latin_pattern(from) || flags & X_HK != 0 {
            continue;
        }
        let key = lower_text(from);
        let letters: Vec<char> = lower_text(to).chars().collect();
        match letters.len() {
            1 => push(letters[0], key),
            2 if letters[1] == 'й' && key.ends_with('i') => push('й', "i".to_string()),
            _ => {}
        }
    }
    // Smart mode (`composer.rs`, the ий to ы rule) turns `ii` into ы after a
    // masculine stem (`aaviin` is аавын), and `ii'` forces ий. Typing these
    // is typing the built-in table, so they are not a custom habit.
    push('ы', "ii".to_string());
    // The double V (`vv`, үү) is a default too; it gives two letters.
    push('ү', "vv".to_string());
    // The habit keys (`ts`, `zh`) are defaults as well.
    push('ц', "ts".to_string());
    push('ж', "zh".to_string());
    push('й', "ii'".to_string());
    push('й', "i'".to_string());
    out
}

fn same_key(from: &[char], key: &[char]) -> bool {
    from.len() == key.len() && from.iter().zip(key).all(|(&a, &b)| to_lower(a) == b)
}

/// The built-in table, plus the changes of a key map. Each change is a key
/// (lowercase) and the donor to copy for it (no rules: remove the key). A
/// change also governs the key plus `i` (`wi` for `w`) when the donor or the
/// built-in key has such a companion, unless another entry names that key.
pub(crate) fn build_rules(changes: &[(Vec<char>, Donor)]) -> RuleSet {
    let mut table = HashMap::new();
    for &(from, to, flags) in SOURCE_RULES {
        add_rule(&mut table, from, to, flags);
    }
    for (key, donor) in changes {
        let mut comp_key = key.clone();
        comp_key.push('i');
        let builtin_companion = SOURCE_RULES
            .iter()
            .any(|r| same_key(&r.0.chars().collect::<Vec<_>>(), &comp_key));
        let comp_free = !changes.iter().any(|(k, _)| *k == comp_key);
        let comp = comp_free && (builtin_companion || !donor.companion.is_empty());
        table.retain(|from: &Vec<char>, _| {
            !same_key(from, key) && !(comp && same_key(from, &comp_key))
        });
        let upper: String = key.iter().map(|&c| to_upper(c)).collect();
        for &(_, to, flags) in &donor.rules {
            add_rule(&mut table, &upper, to, flags);
        }
        if comp {
            let upper = format!("{upper}I");
            for &(_, to, flags) in &donor.companion {
                add_rule(&mut table, &upper, to, flags);
            }
        }
    }
    RuleSet::from_map(table)
}

/// The umlaut letters accepted as input for ө and ү.
fn is_umlaut(ch: char) -> bool {
    matches!(ch, 'ö' | 'ü' | 'Ö' | 'Ü')
}

pub(crate) fn to_upper(ch: char) -> char {
    if ch.is_ascii() {
        ch.to_ascii_uppercase()
    } else if (ch as u32) >> 8 == 4 || is_umlaut(ch) {
        ch.to_uppercase().next().unwrap_or(ch)
    } else {
        ch
    }
}

pub(crate) fn to_lower(ch: char) -> char {
    if ch.is_ascii() {
        ch.to_ascii_lowercase()
    } else if (ch as u32) >> 8 == 4 || is_umlaut(ch) {
        ch.to_lowercase().next().unwrap_or(ch)
    } else {
        ch
    }
}

fn is_alpha(ch: char) -> bool {
    ch.is_ascii_alphabetic() || is_umlaut(ch) || ((ch as u32) >> 8 == 4 && ch.is_alphabetic())
}

fn insert_rule(map: &mut HashMap<Vec<char>, Vec<Rule>>, from: Vec<char>, rule: Rule) {
    map.entry(from).or_default().push(rule);
}

/// Port of `Composer::addRule`: when `X_AC` is set, the k-th alphabetic
/// character of `from` controls the case of the k-th character of `to`
/// (alphabetic characters beyond the length of `to` only affect `from`).
/// Characters of `to` that no key controls (a key shorter than its target)
/// follow the keys as a whole: lower case when every key is lower case.
/// Without `X_AC` the output never changes, but the key still works in every
/// case combination.
fn add_rule(map: &mut HashMap<Vec<char>, Vec<Rule>>, from: &str, to: &str, flags: u8) {
    let from: Vec<char> = from.chars().collect();
    let to: Vec<char> = to.chars().collect();
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
    let case = flags & X_AC != 0;
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
            if let (true, Some(tp)) = (case, pos_map[i]) {
                t[tp] = if upper {
                    to_upper(t[tp])
                } else {
                    to_lower(t[tp])
                };
            }
        }
        // Every key lower case: the whole output is lower case.
        if case && !alpha.is_empty() && mask == (1u32 << alpha.len()) - 1 {
            for c in t.iter_mut() {
                *c = to_lower(*c);
            }
        }
        insert_rule(map, f, Rule { to: t, flags });
    }
}

pub(crate) fn rules() -> &'static RuleSet {
    static RULES: OnceLock<RuleSet> = OnceLock::new();
    RULES.get_or_init(|| build_rules(&[]))
}

/// Human-readable dump of the expanded table (`from = to`), sorted; useful
/// for docs and for diffing against the original IME's debug export.
pub fn export_rules() -> Vec<(String, String, bool, bool)> {
    let mut out = Vec::new();
    for (from, &i) in &rules().map {
        for r in rules()
            .group(i)
            .iter()
            .filter(|r| r.flags & (X_DV | X_HK) == 0)
        {
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

/// The Buuz typing rules as a key table for people, sorted by the Latin
/// keys. Only lower-case keys are listed. Vowel + i pairs (ai gives ай) are
/// left out: they follow from the vowel and i. A key with two outputs (o gives
/// о or ө) is one `Harmony` row. Keys of the `double_v` and `habit_keys`
/// options are `Smart` rows.
pub fn typing_key_table() -> Vec<crate::KeyRow> {
    use crate::KeyWhen;
    let lower_key = |c: char| c.is_ascii_lowercase() || matches!(c, '\'' | '"' | 'ö' | 'ü');
    let mut rows: Vec<crate::KeyRow> = Vec::new();
    for (from, &i) in &rules().map {
        if !from.iter().all(|&c| lower_key(c)) {
            continue;
        }
        let latin: String = from.iter().collect();
        let pair = from.len() >= 2 && from[from.len() - 1] == 'i' && !latin.starts_with("ii");
        if pair {
            continue;
        }
        let group = rules().group(i);
        let smart = |r: &Rule| r.flags & (X_DV | X_HK) != 0;
        let plain: Vec<String> = group
            .iter()
            .filter(|r| !smart(r))
            .map(|r| r.to.iter().collect())
            .collect();
        let mut outs: Vec<String> = Vec::new();
        for o in plain {
            if !outs.contains(&o) {
                outs.push(o);
            }
        }
        if !outs.is_empty() {
            let when = if outs.len() > 1 {
                KeyWhen::Harmony
            } else {
                KeyWhen::Always
            };
            rows.push(crate::KeyRow {
                latin: latin.clone(),
                cyrillic: outs.join("/"),
                when,
            });
        }
        if let Some(r) = group.iter().find(|r| smart(r)) {
            rows.push(crate::KeyRow {
                latin,
                cyrillic: r.to.iter().collect(),
                when: KeyWhen::Smart,
            });
        }
    }
    rows.sort_by(|a, b| a.latin.cmp(&b.latin));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_target_still_gives_the_c_donor_for_ц() {
        let donor = find_target("ц").unwrap();
        assert!(donor.rules.iter().all(|r| r.0 == "C" && r.2 & X_HK == 0));
        let donor = find_target("ж").unwrap();
        assert!(donor.rules.iter().all(|r| r.0 == "J" && r.2 & X_HK == 0));
    }

    #[test]
    fn export_rules_is_the_table_without_the_option_rules() {
        let mut map = HashMap::new();
        for &(from, to, flags) in SOURCE_RULES {
            if flags & (X_DV | X_HK) == 0 {
                add_rule(&mut map, from, to, flags);
            }
        }
        let mut want = Vec::new();
        for (from, rules) in &map {
            for r in rules {
                want.push((
                    from.iter().collect::<String>(),
                    r.to.iter().collect::<String>(),
                    r.flags & X_M != 0,
                    r.flags & X_F != 0,
                ));
            }
        }
        want.sort();
        assert_eq!(export_rules(), want);
    }
}
