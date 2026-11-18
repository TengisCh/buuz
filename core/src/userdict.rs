// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. See the LICENSE file in this crate.

//! The personal dictionary (issue #12): words the user added, corrections the
//! user rejected, and how often the user typed known words. It has no I/O and
//! no clock: the shell owns the file or the storage, and the order of use is a
//! logical counter.
//!
//! The dictionary is private. It never goes into the shareable
//! [`Config`](crate::Config). The composer only reads a snapshot
//! (`Options::user`) and returns [`Learned`] events; the shell applies them to
//! its own `UserDict`. Nothing here sends data anywhere.
//!
//! Text format: one `kind<TAB>word<TAB>count` line per entry, with `kind` one
//! of `add`, `reject` or `use`. Blank lines and lines that start with `#` are
//! skipped. The order of the lines is the order of use, oldest first, so a
//! round trip keeps the order that decides which entry is evicted first.
//! [`UserDict::parse`] skips a bad line and reports it as a warning; one bad
//! line must not cost the user the whole dictionary. A file over
//! [`MAX_USERDICT_BYTES`] is refused as a whole.
//!
//! ```
//! let mut d = buuz_core::UserDict::new();
//! d.add("Тэнгис");
//! d.record_use("сайн");
//! let again = buuz_core::UserDict::parse(&d.to_text()).unwrap().dict;
//! assert!(again.is_added("тэнгис"));
//! assert_eq!(again.uses("сайн"), 1);
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::rules::to_lower;

/// The most entries the dictionary holds. The least recently used entry is
/// evicted first.
pub const MAX_USERDICT_ENTRIES: usize = 5_000;

/// The longest word, in characters.
pub const MAX_USERDICT_WORD_CHARS: usize = 64;

/// The largest text [`UserDict::parse`] accepts, in bytes.
pub const MAX_USERDICT_BYTES: usize = 1024 * 1024;

/// A use count of this size or more makes a word a user word for completion.
pub(crate) const MIN_USES_FOR_COMPLETION: u32 = 3;

/// What the composer learned from a committed word. The shell applies it with
/// [`UserDict::apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Learned {
    /// The user committed this lower-case word, and it is in the word list or
    /// the user dictionary.
    Used(String),
    /// The user undid the correction of this lower-case word with Backspace.
    Rejected(String),
}

/// The kind of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDictKind {
    /// A word the user added: always valid, never corrected.
    Add,
    /// A word whose correction the user undid: never corrected again.
    Reject,
    /// A known word the user typed: the count is how often.
    Use,
}

impl UserDictKind {
    fn name(self) -> &'static str {
        match self {
            UserDictKind::Add => "add",
            UserDictKind::Reject => "reject",
            UserDictKind::Use => "use",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// One entry, for a settings list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDictEntry {
    pub kind: UserDictKind,
    pub word: String,
    /// How often (`Use`, `Reject`). 1 for `Add`.
    pub count: u32,
}

/// What is wrong with a skipped line of [`UserDict::parse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDictProblem {
    /// The line does not have exactly three tab-separated fields.
    BadLine,
    /// The kind is not `add`, `reject` or `use`.
    UnknownKind,
    /// The count is not a number from 1 to 4294967295.
    BadCount,
    /// The word is empty.
    EmptyWord,
    /// The word has white space or a control character.
    BadWord,
    /// The word is longer than [`MAX_USERDICT_WORD_CHARS`].
    WordTooLong,
    /// The same kind and word appeared on an earlier line. The first stays.
    Duplicate,
}

/// A skipped line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserDictWarning {
    /// The 1-based line.
    pub line: usize,
    pub problem: UserDictProblem,
}

/// The text is larger than [`MAX_USERDICT_BYTES`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDictError {
    pub size: usize,
    pub limit: usize,
}

impl fmt::Display for UserDictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "dictionary too large: {} bytes (limit {})",
            self.size, self.limit
        )
    }
}

impl std::error::Error for UserDictError {}

/// The result of a successful [`UserDict::parse`].
#[derive(Debug, Clone)]
pub struct ParsedUserDict {
    pub dict: UserDict,
    /// The lines that were skipped, in line order.
    pub warnings: Vec<UserDictWarning>,
}

