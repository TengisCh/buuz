// SPDX-License-Identifier: Apache-2.0

//! Calibration (issue #4): from sentences and what the user typed to
//! suggested `[keys]` entries.

use std::time::Instant;

use buuz_core::{calibrate, convert_with, Calibration, Config, Options, Suggestion};

/// The Cyrillic side comes from the default table, so no Mongolian is spelled
/// here by hand. The Latin inputs are phrases from `web/site/src/phrases.json`.
fn cyr(latin: &str) -> String {
    convert_with(latin, &Options::smart())
}

/// `(cyrillic, typed)` from the default Latin and what the user typed.
fn pair(default_latin: &str, typed: &str) -> (String, String) {
    (cyr(default_latin), typed.to_string())
}

fn run(pairs: &[(String, String)]) -> Calibration {
    let refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    calibrate(&refs)
}

fn typed_default(list: &[&str]) -> Vec<(String, String)> {
    list.iter().map(|l| pair(l, l)).collect()
}

const DEFAULTS: &[&str] = &[
    "sain uu? 👋",
    "qnqqdqr saihan qdqr baina ☀️",
    "chi yuu hiij baina?",
    "za, oilgoloo 👌",
    "haha, ingej bolohgwi ee",
    "Mongol hel goyo shww",
    "ter yuu ve? lol",
    "yaasan hqqrhqn yum be 🥺",
    "buuzaa ideed unt\"ya 😴",
    "unshlaa, harin hariu bichsengwi 👀",
    "chamd amjilt hws'ye! ✨",
];

#[test]
fn default_typing_gives_no_suggestions() {
    let c = run(&typed_default(DEFAULTS));
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
    assert!(c.conflicts.is_empty(), "{:?}", c.conflicts);
}

#[test]
fn o_for_oe_is_the_default_and_o_u_are_separate_letters() {
    let c = run(&[pair("hqqrhqn buuz 🥺", "hoorhon buuz 🥺")]);
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
}

#[test]
fn v_for_u_front_with_no_v_vowel_proposes_and_displaces() {
    let c = run(&[
        pair("zwgeer zwgeer 😎", "zvgeer zvgeer 😎"),
        pair("hairtai shww ❤️", "hairtai shvv ❤️"),
    ]);
    assert_eq!(c.suggestions.len(), 1, "{:?}", c.suggestions);
    let s = &c.suggestions[0];
    assert_eq!((s.letter, s.chunk.as_str()), ('ү', "v"));
    // `shvv` is the built-in double V (үү), not a habit: only `zvgeer` counts.
    assert_eq!(s.count, 2);
    assert_eq!(s.default_count, 1);
    assert!(!s.tie);
    assert_eq!(s.displaces, Some('в'));
    assert!(c.conflicts.is_empty());
}

fn swap_input() -> Vec<(String, String)> {
    vec![
        pair("zwgeer zwgeer 😎", "zvgeer zvgeer 😎"),
        pair("ter yuu ve? lol", "ter yuu we? lol"),
        pair("aimaar goyo vaib 🔥", "aimaar goyo waib 🔥"),
    ]
}

#[test]
fn a_swap_gives_two_suggestions() {
    let c = run(&swap_input());
    assert!(c.conflicts.is_empty(), "{:?}", c.conflicts);
    let got: Vec<(char, &str, Option<char>)> = c
        .suggestions
        .iter()
        .map(|s| (s.letter, s.chunk.as_str(), s.displaces))
        .collect();
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got.contains(&('ү', "v", Some('в'))), "{got:?}");
    assert!(got.contains(&('в', "w", Some('ү'))), "{got:?}");
}

#[test]
fn a_key_typed_for_two_letters_is_a_conflict() {
    // v for ү and v for в.
    let c = run(&[
        pair("zwgeer zwgeer 😎", "zvgeer zvgeer 😎"),
        pair("ter yuu ve? lol", "ter yuu ve? lol"),
    ]);
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
    assert_eq!(c.conflicts.len(), 1, "{:?}", c.conflicts);
    let k = &c.conflicts[0];
    assert_eq!((k.letter, k.chunk.as_str(), k.count), ('ү', "v", 2));
    assert_eq!((k.other, k.other_count), ('в', 1));
}

