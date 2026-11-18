// SPDX-License-Identifier: Apache-2.0

//! Tab completion (issue #2): the ghost text, Tab and Right to accept it.

use std::sync::Arc;
use std::time::Instant;

use buuz_core::{
    convert_with, Composer, Config, Key, KeyMap, KeyOutcome, Options, Preset, WordList,
};

const FIXTURE: &str = "\
баярлалаа\t100
баяр\t50
баярлах\t40
сайн\t200
сайхан\t150
гэрлэх\t30
үнэн\t20
гоогл\t10
багтэй\t10
";

const REAL: &str = include_str!("../../data/wordlist/mn.txt");

fn with_list(base: Options) -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(FIXTURE).unwrap())),
        ..base
    }
}

fn composer() -> Composer {
    Composer::with_options(with_list(Options::smart()))
}

fn type_str(c: &mut Composer, s: &str) {
    for ch in s.chars() {
        c.process_key(Key::Char(ch));
    }
}

#[test]
fn off_by_default_and_in_every_preset() {
    assert!(Options::default().words.is_none());
    assert!(Options::smart().words.is_none());
    assert!(Options::classic().words.is_none());
    assert!(Options::default().completion);
    assert!(Options::smart().completion);
    assert!(!Options::classic().completion);
}

#[test]
fn ghost_shows_the_rest_of_the_clear_winner() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    assert_eq!(c.preedit(), "баярл");
    assert_eq!(c.completion(), Some("алаа"));
}

#[test]
fn tab_accepts_and_space_commits() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    assert!(c.wants_key(Key::Tab));
    let out = c.process_key(Key::Tab);
    assert!(out.consumed);
    assert_eq!(out.commit, None);
    assert_eq!(c.preedit(), "баярлалаа");
    assert_eq!(c.cursor(), 9);
    assert_eq!(c.completion(), None);
    let out = c.process_key(Key::Space);
    assert!(!out.consumed);
    assert_eq!(out.commit.as_deref(), Some("баярлалаа"));
}

#[test]
fn fewer_than_three_letters_no_ghost() {
    let mut c = composer();
    type_str(&mut c, "ba");
    assert_eq!(c.completion(), None);
    // "ба" has no ghost, the third letter brings it.
    type_str(&mut c, "ya");
    assert_eq!(c.preedit(), "бая");
    let mut c = composer();
    type_str(&mut c, "gerl");
    c.process_key(Key::Backspace);
    c.process_key(Key::Backspace);
    assert_eq!(c.preedit(), "гэ");
    assert_eq!(c.completion(), None);
}

#[test]
fn ambiguous_prefix_no_ghost() {
    let mut c = composer();
    // сай: сайн 200, сайхан 150 are within 2x
    type_str(&mut c, "sai");
    assert_eq!(c.preedit(), "сай");
    assert_eq!(c.completion(), None);
    assert!(!c.wants_key(Key::Tab));
}

#[test]
fn only_candidate_counts_as_clear() {
    let mut c = composer();
    type_str(&mut c, "gerl");
    assert_eq!(c.preedit(), "гэрл");
    assert_eq!(c.completion(), Some("эх"));
}

#[test]
fn tab_without_ghost_passes_through_and_commits() {
    let mut c = composer();
    type_str(&mut c, "ba");
    assert!(!c.wants_key(Key::Tab));
    let out = c.process_key(Key::Tab);
    assert!(!out.consumed);
    assert_eq!(out.commit.as_deref(), Some("ба"));
    assert!(!c.is_composing());
}

#[test]
fn upper_case_preedit_gives_upper_case_ghost() {
    let mut c = composer();
    type_str(&mut c, "BAYARL");
    assert_eq!(c.preedit(), "БАЯРЛ");
    assert_eq!(c.completion(), Some("АЛАА"));
}

#[test]
fn capitalised_preedit_gives_lower_case_ghost() {
    let mut c = composer();
    type_str(&mut c, "Bayarl");
    assert_eq!(c.preedit(), "Баярл");
    assert_eq!(c.completion(), Some("алаа"));
    c.process_key(Key::Tab);
    assert_eq!(c.preedit(), "Баярлалаа");
}

#[test]
fn latin_view_has_no_ghost() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    c.process_key(Key::Escape);
    assert_eq!(c.preedit(), "bayarl");
    assert_eq!(c.completion(), None);
    assert!(!c.wants_key(Key::Tab));
    c.process_key(Key::Escape);
    assert_eq!(c.completion(), Some("алаа"));
}

