// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors

//! The key tables the website shows: one row per Latin key and its Cyrillic.

use buuz_core::{typing_key_table, KeyRow, KeyWhen, Scheme};

fn has(rows: &[KeyRow], latin: &str, cyrillic: &str, when: KeyWhen) -> bool {
    rows.iter()
        .any(|r| r.latin == latin && r.cyrillic == cyrillic && r.when == when)
}

#[test]
fn buuz_table_has_letters_and_harmony_vowels() {
    let t = typing_key_table();
    assert!(has(&t, "q", "ө", KeyWhen::Always));
    assert!(has(&t, "sh", "ш", KeyWhen::Always));
    assert!(has(&t, "ya", "я", KeyWhen::Always));
    assert!(has(&t, "o", "о/ө", KeyWhen::Harmony));
    assert!(has(&t, "u", "у/ү", KeyWhen::Harmony));
    assert!(has(&t, "'", "ь", KeyWhen::Always));
}

#[test]
fn buuz_table_marks_smart_keys() {
    let t = typing_key_table();
    assert!(has(&t, "vv", "үү", KeyWhen::Smart));
    assert!(has(&t, "ts", "ц", KeyWhen::Smart));
    assert!(has(&t, "zh", "ж", KeyWhen::Smart));
}

#[test]
fn buuz_table_has_no_capitals_and_no_vowel_plus_i_pairs() {
    let t = typing_key_table();
    assert!(t.iter().all(|r| !r.latin.chars().any(char::is_uppercase)));
    assert!(t
        .iter()
        .all(|r| r.latin.chars().all(|c| !matches!(c, 'а'..='я' | 'ө' | 'ү'))));
    assert!(!t
        .iter()
        .any(|r| r.latin == "ai" || r.latin == "oi" || r.latin == "vvi"));
    // ii is a key of its own, not a vowel plus i.
    assert!(t.iter().any(|r| r.latin == "ii"));
}

#[test]
fn scheme_table_shows_conditions() {
    let t = Scheme::Russian.key_table();
    assert!(has(&t, "zh", "ж", KeyWhen::Always));
    assert!(has(&t, "y", "й", KeyWhen::AfterVowel));
    assert!(has(&t, "y", "ы", KeyWhen::Always));
    assert!(has(&t, "et", "эт", KeyWhen::WordStart));
}

#[test]
fn every_scheme_has_a_table_and_plain_rows_convert() {
    for s in Scheme::ALL {
        let t = s.key_table();
        assert!(t.len() > 20, "{} has {} rows", s.id(), t.len());
        for r in t.iter().filter(|r| r.when == KeyWhen::Always) {
            // A plain row converts to its own output, with a letter before it
            // for the rows that need one (the soft and hard signs).
            let alone = s.convert(&r.latin);
            let after = s.convert(&format!("t{}", r.latin));
            assert!(
                alone == r.cyrillic || after.ends_with(&r.cyrillic),
                "{}: {} gives {alone} / {after}, not {}",
                s.id(),
                r.latin,
                r.cyrillic
            );
        }
    }
}