#[test]
fn h_for_ts_with_h_for_kh_is_a_conflict() {
    let c = run(&[
        pair("cagaan sar oirtloo 🎉", "hagaan sar oirtloo 🎉"),
        pair("cai uuh uu?", "hai uuh uu?"),
        pair("chi yuu hiij baina?", "chi yuu hiij baina?"),
    ]);
    // ц typed as h twice, and х typed as h too (uuh, hiij).
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
    assert_eq!(c.conflicts.len(), 1, "{:?}", c.conflicts);
    let k = &c.conflicts[0];
    assert_eq!((k.letter, k.chunk.as_str(), k.other), ('ц', "h", 'х'));
    assert_eq!(k.count, 2);
    assert!(k.other_count >= 2);
}

#[test]
fn vv_is_a_default_so_it_is_not_a_second_suggestion() {
    // `vv` is the built-in double V: it counts as a default use of ү, so
    // two single `v` for ү do not beat it, and `vv` is never proposed.
    let c = run(&[
        pair("zwgeer zwgeer 😎", "zvgeer zvgeer 😎"),
        pair(
            "Ulaanbaatar hwiten baina ❄️",
            "Ulaanbaatar hvviten baina ❄️",
        ),
        pair(
            "Ulaanbaatar hwiten baina ❄️",
            "Ulaanbaatar hvviten baina ❄️",
        ),
    ]);
    assert!(
        c.suggestions.iter().all(|s| s.chunk != "vv"),
        "{:?}",
        c.suggestions
    );
    assert!(c.conflicts.is_empty(), "{:?}", c.conflicts);
}

#[test]
fn a_built_in_key_is_not_suggested_even_when_mixed() {
    // v three times, u' (a built-in key for ү) twice: only v is proposed.
    let c = run(&[
        pair("zwgeer zwgeer 😎", "zvgeer zvgeer 😎"),
        pair("Mongol hel goyo shww", "Mongol hel goyo shu'u'"),
        pair("whtlee ineelee 😂", "vhtlee ineelee 😂"),
    ]);
    assert_eq!(c.suggestions.len(), 1, "{:?}", c.suggestions);
    let s = &c.suggestions[0];
    assert_eq!((s.letter, s.chunk.as_str(), s.count), ('ү', "v", 3));
    assert_eq!(s.default_count, 2);
}

#[test]
fn long_vowels_align_letter_by_letter() {
    // шүү typed shww: two ү, each typed w (built-in). shvv is the built-in
    // double V and gives no suggestion either.
    let c = run(&[pair("hairtai shww ❤️", "hairtai shvv ❤️")]);
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
    let c = run(&[pair("hairtai shww ❤️", "hairtai shvw ❤️")]);
    assert_eq!(c.suggestions.len(), 0, "{:?}", c.suggestions);
}

#[test]
fn upper_case_is_folded() {
    let c = run(&[
        pair("Ulaanbaatar hwiten baina ❄️", "ULAANBAATAR HVITEN BAINA ❄️"),
        pair("zwgeer zwgeer 😎", "ZVGEER ZVGEER 😎"),
    ]);
    assert_eq!(c.suggestions.len(), 1, "{:?}", c.suggestions);
    assert_eq!(c.suggestions[0].chunk, "v");
    assert_eq!(c.suggestions[0].count, 3);
}

#[test]
fn letters_that_never_appear_are_skipped() {
    let c = run(&typed_default(&["sain uu? 👋"]));
    assert!(c.skipped_letters.contains(&'щ'));
    assert!(c.skipped_letters.contains(&'п'));
    assert!(!c.skipped_letters.contains(&'с'));
    assert!(!c.skipped_letters.contains(&'у'));
}

#[test]
fn apply_adds_all_keys_and_converts_the_typed_text() {
    let input = swap_input();
    let c = run(&input);
    let mut config = Config::default();
    c.apply(&mut config).unwrap();
    let opts = config.options();
    for (cyrillic, typed) in &input {
        assert_eq!(&convert_with(typed, &opts), cyrillic, "typed {typed:?}");
    }
    // Both entries of the swap hold, whatever the order.
    assert_eq!(convert_with("v w", &opts), "ү в");
}

