// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. Cases come from issue #5 (abbreviation expansion).

use buuz_core::{convert_with, Abbreviations, Composer, Key, Options};

fn opts() -> Options {
    Options {
        abbreviations: Some(Abbreviations::builtin()),
        ..Default::default()
    }
}

fn eq(input: &str, want: &str) {
    assert_eq!(convert_with(input, &opts()), want, "input {input:?}");
}

fn composer() -> Composer {
    Composer::with_options(opts())
}

fn chars(s: &str) -> Vec<Key> {
    s.chars().map(Key::Char).collect()
}

fn type_keys(c: &mut Composer, keys: &[Key]) {
    for &k in keys {
        c.process_key(k);
    }
}

#[test]
fn expands_at_space_or_punctuation() {
    eq("bn ", "байна ");
    eq("bn.", "байна.");
    eq("bnu?", "байна уу?");
    eq("sn bnu ", "сайн байна уу ");
    eq("bga bhgui ", "байгаа байхгүй ");
    // The end of the text is not a typed Space or punctuation.
    eq("bn", "бн");
    eq("sn bnu", "сайн бну");
}

#[test]
fn case_follows_the_keys() {
    eq("Bn ", "Байна ");
    eq("BN ", "БАЙНА ");
    eq("B ", "Б ");
    eq("Sbnu ", "Сайн байна уу ");
}

#[test]
fn particle_follows_the_expansion() {
    eq("bn uu", "байна уу");
    eq("mdku uu", "мэдэхгүй үү");
    eq("zgr uu", "зүгээр үү");
    // -гүй is not evidence: байхгүй is masculine, мэдэхгүй stays feminine.
    eq("bhgwi uu ", "байхгүй уу ");
    eq("bhgui uu ", "байхгүй уу ");
}

#[test]
fn non_abbreviations_are_untouched() {
    eq("bna", "бна");
    eq("bn'", "бнь");
    assert_eq!(convert_with("bn ", &Options::default()), "бн ");
    assert_eq!(convert_with("bn ", &Options::classic()), "бн ");
}

#[test]
fn space_expansion_can_be_undone() {
    let mut c = composer();
    type_keys(&mut c, &[Key::Char('b'), Key::Char('n')]);
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("байна"));
    assert!(!r.consumed);
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(r.delete_before, 6);
    assert_eq!(c.preedit(), "бн");
    assert_eq!(c.cursor(), 2);
    assert!(c.is_composing());
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("бн"));
    // The undone word does not expand again, and the next one does.
    type_keys(&mut c, &[Key::Char('b'), Key::Char('n')]);
    assert_eq!(c.process_key(Key::Space).commit.as_deref(), Some("байна"));
}

#[test]
fn no_undo_after_punctuation() {
    let mut c = composer();
    type_keys(&mut c, &[Key::Char('b'), Key::Char('n')]);
    let r = c.process_key(Key::Other);
    assert_eq!(r.commit.as_deref(), Some("байна"));
    assert!(!c.wants_key(Key::Backspace));
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(r.delete_before, 0);
}

#[test]
fn a_letter_clears_the_undo() {
    let mut c = composer();
    type_keys(
        &mut c,
        &[Key::Char('b'), Key::Char('n'), Key::Space, Key::Char('a')],
    );
    let r = c.process_key(Key::Backspace);
    assert_eq!(r.delete_before, 0);
    assert!(!c.is_composing());
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
}

#[test]
fn undo_restores_the_particle_context() {
    let mut c = composer();
    // irsen is feminine; бн is masculine, so the particle after the undone
    // and re-committed бн is у, not үү.
    type_keys(&mut c, &chars("irsen"));
    c.process_key(Key::Space);
    type_keys(&mut c, &chars("bn"));
    c.process_key(Key::Space);
    c.process_key(Key::Backspace);
    assert_eq!(c.preedit(), "бн");
    type_keys(&mut c, &[Key::Space, Key::Char('u')]);
    assert_eq!(c.preedit(), "у");
}

#[test]
fn undo_puts_back_the_previous_word_gender() {
    let mut c = composer();
    type_keys(&mut c, &chars("irsen"));
    c.process_key(Key::Space);
    type_keys(&mut c, &chars("bn"));
    c.process_key(Key::Space);
    // The expansion made the context masculine; the undo restores irsen's.
    c.process_key(Key::Backspace);
    c.process_key(Key::Backspace);
    c.process_key(Key::Backspace);
    assert!(!c.is_composing());
    type_keys(&mut c, &chars("uu"));
    assert_eq!(c.preedit(), "үү");
}

#[test]
fn reseeded_words_never_expand() {
    let mut c = composer();
    type_keys(
        &mut c,
        &[Key::Char('b'), Key::Char('n'), Key::Left, Key::Right],
    );
    assert_eq!(c.process_key(Key::Space).commit.as_deref(), Some("бн"));
}

#[test]
fn focus_loss_commit_never_expands() {
    let mut c = composer();
    type_keys(&mut c, &[Key::Char('b'), Key::Char('n')]);
    assert_eq!(c.commit().as_deref(), Some("бн"));
}

#[test]
fn commit_and_reset_clear_the_undo() {
    let mut c = composer();
    type_keys(&mut c, &chars("bn"));
    c.process_key(Key::Space);
    assert_eq!(c.commit(), None);
    assert!(!c.wants_key(Key::Backspace));
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(r.delete_before, 0);

    type_keys(&mut c, &chars("bn"));
    c.process_key(Key::Space);
    c.reset();
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(r.delete_before, 0);
}

#[test]
fn modifier_keeps_the_undo() {
    let mut c = composer();
    type_keys(&mut c, &chars("bn"));
    c.process_key(Key::Space);
    let r = c.process_key(Key::Modifier);
    assert!(!r.consumed);
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(r.delete_before, 6);
    assert_eq!(c.preedit(), "бн");
}

#[test]
fn parse_and_to_text() {
    let a = Abbreviations::parse("# c\nbn = байна\n\nBGA=байгаа").unwrap();
    let o = Options {
        abbreviations: Some(std::sync::Arc::new(a.clone())),
        ..Default::default()
    };
    assert_eq!(convert_with("bga ", &o), "байгаа ");
    assert_eq!(convert_with("BGA ", &o), "БАЙГАА ");
    assert_eq!(convert_with("bn ", &o), "байна ");
    assert!(Abbreviations::parse("b n = x").is_err());
    assert!(Abbreviations::parse("bn =").is_err());
    assert!(Abbreviations::parse("bn").is_err());
    assert!(Abbreviations::parse("bn = a\nb1 = c").is_err());
    let text = a.to_text();
    assert_eq!(text, "bga = байгаа\nbn = байна\n");
    assert_eq!(Abbreviations::parse(&text).unwrap().to_text(), text);
}

#[test]
fn last_duplicate_wins() {
    let a = Abbreviations::parse("x = а\nX = б").unwrap();
    assert_eq!(a.to_text(), "x = б\n");
    let b = Abbreviations::from_pairs([("x", "а"), ("x", "в")]).unwrap();
    assert_eq!(b.to_text(), "x = в\n");
}

#[test]
fn builtin_round_trips() {
    let b = Abbreviations::builtin();
    let again = Abbreviations::parse(&b.to_text()).unwrap();
    assert_eq!(again.to_text(), b.to_text());
}
