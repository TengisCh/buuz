// SPDX-License-Identifier: Apache-2.0

//! Issue #63: the habit keys `ts` (ц) and `zh` (ж) in smart mode
//! (`Options::habit_keys`), with a word list guard for тс and зх.

use std::sync::Arc;

use buuz_core::{convert_with, Composer, Config, Key, KeyMap, Options, WordList};

const REAL: &str = include_str!("../../data/wordlist/mn.txt");

fn on(input: &str) -> String {
    convert_with(input, &Options::smart())
}

fn off(input: &str) -> String {
    let o = Options {
        habit_keys: false,
        ..Options::smart()
    };
    convert_with(input, &o)
}

fn listed(input: &str) -> String {
    let o = Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        ..Options::smart()
    };
    convert_with(input, &o)
}

#[test]
fn option_defaults() {
    assert!(Options::smart().habit_keys);
    assert!(Options::default().habit_keys);
    assert!(!Options::classic().habit_keys);
}

#[test]
fn ts_and_zh_without_a_list() {
    assert_eq!(on("tsagaan"), "цагаан");
    assert_eq!(on("tsereg"), "цэрэг");
    assert_eq!(on("gezh"), "гэж");
    assert_eq!(on("baizhee"), "байжээ");
    // No list: always ц.
    assert_eq!(on("hwrtsen"), "хүрцэн");
}

#[test]
fn word_list_guard_keeps_ts_in_past_tense_stems() {
    assert_eq!(listed("hwrtsen"), "хүртсэн");
    assert_eq!(listed("togtson"), "тогтсон");
    assert_eq!(listed("bagtsan"), "багтсан");
    assert_eq!(listed("bwtsen"), "бүтсэн");
    assert_eq!(listed("tatsan"), "татсан");
    assert_eq!(listed("tsagaan"), "цагаан");
    assert_eq!(listed("gezh"), "гэж");
    assert_eq!(listed("baizhee"), "байжээ");
}

#[test]
fn guard_keeps_the_case() {
    assert_eq!(listed("Hwrtsen"), "Хүртсэн");
    assert_eq!(listed("HWRTSEN"), "ХҮРТСЭН");
    assert_eq!(listed("hwrTsen"), "хүрТсэн");
}

#[test]
fn case() {
    assert_eq!(on("Tsagaan"), "Цагаан");
    assert_eq!(on("TSAGAAN"), "ЦАГААН");
    assert_eq!(on("Gezh"), "Гэж");
    assert_eq!(on("GEZH"), "ГЭЖ");
    assert_eq!(on("gezH"), "гэж");
}

#[test]
fn ts_before_h_stays_t_then_sh() {
    assert_eq!(on("tsh"), "тш");
    assert_eq!(on("tsxc"), "тщ");
    assert_eq!(on("TSH"), "ТШ");
    // Trade-off: the `s` here does not start a longer key, so `ts` wins.
    assert_eq!(on("tssh"), "цш");
}

#[test]
fn zh_before_h_is_still_zh() {
    assert_eq!(on("dorzhhand"), "доржханд");
    assert_eq!(listed("dorzhhand"), "доржханд");
}

#[test]
fn ii_after_a_habit_key_uses_the_stem_with_ts() {
    assert_eq!(listed("batsuuriin"), "батсуурийн");
    assert_eq!(listed("uuliin"), "уулын");
    assert_eq!(listed("surguuliin"), "сургуулийн");
    assert_eq!(listed("tsagaan"), "цагаан");
}

fn fixture(words: &[&str]) -> Options {
    let text: String = words.iter().map(|w| format!("{w}\t10\n")).collect();
    Options {
        words: Some(Arc::new(WordList::parse(&text).unwrap())),
        ..Options::smart()
    }
}

#[test]
fn two_habit_keys_the_list_picks_which_become_ts() {
    // Only the second key gives тс.
    assert_eq!(convert_with("tsatsan", &fixture(&["цатсан"])), "цатсан");
    // Only the first key gives тс.
    assert_eq!(convert_with("tsatsan", &fixture(&["тсацан"])), "тсацан");
    // Both give тс.
    assert_eq!(convert_with("tsatsan", &fixture(&["тсатсан"])), "тсатсан");
    // The text is a prefix as it stands: nothing changes.
    assert_eq!(
        convert_with("tsatsan", &fixture(&["цацан", "тсатсан"])),
        "цацан"
    );
    // No variant is a prefix: ц stays.
    assert_eq!(convert_with("tsatsan", &fixture(&["хол"])), "цацан");
    // Same for zh.
    assert_eq!(convert_with("zhazhan", &fixture(&["зхазхан"])), "зхазхан");
    assert_eq!(convert_with("zhazhan", &fixture(&["жазхан"])), "жазхан");
}

#[test]
fn last_word_can_keep_ts_as_ц() {
    // `дарц` starts other words, so the whole text `дарц` stays.
    assert_eq!(
        convert_with("darts", &fixture(&["дарц", "дарцаг", "дартс"])),
        "дарц"
    );
}

