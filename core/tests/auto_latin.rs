// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. Cases come from issue #10, part 3: URLs and emails stay Latin.

use buuz_core::{convert_with, Composer, Key, Options};

fn smart() -> Options {
    Options::smart()
}

fn no_auto() -> Options {
    Options {
        auto_latin: false,
        ..Options::smart()
    }
}

/// What the converter gives when the word is typed as plain Cyrillic.
fn cyr(word: &str) -> String {
    convert_with(word, &no_auto())
}

fn key_for(ch: char) -> Key {
    if buuz_core::is_input_char(ch) {
        Key::Char(ch)
    } else if ch == ' ' {
        Key::Space
    } else {
        Key::Symbol(ch)
    }
}

fn type_str(c: &mut Composer, s: &str) {
    for ch in s.chars() {
        c.process_key(key_for(ch));
    }
}

#[test]
fn auto_latin_is_on_in_smart_mode_and_off_in_classic() {
    assert!(Options::smart().auto_latin);
    assert!(Options::default().auto_latin);
    assert!(!Options::classic().auto_latin);
}

#[test]
fn a_domain_stays_latin() {
    assert_eq!(convert_with("github.com", &smart()), "github.com");
    assert_ne!(cyr("github.com"), "github.com");
}

#[test]
fn a_domain_inside_a_sentence_stays_latin() {
    let want = format!("{} github.com {}", cyr("see"), cyr("now"));
    assert_eq!(convert_with("see github.com now", &smart()), want);
}

#[test]
fn an_email_stays_latin() {
    assert_eq!(convert_with("bat@gmail.com", &smart()), "bat@gmail.com");
    let want = format!("{} bat@gmail.com {}", cyr("mail"), cyr("me"));
    assert_eq!(convert_with("mail bat@gmail.com me", &smart()), want);
}

#[test]
fn urls_stay_latin() {
    let url = "https://buuz.thetengis.com/x";
    assert_eq!(convert_with(url, &smart()), url);
    assert_eq!(convert_with("www.google.com", &smart()), "www.google.com");
    let url = "https://buuz.thetengis.com/x and";
    let want = format!("https://buuz.thetengis.com/x {}", cyr("and"));
    assert_eq!(convert_with(url, &smart()), want);
}

#[test]
fn a_label_with_digits_and_hyphens_is_a_domain() {
    assert_eq!(convert_with("my-site2.dev ", &smart()), "my-site2.dev ");
}

#[test]
fn ordinary_dots_stay_cyrillic() {
    for text in ["bn.ok", "sain.baina", "bn. sain", "sain baina.", "ok."] {
        assert_eq!(convert_with(text, &smart()), cyr(text), "{text}");
    }
}

#[test]
fn a_domain_ends_at_enter_and_tab_too() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "github.com");
    let r = c.process_key(Key::Other);
    assert!(!r.consumed);
    assert_eq!(r.commit.as_deref(), Some("github.com"));
    assert_eq!(r.delete_before, cyr("github.").chars().count());
}

#[test]
fn classic_and_auto_latin_off_keep_the_original_output() {
    for text in [
        "github.com",
        "see github.com now",
        "bat@gmail.com",
        "https://buuz.thetengis.com/x",
        "www.google.com",
    ] {
        let classic = convert_with(text, &Options::classic());
        let off = convert_with(text, &no_auto());
        assert_ne!(classic, text);
        assert_ne!(off, text);
        let mut c = Composer::with_options(Options::classic());
        let mut out = String::new();
        for ch in text.chars() {
            let r = c.process_key(key_for(ch));
            assert_eq!(r.delete_before, 0);
            out.extend(r.commit);
            if !r.consumed {
                out.push(ch);
            }
        }
        out.extend(c.commit());
        assert_eq!(out, classic);
    }
}

#[test]
fn symbol_acts_like_other_without_auto_latin() {
    for opts in [Options::classic(), no_auto()] {
        let mut a = Composer::with_options(opts.clone());
        let mut b = Composer::with_options(opts);
        type_str(&mut a, "sain");
        type_str(&mut b, "sain");
        assert_eq!(a.process_key(Key::Symbol('@')), b.process_key(Key::Other));
    }
}

#[test]
fn at_sign_turns_the_word_latin() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "bat");
    let r = c.process_key(Key::Symbol('@'));
    assert!(r.consumed);
    // The word is still in the preedit, so nothing is committed yet.
    assert_eq!(r.delete_before, 0);
    assert_eq!(r.commit.as_deref(), Some("bat@"));
    assert!(!c.is_composing());
    type_str(&mut c, "gmail");
    assert_eq!(c.preedit(), "gmail");
}