#[test]
fn a_calibration_with_nothing_to_suggest_leaves_the_config_alone() {
    let c = run(&typed_default(DEFAULTS));
    let mut config = Config::default();
    c.apply(&mut config).unwrap();
    assert_eq!(config, Config::default());
}

/// Ten sentences of about 40 letters.
fn ten_sentences() -> Vec<(String, String)> {
    let base = [
        "chi bol neeh goyo hwn shww",
        "sar shinedee saihan shinelj baina uu?",
        "unshlaa, harin hariu bichsengwi",
        "yaasan hqqrhqn yum be",
        "uuchlaarai, hocorloo",
    ];
    (0..10)
        .map(|i| {
            let l = base[i % base.len()];
            pair(l, &l.replace('w', "v"))
        })
        .collect()
}

/// A generous bound in a debug build (the product bound is 50 ms in release).
#[test]
fn ten_sentences_calibrate_fast() {
    let input = ten_sentences();
    let start = Instant::now();
    let c = run(&input);
    let took = start.elapsed();
    assert!(!c.suggestions.is_empty());
    let limit_ms = if cfg!(debug_assertions) { 2000 } else { 50 };
    assert!(took.as_millis() < limit_ms, "took {took:?}");
}

#[test]
#[ignore = "bench: run with --release -- --ignored --nocapture"]
fn bench_ten_sentences() {
    let input = ten_sentences();
    let start = Instant::now();
    let c = run(&input);
    println!(
        "10 sentences: {:?}, {} suggestions",
        start.elapsed(),
        c.suggestions.len()
    );
}

// ---- review fixes ----

fn is_clean(c: &Calibration) -> bool {
    c.suggestions.is_empty() && c.conflicts.is_empty() && c.unsupported.is_empty()
}

fn sugg(c: &Calibration) -> Vec<(char, &str, bool)> {
    c.suggestions
        .iter()
        .map(|s: &Suggestion| (s.letter, s.chunk.as_str(), s.pair))
        .collect()
}

#[test]
fn smart_ii_for_y_is_the_default() {
    let c = run(&[
        ("аавын номыг".to_string(), "aaviin nomiig".to_string()),
        ("хотыг уулын".to_string(), "hotiig uuliin".to_string()),
    ]);
    assert!(is_clean(&c), "{c:?}");
    assert!(c.unmatched.is_empty());
}

#[test]
fn a_missing_question_mark_does_not_ruin_the_sentence() {
    let c = run(&[
        pair("sain uu?", "sain uu"),
        pair("chi yuu hiij baina?", "chi yuu hiij baina?"),
    ]);
    assert!(is_clean(&c), "{c:?}");
    assert!(c.unmatched.is_empty(), "{:?}", c.unmatched);
    // The rest of the sentence still counts.
    let c = run(&[pair("zwgeer zwgeer?", "zvgeer zvgeer")]);
    assert_eq!(sugg(&c), vec![('ү', "v", false)], "{c:?}");
}

#[test]
fn a_missed_word_gives_nothing_wrong() {
    let c = run(&[pair("chi yuu hiij baina?", "chi hiij baina?")]);
    assert!(is_clean(&c), "{c:?}");
}

#[test]
fn a_sentence_that_does_not_match_is_reported_and_not_counted() {
    let c = run(&[
        pair("sain baina uu", "zzzz"),
        pair("zwgeer zwgeer 😎", "zvgeer zvgeer 😎"),
    ]);
    assert_eq!(c.unmatched, vec![0]);
    assert_eq!(c.suggestions.len(), 1);
    // The unmatched sentence adds nothing: its letters count as skipped.
    assert!(c.skipped_letters.contains(&'б'));
}

#[test]
fn a_dropped_emoji_variation_selector_is_harmless() {
    let c = run(&[pair("hairtai shww ❤️", "hairtai shww ❤")]);
    assert!(is_clean(&c), "{c:?}");
    assert!(c.unmatched.is_empty());
    let c = run(&[pair("zwgeer zwgeer ❤️", "zvgeer zvgeer ❤")]);
    assert_eq!(sugg(&c), vec![('ү', "v", false)], "{c:?}");
}

