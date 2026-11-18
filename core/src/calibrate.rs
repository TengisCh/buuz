// SPDX-License-Identifier: Apache-2.0

//! Calibration (issue #4): learn how a user types, from sentences.
//!
//! The user types a few Cyrillic sentences in Latin, with conversion off.
//! [`calibrate`] aligns each sentence with what was typed, counts the keys
//! the user chose for each letter, and suggests `[keys]` entries for the keys
//! that the built-in table does not give. It runs once, offline, and is not on
//! the typing path. It stores nothing.

use std::collections::{HashMap, HashSet};

use crate::composer::is_input_char;
use crate::config::{Config, ConfigError};
use crate::keys::check_mapping;
use crate::rules::{default_letter_chunks, to_lower};

/// The longest key, in characters.
const MAX_CHUNK: usize = 3;

/// The Mongolian Cyrillic alphabet, in the order of the skipped list.
const ALPHABET: &[char] = &[
    'а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'ө', 'п', 'р',
    'с', 'т', 'у', 'ү', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я',
];

/// A key the user types for a letter, which the built-in table does not give
/// for that letter. Add it to a config as a `[keys]` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// The letter (lower case).
    pub letter: char,
    /// The key (lower case), 1 to 3 characters.
    pub chunk: String,
    /// How often the user typed this key for the letter.
    pub count: usize,
    /// How often the user typed a built-in key for the letter.
    pub default_count: usize,
    /// Another key for the same letter was used as often.
    pub tie: bool,
    /// The letter the built-in table gives from this key. Accepting the
    /// suggestion makes the key stop giving it ("`v` will stop giving в").
    pub displaces: Option<char>,
    /// The key is typed for both letters of a gender pair (о and ө, or у and
    /// ү). `letter` is then the masculine one, and the target is `у/ү`.
    pub pair: bool,
}

/// A key the user types for a letter that `[keys]` cannot target (й, or a
/// digit as a key). It is not proposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsupported {
    pub letter: char,
    pub chunk: String,
    pub count: usize,
}

/// A key the user types for two letters, so it cannot be proposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// The letter that would get the key.
    pub letter: char,
    /// The key (lower case).
    pub chunk: String,
    /// How often the user typed the key for `letter`.
    pub count: usize,
    /// The other letter the user also types with the key.
    pub other: char,
    /// How often the user typed the key for `other`.
    pub other_count: usize,
}

/// The result of [`calibrate`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Calibration {
    /// Keys to add, ordered by letter in the alphabet, then by count.
    pub suggestions: Vec<Suggestion>,
    /// Keys the user types for two letters.
    pub conflicts: Vec<Conflict>,
    /// Letters of the alphabet that are not in the input.
    pub skipped_letters: Vec<char>,
    pub unsupported: Vec<Unsupported>,
    pub unmatched: Vec<usize>,
}

impl Calibration {
    /// Add every suggestion to the config as a `[keys]` entry, together. Each
    /// entry copies from the built-in table, so the order does not matter
    /// and a swap (`v = "ү"`, `w = "в"`) holds. The config changes only when
    /// every entry is accepted. "Default" in a calibration means the built-in
    /// table, not the keys the config already has: an existing entry for the
    /// same key is replaced.
    pub fn apply(&self, config: &mut Config) -> Result<(), ConfigError> {
        let mut next = config.clone();
        for s in &self.suggestions {
            next.set_key(&s.chunk, &s.target())?;
        }
        *config = next;
        Ok(())
    }
}

impl Suggestion {
    /// The `[keys]` target: the letter, or a gender pair such as `у/ү`.
    pub fn target(&self) -> String {
        match (self.pair, gender_partner(self.letter)) {
            (true, Some(p)) => format!("{}/{}", self.letter, p),
            _ => self.letter.to_string(),
        }
    }
}

/// The front vowel that goes with a back vowel in a gender pair.
fn gender_partner(letter: char) -> Option<char> {
    match letter {
        'о' => Some('ө'),
        'у' => Some('ү'),
        _ => None,
    }
}

/// The most characters in a sentence or in a typed line. A longer pair is
/// reported in `unmatched` and skipped. With this cap the alignment table
/// holds at most 501 * 501 cells, so its size cannot overflow, even on wasm32.
const MAX_CHARS: usize = 500;

/// The most pairs read. Later pairs are reported in `unmatched`.
const MAX_PAIRS: usize = 50;

type Chunk = [char; MAX_CHUNK];

fn is_letter(c: char) -> bool {
    (c as u32) >> 8 == 4 && c.is_alphabetic()
}

/// A character that may be part of a chunk: an input key, or a digit (a
/// digit as a key is reported as unsupported, not lost).
fn is_chunk_char(c: char) -> bool {
    is_input_char(c) || c.is_ascii_digit()
}

