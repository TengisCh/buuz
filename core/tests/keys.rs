// SPDX-License-Identifier: Apache-2.0

//! Custom key mappings (issue #3): `KeyMap`, `Options::keys` and the
//! `[keys]` section of a config.

use std::sync::Arc;

use buuz_core::{convert_with, Config, ConfigError, KeyError, KeyMap, Options};

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn with_keys(base: Options, list: &[(&str, &str)]) -> Options {
    Options {
        keys: Some(Arc::new(KeyMap::from_pairs(&pairs(list)).unwrap())),
        ..base
    }
}

fn smart(list: &[(&str, &str)]) -> Options {
    with_keys(Options::smart(), list)
}

#[test]
fn no_keys_by_default() {
    assert!(Options::default().keys.is_none());
    assert!(Options::smart().keys.is_none());
    assert!(Options::classic().keys.is_none());
}

#[test]
fn v_as_u_front_copies_the_harmony_evidence() {
    let o = smart(&[("v", "ү")]);
    assert_eq!(convert_with("vnee", &o), "үнээ");
    assert_eq!(convert_with("vnee", &o), convert_with("wnee", &o));
}

#[test]
fn empty_value_removes_a_key() {
    let o = smart(&[("q", "")]);
    assert_eq!(convert_with("q", &o), "q");
    assert_eq!(convert_with("Q", &o), "Q");
    // Other keys are untouched.
    assert_eq!(convert_with("w", &o), "ү");
}

#[test]
fn x_as_h_keeps_h() {
    let o = smart(&[("x", "х")]);
    assert_eq!(convert_with("xaan", &o), convert_with("haan", &o));
    assert_eq!(convert_with("xaan", &o), "хаан");
    assert_eq!(convert_with("haan", &o), "хаан");
}

#[test]
fn kh_longest_match_against_k_and_h() {
    let o = smart(&[("kh", "х")]);
    assert_eq!(convert_with("khaan", &o), "хаан");
    assert_eq!(convert_with("kaan", &o), "каан");
    assert_eq!(convert_with("haan", &o), "хаан");
}

#[test]
fn replaced_key_loses_its_old_rules() {
    // x was х; now x gives ш.
    let o = smart(&[("x", "ш")]);
    assert_eq!(convert_with("xaan", &o), "шаан");
}

#[test]
fn upper_case_follows_the_copied_flags() {
    let o = smart(&[("v", "ү")]);
    assert_eq!(convert_with("V", &o), "Ү");
    assert_eq!(convert_with("v", &o), "ү");
    // Hard and soft signs have no case conversion.
    let s = smart(&[("j", "ь")]);
    assert_eq!(convert_with("J", &s), "ь");
}

#[test]
fn gender_pair_copies_the_rules_of_u() {
    let o = smart(&[("v", "у/ү")]);
    assert_eq!(convert_with("vnee", &o), "үнээ");
    assert_eq!(convert_with("vna", &o), convert_with("una", &o));
}

#[test]
fn a_single_vowel_names_one_letter() {
    // у: the key that gives only у (`u"`), so the word is made masculine.
    let o = smart(&[("v", "у")]);
    assert_eq!(convert_with("v", &o), "у");
}

#[test]
fn keys_work_in_classic_mode() {
    let o = with_keys(Options::classic(), &[("v", "ү")]);
    assert_eq!(
        convert_with("vnee", &o),
        convert_with("wnee", &Options::classic())
    );
}

#[test]
fn no_keys_leaves_output_alone() {
    let a = Options::classic();
    let b = with_keys(Options::classic(), &[]);
    for s in ["geree", "unee", "Sain baina uu", "q w x v", "ii'"] {
        assert_eq!(convert_with(s, &a), convert_with(s, &b));
    }
}

#[test]
fn cyrillic_reseed_rules_survive() {
    let o = smart(&[("v", "ү")]);
    let mut c = buuz_core::Composer::with_options(o);
    for ch in "vnee".chars() {
        c.process_key(buuz_core::Key::Char(ch));
    }
    // The caret moves: the buffer is re-seeded from Cyrillic and typing goes on.
    c.process_key(buuz_core::Key::Left);
    c.process_key(buuz_core::Key::Char('v'));
    assert_eq!(c.preedit(), "үнэүэ");
}

