// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. See the LICENSE file in this crate.

//! The Mongolian word list (issue #11): common words by frequency rank, with
//! exact lookup and prefix search. No I/O and no dependency: the caller hands
//! over the text of `data/wordlist/mn.txt`. Parsing is not on the typing
//! path: shells load the list lazily, when `Config::needs_word_list()` is
//! true. With completion on, `top_two` runs once per key, so it must stay
//! allocation-free and fast.
//!
//! Format: one `word<TAB>count` per line, best rank first. Lines starting with `#` and blank lines are skipped.
//!
//! ```
//! let list = buuz_core::WordList::parse("# demo\nбайна\t9\nбол\t5\nбанк\t2\n").unwrap();
//! assert_eq!(list.rank("бол"), Some(1));
//! assert_eq!(list.prefix("ба", 5), ["байна", "банк"]);
//! ```

use std::fmt;

/// Why a word list did not parse. Every variant carries the 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordListError {
    /// The word before the tab is empty.
    EmptyWord { line: usize },
    /// The word contains whitespace.
    SpaceInWord { line: usize },
    /// The count is missing, not a number, over `u32::MAX`, or there is more than one tab.
    BadCount { line: usize },
    /// The word already appeared on an earlier line.
    Duplicate { line: usize },
    /// The list is larger than 4 GiB.
    TooLarge { line: usize },
}

impl WordListError {
    /// The 1-based line the error is on.
    pub fn line(&self) -> usize {
        match self {
            WordListError::EmptyWord { line }
            | WordListError::SpaceInWord { line }
            | WordListError::BadCount { line }
            | WordListError::Duplicate { line }
            | WordListError::TooLarge { line } => *line,
        }
    }
}

impl fmt::Display for WordListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, what) = match self {
            WordListError::EmptyWord { line } => (line, "empty word"),
            WordListError::SpaceInWord { line } => (line, "space in word"),
            WordListError::BadCount { line } => {
                (line, "expected `word<TAB>count` (count 0 to 4294967295)")
            }
            WordListError::Duplicate { line } => (line, "duplicate word"),
            WordListError::TooLarge { line } => (line, "word list too large"),
        };
        write!(f, "line {line}: {what}")
    }
}

impl std::error::Error for WordListError {}

/// Words in rank order with a sorted index for lookup and prefix search.
///
/// Words are lower case, as `data/wordlist/mn.txt` writes them. Lookups are
/// case-sensitive: `contains("Байна")` is false. Lower-case the query first.
#[derive(Debug, Clone, Default)]
pub struct WordList {
    /// All words back to back, in rank order.
    blob: String,
    /// `ends[r]` is where the word of rank `r` ends in `blob`.
    ends: Vec<u32>,
    /// `counts[r]` is how often the word of rank `r` occurred in the source.
    counts: Vec<u32>,
    /// Ranks sorted by the word they name (byte order equals code point order).
    sorted: Vec<u32>,
}

