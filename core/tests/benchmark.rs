// SPDX-License-Identifier: Apache-2.0

//! Typing-habit benchmark (issue #42).
//!
//! A person types Mongolian in Latin letters with their own habits (`kh` for
//! х, `u` for ө, ...). This file measures how well smart mode turns such
//! Latin back into the right Cyrillic word, and how many words a candidate
//! rule fixes and breaks.
//!
//! The pairs are synthetic. A generator turns a Cyrillic word of the word
//! list into the Latin text a profile would type. Each profile changes one
//! thing from the base spelling, so the effects stay apart. The pairs are
//! stored in `data/benchmark/habits.tsv`. Hand-written pairs are in
//! `data/benchmark/manual.tsv`.
//!
//! - Fast tests (always run): the stored file is current, and the base
//!   profile does not score below a floor.
//! - Report (ignored): `cargo test --test benchmark -- --ignored --nocapture`.
//! - Rewrite the file after a change to the generator:
//!   `BUUZ_BENCHMARK_WRITE=1 cargo test --test benchmark`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use buuz_core::{convert_with, KeyMap, Options, WordList};

const WORDLIST: &str = include_str!("../../data/wordlist/mn.txt");

/// How many of the most frequent words the sample comes from.
const TOP_WORDS: usize = 3000;
/// The most pairs one profile gets.
const MAX_PER_PROFILE: usize = 300;

// ---------------------------------------------------------------------------
// The habit generator
// ---------------------------------------------------------------------------

/// The Buuz table spelling of one Cyrillic letter (typing-rules section 3).
fn base_letter(c: char) -> Option<&'static str> {
    Some(match c {
        'а' => "a",
        'б' => "b",
        'в' => "v",
        'г' => "g",
        'д' => "d",
        'е' => "ye",
        'ё' => "yo",
        'ж' => "j",
        'з' => "z",
        'и' => "i",
        'й' => "i",
        'к' => "k",
        'л' => "l",
        'м' => "m",
        'н' => "n",
        'о' => "o",
        'ө' => "q",
        'п' => "p",
        'р' => "r",
        'с' => "s",
        'т' => "t",
        'у' => "u",
        'ү' => "w",
        'ф' => "f",
        'х' => "h",
        'ц' => "c",
        'ч' => "ch",
        'ш' => "sh",
        'щ' => "sxc",
        'ъ' => "\"",
        'ы' => "y",
        'ь' => "'",
        'э' => "e",
        'ю' => "yu",
        'я' => "ya",
        _ => return None,
    })
}

/// One typing habit: the base spelling with a few changes.
struct Profile {
    name: &'static str,
    /// Letter changes. An empty string drops the letter.
    over: &'static [(char, &'static str)],
    /// How ий is typed. The base types `ii'`, which is always right.
    iy: &'static str,
    /// Collapse a doubled vowel (аа, оо, уу, үү, өө, ээ) to one.
    long_single: bool,
    /// A word joins the sample only when it holds one of these parts.
    /// Empty: every word joins.
    trigger: &'static [&'static str],
}

const P: Profile = Profile {
    name: "",
    over: &[],
    iy: "ii'",
    long_single: false,
    trigger: &[],
};

const LONG: &[&str] = &["аа", "оо", "уу", "үү", "өө", "ээ"];

