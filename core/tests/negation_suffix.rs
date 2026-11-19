// SPDX-License-Identifier: Apache-2.0

//! Issue #87: the negation suffix -гүй is always гүй, also after a back
//! vowel stem (ажлахгүй). Smart mode with `smart_vowels` only.

use std::sync::Arc;

use buuz_core::{convert_with, Composer, Key, KeyMap, Options, WordList};

const REAL: &str = include_str!("../../data/wordlist/mn.txt");

fn on(input: &str) -> String {
    convert_with(input, &Options::smart())
}

fn off(input: &str) -> String {
    let o = Options {
        smart_vowels: false,
        ..Options::smart()
    };
    convert_with(input, &o)
}

fn classic(input: &str) -> String {
    convert_with(input, &Options::classic())
}

fn listed(input: &str) -> String {
    let o = Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        ..Options::smart()
    };
    convert_with(input, &o)
}

#[test]
fn masculine_stems_take_gwi() {
    for (k, v) in [
        ("ajlahgui", "ажлахгүй"),
        ("baihgui", "байхгүй"),
        ("yavahgui", "явахгүй"),
        ("avahgui", "авахгүй"),
        ("ajilgui", "ажилгүй"),
        ("bolohgui", "болохгүй"),
        ("chadaagui", "чадаагүй"),
        ("argagui", "аргагүй"),
        ("baidaggui", "байдаггүй"),
        ("ayuulgui", "аюулгүй"),
        ("durgui", "дургүй"),
        ("ajlahgui ", "ажлахгүй "),
    ] {
        assert_eq!(on(k), v, "{k}");
    }
}

#[test]
fn case_is_kept() {
    assert_eq!(on("Ajlahgui"), "Ажлахгүй");
    assert_eq!(on("AJLAHGUI"), "АЖЛАХГҮЙ");
    assert_eq!(on("AJLAHGUI "), "АЖЛАХГҮЙ ");
    assert_eq!(on("ajlahGUI"), "ажлахГҮЙ");
}

#[test]
fn front_suffix_after_it() {
    assert_eq!(on("ajlahguigeer"), "ажлахгүйгээр");
    assert_eq!(on("ajlahguin"), "ажлахгүйн");
    assert_eq!(on("ajilguichuud"), "ажилгүйчүүд");
}

#[test]
fn particle_ignores_the_forced_front_vowel() {
    assert_eq!(on("ajlahgui uu"), "ажлахгүй уу");
    assert_eq!(on("bhgui uu"), "байхгүй уу");
}

#[test]
fn neighbours_are_unchanged() {
    for (k, v) in [
        ("medehgui", "мэдэхгүй"),
        ("bolohgwi", "болохгүй"),
        ("ugwi", "угүй"),
        ("gui", "гуй"),
        ("guih", "гуйх"),
        ("guilga", "гуйлга"),
        ("gutal", "гутал"),
        ("dugui", "дугуй"),
        ("duguin", "дугуйн"),
        ("agui", "агуй"),
        ("buguivch", "бугуйвч"),
        ("shugui", "шугуй"),
        ("yalanguyaa", "ялангуяа"),
        ("ajlahgu\"i", "ажлахгуй"),
        ("har'cangui", "харьцангуй"),
    ] {
        assert_eq!(on(k), v, "{k}");
    }
}

#[test]
fn word_list_keeps_listed_gui_words() {
    for (k, v) in [
        ("dugui ", "дугуй "),
        ("har'cangui ", "харьцангуй "),
        ("ogtorgui ", "огторгуй "),
        ("daranguilagch ", "дарангуйлагч "),
        ("bolovsrongui ", "боловсронгуй "),
        ("huraangui ", "хураангуй "),
        ("hargui ", "харгуй "),
    ] {
        assert_eq!(listed(k), v, "{k}");
    }
}

#[test]
fn word_list_prefers_gwi() {
    for (k, v) in [
        ("uhaangui ", "ухаангүй "),
        ("baisangui ", "байсангүй "),
        ("durgui ", "дургүй "),
        ("duugui ", "дуугүй "),
        ("ajlahgui ", "ажлахгүй "),
        ("ajlahgui", "ажлахгүй"),
    ] {
        assert_eq!(listed(k), v, "{k}");
    }
}

#[test]
fn off_and_classic_are_unchanged() {
    assert_eq!(off("ajlahgui"), "ажлахгуй");
    assert_eq!(classic("ajlahgui"), "ажлахгуй");
    assert_eq!(classic("baihgui"), "байхгуй");
    assert_eq!(classic("ugui"), "угуй");
}

#[test]
fn preedit_changes_on_the_i_key() {
    let mut c = Composer::with_options(Options::smart());
    for ch in "ajlahgu".chars() {
        c.process_key(Key::Char(ch));
    }
    assert_eq!(c.preedit(), "ажлахгу");
    c.process_key(Key::Char('i'));
    assert_eq!(c.preedit(), "ажлахгүй");
}

fn keyed(list: &[(&str, &str)]) -> Options {
    let pairs: Vec<(String, String)> = list
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Options {
        keys: Some(Arc::new(KeyMap::from_pairs(&pairs).unwrap())),
        ..Options::smart()
    }
}

#[test]
fn custom_keys_follow_the_output() {
    // `j` gives г here: the rule follows what the key gives.
    let o = keyed(&[("j", "г")]);
    assert_eq!(convert_with("avahjui", &o), "авахгүй");
    assert_eq!(convert_with("jui", &o), "гуй");
}

#[test]
fn past_negative_san_son_takes_gwi_without_a_list() {
    assert_eq!(on("baisangui"), "байсангүй");
    assert_eq!(on("chadsangui"), "чадсангүй");
    assert_eq!(on("bolsongui"), "болсонгүй");
    assert_eq!(on("avsangui"), "авсангүй");
    assert_eq!(on("har'cangui"), "харьцангуй");
    assert_eq!(on("bolovsrongui"), "боловсронгуй");
    assert_eq!(on("huraangui"), "хураангуй");
}

#[test]
fn unlisted_words_use_the_no_list_rules() {
    assert_eq!(listed("sulangui "), "сулангуй ");
    assert_eq!(listed("nalangui "), "налангуй ");
    assert_eq!(listed("ajlahgui"), "ажлахгүй");
    assert_eq!(listed("uhaangui "), "ухаангүй ");
    assert_eq!(listed("durgui "), "дургүй ");
    assert_eq!(listed("baisangui"), "байсангүй");
}
