// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. Cases come from issue #7 (smarter vowel harmony).

use buuz_core::{convert, convert_with, Composer, Key, Options};

fn eq(input: &str, want: &str) {
    assert_eq!(convert(input), want, "input {input:?}");
}

fn classic(input: &str, want: &str) {
    assert_eq!(
        convert_with(input, &Options::classic()),
        want,
        "input {input:?}"
    );
}

fn type_keys(c: &mut Composer, keys: &[Key]) {
    for &k in keys {
        c.process_key(k);
    }
}

fn chars(s: &str) -> Vec<Key> {
    s.chars().map(Key::Char).collect()
}

#[test]
fn look_ahead_gender() {
    eq("unee", "үнээ");
    eq("huuhed", "хүүхэд");
    eq("ovqq", "өвөө");
}

#[test]
fn ii_becomes_y_in_masculine_words() {
    eq("aaviin", "аавын");
    eq("nomiig", "номыг");
    eq("ohiniig", "охиныг");
    eq("mashiniig", "машиныг");
    eq("uuliin", "уулын");
    eq("hotiig", "хотыг");
}

#[test]
fn ii_stays_ii() {
    eq("geriin", "гэрийн");
    eq("bagshiig", "багшийг");
    eq("tuujiig", "туужийг");
    eq("achiin", "ачийн");
    // The typed ' is ь and stays; dropping it is suffix spelling (out of scope).
    eq("har'iin", "харьийн");
    eq("bii", "бий");
    eq("iim", "ийм");
    eq("chiig", "чийг");
    eq("angiin", "ангийн");
    eq("airgiig", "айргийг");
    eq("salhiig", "салхийг");
    eq("akadyemiig", "академийг");
}

#[test]
fn unchanged_words() {
    eq("hotyg", "хотыг");
    eq("wg", "үг");
    eq("qrqh", "өрөх");
    eq("Ulaanbaatar", "Улаанбаатар");
}

#[test]
fn particle_follows_previous_word() {
    eq("irsen uu", "ирсэн үү");
    eq("geree uu", "гэрээ үү");
    eq("baina uu", "байна уу");
    eq("sain uu", "сайн уу");
    eq("jaahan buuz ideh uu?", "жаахан бууз идэх үү?");
    eq("öndör uu", "өндөр үү");
}

#[test]
fn particle_needs_a_plain_space() {
    eq("irsen. uu", "ирсэн. уу");
    eq("irsen  uu", "ирсэн  үү");
}

#[test]
fn gui_suffix_is_not_evidence() {
    eq("bolohgwi", "болохгүй");
    eq("uuhgwi", "уухгүй");
    eq("olohgwi", "олохгүй");
    eq("haha, ingej bolohgwi ee", "хаха, ингэж болохгүй ээ");
    eq("ugwi", "угүй");
}

#[test]
fn ye_is_not_evidence() {
    eq("kofye", "кофе");
}

#[test]
fn guards() {
    eq("ijil", "ижил");
    eq("hiij", "хийж");
    eq("hiichihlee", "хийчихлээ");
    eq("hün", "хүн");
}

#[test]
fn forcing_ii_and_y() {
    eq("aavii'n", "аавийн");
    eq("aaviiin", "аавын");
    eq("aavyn", "аавын");
}

#[test]
fn ii_keeps_case() {
    eq("AAVIIN", "ААВЫН");
    eq("Nomiig", "Номыг");
}

#[test]
fn classic_mode_is_the_original() {
    classic("aaviin", "аавийн");
    classic("unee", "унээ");
    classic("ovqq", "овөө");
    classic("irsen uu", "ирсэн уу");
}

#[test]
fn composer_look_ahead_preedit() {
    let mut c = Composer::new();
    let mut seen = Vec::new();
    for k in chars("une") {
        c.process_key(k);
        seen.push(c.preedit());
    }
    assert_eq!(seen, ["у", "ун", "үнэ"]);
}

#[test]
fn composer_particle_after_space() {
    let mut c = Composer::new();
    type_keys(&mut c, &chars("irsen"));
    c.process_key(Key::Space);
    c.process_key(Key::Char('u'));
    assert_eq!(c.preedit(), "ү");
    c.process_key(Key::Char('u'));
    assert_eq!(c.preedit(), "үү");
}

#[test]
fn composer_other_key_forgets_context() {
    let mut c = Composer::new();
    type_keys(&mut c, &chars("irsen"));
    c.process_key(Key::Other);
    type_keys(&mut c, &chars("uu"));
    assert_eq!(c.preedit(), "уу");
}

#[test]
fn composer_clear_context() {
    let mut c = Composer::new();
    type_keys(&mut c, &chars("irsen"));
    c.process_key(Key::Space);
    c.clear_context();
    type_keys(&mut c, &chars("uu"));
    assert_eq!(c.preedit(), "уу");
}

#[test]
fn composer_modifier_keeps_context() {
    let mut c = Composer::new();
    type_keys(&mut c, &chars("irsen"));
    c.process_key(Key::Space);
    c.process_key(Key::Modifier);
    type_keys(&mut c, &chars("uu"));
    assert_eq!(c.preedit(), "үү");
}

#[test]
fn composer_commit_forgets_context() {
    let mut c = Composer::new();
    type_keys(&mut c, &chars("irsen"));
    c.process_key(Key::Space);
    assert_eq!(c.commit(), None);
    type_keys(&mut c, &chars("uu"));
    assert_eq!(c.preedit(), "уу");
}

#[test]
fn composer_set_options() {
    let mut c = Composer::new();
    c.set_options(Options::classic());
    type_keys(&mut c, &chars("unee"));
    assert_eq!(c.preedit(), "унээ");
}
