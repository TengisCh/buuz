// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. See the LICENSE file in this crate.

//! Spelling autocorrect (issue #1, stage 2). It runs once per word, when
//! Space or punctuation ends it, never per key. The search makes Mongolian-
//! aware edits (at most 2) of the word and asks the word list for each one.
//! The buffers are reused, so a correction allocates only its result.

use std::collections::HashSet;

use crate::rules::to_lower;
use crate::wordlist::WordList;

/// The shortest word (in letters) that autocorrect looks at.
pub(crate) const MIN_LETTERS: usize = 3;

/// The longest word that autocorrect looks at. A longer word costs more edits
/// than the 1 ms budget allows. At 16 letters the worst case is about 0.4 to
/// 0.8 ms in release (see the speed test).
pub(crate) const MAX_LETTERS: usize = 16;

/// The smallest count a winner needs.
const MIN_COUNT: u32 = 50;

/// The winner must be this many times more frequent than the next candidate.
const CLEAR_FACTOR: u32 = 3;

/// The most words in the rejected set.
pub(crate) const MAX_REJECTED: usize = 256;

/// Reused buffers of the search.
#[derive(Debug, Clone, Default)]
pub(crate) struct Scratch {
    /// The word in lower case.
    word: Vec<char>,
    /// One edit buffer for each level of edits.
    level: [Vec<char>; 2],
    /// The best candidate so far.
    best: Vec<char>,
    /// The candidate as text, for the lookup.
    text: String,
}

/// The best candidate so far and the count of the next one.
struct Tally {
    top: u32,
    second: u32,
}

/// The corrected word for `comp`, or `None` to leave it as typed. `comp` is
/// the preedit. `rejected` holds lower-case words the user undid.
pub(crate) fn correct(
    words: &WordList,
    comp: &[char],
    rejected: &HashSet<String>,
    s: &mut Scratch,
) -> Option<String> {
    let n = comp.len();
    if !(MIN_LETTERS..=MAX_LETTERS).contains(&n) {
        return None;
    }
    // A capital after the first letter (also an all-upper-case word) is
    // not a slip we can judge.
    if comp[1..].iter().any(|c| c.is_uppercase()) {
        return None;
    }
    s.word.clear();
    s.word.extend(comp.iter().map(|&c| to_lower(c)));
    s.text.clear();
    s.text.extend(s.word.iter());
    if words.contains(&s.text) || rejected.contains(s.text.as_str()) {
        return None;
    }

    let Scratch {
        word,
        level,
        best,
        text,
    } = s;
    let [first, second] = level;
    best.clear();
    let mut tally = Tally { top: 0, second: 0 };
    edits(word, first, &mut |c1| {
        visit(words, c1, text, best, &mut tally);
        edits(c1, second, &mut |c2| {
            visit(words, c2, text, best, &mut tally)
        });
    });

    let clear = tally.top >= MIN_COUNT
        && u64::from(tally.top) >= u64::from(CLEAR_FACTOR) * u64::from(tally.second);
    if !clear {
        return None;
    }
    let mut out = String::with_capacity(best.len() * 2);
    for (i, &c) in best.iter().enumerate() {
        if i == 0 && comp[0].is_uppercase() {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// Look `cand` up and keep the best two distinct candidates.
fn visit(words: &WordList, cand: &[char], text: &mut String, best: &mut Vec<char>, t: &mut Tally) {
    text.clear();
    text.extend(cand.iter());
    let Some(count) = words.count(text) else {
        return;
    };
    if count > t.top {
        t.second = t.top;
        t.top = count;
        best.clear();
        best.extend_from_slice(cand);
    } else if count > t.second && cand != best.as_slice() {
        t.second = count;
    }
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'а' | 'э' | 'и' | 'о' | 'у' | 'ү' | 'ө')
}

/// The partner of a letter in the pairs that are easy to confuse.
fn swap(c: char) -> Option<char> {
    Some(match c {
        'о' => 'ө',
        'ө' => 'о',
        'у' => 'ү',
        'ү' => 'у',
        'э' => 'е',
        'е' => 'э',
        _ => return None,
    })
}

/// Call `f` with each single edit of `cur`, built in `out`.
fn edits(cur: &[char], out: &mut Vec<char>, f: &mut dyn FnMut(&[char])) {
    let n = cur.len();
    for i in 0..n {
        let c = cur[i];
        // The final vowel is a suffix or a case ending (идэе, шинээ): leave it.
        let last = i + 1 == n;
        if let Some(x) = swap(c).filter(|_| !(last && matches!(c, 'э' | 'е'))) {
            fill(out, &cur[..i], &[x], &cur[i + 1..]);
            f(out);
        }
        if c == 'и' && cur.get(i + 1) == Some(&'й') {
            fill(out, &cur[..i], &['ы'], &cur[i + 2..]);
            f(out);
        }
        if c == 'ы' {
            fill(out, &cur[..i], &['и', 'й'], &cur[i + 1..]);
            f(out);
        }
        if is_vowel(c) {
            if cur.get(i + 1) == Some(&c) {
                // Double to single, unless the pair ends the word.
                if i + 2 == n {
                    continue;
                }
                fill(out, &cur[..i], &[], &cur[i + 1..]);
                f(out);
            } else if (i == 0 || cur[i - 1] != c) && !last {
                // Single to double.
                fill(out, &cur[..=i], &[c], &cur[i + 1..]);
                f(out);
            }
        }
        if c == 'ь' {
            fill(out, &cur[..i], &[], &cur[i + 1..]);
            f(out);
        }
    }
    // A soft sign goes after the first letter at the earliest.
    for i in 1..=n {
        fill(out, &cur[..i], &['ь'], &cur[i..]);
        f(out);
    }
}

fn fill(out: &mut Vec<char>, a: &[char], b: &[char], c: &[char]) {
    out.clear();
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    out.extend_from_slice(c);
}
