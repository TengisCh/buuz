// SPDX-License-Identifier: Apache-2.0

//! Suffix choice (issue #52): at Space or punctuation, the word list picks
//! the case suffix of a word. It looks at the genitive, the accusative, the
//! ablative and the comitative (`hotaas` gives `хотоос`, `usaas` gives
//! `уснаас`). Backspace right after Space undoes it.

use std::sync::Arc;
use std::time::Instant;

use buuz_core::{Composer, Config, Key, Learned, Options, WordList};

const REAL: &str = include_str!("../../data/wordlist/mn.txt");

fn real() -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        ..Options::smart()
    }
}

/// Smart mode with a small list, and the other word features off, so a test
/// shows only what suffix choice does.
fn small(list: &str) -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(list).unwrap())),
        word_choice: false,
        autocorrect: false,
        ..Options::smart()
    }
}

fn type_in(c: &mut Composer, s: &str) {
    for ch in s.chars() {
        c.process_key(Key::Char(ch));
    }
}

/// Type `s`, then `end`, and return the commit of the last key.
fn commit_with(opts: Options, s: &str, end: Key) -> String {
    let mut c = Composer::with_options(opts);
    type_in(&mut c, s);
    c.process_key(end).commit.unwrap_or_default()
}

/// Type `s`, then Space, and return the commit of the Space key.
fn commit(opts: Options, s: &str) -> String {
    commit_with(opts, s, Key::Space)
}

#[test]
fn on_in_smart_and_off_in_classic() {
    assert!(Options::smart().suffix_choice);
    assert!(Options::default().suffix_choice);
    assert!(!Options::classic().suffix_choice);
}

#[test]
fn listed_typo_form_loses_by_ratio() {
    // усын 12, усны 3131.
    assert_eq!(commit(real(), "usiin"), "усны");
}

#[test]
fn o_stem_ablative() {
    assert_eq!(commit(real(), "hotaas"), "хотоос");
}

#[test]
fn long_vowel_stem_genitive() {
    // тооны 2360, тоон 631: the winner is 3 times the next.
    assert_eq!(commit(real(), "toogiin"), "тооны");
}

#[test]
fn hidden_n_ablative_real_list() {
    assert_eq!(commit(real(), "usaas"), "уснаас");
}

#[test]
fn front_vowel_ablative() {
    assert_eq!(commit(real(), "geraas"), "гэрээс");
}

#[test]
fn no_clear_winner_stays() {
    // уурын 642 is listed, and the candidates are not 50 times bigger.
    assert_eq!(commit(real(), "uuriin"), "уурын");
}

#[test]
fn leading_capital_stays() {
    assert_eq!(commit(real(), "Hotaas"), "Хотоос");
}

#[test]
fn backspace_undoes_and_rejects() {
    let mut c = Composer::with_options(real());
    type_in(&mut c, "hotaas");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("хотоос"));
    assert!(r.corrected);
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(c.preedit(), "хотаас");
    assert_eq!(r.learned, Some(Learned::Rejected("хотаас".into())));
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("хотаас"));
    // The word is not chosen again in this session.
    type_in(&mut c, "hotaas");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("хотаас"));
}

#[test]
fn off_in_classic_mode_or_without_list() {
    let classic = Options {
        words: real().words,
        ..Options::classic()
    };
    assert_eq!(commit(classic, "hotaas"), "хотаас");
    assert_eq!(commit(Options::smart(), "hotaas"), "хотаас");
    // Rounding harmony would also change хотаас, so it is off here.
    let off = Options {
        suffix_choice: false,
        rounding_harmony: false,
        ..real()
    };
    assert_eq!(commit(off, "hotaas"), "хотаас");
}

#[test]
fn comitative() {
    assert_eq!(commit(real(), "hottai"), "хоттой");
}

#[test]
fn accusative_after_long_vowel() {
    assert_eq!(commit(real(), "toogiig"), "тоог");
}

#[test]
fn g_form_to_n_form_ablative() {
    assert_eq!(commit(real(), "shireegees"), "ширээнээс");
}

#[test]
fn ablative_after_long_vowel() {
    assert_eq!(commit(real(), "toogoos"), "тооноос");
}

#[test]
fn winner_needs_count_50() {
    // номоос 45 and номноос 45: no winner.
    assert_eq!(commit(real(), "nomaas"), "номаас");
}

#[test]
fn stem_is_not_edited() {
    // ажлын 2172 needs a stem change, which is out of scope.
    assert_eq!(commit(real(), "ajiliin"), "ажилын");
}

