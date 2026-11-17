// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. Expected values derive from the rule table and word-state
// logic of the original Buuz IME (Composer.cpp) and its docs/table.html.

use buuz_core::{convert, Composer, Key};

fn eq(input: &str, want: &str) {
    assert_eq!(convert(input), want, "input {input:?}");
}

#[test]
fn headline_examples() {
    eq("buuz id'ye", "бууз идье");
    eq("sain baina uu", "сайн байна уу");
}

#[test]
fn conversion_table_from_docs() {
    // (Latin, Cyrillic) pairs for a single letter, from docs/table.html.
    let cases = [
        ("a", "а"),
        ("b", "б"),
        ("v", "в"),
        ("g", "г"),
        ("d", "д"),
        ("ye", "е"),
        ("yo", "ё"),
        ("j", "ж"),
        ("z", "з"),
        ("i", "и"),
        ("ii", "ий"),
        ("k", "к"),
        ("l", "л"),
        ("m", "м"),
        ("n", "н"),
        ("o", "о"),
        ("p", "п"),
        ("r", "р"),
        ("s", "с"),
        ("t", "т"),
        ("u", "у"),
        ("f", "ф"),
        ("h", "х"),
        ("kh", "х"),
        ("x", "х"),
        ("c", "ц"),
        ("ch", "ч"),
        ("sh", "ш"),
        ("sxc", "щ"),
        ("y", "ы"),
        ("iii", "ы"),
        ("e", "э"),
        ("yu", "ю"),
        ("ya", "я"),
        ("\"", "ъ"),
        ("\"\"", "Ъ"),
        ("'", "ь"),
        ("''", "Ь"),
    ];
    for (l, c) in cases {
        eq(l, c);
    }
}

#[test]
fn uppercase_forms() {
    eq("A", "А");
    eq("Sain", "Сайн");
    eq("SAIN", "САЙН");
    eq("Ch", "Ч");
    eq("CH", "Ч");
    eq("Ye", "Е");
    eq("YE", "Е");
    eq("Ya", "Я");
    eq("Sh", "Ш");
}

#[test]
fn case_quirks_of_multi_letter_rules() {
    // The k-th alphabetic input char controls the k-th output char; extra
    // input letters only matter for matching (original behaviour).
    eq("cH", "ч");
    eq("yA", "я");
    eq("sH", "ш");
    eq("aI", "аЙ");
    eq("Ai", "Ай");
}

#[test]
fn hard_and_soft_signs_ignore_case_rules() {
    eq("\"", "ъ");
    eq("\"\"", "Ъ");
    eq("'", "ь");
    eq("''", "Ь");
}

#[test]
fn vowel_harmony_male_word() {
    // Words start male: o -> о, u -> у.
    eq("boov", "боов");
    eq("uu", "уу");
    eq("buuz", "бууз");
}

#[test]
fn vowel_harmony_female_after_e_q_w() {
    // E / Ye / Q / W / O' / U' switch the word to female: o -> ө, u -> ү.
    eq("ene", "энэ");
    eq("eo", "эө");
    eq("eu", "эү");
    eq("yeo", "еө");
    eq("qo", "өө");
    eq("wu", "үү");
    eq("tuv", "тув");
    eq("tevu", "тэвү"); // "tevu": e makes the word female
    eq("ö", "ө"); // umlauts are input for ө and ү
}

#[test]
fn explicit_o_u_variants() {
    eq("q", "ө");
    eq("w", "ү");
    eq("o'", "ө");
    eq("'o", "ө");
    eq("u'", "ү");
    eq("'u", "ү");
    eq("o\"", "о");
    eq("\"o", "о");
    eq("u\"", "у");
    eq("\"u", "у");
    // diphthong forms
    eq("oi", "ой");
    eq("ui", "уй");
    eq("qi", "өй");
    eq("wi", "үй");
    eq("ei", "эй");
    eq("ai", "ай");
    // male-making rule resets a female word
    eq("eo\"o", "эоо");
}

#[test]
fn female_word_makes_o_and_u_front() {
    eq("ehi", "эхи");
    eq("ehu", "эхү");
    eq("ehoi", "эхөй");
    // A makes the word male again.
    eq("eau", "эау");
    eq("eao", "эао");
    // x_m|x_f letters do not change state.
    eq("enou", "энөү");
}

#[test]
fn longest_match_wins() {
    eq("iii", "ы");
    eq("ii", "ий");
    eq("iiii", "ыи");
    eq("sxc", "щ");
    eq("sxx", "схх");
    eq("kh", "х");
    eq("khh", "хх");
}