#[test]
fn decomposed_text_aligns_like_composed_text() {
    // й as и + combining breve on the Cyrillic side, ü as u + diaeresis typed.
    let (cy, _) = pair("Ulaanbaatar hwiten baina ❄️", "");
    let nfd = cy.replace('й', "и\u{306}");
    let c = run(&[(nfd.clone(), "Ulaanbaatar hwiten baina ❄️".to_string())]);
    assert!(is_clean(&c), "{c:?}");
    assert!(c.unmatched.is_empty(), "{:?}", c.unmatched);
    let c = run(&[
        (
            nfd.clone(),
            "ulaanbaatar hu\u{308}iten baina ❄️".to_string(),
        ),
        (nfd, "ulaanbaatar hu\u{308}iten baina ❄️".to_string()),
    ]);
    assert!(is_clean(&c), "{c:?}");
    // A decomposed sentence still teaches: v for ү.
    let c = run(&[(
        cyr("zwgeer hwiten").replace('й', "и\u{306}"),
        "zvgeer hviten".to_string(),
    )]);
    assert_eq!(sugg(&c), vec![('ү', "v", false)], "{c:?}");
}

#[test]
fn big_input_is_capped_and_reported() {
    let long = "a".repeat(501);
    let c = run(&[(long.clone(), long)]);
    assert_eq!(c.unmatched, vec![0]);
    let ok = "a".repeat(500);
    let c = run(&[(ok.clone(), ok)]);
    assert!(c.unmatched.is_empty());
    let many: Vec<(String, String)> = (0..52)
        .map(|_| pair("sain uu? 👋", "sain uu? 👋"))
        .collect();
    let c = run(&many);
    assert_eq!(c.unmatched, vec![50, 51]);
    // A huge pair does not hang or panic.
    let huge = "я".repeat(200_000);
    let c = run(&[(huge.clone(), huge)]);
    assert_eq!(c.unmatched, vec![0]);
}

#[test]
fn a_typed_word_in_the_middle_does_not_make_a_chunk() {
    let c = run(&[
        pair("zwgeer zwgeer", "zvgeer blah zvgeer"),
        pair("zwgeer zwgeer", "zvgeer blah zvgeer"),
    ]);
    assert_eq!(sugg(&c), vec![('ү', "v", false)], "{c:?}");
    assert!(!c.suggestions[0].tie);
    assert!(c.unmatched.is_empty());
}

#[test]
fn equal_counts_for_two_letters_propose_neither() {
    // h for ц twice and for ч twice; no х in the input.
    let c = run(&[
        pair("cagaan sar oirtloo 🎉", "hagaan sar oirtloo 🎉"),
        pair("cagaan sar oirtloo 🎉", "hagaan sar oirtloo 🎉"),
        pair("za, ochloo 🏃", "za, ohloo 🏃"),
        pair("za, ochloo 🏃", "za, ohloo 🏃"),
    ]);
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
    assert_eq!(c.conflicts.len(), 1, "{:?}", c.conflicts);
    let k = &c.conflicts[0];
    assert_eq!((k.letter, k.other, k.chunk.as_str()), ('ц', 'ч', "h"));
}

#[test]
fn three_letters_on_one_chunk_give_three_pair_conflicts() {
    // x for ц, ч and д twice each; no х in the input.
    let c = run(&[
        pair("cagaan sar oirtloo 🎉", "xagaan sar oirtloo 🎉"),
        pair("cagaan sar oirtloo 🎉", "xagaan sar oirtloo 🎉"),
        pair("za, ochloo 🏃", "za, oxloo 🏃"),
        pair("za, ochloo 🏃", "za, oxloo 🏃"),
        pair("buuz duuslaa 😭", "buuz xuuslaa 😭"),
        pair("buuz duuslaa 😭", "buuz xuuslaa 😭"),
    ]);
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
    assert_eq!(c.conflicts.len(), 3, "{:?}", c.conflicts);
}

