// SPDX-License-Identifier: Apache-2.0

//! Spelling autocorrect (issue #1, stage 2): a typed word that is not in the
//! word list and has one clear Mongolian-aware neighbour that is, is
//! corrected at Space or punctuation. Backspace right after undoes it.

use std::sync::Arc;
use std::time::Instant;

use buuz_core::{Abbreviations, Composer, Config, Key, Options, Preset, WordList};

const FIXTURE: &str = "\
хүүхэд\t500
хүн\t900
бөр\t50
боор\t45
сайн\t200
байна\t300
үнэн\t50
хол\t40
өр\t100
хөгжим\t49
хөгжиим\t1
дэлгүүр\t300
сургууль\t400
хүний\t500
гэр\t100
тоног\t100
төного\t100
";

const REAL: &str = include_str!("../../data/wordlist/mn.txt");

fn on(base: Options) -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(FIXTURE).unwrap())),
        autocorrect: true,
        word_choice: false,
        ..base
    }
}

/// Smart mode with smart vowels off: the preedit keeps the typed harmony
/// (`huuhed` gives `хуухэд`), so autocorrect has a word to fix. With smart
/// vowels on, most harmony slips are already fixed while typing.
fn plain() -> Options {
    on(Options {
        smart_vowels: false,
        ..Options::smart()
    })
}

fn composer() -> Composer {
    Composer::with_options(plain())
}

/// Feed keys the way a shell does: apply `delete_before` and `commit`, and
/// pass through what the composer does not consume.
fn feed(c: &mut Composer, keys: &[Key]) -> String {
    let mut out = String::new();
    for &k in keys {
        let r = c.process_key(k);
        for _ in 0..r.delete_before {
            out.pop();
        }
        if let Some(s) = r.commit {
            out.push_str(&s);
        }
        if !r.consumed {
            match k {
                Key::Space => out.push(' '),
                Key::Symbol(ch) => out.push(ch),
                _ => {}
            }
        }
    }
    out
}

fn typed(s: &str) -> Vec<Key> {
    s.chars()
        .map(|ch| match ch {
            ' ' => Key::Space,
            '.' | ',' | '!' | '@' | ':' | '/' => Key::Symbol(ch),
            _ => Key::Char(ch),
        })
        .collect()
}

fn run(opts: Options, s: &str) -> String {
    feed(&mut Composer::with_options(opts), &typed(s))
}

#[test]
fn off_by_default_and_in_every_preset() {
    assert!(!Options::default().autocorrect);
    assert!(!Options::smart().autocorrect);
    assert!(!Options::classic().autocorrect);
}

#[test]
fn corrects_two_harmony_edits_at_space() {
    let mut c = composer();
    let out = feed(&mut c, &typed("huuhed"));
    assert_eq!(out, "");
    assert_eq!(c.preedit(), "хуухэд");
    let r = c.process_key(Key::Space);
    assert!(r.corrected);
    assert_eq!(r.commit.as_deref(), Some("хүүхэд"));
}

#[test]
fn corrected_is_false_for_other_words() {
    let mut c = composer();
    for k in typed("sain") {
        c.process_key(k);
    }
    let r = c.process_key(Key::Space);
    assert!(!r.corrected);
    assert_eq!(r.commit.as_deref(), Some("сайн"));
}

#[test]
fn backspace_restores_and_space_commits_unchanged() {
    let mut c = composer();
    let mut out = feed(&mut c, &typed("huuhed "));
    assert_eq!(out, "хүүхэд ");
    let r = c.process_key(Key::Backspace);
    assert!(r.consumed);
    assert_eq!(r.delete_before, 7);
    assert!(!r.corrected);
    assert_eq!(c.preedit(), "хуухэд");
    for _ in 0..r.delete_before {
        out.pop();
    }
    out.push_str(&feed(&mut c, &[Key::Space]));
    assert_eq!(out, "хуухэд ");
}

#[test]
fn a_rejected_word_is_not_corrected_again() {
    let mut c = composer();
    feed(&mut c, &typed("huuhed "));
    c.process_key(Key::Backspace);
    c.process_key(Key::Space);
    assert_eq!(feed(&mut c, &typed("huuhed ")), "хуухэд ");
}

