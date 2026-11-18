// SPDX-License-Identifier: Apache-2.0

//! The personal dictionary (issue #12): the `UserDict` type and the learning
//! events of the composer.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;

use buuz_core::{
    Composer, Config, Key, Learned, Options, Preset, UserDict, UserDictKind, UserDictProblem,
    WordList, MAX_USERDICT_ENTRIES, MAX_USERDICT_WORD_CHARS,
};

// ---- an allocation counter, per thread, for the typing path test ----

struct Counting;

thread_local! {
    static ALLOCS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCS.try_with(|a| a.set(a.get() + 1));
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let _ = ALLOCS.try_with(|a| a.set(a.get() + 1));
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn allocs() -> usize {
    ALLOCS.with(Cell::get)
}

// ---- helpers ----

const FIXTURE: &str = "\
хүүхэд\t500
хүн\t900
сайн\t200
байна\t300
баярлалаа\t100
баяр\t50
баярлах\t40
хүний\t500
";

fn list() -> Arc<WordList> {
    Arc::new(WordList::parse(FIXTURE).unwrap())
}

/// Smart mode, smart vowels off (so `huuhed` stays a slip for autocorrect),
/// autocorrect on, with the given snapshot.
fn opts(user: Option<UserDict>) -> Options {
    Options {
        words: Some(list()),
        autocorrect: true,
        smart_vowels: false,
        user: user.map(|u| Arc::new(u.snapshot())),
        ..Options::smart()
    }
}

fn type_str(c: &mut Composer, s: &str) {
    for ch in s.chars() {
        c.process_key(Key::Char(ch));
    }
}

/// Type `s`, then press `end`; return the outcome of `end`.
fn word_then(c: &mut Composer, s: &str, end: Key) -> buuz_core::KeyOutcome {
    type_str(c, s);
    c.process_key(end)
}

fn used(w: &str) -> Option<Learned> {
    Some(Learned::Used(w.to_string()))
}

/// A pseudo-random Cyrillic word, different for each `i`.
fn spread_word(i: usize) -> String {
    const L: [char; 12] = ['а', 'б', 'в', 'г', 'д', 'е', 'ж', 'з', 'и', 'к', 'л', 'м'];
    let mut x = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xABCD;
    let mut w = String::new();
    for _ in 0..8 {
        w.push(L[(x % 12) as usize]);
        x /= 12;
        x ^= x >> 7;
    }
    format!("{w}{i}")
}

// ---- UserDict ----

#[test]
fn add_reject_and_use_are_remembered() {
    let mut d = UserDict::new();
    assert!(d.is_empty());
    assert!(d.add("Тэнгис"));
    assert!(d.is_added("тэнгис"));
    assert!(d.reject("хуухэд"));
    assert!(d.is_rejected("хуухэд"));
    assert!(d.record_use("сайн"));
    assert!(d.record_use("сайн"));
    assert_eq!(d.uses("сайн"), 2);
    assert_eq!(d.uses("байна"), 0);
    assert_eq!(d.len(), 3);
    assert!(d.remove("тэнгис"));
    assert!(!d.remove("тэнгис"));
    assert!(!d.is_added("тэнгис"));
    d.clear();
    assert!(d.is_empty());
}

#[test]
fn bad_words_are_refused() {
    let mut d = UserDict::new();
    assert!(!d.add(""));
    assert!(!d.add("two words"));
    assert!(!d.add("tab\tbed"));
    assert!(!d.add(&"а".repeat(MAX_USERDICT_WORD_CHARS + 1)));
    assert!(d.add(&"а".repeat(MAX_USERDICT_WORD_CHARS)));
    assert_eq!(d.len(), 1);
}

#[test]
fn apply_takes_learned_events() {
    let mut d = UserDict::new();
    d.apply(&Learned::Used("сайн".into()));
    d.apply(&Learned::Rejected("хуухэд".into()));
    assert_eq!(d.uses("сайн"), 1);
    assert!(d.is_rejected("хуухэд"));
}

#[test]
fn round_trip_keeps_entries_and_order() {
    let mut d = UserDict::new();
    d.add("тэнгис");
    d.record_use("сайн");
    d.record_use("сайн");
    d.reject("хуухэд");
    let text = d.to_text();
    assert!(text.lines().any(|l| l == "add\tтэнгис\t1"));
    assert!(text.lines().any(|l| l == "use\tсайн\t2"));
    assert!(text.lines().any(|l| l == "reject\tхуухэд\t1"));
    let parsed = UserDict::parse(&text).unwrap();
    assert!(parsed.warnings.is_empty());
    assert_eq!(parsed.dict.to_text(), text);
    assert!(parsed.dict.is_added("тэнгис"));
    assert_eq!(parsed.dict.uses("сайн"), 2);
}

#[test]
fn entries_are_listed_oldest_first() {
    let mut d = UserDict::new();
    d.add("а");
    d.reject("б");
    d.record_use("в");
    let e = d.entries();
    assert_eq!(e.len(), 3);
    assert_eq!((e[0].kind, e[0].word.as_str()), (UserDictKind::Add, "а"));
    assert_eq!((e[1].kind, e[1].word.as_str()), (UserDictKind::Reject, "б"));
    assert_eq!((e[2].kind, e[2].count), (UserDictKind::Use, 1));
}

#[test]
fn the_cap_evicts_the_least_recently_used_entry() {
    let mut d = UserDict::new();
    for i in 0..MAX_USERDICT_ENTRIES {
        assert!(d.add(&format!("w{i}")));
    }
    assert_eq!(d.len(), MAX_USERDICT_ENTRIES);
    // Touch the oldest: it is now the newest.
    assert!(d.add("w0"));
    assert!(d.add("extra"));
    assert_eq!(d.len(), MAX_USERDICT_ENTRIES);
    assert!(d.is_added("w0"));
    assert!(d.is_added("extra"));
    assert!(!d.is_added("w1"), "w1 was the least recently used");
    assert!(d.is_added("w2"));
}

#[test]
fn normal_typing_does_not_evict_added_words_or_rejections() {
    let mut d = UserDict::new();
    d.add("тэнгис");
    d.reject("хуухэд");
    for i in 0..MAX_USERDICT_ENTRIES + 500 {
        d.record_use(&format!("w{i}"));
    }
    assert_eq!(d.len(), MAX_USERDICT_ENTRIES);
    assert!(d.is_added("тэнгис"));
    assert!(d.is_rejected("хуухэд"));
    // The oldest use went first.
    assert_eq!(d.uses("w0"), 0);
    assert_eq!(d.uses(&format!("w{}", MAX_USERDICT_ENTRIES + 499)), 1);
}

#[test]
fn a_full_dictionary_of_added_words_evicts_the_oldest_added_word() {
    let mut d = UserDict::new();
    for i in 0..MAX_USERDICT_ENTRIES {
        d.add(&format!("w{i}"));
    }
    // A use count is dropped rather than evicting a word the user added.
    d.record_use("сайн");
    assert_eq!(d.uses("сайн"), 0);
    assert!(d.is_added("w0"));
    // An added word is a deliberate act: the oldest added word goes.
    d.add("new");
    assert!(d.is_added("new"));
    assert!(!d.is_added("w0"));
}

#[test]
fn using_an_added_word_keeps_it_fresh() {
    let mut d = UserDict::new();
    for i in 0..MAX_USERDICT_ENTRIES {
        d.add(&format!("w{i}"));
    }
    d.record_use("w0"); // refreshes the add slot of w0
    d.add("new");
    assert!(d.is_added("w0"));
    assert!(!d.is_added("w1"));
}

#[test]
fn parse_skips_bad_lines_and_reports_them() {
    let text = "# a comment\n\
        add\tтэнгис\t1\n\
        \n\
        nope\tx\t1\n\
        add\tbad\n\
        use\tсайн\tmany\n\
        use\tсайн\t0\n\
        add\t\t1\n\
        add\ttwo words\t1\n\
        add\tтэнгис\t1\n\
        reject\tхуухэд\t3\n\
        use\tbyte\t1\textra\n";
    let p = UserDict::parse(text).unwrap();
    assert_eq!(p.dict.len(), 2);
    assert!(p.dict.is_added("тэнгис"));
    assert!(p.dict.is_rejected("хуухэд"));
    let got: Vec<(usize, UserDictProblem)> =
        p.warnings.iter().map(|w| (w.line, w.problem)).collect();
    assert_eq!(
        got,
        [
            (4, UserDictProblem::UnknownKind),
            (5, UserDictProblem::BadLine),
            (6, UserDictProblem::BadCount),
            (7, UserDictProblem::BadCount),
            (8, UserDictProblem::EmptyWord),
            (9, UserDictProblem::BadWord),
            (10, UserDictProblem::Duplicate),
            (12, UserDictProblem::BadLine),
        ]
    );
}

#[test]
fn parse_skips_long_words_and_refuses_an_oversize_file() {
    let long = "а".repeat(MAX_USERDICT_WORD_CHARS + 1);
    let p = UserDict::parse(&format!("add\t{long}\t1\nadd\tок\t1\n")).unwrap();
    assert_eq!(p.dict.len(), 1);
    assert_eq!(p.warnings[0].problem, UserDictProblem::WordTooLong);
    assert!(UserDict::parse(&" ".repeat(buuz_core::MAX_USERDICT_BYTES + 1)).is_err());
}

#[test]
fn parse_normalises_the_count_of_an_added_word() {
    let p = UserDict::parse("add\tтэнгис\t77\n").unwrap();
    assert_eq!(p.dict.to_text(), "add\tтэнгис\t1\n");
}

#[test]
fn a_duplicate_after_the_cap_is_still_reported() {
    let mut text = String::new();
    for i in 0..MAX_USERDICT_ENTRIES + 3 {
        text.push_str(&format!("add\tw{i}\t1\n"));
    }
    text.push_str("add\tw0\t1\n");
    let p = UserDict::parse(&text).unwrap();
    assert_eq!(p.warnings.len(), 1);
    assert_eq!(p.warnings[0].line, MAX_USERDICT_ENTRIES + 4);
    assert_eq!(p.warnings[0].problem, UserDictProblem::Duplicate);
    assert_eq!(p.dict.len(), MAX_USERDICT_ENTRIES);
}

#[test]
fn parse_trims_use_entries_before_added_ones() {
    let mut text = String::from("add\tkeep\t1\nreject\tkeepr\t1\n");
    for i in 0..MAX_USERDICT_ENTRIES {
        text.push_str(&format!("use\tw{i}\t1\n"));
    }
    let p = UserDict::parse(&text).unwrap();
    assert_eq!(p.dict.len(), MAX_USERDICT_ENTRIES);
    assert!(p.dict.is_added("keep"));
    assert!(p.dict.is_rejected("keepr"));
    assert_eq!(p.dict.uses("w0"), 0);
    assert_eq!(p.dict.uses("w1"), 0);
    assert_eq!(p.dict.uses(&format!("w{}", MAX_USERDICT_ENTRIES - 1)), 1);
}

#[test]
fn a_one_mebibyte_file_parses_fast() {
    // Short lines: about 170,000 of them, far above the cap.
    let mut text = String::new();
    let mut i = 0u32;
    while text.len() + 16 < buuz_core::MAX_USERDICT_BYTES {
        text.push_str(&format!("use\tw{i}\t1\n"));
        i += 1;
    }
    let start = Instant::now();
    let p = UserDict::parse(&text).unwrap();
    let took = start.elapsed();
    eprintln!("parse of {} bytes: {took:?}", text.len());
    assert_eq!(p.dict.len(), MAX_USERDICT_ENTRIES);
    if !cfg!(debug_assertions) {
        assert!(took.as_millis() < 100, "took {took:?}");
    }
}

#[test]
fn parse_over_the_cap_keeps_the_last_entries() {
    let mut text = String::new();
    for i in 0..MAX_USERDICT_ENTRIES + 3 {
        text.push_str(&format!("add\tw{i}\t1\n"));
    }
    let p = UserDict::parse(&text).unwrap();
    assert_eq!(p.dict.len(), MAX_USERDICT_ENTRIES);
    assert!(!p.dict.is_added("w0"));
    assert!(p.dict.is_added(&format!("w{}", MAX_USERDICT_ENTRIES + 2)));
}

// ---- learning events ----

#[test]
fn a_known_word_gives_a_used_event_on_commit_only() {
    let mut c = Composer::with_options(opts(None));
    for ch in "sain".chars() {
        let r = c.process_key(Key::Char(ch));
        assert_eq!(r.learned, None, "no event before the commit");
    }
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("сайн"));
    assert_eq!(r.learned, used("сайн"));
    // The next key carries no event.
    assert_eq!(c.process_key(Key::Char('b')).learned, None);
}

