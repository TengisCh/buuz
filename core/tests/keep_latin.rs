// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. Cases come from issue #10: Esc shows the raw Latin, and
// Backspace right after a word and a space reopens the word.

use buuz_core::{Composer, Key, Options};

fn smart() -> Composer {
    Composer::with_options(Options::smart())
}

fn type_str(c: &mut Composer, s: &str) {
    for ch in s.chars() {
        let key = if ch == ' ' { Key::Space } else { Key::Char(ch) };
        c.process_key(key);
    }
}

#[test]
fn keep_latin_is_on_in_smart_mode_and_off_in_classic() {
    assert!(Options::smart().keep_latin);
    assert!(Options::default().keep_latin);
    assert!(!Options::classic().keep_latin);
}

#[test]
fn escape_shows_the_raw_latin_and_space_commits_it() {
    let mut c = smart();
    type_str(&mut c, "ok");
    assert_eq!(c.preedit(), "ок");
    assert!(c.wants_key(Key::Escape));
    let r = c.process_key(Key::Escape);
    assert!(r.consumed);
    assert_eq!(r.commit, None);
    assert_eq!(c.preedit(), "ok");
    assert_eq!(c.cursor(), 2);
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("ok"));
    assert!(!r.consumed);
}

#[test]
fn escape_again_gives_the_cyrillic_back() {
    let mut c = smart();
    type_str(&mut c, "ok");
    c.process_key(Key::Escape);
    c.process_key(Key::Escape);
    assert_eq!(c.preedit(), "ок");
    assert_eq!(c.process_key(Key::Space).commit.as_deref(), Some("ок"));
}

#[test]
fn latin_keeps_the_case_as_typed() {
    let mut c = smart();
    type_str(&mut c, "GitHub");
    c.process_key(Key::Escape);
    assert_eq!(c.preedit(), "GitHub");
}

#[test]
fn typing_on_keeps_the_latin_view() {
    let mut c = smart();
    type_str(&mut c, "zo");
    c.process_key(Key::Escape);
    type_str(&mut c, "om");
    assert_eq!(c.preedit(), "zoom");
    c.process_key(Key::Backspace);
    assert_eq!(c.preedit(), "zoo");
    assert_eq!(c.cursor(), 3);
}

#[test]
fn a_latin_word_never_expands() {
    let mut c = smart();
    type_str(&mut c, "bn");
    c.process_key(Key::Escape);
    assert_eq!(c.process_key(Key::Space).commit.as_deref(), Some("bn"));
    let mut c = smart();
    type_str(&mut c, "bn");
    c.process_key(Key::Escape);
    assert_eq!(c.process_key(Key::Other).commit.as_deref(), Some("bn"));
}

#[test]
fn the_next_word_is_cyrillic_again() {
    let mut c = smart();
    type_str(&mut c, "ok");
    c.process_key(Key::Escape);
    type_str(&mut c, " sain");
    assert_eq!(c.preedit(), "сайн");
}

#[test]
fn escape_with_nothing_composed_passes_through() {
    let mut c = smart();
    assert!(!c.wants_key(Key::Escape));
    let r = c.process_key(Key::Escape);
    assert!(!r.consumed);
    assert_eq!(r.commit, None);
}

#[test]
fn escape_after_moving_the_caret_does_nothing() {
    let mut c = smart();
    type_str(&mut c, "sain");
    c.process_key(Key::Left);
    let r = c.process_key(Key::Escape);
    assert!(r.consumed);
    assert_eq!(c.preedit(), "сайн");
    assert_eq!(c.cursor(), 3);
}

#[test]
fn backspace_after_a_word_and_space_reopens_it() {
    let mut c = smart();
    type_str(&mut c, "sain");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("сайн"));
    assert!(c.wants_key(Key::Backspace));
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(r.delete_before, 5);
    assert_eq!(c.preedit(), "сайн");
    assert_eq!(c.raw(), "sain");
    assert_eq!(c.cursor(), 4);
    // The reopened word can be fixed with Latin keys and committed again.
    c.process_key(Key::Escape);
    assert_eq!(c.process_key(Key::Space).commit.as_deref(), Some("sain"));
}

#[test]
fn a_latin_word_reopens_in_latin() {
    let mut c = smart();
    type_str(&mut c, "ok");
    c.process_key(Key::Escape);
    c.process_key(Key::Space);
    let r = c.process_key(Key::Backspace);
    assert_eq!(r.delete_before, 3);
    assert_eq!(c.preedit(), "ok");
    c.process_key(Key::Escape);
    assert_eq!(c.preedit(), "ок");
}

#[test]
fn reopening_restores_the_particle_context() {
    let mut c = smart();
    type_str(&mut c, "irsen ");
    type_str(&mut c, "uu ");
    c.process_key(Key::Backspace);
    assert_eq!(c.preedit(), "үү");
}