const PROFILES: &[Profile] = &[
    Profile { name: "base", ..P },
    Profile {
        name: "kh",
        over: &[('х', "kh")],
        trigger: &["х"],
        ..P
    },
    Profile {
        name: "x_for_h",
        over: &[('х', "x")],
        trigger: &["х"],
        ..P
    },
    Profile {
        name: "ts",
        over: &[('ц', "ts")],
        trigger: &["ц"],
        ..P
    },
    Profile {
        name: "zh_for_ж",
        over: &[('ж', "zh")],
        trigger: &["ж"],
        ..P
    },
    Profile {
        name: "c_for_ч",
        over: &[('ч', "c")],
        trigger: &["ч"],
        ..P
    },
    Profile {
        name: "u_for_ү",
        over: &[('ү', "u")],
        trigger: &["ү"],
        ..P
    },
    Profile {
        name: "v_for_ү",
        over: &[('ү', "v")],
        trigger: &["ү"],
        ..P
    },
    Profile {
        name: "y_for_ү",
        over: &[('ү', "y")],
        trigger: &["ү"],
        ..P
    },
    Profile {
        name: "umlaut_ü",
        over: &[('ү', "ü")],
        trigger: &["ү"],
        ..P
    },
    Profile {
        name: "o_for_ө",
        over: &[('ө', "o")],
        trigger: &["ө"],
        ..P
    },
    Profile {
        name: "umlaut_ö",
        over: &[('ө', "ö")],
        trigger: &["ө"],
        ..P
    },
    Profile {
        name: "8_for_ө",
        over: &[('ө', "8")],
        trigger: &["ө"],
        ..P
    },
    Profile {
        name: "u_for_ө",
        over: &[('ө', "u")],
        trigger: &["ө"],
        ..P
    },
    Profile {
        name: "ia_for_я",
        over: &[('я', "ia")],
        trigger: &["я"],
        ..P
    },
    Profile {
        name: "iu_for_ю",
        over: &[('ю', "iu")],
        trigger: &["ю"],
        ..P
    },
    Profile {
        name: "e_for_е",
        over: &[('е', "e")],
        trigger: &["е"],
        ..P
    },
    Profile {
        name: "io_for_ё",
        over: &[('ё', "io")],
        trigger: &["ё"],
        ..P
    },
    Profile {
        name: "y_for_й",
        over: &[('й', "y")],
        trigger: &["ай", "ой", "уй", "үй", "эй", "өй"],
        ..P
    },
    Profile {
        name: "ii_for_ы",
        over: &[('ы', "ii")],
        trigger: &["ы"],
        ..P
    },
    Profile {
        name: "i_for_ы",
        over: &[('ы', "i")],
        trigger: &["ы"],
        ..P
    },
    Profile {
        name: "ii_for_ий",
        iy: "ii",
        trigger: &["ий"],
        ..P
    },
    Profile {
        name: "drop_soft_sign",
        over: &[('ь', "")],
        trigger: &["ь"],
        ..P
    },
    Profile {
        name: "i_for_soft_sign",
        over: &[('ь', "i")],
        trigger: &["ь"],
        ..P
    },
    Profile {
        name: "single_long_vowel",
        long_single: true,
        trigger: LONG,
        ..P
    },
    // A chat profile: h for х, ts for ц, u for ү, o for ө, ya and yu,
    // no ь, and ii for ий (nobody types the apostrophe).
    Profile {
        name: "mixed",
        over: &[('ц', "ts"), ('ү', "u"), ('ө', "o"), ('ь', "")],
        iy: "ii",
        ..P
    },
];

fn is_vowel(c: char) -> bool {
    "аоуүөэиыеёюя".contains(c)
}

/// The Latin a person with `p` habits types for `word`. `None` when the word
/// holds a letter the table does not cover.
fn latin(word: &str, p: &Profile) -> Option<String> {
    let mut chars: Vec<char> = word.chars().collect();
    if p.long_single {
        let mut out: Vec<char> = Vec::with_capacity(chars.len());
        for &c in &chars {
            if "аоуүөэ".contains(c) && out.last() == Some(&c) {
                continue;
            }
            out.push(c);
        }
        chars = out;
    }
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == 'и' && chars.get(i + 1) == Some(&'й') {
            out.push_str(p.iy);
            i += 2;
            continue;
        }
        let _ = base_letter(c)?;
        let after_vowel = i > 0 && is_vowel(chars[i - 1]);
        let typed = match p.over.iter().find(|(from, _)| *from == c) {
            // A й that is not after a vowel keeps the base spelling.
            Some((_, to)) if c != 'й' || after_vowel => to,
            _ => base_letter(c)?,
        };
        out.push_str(typed);
        i += 1;
    }
    Some(out)
}