#[test]
fn spaces_and_punctuation_end_words_and_reset_harmony() {
    eq("e o", "э о");
    eq("e. o", "э. о");
    eq("1ene 2", "1энэ 2");
    eq("", "");
}

#[test]
fn composer_basic_flow() {
    let mut c = Composer::new();
    assert!(!c.wants_key(Key::Backspace));
    assert!(!c.wants_key(Key::Other));
    for ch in "buuz".chars() {
        let r = c.process_key(Key::Char(ch));
        assert!(r.consumed && r.commit.is_none());
    }
    assert_eq!(c.preedit(), "бууз");
    assert_eq!(c.cursor(), 4);
    assert_eq!(c.raw(), "buuz");
    // Space commits and passes through.
    let r = c.process_key(Key::Other);
    assert!(!r.consumed);
    assert_eq!(r.commit.as_deref(), Some("бууз"));
    assert!(!c.is_composing());
}

#[test]
fn composer_backspace_recomposes_and_cancels() {
    let mut c = Composer::new();
    for ch in "sh".chars() {
        c.process_key(Key::Char(ch));
    }
    assert_eq!(c.preedit(), "ш");
    c.process_key(Key::Backspace);
    assert_eq!(c.preedit(), "с");
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed && r.commit.is_none());
    assert!(!c.is_composing());
    // Backspace now passes through.
    assert!(!c.process_key(Key::Backspace).consumed);
}

#[test]
fn composer_cursor_editing() {
    let mut c = Composer::new();
    for ch in "sain".chars() {
        c.process_key(Key::Char(ch));
    }
    assert_eq!(c.preedit(), "сайн");
    c.process_key(Key::Left);
    assert_eq!(c.cursor(), 3);
    c.process_key(Key::Char('b'));
    assert_eq!(c.preedit(), "сайбн");
    assert_eq!(c.cursor(), 4);
    c.process_key(Key::Home);
    assert_eq!(c.cursor(), 0);
    c.process_key(Key::Delete);
    assert_eq!(c.preedit(), "айбн");
    c.process_key(Key::End);
    assert_eq!(c.cursor(), 4);
    assert_eq!(c.commit().as_deref(), Some("айбн"));
}

#[test]
fn cyrillic_reseed_after_cursor_move() {
    // After moving the cursor the buffer holds Cyrillic; typing 'i' after
    // а/о/у/э/ө/ү/и joins into the -й form via the Cyrillic rules.
    let mut c = Composer::new();
    c.process_key(Key::Char('a'));
    c.process_key(Key::Char('b'));
    c.process_key(Key::Left); // "а|б"
    c.process_key(Key::Char('i')); // "аи|б" -> "ай|б"
    assert_eq!(c.preedit(), "айб");
}

#[test]
fn modifiers_do_not_end_composition() {
    let mut c = Composer::new();
    c.process_key(Key::Char('a'));
    let r = c.process_key(Key::Modifier);
    assert!(r.consumed);
    assert!(c.is_composing());
}

#[test]
fn max_length_is_enforced() {
    let mut c = Composer::new();
    for _ in 0..buuz_core::MAX_COMP_LEN {
        assert!(!c.process_key(Key::Char('b')).rejected);
    }
    let r = c.process_key(Key::Char('b'));
    assert!(r.consumed && r.rejected);
    assert_eq!(c.preedit().chars().count(), buuz_core::MAX_COMP_LEN);
}

#[test]
fn expanded_rule_table_is_exported() {
    let t = buuz_core::export_rules();
    assert!(t.iter().any(|(f, to, m, _)| f == "Sh" && to == "Ш" && *m));
    assert!(t.iter().any(|(f, to, _, fm)| f == "o" && to == "ө" && *fm));
}

#[test]
fn umlaut_input() {
    for (i, o) in [
        ("hün", "хүн"),
        ("ögöh", "өгөх"),
        ("Ünee", "Үнээ"),
        ("öndör", "өндөр"),
        ("Öndör", "Өндөр"),
        ("ÖNDÖR", "ӨНДӨР"),
        ("ögoh", "өгөх"),
        ("hüühed", "хүүхэд"),
        ("öi", "өй"),
        ("Üi", "Үй"),
        ("üI", "үЙ"),
        ("ü", "ү"),
    ] {
        eq(i, o);
    }
    assert!(buuz_core::is_input_char('ü'));
    assert!(buuz_core::is_input_char('Ö'));
    assert!(!buuz_core::is_input_char('ä'));
    assert!(Composer::new().wants_key(Key::Char('ö')));
}
