// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. Cases come from issue #8 (portable config format).

use buuz_core::{
    convert_with, AbbrevError, Config, ConfigError, ConfigWarning, Options, Parsed, Preset,
};

fn parsed(text: &str) -> Parsed {
    Config::parse(text).unwrap_or_else(|e| panic!("{text:?} failed: {e}"))
}

fn parse(text: &str) -> Config {
    parsed(text).config
}

fn err(text: &str) -> ConfigError {
    Config::parse(text).expect_err(text)
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn same_options(a: &Options, b: &Options) {
    assert_eq!(a.smart_vowels, b.smart_vowels);
    assert_eq!(a.keep_latin, b.keep_latin);
    assert_eq!(a.auto_latin, b.auto_latin);
    assert_eq!(
        a.abbreviations.as_ref().map(|t| t.to_text()),
        b.abbreviations.as_ref().map(|t| t.to_text())
    );
}

// ---- defaults and presets ----

#[test]
fn default_is_smart_without_overrides() {
    let c = Config::default();
    assert_eq!(c.preset(), Preset::Smart);
    assert_eq!(c.smart_vowels(), None);
    assert_eq!(c.abbreviations(), None);
    assert_eq!(c.keep_latin(), None);
    assert_eq!(c.auto_latin(), None);
    assert!(c.abbreviation_entries().is_empty());
    assert!(c.keys().is_empty());
}

#[test]
fn bare_file_is_the_default_config() {
    assert_eq!(parse("buuz_config = 1\n"), Config::default());
}

#[test]
fn bare_smart_matches_options_smart() {
    same_options(&Config::default().options(), &Options::smart());
    same_options(
        &parse("buuz_config = 1\npreset = \"smart\"").options(),
        &Options::smart(),
    );
}

#[test]
fn bare_classic_matches_options_classic() {
    let c = parse("buuz_config = 1\npreset = \"classic\"\n");
    assert_eq!(c.preset(), Preset::Classic);
    same_options(&c.options(), &Options::classic());
}

#[test]
fn classic_output_is_unchanged() {
    let o = parse("buuz_config = 1\npreset = \"classic\"").options();
    assert_eq!(
        convert_with("unee bn", &o),
        convert_with("unee bn", &Options::classic())
    );
}

// ---- overrides ----

#[test]
fn overrides_apply_on_smart() {
    let c = parse(
        "buuz_config = 1\n[options]\nsmart_vowels = false\nabbreviations = false\n\
         keep_latin = false\nauto_latin = false\n",
    );
    same_options(&c.options(), &Options::classic());
}

#[test]
fn overrides_apply_on_classic() {
    let c = parse(
        "buuz_config = 1\npreset = \"classic\"\n[options]\nsmart_vowels = true\nkeep_latin = true\n",
    );
    let o = c.options();
    assert!(o.smart_vowels && o.keep_latin);
    assert!(!o.auto_latin);
    assert!(o.abbreviations.is_none());
}

#[test]
fn abbreviations_can_be_enabled_on_classic() {
    let c = parse("buuz_config = 1\npreset = \"classic\"\n[options]\nabbreviations = true\n");
    assert_eq!(convert_with("bn ", &c.options()), "байна ");
}

// ---- user abbreviations ----

#[test]
fn user_abbreviation_is_added() {
    let c = parse("buuz_config = 1\n[abbreviations]\nmgl = \"Монгол\"\n");
    assert_eq!(c.abbreviation_entries(), pairs(&[("mgl", "Монгол")]));
    let o = c.options();
    assert_eq!(convert_with("mgl bn ", &o), "Монгол байна ");
}

#[test]
fn user_abbreviation_replaces_builtin() {
    let c = parse("buuz_config = 1\n[abbreviations]\nbn = \"байна аа\"\n");
    assert_eq!(convert_with("bn ", &c.options()), "байна аа ");
}

#[test]
fn empty_value_removes_builtin() {
    let c = parse("buuz_config = 1\n[abbreviations]\nbn = \"\"\n");
    let o = c.options();
    assert_eq!(convert_with("bn ", &o), "бн ");
    assert_eq!(convert_with("sn ", &o), "сайн ");
}

#[test]
fn abbreviations_false_gives_none() {
    let c = parse(
        "buuz_config = 1\n[options]\nabbreviations = false\n[abbreviations]\nmgl = \"Монгол\"\n",
    );
    assert!(c.options().abbreviations.is_none());
}

#[test]
fn abbreviation_keys_are_lowercased() {
    let c = parse("buuz_config = 1\n[abbreviations]\nMGL = \"Монгол\"\n");
    assert_eq!(c.abbreviation_entries()[0].0, "mgl");
}

#[test]
fn abbreviation_order_is_kept() {
    let c = parse("buuz_config = 1\n[abbreviations]\nzz = \"а\"\naa = \"б\"\n");
    assert_eq!(c.abbreviation_entries(), pairs(&[("zz", "а"), ("aa", "б")]));
}

#[test]
fn bad_abbreviation_key_has_line() {
    let e = err("buuz_config = 1\n\n[abbreviations]\n\"b1\" = \"x\"\n");
    assert!(matches!(
        e,
        ConfigError::Abbreviation(AbbrevError::BadKey { line: 4, .. })
    ));
    assert!(e.to_string().contains("line 4"));
}

#[test]
fn bad_abbreviation_value_has_line() {
    let e = err("buuz_config = 1\n[abbreviations]\nbn = \"a\\nb\"\n");
    assert!(matches!(
        e,
        ConfigError::Abbreviation(AbbrevError::BadValue { line: 3, .. })
    ));
}

// ---- keys ----

#[test]
fn keys_are_validated_and_stored() {
    let c = parse("buuz_config = 1\n[keys]\n\"o'\" = \"ө\"\nu = \"ү\"\n");
    assert_eq!(c.keys(), pairs(&[("o'", "ө"), ("u", "ү")]));
    // Stored, not applied (issue #3 applies them).
    same_options(&c.options(), &Options::smart());
}

#[test]
fn keys_are_case_insensitive_and_stored_lowercase() {
    let c = parse("buuz_config = 1\n[keys]\nO = \"о\"\n\"Ö\" = \"ө\"\n");
    assert_eq!(c.keys(), pairs(&[("o", "о"), ("ö", "ө")]));
}

#[test]
fn bad_key_sequence_is_refused() {
    assert!(matches!(
        err("buuz_config = 1\n[keys]\n\"1\" = \"ө\"\n"),
        ConfigError::BadKeys { line: 3, .. }
    ));
    assert!(matches!(
        err("buuz_config = 1\n[keys]\nabcd = \"ө\"\n"),
        ConfigError::BadKeys { line: 3, .. }
    ));
}

#[test]
fn non_cyrillic_key_value_is_refused() {
    assert!(matches!(
        err("buuz_config = 1\n[keys]\nq = \"x\"\n"),
        ConfigError::BadKeys { line: 3, .. }
    ));
    // An empty value removes a key, so it is allowed.
    assert!(!is_err("buuz_config = 1\n[keys]\nq = \"\"\n"));
}

// ---- syntax: comments, blanks, quoting ----

#[test]
fn comments_and_blank_lines_are_skipped() {
    let c = parse(
        "# my settings\n\n  # indented comment\nbuuz_config = 1 # version\n\n\
         preset = \"classic\" # base\n[options] # section\nkeep_latin = true # on\n",
    );
    assert_eq!(c.preset(), Preset::Classic);
    assert_eq!(c.keep_latin(), Some(true));
}

#[test]
fn hash_inside_string_is_not_a_comment() {
    let c = parse("buuz_config = 1\n[abbreviations]\nhh = \"а # б\"\n");
    assert_eq!(c.abbreviation_entries(), pairs(&[("hh", "а # б")]));
}

#[test]
fn spaces_around_equals_are_optional() {
    let c = parse("buuz_config=1\npreset=\"classic\"\n");
    assert_eq!(c.preset(), Preset::Classic);
}

#[test]
fn crlf_line_endings_work() {
    let c = parse("buuz_config = 1\r\npreset = \"classic\"\r\n");
    assert_eq!(c.preset(), Preset::Classic);
}

#[test]
fn quoted_keys_work() {
    let c = parse("buuz_config = 1\n[abbreviations]\n\"o'\" = \"ө\"\n\"a\\\"b\" = \"x\"\n");
    assert_eq!(
        c.abbreviation_entries(),
        pairs(&[("o'", "ө"), ("a\"b", "x")])
    );
}

#[test]
fn string_escapes_work() {
    let c = parse("buuz_config = 1\n[abbreviations]\nq = \"a\\\"b\\\\c\"\n");
    assert_eq!(c.abbreviation_entries(), pairs(&[("q", "a\"b\\c")]));
}

// ---- round trips ----

#[test]
fn default_round_trips() {
    let c = Config::default();
    assert_eq!(parse(&c.to_text()), c);
}

#[test]
fn full_config_round_trips() {
    let mut c = Config::default();
    c.set_preset(Preset::Classic);
    c.set_smart_vowels(Some(true));
    c.set_abbreviations(Some(true));
    c.set_keep_latin(Some(false));
    c.set_auto_latin(Some(true));
    for (k, v) in [
        ("mgl", "Монгол"),
        ("o'", "ө"),
        ("bn", ""),
        ("q\"", "a\"b\\c"),
    ] {
        c.set_abbreviation(k, v).unwrap();
    }
    c.set_key("o'", "ө").unwrap();
    c.set_key("u\"", "ү").unwrap();
    let text = c.to_text();
    assert_eq!(parse(&text), c);
    assert_eq!(parse(&text).to_text(), text);
}

#[test]
fn to_text_is_canonical() {
    let messy = "# hi\nbuuz_config=1\n\n\npreset = \"smart\"  # x\n[options]\nauto_latin=false\nsmart_vowels = true\n";
    let once = parse(messy).to_text();
    assert_eq!(parse(&once).to_text(), once);
    assert_eq!(
        once,
        "buuz_config = 1\npreset = \"smart\"\n\n[options]\nsmart_vowels = true\nauto_latin = false\n"
    );
}

#[test]
fn default_text_is_short() {
    assert_eq!(
        Config::default().to_text(),
        "buuz_config = 1\npreset = \"smart\"\n"
    );
}

// ---- errors with line numbers ----

#[test]
fn missing_version() {
    assert!(matches!(err(""), ConfigError::MissingVersion { line: 1 }));
    assert!(matches!(
        err("# c\n\npreset = \"smart\"\n"),
        ConfigError::MissingVersion { line: 3 }
    ));
    assert!(matches!(
        err("[options]\n"),
        ConfigError::MissingVersion { line: 1 }
    ));
}

#[test]
fn version_too_new() {
    let e = err("buuz_config = 2\n");
    assert_eq!(
        e,
        ConfigError::TooNew {
            found: 2,
            supported: 1
        }
    );
    assert!(e.to_string().contains('2'));
    // A newer file is refused even when the rest is unknown to us.
    assert!(matches!(
        err("buuz_config = 7\n[future]\nx = [1, 2]\n"),
        ConfigError::TooNew { found: 7, .. }
    ));
}

#[test]
fn bad_version_value() {
    assert!(matches!(
        err("buuz_config = \"1\"\n"),
        ConfigError::BadValue { line: 1, .. }
    ));
    assert!(matches!(
        err("buuz_config = 0\n"),
        ConfigError::BadValue { line: 1, .. }
    ));
}

#[test]
fn unknown_section_is_a_warning_and_is_skipped() {
    let p = parsed(
        "buuz_config = 1\n\n[colours]\nfoo = [1, 2\nthis is not even toml\n[options]\nkeep_latin = false\n",
    );
    assert_eq!(p.config.keep_latin(), Some(false));
    assert_eq!(
        p.warnings,
        vec![ConfigWarning::UnknownSection {
            line: 3,
            name: "colours".into()
        }]
    );
    assert_eq!(
        p.warnings[0].to_string(),
        "line 3: unknown section [colours], ignored"
    );
}

#[test]
fn unknown_option_is_a_warning() {
    let p = parsed(
        "buuz_config = 1\n[options]\nkeep_latin = false\nsmart_vowel = [1]\nauto_latin = false\n",
    );
    assert_eq!(p.config.keep_latin(), Some(false));
    assert_eq!(p.config.auto_latin(), Some(false));
    assert_eq!(
        p.warnings,
        vec![ConfigWarning::UnknownOption {
            line: 4,
            key: "smart_vowel".into()
        }]
    );
    assert_eq!(
        p.warnings[0].to_string(),
        "line 4: unknown option `smart_vowel`, ignored"
    );
}

#[test]
fn clean_file_has_no_warnings() {
    assert!(parsed("buuz_config = 1\n[options]\nkeep_latin = true\n")
        .warnings
        .is_empty());
}

#[test]
fn unknown_top_level_key_is_an_error() {
    assert!(matches!(
        err("buuz_config = 1\nmode = \"smart\"\n"),
        ConfigError::UnknownKey { line: 2, .. }
    ));
}

#[test]
fn bad_bool() {
    assert!(matches!(
        err("buuz_config = 1\n[options]\nkeep_latin = yes\n"),
        ConfigError::BadValue { line: 3, .. }
    ));
    assert!(matches!(
        err("buuz_config = 1\n[options]\nkeep_latin = \"true\"\n"),
        ConfigError::BadValue { line: 3, .. }
    ));
    assert!(matches!(
        err("buuz_config = 1\n[options]\nkeep_latin = 1\n"),
        ConfigError::BadValue { line: 3, .. }
    ));
}

#[test]
fn bad_preset() {
    assert!(matches!(
        err("buuz_config = 1\npreset = \"fancy\"\n"),
        ConfigError::BadValue { line: 2, .. }
    ));
}

#[test]
fn bad_string_escape() {
    assert!(matches!(
        err("buuz_config = 1\n[abbreviations]\nq = \"a\\tb\"\n"),
        ConfigError::BadEscape { line: 3 }
    ));
}

#[test]
fn duplicate_key() {
    assert!(matches!(
        err("buuz_config = 1\npreset = \"smart\"\npreset = \"classic\"\n"),
        ConfigError::DuplicateKey { line: 3, .. }
    ));
    // The duplicate is reported before the bad value.
    assert!(matches!(
        err("buuz_config = 1\npreset = \"smart\"\npreset = \"fancy\"\n"),
        ConfigError::DuplicateKey { line: 3, .. }
    ));
    assert!(matches!(
        err("buuz_config = 1\n[keys]\no = \"о\"\nO = \"о\"\n"),
        ConfigError::DuplicateKey { line: 4, .. }
    ));
    assert!(matches!(
        err("buuz_config = 1\n[options]\nkeep_latin = true\nkeep_latin = false\n"),
        ConfigError::DuplicateKey { line: 4, .. }
    ));
    assert!(matches!(
        err("buuz_config = 1\n[abbreviations]\nbn = \"а\"\nBN = \"б\"\n"),
        ConfigError::DuplicateKey { line: 4, .. }
    ));
}

#[test]
fn duplicate_section() {
    assert!(matches!(
        err("buuz_config = 1\n[options]\n[options]\n"),
        ConfigError::DuplicateSection { line: 3, .. }
    ));
}

#[test]
fn syntax_errors() {
    for (text, line) in [
        ("buuz_config = 1\njunk\n", 2),
        ("buuz_config = 1\npreset = \n", 2),
        ("buuz_config = 1\npreset = \"smart\n", 2),
        ("buuz_config = 1\npreset = \"smart\" x\n", 2),
        ("buuz_config = 1\n[options\n", 2),
        ("buuz_config = 1\n= 1\n", 2),
    ] {
        assert!(
            matches!(err(text), ConfigError::Syntax { line: l } if l == line),
            "{text:?}"
        );
    }
}

#[test]
fn errors_are_std_errors() {
    fn is_error<E: std::error::Error>(_: &E) {}
    is_error(&err(""));
}

// ---- valid TOML only ----

fn is_err(text: &str) -> bool {
    Config::parse(text).is_err()
}

#[test]
fn tabs_work_as_separators() {
    let c = parse("buuz_config\t=\t1\t# c\n\t[options]\t\n\tkeep_latin\t=\ttrue\n");
    assert_eq!(c.keep_latin(), Some(true));
    let c = parse("buuz_config = 1\n[abbreviations]\nq = \"\ta\t\"\n");
    assert_eq!(c.abbreviation_entries(), pairs(&[("q", "a")]));
}

#[test]
fn unicode_whitespace_is_refused() {
    assert!(is_err("buuz_config\u{a0}= 1\n"));
    assert!(is_err("buuz_config = 1\u{a0}\n"));
    assert!(is_err("buuz_config = 1\n\u{3000}preset = \"smart\"\n"));
    assert!(is_err("buuz_config = 1\n[\u{a0}options]\n"));
    assert!(is_err(
        "buuz_config = 1\n[options]\nkeep_latin =\u{2003}true\n"
    ));
    assert!(is_err("buuz_config = 1\n[options]\u{a0}\n"));
}

#[test]
fn leading_zeros_and_signs_are_refused() {
    for v in ["01", "00", "+1", "-1"] {
        assert!(
            matches!(
                err(&format!("buuz_config = {v}\n")),
                ConfigError::BadValue { line: 1, .. }
            ),
            "{v}"
        );
    }
}

#[test]
fn huge_integers_are_bad_values_not_too_new() {
    assert!(matches!(
        err("buuz_config = 9223372036854775808\n"),
        ConfigError::BadValue { line: 1, .. }
    ));
    assert!(matches!(
        err("buuz_config = 99999999999999999999999\n"),
        ConfigError::BadValue { line: 1, .. }
    ));
    assert_eq!(
        err("buuz_config = 9223372036854775807\n"),
        ConfigError::TooNew {
            found: i64::MAX as u64,
            supported: 1
        }
    );
}

#[test]
fn control_characters_are_refused() {
    for c in [
        '\u{0}', '\u{1}', '\u{8}', '\u{b}', '\u{c}', '\u{d}', '\u{1f}', '\u{7f}',
    ] {
        let in_string = format!("buuz_config = 1\n[abbreviations]\nq = \"a{c}b\"\n");
        assert!(is_err(&in_string), "string {c:?}");
        let in_key = format!("buuz_config = 1\n[abbreviations]\n\"a{c}b\" = \"x\"\n");
        assert!(is_err(&in_key), "key {c:?}");
        let in_comment = format!("buuz_config = 1 # a{c}b\n");
        assert!(is_err(&in_comment), "comment {c:?}");
        let full_comment = format!("# a{c}b\nbuuz_config = 1\n");
        assert!(is_err(&full_comment), "full comment {c:?}");
    }
    // Tab is allowed.
    assert!(!is_err("buuz_config = 1 # a\tb\n"));
}

#[test]
fn refused_forms() {
    for text in [
        "buuz_config = 1\n[abbreviations]\nq = 'x'\n",
        "buuz_config = 1\n[abbreviations]\n'q' = \"x\"\n",
        "buuz_config = 1\n[[options]]\n",
        "buuz_config = 1\n[\"options\"]\n",
        "buuz_config = 1\n[options]\nkeep.latin = true\n",
        "buuz_config = 1\n[abbreviations]\nq = \"\"\"x\"\"\"\n",
        "buuz_config = 1\n[abbreviations]\nq = \"\"\"\nx\n\"\"\"\n",
        "buuz_config = 1\n[options]\nkeep_latin = [true]\n",
        "buuz_config = 1\n[options]\nkeep_latin = { a = 1 }\n",
        "buuz_config = 1.0\n",
        "buuz_config = 1_0\n",
        "buuz_config = 0x1\n",
    ] {
        assert!(is_err(text), "{text:?}");
    }
}

#[test]
fn single_quotes_give_a_clear_error() {
    let e = err("buuz_config = 1\n[abbreviations]\nq = 'x'\n");
    assert_eq!(e, ConfigError::SingleQuotes { line: 3 });
    assert!(e.to_string().contains("double quotes"));
    assert!(matches!(
        err("buuz_config = 1\n[abbreviations]\n'q' = \"x\"\n"),
        ConfigError::SingleQuotes { line: 3 }
    ));
}

#[test]
fn non_ascii_quoted_abbreviation_key() {
    let e = err("buuz_config = 1\n[abbreviations]\n\"бн\" = \"x\"\n");
    assert!(matches!(
        e,
        ConfigError::Abbreviation(AbbrevError::BadKey { line: 3, .. })
    ));
    // ö and ü are input characters.
    let c = parse("buuz_config = 1\n[abbreviations]\n\"ÖÜ\" = \"x\"\n");
    assert_eq!(c.abbreviation_entries(), pairs(&[("öü", "x")]));
}

#[test]
fn empty_quoted_key() {
    assert!(matches!(
        err("buuz_config = 1\n[abbreviations]\n\"\" = \"x\"\n"),
        ConfigError::Abbreviation(AbbrevError::BadKey { line: 3, .. })
    ));
    assert!(matches!(
        err("buuz_config = 1\n[keys]\n\"\" = \"ө\"\n"),
        ConfigError::BadKeys { line: 3, .. }
    ));
}

#[test]
fn backslash_at_end_of_line() {
    assert!(matches!(
        err("buuz_config = 1\n[abbreviations]\nq = \"a\\\n"),
        ConfigError::Syntax { line: 3 }
    ));
    assert!(matches!(
        err("buuz_config = 1\n[abbreviations]\nq = \"a\\"),
        ConfigError::Syntax { line: 3 }
    ));
}

#[test]
fn crlf_error_line_number() {
    assert!(matches!(
        err("buuz_config = 1\r\n[options]\r\nbad\r\n"),
        ConfigError::Syntax { line: 3 }
    ));
}

#[test]
fn bom_is_ignored() {
    let c = parse("\u{feff}buuz_config = 1\npreset = \"classic\"\n");
    assert_eq!(c.preset(), Preset::Classic);
    // Only one BOM.
    assert!(is_err("\u{feff}\u{feff}buuz_config = 1\n"));
}

#[test]
fn long_values_are_refused() {
    let ok = "ө".repeat(64);
    let long = "ө".repeat(65);
    assert!(!is_err(&format!(
        "buuz_config = 1\n[abbreviations]\nq = \"{ok}\"\n"
    )));
    assert!(matches!(
        err(&format!(
            "buuz_config = 1\n[abbreviations]\nq = \"{long}\"\n"
        )),
        ConfigError::Abbreviation(AbbrevError::BadValue { line: 3, .. })
    ));
    assert!(matches!(
        err(&format!("buuz_config = 1\n[keys]\nq = \"{long}\"\n")),
        ConfigError::BadKeys { line: 3, .. }
    ));
}

#[test]
fn abbreviation_values_are_trimmed() {
    let c = parse("buuz_config = 1\n[abbreviations]\nq = \"  а б \"\n");
    assert_eq!(c.abbreviation_entries(), pairs(&[("q", "а б")]));
    assert!(is_err("buuz_config = 1\n[abbreviations]\nq = \"   \"\n"));
}

// ---- size limits ----

#[test]
fn input_over_1_mib_is_too_large() {
    let mut text = String::from("buuz_config = 1\n");
    while text.len() <= 1024 * 1024 {
        text.push_str("# padding padding padding padding padding\n");
    }
    assert!(matches!(err(&text), ConfigError::TooLarge { line: 0, .. }));
}

#[test]
fn too_many_entries_in_a_section() {
    let mut text = String::from("buuz_config = 1\n[keys]\n");
    for i in 0..10_001 {
        // 3-letter keys: 26^3 combinations are enough.
        let k: String = [i / 676, (i / 26) % 26, i % 26]
            .iter()
            .map(|&n| (b'a' + n as u8) as char)
            .collect();
        text.push_str(&format!("{k} = \"о\"\n"));
    }
    assert!(matches!(err(&text), ConfigError::TooLarge { .. }));
    // 10,000 entries are fine.
    let ok: String = text.lines().take(2 + 10_000).collect::<Vec<_>>().join("\n");
    assert_eq!(parse(&ok).keys().len(), 10_000);
}

// ---- setters ----

#[test]
fn setters_normalise_like_parse() {
    let mut c = Config::default();
    c.set_abbreviation("MGL", "  Монгол ").unwrap();
    c.set_abbreviation("bn", "").unwrap();
    c.set_key("O", "о").unwrap();
    assert_eq!(
        c.abbreviation_entries(),
        pairs(&[("mgl", "Монгол"), ("bn", "")])
    );
    assert_eq!(c.keys(), pairs(&[("o", "о")]));
    assert_eq!(parse(&c.to_text()), c);
}

#[test]
fn setters_replace_in_place_and_remove() {
    let mut c = Config::default();
    c.set_abbreviation("a", "x").unwrap();
    c.set_abbreviation("b", "y").unwrap();
    c.set_abbreviation("A", "z").unwrap();
    assert_eq!(c.abbreviation_entries(), pairs(&[("a", "z"), ("b", "y")]));
    assert!(c.remove_abbreviation("A"));
    assert!(!c.remove_abbreviation("A"));
    assert_eq!(c.abbreviation_entries(), pairs(&[("b", "y")]));
    c.set_key("o'", "ө").unwrap();
    c.set_key("O'", "о").unwrap();
    assert_eq!(c.keys(), pairs(&[("o'", "о")]));
    assert!(c.remove_key("O'"));
    assert!(c.keys().is_empty());
}

#[test]
fn setters_refuse_what_parse_refuses() {
    let mut c = Config::default();
    assert!(c.set_abbreviation("b1", "x").is_err());
    assert!(c.set_abbreviation("", "x").is_err());
    assert!(c.set_abbreviation("q", "a\nb").is_err());
    assert!(c.set_abbreviation("q", "a\u{1}b").is_err());
    assert!(c.set_abbreviation("q", "   ").is_err());
    assert!(c.set_abbreviation("q", &"x".repeat(65)).is_err());
    assert!(c.set_key("1", "ө").is_err());
    assert!(c.set_key("abcd", "ө").is_err());
    assert!(c.set_key("q", "x").is_err());
    assert!(c.set_key("q", "жжж").is_err());
    assert!(c.abbreviation_entries().is_empty() && c.keys().is_empty());
}

#[test]
fn option_setters_work() {
    let mut c = Config::default();
    c.set_smart_vowels(Some(false));
    c.set_keep_latin(Some(false));
    c.set_keep_latin(None);
    assert_eq!(c.smart_vowels(), Some(false));
    assert_eq!(c.keep_latin(), None);
    assert!(!c.options().smart_vowels);
}

// ---- the documented example ----

#[test]
fn adr_example_parses() {
    let p = parsed(include_str!("data/example.buuz"));
    assert!(p.warnings.is_empty());
    let c = p.config;
    assert_eq!(c.preset(), Preset::Smart);
    assert_eq!(c.auto_latin(), Some(true));
    assert_eq!(c.double_v(), Some(true));
    assert_eq!(
        c.abbreviation_entries(),
        pairs(&[("mgl", "Монгол"), ("bn", "байна аа"), ("tgd", "")])
    );
    assert_eq!(c.keys(), pairs(&[("x", "ш"), ("kh", "ү")]));
    let o = c.options();
    assert_eq!(convert_with("mgl bn tgd ", &o), "Монгол байна аа тгд ");
    assert_eq!(convert_with("xaan khnee ", &o), "шаан үнээ ");
    assert_eq!(parse(&c.to_text()), c);
}

// ---- deterministic fuzz: no panic, round trip holds ----

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize
    }
}