/// The most frequent `TOP_WORDS` words, best rank first.
fn top_words() -> Vec<&'static str> {
    WORDLIST
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split('\t').next())
        .take(TOP_WORDS)
        .collect()
}

#[derive(Debug, Clone)]
struct Pair {
    profile: String,
    latin: String,
    cyrillic: String,
}

/// Every habit pair, in profile order and then rank order.
fn generate() -> Vec<Pair> {
    let words = top_words();
    let mut pairs = Vec::new();
    for p in PROFILES {
        let eligible: Vec<(&str, String)> = words
            .iter()
            .filter(|w| p.trigger.is_empty() || p.trigger.iter().any(|t| w.contains(t)))
            .filter_map(|w| latin(w, p).map(|l| (*w, l)))
            .collect();
        let step = (eligible.len() + MAX_PER_PROFILE - 1) / MAX_PER_PROFILE;
        for (w, l) in eligible.into_iter().step_by(step).take(MAX_PER_PROFILE) {
            pairs.push(Pair {
                profile: p.name.to_string(),
                latin: l,
                cyrillic: w.to_string(),
            });
        }
    }
    pairs
}

const HABITS_HEADER: &str = "\
# SPDX-License-Identifier: CC-BY-SA-4.0
# Synthetic typing-habit pairs derived from data/wordlist/mn.txt (Mongolian Wikipedia contributors, CC BY-SA 4.0). See data/benchmark/README.md.
# Generated by core/tests/benchmark.rs. Do not edit by hand.
# profile<TAB>latin<TAB>cyrillic
";

fn habits_text() -> String {
    let mut s = String::from(HABITS_HEADER);
    for p in generate() {
        let _ = writeln!(s, "{}\t{}\t{}", p.profile, p.latin, p.cyrillic);
    }
    s
}

// ---------------------------------------------------------------------------
// Files
// ---------------------------------------------------------------------------

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../data/benchmark")
        .join(name)
}

fn parse_pairs(text: &str) -> Vec<Pair> {
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let cols: Vec<&str> = l.split('\t').collect();
            assert_eq!(cols.len(), 3, "bad benchmark line: {l:?}");
            Pair {
                profile: cols[0].to_string(),
                latin: cols[1].to_string(),
                cyrillic: cols[2].to_string(),
            }
        })
        .collect()
}

fn read_pairs(name: &str) -> Vec<Pair> {
    let text = std::fs::read_to_string(data_path(name))
        .unwrap_or_else(|e| panic!("cannot read data/benchmark/{name}: {e}"));
    parse_pairs(&text)
}

/// Habit pairs, then hand-written pairs (their profile is `manual/<name>`).
fn all_pairs() -> Vec<Pair> {
    let mut pairs = read_pairs("habits.tsv");
    for mut p in read_pairs("manual.tsv") {
        p.profile = format!("manual/{}", p.profile);
        pairs.push(p);
    }
    pairs
}

// ---------------------------------------------------------------------------
// Conversion and scoring
// ---------------------------------------------------------------------------

fn full_list() -> Arc<WordList> {
    Arc::new(WordList::parse(WORDLIST).expect("word list parses"))
}

/// Convert one typed word. A Space ends it, as in real typing, so that
/// autocorrect can run. The Space is cut from the result.
fn conv(opts: &Options, latin: &str) -> String {
    let out = convert_with(&format!("{latin} "), opts);
    out.trim_end_matches(' ').to_string()
}

fn smart() -> Options {
    Options::smart()
}

fn classic() -> Options {
    Options::classic()
}

fn autocorrect(words: &Arc<WordList>) -> Options {
    Options {
        words: Some(words.clone()),
        autocorrect: true,
        ..Options::smart()
    }
}

fn smart_with_keys(pairs: &[(&str, &str)]) -> Options {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Options {
        keys: Some(Arc::new(
            KeyMap::from_pairs(&pairs).expect("keys are valid"),
        )),
        ..Options::smart()
    }
}