#[test]
fn common_listed_word_stays() {
    assert_eq!(commit(real(), "hotiin"), "хотын");
}

#[test]
fn listed_word_with_50_or_more_stays() {
    // сарны 324 is listed; сарын 44902 is more than 50 times bigger.
    // сумны 101 against сумын 7796 is the same.
    assert_eq!(commit(real(), "sarnii"), "сарны");
    assert_eq!(commit(real(), "sumnii"), "сумны");
}

#[test]
fn stem_letter_is_not_a_suffix() {
    assert_eq!(commit(real(), "negiig"), "нэгийг");
    assert_eq!(commit(real(), "bolsonii"), "болсоны");
    assert_eq!(commit(real(), "ulaanii"), "улааны");
}

#[test]
fn one_letter_variant_is_a_candidate_only_for_an_unlisted_word() {
    // The typed word is unlisted, so тоон may win (G1).
    let list = "тоо\t3000\nтоон\t900\n";
    assert_eq!(commit(small(list), "toogiin"), "тоон");
    // A one-letter variant is never a typed parse: `тоон` has no stem `тоо`,
    // so a word that is listed and common stays.
    assert_eq!(
        commit(small("тоо\t3000\nтоон\t900\nтоогийн\t60\n"), "toogiin"),
        "тоогийн"
    );
}

#[test]
fn g_form_only_after_a_vowel() {
    // G2: the list has үүргийг (1033) and үүрийг (20). The stem үүр ends in
    // a consonant, so үүрийг is not read as үүр + г + ийг.
    assert_eq!(commit(real(), "vvriig"), "үүрийг");
}

#[test]
fn all_capitals_stay() {
    assert_eq!(commit(real(), "HOTAAS"), "ХОТААС");
}

#[test]
fn punctuation_runs_without_undo() {
    let mut c = Composer::with_options(real());
    type_in(&mut c, "hotaas");
    let r = c.process_key(Key::Symbol('.'));
    assert_eq!(r.commit.as_deref(), Some("хотоос"));
    let r = c.process_key(Key::Backspace);
    assert!(!r.consumed);
    assert_eq!(c.preedit(), "");
}

#[test]
fn enter_does_not_choose() {
    assert_eq!(commit_with(real(), "hotaas", Key::Other), "хотаас");
}

#[test]
fn latin_view_skips() {
    let mut c = Composer::with_options(real());
    type_in(&mut c, "hotaas");
    c.process_key(Key::Escape);
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("hotaas"));
}

#[test]
fn no_used_event_for_a_changed_word() {
    let mut c = Composer::with_options(real());
    type_in(&mut c, "hotaas");
    let r = c.process_key(Key::Space);
    assert_eq!(r.learned, None);
}

#[test]
fn runs_without_word_choice() {
    let o = Options {
        word_choice: false,
        ..real()
    };
    assert_eq!(commit(o, "hotaas"), "хотоос");
}

#[test]
fn word_choice_wins_first() {
    assert_eq!(commit(real(), "tov"), "төв");
}

#[test]
fn a_user_added_word_stays() {
    use buuz_core::UserDict;
    let mut d = UserDict::new();
    assert!(d.add("хотаас"));
    let o = Options {
        user: Some(Arc::new(d.snapshot())),
        ..real()
    };
    assert_eq!(commit(o, "hotaas"), "хотаас");
}

#[test]
fn words_over_16_letters_stay() {
    // 18 letters: хотхотхотхотхотоос is listed, but the word is too long.
    let list = "хотхотхотхотхот\t500\nхотхотхотхотхотоос\t500\n";
    assert_eq!(
        commit(small(list), "hotxotxotxotxotaas"),
        "хотхотхотхотхотаас"
    );
}

// Step 2: hidden н and g insertion, with small lists.

#[test]
fn hidden_n_from_genitive() {
    let list = "ус\t3038\nусны\t3131\n";
    assert_eq!(commit(small(list), "usaas"), "уснаас");
}

#[test]
fn g_insertion_from_genitive() {
    let list = "баа\t500\nбаагийн\t200\n";
    assert_eq!(commit(small(list), "baanaas"), "баагаас");
}

#[test]
fn genitive_target_from_the_ablative() {
    let list = "ус\t3038\nуснаас\t200\n";
    assert_eq!(commit(small(list), "usiin"), "усны");
    let list = "баа\t500\nбаагаас\t200\n";
    assert_eq!(commit(small(list), "baanii"), "баагийн");
}

#[test]
fn accusative_gets_no_hidden_n() {
    let list = "ус\t3038\nусны\t3131\n";
    assert_eq!(commit(small(list), "usiig"), "усыг");
}