#[test]
fn caret_not_at_end_or_reseeded_has_no_ghost() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    c.process_key(Key::Left);
    assert_eq!(c.completion(), None);
    c.process_key(Key::End);
    // re-seeded words never complete, even with the caret back at the end
    assert_eq!(c.completion(), None);
}

#[test]
fn a_suffix_after_tab_still_composes_with_the_gender_evidence() {
    // гэрлэх is feminine: ii after it stays ий, it does not turn into ы.
    let mut c = composer();
    type_str(&mut c, "gerl");
    c.process_key(Key::Tab);
    assert_eq!(c.preedit(), "гэрлэх");
    type_str(&mut c, "ii");
    assert_eq!(c.preedit(), "гэрлэхий");
    // and a masculine word takes ы
    let mut c = composer();
    type_str(&mut c, "saih");
    c.process_key(Key::Tab);
    assert_eq!(c.preedit(), "сайхан");
    type_str(&mut c, "ii");
    assert_eq!(c.preedit(), "сайханы");
    let out = c.process_key(Key::Space);
    assert_eq!(out.commit.as_deref(), Some("сайханы"));
}

#[test]
fn right_at_the_end_accepts() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    let out = c.process_key(Key::Right);
    assert!(out.consumed);
    assert_eq!(c.preedit(), "баярлалаа");
    assert_eq!(c.cursor(), 9);
}

#[test]
fn right_without_ghost_or_inside_the_word_keeps_its_meaning() {
    let mut c = composer();
    type_str(&mut c, "ba");
    let out = c.process_key(Key::Right);
    assert!(out.consumed);
    assert_eq!(c.preedit(), "ба");
    assert_eq!(c.cursor(), 2);
    let mut c = composer();
    type_str(&mut c, "bayarl");
    c.process_key(Key::Left);
    c.process_key(Key::Right);
    assert_eq!(c.preedit(), "баярл");
}

#[test]
fn classic_mode_passes_tab_through() {
    let mut c = Composer::with_options(with_list(Options::classic()));
    type_str(&mut c, "bayarl");
    // classic with a list set by hand: completion is off, typing is unchanged
    assert!(!c.wants_key(Key::Tab));
    assert_eq!(c.completion(), None);
    let mut plain = Composer::with_options(Options::classic());
    type_str(&mut plain, "bayarl");
    assert_eq!(c.preedit(), plain.preedit());
    let mut c = Composer::with_options(Options::classic());
    type_str(&mut c, "bayarl");
    assert!(!c.wants_key(Key::Tab));
    let out = c.process_key(Key::Tab);
    assert_eq!(
        out,
        KeyOutcome {
            consumed: false,
            commit: Some("баярл".into()),
            rejected: false,
            delete_before: 0
        }
    );
    assert_eq!(c.completion(), None);
}

#[test]
fn tab_without_a_word_list_is_exactly_other() {
    for keys in ["sain", "bayarl", "x", ""] {
        let mut a = Composer::with_options(Options::smart());
        let mut b = Composer::with_options(Options::smart());
        type_str(&mut a, keys);
        type_str(&mut b, keys);
        assert_eq!(a.wants_key(Key::Tab), a.wants_key(Key::Other));
        assert_eq!(a.process_key(Key::Tab), b.process_key(Key::Other));
        assert_eq!(a.preedit(), b.preedit());
    }
}

#[test]
fn convert_with_is_unchanged_by_a_word_list() {
    for text in ["bayarlalaa sain baina uu", "BAYARL bn gerl"] {
        for base in [Options::smart(), Options::classic()] {
            let with = with_list(base.clone());
            assert_eq!(convert_with(text, &with), convert_with(text, &base));
        }
    }
}

#[test]
fn custom_keys_use_the_converted_preedit() {
    let keys = KeyMap::from_pairs(&[("v".to_string(), "ү".to_string())]).unwrap();
    let opts = Options {
        keys: Some(Arc::new(keys)),
        ..with_list(Options::smart())
    };
    let mut c = Composer::with_options(opts);
    type_str(&mut c, "vne");
    assert_eq!(c.preedit(), "үнэ");
    assert_eq!(c.completion(), Some("н"));
}

#[test]
fn a_word_equal_to_the_preedit_has_no_ghost() {
    let mut c = composer();
    type_str(&mut c, "sain");
    assert_eq!(c.completion(), None);
}