#[test]
fn the_rejected_set_keeps_at_most_256_words() {
    let letters = ['b', 'd', 'l', 'm', 'n', 'p', 'r', 't'];
    let mut raws = Vec::new();
    let mut fixture = String::new();
    for a in letters {
        for b in letters {
            for d in letters {
                let raw = format!("huuhed{a}{b}{d}");
                let typed_word = run(
                    Options {
                        autocorrect: false,
                        ..plain()
                    },
                    &format!("{raw} "),
                );
                fixture.push_str(&format!(
                    "{}\t100\n",
                    typed_word.trim().replace("хуу", "хүү")
                ));
                raws.push(raw);
            }
        }
    }
    let opts = Options {
        words: Some(Arc::new(WordList::parse(&fixture).unwrap())),
        ..plain()
    };
    let mut c = Composer::with_options(opts);
    for raw in raws.iter().take(257) {
        let corrected = feed(&mut c, &typed(&format!("{raw} ")));
        assert!(corrected.contains("хүү"), "{raw} -> {corrected}");
        // The undo works also when the set is full (the 257th word).
        let undone = feed(&mut c, &[Key::Backspace, Key::Space]);
        assert!(undone.contains("хуу"), "{raw} -> {undone}");
    }
    // The first 256 are remembered; the 257th is not.
    assert!(feed(&mut c, &typed(&format!("{} ", raws[0]))).contains("хуу"));
    assert!(feed(&mut c, &typed(&format!("{} ", raws[255]))).contains("хуу"));
    assert!(feed(&mut c, &typed(&format!("{} ", raws[256]))).contains("хүү"));
}

#[test]
fn known_words_are_untouched() {
    assert_eq!(run(plain(), "sain "), "сайн ");
    assert_eq!(run(plain(), "hol "), "хол ");
    // Smart vowels already give a listed word: nothing to correct.
    assert_eq!(run(on(Options::smart()), "huuhed "), "хүүхэд ");
}

#[test]
fn two_close_candidates_leave_the_word_alone() {
    // бор is not listed; бөр (50) and боор (45) are less than 3x apart.
    assert_eq!(run(on(Options::smart()), "bor "), "бор ");
}

#[test]
fn a_weak_winner_is_not_enough() {
    // хөгжим has count 49, below the floor of 50; the typed word is хогжим.
    assert_eq!(run(on(Options::smart()), "hogjim "), "хогжим ");
}

#[test]
fn a_clear_winner_over_a_weak_neighbour_wins() {
    // үнэн has exactly 50: enough. Typed унэн needs one edit (у to ү).
    assert_eq!(run(plain(), "unen "), "үнэн ");
}

#[test]
fn smart_mode_fixes_a_missing_double_vowel() {
    let opts = on(Options::smart());
    assert_eq!(run(opts, "delgur "), "дэлгүүр ");
}

#[test]
fn smart_mode_fixes_a_missing_soft_sign() {
    let opts = on(Options::smart());
    assert_eq!(run(opts, "surguul "), "сургууль ");
}

#[test]
fn smart_mode_swaps_ii_and_y() {
    let opts = on(Options::smart());
    assert_eq!(run(opts, "hunii "), "хүний ");
}

#[test]
fn a_double_vowel_is_cut_to_one() {
    let opts = on(Options::smart());
    assert_eq!(run(opts, "huun "), "хүн ");
}

#[test]
fn a_leading_capital_is_kept() {
    assert_eq!(run(plain(), "Huuhed "), "Хүүхэд ");
}

#[test]
fn a_capital_inside_the_word_skips() {
    assert_eq!(run(plain(), "hUuhed "), "хУухэд ");
}

#[test]
fn an_all_upper_case_word_is_left_alone() {
    assert_eq!(run(plain(), "HUUHED "), "ХУУХЭД ");
}

#[test]
fn words_under_three_letters_are_skipped() {
    // ор is not listed and өр (100) is one edit away.
    assert_eq!(run(plain(), "or "), "ор ");
}