#[derive(Debug, Clone, Copy)]
struct Slot {
    count: u32,
    /// The logical time of the last use.
    stamp: u64,
}

/// The personal dictionary. Words are lower case; [`UserDict::add`],
/// [`UserDict::reject`] and [`UserDict::record_use`] lower-case their input.
/// The query methods expect lower case, so the typing path does not allocate.
#[derive(Debug, Clone, Default)]
pub struct UserDict {
    /// One map for each [`UserDictKind`], in the order of its discriminant.
    maps: [HashMap<String, Slot>; 3],
    clock: u64,
}

fn valid_word(word: &str) -> Result<(), UserDictProblem> {
    if word.is_empty() {
        return Err(UserDictProblem::EmptyWord);
    }
    if word.chars().count() > MAX_USERDICT_WORD_CHARS {
        return Err(UserDictProblem::WordTooLong);
    }
    if word.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(UserDictProblem::BadWord);
    }
    Ok(())
}

impl UserDict {
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of entries (a word can have an entry of each kind).
    pub fn len(&self) -> usize {
        self.maps.iter().map(HashMap::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Forget everything.
    pub fn clear(&mut self) {
        for m in &mut self.maps {
            m.clear();
        }
    }

    /// Add a word the user wants to keep. Returns false if the word is empty,
    /// longer than [`MAX_USERDICT_WORD_CHARS`] or has white space.
    pub fn add(&mut self, word: &str) -> bool {
        self.touch(UserDictKind::Add, word, 1, false)
    }

    /// Remember that the user undid the correction of `word`.
    pub fn reject(&mut self, word: &str) -> bool {
        self.touch(UserDictKind::Reject, word, 1, true)
    }

    /// Count one use of `word`.
    pub fn record_use(&mut self, word: &str) -> bool {
        self.touch(UserDictKind::Use, word, 1, true)
    }

    /// Apply an event from the composer.
    pub fn apply(&mut self, event: &Learned) {
        match event {
            Learned::Used(w) => self.record_use(w),
            Learned::Rejected(w) => self.reject(w),
        };
    }

    /// Delete every entry of `word`. Returns true if there was one.
    pub fn remove(&mut self, word: &str) -> bool {
        let word = lower(word);
        let mut found = false;
        for m in &mut self.maps {
            found |= m.remove(&word).is_some();
        }
        found
    }

    /// The user added `word` (lower case).
    pub fn is_added(&self, word: &str) -> bool {
        self.maps[UserDictKind::Add.index()].contains_key(word)
    }

    /// The user rejected the correction of `word` (lower case).
    pub fn is_rejected(&self, word: &str) -> bool {
        self.maps[UserDictKind::Reject.index()].contains_key(word)
    }

    /// How often the user typed `word` (lower case) as a known word.
    pub fn uses(&self, word: &str) -> u32 {
        self.maps[UserDictKind::Use.index()]
            .get(word)
            .map_or(0, |s| s.count)
    }

    /// All entries, the least recently used first.
    pub fn entries(&self) -> Vec<UserDictEntry> {
        let mut all: Vec<(u64, UserDictEntry)> = Vec::with_capacity(self.len());
        for kind in KINDS {
            for (word, slot) in &self.maps[kind.index()] {
                all.push((
                    slot.stamp,
                    UserDictEntry {
                        kind,
                        word: word.clone(),
                        count: slot.count,
                    },
                ));
            }
        }
        all.sort_by_key(|(stamp, _)| *stamp);
        all.into_iter().map(|(_, e)| e).collect()
    }

    /// Build the read-only snapshot for the typing path (`Options::user`).
    /// It costs a sort of at most 5,000 words, so build it when a field gets
    /// focus or after an idle pause, not per key. See [`UserSnapshot`].
    pub fn snapshot(&self) -> UserSnapshot {
        let uses = &self.maps[UserDictKind::Use.index()];
        let adds = &self.maps[UserDictKind::Add.index()];
        let rejected = &self.maps[UserDictKind::Reject.index()];
        let mut cands: Vec<(&str, u32, u64)> = Vec::new();
        for (w, slot) in adds {
            let count = uses.get(w.as_str()).map_or(0, |u| u.count);
            cands.push((w.as_str(), count, slot.stamp));
        }
        for (w, slot) in uses {
            if slot.count >= MIN_USES_FOR_COMPLETION && !adds.contains_key(w.as_str()) {
                cands.push((w.as_str(), slot.count, slot.stamp));
            }
        }
        cands.retain(|(w, _, _)| !rejected.contains_key(*w));
        cands.sort_unstable_by(|a, b| a.0.cmp(b.0));
        UserSnapshot {
            words: cands.iter().map(|c| c.0.to_string()).collect(),
            ranks: cands.iter().map(|c| (c.1, c.2)).collect(),
            added: adds.keys().cloned().collect(),
            rejected: rejected.keys().cloned().collect(),
        }
    }

    /// Parse the text format. See the module documentation.
    pub fn parse(text: &str) -> Result<ParsedUserDict, UserDictError> {
        if text.len() > MAX_USERDICT_BYTES {
            return Err(UserDictError {
                size: text.len(),
                limit: MAX_USERDICT_BYTES,
            });
        }
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut dict = UserDict::new();
        let mut warnings = Vec::new();
        for (i, raw) in text.lines().enumerate() {
            let line = i + 1;
            if raw.trim().is_empty() || raw.starts_with('#') {
                continue;
            }
            if let Err(problem) = dict.parse_line(raw) {
                warnings.push(UserDictWarning { line, problem });
            }
        }
        dict.trim();
        Ok(ParsedUserDict { dict, warnings })
    }

    fn parse_line(&mut self, raw: &str) -> Result<(), UserDictProblem> {
        let mut f = raw.split('\t');
        let (Some(kind), Some(word), Some(count), None) = (f.next(), f.next(), f.next(), f.next())
        else {
            return Err(UserDictProblem::BadLine);
        };
        let kind = match kind {
            "add" => UserDictKind::Add,
            "reject" => UserDictKind::Reject,
            "use" => UserDictKind::Use,
            _ => return Err(UserDictProblem::UnknownKind),
        };
        let count = match count.parse::<u32>() {
            Ok(n) if n >= 1 && count.bytes().all(|b| b.is_ascii_digit()) => n,
            _ => return Err(UserDictProblem::BadCount),
        };
        valid_word(word)?;
        let word = lower(word);
        if self.maps[kind.index()].contains_key(&word) {
            return Err(UserDictProblem::Duplicate);
        }
        // The count of an added word has no meaning: keep it 1.
        let count = if kind == UserDictKind::Add { 1 } else { count };
        let stamp = self.tick();
        self.maps[kind.index()].insert(word, Slot { count, stamp });
        Ok(())
    }

    /// Drop the oldest entries above the cap: `use` entries first, then the
    /// others. `parse` inserts everything and calls this once at the end, so
    /// a line after the cap still counts as a duplicate.
    fn trim(&mut self) {
        let over = self.len().saturating_sub(MAX_USERDICT_ENTRIES);
        if over == 0 {
            return;
        }
        let mut order: Vec<(bool, u64, usize)> = Vec::with_capacity(self.len());
        let mut words: Vec<&str> = Vec::with_capacity(self.len());
        for kind in KINDS {
            for (w, slot) in &self.maps[kind.index()] {
                order.push((kind != UserDictKind::Use, slot.stamp, words.len()));
                words.push(w);
            }
        }
        let kinds: Vec<usize> = KINDS
            .iter()
            .flat_map(|k| std::iter::repeat(k.index()).take(self.maps[k.index()].len()))
            .collect();
        order.sort_unstable();
        let drop: Vec<(usize, String)> = order[..over]
            .iter()
            .map(|&(_, _, i)| (kinds[i], words[i].to_string()))
            .collect();
        for (k, w) in drop {
            self.maps[k].remove(&w);
        }
    }

    /// The text format, the least recently used entry first.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        for e in self.entries() {
            s.push_str(e.kind.name());
            s.push('\t');
            s.push_str(&e.word);
            s.push('\t');
            s.push_str(&e.count.to_string());
            s.push('\n');
        }
        s
    }