#[test]
fn config_switch_follows_the_preset() {
    let smart = Config::default();
    assert!(smart.needs_word_list());
    assert!(smart.options().completion);
    let mut classic = Config::default();
    classic.set_preset(Preset::Classic);
    assert!(!classic.needs_word_list());
    classic.set_completion(Some(true));
    assert!(classic.needs_word_list());
    smart.options(); // options() never loads data
    assert!(smart.options().words.is_none());
    let parsed = Config::parse("buuz_config = 1\n[options]\ncompletion = false\n")
        .unwrap()
        .config;
    assert_eq!(parsed.completion(), Some(false));
    assert!(!parsed.needs_word_list());
    assert_eq!(Config::parse(&parsed.to_text()).unwrap().config, parsed);
}

/// Key plus query against the 10 us budget of docs/architecture.md section 4.
/// The assert runs in release only; CI runs the debug build, where it only
/// prints the numbers.
#[test]
fn completion_is_fast_per_key_on_the_real_list() {
    let opts = Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        ..Options::smart()
    };
    let words = [
        "bayarlalaa",
        "mongol",
        "hun",
        "sain",
        "tuhai",
        "ulsiin",
        "bolovsrol",
    ];
    let mut c = Composer::with_options(opts);
    let mut n = 0u128;
    let mut total = 0u128;
    let mut worst = 0u128;
    // The first round warms the caches and buffers; only later rounds count.
    for round in 0..21 {
        for w in words {
            for ch in w.chars() {
                // The ghost is computed inside the key, so time key plus ask.
                let start = Instant::now();
                c.process_key(Key::Char(ch));
                let g = c.completion();
                let took = start.elapsed().as_nanos();
                std::hint::black_box(g);
                if round > 0 {
                    n += 1;
                    total += took;
                    worst = worst.max(took);
                }
            }
            c.process_key(Key::Space);
        }
    }
    eprintln!("completion(): mean {} ns, worst {} ns", total / n, worst);
    if !cfg!(debug_assertions) {
        assert!(worst < 10_000, "worst {worst} ns, mean {} ns", total / n);
    }
}

/// Feed keys the way a shell does: apply `delete_before` and `commit`, and
/// pass through what the composer does not consume.
fn feed(c: &mut Composer, keys: &[Key]) -> String {
    let mut out = String::new();
    for &k in keys {
        let r = c.process_key(k);
        for _ in 0..r.delete_before {
            out.pop();
        }
        if let Some(s) = r.commit {
            out.push_str(&s);
        }
        if !r.consumed {
            match k {
                Key::Space => out.push(' '),
                Key::Symbol(ch) => out.push(ch),
                _ => {}
            }
        }
    }
    out
}

fn chars(s: &str) -> Vec<Key> {
    s.chars().map(Key::Char).collect()
}

#[test]
fn an_accepted_word_reopens_after_space() {
    let mut c = composer();
    let mut keys = chars("bayarl");
    keys.extend([Key::Tab, Key::Space]);
    assert_eq!(feed(&mut c, &keys), "баярлалаа ");
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(r.delete_before, 10);
    assert_eq!(c.preedit(), "баярлалаа");
    assert_eq!(c.cursor(), 9);
    assert_eq!(c.completion(), None);
    // still frozen: Esc does nothing, and the next Space commits it
    assert!(c.process_key(Key::Escape).consumed);
    assert_eq!(c.preedit(), "баярлалаа");
    assert_eq!(
        c.process_key(Key::Space).commit.as_deref(),
        Some("баярлалаа")
    );
}

#[test]
fn a_word_frozen_by_the_caret_reopens_and_stays_frozen() {
    let mut c = composer();
    type_str(&mut c, "sain");
    c.process_key(Key::Left);
    c.process_key(Key::End);
    c.process_key(Key::Space);
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(r.delete_before, 5);
    assert_eq!(c.preedit(), "сайн");
    c.process_key(Key::Escape);
    assert_eq!(c.preedit(), "сайн");
    // a frozen word never expands, also after a reopen
    let mut c = composer();
    type_str(&mut c, "bn");
    c.process_key(Key::Left);
    c.process_key(Key::End);
    c.process_key(Key::Space);
    c.process_key(Key::Backspace);
    assert_eq!(c.process_key(Key::Space).commit.as_deref(), Some("бн"));
}

#[test]
fn tab_then_escape_is_a_consumed_no_op() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    c.process_key(Key::Tab);
    let r = c.process_key(Key::Escape);
    assert!(r.consumed);
    assert_eq!(r.commit, None);
    assert_eq!(c.preedit(), "баярлалаа");
}

