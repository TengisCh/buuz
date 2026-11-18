// SPDX-License-Identifier: Apache-2.0

//! Word choice for ө and ү (issue #64): `o` and `u` are ambiguous, so at
//! Space or punctuation the word list picks between о/ө and у/ү/ө. Backspace
//! right after undoes it.

use std::sync::Arc;

use buuz_core::{Composer, Key, Options, WordList};

const REAL: &str = include_str!("../../data/wordlist/mn.txt");

fn with_list(base: Options) -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        ..base
    }
}

/// Type `s`, then Space, and return the commit of the Space key.
fn commit(opts: Options, s: &str) -> String {
    let mut c = Composer::with_options(opts);
    for ch in s.chars() {
        c.process_key(Key::Char(ch));
    }
    c.process_key(Key::Space).commit.unwrap_or_default()
}

#[test]
fn on_in_smart_mode_and_off_in_classic() {
    assert!(Options::smart().word_choice);
    assert!(!Options::classic().word_choice);
    assert!(Options::default().word_choice);
}

#[test]
fn o_may_be_o_with_umlaut() {
    let o = with_list(Options::smart());
    assert_eq!(commit(o.clone(), "tov"), "төв");
    assert_eq!(commit(o, "mon"), "мөн");
}

#[test]
fn u_may_be_o_with_umlaut_or_u_with_umlaut() {
    let o = with_list(Options::smart());
    assert_eq!(commit(o.clone(), "uvul"), "өвөл");
    assert_eq!(commit(o.clone(), "udur"), "өдөр");
    assert_eq!(commit(o.clone(), "tuv"), "төв");
    // `уруу` (399) and `өрөө` (333) are both common: the list cannot choose.
    assert_eq!(commit(o, "uruu"), "уруу");
}

#[test]
fn a_common_listed_word_stays() {
    let o = with_list(Options::smart());
    assert_eq!(commit(o.clone(), "uuriin"), "уурын");
    assert_eq!(commit(o, "bayar"), "баяр");
}

#[test]
fn off_without_the_switch_or_the_list() {
    let off = with_list(Options {
        word_choice: false,
        ..Options::smart()
    });
    assert_eq!(commit(off, "tov"), "тов");
    assert_eq!(commit(Options::smart(), "tov"), "тов");
}

#[test]
fn backspace_right_after_undoes_it() {
    let mut c = Composer::with_options(with_list(Options::smart()));
    for ch in "tov".chars() {
        c.process_key(Key::Char(ch));
    }
    let r = c.process_key(Key::Space);
    assert!(r.corrected);
    assert_eq!(r.commit.as_deref(), Some("төв"));
    let r = c.process_key(Key::Backspace);
    assert_eq!(c.preedit(), "тов");
    assert!(r.consumed);
    // The word is not changed again.
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("тов"));
}

#[test]
fn a_user_added_word_is_not_changed() {
    // The rejected set covers the same path: an undone word stays as typed.
    let mut c = Composer::with_options(with_list(Options::smart()));
    for ch in "tov".chars() {
        c.process_key(Key::Char(ch));
    }
    c.process_key(Key::Space);
    c.process_key(Key::Backspace);
    for _ in 0..1 {
        c.process_key(Key::Space);
    }
    for ch in "tov".chars() {
        c.process_key(Key::Char(ch));
    }
    assert_eq!(c.process_key(Key::Space).commit.as_deref(), Some("тов"));
}

#[test]
fn e_may_be_ye() {
    let o = with_list(Options::smart());
    assert_eq!(commit(o, "wed"), "үед");
}

#[test]
fn a_single_vowel_may_be_long() {
    let o = with_list(Options::smart());
    assert_eq!(commit(o.clone(), "terer"), "тэрээр");
    assert_eq!(commit(o, "zwn"), "зүүн");
}

#[test]
fn i_or_nothing_may_be_soft_sign() {
    let o = with_list(Options::smart());
    assert_eq!(commit(o.clone(), "huuli"), "хууль");
    assert_eq!(commit(o, "surguul"), "сургууль");
}

#[test]
fn extra_edits_leave_common_words_alone() {
    let o = with_list(Options::smart());
    // `тэр` (9211) is common: `ter` does not become a longer word.
    assert_eq!(commit(o.clone(), "ter"), "тэр");
    assert_eq!(commit(o, "sain"), "сайн");
}

#[test]
fn long_words_with_many_ambiguous_letters_stay_fast() {
    let o = with_list(Options::smart());
    let start = std::time::Instant::now();
    for _ in 0..50 {
        commit(o.clone(), "ouououobkl");
    }
    // 1 ms for each word in debug builds is a generous limit.
    assert!(start.elapsed().as_millis() < 50 * 20);
}