#[test]
fn punctuation_commits_a_used_event_too() {
    let mut c = Composer::with_options(opts(None));
    let r = word_then(&mut c, "sain", Key::Symbol(','));
    assert_eq!(r.learned, used("сайн"));
}

#[test]
fn an_unknown_word_gives_no_event() {
    let mut c = Composer::with_options(opts(None));
    let r = word_then(&mut c, "tengis", Key::Space);
    assert_eq!(r.commit.as_deref(), Some("тэнгис"));
    assert_eq!(r.learned, None);
}

#[test]
fn an_added_word_is_known_and_counts_as_used() {
    let mut d = UserDict::new();
    d.add("тэнгис");
    let mut c = Composer::with_options(opts(Some(d)));
    let r = word_then(&mut c, "tengis", Key::Space);
    assert_eq!(r.learned, used("тэнгис"));
}

#[test]
fn a_capitalised_word_is_recorded_in_lower_case() {
    let mut c = Composer::with_options(opts(None));
    let r = word_then(&mut c, "Sain", Key::Space);
    assert_eq!(r.learned, used("сайн"));
}

#[test]
fn undoing_a_correction_gives_a_rejected_event() {
    let mut c = Composer::with_options(opts(None));
    let r = word_then(&mut c, "huuhed", Key::Space);
    assert!(r.corrected);
    assert_eq!(r.learned, None, "the correction itself teaches nothing");
    let r = c.process_key(Key::Backspace);
    assert_eq!(r.learned, Some(Learned::Rejected("хуухэд".into())));
    assert_eq!(c.preedit(), "хуухэд");
    // Committing the reopened word again is not a use of a list word.
    let r = c.process_key(Key::Space);
    assert_eq!(r.learned, None);
}