fn percent(n: usize, of: usize) -> f64 {
    if of == 0 {
        0.0
    } else {
        100.0 * n as f64 / of as f64
    }
}

fn score(pairs: &[&Pair], f: &dyn Fn(&str) -> String) -> usize {
    pairs.iter().filter(|p| f(&p.latin) == p.cyrillic).count()
}

fn by_profile(pairs: &[Pair]) -> Vec<(String, Vec<&Pair>)> {
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, Vec<&Pair>> = BTreeMap::new();
    for p in pairs {
        if !map.contains_key(&p.profile) {
            order.push(p.profile.clone());
        }
        map.entry(p.profile.clone()).or_default().push(p);
    }
    order
        .into_iter()
        .map(|n| {
            let v = map.remove(&n).unwrap();
            (n, v)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Fix and break harness
// ---------------------------------------------------------------------------

/// One pair whose result changed between two converters.
struct Change {
    pair: Pair,
    before: String,
    after: String,
}

/// Pairs the second converter gets right and the first gets wrong, and the
/// other way round.
#[derive(Default)]
struct Diff {
    fixed: Vec<Change>,
    broken: Vec<Change>,
    /// Per profile: pairs, right before, right after.
    totals: BTreeMap<String, (usize, usize, usize)>,
}

fn compare(
    pairs: &[Pair],
    before: &dyn Fn(&str) -> String,
    after: &dyn Fn(&str) -> String,
) -> Diff {
    let mut d = Diff::default();
    for p in pairs {
        let b = before(&p.latin);
        let a = after(&p.latin);
        let b_ok = b == p.cyrillic;
        let a_ok = a == p.cyrillic;
        let t = d.totals.entry(p.profile.clone()).or_default();
        t.0 += 1;
        t.1 += usize::from(b_ok);
        t.2 += usize::from(a_ok);
        let change = || Change {
            pair: p.clone(),
            before: b.clone(),
            after: a.clone(),
        };
        if !b_ok && a_ok {
            d.fixed.push(change());
        } else if b_ok && !a_ok {
            d.broken.push(change());
        }
    }
    d
}

fn count_in(changes: &[Change], profiles: &[&str]) -> usize {
    changes
        .iter()
        .filter(|c| profiles.contains(&c.pair.profile.as_str()))
        .count()
}

/// How a word-list estimate treats the plain result.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// Try the alternatives only when the plain result is not a listed word.
    Fallback,
    /// Take the most frequent listed word among the plain result and the
    /// alternatives.
    Frequency,
}

/// Word-list estimate of a rule that the core cannot express today: make
/// alternative Latin spellings, convert each, and keep a listed word.
fn decode(
    opts: &Options,
    words: &WordList,
    mode: Mode,
    alts: &dyn Fn(&str) -> Vec<String>,
    latin: &str,
) -> String {
    let plain = conv(opts, latin);
    if mode == Mode::Fallback && words.contains(&plain) {
        return plain;
    }
    let mut best = (words.count(&plain), plain);
    for a in alts(latin) {
        let out = conv(opts, &a);
        if let Some(c) = words.count(&out) {
            if best.0.map_or(true, |b| c > b) {
                best = (Some(c), out);
            }
        }
    }
    best.1
}

/// Every spelling where each occurrence of a `from` letter may stay or become
/// one of the `to` letters. At most 6 occurrences change (the rest stay).
fn choices(latin: &str, rules: &[(char, &[char])]) -> Vec<String> {
    let chars: Vec<char> = latin.chars().collect();
    let slots: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, c)| rules.iter().any(|(f, _)| f == *c))
        .map(|(i, _)| i)
        .take(6)
        .collect();
    let options = |i: usize| -> Vec<char> {
        let mut o = vec![chars[i]];
        for (f, to) in rules {
            if *f == chars[i] {
                o.extend(to.iter());
            }
        }
        o
    };
    let mut results: Vec<Vec<char>> = vec![chars.clone()];
    for &i in &slots {
        let mut next = Vec::new();
        for r in &results {
            for o in options(i) {
                let mut r2 = r.clone();
                r2[i] = o;
                next.push(r2);
            }
        }
        results = next;
    }
    results
        .into_iter()
        .map(|r| r.into_iter().collect::<String>())
        .filter(|s| s != latin)
        .collect()
}

