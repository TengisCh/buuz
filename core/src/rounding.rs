// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. See the LICENSE file in this crate.

//! Rounding harmony (issue #141). In a word whose first vowel is `о` or `ё`,
//! the word list may change an `а` after the first syllable to `о` (`болсан`
//! gives `болсон`). After a first `ө` it may change an `э` to `ө` (`өгсэн`
//! gives `өгсөн`). It runs once per word, when Space or punctuation ends it,
//! never per key. The buffers are reused, so a choice allocates only its
//! result.

use std::collections::HashSet;

use crate::autocorrect::{clear_winner, visit, Scratch, Tally, MAX_LETTERS, MIN_LETTERS};
use crate::rules::to_lower;
use crate::userdict::UserSnapshot;
use crate::wordlist::WordList;

/// The most letters of a word that a choice may change. Each combination is
/// a candidate, so 6 letters give at most 63 lookups.
const MAX_POSITIONS: usize = 6;

fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'а' | 'э' | 'и' | 'о' | 'у' | 'ү' | 'ө' | 'е' | 'ё' | 'ю' | 'я' | 'ы'
    )
}

/// Whether `word` ends with `tail`.
fn ends_with(word: &[char], tail: &[char]) -> bool {
    tail.len() <= word.len() && word[word.len() - tail.len()..] == *tail
}

/// The word with the rounding of the word list applied, for `comp` (the
/// preedit), or `None` to leave it as typed. `rejected` holds lower-case
/// words the user undid in this session; `user` is the personal dictionary
/// snapshot.
pub(crate) fn choose(
    words: &WordList,
    comp: &[char],
    rejected: &HashSet<String>,
    user: Option<&UserSnapshot>,
    s: &mut Scratch,
) -> Option<String> {
    let n = comp.len();
    if !(MIN_LETTERS..=MAX_LETTERS).contains(&n) || comp[1..].iter().any(|c| c.is_uppercase()) {
        return None;
    }
    s.word.clear();
    s.word.extend(comp.iter().map(|&c| to_lower(c)));

    // The letters that may change, and the letter they become.
    let first = s.word.iter().position(|&c| is_vowel(c))?;
    let (source, target) = match s.word[first] {
        'о' | 'ё' => ('а', 'о'),
        'ө' => ('э', 'ө'),
        _ => return None,
    };
    let word: &[char] = &s.word;
    let mut start = first + 1;
    while start < n && matches!(word[start], 'о' | 'ё' | 'ө') {
        start += 1;
    }
    let mut positions = [0usize; MAX_POSITIONS];
    let mut count = 0;
    for (i, &c) in word.iter().enumerate().skip(start) {
        if matches!(c, 'у' | 'ү' | 'ю') {
            break;
        }
        if c != source {
            continue;
        }
        if target == 'ө' {
            // The final `жээ`, `чээ` and `тэй` are suffixes of their own.
            let tail_cee = ends_with(word, &['ж', 'э', 'э']) || ends_with(word, &['ч', 'э', 'э']);
            if (tail_cee && i + 2 >= n) || (ends_with(word, &['т', 'э', 'й']) && i + 2 == n) {
                continue;
            }
        }
        if count == MAX_POSITIONS {
            return None;
        }
        positions[count] = i;
        count += 1;
    }
    if count == 0 {
        return None;
    }

    s.text.clear();
    s.text.extend(word.iter());
    if rejected.contains(s.text.as_str())
        || user.is_some_and(|u| u.is_added(&s.text) || u.is_rejected(&s.text))
    {
        return None;
    }
    let plain = words.count(&s.text).unwrap_or(0);

    let Scratch {
        word,
        level,
        best,
        text,
        ..
    } = s;
    let [cand, _] = level;
    best.clear();
    let mut tally = Tally { top: 0, second: 0 };
    for mask in 1u32..(1 << count) {
        cand.clear();
        cand.extend_from_slice(word);
        for (k, &p) in positions[..count].iter().enumerate() {
            if mask & (1 << k) != 0 {
                cand[p] = target;
            }
        }
        visit(words, cand, text, best, &mut tally);
    }
    if tally.top == 0 || !clear_winner(tally.top, tally.second, plain) {
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