    /// Record one touch of an entry; `counting` kinds add 1 to the count.
    fn touch(&mut self, kind: UserDictKind, word: &str, by: u32, counting: bool) -> bool {
        if valid_word(word).is_err() {
            return false;
        }
        let word = lower(word);
        let stamp = self.tick();
        if let Some(slot) = self.maps[kind.index()].get_mut(&word) {
            slot.stamp = stamp;
            if counting {
                slot.count = slot.count.saturating_add(by);
            }
        } else {
            self.insert(kind, word.clone(), by);
        }
        // Using an added word keeps it fresh.
        if kind == UserDictKind::Use {
            if let Some(add) = self.maps[UserDictKind::Add.index()].get_mut(&word) {
                add.stamp = stamp;
            }
        }
        true
    }

    fn insert(&mut self, kind: UserDictKind, word: String, count: u32) {
        if self.len() >= MAX_USERDICT_ENTRIES {
            // Full of added and rejected words: a new use count is not worth
            // an entry the user made on purpose.
            if kind == UserDictKind::Use && self.maps[UserDictKind::Use.index()].is_empty() {
                return;
            }
            self.evict();
        }
        let stamp = self.tick();
        self.maps[kind.index()].insert(word, Slot { count, stamp });
    }

    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    /// Free one slot: the oldest `use` entry, or, when there is none, the
    /// oldest `add` or `reject` entry.
    fn evict(&mut self) {
        let oldest = |kinds: &[UserDictKind]| {
            kinds
                .iter()
                .flat_map(|k| {
                    self.maps[k.index()]
                        .iter()
                        .map(move |(w, s)| (s.stamp, k.index(), w.clone()))
                })
                .min_by_key(|(stamp, _, _)| *stamp)
        };
        let victim = oldest(&[UserDictKind::Use])
            .or_else(|| oldest(&[UserDictKind::Add, UserDictKind::Reject]));
        if let Some((_, k, w)) = victim {
            self.maps[k].remove(&w);
        }
    }
}