fn chunk_of(chars: &[char]) -> Chunk {
    let mut k = ['\0'; MAX_CHUNK];
    k[..chars.len()].copy_from_slice(chars);
    k
}

/// What the built-in table gives.
struct Defaults {
    /// `(letter, key)` pairs.
    pairs: HashSet<(char, Chunk)>,
    /// Letters per key.
    letters: HashMap<Chunk, Vec<char>>,
}

impl Defaults {
    fn build() -> Defaults {
        let mut pairs = HashSet::new();
        let mut letters: HashMap<Chunk, Vec<char>> = HashMap::new();
        for (letter, key) in default_letter_chunks() {
            let chars: Vec<char> = key.chars().collect();
            if chars.is_empty() || chars.len() > MAX_CHUNK {
                continue;
            }
            let k = chunk_of(&chars);
            if pairs.insert((letter, k)) {
                letters.entry(k).or_default().push(letter);
            }
        }
        Defaults { pairs, letters }
    }

    fn gives(&self, letter: char, chunk: &[char]) -> bool {
        !chunk.is_empty() && self.pairs.contains(&(letter, chunk_of(chunk)))
    }

    /// The letters the table gives from a chunk, in alphabet order.
    fn letters_of(&self, chunk: &[char]) -> Vec<char> {
        let mut v = self
            .letters
            .get(&chunk_of(chunk))
            .cloned()
            .unwrap_or_default();
        v.sort_by_key(|c| rank(*c));
        v
    }
}

fn rank(c: char) -> usize {
    ALPHABET.iter().position(|a| *a == c).unwrap_or(usize::MAX)
}

// Costs in sixteenths, so that "cost 1" ties break towards fewer gaps: a
// non-default key and a skipped character cost 1 (a key of k characters
// costs about k); a letter with no key costs
// a little more.
const COST_ONE: u32 = 16;
const COST_NO_KEY: u32 = 17;

fn match_cost(defaults: &Defaults, letter: char, chunk: &[char]) -> u32 {
    if chunk.is_empty() {
        COST_NO_KEY
    } else if defaults.gives(letter, chunk) {
        0
    } else {
        // A longer chunk costs a little less than a short chunk plus skipped
        // characters (`vv` for ү is one key, not `v` and a stray `v`), but
        // more than a short one, so that a typed word that is missing from the
        // sentence is skipped and does not become a key.
        let k = chunk.len() as u32;
        COST_ONE * k - (k - 1)
    }
}

/// What one aligned pair gave.
struct Aligned {
    /// `(letter, key)` for every letter that got a key.
    uses: Vec<(char, String)>,
    /// Letters, non-letters and typed characters left without a partner.
    gaps: usize,
}

/// Align one pair. Both sides are at most `MAX_CHARS` characters.
fn align(defaults: &Defaults, cyrillic: &[char], typed: &[char]) -> Aligned {
    let n = cyrillic.len();
    let m = typed.len();
    let w = m + 1;
    // run[j]: the typed characters from j on that may be in a chunk.
    let mut run = vec![0usize; m + 1];
    for j in (0..m).rev() {
        run[j] = if is_chunk_char(typed[j]) {
            run[j + 1] + 1
        } else {
            0
        };
    }
    let mut dp = vec![u32::MAX; (n + 1) * w];
    dp[0] = 0;
    for i in 0..=n {
        for j in 0..=m {
            let here = dp[i * w + j];
            if here == u32::MAX {
                continue;
            }
            let mut relax = |ni: usize, nj: usize, cost: u32| {
                let cell = &mut dp[ni * w + nj];
                *cell = (*cell).min(here + cost);
            };
            if j < m {
                relax(i, j + 1, COST_ONE); // a typed character with no letter
            }
            if i < n {
                let c = cyrillic[i];
                if is_letter(c) {
                    for k in 0..=MAX_CHUNK.min(run[j]) {
                        relax(i + 1, j + k, match_cost(defaults, c, &typed[j..j + k]));
                    }
                } else {
                    relax(i + 1, j, COST_ONE); // a non-letter that was not typed
                    if j < m && typed[j] == c {
                        relax(i + 1, j + 1, 0);
                    }
                }
            }
        }
    }
    // Walk back from the end. Every reachable cell has a way back.
    let mut out = Aligned {
        uses: Vec::new(),
        gaps: 0,
    };
    let (mut i, mut j) = (n, m);
    while i > 0 || j > 0 {
        let here = dp[i * w + j];
        let mut step: Option<(usize, usize)> = None;
        if i > 0 {
            let c = cyrillic[i - 1];
            if is_letter(c) {
                let mut best: Option<(u32, usize)> = None;
                for k in 0..=MAX_CHUNK.min(j) {
                    let prev = dp[(i - 1) * w + (j - k)];
                    if prev == u32::MAX || j - k + run[j - k] < j {
                        continue;
                    }
                    let cost = match_cost(defaults, c, &typed[j - k..j]);
                    if prev + cost == here && best.map_or(true, |b: (u32, usize)| cost < b.0) {
                        best = Some((cost, k));
                    }
                }
                if let Some((_, k)) = best {
                    if k == 0 {
                        out.gaps += 1;
                    } else {
                        out.uses.push((c, typed[j - k..j].iter().collect()));
                    }
                    step = Some((i - 1, j - k));
                }
            } else if j > 0 && typed[j - 1] == c && dp[(i - 1) * w + j - 1] == here {
                step = Some((i - 1, j - 1));
            } else if dp[(i - 1) * w + j] != u32::MAX && dp[(i - 1) * w + j] + COST_ONE == here {
                out.gaps += 1;
                step = Some((i - 1, j));
            }
        }
        if step.is_none() && j > 0 {
            out.gaps += 1;
            step = Some((i, j - 1));
        }
        match step {
            Some((ni, nj)) => {
                i = ni;
                j = nj;
            }
            None => break,
        }
    }
    out
}