#[test]
fn the_latin_view_is_skipped() {
    let mut keys = typed("huuhed");
    keys.extend([Key::Escape, Key::Space]);
    assert_eq!(feed(&mut composer(), &keys), "huuhed ");
}

#[test]
fn an_auto_latin_run_is_skipped() {
    assert_eq!(run(plain(), "a@huuhed "), "a@huuhed ");
    assert_eq!(run(plain(), "https://huuhed "), "https://huuhed ");
}

#[test]
fn a_reseeded_word_is_skipped() {
    let mut keys = typed("huuhed");
    keys.extend([Key::Left, Key::Right, Key::Space]);
    assert_eq!(feed(&mut composer(), &keys), "хуухэд ");
}

#[test]
fn an_abbreviation_expansion_is_skipped() {
    let abbr = Abbreviations::from_pairs([("huuhed", "хуухэд")]).unwrap();
    let opts = Options {
        abbreviations: Some(Arc::new(abbr)),
        ..plain()
    };
    assert_eq!(run(opts, "huuhed "), "хуухэд ");
}

#[test]
fn punctuation_commit_corrects() {
    assert_eq!(run(plain(), "huuhed."), "хүүхэд.");
    assert_eq!(run(plain(), "huuhed,"), "хүүхэд,");
}

#[test]
fn commit_does_not_correct() {
    let mut c = composer();
    feed(&mut c, &typed("huuhed"));
    assert_eq!(c.commit().as_deref(), Some("хуухэд"));
}

#[test]
fn keys_do_not_correct_inside_the_word() {
    let mut c = composer();
    for k in typed("huuhed") {
        let r = c.process_key(k);
        assert!(!r.corrected);
        assert_eq!(r.commit, None);
    }
}

#[test]
fn off_is_unchanged() {
    let off = Options {
        autocorrect: false,
        word_choice: false,
        ..plain()
    };
    assert_eq!(run(off, "huuhed "), "хуухэд ");
}

#[test]
fn classic_options_with_a_list_and_autocorrect_off_are_unchanged() {
    let classic = Options {
        autocorrect: false,
        ..on(Options::classic())
    };
    let plain = Options::classic();
    for s in ["huuhed ", "sain bor ", "unen hol."] {
        assert_eq!(run(classic.clone(), s), run(plain.clone(), s), "{s}");
    }
}

#[test]
fn an_unlisted_word_with_no_neighbour_is_untouched() {
    let off = Options {
        autocorrect: false,
        ..plain()
    };
    assert_eq!(run(plain(), "qwrtz "), run(off, "qwrtz "));
}

#[test]
fn no_list_means_no_change() {
    let opts = Options {
        words: None,
        ..plain()
    };
    assert_eq!(run(opts, "huuhed "), "хуухэд ");
}

#[test]
fn config_switch_and_word_list_need() {
    let smart = Config::default();
    assert_eq!(smart.autocorrect(), None);
    assert!(!smart.options().autocorrect);
    let mut classic = Config::default();
    classic.set_preset(Preset::Classic);
    assert!(!classic.needs_word_list());
    classic.set_autocorrect(Some(true));
    assert!(classic.needs_word_list());
    assert!(classic.options().autocorrect);
    let mut smart = Config::default();
    smart.set_completion(Some(false));
    // Word choice is on in smart mode and also needs the list.
    assert!(smart.needs_word_list());
    smart.set_word_choice(Some(false));
    assert!(!smart.needs_word_list());
    smart.set_autocorrect(Some(true));
    assert!(smart.needs_word_list());
    assert!(smart.options().autocorrect);
    assert!(smart.options().words.is_none());
}

#[test]
fn config_parses_and_round_trips() {
    let parsed = Config::parse("buuz_config = 1\n[options]\nautocorrect = true\n")
        .unwrap()
        .config;
    assert_eq!(parsed.autocorrect(), Some(true));
    assert!(parsed.options().autocorrect);
    assert_eq!(Config::parse(&parsed.to_text()).unwrap().config, parsed);
    assert!(parsed.to_text().contains("autocorrect = true"));
}

