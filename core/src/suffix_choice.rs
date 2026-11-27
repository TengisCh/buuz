// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. See the LICENSE file in this crate.

//! Suffix choice (issue #52). The word list picks the case suffix of a word:
//! genitive, accusative, ablative and comitative (`hotaas` gives `хотоос`,
//! `usaas` gives `уснаас`). It runs once per word, when Space or punctuation
//! ends it, never per key. The buffers are reused, so a choice allocates only
//! its result.
//!
//! Step 1 reads the word as a stem and a suffix, and asks the list for the
//! stem with each other suffix of the same case. Step 2 runs when the list
//! knows none of them: it infers a hidden `н` or an inserted `г` from other
//! listed forms of the stem.

use std::collections::HashSet;

use crate::autocorrect::{
    clear_winner, visit, Scratch, Tally, MAX_LETTERS, MIN_COUNT, MIN_LETTERS,
};
use crate::rules::to_lower;
use crate::userdict::UserSnapshot;
use crate::wordlist::WordList;

/// A typed word that is listed with this count or more always stays.
const STAY_COUNT: u32 = 50;

/// Evidence of one stem type must be this many times the other type.
const TYPE_FACTOR: u32 = 3;

/// The shortest stem, in letters.
const MIN_STEM: usize = 2;

/// How the suffix joins the stem.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// After a consonant: `хот` + `оос`.
    Plain,
    /// With a hidden `н`: `ус` + `ны`, `ус` + `наас`.
    N,
    /// With an inserted `г`: `баа` + `гийн`, `баа` + `гаас`.
    G,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Genitive,
    Accusative,
    Ablative,
    Comitative,
}

struct Variant {
    text: &'static str,
    kind: Kind,
}

const fn var(text: &'static str, kind: Kind) -> Variant {
    Variant { text, kind }
}

const GENITIVE: &[Variant] = &[
    var("ийн", Kind::Plain),
    var("ын", Kind::Plain),
    var("ний", Kind::N),
    var("ны", Kind::N),
    var("н", Kind::N),
    var("гийн", Kind::G),
];

const ACCUSATIVE: &[Variant] = &[
    var("ийг", Kind::Plain),
    var("ыг", Kind::Plain),
    var("гийг", Kind::G),
    var("г", Kind::G),
];

const ABLATIVE: &[Variant] = &[
    var("аас", Kind::Plain),
    var("оос", Kind::Plain),
    var("ээс", Kind::Plain),
    var("өөс", Kind::Plain),
    var("наас", Kind::N),
    var("ноос", Kind::N),
    var("нээс", Kind::N),
    var("нөөс", Kind::N),
    var("гаас", Kind::G),
    var("гоос", Kind::G),
    var("гээс", Kind::G),
    var("гөөс", Kind::G),
];

const COMITATIVE: &[Variant] = &[
    var("тай", Kind::Plain),
    var("той", Kind::Plain),
    var("тэй", Kind::Plain),
];

const FAMILIES: [(Family, &[Variant]); 4] = [
    (Family::Genitive, GENITIVE),
    (Family::Accusative, ACCUSATIVE),
    (Family::Ablative, ABLATIVE),
    (Family::Comitative, COMITATIVE),
];

/// Evidence forms of a stem with a hidden `н`, and with an inserted `г`.
const N_FORMS: [&str; 6] = ["ны", "ний", "наас", "ноос", "нээс", "нөөс"];
const G_FORMS: [&str; 6] = ["гийн", "гийг", "гаас", "гоос", "гээс", "гөөс"];

fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'а' | 'э' | 'и' | 'о' | 'у' | 'ү' | 'ө' | 'е' | 'ё' | 'ю' | 'я' | 'ы'
    )
}

/// The letters of a long vowel or a diphthong: not я, е, ё, ю, which are
/// two sounds (`ая` is not long).
fn is_core_vowel(c: char) -> bool {
    matches!(c, 'а' | 'э' | 'и' | 'о' | 'у' | 'ү' | 'ө' | 'ы')
}

/// Whether the stem ends in a long vowel, a diphthong, or a vowel and й
/// (`тоо`, `ширээ`, `ай`). Only then can a `г` or `н` be an inserted letter:
/// after other stems it is part of the stem (`зарлаг`, `валентин`).
fn long_end(stem: &[char]) -> bool {
    match stem {
        [.., a, b] => is_core_vowel(*a) && (is_core_vowel(*b) || *b == 'й'),
        _ => false,
    }
}