/// Fold case and compose the marks that Unicode may write apart: и + breve
/// is й, е + diaeresis is ё, u/o + diaeresis is ü/ö. Variation selectors and
/// other combining marks are dropped.
fn normalize(s: &str) -> Vec<char> {
    let mut out: Vec<char> = Vec::with_capacity(s.len());
    for ch in s.chars() {
        let c = to_lower(ch);
        match c {
            '\u{fe00}'..='\u{fe0f}' | '\u{200d}' => {}
            '\u{306}' if out.last() == Some(&'и') => *out.last_mut().unwrap() = 'й',
            '\u{308}' if matches!(out.last(), Some('е' | 'u' | 'o')) => {
                let last = out.last_mut().unwrap();
                *last = match *last {
                    'е' => 'ё',
                    'u' => 'ü',
                    _ => 'ö',
                };
            }
            '\u{300}'..='\u{36f}' => {}
            _ => out.push(c),
        }
    }
    out
}

/// One chunk that the user typed for a letter, or a gender pair of them.
struct Item {
    letter: char,
    pair: bool,
    count: usize,
    default_count: usize,
}

/// Calibrate from `(cyrillic sentence, what the user typed)` pairs. The user
/// types each sentence in Latin with conversion off. Case is folded. A pair
/// over 500 characters, a pair past the 50th, and a pair that does not align
/// (more than half of its letters have no partner) is listed in `unmatched`
/// and not counted.
pub fn calibrate(pairs: &[(&str, &str)]) -> Calibration {
    let defaults = Defaults::build();
    let mut usage: HashMap<(char, String), usize> = HashMap::new();
    let mut default_use: HashMap<char, usize> = HashMap::new();
    let mut present: HashSet<char> = HashSet::new();
    let mut unmatched: Vec<usize> = Vec::new();
    for (index, (cyrillic, typed)) in pairs.iter().enumerate() {
        if index >= MAX_PAIRS
            || cyrillic.chars().take(MAX_CHARS + 1).count() > MAX_CHARS
            || typed.chars().take(MAX_CHARS + 1).count() > MAX_CHARS
        {
            unmatched.push(index);
            continue;
        }
        let cyrillic = normalize(cyrillic);
        let typed = normalize(typed);
        let letters = cyrillic.iter().filter(|c| is_letter(**c)).count();
        let aligned = align(&defaults, &cyrillic, &typed);
        if aligned.gaps * 2 > letters {
            unmatched.push(index);
            continue;
        }
        present.extend(cyrillic.iter().copied().filter(|&c| is_letter(c)));
        for (letter, chunk) in aligned.uses {
            let chars: Vec<char> = chunk.chars().collect();
            if defaults.gives(letter, &chars) {
                *default_use.entry(letter).or_insert(0) += 1;
            }
            *usage.entry((letter, chunk)).or_insert(0) += 1;
        }
    }

    // Candidates: non-default chunks used at least twice and more often than
    // the built-in chunks, by chunk. A chunk with no `[keys]` target is
    // unsupported; a chunk that the user also types for the letter the table
    // gives from it is a conflict.
    let mut by_chunk: HashMap<String, Vec<Item>> = HashMap::new();
    let mut unsupported: Vec<Unsupported> = Vec::new();
    let mut conflicts: Vec<Conflict> = Vec::new();
    let mut keys: Vec<&(char, String)> = usage.keys().collect();
    keys.sort_by(|a, b| rank(a.0).cmp(&rank(b.0)).then_with(|| a.1.cmp(&b.1)));
    for key in keys {
        let (letter, chunk) = key;
        let count = usage[key];
        let chars: Vec<char> = chunk.chars().collect();
        let default_count = default_use.get(letter).copied().unwrap_or(0);
        if defaults.gives(*letter, &chars) || count < 2 || count <= default_count {
            continue;
        }
        if check_mapping(chunk, &letter.to_string()).is_err() {
            unsupported.push(Unsupported {
                letter: *letter,
                chunk: chunk.clone(),
                count,
            });
            continue;
        }
        let mut conflicted = false;
        for other in defaults.letters_of(&chars) {
            let other_count = usage.get(&(other, chunk.clone())).copied().unwrap_or(0);
            if other != *letter && other_count > 0 {
                conflicts.push(Conflict {
                    letter: *letter,
                    chunk: chunk.clone(),
                    count,
                    other,
                    other_count,
                });
                conflicted = true;
            }
        }
        if !conflicted {
            by_chunk.entry(chunk.clone()).or_default().push(Item {
                letter: *letter,
                pair: false,
                count,
                default_count,
            });
        }
    }

    // One chunk gives one target. Both letters of a gender pair become one
    // pair target; otherwise the most common letter wins, and equal top
    // counts propose none.
    let mut chunks: Vec<String> = by_chunk.keys().cloned().collect();
    chunks.sort();
    let mut proposed: Vec<(String, Item)> = Vec::new();
    for chunk in chunks {
        let mut items = by_chunk.remove(&chunk).unwrap_or_default();
        for front in ['ө', 'ү'] {
            let Some(back) = items.iter().position(|i| Some(i.letter) == back_of(front)) else {
                continue;
            };
            let Some(fr) = items.iter().position(|i| i.letter == front) else {
                continue;
            };
            let (a, b) = (items[back].count, items[fr].count);
            let (da, db) = (items[back].default_count, items[fr].default_count);
            let letter = items[back].letter;
            items.retain(|i| i.letter != letter && i.letter != front);
            items.push(Item {
                letter,
                pair: true,
                count: a + b,
                default_count: da + db,
            });
        }
        items.sort_by_key(|i| rank(i.letter));
        let top = items.iter().map(|i| i.count).max().unwrap_or(0);
        let leaders = items.iter().filter(|i| i.count == top).count();
        if items.len() == 1 || leaders == 1 {
            let at = items.iter().position(|i| i.count == top).unwrap_or(0);
            let win = items.remove(at);
            for lose in &items {
                conflicts.push(Conflict {
                    letter: lose.letter,
                    chunk: chunk.clone(),
                    count: lose.count,
                    other: win.letter,
                    other_count: win.count,
                });
            }
            proposed.push((chunk, win));
        } else {
            for (x, a) in items.iter().enumerate() {
                for b in &items[x + 1..] {
                    conflicts.push(Conflict {
                        letter: a.letter,
                        chunk: chunk.clone(),
                        count: a.count,
                        other: b.letter,
                        other_count: b.count,
                    });
                }
            }
        }
    }
    proposed.sort_by(|a, b| {
        rank(a.1.letter)
            .cmp(&rank(b.1.letter))
            .then(b.1.count.cmp(&a.1.count))
            .then_with(|| a.0.cmp(&b.0))
    });
    let suggestions: Vec<Suggestion> = proposed
        .iter()
        .map(|(chunk, item)| {
            let chars: Vec<char> = chunk.chars().collect();
            let partner = gender_partner(item.letter).filter(|_| item.pair);
            let displaces = defaults
                .letters_of(&chars)
                .into_iter()
                .find(|l| *l != item.letter && Some(*l) != partner);
            // A tie: another proposed chunk for the same letter, as common.
            let tie = proposed.iter().any(|(k, o)| {
                k != chunk
                    && o.letter == item.letter
                    && o.pair == item.pair
                    && o.count == item.count
            });
            Suggestion {
                letter: item.letter,
                chunk: chunk.clone(),
                count: item.count,
                default_count: item.default_count,
                tie,
                displaces,
                pair: item.pair,
            }
        })
        .collect();

    let skipped_letters = ALPHABET
        .iter()
        .copied()
        .filter(|c| !present.contains(c))
        .collect();
    Calibration {
        suggestions,
        conflicts,
        skipped_letters,
        unsupported,
        unmatched,
    }
}

/// The back vowel of a gender pair, by its front vowel.
fn back_of(front: char) -> Option<char> {
    match front {
        'ө' => Some('о'),
        'ү' => Some('у'),
        _ => None,
    }
}