#[test]
fn a_reopened_word_is_not_learned_as_used() {
    let mut c = Composer::with_options(opts(None));
    word_then(&mut c, "sain", Key::Space);
    c.process_key(Key::Backspace); // reopens `sain` (keep_latin)
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("сайн"));
    assert_eq!(r.learned, None);
}

#[test]
fn a_word_accepted_with_tab_counts_as_used() {
    let mut c = Composer::with_options(opts(None));
    type_str(&mut c, "sai");
    assert_eq!(c.completion(), Some("н"));
    assert_eq!(c.process_key(Key::Tab).learned, None);
    assert_eq!(c.preedit(), "сайн");
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("сайн"));
    assert_eq!(r.learned, used("сайн"));
}

#[test]
fn a_known_word_before_a_dot_is_learned_before_the_run_turns_latin() {
    // `sain.mn` is a domain: the run turns Latin at the Space, but `sain`
    // was already committed at the dot. This is documented, not a goal.
    let mut c = Composer::with_options(opts(None));
    let mut events = Vec::new();
    for k in [
        Key::Char('s'),
        Key::Char('a'),
        Key::Char('i'),
        Key::Char('n'),
        Key::Symbol('.'),
        Key::Char('m'),
        Key::Char('n'),
        Key::Space,
    ] {
        events.extend(c.process_key(k).learned);
    }
    assert_eq!(events, [Learned::Used("сайн".into())]);
}