#[test]
fn generated_inputs_never_panic_and_round_trip() {
    const PIECES: &[&str] = &[
        "buuz_config = 1\n",
        "buuz_config = 2\n",
        "preset = \"smart\"\n",
        "preset = \"classic\"\n",
        "[options]\n",
        "[abbreviations]\n",
        "[keys]\n",
        "[future]\n",
        "keep_latin = true\n",
        "auto_latin = false\n",
        "smart_vowels = false # c\n",
        "mgl = \"Монгол\"\n",
        "\"o'\" = \"ө\"\n",
        "tgd = \"\"\n",
        "q = \" a b \"\n",
        "newkey = [1, 2]\n",
        "smart_vowels",
        "keep_latin",
        "auto_latin",
        "abbreviations",
        "bn",
        "O'",
        "\"o'\"",
        "\"",
        "'",
        "\\",
        "\\n",
        "\\\"",
        " = ",
        "=",
        "true",
        "false",
        "1",
        "01",
        "\"ө\"",
        "\"Монгол\"",
        "\"\"",
        "# c",
        "\n",
        "\r\n",
        "\t",
        " ",
        "\u{a0}",
        "\u{feff}",
        "ö",
        "[",
        "]",
        "\u{1}",
        "x",
    ];
    // The first LINES pieces are whole lines. Most cases use only these.
    const LINES: usize = 16;
    let mut rng = Lcg(0x5eed_1234_abcd_ef01);
    let mut accepted = 0;
    for case in 0..20_000 {
        let mut text = String::new();
        if rng.next() % 2 == 0 {
            text.push_str("buuz_config = 1\n");
        }
        let noisy = rng.next() % 4 == 0;
        for _ in 0..(rng.next() % 8) {
            let pool = if noisy { PIECES.len() } else { LINES };
            text.push_str(PIECES[rng.next() % pool]);
        }
        if let Ok(p) = Config::parse(&text) {
            accepted += 1;
            let c = p.config;
            let out = c.to_text();
            let back = Config::parse(&out)
                .unwrap_or_else(|e| panic!("case {case}: {text:?} wrote {out:?}: {e}"));
            assert_eq!(back.config, c, "case {case}: {text:?}");
            assert!(back.warnings.is_empty());
            assert_eq!(back.config.to_text(), out);
            let _ = c.options();
        }
    }
    // The loop must reach the accepted path often, or it tests nothing.
    assert!(accepted > 2_000, "only {accepted} accepted");
}