impl WordList {
    /// Parse the text of a word list: `word<TAB>count` per line, best rank
    /// first. Rank is the order of the words, from 0. A leading BOM, `#`
    /// comment lines and blank lines are skipped. The first error in line
    /// order is reported.
    pub fn parse(text: &str) -> Result<WordList, WordListError> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        match Self::parse_lines(text) {
            Ok(list) => Ok(list),
            // A duplicate on an earlier line beats a later parse error.
            Err(e) => Err(duplicate_line(text, e.line()).unwrap_or(e)),
        }
    }

    fn parse_lines(text: &str) -> Result<WordList, WordListError> {
        let mut blob = String::with_capacity(text.len());
        let mut ends: Vec<u32> = Vec::with_capacity(text.len() / 16);
        let mut counts: Vec<u32> = Vec::with_capacity(text.len() / 16);
        for (i, raw) in text.lines().enumerate() {
            let line = i + 1;
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let mut fields = raw.split('\t');
            let word = fields.next().unwrap_or("");
            let count = match (fields.next(), fields.next()) {
                (Some(c), None) if !c.is_empty() && c.bytes().all(|b| b.is_ascii_digit()) => c
                    .parse::<u32>()
                    .map_err(|_| WordListError::BadCount { line })?,
                _ => return Err(WordListError::BadCount { line }),
            };
            if word.is_empty() {
                return Err(WordListError::EmptyWord { line });
            }
            if word.chars().any(char::is_whitespace) {
                return Err(WordListError::SpaceInWord { line });
            }
            blob.push_str(word);
            let end = u32::try_from(blob.len()).map_err(|_| WordListError::TooLarge { line })?;
            ends.push(end);
            counts.push(count);
        }
        // Sort (word, rank) pairs: comparing slices directly is much faster
        // than looking each word up by rank on every comparison.
        let mut pairs: Vec<(&str, u32)> = Vec::with_capacity(ends.len());
        let mut start = 0usize;
        for (rank, &end) in ends.iter().enumerate() {
            pairs.push((&blob[start..end as usize], rank as u32));
            start = end as usize;
        }
        pairs.sort_unstable();
        // Duplicates are neighbours once sorted.
        if pairs.windows(2).any(|p| p[0].0 == p[1].0) {
            return Err(
                duplicate_line(text, usize::MAX).unwrap_or(WordListError::Duplicate { line: 0 })
            );
        }
        let sorted: Vec<u32> = pairs.iter().map(|p| p.1).collect();
        Ok(WordList {
            blob,
            ends,
            counts,
            sorted,
        })
    }

    /// Add the built-in everyday chat words (issue #101), each with a count
    /// of at least 10,000. A word that the list has keeps its rank. A new
    /// word goes after the last rank. The Wikipedia list knows few chat
    /// words, so the shells call this after `parse`.
    pub fn with_chat_words(mut self) -> WordList {
        use crate::chat_words::{CHAT_COUNT, CHAT_WORDS};
        let mut added = false;
        for &w in CHAT_WORDS {
            match self.rank(w) {
                Some(r) => {
                    let c = &mut self.counts[r as usize];
                    *c = (*c).max(CHAT_COUNT);
                }
                None => {
                    self.blob.push_str(w);
                    self.ends.push(self.blob.len() as u32);
                    self.counts.push(CHAT_COUNT);
                    added = true;
                }
            }
        }
        if added {
            let mut sorted: Vec<u32> = (0..self.ends.len() as u32).collect();
            sorted.sort_unstable_by(|&a, &b| self.word(a).cmp(self.word(b)));
            self.sorted = sorted;
        }
        self
    }

    fn word(&self, rank: u32) -> &str {
        let r = rank as usize;
        let start = if r == 0 { 0 } else { self.ends[r - 1] as usize };
        &self.blob[start..self.ends[r] as usize]
    }

    /// Number of words.
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    /// True when the list has no words.
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    /// Rank of `word` (0 is the most frequent), or `None` when it is not listed.
    pub fn rank(&self, word: &str) -> Option<u32> {
        let at = self.sorted.partition_point(|&r| self.word(r) < word);
        let r = *self.sorted.get(at)?;
        (self.word(r) == word).then_some(r)
    }

    /// How often `word` occurred in the source text, or `None` when not listed.
    pub fn count(&self, word: &str) -> Option<u32> {
        self.rank(word).map(|r| self.counts[r as usize])
    }

    /// True when `word` is in the list (exact match).
    pub fn contains(&self, word: &str) -> bool {
        self.rank(word).is_some()
    }

    /// The `n` best-ranked words that start with `prefix`, best first. An
    /// empty prefix gives the top `n` words overall.
    pub fn prefix(&self, prefix: &str, n: usize) -> Vec<&str> {
        if n == 0 {
            return Vec::new();
        }
        if prefix.is_empty() {
            return (0..self.len().min(n) as u32)
                .map(|r| self.word(r))
                .collect();
        }
        // Words with the prefix are one contiguous run in the sorted index.
        let lo = self.sorted.partition_point(|&r| self.word(r) < prefix);
        let hi = lo + self.sorted[lo..].partition_point(|&r| self.word(r).starts_with(prefix));
        let mut best: Vec<u32> = Vec::with_capacity(n.min(hi - lo));
        for &r in &self.sorted[lo..hi] {
            if best.len() == n {
                if r >= best[n - 1] {
                    continue;
                }
                best.pop();
            }
            let at = best.partition_point(|&b| b < r);
            best.insert(at, r);
        }
        best.into_iter().map(|r| self.word(r)).collect()
    }

    /// True when some word starts with `prefix` (or equals it). One binary
    /// search, no allocation. An empty prefix is true for a non-empty list.
    pub fn has_prefix(&self, prefix: &str) -> bool {
        let lo = self.sorted.partition_point(|&r| self.word(r) < prefix);
        self.sorted
            .get(lo)
            .is_some_and(|&r| self.word(r).starts_with(prefix))
    }

    /// The two best-ranked words that start with `prefix`, with their counts,
    /// best first. One scan, no allocation. An empty prefix gives ranks 0 and 1.
    #[allow(clippy::type_complexity)]
    pub fn top_two(&self, prefix: &str) -> (Option<(&str, u32)>, Option<(&str, u32)>) {
        let (mut first, mut second) = (u32::MAX, u32::MAX);
        if prefix.is_empty() {
            let n = self.len() as u32;
            first = if n > 0 { 0 } else { u32::MAX };
            second = if n > 1 { 1 } else { u32::MAX };
        } else {
            let lo = self.sorted.partition_point(|&r| self.word(r) < prefix);
            let hi = lo + self.sorted[lo..].partition_point(|&r| self.word(r).starts_with(prefix));
            for &r in &self.sorted[lo..hi] {
                if r < first {
                    second = first;
                    first = r;
                } else if r < second {
                    second = r;
                }
            }
        }
        let pick = |r: u32| (r != u32::MAX).then(|| (self.word(r), self.counts[r as usize]));
        (pick(first), pick(second))
    }
}

/// Line of the first repeated word before line `limit` (error path only).
fn duplicate_line(text: &str, limit: usize) -> Option<WordListError> {
    let mut seen = std::collections::HashSet::new();
    for (i, raw) in text.lines().enumerate() {
        if i + 1 >= limit {
            break;
        }
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let word = raw.split('\t').next().unwrap_or("");
        if !seen.insert(word) {
            return Some(WordListError::Duplicate { line: i + 1 });
        }
    }
    None
}