/// Every spelling with one `'` (ь) added after a consonant letter.
fn add_soft_sign(latin: &str) -> Vec<String> {
    let chars: Vec<char> = latin.chars().collect();
    let mut out = Vec::new();
    for i in 0..chars.len() {
        // `'o` and `'u` are explicit keys (ө, ү), so do not add before them.
        let next = chars.get(i + 1);
        if "bcdfghjklmnprstvxz".contains(chars[i]) && !matches!(next, Some('\'' | 'o' | 'u')) {
            let mut s: String = chars[..=i].iter().collect();
            s.push('\'');
            s.extend(chars[i + 1..].iter());
            out.push(s);
        }
    }
    out
}

/// Every spelling with one vowel letter doubled (long vowel).
fn double_a_vowel(latin: &str) -> Vec<String> {
    let chars: Vec<char> = latin.chars().collect();
    let mut out = Vec::new();
    for i in 0..chars.len() {
        if "aouweqü".contains(chars[i]) {
            let mut s: String = chars[..=i].iter().collect();
            s.push(chars[i]);
            s.extend(chars[i + 1..].iter());
            out.push(s);
        }
    }
    out
}

fn examples(changes: &[Change], n: usize) -> impl Iterator<Item = &Change> {
    changes.iter().take(n)
}

fn print_diff(name: &str, estimate: bool, targets: &[&str], pairs: &[Pair], d: &Diff) {
    let tag = if estimate { " [estimate]" } else { "" };
    let on_base = |c: &[Change]| count_in(c, &["base"]);
    let (n, b, a) = targets
        .iter()
        .filter_map(|t| d.totals.get(*t))
        .fold((0, 0, 0), |x, t| (x.0 + t.0, x.1 + t.1, x.2 + t.2));
    println!(
        "{name}{tag}: target {n} pairs, right {b} -> {a} ({:.1}% -> {:.1}%); all {} pairs: fixed {}, broken {} (base {})",
        percent(b, n),
        percent(a, n),
        pairs.len(),
        d.fixed.len(),
        d.broken.len(),
        on_base(&d.broken),
    );
    for c in examples(&d.fixed, 3) {
        println!(
            "    fixed  [{}] {}: {} -> {}",
            c.pair.profile, c.pair.latin, c.before, c.after
        );
    }
    for c in examples(&d.broken, 3) {
        println!(
            "    broken [{}] {}: {} -> {} (want {})",
            c.pair.profile, c.pair.latin, c.before, c.after, c.pair.cyrillic
        );
    }
}

// ---------------------------------------------------------------------------
// Fast tests
// ---------------------------------------------------------------------------

/// The base profile types exactly what the Buuz table says, so smart mode
/// must get nearly all of it right. Measured 2026-11: see the report.
const BASE_SMART_FLOOR: f64 = 97.0;

#[test]
fn habits_file_is_current() {
    let want = habits_text();
    let path = data_path("habits.tsv");
    if std::env::var_os("BUUZ_BENCHMARK_WRITE").is_some() {
        std::fs::write(&path, &want).expect("write habits.tsv");
        return;
    }
    let have = std::fs::read_to_string(&path).expect("read habits.tsv");
    assert!(
        have == want,
        "data/benchmark/habits.tsv is out of date: run `BUUZ_BENCHMARK_WRITE=1 cargo test --test benchmark`"
    );
}