#[test]
fn config_duplicate_is_an_error() {
    let text = "buuz_config = 1\n[options]\nautocorrect = true\nautocorrect = false\n";
    let err = Config::parse(text).unwrap_err();
    assert!(err.to_string().contains("autocorrect"), "{err}");
    assert!(err.to_string().contains("line 4"), "{err}");
}

/// Time of the Space key (the whole correction) per word against the budget
/// of 1 ms in docs/architecture.md section 4. The assert on the 99th
/// percentile runs in release only (CI runs `cargo test --release` too).
#[test]
fn autocorrect_is_fast_per_word_on_the_real_list() {
    let opts = Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        autocorrect: true,
        ..Options::smart()
    };
    let words = [
        "huuhed",
        "sain",
        "bayrlalaa",
        "uulzalt",
        "zahidal",
        "oyuutan",
        "tuhai",
        "bolovsrol",
        "ulsiin",
        "gazrin",
        "oeoeoeoeoeoeoeoe",
        "qwqwqwqwqwqwqwqw",
        "xxxxxxxxxxxxxxxxxxxx",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ];
    let mut c = Composer::with_options(opts);
    let mut samples = Vec::new();
    for round in 0..11 {
        for w in words {
            for ch in w.chars() {
                c.process_key(Key::Char(ch));
            }
            let start = Instant::now();
            let r = c.process_key(Key::Space);
            let took = start.elapsed().as_nanos();
            std::hint::black_box(r);
            if round > 0 {
                samples.push(took);
            }
        }
    }
    let mean = samples.iter().sum::<u128>() / samples.len() as u128;
    let p = p99(&mut samples);
    eprintln!("autocorrect: mean {mean} ns, p99 {p} ns");
    if !cfg!(debug_assertions) {
        assert!(p < 1_000_000, "p99 {p} ns, mean {mean} ns");
    }
}

/// The 99th percentile of `samples` (sorts them). One slow sample on a busy
/// machine (a context switch) must not fail a speed budget.
fn p99(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 99 / 100).min(samples.len() - 1)]
}

#[test]
fn enter_does_not_correct() {
    let mut c = composer();
    feed(&mut c, &typed("huuhed"));
    let r = c.process_key(Key::Other);
    assert_eq!(r.commit.as_deref(), Some("хуухэд"));
    assert!(!r.corrected);
}

#[test]
fn escape_without_keep_latin_does_not_correct() {
    let mut c = Composer::with_options(Options {
        keep_latin: false,
        ..plain()
    });
    feed(&mut c, &typed("huuhed"));
    let r = c.process_key(Key::Escape);
    assert_eq!(r.commit.as_deref(), Some("хуухэд"));
    assert!(!r.corrected);
}

#[test]
fn punctuation_corrects_with_auto_latin_off() {
    let opts = Options {
        auto_latin: false,
        ..plain()
    };
    assert_eq!(run(opts, "huuhed."), "хүүхэд.");
}

#[test]
fn a_word_finished_after_tab_is_not_corrected() {
    let mut c = Composer::with_options(on(Options::smart()));
    feed(&mut c, &typed("ton"));
    assert_eq!(c.completion(), Some("ог"));
    c.process_key(Key::Tab);
    c.process_key(Key::Char('o'));
    assert_eq!(c.preedit(), "тоного");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("тоного"));
    assert!(!r.corrected);
}

fn real_smart() -> Options {
    Options {
        words: Some(Arc::new(WordList::parse(REAL).unwrap())),
        autocorrect: true,
        ..Options::smart()
    }
}

#[test]
fn final_vowels_are_never_edited_on_the_real_list() {
    for (typed_word, stays) in [
        ("ideye ", "идэе "),
        ("huleeye ", "хүлээе "),
        ("zahiaa ", "захиаа "),
        ("shinee ", "шинээ "),
    ] {
        assert_eq!(run(real_smart(), typed_word), stays);
    }
}

#[test]
fn good_corrections_still_work_on_the_real_list() {
    for (typed_word, fixed) in [
        ("delgur ", "дэлгүүр "),
        ("hunii ", "хүний "),
        ("ugui ", "үгүй "),
        ("bolohgui ", "болохгүй "),
    ] {
        assert_eq!(run(real_smart(), typed_word), fixed);
    }
}
