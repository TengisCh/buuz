// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. Cases come from issue #11 (on-device Mongolian word list).

use buuz_core::{WordList, WordListError};
use std::time::Instant;

const FIXTURE: &str = "\
# a comment line
байна\t100
болон\t90
бол\t80

монгол\t70
банк\t60
гэр\t50
";

fn list() -> WordList {
    WordList::parse(FIXTURE).expect("fixture parses")
}

#[test]
fn len_counts_words_not_comments_or_blank_lines() {
    let l = list();
    assert_eq!(l.len(), 6);
    assert!(!l.is_empty());
    assert!(WordList::parse("# only a comment\n").unwrap().is_empty());
    assert!(WordList::parse("").unwrap().is_empty());
}

#[test]
fn contains_is_exact() {
    let l = list();
    assert!(l.contains("байна"));
    assert!(l.contains("гэр"));
    assert!(!l.contains("бай"));
    assert!(!l.contains("байнаа"));
    assert!(!l.contains(""));
}

#[test]
fn rank_is_line_order_from_zero() {
    let l = list();
    assert_eq!(l.rank("байна"), Some(0));
    assert_eq!(l.rank("болон"), Some(1));
    assert_eq!(l.rank("гэр"), Some(5));
    assert_eq!(l.rank("нэг"), None);
}

#[test]
fn prefix_returns_top_n_by_rank() {
    let l = list();
    assert_eq!(l.prefix("б", 10), ["байна", "болон", "бол", "банк"]);
    assert_eq!(l.prefix("б", 2), ["байна", "болон"]);
    assert_eq!(l.prefix("ба", 10), ["байна", "банк"]);
    assert_eq!(l.prefix("бол", 10), ["болон", "бол"]);
    assert_eq!(l.prefix("гэр", 10), ["гэр"]);
}

#[test]
fn prefix_edge_cases() {
    let l = list();
    assert_eq!(l.prefix("", 3), ["байна", "болон", "бол"]);
    assert_eq!(l.prefix("", 100).len(), 6);
    assert!(l.prefix("х", 5).is_empty());
    assert!(l.prefix("байнаа", 5).is_empty());
    assert!(l.prefix("б", 0).is_empty());
    assert!(WordList::parse("").unwrap().prefix("б", 5).is_empty());
}

#[test]
fn count_returns_the_stored_count() {
    let l = list();
    assert_eq!(l.count("байна"), Some(100));
    assert_eq!(l.count("гэр"), Some(50));
    assert_eq!(l.count("нэг"), None);
}

#[test]
fn count_must_be_present_and_fit_u32() {
    assert_eq!(WordList::parse("нэг\n").unwrap_err().line(), 1);
    assert_eq!(WordList::parse("нэг\t\n").unwrap_err().line(), 1);
    let ok = WordList::parse("нэг\t4294967295\n").unwrap();
    assert_eq!(ok.count("нэг"), Some(u32::MAX));
    let err = WordList::parse("хоёр\t5\nнэг\t4294967296\n").unwrap_err();
    assert_eq!(err, WordListError::BadCount { line: 2 });
}

#[test]
fn top_two_gives_the_two_best_ranked_with_counts() {
    let l = list();
    assert_eq!(l.top_two("х"), (None, None));
    assert_eq!(l.top_two("мон"), (Some(("монгол", 70)), None));
    assert_eq!(l.top_two("гэ"), (Some(("гэр", 50)), None));
    assert_eq!(l.top_two("бол"), (Some(("болон", 90)), Some(("бол", 80))));
    // Three or more matches: best two by rank, not by spelling.
    assert_eq!(l.top_two("б"), (Some(("байна", 100)), Some(("болон", 90))));
    assert_eq!(l.top_two(""), (Some(("байна", 100)), Some(("болон", 90))));
    assert_eq!(WordList::parse("").unwrap().top_two("б"), (None, None));
}

#[test]
fn leading_bom_is_ignored() {
    let l = WordList::parse("\u{feff}# c\nнэг\t5\n").unwrap();
    assert_eq!(l.rank("нэг"), Some(0));
    let l = WordList::parse("\u{feff}нэг\t5\n").unwrap();
    assert!(l.contains("нэг"));
}