#[test]
fn auto_latin_stops_for_the_rest_of_the_run_after_an_accept() {
    let mut c = composer();
    let mut keys = chars("goog");
    keys.extend([Key::Tab, Key::Symbol('.')]);
    keys.extend(chars("com"));
    keys.push(Key::Space);
    assert_eq!(feed(&mut c, &keys), "гоогл.цом ");
}

#[test]
fn a_run_that_turned_latin_has_no_ghost() {
    let mut c = composer();
    c.process_key(Key::Char('a'));
    c.process_key(Key::Symbol('@'));
    type_str(&mut c, "bayarl");
    assert_eq!(c.preedit(), "bayarl");
    assert_eq!(c.completion(), None);
}

#[test]
fn the_particle_follows_the_gender_of_an_accepted_word() {
    // гэрлэх is feminine, баярлалаа masculine.
    let mut c = composer();
    let mut keys = chars("gerl");
    keys.extend([Key::Tab, Key::Space]);
    keys.extend(chars("uu"));
    assert_eq!(feed(&mut c, &keys), "гэрлэх ");
    assert_eq!(c.preedit(), "үү");
    let mut c = composer();
    let mut keys = chars("bayarl");
    keys.extend([Key::Tab, Key::Space]);
    keys.extend(chars("uu"));
    feed(&mut c, &keys);
    assert_eq!(c.preedit(), "уу");
    // the suffix decides, not the typed part: баг is masculine, багтэй is not
    let mut c = composer();
    let mut keys = chars("bag");
    keys.extend([Key::Tab, Key::Space]);
    keys.extend(chars("uu"));
    feed(&mut c, &keys);
    assert_eq!(c.preedit(), "үү");
}

#[test]
fn set_options_clears_the_ghost_when_completion_goes_off() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    assert!(c.completion().is_some());
    c.set_options(with_list(Options::classic()));
    assert_eq!(c.completion(), None);
    assert!(!c.wants_key(Key::Tab));
    let mut c = composer();
    type_str(&mut c, "bayarl");
    c.set_options(Options::smart());
    assert_eq!(c.completion(), None);
    assert!(!c.wants_key(Key::Tab));
    // and back on: the next key brings it again
    c.set_options(with_list(Options::smart()));
    assert_eq!(c.completion(), Some("алаа"));
}

#[test]
fn commit_cancel_and_reset_clear_the_ghost() {
    for how in 0..3 {
        let mut c = composer();
        type_str(&mut c, "bayarl");
        assert!(c.completion().is_some());
        match how {
            0 => {
                c.commit();
            }
            1 => c.cancel(),
            _ => c.reset(),
        }
        assert_eq!(c.completion(), None, "case {how}");
        assert!(!c.wants_key(Key::Tab));
    }
}

#[test]
fn a_reopened_word_has_a_ghost_again() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    c.process_key(Key::Space);
    c.process_key(Key::Backspace);
    assert_eq!(c.preedit(), "баярл");
    assert_eq!(c.completion(), Some("алаа"));
}

#[test]
fn a_modifier_keeps_the_ghost() {
    let mut c = composer();
    type_str(&mut c, "bayarl");
    c.process_key(Key::Modifier);
    assert_eq!(c.completion(), Some("алаа"));
    assert!(c.wants_key(Key::Tab));
}

#[test]
fn no_ghost_when_the_accepted_word_would_not_fit() {
    let long = |n: usize| {
        let opts = Options {
            words: Some(Arc::new(
                WordList::parse(&format!("{}\t5\n", "а".repeat(n))).unwrap(),
            )),
            ..Options::smart()
        };
        let mut c = Composer::with_options(opts);
        type_str(&mut c, "aaa");
        assert_eq!(c.preedit(), "ааа");
        c.completion().map(|g| g.chars().count())
    };
    assert_eq!(long(50), Some(47));
    assert_eq!(long(51), None);
}

#[test]
fn config_completion_parses_without_warning_and_a_duplicate_is_an_error() {
    let p = Config::parse("buuz_config = 1\n[options]\ncompletion = true\n").unwrap();
    assert!(p.warnings.is_empty());
    assert_eq!(p.config.completion(), Some(true));
    let e = Config::parse("buuz_config = 1\n[options]\ncompletion = true\ncompletion = false\n")
        .unwrap_err();
    assert!(matches!(
        e,
        buuz_core::ConfigError::DuplicateKey { line: 4, .. }
    ));
}
