// SPDX-License-Identifier: Apache-2.0

//! Issue #62: `ii` after a stem that ends in ь (сургууль, хууль) gives ий in
//! smart mode when the word list knows the ий word and not the ы word.

use std::sync::Arc;

use buuz_core::{convert_with, Options, WordList};

const FIXTURE: &str = "\
сургуулийн\t100
хуулийн\t90
хувийн\t80
уулын\t70
аавын\t60
номыг\t50
";

fn with(words: &str, opts: Options) -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(words).unwrap())),
        ..opts
    }
}

fn smart() -> Options {
    with(FIXTURE, Options::smart())
}

fn conv(input: &str) -> String {
    convert_with(input, &smart())
}

#[test]
fn list_gives_ii_when_only_the_ii_word_is_listed() {
    assert_eq!(conv("surguuliin "), "сургуулийн ");
    assert_eq!(conv("huuliin "), "хуулийн ");
    assert_eq!(conv("huviin "), "хувийн ");
}

#[test]
fn list_keeps_y_when_the_y_word_is_listed() {
    assert_eq!(conv("uuliin "), "уулын ");
    assert_eq!(conv("aaviin "), "аавын ");
    assert_eq!(conv("nomiig "), "номыг ");
}

#[test]
fn case_is_kept() {
    assert_eq!(conv("Surguuliin "), "Сургуулийн ");
    assert_eq!(conv("SURGUULIIN "), "СУРГУУЛИЙН ");
    assert_eq!(conv("Uuliin "), "Уулын ");
}

#[test]
fn explicit_keys_still_force() {
    assert_eq!(conv("surguulyn "), "сургуулын ");
    assert_eq!(conv("surguuliiin "), "сургуулын ");
    assert_eq!(conv("uulii'n "), "уулийн ");
}

#[test]
fn preedit_shows_ii_at_once() {
    use buuz_core::{Composer, Key};
    let mut c = Composer::with_options(smart());
    for ch in "surguuli".chars() {
        c.process_key(Key::Char(ch));
    }
    c.process_key(Key::Char('i'));
    assert_eq!(c.preedit(), "сургуулий");
}

#[test]
fn without_a_list_nothing_changes() {
    let o = Options::smart();
    assert_eq!(convert_with("surguuliin ", &o), "сургуулын ");
    assert_eq!(convert_with("huviin ", &o), "хувын ");
}

#[test]
fn smart_vowels_off_and_classic_are_unchanged() {
    let off = Options {
        smart_vowels: false,
        ..smart()
    };
    assert_eq!(convert_with("surguuliin ", &off), "сургуулийн ");
    let classic = with(FIXTURE, Options::classic());
    assert_eq!(convert_with("surguuliin ", &classic), "сургуулийн ");
    assert_eq!(convert_with("uuliin ", &classic), "уулийн ");
}

#[test]
fn real_word_list() {
    let real = with(include_str!("../../data/wordlist/mn.txt"), Options::smart());
    for (typed, want) in [
        ("surguuliin ", "сургуулийн "),
        ("huuliin ", "хуулийн "),
        ("huviin ", "хувийн "),
        ("uuliin ", "уулын "),
        ("aaviin ", "аавын "),
        ("nomiig ", "номыг "),
    ] {
        assert_eq!(convert_with(typed, &real), want, "{typed}");
    }
}