#[test]
fn off_gives_the_plain_keys() {
    assert_eq!(off("tsagaan"), "тсагаан");
    assert_eq!(off("gezh"), "гэзх");
}

#[test]
fn classic_is_unchanged() {
    let c = Options::classic();
    assert_eq!(convert_with("tsagaan", &c), "тсагаан");
    assert_eq!(convert_with("gezh", &c), "гэзх");
}

#[test]
fn harmony_matches_c_and_j() {
    for (a, b) in [("tsagaan", "cagaan"), ("tsereg", "cereg"), ("gezh", "gej")] {
        assert_eq!(on(a), on(b));
    }
    // Neutral for the particle gender like c and j.
    assert_eq!(on("tsu ts"), on("cu c"));
    assert_eq!(on("zhu"), on("ju"));
    assert_eq!(on("tsiin"), on("ciin"));
}

#[test]
fn custom_key_wins() {
    let keyed = |pairs: &[(&str, &str)]| {
        let p: Vec<(String, String)> = pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        Options {
            keys: Some(Arc::new(KeyMap::from_pairs(&p).unwrap())),
            ..Options::smart()
        }
    };
    assert_eq!(convert_with("gezh", &keyed(&[("zh", "ш")])), "гэш");
    assert_eq!(convert_with("tsa", &keyed(&[("ts", "ч")])), "ча");
    // Another key is untouched.
    assert_eq!(convert_with("tsagaan", &keyed(&[("x", "ш")])), "цагаан");
    let mut o = keyed(&[("x", "ш")]);
    o.habit_keys = false;
    assert_eq!(convert_with("tsagaan", &o), "тсагаан");
}

#[test]
fn config_option_and_round_trip() {
    let c = Config::parse("buuz_config = 1\n[options]\nhabit_keys = false\n")
        .unwrap()
        .config;
    assert_eq!(c.habit_keys(), Some(false));
    assert!(!c.options().habit_keys);
    assert_eq!(Config::parse(&c.to_text()).unwrap().config, c);
    assert!(c.to_text().contains("habit_keys = false"));

    assert!(Config::default().options().habit_keys);
    let classic = Config::parse("buuz_config = 1\npreset = \"classic\"\n")
        .unwrap()
        .config;
    assert!(!classic.options().habit_keys);
    let c = Config::parse("buuz_config = 1\npreset = \"classic\"\n[options]\nhabit_keys = true\n")
        .unwrap()
        .config;
    assert!(c.options().habit_keys);

    let mut c = Config::default();
    c.set_habit_keys(Some(false));
    assert_eq!(c.habit_keys(), Some(false));
    assert_eq!(Config::parse(&c.to_text()).unwrap().config, c);
}

#[test]
fn config_duplicate_is_an_error() {
    assert!(
        Config::parse("buuz_config = 1\n[options]\nhabit_keys = true\nhabit_keys = false\n")
            .is_err()
    );
}

#[test]
fn export_hides_the_habit_rules() {
    assert!(buuz_core::export_rules()
        .iter()
        .all(|(f, _, _, _)| !f.eq_ignore_ascii_case("ts") && !f.eq_ignore_ascii_case("zh")));
}

#[test]
fn calibration_treats_ts_and_zh_as_defaults() {
    // Typing `ts` for ц and `zh` for ж is the built-in table, not a habit.
    let c = buuz_core::calibrate(&[
        ("цагаан", "tsagaan"),
        ("цэрэг", "tsereg"),
        ("гэж", "gezh"),
        ("байжээ", "baizhee"),
        ("цай", "cai"),
    ]);
    assert!(c.conflicts.is_empty(), "{:?}", c.conflicts);
    assert!(c.suggestions.is_empty(), "{:?}", c.suggestions);
}

#[test]
fn preedit_flips_while_typing_with_the_list() {
    let o = Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        ..Options::smart()
    };
    let mut c = Composer::with_options(o);
    let mut seen = Vec::new();
    for ch in "bagtsan".chars() {
        c.process_key(Key::Char(ch));
        seen.push(c.preedit().to_string());
    }
    // багц, багца (prefix of багцад), then багтсан: багцан starts no word.
    assert_eq!(&seen[3..], ["багт", "багц", "багца", "багтсан"]);
}

#[test]
fn two_habit_keys_without_a_list() {
    assert_eq!(on("tsatsan"), "цацан");
    assert_eq!(listed("tatsan"), "татсан");
}

#[test]
fn word_list_has_prefix() {
    let l = WordList::parse("хол\t5\nхот\t4\nцай\t3\n").unwrap();
    assert!(l.has_prefix("х"));
    assert!(l.has_prefix("хол"));
    assert!(l.has_prefix("цай"));
    assert!(!l.has_prefix("холо"));
    assert!(!l.has_prefix("ч"));
    assert!(!l.has_prefix("а"));
    assert!(l.has_prefix(""));
}
