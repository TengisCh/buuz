// SPDX-License-Identifier: Apache-2.0

//! Rounding harmony (issue #141): at Space or punctuation, the word list
//! changes а to о after a first о (or э to ө after a first ө). `bolsan`
//! gives `болсон`. Backspace right after Space undoes it.

use std::sync::Arc;
use std::time::Instant;

use buuz_core::{convert_with, Composer, Key, Options, WordList};

const REAL: &str = include_str!("../../data/wordlist/mn.txt");

fn real() -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        ..Options::smart()
    }
}

fn type_in(c: &mut Composer, s: &str) {
    for ch in s.chars() {
        c.process_key(Key::Char(ch));
    }
}

/// Type `s`, then Space, and return the commit of the Space key.
fn commit(opts: Options, s: &str) -> String {
    let mut c = Composer::with_options(opts);
    type_in(&mut c, s);
    c.process_key(Key::Space).commit.unwrap_or_default()
}

#[test]
fn on_in_smart_and_off_in_classic() {
    assert!(Options::smart().rounding_harmony);
    assert!(Options::default().rounding_harmony);
    assert!(!Options::classic().rounding_harmony);
}

#[test]
fn rounds_after_a_first_o() {
    for (typed, want) in [
        ("bolsan", "болсон"),
        ("bolna", "болно"),
        ("boldag", "болдог"),
        ("bolaad", "болоод"),
        ("bollaa", "боллоо"),
        ("toglosan", "тоглосон"),
        ("hotaar", "хотоор"),
        ("mongolaar", "монголоор"),
        ("dotaad", "дотоод"),
    ] {
        assert_eq!(commit(real(), typed), want, "{typed}");
    }
}

#[test]
fn rounds_after_a_first_o_with_umlaut() {
    for (typed, want) in [("ogsen", "өгсөн"), ("ogne", "өгнө"), ("hogjsen", "хөгжсөн")]
    {
        assert_eq!(commit(real(), typed), want, "{typed}");
    }
}

#[test]
fn guards_keep_the_word() {
    for (typed, want) in [
        ("togluulsan", "тоглуулсан"),
        ("bolohgui", "болохгүй"),
        ("romanaas", "романаас"),
        ("dollar", "доллар"),
        ("ogchee", "өгчээ"),
        ("tqstei", "төстэй"),
    ] {
        assert_eq!(commit(real(), typed), want, "{typed}");
    }
}

#[test]
fn leading_capital_stays_and_all_capitals_stay() {
    assert_eq!(commit(real(), "Bolsan"), "Болсон");
    assert_eq!(commit(real(), "BOLSAN"), "БОЛСАН");
}

#[test]
fn off_in_classic_mode_with_the_switch_off_or_without_list() {
    let classic = Options {
        words: real().words,
        ..Options::classic()
    };
    assert_eq!(commit(classic, "bolsan"), "болсан");
    let off = Options {
        rounding_harmony: false,
        ..real()
    };
    assert_eq!(commit(off, "bolsan"), "болсан");
    assert_eq!(commit(Options::smart(), "bolsan"), "болсан");
}

#[test]
fn backspace_undoes_and_rejects() {
    let mut c = Composer::with_options(real());
    type_in(&mut c, "bolsan");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("болсон"));
    assert!(r.corrected);
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(c.preedit(), "болсан");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("болсан"));
    // The word is not rounded again in this session.
    type_in(&mut c, "bolsan");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("болсан"));
}

#[test]
fn punctuation_runs_it() {
    let mut c = Composer::with_options(real());
    type_in(&mut c, "bolsan");
    let r = c.process_key(Key::Symbol('.'));
    assert_eq!(r.commit.as_deref(), Some("болсон"));
    assert_eq!(convert_with("bolsan.", &real()), "болсон.");
}

#[test]
fn whole_text_converter_runs_it_at_the_end_and_before_a_new_line() {
    let o = real();
    assert_eq!(convert_with("bolsan", &o), "болсон");
    assert_eq!(convert_with("bolsan\nogsen", &o), "болсон\nөгсөн");
    assert_eq!(convert_with("bolsan ", &o), "болсон ");
    let only = Options {
        word_choice: false,
        suffix_choice: false,
        ..real()
    };
    assert_eq!(convert_with("bolsan", &only), "болсон");
}