#[test]
fn backspace_reopens_only_once() {
    let mut c = smart();
    type_str(&mut c, "ok ");
    c.process_key(Key::Backspace);
    c.process_key(Key::Backspace);
    c.process_key(Key::Backspace);
    assert!(!c.is_composing());
    // The composition was emptied; the next Backspace is the application's.
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(r.delete_before, 0);
}

#[test]
fn no_reopen_after_punctuation_a_letter_or_commit() {
    let mut c = smart();
    type_str(&mut c, "ok");
    c.process_key(Key::Other);
    assert!(!c.wants_key(Key::Backspace));

    let mut c = smart();
    type_str(&mut c, "ok a");
    c.process_key(Key::Backspace);
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);

    let mut c = smart();
    type_str(&mut c, "ok ");
    c.commit();
    assert!(!c.wants_key(Key::Backspace));

    let mut c = smart();
    type_str(&mut c, "ok ");
    c.clear_context();
    assert!(!c.wants_key(Key::Backspace));
}

#[test]
fn shift_alone_keeps_the_reopen() {
    let mut c = smart();
    type_str(&mut c, "ok ");
    c.process_key(Key::Modifier);
    assert_eq!(c.process_key(Key::Backspace).delete_before, 3);
}

#[test]
fn abbreviations_still_undo_to_the_raw_keys() {
    let mut c = smart();
    type_str(&mut c, "bn ");
    let r = c.process_key(Key::Backspace);
    assert_eq!(r.delete_before, 6);
    assert_eq!(c.preedit(), "бн");
}

#[test]
fn classic_mode_is_unchanged() {
    let mut c = Composer::with_options(Options::classic());
    type_str(&mut c, "ok");
    assert!(!c.wants_key(Key::Escape));
    let r = c.process_key(Key::Escape);
    assert!(!r.consumed);
    assert_eq!(r.commit.as_deref(), Some("ок"));

    type_str(&mut c, "sain");
    c.process_key(Key::Space);
    assert!(!c.wants_key(Key::Backspace));
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(r.delete_before, 0);
}

#[test]
fn keep_latin_off_in_smart_mode() {
    let opts = Options {
        keep_latin: false,
        ..Options::smart()
    };
    let mut c = Composer::with_options(opts);
    type_str(&mut c, "ok");
    assert!(!c.process_key(Key::Escape).consumed);
    type_str(&mut c, "sain ");
    assert!(!c.process_key(Key::Backspace).consumed);
    // Abbreviation undo is part of abbreviations, not of keep_latin.
    type_str(&mut c, "bn ");
    assert_eq!(c.process_key(Key::Backspace).delete_before, 6);
}

#[test]
fn reopening_restores_the_gender_the_word_was_typed_with() {
    let mut c = smart();
    type_str(&mut c, "irsen uu");
    c.process_key(Key::Escape);
    c.process_key(Key::Space);
    c.process_key(Key::Backspace);
    c.process_key(Key::Escape);
    assert_eq!(c.preedit(), "үү");
}

#[test]
fn reset_and_cancel_drop_the_reopen() {
    let mut c = smart();
    type_str(&mut c, "ok ");
    c.reset();
    assert!(!c.wants_key(Key::Backspace));
    let mut c = smart();
    type_str(&mut c, "ok ");
    c.cancel();
    assert!(!c.wants_key(Key::Backspace));
}

#[test]
fn enter_after_a_word_drops_the_reopen() {
    let mut c = smart();
    type_str(&mut c, "ok");
    c.process_key(Key::Other);
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(r.delete_before, 0);
}

#[test]
fn delete_in_the_latin_view_stays_latin() {
    let mut c = smart();
    type_str(&mut c, "ok");
    c.process_key(Key::Escape);
    c.process_key(Key::Left);
    c.process_key(Key::Delete);
    assert_eq!(c.preedit(), "o");
}

#[test]
fn a_reopened_abbreviation_can_be_shown_in_latin() {
    let mut c = smart();
    type_str(&mut c, "bn ");
    c.process_key(Key::Backspace);
    c.process_key(Key::Escape);
    assert_eq!(c.process_key(Key::Space).commit.as_deref(), Some("bn"));
}

#[test]
fn escape_with_only_a_reopen_armed_is_not_consumed() {
    let mut c = smart();
    type_str(&mut c, "ok ");
    let r = c.process_key(Key::Escape);
    assert!(!r.consumed);
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(r.delete_before, 0);
}

#[test]
fn a_second_space_drops_the_reopen() {
    let mut c = smart();
    type_str(&mut c, "ok  ");
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(r.delete_before, 0);
}

#[test]
fn a_latin_word_gives_no_particle_evidence() {
    let mut c = smart();
    type_str(&mut c, "irsen ok");
    c.process_key(Key::Escape);
    type_str(&mut c, " uu");
    assert_eq!(c.preedit(), "уу");
}