fn one_letter(v: &Variant) -> bool {
    v.text.chars().count() == 1
}

/// Whether `word` ends with `tail`.
fn ends_with(word: &[char], tail: &str) -> bool {
    let n = tail.chars().count();
    n <= word.len() && word[word.len() - n..].iter().copied().eq(tail.chars())
}

/// Look up `stem` + `tail` and return its count, or 0.
fn count_of(words: &WordList, stem: &[char], tail: &str, text: &mut String) -> u32 {
    text.clear();
    text.extend(stem.iter());
    text.push_str(tail);
    words.count(text).unwrap_or(0)
}

/// The best count of a stem with each of `forms` (a form that cannot follow
/// the stem, `skip`, counts 0).
fn best_of(
    words: &WordList,
    stem: &[char],
    forms: &[&str],
    skip: &[&str],
    text: &mut String,
) -> u32 {
    forms
        .iter()
        .filter(|f| !skip.contains(f))
        .map(|f| count_of(words, stem, f, text))
        .max()
        .unwrap_or(0)
}

/// The raw keys at the end of the word that decide whether the suffix was
/// forced: `y` and `iii` give ы, `ii'` gives ий (section 4.2).
const RAW_TAIL: usize = 5;

/// Whether the raw keys of the word end in a key that forces the ы or ий of
/// the suffix.
fn forced_by_keys(raw: &[char]) -> bool {
    let tail = &raw[raw.len().saturating_sub(RAW_TAIL)..];
    let has = |p: [char; 3]| {
        tail.windows(3)
            .any(|w| w.iter().zip(p).all(|(&a, b)| to_lower(a) == b))
    };
    tail.iter().any(|&c| to_lower(c) == 'y') || has(['i', 'i', 'i']) || has(['i', 'i', '\''])
}

/// The word with its suffix changed, for `comp` (the preedit) and its raw
/// keys `raw`, or `None` to leave it as typed. A word typed with a forcing
/// key (`y`, `iii`, `ii'`) stays. `rejected` holds lower-case words the user undid in
/// this session; `user` is the personal dictionary snapshot.
pub(crate) fn choose(
    words: &WordList,
    comp: &[char],
    raw: &[char],
    rejected: &HashSet<String>,
    user: Option<&UserSnapshot>,
    s: &mut Scratch,
) -> Option<String> {
    let n = comp.len();
    if !(MIN_LETTERS..=MAX_LETTERS).contains(&n) || comp[1..].iter().any(|c| c.is_uppercase()) {
        return None;
    }
    if forced_by_keys(raw) {
        return None;
    }
    s.word.clear();
    s.word.extend(comp.iter().map(|&c| to_lower(c)));
    s.text.clear();
    s.text.extend(s.word.iter());
    if rejected.contains(s.text.as_str())
        || user.is_some_and(|u| u.is_added(&s.text) || u.is_rejected(&s.text))
    {
        return None;
    }
    let plain = words.count(&s.text).unwrap_or(0);
    if plain >= STAY_COUNT {
        return None;
    }

    let Scratch {
        word,
        level,
        best,
        text,
        ..
    } = s;
    let [cand, _] = level;
    let word: &[char] = word;

    // Step 1: one tally over the other forms of every reading of the word.
    best.clear();
    let mut tally = Tally { top: 0, second: 0 };
    for (_, variants) in FAMILIES {
        for typed in variants {
            let Some(stem) = parse(word, typed) else {
                continue;
            };
            let long = long_end(stem);
            for other in variants {
                if other.text == typed.text
                    || ((other.kind == Kind::G || one_letter(other)) && !long)
                    || (one_letter(other) && plain > 0)
                    // A hidden н of a listed word is never taken out.
                    || (plain > 0 && typed.kind == Kind::N && other.kind == Kind::Plain)
                {
                    continue;
                }
                join(cand, stem, other.text);
                visit(words, cand, text, best, &mut tally);
            }
        }
    }
    if tally.top > 0 {
        if !clear_winner(tally.top, tally.second, plain) {
            return None;
        }
        return Some(output(comp, best));
    }
    if plain > 0 {
        return None;
    }

    // Step 2: no form is listed. Infer a hidden н or an inserted г.
    let mut found = false;
    for (family, variants) in FAMILIES {
        if family == Family::Comitative {
            continue;
        }
        for typed in variants {
            let Some(stem) = parse(word, typed) else {
                continue;
            };
            let Some(target) = inferred(words, stem, family, typed, text) else {
                continue;
            };
            join(cand, stem, target);
            if !found {
                found = true;
                best.clear();
                best.extend_from_slice(cand);
            } else if best.as_slice() != cand.as_slice() {
                // Two readings disagree: leave the word.
                return None;
            }
        }
    }
    found.then(|| output(comp, best))
}