#[test]
fn conflicting_stem_types_stay() {
    let list = "баа\t500\nбаагаас\t200\nбаанаас\t150\n";
    assert_eq!(commit(small(list), "baanii"), "бааны");
}

#[test]
fn own_final_g_is_not_evidence_for_the_short_stem() {
    // G3: баагийн (4287) is бааг + ийн, not баа + гийн. бааг is listed, so
    // the г evidence does not count for the stem баа.
    let list = "баа\t500\nбааг\t900\nбаагийн\t4287\n";
    assert_eq!(commit(small(list), "baanaas"), "баанаас");
    // Without the listed бааг, the same evidence inserts г.
    let list = "баа\t500\nбаагийн\t4287\n";
    assert_eq!(commit(small(list), "baanaas"), "баагаас");
}

#[test]
fn own_final_n_is_not_evidence_for_the_short_stem() {
    // G3 for н: ус is not a stem of уснаас when усн is a listed word.
    let list = "ус\t3038\nусн\t900\nуснаас\t10\nуснын\t4000\n";
    assert_eq!(commit(small(list), "usaas"), "усаас");
}

#[test]
fn listed_words_with_a_stem_letter_stay() {
    assert_eq!(commit(real(), "bagiig"), "багийг");
    assert_eq!(commit(real(), "tsegiin"), "цэгийн");
}

#[test]
fn step_2_needs_a_listed_stem() {
    // оюутны is not оюут + ны: оюут is not a word.
    let list = "оюутны\t300\n";
    assert_eq!(commit(small(list), "oyuutaas"), "оюутаас");
    // With the stem listed, the evidence counts.
    let list = "оюут\t900\nоюутны\t300\n";
    assert_eq!(commit(small(list), "oyuutaas"), "оюутнаас");
}

#[test]
fn hidden_n_of_one_meaning_stays() {
    // цагны and цагийн are both listed. The typed word is not 50 times
    // smaller than the other form, so it stays.
    assert_eq!(commit(real(), "tsagnii"), "цагны");
}

#[test]
fn explicit_y_blocks_suffix_choice() {
    // The list has сургуулийн 100 and not сургуулын. `y`, `iii` and `ii'`
    // force the letter, so the typed form stays.
    let list = "сургуулийн\t100\n";
    assert_eq!(commit(small(list), "surguuliin"), "сургуулийн");
    assert_eq!(commit(small(list), "surguulyn"), "сургуулын");
    assert_eq!(commit(small(list), "surguuliiin"), "сургуулын");
    let list = "аавын\t100\n";
    assert_eq!(commit(small(list), "aavii'n"), "аавийн");
    // Without a forcing key the list still decides.
    assert_eq!(commit(small("хотоос\t500\n"), "hotaas"), "хотоос");
}

#[test]
fn own_final_letter_is_not_a_suffix() {
    // The stem ends in its own г or н (not a long vowel), so no г or н is
    // inserted. The word stays, or takes the right stem: the one-letter and г forms need a stem that ends in a long
    // vowel (or vowel + й).
    for (typed, want) in [
        ("zarlagiin", "зарлагын"),
        // байгуулагын has only 21, below the floor of 50: the word stays.
        ("baiguulagiin", "байгуулагийн"),
        // валентины has only 27, below the floor of 50: the word stays.
        ("valyentinii", "валентиний"),
        // концерны has only 12: the word stays.
        ("koncyernii", "концерний"),
        ("hagiin", "хагийн"),
        // аягыг has only 13: the word stays, and is not turned into аяыг.
        ("ayagiig", "аягийг"),
        ("palyestinii", "палестины"),
    ] {
        assert_eq!(commit(real(), typed), want, "{typed}");
    }
}

#[test]
fn a_typed_hidden_n_is_not_removed() {
    // сарнаас 11 and эдний 37 are listed: the н stays.
    assert_eq!(commit(real(), "sarnaas"), "сарнаас");
    assert_eq!(commit(real(), "ednii"), "эдний");
    // A typed plain form still changes to the н form.
    assert_eq!(commit(real(), "usiin"), "усны");
    assert_eq!(commit(real(), "heliin"), "хэлний");
}

#[test]
fn step_2_genitive_takes_ny_or_nii_from_the_stem_vowel() {
    let list = "тоо\t500\nтооноос\t200\n";
    assert_eq!(commit(small(list), "toogiin"), "тооны");
    let list = "баа\t500\nбаанаас\t200\n";
    assert_eq!(commit(small(list), "baagiin"), "бааны");
    let list = "хүү\t500\nхүүнээс\t200\n";
    assert_eq!(commit(small(list), "hwwgiin"), "хүүний");
}

