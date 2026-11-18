// SPDX-License-Identifier: Apache-2.0

//! Issue #46, stage 1: the key pair `vv` gives үү in smart mode
//! (`Options::double_v`). A single `v` stays в.

use std::sync::Arc;

use buuz_core::{convert_with, Composer, Config, KeyMap, Options};

fn on(input: &str) -> String {
    convert_with(input, &Options::smart())
}

fn off(input: &str) -> String {
    let o = Options {
        double_v: false,
        ..Options::smart()
    };
    convert_with(input, &o)
}

#[test]
fn option_defaults() {
    assert!(Options::smart().double_v);
    assert!(Options::default().double_v);
    assert!(!Options::classic().double_v);
}

#[test]
fn vv_gives_uu() {
    assert_eq!(on("svv"), "сүү");
    assert_eq!(on("vv"), "үү");
    assert_eq!(on("tvvhen"), "түүхэн");
}

#[test]
fn single_v_stays_v() {
    assert_eq!(on("avah"), "авах");
    assert_eq!(on("hev"), "хэв");
    assert_eq!(on("ovog"), "овог");
    assert_eq!(on("v"), "в");
}

#[test]
fn upper_and_title_case() {
    assert_eq!(on("VV"), "ҮҮ");
    assert_eq!(on("Vv"), "Үү");
    assert_eq!(on("SVV"), "СҮҮ");
    assert_eq!(on("Svv"), "Сүү");
}

#[test]
fn three_v_is_uu_then_v() {
    assert_eq!(on("vvv"), "үүв");
}

#[test]
fn vv_is_feminine_for_the_look_ahead() {
    // uvv: vv decides the word, so the u before it is ү.
    assert_eq!(on("uvv"), "үүү");
    assert_eq!(on("uvv"), on("uww"));
    // Plain u and o after vv follow the feminine word.
    assert_eq!(on("hvvhduud"), "хүүхдүүд");
    assert_eq!(on("hvvhduud"), on("hwwhduud"));
    assert_eq!(on("hvvhdo"), on("hwwhdo"));
}

#[test]
fn vv_gives_the_particle_the_feminine_form() {
    assert_eq!(on("svv uu"), "сүү үү");
}

#[test]
fn gvv_follows_the_gww_look_ahead() {
    for (v, w) in [("ogvv", "ogww"), ("gvvo", "gwwo"), ("agvv", "agww")] {
        assert_eq!(on(v), on(w), "{v} against {w}");
    }
    // Like `gww`, the second ү is evidence: гүү is a feminine word.
    assert_eq!(on("ogvv"), "өгүү");
    assert_eq!(on("gvv"), "гүү");
    // A single explicit ү after г is still the suffix -гүй.
    assert_eq!(on("agw"), "агү");
}

#[test]
fn option_off_keeps_two_v() {
    assert_eq!(off("svv"), "свв");
    assert_eq!(off("VV"), "ВВ");
    assert_eq!(off("avah"), "авах");
}

#[test]
fn classic_is_unchanged() {
    let c = Options::classic();
    assert_eq!(convert_with("svv", &c), "свв");
    assert_eq!(convert_with("VV", &c), "ВВ");
    // Turning the option on in classic mode works on its own.
    let o = Options {
        double_v: true,
        ..Options::classic()
    };
    assert_eq!(convert_with("svv", &o), "сүү");
}

#[test]
fn smart_vowels_off_still_gives_uu() {
    let o = Options {
        smart_vowels: false,
        ..Options::smart()
    };
    assert_eq!(convert_with("svv", &o), "сүү");
}

#[test]
fn composer_updates_on_each_key() {
    let mut c = Composer::with_options(Options::smart());
    for ch in "sv".chars() {
        c.process_key(buuz_core::Key::Char(ch));
    }
    assert_eq!(c.preedit(), "св");
    c.process_key(buuz_core::Key::Char('v'));
    assert_eq!(c.preedit(), "сүү");
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
fn custom_keys_work_with_vv() {
    // Another key is untouched.
    assert_eq!(convert_with("svv", &keyed(&[("x", "ш")])), "сүү");
    // A user mapping of the pair itself wins.
    assert_eq!(convert_with("svv", &keyed(&[("vv", "в")])), "св");
    // The option off also turns the pair off with a key map.
    let mut o = keyed(&[("x", "ш")]);
    o.double_v = false;
    assert_eq!(convert_with("svv", &o), "свв");
}

#[test]
fn config_option_and_round_trip() {
    let c = Config::parse("buuz_config = 1\n[options]\ndouble_v = false\n")
        .unwrap()
        .config;
    assert_eq!(c.double_v(), Some(false));
    assert!(!c.options().double_v);
    assert_eq!(Config::parse(&c.to_text()).unwrap().config, c);
    assert!(c.to_text().contains("double_v = false"));

    let c = Config::parse("buuz_config = 1\npreset = \"classic\"\n[options]\ndouble_v = true\n")
        .unwrap()
        .config;
    assert!(c.options().double_v);
    assert!(Config::default().options().double_v);
    let classic = Config::parse("buuz_config = 1\npreset = \"classic\"\n")
        .unwrap()
        .config;
    assert!(!classic.options().double_v);

    let mut c = Config::default();
    c.set_double_v(Some(false));
    assert_eq!(c.double_v(), Some(false));
    assert_eq!(Config::parse(&c.to_text()).unwrap().config, c);
}

#[test]
fn config_duplicate_is_an_error() {
    assert!(
        Config::parse("buuz_config = 1\n[options]\ndouble_v = true\ndouble_v = false\n").is_err()
    );
}

#[test]
fn target_uu_is_still_unknown() {
    let pairs = vec![("x".to_string(), "үү".to_string())];
    assert!(matches!(
        KeyMap::from_pairs(&pairs),
        Err(buuz_core::KeyError::UnknownTarget { .. })
    ));
}

#[test]
fn calibration_treats_vv_as_a_default() {
    let c = buuz_core::calibrate(&[
        ("сүү", "svv"),
        ("түүх", "tvvh"),
        ("хүүхэд", "hvvhed"),
        ("үүд", "vvd"),
        ("авах", "avah"),
    ]);
    assert!(c.conflicts.is_empty(), "{:?}", c.conflicts);
    assert!(
        c.suggestions.iter().all(|s| s.chunk != "v"),
        "{:?}",
        c.suggestions
    );
}

#[test]
fn export_hides_the_double_v_rule() {
    assert!(buuz_core::export_rules()
        .iter()
        .all(|(f, _, _, _)| !f.eq_ignore_ascii_case("vv") && !f.eq_ignore_ascii_case("vvi")));
}

#[test]
fn vvi_is_uui() {
    assert_eq!(on("vvi"), "үүй");
    assert_eq!(on("bolohgvvi"), "бөлөхгүүй");
    assert_eq!(off("vvi"), "вви");
}

#[test]
fn more_pairs_and_backspace() {
    assert_eq!(on("vV"), "үҮ");
    assert_eq!(on("vvvv"), "үүүү");
    let mut c = Composer::with_options(Options::smart());
    for ch in "svv".chars() {
        c.process_key(buuz_core::Key::Char(ch));
    }
    c.process_key(buuz_core::Key::Backspace);
    assert_eq!(c.preedit(), "св");
}

#[test]
fn gwi_suffix_and_single_v() {
    assert_eq!(on("bolohgwi"), "болохгүй");
    assert_eq!(on("bolohgvi"), "болохгви");
}