#[test]
fn words_are_lower_case_and_lookups_are_case_sensitive() {
    let l = list();
    assert!(l.contains("байна"));
    assert!(!l.contains("Байна"));
    assert_eq!(l.rank("БАЙНА"), None);
    assert!(l.prefix("Ба", 5).is_empty());
}

#[test]
fn first_error_in_line_order_wins() {
    // The duplicate on line 2 comes before the bad count on line 3.
    let err = WordList::parse("нэг\t1\nнэг\t2\nхоёр\tx\n").unwrap_err();
    assert_eq!(err, WordListError::Duplicate { line: 2 });
    // The bad count on line 2 comes before the duplicate on line 3.
    let err = WordList::parse("нэг\t1\nхоёр\tx\nнэг\t2\n").unwrap_err();
    assert_eq!(err, WordListError::BadCount { line: 2 });
}

#[test]
fn crlf_line_endings_are_accepted() {
    let l = WordList::parse("нэг\t5\r\nхоёр\t4\r\n").unwrap();
    assert_eq!(l.len(), 2);
    assert!(l.contains("хоёр"));
}

#[test]
fn bad_lines_are_errors_with_a_line_number() {
    let err = WordList::parse("# c\nнэг\t5\nхоёр\tmany\n").unwrap_err();
    assert_eq!(err.line(), 3);
    let err = WordList::parse("нэг\t1\n\t5\n").unwrap_err();
    assert_eq!(err.line(), 2);
    let err = WordList::parse("нэг хоёр\t5\n").unwrap_err();
    assert_eq!(err.line(), 1);
    let err = WordList::parse("нэг\t5\t6\n").unwrap_err();
    assert_eq!(err.line(), 1);
    let err = WordList::parse("нэг\t1\nхоёр\t2\nнэг\t3\n").unwrap_err();
    assert_eq!(err.line(), 3, "duplicate word");
    assert!(err.to_string().contains("line 3"));
    let _: &WordListError = &err;
}

// Speed budgets from the plan: parse 50k entries under 20 ms, prefix query
// and top_two under 50 microseconds. Release builds enforce them; debug
// builds only check a generous bound so `cargo test` stays meaningful. CI
// runs debug, so the release budgets are NOT checked in CI: run
// `cargo test --release --test wordlist` by hand when touching wordlist.rs.
//
// The path leaves the crate on purpose. The data is not packaged with
// buuz-core (it is a separate asset, ADR 0006), and a published crate never
// compiles this test.
const REAL: &str = include_str!("../../data/wordlist/mn.txt");

fn budget(release_us: u128) -> u128 {
    if cfg!(debug_assertions) {
        release_us * 50
    } else {
        release_us
    }
}

#[test]
fn real_list_parses_fast_and_is_sane() {
    let start = Instant::now();
    let l = WordList::parse(REAL).expect("data/wordlist/mn.txt parses");
    let took = start.elapsed().as_micros();
    assert!(l.len() > 10_000, "got {} words", l.len());
    assert_eq!(l.rank("нь"), Some(0));
    assert!(l.contains("болон"));
    eprintln!("parse: {took} us for {} words", l.len());
    assert!(took < budget(20_000), "parse took {took} us");
}

#[test]
fn real_list_prefix_query_is_fast() {
    let l = WordList::parse(REAL).unwrap();
    let mut worst = 0;
    for p in ["б", "ба", "бай", "мо", "хү", "э", "зөв"] {
        let start = Instant::now();
        let got = l.prefix(p, 5);
        let took = start.elapsed().as_micros();
        worst = worst.max(took);
        assert!(!got.is_empty(), "prefix {p}");
        assert!(got.iter().all(|w| w.starts_with(p)));
    }
    eprintln!("worst prefix query: {worst} us");
    assert!(worst < budget(50), "worst prefix query took {worst} us");
}

#[test]
fn real_list_top_two_is_fast() {
    let l = WordList::parse(REAL).unwrap();
    let start = Instant::now();
    let (a, b) = l.top_two("бай");
    let took = start.elapsed().as_micros();
    eprintln!("top_two(бай): {took} us");
    let (a, b) = (a.expect("first"), b.expect("second"));
    assert!(a.0.starts_with("бай") && b.0.starts_with("бай"));
    assert!(l.rank(a.0) < l.rank(b.0));
    assert_eq!(l.count(a.0), Some(a.1));
    assert!(took < budget(50), "top_two took {took} us");
}
