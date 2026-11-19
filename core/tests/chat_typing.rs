// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0.

//! Real chat typing from the maintainer: shortcuts and the short particle `u`.
//! Each case is a word that the maintainer typed for an everyday sentence.

use std::sync::Arc;

use buuz_core::{convert_with, Options, WordList};

const REAL: &str = include_str!("../../data/wordlist/mn.txt");

/// The web defaults: smart mode with the word list.
fn web() -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        ..Options::smart()
    }
}

fn eq(input: &str, want: &str) {
    assert_eq!(convert_with(input, &web()), want, "input {input:?}");
}

#[test]
fn chat_shortcuts_expand() {
    eq("snu yu bn?", "сайн уу юу байна?");
    eq("sainu yu bn ", "сайн уу юу байна ");
    eq("ih huiten bna, ", "их хүйтэн байна, ");
    eq("mar heden ", "маргааш хэдэн ");
    eq("bodood uzi ", "бодоод үзье ");
    eq("daraa yri ", "дараа ярья ");
    eq("huleej baiy ", "хүлээж байя ");
    eq("mungu baihgui shde ", "мөнгө байхгүй ш дээ ");
}

#[test]
fn shortcut_case_follows_the_keys() {
    eq("Snu ", "Сайн уу ");
    eq("Yu ", "Юу ");
}

#[test]
fn a_short_particle_u_is_doubled_after_a_word() {
    eq("boloh u?", "болох уу?");
    eq("uusan u?", "уусан уу?");
    eq("uzsen u ", "үзсэн үү ");
    eq("Boloh U?", "Болох Уу?");
}

#[test]
fn a_lone_u_without_a_word_before_stays() {
    eq("u", "у");
    eq("u ", "у ");
}

#[test]
fn classic_mode_is_unchanged() {
    assert_eq!(
        convert_with("boloh u? snu ", &Options::classic()),
        "болох у? сну "
    );
}

#[test]
fn a_short_final_long_vowel_is_chosen_from_the_list() {
    eq("boro orson ", "бороо орсон ");
    eq("bi odo gerte ", "би одоо гэртээ ");
    eq("dara ", "дараа ");
    eq("ajlaasa garaad ", "ажлаасаа гараад ");
}

#[test]
fn a_listed_word_with_a_short_final_vowel_stays() {
    eq("shine ", "шинэ ");
    eq("ene ", "энэ ");
}

#[test]
fn a_merged_past_and_particle_split() {
    eq("boroo orsnu?", "бороо орсон уу?");
    eq("irsnu ", "ирсэн үү ");
    eq("uzsnu ", "үзсэн үү ");
    eq("bolsnuu ", "болсон уу ");
}

#[test]
fn a_merged_sh_dee_splits() {
    eq("mungu baihguishde ", "мөнгө байхгүй ш дээ ");
    eq("medehguishdee ", "мэдэхгүй ш дээ ");
}

#[test]
fn a_merged_dee_splits_after_a_chat_word() {
    eq("zugerdee ", "зүгээр дээ ");
    eq("zugeerdee ", "зүгээр дээ ");
    eq("Bainadaa ", "Байна даа ");
}

#[test]
fn a_case_ending_is_not_split() {
    // -даа/-дээ is also the dative with a reflexive ending.
    eq("ajildaa ", "ажилдаа ");
}