#[test]
fn a_word_before_an_at_sign_is_not_learned() {
    let mut c = Composer::with_options(opts(None));
    type_str(&mut c, "sain");
    let r = c.process_key(Key::Symbol('@'));
    assert_eq!(r.learned, None);
}

#[test]
fn the_classic_preset_with_learning_true_learns() {
    let p = Config::parse("buuz_config = 1\npreset = \"classic\"\n[options]\nlearning = true\n")
        .unwrap();
    assert_eq!(p.config.preset(), Preset::Classic);
    let o = Options {
        words: Some(list()),
        ..p.config.options()
    };
    assert!(o.learning);
    let mut c = Composer::with_options(o);
    let r = word_then(&mut c, "sain", Key::Space);
    // The output is the classic output; only the event is new.
    assert_eq!(r.commit.as_deref(), Some("сайн"));
    assert_eq!(r.learned, used("сайн"));
    let off = Config::parse("buuz_config = 1\npreset = \"classic\"\n").unwrap();
    assert!(!off.config.options().learning);
}

#[test]
fn learning_off_gives_no_events() {
    let mut c = Composer::with_options(Options {
        learning: false,
        ..opts(None)
    });
    assert_eq!(word_then(&mut c, "sain", Key::Space).learned, None);
    word_then(&mut c, "huuhed", Key::Space);
    assert_eq!(c.process_key(Key::Backspace).learned, None);
}