#[test]
fn unknown_target_is_a_key_error() {
    assert!(matches!(
        KeyMap::from_pairs(&pairs(&[("v", "жжж")])),
        Err(KeyError::UnknownTarget { .. })
    ));
}

#[test]
fn bad_key_is_a_key_error() {
    assert!(matches!(
        KeyMap::from_pairs(&pairs(&[("abcd", "х")])),
        Err(KeyError::BadKey { .. })
    ));
    assert!(matches!(
        KeyMap::from_pairs(&pairs(&[("1", "х")])),
        Err(KeyError::BadKey { .. })
    ));
}

#[test]
fn config_errors_have_a_line() {
    let e = Config::parse("buuz_config = 1\n[keys]\nv = \"жжж\"\n").unwrap_err();
    assert!(
        matches!(e, ConfigError::UnknownTarget { line: 3, .. }),
        "{e:?}"
    );
    let e = Config::parse("buuz_config = 1\n[keys]\n\nabcd = \"х\"\n").unwrap_err();
    assert!(matches!(e, ConfigError::BadKeys { line: 4, .. }), "{e:?}");
    assert!(e.to_string().contains("line 4"));
}

#[test]
fn config_end_to_end() {
    let p = Config::parse("buuz_config = 1\n[keys]\nv = \"ү\"\nq = \"\"\nkh = \"х\"\n").unwrap();
    let o = p.config.options();
    assert!(o.keys.is_some());
    assert_eq!(convert_with("vnee", &o), "үнээ");
    assert_eq!(convert_with("q", &o), "q");
    assert_eq!(convert_with("khaan", &o), "хаан");
}

#[test]
fn config_without_keys_has_no_key_map() {
    let p = Config::parse("buuz_config = 1\n").unwrap();
    assert!(p.config.options().keys.is_none());
}

#[test]
fn config_keys_round_trip() {
    let p = Config::parse("buuz_config = 1\n[keys]\nq = \"\"\nv = \"у/ү\"\n").unwrap();
    let again = Config::parse(&p.config.to_text()).unwrap();
    assert_eq!(again.config, p.config);
}

// ---- review fixes ----

fn classic(list: &[(&str, &str)]) -> Options {
    with_keys(Options::classic(), list)
}

#[test]
fn target_without_case_conversion_keeps_the_lower_case_key() {
    let o = smart(&[("j", "ь")]);
    assert_eq!(convert_with("j", &o), "ь");
    assert_eq!(convert_with("J", &o), "ь");
    assert_eq!(convert_with("bajn", &o), convert_with("ba'n", &o));
    let o = smart(&[("x", "ъ")]);
    assert_eq!(convert_with("x", &o), "ъ");
}

#[test]
fn vowel_keys_get_the_i_companion() {
    let o = smart(&[("v", "ү")]);
    assert_eq!(convert_with("vi", &o), convert_with("wi", &o));
    assert_eq!(convert_with("vi", &o), "үй");
    let o = smart(&[("v", "у/ү")]);
    assert_eq!(
        convert_with("vilchilgee", &o),
        convert_with("uilchilgee", &o)
    );
    assert_eq!(convert_with("vi", &o), convert_with("ui", &o));
}

#[test]
fn replacing_a_key_drops_its_old_i_companion() {
    // w had wi = үй; now w is в and wi is в + и.
    let o = smart(&[("w", "в")]);
    assert_eq!(convert_with("wi", &o), "ви");
    // Removing q removes qi too.
    let o = smart(&[("q", "")]);
    assert_eq!(convert_with("qi", &o), "qи");
}

#[test]
fn an_explicit_entry_beats_an_implied_companion() {
    let o = smart(&[("qi", "х"), ("q", "")]);
    assert_eq!(convert_with("qi", &o), "х");
}

#[test]
fn longer_target_follows_the_case_of_the_key() {
    let o = smart(&[("y", "ий")]);
    assert_eq!(convert_with("y", &o), "ий");
    assert_eq!(convert_with("Y", &o), "ИЙ");
    let c = classic(&[("y", "ий")]);
    assert_eq!(
        convert_with("salhy", &c),
        convert_with("salhii", &Options::classic())
    );
}