#[test]
fn overlaps_with_suffix_choice() {
    // With suffix choice off, rounding harmony still rounds an ablative.
    let o = Options {
        suffix_choice: false,
        ..real()
    };
    assert_eq!(commit(o, "hotaas"), "хотоос");
    let both_off = Options {
        suffix_choice: false,
        rounding_harmony: false,
        ..real()
    };
    assert_eq!(commit(both_off, "hotaas"), "хотаас");
}

#[test]
fn latin_view_stays() {
    let mut c = Composer::with_options(real());
    type_in(&mut c, "bolsan");
    c.process_key(Key::Escape);
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("bolsan"));
}

#[test]
fn latin_run_stays() {
    let out = convert_with("a@bolsan ", &real());
    assert_eq!(out, "a@bolsan ");
}

#[test]
fn a_reseeded_word_stays() {
    let mut c = Composer::with_options(real());
    type_in(&mut c, "bolsan");
    c.process_key(Key::Left);
    c.process_key(Key::Right);
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("болсан"));
}

#[test]
fn a_word_reopened_by_undo_stays() {
    // The reopened word has the no-correct flag: Space keeps it, also after
    // an edit of the same word.
    let mut c = Composer::with_options(real());
    type_in(&mut c, "bolsan");
    c.process_key(Key::Space);
    c.process_key(Key::Backspace);
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("болсан"));
}

#[test]
fn word_choice_wins_first() {
    assert_eq!(commit(real(), "tov"), "төв");
}

#[test]
fn a_user_added_word_stays() {
    use buuz_core::UserDict;
    let mut d = UserDict::new();
    assert!(d.add("болсан"));
    let o = Options {
        user: Some(Arc::new(d.snapshot())),
        ..real()
    };
    assert_eq!(commit(o, "bolsan"), "болсан");
}

#[test]
fn small_list_rules() {
    let small = |list: &str| Options {
        words: Some(Arc::new(WordList::parse(list).unwrap())),
        word_choice: false,
        suffix_choice: false,
        ..Options::smart()
    };
    // Too rare: under 50.
    assert_eq!(commit(small("болсон\t49\n"), "bolsan"), "болсан");
    // 50 times the typed word: 200 against 3.
    assert_eq!(
        commit(small("болсон\t200\nболсан\t3\n"), "bolsan"),
        "болсон"
    );
    // Not 50 times the typed word: 100 against 10.
    assert_eq!(
        commit(small("болсон\t100\nболсан\t10\n"), "bolsan"),
        "болсан"
    );
    // Two candidates that are close: no clear winner.
    assert_eq!(commit(small("болоа\t100\nболао\t100\n"), "bolaa"), "болаа");
    // More than 6 positions: no change.
    let list = "болололололололо\t500\n";
    assert_eq!(commit(small(list), "bolalalalalalala"), "болалалалалалала");
}

#[test]
fn rounding_harmony_is_fast_per_word_on_the_real_list() {
    let mut c = Composer::with_options(real());
    let words = [
        "bolsan",
        "toglosan",
        "bolalalalalalala",
        "bolaaaaaaaaaaaaa",
        "ogsenenenenenen",
        "hogjsen",
        "bolalalalalala",
        "bolalalalalal",
        "bolaaaaaaaaaaa",
        "qwqwqwqwqwqwqw",
    ];
    let mut samples = Vec::new();
    for round in 0..11 {
        for w in words {
            type_in(&mut c, w);
            let start = Instant::now();
            let r = c.process_key(Key::Space);
            let took = start.elapsed().as_nanos();
            std::hint::black_box(r);
            if round > 0 {
                samples.push(took);
            }
        }
    }
    samples.sort_unstable();
    let p99 = samples[(samples.len() * 99 / 100).min(samples.len() - 1)];
    eprintln!(
        "rounding harmony: p99 {p99} ns, max {} ns",
        samples[samples.len() - 1]
    );
    if !cfg!(debug_assertions) {
        assert!(p99 < 1_000_000, "p99 {p99} ns");
    }
}