#[test]
fn classic_mode_learns_nothing_and_is_unchanged() {
    assert!(!Options::classic().learning);
    assert!(Options::smart().learning);
    let classic = Options {
        words: Some(list()),
        user: Some(Arc::new(UserDict::new().snapshot())),
        ..Options::classic()
    };
    let mut c = Composer::with_options(classic);
    let r = word_then(&mut c, "sain", Key::Space);
    assert_eq!(r.learned, None);
    assert_eq!(r.commit.as_deref(), Some("сайн"));
    let plain = buuz_core::convert_with("sain baina", &Options::classic());
    let with_user = buuz_core::convert_with(
        "sain baina",
        &Options {
            user: Some(Arc::new({
                let mut d = UserDict::new();
                d.add("сайн");
                d.snapshot()
            })),
            ..Options::classic()
        },
    );
    assert_eq!(plain, with_user);
}

#[test]
fn the_latin_view_gives_no_events() {
    let mut c = Composer::with_options(opts(None));
    type_str(&mut c, "sain");
    c.process_key(Key::Escape); // the raw Latin keys
    let r = c.process_key(Key::Space);
    assert_eq!(r.commit.as_deref(), Some("sain"));
    assert_eq!(r.learned, None);
}

#[test]
fn an_auto_latin_run_gives_no_events() {
    let mut c = Composer::with_options(opts(None));
    // After `@` the run is Latin: `sain` stays Latin.
    c.process_key(Key::Char('a'));
    c.process_key(Key::Symbol('@'));
    let r = word_then(&mut c, "sain", Key::Space);
    assert_eq!(r.commit.as_deref(), Some("sain"));
    assert_eq!(r.learned, None);
}

#[test]
fn an_abbreviation_expansion_gives_no_events() {
    let mut c = Composer::with_options(opts(None));
    let r = word_then(&mut c, "bn", Key::Space);
    assert_eq!(r.commit.as_deref(), Some("байна"));
    assert_eq!(r.learned, None);
}

#[test]
fn a_corrected_word_gives_no_used_event() {
    let mut c = Composer::with_options(opts(None));
    let r = word_then(&mut c, "huuhed", Key::Space);
    assert_eq!(r.commit.as_deref(), Some("хүүхэд"));
    assert_eq!(r.learned, None);
}

// ---- autocorrect ----