/// The stem of `word` when it ends with `typed`. A one-letter variant is
/// never a typed suffix (a stem's own `н` or `г`), a stem has at least
/// `MIN_STEM` letters, and a `г` form needs a stem that ends in a long vowel.
fn parse<'a>(word: &'a [char], typed: &Variant) -> Option<&'a [char]> {
    if one_letter(typed) || !ends_with(word, typed.text) {
        return None;
    }
    let stem = &word[..word.len() - typed.text.chars().count()];
    if stem.len() < MIN_STEM || (typed.kind == Kind::G && !long_end(stem)) {
        return None;
    }
    Some(stem)
}

/// The suffix that step 2 picks for the stem, or `None`. The stem must be a
/// listed word. The evidence of a type is the best count of the stem with a form of that type. It needs
/// `MIN_COUNT` and `TYPE_FACTOR` times the other type. A form `S` + `г` (or
/// `S` + `н`) that is listed itself shows that the `г` (or `н`) belongs to the
/// stem, so the evidence of that type does not count (`багийн` is `баг` +
/// `ийн`).
fn inferred(
    words: &WordList,
    stem: &[char],
    family: Family,
    typed: &Variant,
    text: &mut String,
) -> Option<&'static str> {
    // Strip a suffix only from a word: `оюутны` is not `оюут` + `ны`.
    if count_of(words, stem, "", text) == 0 {
        return None;
    }
    let vowel_end = stem.last().copied().is_some_and(is_vowel);
    let own = |c: &str, text: &mut String| count_of(words, stem, c, text) > 0;
    let n_score = if own("н", text) {
        0
    } else {
        // `ны` and `ний` after a vowel are not evidence (a long vowel).
        let skip: &[&str] = if vowel_end { &["ны", "ний"] } else { &[] };
        best_of(words, stem, &N_FORMS, skip, text)
    };
    let g_score = if !long_end(stem) || own("г", text) {
        0
    } else {
        best_of(words, stem, &G_FORMS, &[], text)
    };
    let want = if n_score >= MIN_COUNT && n_score >= TYPE_FACTOR * g_score {
        Kind::N
    } else if g_score >= MIN_COUNT && g_score >= TYPE_FACTOR * n_score {
        Kind::G
    } else {
        return None;
    };
    if want == typed.kind {
        return None;
    }
    // The target keeps the typed vowels. A typed `гийн` has none, so the
    // last vowel of the stem (not и) decides: back gives `ны`, front `ний`.
    let back = |c: char| matches!(c, 'а' | 'о' | 'у' | 'ы' | 'я' | 'ё' | 'ю');
    let stem_back = stem
        .iter()
        .rev()
        .find(|&&c| is_vowel(c) && c != 'и')
        .copied()
        .is_some_and(back);
    let typed_back = typed.text.contains('ы');
    let vowels = typed.text.trim_start_matches(['н', 'г']);
    Some(match (family, want) {
        (Family::Genitive, Kind::N) => {
            let back = if typed.kind == Kind::G {
                stem_back
            } else {
                typed_back
            };
            if back {
                "ны"
            } else {
                "ний"
            }
        }
        (Family::Genitive, _) => "гийн",
        (Family::Accusative, Kind::G) => "гийг",
        (Family::Ablative, Kind::N) => match vowels {
            "аас" => "наас",
            "оос" => "ноос",
            "ээс" => "нээс",
            _ => "нөөс",
        },
        (Family::Ablative, _) => match vowels {
            "аас" => "гаас",
            "оос" => "гоос",
            "ээс" => "гээс",
            _ => "гөөс",
        },
        _ => return None,
    })
}

/// Build `stem` + `tail` in `out`.
fn join(out: &mut Vec<char>, stem: &[char], tail: &str) {
    out.clear();
    out.extend_from_slice(stem);
    out.extend(tail.chars());
}

/// The result text, with the leading capital of `comp` kept.
fn output(comp: &[char], best: &[char]) -> String {
    let mut out = String::with_capacity(best.len() * 2);
    for (i, &c) in best.iter().enumerate() {
        if i == 0 && comp[0].is_uppercase() {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
    }
    out
}