/// The read-only view of a [`UserDict`] for the typing path. Build it with
/// [`UserDict::snapshot`] and share it as `Options::user`. It holds the
/// completion candidates sorted by word (added words and words used 3 times
/// or more, minus rejected ones), so a key finds the prefix range with a
/// binary search and scans only that range. It also holds the added and the
/// rejected words for autocorrect. Immutable: no lock.
#[derive(Debug, Clone, Default)]
pub struct UserSnapshot {
    /// Completion candidates, sorted by byte order.
    words: Vec<String>,
    /// (use count, last use) for each word in `words`.
    ranks: Vec<(u32, u64)>,
    added: HashSet<String>,
    rejected: HashSet<String>,
}

impl UserSnapshot {
    /// The user added `word` (lower case).
    pub fn is_added(&self, word: &str) -> bool {
        self.added.contains(word)
    }

    /// The user rejected the correction of `word` (lower case).
    pub fn is_rejected(&self, word: &str) -> bool {
        self.rejected.contains(word)
    }

    /// How many completion candidates start with `prefix`: the size of the
    /// range that [`UserSnapshot`] scans for a key.
    pub fn candidates(&self, prefix: &str) -> usize {
        let start = self.words.partition_point(|w| w.as_str() < prefix);
        self.words[start..]
            .iter()
            .take_while(|w| w.starts_with(prefix))
            .count()
    }

    /// The best candidate that starts with `prefix` (lower case). The word
    /// can equal `prefix`: the composer then shows no ghost, because the user
    /// prefers the word as typed. More uses win; the more recent word wins a
    /// tie. Allocation-free.
    pub(crate) fn completion(&self, prefix: &str) -> Option<&str> {
        let start = self.words.partition_point(|w| w.as_str() < prefix);
        let mut best: Option<usize> = None;
        for i in start..self.words.len() {
            if !self.words[i].starts_with(prefix) {
                break;
            }
            if best.map_or(true, |b| self.ranks[i] > self.ranks[b]) {
                best = Some(i);
            }
        }
        best.map(|i| self.words[i].as_str())
    }
}

const KINDS: [UserDictKind; 3] = [UserDictKind::Add, UserDictKind::Reject, UserDictKind::Use];

fn lower(word: &str) -> String {
    word.chars().map(to_lower).collect()
}