#[test]
fn an_added_word_is_never_corrected() {
    let mut d = UserDict::new();
    d.add("хуухэд");
    let mut c = Composer::with_options(opts(Some(d)));
    let r = word_then(&mut c, "huuhed", Key::Space);
    assert!(!r.corrected);
    assert_eq!(r.commit.as_deref(), Some("хуухэд"));
}

#[test]
fn a_rejected_word_is_never_corrected_in_a_new_session() {
    let mut c = Composer::with_options(opts(None));
    assert!(word_then(&mut c, "huuhed", Key::Space).corrected);
    // Without the user dictionary a new composer corrects again.
    let mut d = UserDict::new();
    d.reject("хуухэд");
    let mut c = Composer::with_options(opts(Some(d)));
    let r = word_then(&mut c, "huuhed", Key::Space);
    assert!(!r.corrected);
    assert_eq!(r.commit.as_deref(), Some("хуухэд"));
}

// ---- completion ----

fn ghost_for(user: Option<UserDict>, typed: &str) -> Option<String> {
    let mut c = Composer::with_options(opts(user));
    type_str(&mut c, typed);
    c.completion().map(str::to_string)
}

#[test]
fn an_added_word_appears_in_completion() {
    let mut d = UserDict::new();
    d.add("тэнгисбаяр");
    assert_eq!(ghost_for(None, "tengi"), None);
    assert_eq!(ghost_for(Some(d), "tengi").as_deref(), Some("сбаяр"));
}

#[test]
fn a_user_word_beats_a_list_word_with_the_same_prefix() {
    // The list has `баярлалаа` as the clear winner for `баярл`.
    assert_eq!(ghost_for(None, "bayarl").as_deref(), Some("алаа"));
    let mut d = UserDict::new();
    d.add("баярлана");
    assert_eq!(ghost_for(Some(d), "bayarl").as_deref(), Some("ана"));
}

#[test]
fn a_word_used_three_times_beats_the_list_but_two_uses_do_not() {
    let mut d = UserDict::new();
    d.record_use("баярлах");
    d.record_use("баярлах");
    assert_eq!(
        ghost_for(Some(d.clone()), "bayarl").as_deref(),
        Some("алаа")
    );
    d.record_use("баярлах");
    assert_eq!(ghost_for(Some(d), "bayarl").as_deref(), Some("ах"));
}

#[test]
fn the_more_used_user_word_wins_among_user_words() {
    let mut d = UserDict::new();
    d.add("баярлана");
    d.add("баярлаарай");
    d.record_use("баярлаарай");
    assert_eq!(ghost_for(Some(d), "bayarl").as_deref(), Some("аарай"));
}

#[test]
fn the_word_as_typed_wins_over_a_longer_user_word_and_no_ghost_shows() {
    let mut d = UserDict::new();
    for _ in 0..10 {
        d.record_use("сайн");
    }
    for _ in 0..3 {
        d.record_use("сайнаар");
    }
    assert_eq!(ghost_for(Some(d), "sain"), None);
}

#[test]
fn a_user_word_ghost_is_upper_case_after_an_upper_case_preedit() {
    let mut d = UserDict::new();
    d.add("тэнгисбаяр");
    assert_eq!(ghost_for(Some(d), "TENGI").as_deref(), Some("СБАЯР"));
}

#[test]
fn the_snapshot_scans_only_the_prefix_range() {
    let mut d = UserDict::new();
    // 5,000 words spread over many prefixes.
    for i in 0..MAX_USERDICT_ENTRIES {
        d.add(&spread_word(i));
    }
    let snap = d.snapshot();
    assert_eq!(snap.candidates(""), MAX_USERDICT_ENTRIES);
    let biggest = (0..400)
        .map(|i| snap.candidates(&spread_word(i)[..6]))
        .max()
        .unwrap();
    assert!(biggest < 50, "range of {biggest} for a 3-letter prefix");
    assert!(snap.candidates("zzz") == 0);
}

