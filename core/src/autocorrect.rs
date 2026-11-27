// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. See the LICENSE file in this crate.

//! Spelling autocorrect (issue #1, stage 2). It runs once per word, when
//! Space or punctuation ends it, never per key. The search makes Mongolian-
//! aware edits (at most 2) of the word and asks the word list for each one.
//! The buffers are reused, so a correction allocates only its result.

use std::collections::HashSet;

use crate::rules::to_lower;
use crate::userdict::UserSnapshot;
use crate::wordlist::WordList;

/// The shortest word (in letters) that autocorrect looks at.
pub(crate) const MIN_LETTERS: usize = 3;

/// The longest word that autocorrect looks at. A longer word costs more edits
/// than the 1 ms budget allows. At 16 letters the worst case is about 0.4 to
/// 0.8 ms in release (see the speed test).
pub(crate) const MAX_LETTERS: usize = 16;

/// The smallest count a winner needs.
pub(crate) const MIN_COUNT: u32 = 50;

/// The winner must be this many times more frequent than the next candidate.
const CLEAR_FACTOR: u32 = 3;

/// The most words in the rejected set.
pub(crate) const MAX_REJECTED: usize = 256;

/// Reused buffers of the search.
#[derive(Debug, Clone, Default)]
pub(crate) struct Scratch {
    /// The word in lower case.
    pub(crate) word: Vec<char>,
    /// One edit buffer for each level of edits.
    pub(crate) level: [Vec<char>; 2],
    /// The best candidate so far.
    pub(crate) best: Vec<char>,
    /// The candidate as text, for the lookup.
    pub(crate) text: String,
    /// The candidate the user used most, for a tie.
    liked: Vec<char>,
}

/// The best candidate so far and the count of the next one.
pub(crate) struct Tally {
    pub(crate) top: u32,
    pub(crate) second: u32,
}