#[test]
fn three_character_key_with_mixed_case() {
    let o = smart(&[("sxk", "щ")]);
    assert_eq!(convert_with("sxk", &o), "щ");
    assert_eq!(convert_with("Sxk", &o), "Щ");
    assert_eq!(convert_with("SXK", &o), "Щ");
    assert_eq!(convert_with("sXk", &o), "щ");
}

#[test]
fn kh_plus_k_removed() {
    let o = smart(&[("kh", "х"), ("k", "")]);
    assert_eq!(convert_with("khaan", &o), "хаан");
    assert_eq!(convert_with("kaan", &o), "kаан");
}

#[test]
fn replace_then_remove_the_same_key() {
    let o = smart(&[("x", "ш"), ("x", "")]);
    assert_eq!(convert_with("x", &o), "x");
}

#[test]
fn ii_swap_follows_the_rule_output() {
    // ii now gives х: no swap, no ий.
    let o = smart(&[("ii", "х")]);
    assert_eq!(convert_with("sainii", &o), "сайнх");
    // y gives ий and gets the smart ы where ii would.
    let o = smart(&[("y", "ий")]);
    assert_eq!(
        convert_with("aavyn", &o),
        convert_with("aaviin", &Options::smart())
    );
    assert_eq!(
        convert_with("saly", &o),
        convert_with("salii", &Options::smart())
    );
    // ii' keeps ий.
    assert_eq!(convert_with("salii'", &Options::smart()), "салий");
}

#[test]
fn ye_and_g_follow_the_rule_output() {
    // A key that gives е says nothing about the word, like ye.
    let o = smart(&[("v", "е")]);
    assert_eq!(
        convert_with("vnuu", &o),
        convert_with("yenuu", &Options::smart())
    );
    // A key that gives г: an explicit ү after it is the suffix -гүй.
    let o = smart(&[("k", "г")]);
    assert_eq!(
        convert_with("bolohkw", &o),
        convert_with("bolohgw", &Options::smart())
    );
}

#[test]
fn particle_rule_still_looks_at_raw_u_keys() {
    // Documented limit (typing-rules section 10): the particle rule reads
    // the typed keys, so a key remapped to у/ү is not a particle.
    let o = smart(&[("v", "у/ү")]);
    assert_eq!(convert_with("unee uu", &o), "үнээ үү");
    assert_eq!(convert_with("unee v", &o), "үнээ у");
}

#[test]
fn upper_case_target_is_refused() {
    assert!(matches!(
        KeyMap::from_pairs(&pairs(&[("j", "Ь")])),
        Err(KeyError::UpperCaseTarget { .. })
    ));
    let e = Config::parse("buuz_config = 1\n[keys]\nj = \"Ъ\"\n").unwrap_err();
    assert!(
        matches!(e, ConfigError::UpperCaseTarget { line: 3, .. }),
        "{e:?}"
    );
    assert!(e.to_string().contains("lower case"));
}

#[test]
fn setters_check_keys() {
    let mut c = Config::default();
    c.set_key("q", "").unwrap();
    let e = c.set_key("v", "жжж").unwrap_err();
    assert!(
        matches!(e, ConfigError::UnknownTarget { line: 0, .. }),
        "{e:?}"
    );
}

#[test]
fn error_texts() {
    let e = Config::parse("buuz_config = 1\n[keys]\nv = \"жжж\"\n").unwrap_err();
    let t = e.to_string();
    assert!(t.starts_with("line 3: "), "{t}");
    assert!(t.contains("\"v\"") && t.contains("\"жжж\""), "{t}");
    let e = KeyMap::from_pairs(&pairs(&[("v", "жжж")])).unwrap_err();
    assert!(e.to_string().contains("no built-in key gives"));
    let e = Config::parse("buuz_config = 1\n[keys]\nabcd = \"х\"\n").unwrap_err();
    let t = e.to_string();
    assert!(t.contains("empty") && t.contains('/'), "{t}");
}

#[test]
fn doc_example_kh_as_u_front() {
    let o = smart(&[("kh", "ү")]);
    assert_eq!(convert_with("khaan", &o), "үаан");
}