#[test]
fn rejected_words_are_not_candidates_in_the_snapshot() {
    let mut d = UserDict::new();
    d.add("баярлана");
    d.reject("баярлана");
    let snap = d.snapshot();
    assert_eq!(snap.candidates("баярл"), 0);
    assert!(snap.is_rejected("баярлана"));
    assert!(snap.is_added("баярлана"));
}

#[test]
fn a_rejected_word_is_not_offered() {
    let mut d = UserDict::new();
    d.reject("баярлана");
    assert_eq!(ghost_for(Some(d), "bayarl").as_deref(), Some("алаа"));
}

// ---- config ----

#[test]
fn the_config_switch_sets_learning() {
    assert_eq!(Config::default().learning(), None);
    assert!(Config::default().options().learning);
    let p = Config::parse("buuz_config = 1\n[options]\nlearning = false\n").unwrap();
    assert_eq!(p.config.learning(), Some(false));
    assert!(!p.config.options().learning);
    assert!(p.config.to_text().contains("learning = false"));
    let mut c = Config::default();
    c.set_learning(Some(true));
    assert_eq!(Config::parse(&c.to_text()).unwrap().config, c);
    assert!(
        Config::parse("buuz_config = 1\n[options]\nlearning = true\nlearning = true\n").is_err()
    );
}

#[test]
fn the_dictionary_is_not_in_the_config_text() {
    let c = Config::default();
    assert!(!c.to_text().contains("add\t"));
}

// ---- cost ----

/// Allocations for typing `words` after a warm-up, key by key. Every
/// allocation of a key is counted, and a commit key is not counted.
fn key_allocs(opts: Options, words: &[&str]) -> usize {
    let mut c = Composer::with_options(opts);
    for _ in 0..3 {
        for w in words {
            type_str(&mut c, w);
            c.process_key(Key::Space);
        }
    }
    let mut total = 0;
    for w in words {
        for ch in w.chars() {
            let before = allocs();
            let r = c.process_key(Key::Char(ch));
            total += allocs() - before;
            assert_eq!(r.learned, None);
        }
        c.process_key(Key::Space);
    }
    total
}

#[test]
fn the_allocation_counter_counts() {
    let before = allocs();
    let v = std::hint::black_box(vec![1u8; 100]);
    assert!(allocs() > before);
    drop(v);
}

#[test]
fn learning_and_a_user_dictionary_add_no_allocation_per_key() {
    let words = ["sain", "baina", "huuhed", "bayarlalaa", "tengis"];
    let mut d = UserDict::new();
    for i in 0..200 {
        d.add(&spread_word(i));
    }
    d.add("тэнгис");
    let base = key_allocs(
        Options {
            learning: false,
            user: None,
            ..opts(None)
        },
        &words,
    );
    let with = key_allocs(opts(Some(d)), &words);
    eprintln!("allocations over all keys: baseline {base}, with user dictionary {with}");
    assert!(
        with <= base,
        "user dictionary and learning cost {} extra allocations",
        with - base
    );
}

#[test]
fn completion_with_a_full_dictionary_scans_a_small_range_and_is_fast() {
    let mut d = UserDict::new();
    for i in 0..MAX_USERDICT_ENTRIES {
        d.add(&spread_word(i));
    }
    d.add("тэнгис");
    let snap = d.snapshot();
    assert!(snap.candidates("тэн") < 50);
    let mut c = Composer::with_options(Options {
        user: Some(Arc::new(snap)),
        ..opts(None)
    });
    let mut worst = 0u128;
    for _ in 0..50 {
        for ch in "tengis".chars() {
            let start = Instant::now();
            c.process_key(Key::Char(ch));
            std::hint::black_box(c.completion());
            worst = worst.max(start.elapsed().as_nanos());
        }
        c.process_key(Key::Space);
    }
    eprintln!("full user dictionary: worst {worst} ns");
    // A generous bound in debug, the completion budget in release.
    let limit = if cfg!(debug_assertions) {
        5_000_000
    } else {
        10_000
    };
    assert!(worst < limit, "worst {worst} ns");
}