/// The corrected word for `comp`, or `None` to leave it as typed. `comp` is
/// the preedit. `rejected` holds lower-case words the user undid in this
/// session; `user` is the personal dictionary snapshot.
pub(crate) fn correct(
    words: &WordList,
    comp: &[char],
    rejected: &HashSet<String>,
    user: Option<&UserSnapshot>,
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
    // A word the user added or rejected is never corrected.
    let user_knows = user.is_some_and(|u| u.is_added(&s.text) || u.is_rejected(&s.text));
    if words.contains(&s.text) || rejected.contains(s.text.as_str()) || user_knows {
        return None;
    }

    let Scratch {
        word,
        level,
        best,
        text,
        ..
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

/// A listed variant must be this many times more frequent than the word as
/// typed when that word is listed too (`мөн` 20067 against `мон` 114; but
/// `өөрийн` is only 12 times `уурын`, so `uuriin` stays).
const PLAIN_FACTOR: u32 = 50;

/// Whether the best candidate wins: it has the minimum count, is
/// `CLEAR_FACTOR` times the next candidate, and is `PLAIN_FACTOR` times the
/// word as typed (`plain`, 0 if unlisted). Word choice and suffix choice
/// share this rule.
pub(crate) fn clear_winner(top: u32, second: u32, plain: u32) -> bool {
    top >= MIN_COUNT
        && u64::from(top) >= u64::from(CLEAR_FACTOR) * u64::from(second)
        && u64::from(top) >= u64::from(PLAIN_FACTOR) * u64::from(plain)
}

/// The most letters in a word that `choose` varies. Each `у` or `ү` has 3
/// choices, each `о` has 2, so 5 letters give at most 243 lookups.
const MAX_VARIED: usize = 5;

/// A word with more varied letters than this gets no extra edits (ye, long
/// vowel, ь), so the lookups stay under about 1500.
const MAX_VARIED_EXTRA: usize = 3;

/// Word choice (issue #64): the word list picks between о/ө and у/ү/ө.
/// It runs once per word, like [`correct`]. A variant wins when it is listed
/// with the minimum count, is `CLEAR_FACTOR` times more frequent than the
/// next variant, and is `PLAIN_FACTOR` times more frequent than the word as
/// typed (which counts 0 when unlisted).
pub(crate) fn choose(
    words: &WordList,
    comp: &[char],
    rejected: &HashSet<String>,
    user: Option<&UserSnapshot>,
    s: &mut Scratch,
) -> Option<String> {
    let n = comp.len();
    // A word in all capitals (UVUL) is chosen too; other mixed case stays.
    let caps = n > 1 && comp.iter().all(|c| !c.is_lowercase());
    if !(MIN_LETTERS..=MAX_LETTERS).contains(&n)
        || (!caps && comp[1..].iter().any(|c| c.is_uppercase()))
    {
        return None;
    }
    s.word.clear();
    s.word.extend(comp.iter().map(|&c| to_lower(c)));
    // Positions that vary, and how many choices each has.
    let mut pos = [0usize; MAX_VARIED];
    let mut k = 0;
    for (i, &c) in s.word.iter().enumerate() {
        if matches!(c, 'о' | 'у' | 'ү') {
            if k == MAX_VARIED {
                return None;
            }
            pos[k] = i;
            k += 1;
        }
    }
    s.text.clear();
    s.text.extend(s.word.iter());
    if rejected.contains(s.text.as_str())
        || user.is_some_and(|u| u.is_added(&s.text) || u.is_rejected(&s.text))
    {
        return None;
    }
    let plain = words.count(&s.text).unwrap_or(0);
    let plain_uses = user.map_or(0, |u| u.uses(&s.text));

    let Scratch {
        word,
        level,
        best,
        text,
        liked,
    } = s;
    let [cand, extra] = level;
    let choices = |c: char| -> &'static [char] {
        if c == 'о' {
            &['о', 'ө']
        } else {
            &['у', 'ү', 'ө']
        }
    };
    // Each ө/ү variant is a candidate. For a word with few ambiguous letters
    // each variant also gets one more edit (ye, a long vowel, ь), so the work
    // stays small. All candidates share one tally: `зүн` has the clear
    // winner `зүүн`, not `зун`.
    let extras = k <= MAX_VARIED_EXTRA;
    let mut idx = [0usize; MAX_VARIED];
    best.clear();
    let mut tally = Tally { top: 0, second: 0 };
    // The listed candidate the user used most (from 3 uses), with its count.
    liked.clear();
    let mut fav = Fav { uses: 0, count: 0 };
    loop {
        cand.clear();
        cand.extend_from_slice(word);
        for d in 0..k {
            cand[pos[d]] = choices(word[pos[d]])[idx[d]];
        }
        // Skip the combination that rebuilds the word as typed.
        if cand.as_slice() != word.as_slice() {
            visit(words, cand, text, best, &mut tally);
            note_use(words, user, cand, text, liked, &mut fav);
        }
        if extras {
            let vowels_changed = cand.as_slice() != word.as_slice();
            extra_edits(cand, extra, vowels_changed, &mut |c| {
                visit(words, c, text, best, &mut tally);
                note_use(words, user, c, text, liked, &mut fav);
            });
        }
        // Next combination: a counter with one digit for each position.
        let mut d = 0;
        while d < k {
            idx[d] += 1;
            if idx[d] < choices(word[pos[d]]).len() {
                break;
            }
            idx[d] = 0;
            d += 1;
        }
        if d == k {
            break;
        }
    }
    let clear = clear_winner(tally.top, tally.second, plain);
    // No clear winner: the word the user used most breaks a tie between
    // candidates of similar rank. It never beats a clear winner.
    // Similar rank: within `CLEAR_FACTOR` of the word as typed when it is
    // listed (it then decides what the candidates must beat), else of the top.
    let near = |a: u32, b: u32| u64::from(a) * u64::from(CLEAR_FACTOR) > u64::from(b);
    let similar = if plain > 0 {
        near(fav.count, plain) && near(plain, fav.count)
    } else {
        near(fav.count, tally.top)
    };
    let best = if clear {
        best
    } else if fav.uses > 0 && fav.count >= MIN_COUNT && similar && plain_uses < fav.uses {
        liked
    } else {
        return None;
    };
    let mut out = String::with_capacity(best.len() * 2);
    for (i, &c) in best.iter().enumerate() {
        if caps || (i == 0 && comp[0].is_uppercase()) {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// Call `f` with each single extra edit of `cur`, built in `out`: `э` to `е`
/// (not at the end), a doubled vowel (also at the end: `боро` gives
/// `бороо`), `и` to `ь`, an added `ь` after the first letter at the
/// earliest, and the edits for `y` ([`y_edits`]).
fn extra_edits(
    cur: &[char],
    out: &mut Vec<char>,
    vowels_changed: bool,
    f: &mut dyn FnMut(&[char]),
) {
    let n = cur.len();
    for i in 0..n {
        let c = cur[i];
        let last = i + 1 == n;
        if c == 'э' && !last {
            fill(out, &cur[..i], &['е'], &cur[i + 1..]);
            f(out);
        }
        if c == 'и' && i > 0 {
            fill(out, &cur[..i], &['ь'], &cur[i + 1..]);
            f(out);
        }
        if is_vowel(c) && (i == 0 || cur[i - 1] != c) && cur.get(i + 1) != Some(&c) {
            fill(out, &cur[..=i], &[c], &cur[i + 1..]);
            f(out);
        }
    }
    for i in 1..=n {
        fill(out, &cur[..i], &['ь'], &cur[i..]);
        f(out);
    }
    y_edits(cur, out, vowels_changed, f);
}

/// Edits for the Latin `y`, which chat uses for ё, е, я and ы (issue #101):
/// `ы` to `ий` when an о/у/ү was changed too, because the ы came from the
/// wrong vowel (`ugluunii` gives өглөөний; a typed `surguulyn` stays), a first `ы` to `ё` or `е`
/// (`ystoi` gives ёстой), a first `ю` to `е` (`yusun` gives есөн), a last
/// `ы` to `я` or `ё` (`goy` gives гоё), and a last `иы` to `ъя` or `ья`
/// (`uulziy` gives уулзъя).
fn y_edits(cur: &[char], out: &mut Vec<char>, vowels_changed: bool, f: &mut dyn FnMut(&[char])) {
    let n = cur.len();
    for i in 0..n {
        if vowels_changed && cur[i] == 'ы' {
            fill(out, &cur[..i], &['и', 'й'], &cur[i + 1..]);
            f(out);
        }
    }
    match cur.first() {
        Some('ы') => {
            for c in ['ё', 'е'] {
                fill(out, &[], &[c], &cur[1..]);
                f(out);
            }
        }
        Some('ю') => {
            fill(out, &[], &['е'], &cur[1..]);
            f(out);
        }
        _ => {}
    }
    if n >= 3 && cur[n - 2] == 'и' && cur[n - 1] == 'ы' {
        for sign in ['ъ', 'ь'] {
            fill(out, &cur[..n - 2], &[sign, 'я'], &[]);
            f(out);
        }
    } else if n >= 2 && cur[n - 1] == 'ы' {
        for c in ['я', 'ё'] {
            fill(out, &cur[..n - 1], &[c], &[]);
            f(out);
        }
    }
}

/// The candidate the user used most so far.
struct Fav {
    uses: u32,
    count: u32,
}

/// Keep `cand` in `liked` when the user used it more than the best so far
/// and the list knows it. Does nothing without a user snapshot.
fn note_use(
    words: &WordList,
    user: Option<&UserSnapshot>,
    cand: &[char],
    text: &mut String,
    liked: &mut Vec<char>,
    fav: &mut Fav,
) {
    let Some(u) = user else { return };
    text.clear();
    text.extend(cand.iter());
    let uses = u.uses(text);
    if uses > fav.uses {
        if let Some(count) = words.count(text) {
            fav.uses = uses;
            fav.count = count;
            liked.clear();
            liked.extend_from_slice(cand);
        }
    }
}

/// Look `cand` up and keep the best two distinct candidates.
pub(crate) fn visit(
    words: &WordList,
    cand: &[char],
    text: &mut String,
    best: &mut Vec<char>,
    t: &mut Tally,
) {
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

/// Chat words that people often run together with the particle дээ or даа
/// (`zugerdee` for зүгээр дээ). Only these split: after other words -дээ is
/// usually the dative with a reflexive ending (ажилдаа).
const DEE_WORDS: &[&str] = &[
    "зүгээр",
    "тийм",
    "байна",
    "байгаа",
    "байхгүй",
    "мэдэхгүй",
    "гоё",
    "сайн",
    "болно",
    "үгүй",
    "мэднэ",
];

/// A typed form of a word that the list knows: the form itself, or what
/// word choice makes of it. Returns the word and its count.
fn listed_form(
    words: &WordList,
    form: &[char],
    rejected: &HashSet<String>,
    user: Option<&UserSnapshot>,
    s: &mut Scratch,
) -> Option<(String, u32)> {
    let text: String = form.iter().collect();
    if let Some(n) = words.count(&text) {
        return Some((text, n));
    }
    let chosen = choose(words, form, rejected, user, s)?;
    let n = words.count(&chosen)?;
    Some((chosen, n))
}

/// The vowel that decides a particle's form: the last vowel that is not и.
fn last_vowel(word: &str) -> Option<char> {
    word.chars()
        .rev()
        .find(|&c| matches!(c, 'а' | 'э' | 'о' | 'у' | 'ү' | 'ө' | 'я' | 'е' | 'ё' | 'ю'))
}

/// A word run together with a following particle, split again (issue #101):
/// `орсну` gives `орсон уу`, `байхгүйшдэ` gives `байхгүй ш дээ`, `зүгэрдээ`
/// gives `зүгээр дээ`. `comp` is the lower-case preedit of a word that the
/// list does not know. It runs once per word, like [`choose`].
pub(crate) fn split_particle(
    words: &WordList,
    comp: &[char],
    rejected: &HashSet<String>,
    user: Option<&UserSnapshot>,
    s: &mut Scratch,
) -> Option<String> {
    let n = comp.len();
    if !(MIN_LETTERS..=MAX_LETTERS + 4).contains(&n) || comp[1..].iter().any(|c| c.is_uppercase()) {
        return None;
    }
    let word: Vec<char> = comp.iter().map(|&c| to_lower(c)).collect();
    let text: String = word.iter().collect();
    if words.contains(&text) || rejected.contains(text.as_str()) {
        return None;
    }
    let ends = |tail: &str| -> Option<usize> {
        let t: Vec<char> = tail.chars().collect();
        (n > t.len() + 1 && word.ends_with(&t)).then(|| n - t.len())
    };
    let cap = |w: String| -> String {
        if comp[0].is_uppercase() {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect())
                .unwrap_or_default()
        } else {
            w
        }
    };

    // 1. ш дээ: any listed word before it.
    if let Some(at) = ["шдээ", "шдэ"].iter().find_map(|t| ends(t)) {
        let (stem, n) = listed_form(words, &word[..at], rejected, user, s)?;
        return (n >= MIN_COUNT).then(|| cap(format!("{stem} ш дээ")));
    }
    // 2. The past -сан/-сон/-сэн/-сөн with уу or үү, typed without its vowel.
    if let Some(at) = ["сну", "снуу", "снүү", "снү"].iter().find_map(|t| ends(t)) {
        let mut best: Option<(String, u32)> = None;
        for v in ['а', 'о', 'э', 'ө'] {
            let mut cand = word[..at].to_vec();
            cand.extend(['с', v, 'н']);
            if let Some((w, c)) = listed_form(words, &cand, rejected, user, s) {
                if best.as_ref().map_or(true, |b| c > b.1) {
                    best = Some((w, c));
                }
            }
        }
        let (stem, c) = best?;
        if c < MIN_COUNT {
            return None;
        }
        let particle = match last_vowel(&stem) {
            Some('э' | 'ө' | 'ү' | 'е') => "үү",
            _ => "уу",
        };
        return Some(cap(format!("{stem} {particle}")));
    }
    // 3. дээ or даа after a chat word.
    if let Some(at) = ["дээ", "дэ", "даа", "да"].iter().find_map(|t| ends(t)) {
        let (stem, _) = listed_form(words, &word[..at], rejected, user, s)?;
        if !DEE_WORDS.contains(&stem.as_str()) {
            return None;
        }
        let particle = match last_vowel(&stem) {
            Some('э' | 'ү' | 'е') => "дээ",
            Some('ө') => "дөө",
            Some('о' | 'ё') => "доо",
            _ => "даа",
        };
        return Some(cap(format!("{stem} {particle}")));
    }
    None
}