#[test]
fn the_trigger_deletes_what_the_run_already_committed() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "a.bat");
    let r = c.process_key(Key::Symbol('@'));
    assert!(r.consumed);
    assert_eq!(r.delete_before, cyr("a.").chars().count());
    assert_eq!(r.commit.as_deref(), Some("a.bat@"));
}

#[test]
fn the_url_trigger_deletes_the_scheme_already_shown() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "https:/");
    let r = c.process_key(Key::Symbol('/'));
    assert!(r.consumed);
    assert_eq!(r.delete_before, cyr("https:/").chars().count());
    assert_eq!(r.commit.as_deref(), Some("https://"));
    type_str(&mut c, "abc");
    assert_eq!(c.preedit(), "abc");
}

#[test]
fn a_lone_slash_pair_is_not_a_trigger() {
    let r = Composer::with_options(smart()).process_key(Key::Symbol('/'));
    assert!(!r.consumed);
    assert_eq!(convert_with("a//b", &smart()), cyr("a//b"));
}

#[test]
fn www_is_not_a_live_trigger_but_its_domain_is_latin() {
    assert_eq!(convert_with("www. sain", &smart()), cyr("www. sain"));
    assert_eq!(convert_with("www.google.com ", &smart()), "www.google.com ");
}

#[test]
fn escape_shows_the_cyrillic_of_the_current_word_after_a_trigger() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "bat@gmail");
    assert_eq!(c.preedit(), "gmail");
    c.process_key(Key::Escape);
    assert_eq!(c.preedit(), cyr("gmail"));
}

#[test]
fn the_next_run_is_cyrillic_again() {
    assert_eq!(
        convert_with("bat@gmail.com sain", &smart()),
        format!("bat@gmail.com {}", cyr("sain"))
    );
}

#[test]
fn backspace_after_an_automatic_domain_restores_the_cyrillic() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "github.com");
    let r = c.process_key(Key::Space);
    assert!(!r.consumed);
    assert_eq!(r.commit.as_deref(), Some("github.com"));
    assert!(c.wants_key(Key::Backspace));
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(r.delete_before, "github.com ".chars().count());
    assert_eq!(r.commit, Some(cyr("github.com")));
    assert!(!c.is_composing());
    // Only that one Backspace is special.
    assert!(!c.process_key(Key::Backspace).consumed);
}

#[test]
fn any_other_key_drops_the_domain_undo() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "github.com a");
    c.process_key(Key::Backspace);
    c.process_key(Key::Backspace);
    let r = c.process_key(Key::Backspace);
    assert_eq!(r.commit, None);
}

#[test]
fn a_run_longer_than_100_characters_is_never_changed() {
    let long = "ab.".repeat(40); // 120 characters
    let text = format!("{long}github.com");
    assert_eq!(convert_with(&text, &smart()), cyr(&text));
    let text = format!("{long}bat@x");
    assert_eq!(convert_with(&text, &smart()), cyr(&text));
    // A shorter run is still changed.
    let short = "ab.".repeat(10);
    let text = format!("{short}github.com");
    assert_eq!(convert_with(&text, &smart()), text);
}

#[test]
fn the_domain_list_is_closed() {
    for tld in ["com", "net", "org", "mn", "io", "xyz", "dev", "info"] {
        let text = format!("site.{tld} ");
        assert_eq!(convert_with(&text, &smart()), text, "{tld}");
    }
    assert_eq!(convert_with("site.zzz ", &smart()), cyr("site.zzz "));
}

#[test]
fn prose_without_a_space_after_a_full_stop_stays_cyrillic() {
    for text in ["bi us.us ", "Ter yavsan.Ai ", "tsai.de ", "ok.me ", "1.us "] {
        assert_eq!(convert_with(text, &smart()), cyr(text), "{text}");
    }
}

#[test]
fn mn_is_a_domain_even_after_an_abbreviation_like_word() {
    // `mn` is in the list, so this turns Latin as a whole.
    assert_eq!(convert_with("sn.mn ", &smart()), "sn.mn ");
}

