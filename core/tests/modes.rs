// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. Smart mode (the default, every smart feature on) and classic
// mode (the original Buuz, unchanged). See docs/glossary.md.

use buuz_core::{convert_with, Options};

fn smart(input: &str) -> String {
    convert_with(input, &Options::smart())
}

fn classic(input: &str) -> String {
    convert_with(input, &Options::classic())
}

#[test]
fn smart_mode_turns_every_smart_feature_on() {
    let o = Options::smart();
    assert!(o.smart_vowels);
    assert!(o.abbreviations.is_some());
    assert_eq!(smart("unee"), "үнээ");
    assert_eq!(smart("aaviin"), "аавын");
    assert_eq!(smart("bn uu "), "байна уу ");
}

#[test]
fn classic_mode_is_the_original_buuz() {
    let o = Options::classic();
    assert!(!o.smart_vowels);
    assert!(o.abbreviations.is_none());
    assert_eq!(classic("unee"), "унээ");
    assert_eq!(classic("aaviin"), "аавийн");
    assert_eq!(classic("irsen uu"), "ирсэн уу");
    assert_eq!(classic("bn uu "), "бн уу ");
}

#[test]
fn both_modes_share_the_plain_letter_table() {
    for w in ["sain baina", "geree", "gar", "shine", "wnen", "qvqr", "id'ye"] {
        assert_eq!(smart(w), classic(w), "input {w:?}");
    }
}