#[test]
fn the_more_common_letter_wins_a_shared_chunk() {
    let c = run(&[
        pair("cagaan sar oirtloo 🎉", "hagaan sar oirtloo 🎉"),
        pair("cagaan sar oirtloo 🎉", "hagaan sar oirtloo 🎉"),
        pair("cagaan sar oirtloo 🎉", "hagaan sar oirtloo 🎉"),
        pair("za, ochloo 🏃", "za, ohloo 🏃"),
        pair("za, ochloo 🏃", "za, ohloo 🏃"),
    ]);
    assert_eq!(sugg(&c), vec![('ц', "h", false)], "{c:?}");
    assert_eq!(c.conflicts.len(), 1, "{:?}", c.conflicts);
    let k = &c.conflicts[0];
    assert_eq!((k.letter, k.other, k.chunk.as_str()), ('ч', 'ц', "h"));
}

#[test]
fn y_for_both_u_and_u_front_is_a_gender_pair() {
    let input = vec![pair("buuz unt zwgeer zwgeer", "byyz ynt zygeer zygeer")];
    let c = run(&input);
    assert_eq!(sugg(&c), vec![('у', "y", true)], "{c:?}");
    assert_eq!(c.suggestions[0].displaces, Some('ы'));
    let mut config = Config::default();
    c.apply(&mut config).unwrap();
    let opts = config.options();
    assert_eq!(convert_with(&input[0].1, &opts), input[0].0);
}

#[test]
fn keys_that_cannot_be_targeted_are_unsupported() {
    let c = run(&[pair("sain sain bain", "sajn sajn bajn")]);
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
    let u: Vec<_> = c
        .unsupported
        .iter()
        .map(|u| (u.letter, u.chunk.as_str(), u.count))
        .collect();
    assert_eq!(u, vec![('й', "j", 3)], "{c:?}");

    let c = run(&[pair("chi chi chamd", "4i 4i 4amd")]);
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
    let u: Vec<_> = c
        .unsupported
        .iter()
        .map(|u| (u.letter, u.chunk.as_str(), u.count))
        .collect();
    assert_eq!(u, vec![('ч', "4", 3)], "{c:?}");
}

#[test]
fn apply_keeps_existing_keys_and_is_all_or_nothing() {
    let mut config = Config::default();
    config.set_key("q", "ш").unwrap();
    let c = run(&swap_input());
    c.apply(&mut config).unwrap();
    let opts = config.options();
    assert_eq!(convert_with("q v w", &opts), "ш ү в");

    let mut config = Config::default();
    let before = config.clone();
    let mut bad = c.clone();
    bad.suggestions.push(Suggestion {
        letter: 'ч',
        chunk: "4".to_string(),
        count: 3,
        default_count: 0,
        tie: false,
        displaces: None,
        pair: false,
    });
    assert!(bad.apply(&mut config).is_err());
    assert_eq!(config, before);
}

#[test]
fn single_slips_are_not_suggestions() {
    let mut input = typed_default(DEFAULTS);
    input.push(pair("zwgeer zwgeer 😎", "zvgeer zwgeer 😎"));
    input.push(pair("ter yuu ve? lol", "ter yuu we? lol"));
    let c = run(&input);
    assert!(is_clean(&c), "{c:?}");
}

#[test]
fn empty_input_and_empty_strings() {
    let c = run(&[]);
    assert!(is_clean(&c));
    assert!(c.unmatched.is_empty());
    assert_eq!(c.skipped_letters.len(), 35);
    let c = run(&[(String::new(), String::new())]);
    assert!(is_clean(&c));
    assert!(c.unmatched.is_empty());
}

#[test]
fn umlaut_capitals_and_latin_in_the_sentence_are_fine() {
    let c = run(&[pair("zwgeer zwgeer 😎", "ZÜGEER ZÜGEER 😎")]);
    assert!(is_clean(&c), "{c:?}");
    let c = run(&[("бууз ok ok".to_string(), "buuz ok ok".to_string())]);
    assert!(is_clean(&c), "{c:?}");
    assert!(c.unmatched.is_empty());
}