#[test]
fn a_run_that_turned_latin_does_not_survive_an_outside_edit() {
    let sain = cyr("sain");
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "bat@");
    for _ in 0..4 {
        c.process_key(Key::Backspace);
    }
    type_str(&mut c, "sain");
    assert_eq!(c.preedit(), sain);

    let mut c = Composer::with_options(smart());
    type_str(&mut c, "bat@");
    c.process_key(Key::Left);
    c.process_key(Key::Left);
    type_str(&mut c, "sain");
    assert_eq!(c.preedit(), sain);

    let mut c = Composer::with_options(smart());
    type_str(&mut c, "bat@");
    let r = c.process_key(Key::Escape);
    assert!(!r.consumed);
    type_str(&mut c, "sain");
    assert_eq!(c.preedit(), sain);
}

#[test]
fn a_caret_move_inside_a_word_stops_the_run() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "bat@gma");
    c.process_key(Key::Left);
    type_str(&mut c, "x");
    c.process_key(Key::Space);
    type_str(&mut c, "sain");
    assert_eq!(c.preedit(), cyr("sain"));
}

#[test]
fn links_with_punctuation_keep_only_the_domain_latin() {
    let sain = cyr("sain");
    let want = format!("{} github.com. {}", cyr("visit"), sain);
    assert_eq!(convert_with("visit github.com. sain", &smart()), want);
    let want = format!("{} github.com.", cyr("visit"));
    assert_eq!(convert_with("visit github.com.", &smart()), want);
    assert_eq!(
        convert_with("github.com, sain", &smart()),
        format!("github.com, {sain}")
    );
    assert_eq!(
        convert_with("(github.com) sain", &smart()),
        format!("(github.com) {sain}")
    );
    assert_eq!(convert_with("(github.com)", &smart()), "(github.com)");
    assert_eq!(
        convert_with("\"github.com\"! ", &smart()),
        "\"github.com\"! "
    );
}

#[test]
fn a_domain_with_punctuation_counts_delete_and_undo() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "(github.com),");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("(github.com),"));
    assert_eq!(r.delete_before, cyr("(github.com),").chars().count());
    let r = c.process_key(Key::Backspace);
    assert_eq!(r.delete_before, "(github.com), ".chars().count());
    assert_eq!(r.commit, Some(cyr("(github.com),")));
}

#[test]
fn a_slash_after_a_domain_keeps_the_path_latin() {
    let url = "github.com/TengisCh/buuz";
    assert_eq!(convert_with(url, &smart()), url);
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "github.com");
    let r = c.process_key(Key::Symbol('/'));
    assert!(r.consumed);
    assert_eq!(r.delete_before, cyr("github.").chars().count());
    assert_eq!(r.commit.as_deref(), Some("github.com/"));
    assert_eq!(convert_with("ok.zzz/ab", &smart()), cyr("ok.zzz/ab"));
}

#[test]
fn a_latin_run_gives_no_particle_evidence() {
    let uu = cyr("uu");
    for text in ["github.dev uu", "bat@x.dev uu"] {
        let mut c = Composer::with_options(smart());
        type_str(&mut c, text);
        assert_eq!(c.preedit(), uu, "{text}");
    }
}

#[test]
fn an_abbreviation_inside_a_run_is_undone_with_the_run() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "bn.com");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("bn.com"));
    assert_eq!(r.delete_before, "байна.".chars().count());
    let r = c.process_key(Key::Backspace);
    assert_eq!(r.delete_before, "bn.com ".chars().count());
    assert_eq!(r.commit, Some(cyr("bn.com")));
    assert!(r.commit.as_deref().unwrap().starts_with("байна."));
}

#[test]
fn an_abbreviation_then_at_sign_deletes_the_expansion() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "bn.bat");
    let r = c.process_key(Key::Symbol('@'));
    assert_eq!(r.delete_before, "байна.".chars().count());
    assert_eq!(r.commit.as_deref(), Some("bn.bat@"));
}

#[test]
fn enter_arms_no_undo() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "github.com");
    let r = c.process_key(Key::Other);
    assert_eq!(r.commit.as_deref(), Some("github.com"));
    assert!(!c.wants_key(Key::Backspace));
    assert!(!c.process_key(Key::Backspace).consumed);
}

#[test]
fn a_domain_before_a_newline_is_latin() {
    assert_eq!(
        convert_with("github.com\nsain", &smart()),
        format!("github.com\n{}", cyr("sain"))
    );
}

#[test]
fn undo_restores_the_cyrillic_that_was_shown() {
    let mut c = Composer::with_options(smart());
    type_str(&mut c, "github.com");
    c.process_key(Key::Escape); // the last label in Latin
    assert_eq!(c.preedit(), "com");
    c.process_key(Key::Space);
    let r = c.process_key(Key::Backspace);
    assert_eq!(r.commit, Some(format!("{}com", cyr("github."))));
}