#[test]
fn files_parse_and_cover_every_profile() {
    let pairs = all_pairs();
    for p in &pairs {
        assert!(!p.latin.is_empty() && !p.cyrillic.is_empty());
        assert!(!p.latin.contains(' ') && !p.cyrillic.contains(' '));
    }
    let groups = by_profile(&pairs);
    for p in PROFILES {
        assert!(
            groups.iter().any(|(n, v)| n == p.name && !v.is_empty()),
            "profile {} has no pairs",
            p.name
        );
    }
    let manual = read_pairs("manual.tsv");
    assert!(manual
        .iter()
        .any(|p| p.latin == "uvul" && p.cyrillic == "өвөл"));
}

#[test]
fn base_profile_stays_above_the_floor() {
    let pairs = read_pairs("habits.tsv");
    let base: Vec<&Pair> = pairs.iter().filter(|p| p.profile == "base").collect();
    assert!(base.len() >= MAX_PER_PROFILE);
    let opts = smart();
    let ok = score(&base, &|l| conv(&opts, l));
    let pct = percent(ok, base.len());
    assert!(
        pct >= BASE_SMART_FLOOR,
        "base profile in smart mode fell to {pct:.1}% (floor {BASE_SMART_FLOOR}%)"
    );
}

// ---------------------------------------------------------------------------
// Report (ignored)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "report: cargo test --test benchmark -- --ignored --nocapture"]
fn report() {
    let pairs = all_pairs();
    let words = full_list();
    let (sm, cl, ac) = (smart(), classic(), autocorrect(&words));
    println!();
    println!("profile                words   smart  classic  autocorrect");
    let mut totals = (0, 0, 0, 0);
    let groups = by_profile(&pairs);
    for (name, ps) in &groups {
        let s = score(ps, &|l| conv(&sm, l));
        let c = score(ps, &|l| conv(&cl, l));
        let a = score(ps, &|l| conv(&ac, l));
        println!(
            "{name:<22} {:>5}  {:>5.1}%  {:>6.1}%  {:>10.1}%",
            ps.len(),
            percent(s, ps.len()),
            percent(c, ps.len()),
            percent(a, ps.len())
        );
        totals = (
            totals.0 + ps.len(),
            totals.1 + s,
            totals.2 + c,
            totals.3 + a,
        );
    }
    println!(
        "{:<22} {:>5}  {:>5.1}%  {:>6.1}%  {:>10.1}%",
        "all",
        totals.0,
        percent(totals.1, totals.0),
        percent(totals.2, totals.0),
        percent(totals.3, totals.0)
    );
    println!();
    println!("Wrong outputs in smart mode (first 5 by rank): latin -> got (want)");
    for (name, ps) in &groups {
        let wrong: Vec<&&Pair> = ps
            .iter()
            .filter(|p| conv(&sm, &p.latin) != p.cyrillic)
            .collect();
        if wrong.is_empty() {
            continue;
        }
        println!("  {name} ({} wrong)", wrong.len());
        for p in wrong.iter().take(5) {
            println!(
                "    {} -> {} (want {})",
                p.latin,
                conv(&sm, &p.latin),
                p.cyrillic
            );
        }
    }
}