#[test]
fn a_forcing_key_at_the_edge_of_the_tail_blocks() {
    // `y` is the 5th key from the end.
    let list = "баынтой\t500\n";
    assert_eq!(commit(small(list), "bayntai"), "баынтай");
    // The same word without `y` changes.
    let list = "баантой\t500\n";
    assert_eq!(commit(small(list), "baantai"), "баантой");
}

#[test]
fn a_y_in_the_stem_only_blocks() {
    // Blocked, not changed: тоянаас (300) would win otherwise.
    let list = "тоя\t500\nтоянаас\t300\n";
    assert_eq!(commit(small(list), "toyaaas"), "тояаас");
    // Without the `y` the same suffix changes.
    let list = "тоа\t500\nтоанаас\t300\n";
    assert_eq!(commit(small(list), "toaaas"), "тоанаас");
}

#[test]
fn evidence_below_50_stays() {
    let list = "ус\t3038\nусны\t40\n";
    assert_eq!(commit(small(list), "usaas"), "усаас");
}

#[test]
fn step_2_skips_a_listed_word() {
    let list = "ус\t3038\nусны\t3131\nусаас\t5\n";
    assert_eq!(commit(small(list), "usaas"), "усаас");
}

#[test]
fn step_2_skips_when_a_candidate_is_listed() {
    // уснаас 20 is listed but too small to win. Step 2 does not run.
    let list = "ус\t3038\nусны\t3131\nуснаас\t20\n";
    assert_eq!(commit(small(list), "usaas"), "усаас");
}

#[test]
fn comitative_has_no_step_2() {
    let list = "ус\t3038\nусны\t3131\n";
    assert_eq!(commit(small(list), "ustai"), "устай");
}

// Config.

#[test]
fn config_key_sets_the_option() {
    let text = "buuz_config = 1\n[options]\nsuffix_choice = false\n";
    let parsed = Config::parse(text).unwrap();
    assert_eq!(parsed.config.suffix_choice(), Some(false));
    assert!(!parsed.config.options().suffix_choice);
    assert!(Config::default().options().suffix_choice);
    let classic =
        Config::parse("buuz_config = 1\npreset = \"classic\"\n[options]\nsuffix_choice = true\n")
            .unwrap();
    assert!(classic.config.options().suffix_choice);
}

#[test]
fn config_round_trip_and_duplicate_key() {
    let mut c = Config::default();
    c.set_suffix_choice(Some(true));
    let text = c.to_text();
    assert!(text.contains("suffix_choice = true"));
    assert_eq!(Config::parse(&text).unwrap().config, c);
    let dup = "buuz_config = 1\n[options]\nsuffix_choice = true\nsuffix_choice = false\n";
    assert!(Config::parse(dup).is_err());
}

#[test]
fn suffix_choice_needs_the_word_list() {
    let mut c =
        Config::parse("buuz_config = 1\n[options]\ncompletion = false\nword_choice = false\nrounding_harmony = false\n")
            .unwrap()
            .config;
    assert!(
        c.needs_word_list(),
        "suffix choice is on in the smart preset"
    );
    c.set_suffix_choice(Some(false));
    assert!(!c.needs_word_list());
    c.set_suffix_choice(Some(true));
    assert!(c.needs_word_list());
}

// Speed.

#[test]
fn suffix_choice_is_fast_per_word_on_the_real_list() {
    let mut c = Composer::with_options(real());
    let words = [
        "hotaas",
        "usaas",
        "toogiin",
        "shireegees",
        "ajiliin",
        "uuriin",
        "nomaas",
        "bayrlalaa",
        "oeoeoeoeoeoeoeoe",
        "aaaaaaaaaaaaaaaaaaaa",
        "aanaanaanaanaanaas",
        "qwqwqwqwqwqwqwqw",
    ];
    let mut samples = Vec::new();
    for round in 0..11 {
        for w in words {
            type_in(&mut c, w);
            let start = Instant::now();
            let r = c.process_key(Key::Space);
            let took = start.elapsed().as_nanos();
            std::hint::black_box(r);
            if round > 0 {
                samples.push(took);
            }
        }
    }
    samples.sort_unstable();
    let p99 = samples[(samples.len() * 99 / 100).min(samples.len() - 1)];
    eprintln!("suffix choice: p99 {p99} ns");
    if !cfg!(debug_assertions) {
        assert!(p99 < 1_000_000, "p99 {p99} ns");
    }
}