#[test]
#[ignore = "report: cargo test --test benchmark -- --ignored --nocapture"]
fn candidate_rules() {
    let pairs = all_pairs();
    let words = full_list();
    let sm = smart();
    let before = |l: &str| conv(&sm, l);
    println!();
    println!(
        "Candidate rules against smart mode (before). Over {} pairs.",
        pairs.len()
    );
    println!("Key map rules (exact: the core can do this today):");

    type KeyRule<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a [&'a str]);
    let key_rules: &[KeyRule] = &[
        ("keys ts=ц", &[("ts", "ц")], &["ts"]),
        ("keys zh=ж", &[("zh", "ж")], &["zh_for_ж"]),
        ("keys c=ч", &[("c", "ч")], &["c_for_ч"]),
        ("keys ia=я", &[("ia", "я")], &["ia_for_я"]),
        ("keys iu=ю", &[("iu", "ю")], &["iu_for_ю"]),
        ("keys io=ё", &[("io", "ё")], &["io_for_ё"]),
        ("keys v=ү", &[("v", "ү")], &["v_for_ү"]),
        ("keys y=ү", &[("y", "ү")], &["y_for_ү"]),
    ];
    for (name, keys, targets) in key_rules {
        let o = smart_with_keys(keys);
        let d = compare(&pairs, &before, &|l| conv(&o, l));
        print_diff(name, false, targets, &pairs, &d);
    }

    println!("Latin pre-processing (estimates):");
    let pre: &[(&str, &str, &str, &[&str])] = &[("pre 8 -> q", "8", "q", &["8_for_ө"])];
    for (name, from, to, targets) in pre {
        let d = compare(&pairs, &before, &|l| conv(&sm, &l.replace(from, to)));
        print_diff(name, true, targets, &pairs, &d);
    }

    println!("Word-list rules (estimates; the sample words come from the same list):");
    let uo: &[(char, &[char])] = &[('u', &['q'])];
    let uwq: &[(char, &[char])] = &[('u', &['w', 'q'])];
    let oq: &[(char, &[char])] = &[('o', &['q'])];
    let chat: &[(char, &[char])] = &[('u', &['w', 'q']), ('o', &['q'])];
    type Alts<'a> = Box<dyn Fn(&str) -> Vec<String> + 'a>;
    let rules: Vec<(&str, Mode, Alts, &[&str])> = vec![
        (
            "words u -> ө (best by count)",
            Mode::Frequency,
            Box::new(|l| choices(l, uo)),
            &["u_for_ө", "manual/u_for_ө"],
        ),
        (
            "words u -> ө (only if the plain word is unlisted)",
            Mode::Fallback,
            Box::new(|l| choices(l, uo)),
            &["u_for_ө", "manual/u_for_ө"],
        ),
        (
            "words u -> ү/ө (best by count)",
            Mode::Frequency,
            Box::new(|l| choices(l, uwq)),
            &["u_for_ү", "u_for_ө", "manual/u_for_ө"],
        ),
        (
            "words o -> ө (best by count)",
            Mode::Frequency,
            Box::new(|l| choices(l, oq)),
            &["o_for_ө"],
        ),
        (
            "words chat: u -> ү/ө, o -> ө (best by count)",
            Mode::Frequency,
            Box::new(|l| choices(l, chat)),
            &["mixed"],
        ),
        (
            "words add ь (only if the plain word is unlisted)",
            Mode::Fallback,
            Box::new(add_soft_sign),
            &["drop_soft_sign"],
        ),
        (
            "words i -> ь (only if the plain word is unlisted)",
            Mode::Fallback,
            Box::new(|l| {
                let chars: Vec<char> = l.chars().collect();
                (0..chars.len())
                    .filter(|&i| chars[i] == 'i')
                    .map(|i| {
                        let mut c = chars.clone();
                        c[i] = '\'';
                        c.into_iter().collect()
                    })
                    .collect()
            }),
            &["i_for_soft_sign"],
        ),
        (
            "words e -> ye (only if the plain word is unlisted)",
            Mode::Fallback,
            Box::new(|l| {
                let chars: Vec<char> = l.chars().collect();
                (0..chars.len())
                    .filter(|&i| chars[i] == 'e')
                    .map(|i| {
                        let mut s: String = chars[..i].iter().collect();
                        s.push_str("ye");
                        s.extend(chars[i + 1..].iter());
                        s
                    })
                    .collect()
            }),
            &["e_for_е"],
        ),
        (
            "words double a vowel (only if the plain word is unlisted)",
            Mode::Fallback,
            Box::new(double_a_vowel),
            &["single_long_vowel"],
        ),
    ];
    for (name, mode, alts, targets) in &rules {
        let d = compare(&pairs, &before, &|l| {
            decode(&sm, &words, *mode, alts.as_ref(), l)
        });
        print_diff(name, true, targets, &pairs, &d);
    }
}
