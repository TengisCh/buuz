// SPDX-License-Identifier: Apache-2.0
// Copyright 2009 Odbayar Nyamtseren <odbayar.n@gmail.com> (original Buuz rules)
// Rust port: Copyright 2026 the Buuz contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Composition-buffer API mirroring the original Buuz IME `Composer`.
//!
//! A front end (Windows TSF, macOS InputMethodKit, ...) feeds [`Key`]s to a
//! [`Composer`] and renders [`Composer::preedit`] / [`Composer::cursor`],
//! inserting any text returned as `commit`.

use std::collections::HashSet;
use std::sync::Arc;

use crate::abbrev::Abbreviations;
use crate::autocorrect::{self, Scratch};
use crate::keys::KeyMap;
use crate::rules::{
    rules, to_lower, Rule, RuleSet, II_FINAL_STEMS, KEEP_II, X_DV, X_F, X_HK, X_II, X_M, X_MF, X_MM,
};
use crate::userdict::{Learned, UserSnapshot};
use crate::wordlist::WordList;

/// Maximum number of raw input characters in one composition (as original).
pub const MAX_COMP_LEN: usize = 50;

/// A key event, already translated to text by the platform layer
/// (i.e. `Char` carries the case produced by Shift/CapsLock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    /// Modifier keys that must not end a composition (Shift, CapsLock).
    Modifier,
    /// A space: like `Other` (commits and passes through), but the composer
    /// remembers the previous word's gender for the particle `uu`/`ү`.
    Space,
    /// Esc. With [`Options::keep_latin`] and a composition in progress it
    /// toggles the preedit between Cyrillic and the raw Latin keys; otherwise
    /// it acts like [`Key::Other`].
    Escape,
    /// A printable character that is not a letter and not a space (`.` `@`
    /// `:` `/` `-` `_`, digits and others). Like [`Key::Other`] it commits the
    /// word and passes through. With [`Options::auto_latin`] the composer also
    /// watches the run for a link or an email and may turn it Latin.
    Symbol(char),
    /// Tab. With a completion (see [`Composer::completion`]) it is consumed
    /// and accepts the ghost text into the preedit. Otherwise it acts like
    /// [`Key::Other`].
    Tab,
    /// Anything else (Enter and other non-printing keys).
    /// Ends the composition and is passed through to the application.
    Other,
}

/// Result of [`Composer::process_key`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyOutcome {
    /// `true` if the IME consumed the key; `false` means the application
    /// should receive it (after inserting `commit`, if any).
    pub consumed: bool,
    /// Text to insert into the document now.
    pub commit: Option<String>,
    /// Input was refused because the composition is full (original beeps).
    pub rejected: bool,
    /// Number of characters (Unicode scalars) right before the caret that the
    /// application must delete first. Non-zero when Backspace reopens a word
    /// committed by Space (an abbreviation expansion, or any word with
    /// `keep_latin`), and when `auto_latin` turns a run Latin (the run was
    /// already committed as Cyrillic).
    ///
    /// Order for shells: delete `delete_before` characters, insert `commit`,
    /// then show the preedit.
    pub delete_before: usize,
    /// The commit holds a word that autocorrect changed (see
    /// [`Options::autocorrect`]). Shells can flash or underline it. Backspace
    /// right after the Space undoes the change.
    pub corrected: bool,
    /// What the composer learned from this key (see [`Options::learning`]). It
    /// is set only on a key that commits a word, or on the Backspace that
    /// undoes a correction, so it costs nothing on other keys. The shell
    /// applies it to its own [`crate::UserDict`] with [`crate::UserDict::apply`].
    pub learned: Option<Learned>,
}

/// True for characters that start/extend a composition: ASCII letters,
/// `'`, `"` and the umlauts `ö ü Ö Ü` (typed for ө and ү).
pub fn is_input_char(ch: char) -> bool {
    ch == '\'' || ch == '"' || ch.is_ascii_alphabetic() || matches!(ch, 'ö' | 'ü' | 'Ö' | 'Ü')
}

/// Behaviour switches for a [`Composer`].
///
/// The fields are public, so build a value with struct-update syntax to stay
/// source-compatible when options are added:
///
/// ```
/// use buuz_core::Options;
/// let opts = Options { smart_vowels: false, ..Default::default() };
/// # let _ = opts;
/// ```
#[derive(Debug, Clone)]
pub struct Options {
    /// Smart vowel harmony: look ahead for the word's gender (`unee` -> үнээ),
    /// choose ий or ы (`aaviin` -> аавын) and let the particle уу/үү follow the
    /// previous word. `false` is the original Buuz behaviour.
    pub smart_vowels: bool,
    /// Abbreviations to expand when a word is committed by Space or
    /// punctuation (`bn ` -> `байна `). `None` (the default, and for
    /// [`Options::classic`]) keeps the core faithful: no expansion.
    pub abbreviations: Option<Arc<Abbreviations>>,
    /// Keep Latin: Esc while composing shows the raw Latin keys (`ok` stays
    /// `ok`, not `ок`), and Backspace right after a word and a Space reopens
    /// that word in the preedit. `false` is the original Buuz behaviour.
    pub keep_latin: bool,
    /// Auto Latin: a run that holds `@`, `://` or a leading `www.`, or that
    /// ends as a domain (`github.com`), turns Latin by itself. Separate from
    /// `keep_latin`. `false` is the original Buuz behaviour.
    pub auto_latin: bool,
    /// Custom key mappings. `None` (the default, also for [`Options::smart`]
    /// and [`Options::classic`]) uses the shared built-in table. Build a
    /// [`KeyMap`] once and share it; keys cost nothing extra per keystroke.
    pub keys: Option<Arc<KeyMap>>,
    /// The loaded word list. `None` (the default, also for [`Options::smart`]
    /// and [`Options::classic`]): features that need words stay off. The shell
    /// loads it only when [`crate::Config::needs_word_list`] says so.
    pub words: Option<Arc<WordList>>,
    /// Completion: show the rest of a clear winner from `words` as ghost text,
    /// and let Tab (or Right at the end) accept it. Active only with `words`
    /// set. `false` is the original Buuz behaviour.
    pub completion: bool,
    /// Autocorrect: when Space or punctuation ends a word that is not in
    /// `words` but has one clear neighbour that is (`хуухэд` -> `хүүхэд`),
    /// commit the neighbour. Active only with `words` set. Backspace right after the
    /// Space undoes it. `false` is the original Buuz behaviour, and the
    /// default in every preset.
    pub autocorrect: bool,
    /// A read-only snapshot of the personal dictionary. `None` (the default,
    /// also for [`Options::smart`] and [`Options::classic`]) means no user
    /// words. The composer never changes it and takes no lock. The shell owns
    /// the live [`crate::UserDict`], applies [`KeyOutcome::learned`] to it, builds a
    /// new snapshot with [`crate::UserDict::snapshot`] and sets it with
    /// [`Composer::set_options`] when it wants the change to show.
    /// Recommended: refresh at the start of a field or after an idle pause,
    /// not after every word.
    pub user: Option<Arc<UserSnapshot>>,
    /// Learning: return [`KeyOutcome::learned`] events. `true` in smart mode,
    /// off by default in classic mode (a config override can turn it on).
    /// Shells set it to `false` for password fields and, where the platform
    /// tells them, for private windows. With `false` the composer records
    /// nothing.
    pub learning: bool,
    /// Double V: the key pair `vv` gives үү (`svv` is сүү). A single `v`
    /// stays в. The pair counts as feminine for vowel harmony, like `w`.
    /// `true` in smart mode, `false` in classic mode (the original Buuz gives
    /// вв).
    pub double_v: bool,
    /// Habit keys: `ts` gives ц and `zh` gives ж. With a word list, a stem
    /// that only exists with тс (`хүртсэн`) or зх keeps the two letters.
    /// `true` in smart mode, `false` in classic mode (the original Buuz gives
    /// тс and зх).
    pub habit_keys: bool,
    /// Word choice: when Space or punctuation ends a word whose `о` or `у`
    /// may be ө or ү (`tov`, `uvul`), commit the variant the word list
    /// clearly prefers (`төв`, `өвөл`). Active only with `words` set.
    /// Backspace right after the Space undoes it. `true` in smart mode,
    /// `false` in classic mode.
    pub word_choice: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            smart_vowels: true,
            abbreviations: None,
            keep_latin: true,
            auto_latin: true,
            keys: None,
            words: None,
            completion: true,
            autocorrect: false,
            user: None,
            learning: true,
            double_v: true,
            habit_keys: true,
            word_choice: true,
        }
    }
}

impl Options {
    /// Smart mode with every smart feature on: smart vowels and the built-in
    /// abbreviations. This is what the shells use by default; each feature can
    /// still be switched off one by one.
    pub fn smart() -> Self {
        Options {
            smart_vowels: true,
            abbreviations: Some(Abbreviations::builtin()),
            keep_latin: true,
            auto_latin: true,
            keys: None,
            words: None,
            completion: true,
            autocorrect: false,
            user: None,
            learning: true,
            double_v: true,
            habit_keys: true,
            word_choice: true,
        }
    }

    /// Classic mode: the original left-to-right Buuz behaviour, with every
    /// smart rule off. New smart features must leave this output unchanged.
    pub fn classic() -> Self {
        Options {
            smart_vowels: false,
            abbreviations: None,
            keep_latin: false,
            auto_latin: false,
            keys: None,
            words: None,
            completion: false,
            autocorrect: false,
            user: None,
            learning: false,
            double_v: false,
            habit_keys: false,
            word_choice: false,
        }
    }
}

/// What Backspace needs to reopen the word just committed by Space: an
/// abbreviation expansion, or (with [`Options::keep_latin`]) any word.
#[derive(Debug, Clone)]
struct Undo {
    /// The raw keys of the word; for a re-seeded word its Cyrillic text.
    raw: Vec<char>,
    /// The word was frozen (re-seeded) when it was committed: it reopens frozen.
    reseeded: bool,
    /// The word was shown in Latin (Esc) when it was committed.
    latin: bool,
    /// Characters to delete: the committed word plus the space.
    delete: usize,
    /// The previous-word gender the word was typed with.
    prev_gender: Option<u8>,
    /// For an automatic change: the Cyrillic run to commit instead of reopening
    /// a word.
    restore: Option<String>,
    /// Autocorrect changed the word: reopening it adds the typed word to the
    /// rejected set.
    corrected: bool,
}

/// Longest run (in characters) the composer tracks for Auto Latin.
const RUN_MAX: usize = 100;

/// Top-level domains that make `label.label` a domain at the end of a run.
/// Short ones that are also Mongolian words (`us`, `ai`, `de`, `me`...) are
/// left out: `tsai.de` after a full stop without a space is prose.
const TLDS: &[&str] = &[
    "com", "net", "org", "edu", "gov", "io", "dev", "app", "info", "mn", "ru", "biz", "xyz",
];

/// Punctuation that may sit before or after a link in prose.
fn is_lead_punct(c: char) -> bool {
    matches!(c, '(' | '[' | '"' | '\'')
}

fn is_trail_punct(c: char) -> bool {
    matches!(
        c,
        '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '"' | '\''
    )
}

/// [`is_domain`] on `a ++ b` after trimming surrounding punctuation.
fn is_domain_trimmed(a: &[char], b: &[char]) -> bool {
    let n = a.len() + b.len();
    let at = |i: usize| if i < a.len() { a[i] } else { b[i - a.len()] };
    let mut start = 0;
    while start < n && is_lead_punct(at(start)) {
        start += 1;
    }
    let mut end = n;
    while end > start && is_trail_punct(at(end - 1)) {
        end -= 1;
    }
    is_domain((start..end).map(at))
}

/// True if the keys read `label.label...tld`: labels of letters, digits or
/// `-`, at least one dot, and a last label from [`TLDS`].
fn is_domain(keys: impl Iterator<Item = char>) -> bool {
    let mut dots = 0;
    let mut label = [0u8; 8];
    let mut n = 0;
    for c in keys {
        if c == '.' {
            if n == 0 {
                return false;
            }
            dots += 1;
            n = 0;
        } else if c.is_ascii_alphanumeric() || c == '-' {
            if n < label.len() {
                label[n] = c.to_ascii_lowercase() as u8;
            }
            n += 1;
        } else {
            return false;
        }
    }
    dots > 0 && n > 0 && n <= label.len() && TLDS.iter().any(|t| t.as_bytes() == &label[..n])
}

/// Particle gender of expanded Cyrillic text: its last back vowel makes it
/// masculine, its last front vowel feminine. Like the typed-word rule, a ү
/// right after г is the suffix -гүй and is no evidence (байхгүй stays
/// masculine, мэдэхгүй stays feminine through its э).
fn text_gender(text: &str) -> Option<u8> {
    let chars: Vec<char> = text.chars().map(to_lower).collect();
    chars.iter().enumerate().rev().find_map(|(i, &c)| match c {
        'а' | 'о' | 'у' | 'я' | 'ё' | 'ю' => Some(X_M),
        'ү' if i > 0 && chars[i - 1] == 'г' => None,
        'э' | 'ө' | 'ү' | 'е' => Some(X_F),
        _ => None,
    })
}

/// One matched rule position in the raw keys (scratch data of `read_to_comp`).
#[derive(Debug, Clone)]
pub(crate) struct Token {
    start: usize,
    len: usize,
    /// Index of the rule group in the table, if the keys matched.
    group: Option<usize>,
}

/// What `read_to_comp` needs to know besides the raw keys.
#[derive(Debug, Clone, Copy)]
struct Context<'a> {
    smart: bool,
    /// The word list, to choose ий or ы by prefix (typing-rules 4.2).
    words: Option<&'a WordList>,
    /// The `X_DV` rules (`vv`) are active.
    double_v: bool,
    /// The `X_HK` rules (`ts`, `zh`) are active.
    habit_keys: bool,
    /// Gender of the word before a space, if the last key was a space.
    prev_gender: Option<u8>,
}

/// True if the first rule of `tok` gives exactly the lower case letter `ch`.
/// The smart rules look at what a key gives, so a custom key that gives the
/// same letter acts the same.
fn gives(rs: &RuleSet, tok: &Token, ch: char) -> bool {
    matches!(tok_rules(rs, tok).first(), Some(r) if r.to.len() == 1 && to_lower(r.to[0]) == ch)
}

/// The rules of a token: none when the keys matched nothing.
fn tok_rules<'a>(rs: &'a RuleSet, tok: &Token) -> &'a [Rule] {
    match tok.group {
        Some(g) => rs.group(g),
        None => &[],
    }
}

/// The gender this token proves for the word, if any.
fn evidence(rs: &RuleSet, tokens: &[Token], idx: usize) -> Option<u8> {
    let tok = &tokens[idx];
    // `ye` (е) is mostly a loanword vowel and says nothing about the word.
    if gives(rs, tok, 'е') {
        return None;
    }
    for rule in tok_rules(rs, tok) {
        if rule.flags & X_MF != 0 {
            // An explicit ү right after г is the suffix -гүй (болохгүй). The
            // pair `vv` is not: its second ү is evidence, as in `gww`.
            let explicit_u =
                rule.flags & X_DV == 0 && rule.to.first().map(|&c| to_lower(c)) == Some('ү');
            if explicit_u && idx > 0 && gives(rs, &tokens[idx - 1], 'г') {
                return None;
            }
            return Some(X_F);
        }
        if rule.flags & X_MM != 0 {
            return Some(X_M);
        }
    }
    None
}

fn is_vowel(c: char) -> bool {
    matches!(
        to_lower(c),
        'а' | 'э' | 'и' | 'о' | 'ө' | 'у' | 'ү' | 'ы' | 'е' | 'ё' | 'ю' | 'я'
    )
}

fn is_cyrillic_consonant(c: char) -> bool {
    matches!(c, 'а'..='я' | 'ё' | 'ө' | 'ү') && !is_vowel(c)
}

/// Should this `ii` become ы? See docs/typing-rules.md, section 4.
fn ii_is_y(out: &[char], word: u8) -> bool {
    if word != X_M || !out.iter().any(|&c| is_vowel(c)) {
        return false;
    }
    let prev = match out.last() {
        Some(&c) => to_lower(c),
        None => return false,
    };
    if !is_cyrillic_consonant(prev) || KEEP_II.contains(&prev) {
        return false;
    }
    !II_FINAL_STEMS.iter().any(|stem| {
        out.len() == stem.chars().count()
            && out.iter().zip(stem.chars()).all(|(&c, s)| to_lower(c) == s)
    })
}

/// Word list check for `ii` after the text so far (`out`): true when some
/// listed word starts with `out` + ий and none starts with `out` + ы.
/// `buf` is a reused scratch string; it does not allocate once it has grown.
fn list_prefers_ii(words: &WordList, out: &[char], buf: &mut String) -> bool {
    let mut probe = |tail: &str| {
        buf.clear();
        buf.extend(out.iter().map(|&c| to_lower(c)));
        buf.push_str(tail);
        words.has_prefix(buf)
    };
    !probe("ы") && probe("ий")
}

/// True when the keys `rest` (the part of a word that starts at the `s` of
/// `ts`) start a key longer than one character, such as `sh` or `sxc`.
fn s_starts_longer_key(rs: &RuleSet, rest: &[char]) -> bool {
    rs.lengths
        .iter()
        .any(|&l| l >= 2 && l <= rest.len() && rs.get(&rest[..l]).is_some())
}

/// A change to `out` that the habit check may make: the letter at `at`
/// becomes the two letters `alt`. A habit key (`ts`, `zh`) gives ц or ж that
/// may become тс or зх. The smart ий rule gives ы that may become ий again
/// when its stem changes.
#[derive(Debug, Clone, Copy, Default)]
struct Edit {
    at: usize,
    alt: [char; 2],
}

/// The habit keys of a word (at most `HABIT_MAX`, checked against the word
/// list) and the first ы of the ий rule. They are in the order of `at`.
#[derive(Debug, Default)]
struct Edits {
    list: [Edit; HABIT_MAX + 1],
    n: usize,
    /// Indexes into `list`.
    habits: [usize; HABIT_MAX],
    n_habits: usize,
    ii: Option<usize>,
}

impl Edits {
    fn push_habit(&mut self, e: Edit) {
        if self.n_habits < HABIT_MAX {
            self.habits[self.n_habits] = self.n;
            self.n_habits += 1;
            self.list[self.n] = e;
            self.n += 1;
        }
    }

    fn push_ii(&mut self, e: Edit) {
        if self.ii.is_none() {
            self.ii = Some(self.n);
            self.list[self.n] = e;
            self.n += 1;
        }
    }
}

/// At most this many habit keys in a word are checked against the word list.
const HABIT_MAX: usize = 2;

/// Word list check for the habit keys (typing-rules 3.1). `out` holds ц or ж
/// at each habit position. When the whole text so far is not the start of a
/// listed word, try тс (зх) at each single position, then at both, and keep
/// the first variant that starts a listed word. A variant is tried with the
/// ы of the ий rule as it is, then with ий: the stem changed, so the ий
/// check no longer holds (`батсуурийн`). `buf` is a reused scratch string.
/// Only a winning variant changes `out`.
fn apply_habit_list(words: &WordList, edits: &Edits, out: &mut Vec<char>, buf: &mut String) {
    let list = &edits.list[..edits.n];
    let probe = |out: &[char], mask: u8, buf: &mut String| -> bool {
        buf.clear();
        for (i, &c) in out.iter().enumerate() {
            match list.iter().position(|e| e.at == i) {
                Some(k) if mask & (1 << k) != 0 => {
                    buf.extend(list[k].alt.iter().map(|&a| to_lower(a)))
                }
                _ => buf.push(to_lower(c)),
            }
        }
        words.has_prefix(buf)
    };
    if probe(out, 0, buf) {
        return;
    }
    // Each single habit key, then both.
    let mut masks = [0u8; HABIT_MAX + 1];
    let mut n_masks = 0;
    for &h in edits.habits.iter().take(edits.n_habits) {
        masks[n_masks] = 1 << h;
        n_masks += 1;
    }
    if edits.n_habits > 1 {
        masks[n_masks] = masks[0] | masks[1];
        n_masks += 1;
    }
    let ii_bit = edits.ii.map(|k| 1u8 << k);
    let found = masks[..n_masks].iter().find_map(|&m| {
        if probe(out, m, buf) {
            return Some(m);
        }
        let with_ii = m | ii_bit?;
        probe(out, with_ii, buf).then_some(with_ii)
    });
    let Some(mask) = found else {
        return;
    };
    // Later positions first, so the earlier indexes stay valid.
    for k in (0..edits.n).rev() {
        if mask & (1 << k) != 0 {
            let e = list[k];
            out[e.at] = e.alt[0];
            out.insert(e.at + 1, e.alt[1]);
        }
    }
}

/// Transliterate a raw (possibly mixed Latin/Cyrillic) string as one word.
/// Returns the text and the gender the word leaves for a following particle.
fn read_to_comp(
    rs: &RuleSet,
    read: &[char],
    ctx: Context,
    tokens: &mut Vec<Token>,
    buf: &mut String,
) -> (Vec<char>, u8) {
    // Pass 0: split the keys into longest-match tokens.
    tokens.clear();
    let mut i = 0;
    while i < read.len() {
        let mut len = 1;
        let mut group = None;
        for &l in &rs.lengths {
            if l <= read.len() - i {
                if let Some((g, rules)) = rs.get(&read[i..i + l]) {
                    // An `X_DV` rule (`vv`) exists only with the option on.
                    if !ctx.double_v && rules.iter().all(|r| r.flags & X_DV != 0) {
                        continue;
                    }
                    // The same for an `X_HK` rule (`ts`, `zh`). `ts` also gives
                    // way when the `s` starts a longer key: `tsh` stays т and
                    // ш, `tsxc` stays т and щ.
                    if rules.iter().all(|r| r.flags & X_HK != 0)
                        && (!ctx.habit_keys
                            || (to_lower(rules[0].to[0]) == 'ц'
                                && s_starts_longer_key(rs, &read[i + 1..])))
                    {
                        continue;
                    }
                    len = l;
                    group = Some(g);
                    break;
                }
            }
        }
        tokens.push(Token {
            start: i,
            len,
            group,
        });
        i += len;
    }
    // Pass 1: the word's gender from the first (and last) deciding vowel.
    let mut initial = X_M;
    let mut last_ev = None;
    if ctx.smart {
        let mut first_ev = None;
        for idx in 0..tokens.len() {
            if let Some(g) = evidence(rs, tokens, idx) {
                first_ev.get_or_insert(g);
                last_ev = Some(g);
            }
        }
        let particle = read.iter().all(|&c| to_lower(c) == 'u') && read.len() <= 2;
        initial = match ctx.prev_gender {
            Some(g) if particle => g,
            _ => first_ev.unwrap_or(X_M),
        };
    }
    // Pass 2: convert left to right, starting from that gender.
    let mut word = initial;
    let mut out = Vec::with_capacity(read.len());
    let mut edits = Edits::default();
    for tok in tokens.iter() {
        let chosen = tok_rules(rs, tok).iter().find(|r| word & r.flags == word);
        match chosen {
            Some(rule) => {
                if rule.flags & X_MF != 0 {
                    word = X_F;
                } else if rule.flags & X_MM != 0 {
                    word = X_M;
                }
                if rule.flags & X_HK != 0 {
                    let (a, b) = if to_lower(rule.to[0]) == 'ж' {
                        ('з', 'х')
                    } else {
                        ('т', 'с')
                    };
                    let cased = |l: char, key: char| {
                        if key != to_lower(key) {
                            l.to_uppercase().next().unwrap_or(l)
                        } else {
                            l
                        }
                    };
                    edits.push_habit(Edit {
                        at: out.len(),
                        alt: [cased(a, read[tok.start]), cased(b, read[tok.start + 1])],
                    });
                    out.extend_from_slice(&rule.to);
                } else if ctx.smart
                    && rule.flags & X_II != 0
                    && ii_is_y(&out, word)
                    && !ctx.words.is_some_and(|w| list_prefers_ii(w, &out, buf))
                {
                    let upper = rule.to[0] != to_lower(rule.to[0]);
                    if let [a, b, ..] = rule.to[..] {
                        edits.push_ii(Edit {
                            at: out.len(),
                            alt: [a, b],
                        });
                    }
                    out.push(if upper { 'Ы' } else { 'ы' });
                } else {
                    out.extend_from_slice(&rule.to);
                }
            }
            None => out.extend_from_slice(&read[tok.start..tok.start + tok.len]),
        }
    }
    if edits.n_habits > 0 {
        if let Some(w) = ctx.words {
            apply_habit_list(w, &edits, &mut out, buf);
        }
    }
    (out, last_ev.unwrap_or(initial))
}

/// Stateful composition buffer.
#[derive(Debug, Clone)]
pub struct Composer {
    /// Raw input; becomes the Cyrillic text after cursor movement.
    read: Vec<char>,
    comp: Vec<char>,
    /// Cursor position within `comp`, in characters.
    cursor: usize,
    opts: Options,
    /// Particle gender of the current composition (set by `recompose`).
    word_gender: u8,
    /// Gender of the word before the last space, if the last key was a space.
    prev_gender: Option<u8>,
    /// The composition was re-seeded from Cyrillic (caret moved): never expands.
    reseeded: bool,
    /// The preedit holds a word that Tab (or Right) accepted from the ghost
    /// text. It is re-seeded, but it still counts as a use of that word.
    accepted: bool,
    /// Backspace just undid an expansion: this word will not expand again.
    no_expand: bool,
    /// Esc switched the preedit to the raw Latin keys.
    latin: bool,
    /// Backspace undid a correction: this word is not corrected again (also
    /// when the rejected set is full).
    no_correct: bool,
    /// Set right after a word committed by Space that Backspace can reopen.
    undo: Option<Undo>,
    /// Raw keys of the finished words and symbols of the current run (Auto
    /// Latin). The word being typed is in `read`.
    run: Vec<char>,
    /// What the application shows for `run`: the committed text and symbols.
    run_text: String,
    /// The run is tracked: no caret move, no outside edit, at most `RUN_MAX`.
    run_ok: bool,
    /// The run turned Latin: words in it start in the Latin view.
    run_latin: bool,
    /// A spare buffer for `Undo::raw`. Arming an undo happens after every word,
    /// so it copies into this recycled `Vec` instead of cloning `read`: no
    /// allocation per word once the buffer has grown to the longest word.
    spare: Vec<char>,
    /// Scratch buffer reused across keystrokes.
    tokens: Vec<Token>,
    /// Scratch for the word list check of `ii`.
    ii_buf: String,
    /// The preedit in lower case, reused by `refresh_ghost` on every key.
    lower: String,
    /// The ghost text after the preedit; valid when `ghost_on`.
    ghost: String,
    ghost_on: bool,
    /// Words (lower case, as typed) whose correction the user undid in this
    /// session. An empty set does not allocate: memory is taken at the first
    /// undo. At most `MAX_REJECTED` words.
    rejected: HashSet<String>,
    /// Buffers of the autocorrect search.
    fix: Scratch,
}

impl Default for Composer {
    fn default() -> Self {
        Self::with_options(Options::default())
    }
}

impl Composer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_options(opts: Options) -> Self {
        Composer {
            read: Vec::new(),
            comp: Vec::new(),
            cursor: 0,
            opts,
            word_gender: X_M,
            prev_gender: None,
            reseeded: false,
            accepted: false,
            no_expand: false,
            latin: false,
            no_correct: false,
            undo: None,
            run: Vec::new(),
            run_text: String::new(),
            run_ok: true,
            run_latin: false,
            spare: Vec::new(),
            tokens: Vec::new(),
            ii_buf: String::new(),
            lower: String::new(),
            ghost: String::new(),
            ghost_on: false,
            rejected: HashSet::new(),
            fix: Scratch::default(),
        }
    }

    /// Change the options; takes effect from the next recomposition.
    pub fn set_options(&mut self, opts: Options) {
        self.opts = opts;
        self.refresh_ghost();
    }

    /// Forget the previous word (the particle `uu` then stays masculine) and
    /// any pending undo. Shells must call this on any caret move or focus
    /// change the composer did not see (a click, a window switch, an arrow key
    /// sent while nothing was composed and handled elsewhere).
    pub fn clear_context(&mut self) {
        self.prev_gender = None;
        self.drop_undo();
        self.reset_run();
    }

    pub fn is_composing(&self) -> bool {
        !self.comp.is_empty()
    }

    /// Current preedit (composition) text.
    pub fn preedit(&self) -> String {
        self.comp.iter().collect()
    }

    /// Cursor position inside [`preedit`](Self::preedit), in `char`s.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The ghost text to show after the preedit, when the list has a clear
    /// winner: the word list is set, the preedit is Cyrillic (not the Latin
    /// view, not re-seeded) with the caret at the end and at least 3 letters
    /// (converted Cyrillic letters, not typed keys), and the best word for it is longer and at least twice as frequent as
    /// the second best (or the only one). Upper case when the preedit is all
    /// upper case; otherwise lower case. Tab accepts it. Borrows the composer's
    /// buffer: no allocation (FFI and wasm layers copy at their boundary).
    pub fn completion(&self) -> Option<&str> {
        self.ghost_on.then_some(self.ghost.as_str())
    }

    /// Recompute the ghost. Runs after every key; reuses its buffers, so it
    /// does not allocate once they have grown.
    fn refresh_ghost(&mut self) {
        self.ghost_on = false;
        let words = match &self.opts.words {
            Some(w) if self.opts.completion => w,
            _ => return,
        };
        let n = self.comp.len();
        if self.latin || self.reseeded || n < 3 || self.cursor != n {
            return;
        }
        self.lower.clear();
        self.lower.extend(self.comp.iter().map(|&c| to_lower(c)));
        // A user word wins over a list word with the same prefix.
        let user_word = self
            .opts
            .user
            .as_ref()
            .and_then(|u| u.completion(&self.lower));
        let (word, count, second) = match user_word {
            Some(w) => (w, 0, None),
            None => {
                let (top, second) = words.top_two(&self.lower);
                let Some((word, count)) = top else { return };
                (word, count, second)
            }
        };
        let clear = match second {
            None => true,
            Some((_, c2)) => u64::from(count) >= 2 * u64::from(c2),
        };
        if !clear || word.len() <= self.lower.len() {
            return;
        }
        let suffix = &word[self.lower.len()..];
        if n + suffix.chars().count() > MAX_COMP_LEN {
            return;
        }
        let upper = self.comp.iter().all(|c| c.is_uppercase());
        self.ghost.clear();
        for ch in suffix.chars() {
            if upper {
                self.ghost.extend(ch.to_uppercase());
            } else {
                self.ghost.push(ch);
            }
        }
        self.ghost_on = true;
    }

    /// Accept the ghost: the preedit becomes preedit plus suffix, frozen as
    /// Cyrillic like after a caret move.
    fn accept_ghost(&mut self) {
        let suffix = std::mem::take(&mut self.ghost);
        self.accepted = true;
        self.comp.extend(suffix.chars());
        self.ghost = suffix;
        self.comp_to_read();
        // The accepted word decides the gender it leaves for a particle.
        self.recompose();
        self.cursor = self.comp.len();
    }

    /// Raw input buffer (what the user typed).
    pub fn raw(&self) -> String {
        self.read.iter().collect()
    }

    /// Whether the IME wants this key (original `ImeProcessKey`).
    /// Pure query: does not modify state.
    pub fn wants_key(&self, key: Key) -> bool {
        match key {
            Key::Char(c) if is_input_char(c) => true,
            Key::Backspace if self.undo.is_some() => true,
            Key::Escape => self.opts.keep_latin && !self.comp.is_empty(),
            Key::Tab => self.ghost_on,
            _ if self.comp.is_empty() => false,
            Key::Backspace
            | Key::Delete
            | Key::Modifier
            | Key::Left
            | Key::Right
            | Key::Home
            | Key::End => true,
            _ => false,
        }
    }

    /// Commit the composition as typed (e.g. on focus loss). This never
    /// expands an abbreviation; Space and punctuation do, via `process_key`.
    /// Also forgets the previous word and any pending undo, since the next key
    /// may land in a different place.
    pub fn commit(&mut self) -> Option<String> {
        self.prev_gender = None;
        self.drop_undo();
        self.reset_run();
        self.take_composition()
    }

    /// Hand out the composition and reset the buffer; keeps the context.
    fn take_composition(&mut self) -> Option<String> {
        if self.comp.is_empty() {
            return None;
        }
        let s = self.preedit();
        self.clear_buffer();
        Some(s)
    }

    /// Discard the composition without inserting anything.
    pub fn cancel(&mut self) {
        self.clear_buffer();
        self.prev_gender = None;
        self.drop_undo();
        self.reset_run();
    }

    /// Drop the composition and any pending undo (but keep the previous-word
    /// context, as before).
    pub fn reset(&mut self) {
        self.clear_buffer();
        self.drop_undo();
        self.reset_run();
    }

    /// A caret move or an outside edit: forget the run, and track nothing
    /// until the next Space, Enter or Tab.
    fn stop_run(&mut self) {
        self.reset_run();
        self.run_ok = false;
    }

    /// Start a new run; keeps the buffers.
    fn reset_run(&mut self) {
        self.run.clear();
        self.run_text.clear();
        self.run_ok = true;
        self.run_latin = false;
    }

    /// Forget the pending undo, keeping its buffer for the next one.
    fn drop_undo(&mut self) {
        if let Some(u) = self.undo.take() {
            self.spare = u.raw;
        }
    }

    /// Arm the undo for the word in `read`, copying it into the spare buffer.
    fn arm_undo(&mut self, latin: bool, delete: usize) {
        let mut raw = std::mem::take(&mut self.spare);
        raw.clear();
        raw.extend_from_slice(&self.read);
        self.undo = Some(Undo {
            raw,
            reseeded: self.reseeded,
            latin,
            delete,
            prev_gender: self.prev_gender,
            restore: None,
            corrected: false,
        });
    }

    /// Empty the buffer; the previous-word context and the undo stay.
    fn clear_buffer(&mut self) {
        self.ghost_on = false;
        self.reseeded = false;
        self.accepted = false;
        self.no_expand = false;
        self.no_correct = false;
        self.latin = false;
        self.read.clear();
        self.comp.clear();
        self.cursor = 0;
    }

    fn recompose(&mut self) {
        if self.latin {
            // Shown as typed. A Latin word gives no evidence for the particle.
            self.comp.clear();
            self.comp.extend_from_slice(&self.read);
            self.word_gender = X_M;
            return;
        }
        let ctx = Context {
            smart: self.opts.smart_vowels,
            words: self.opts.words.as_deref(),
            double_v: self.opts.double_v,
            habit_keys: self.opts.habit_keys,
            prev_gender: self.prev_gender,
        };
        let rs = match &self.opts.keys {
            Some(k) => k.rules(),
            None => rules(),
        };
        let (comp, gender) = read_to_comp(rs, &self.read, ctx, &mut self.tokens, &mut self.ii_buf);
        self.comp = comp;
        self.word_gender = gender;
    }

    fn comp_to_read(&mut self) {
        self.reseeded = true;
        self.stop_run();
        self.read = self.comp.clone();
    }

    /// (back cursor distance, cursor index within `read`)
    fn read_cursor(&self) -> (usize, usize) {
        let back = self.comp.len().saturating_sub(self.cursor);
        (back, self.read.len().saturating_sub(back))
    }

    /// Commit the word at Space or punctuation, expanding an abbreviation if
    /// one matches. Returns the text and the gender it leaves for the next
    /// word. After a Space expansion, or any word with `keep_latin`, the undo
    /// is armed (it needs the previous-word gender, so this runs before the
    /// caller updates it).
    fn finish_word(
        &mut self,
        space: bool,
        may_correct: bool,
    ) -> (Option<String>, Option<u8>, bool, Option<Learned>) {
        let expansion = match &self.opts.abbreviations {
            Some(abbr)
                if !self.reseeded && !self.no_expand && !self.latin && !self.comp.is_empty() =>
            {
                abbr.lookup(&self.read)
            }
            _ => None,
        };
        match expansion {
            Some(text) => {
                let gender = text_gender(&text);
                if space {
                    self.arm_undo(false, text.chars().count() + 1);
                }
                self.clear_buffer();
                (Some(text), gender, false, None)
            }
            None => {
                if let Some(text) = self.autocorrected(may_correct) {
                    let gender = text_gender(&text);
                    if space {
                        self.arm_undo(false, text.chars().count() + 1);
                        if let Some(u) = self.undo.as_mut() {
                            u.corrected = true;
                        }
                    }
                    self.clear_buffer();
                    return (Some(text), gender, true, None);
                }
                // A re-seeded word reopens from its Cyrillic text, still frozen.
                if space && self.opts.keep_latin && !self.comp.is_empty() {
                    self.arm_undo(self.latin, self.comp.len() + 1);
                }
                let gender = Some(self.word_gender);
                let learned = self.used_event(may_correct);
                (self.take_composition(), gender, false, learned)
            }
        }
    }

    /// The corrected spelling of the word in `comp`, if autocorrect is on and
    /// finds one. `may_correct` is true only for Space and punctuation, and
    /// false for a word that will not stay Cyrillic (the end of a domain).
    /// Runs once per word.
    fn autocorrected(&mut self, may_correct: bool) -> Option<String> {
        let words = match &self.opts.words {
            Some(w) if self.opts.autocorrect || self.opts.word_choice => w,
            _ => return None,
        };
        if !may_correct || self.no_correct || self.latin || self.run_latin || self.reseeded {
            return None;
        }
        if self.opts.word_choice {
            let chosen = autocorrect::choose(
                words,
                &self.comp,
                &self.rejected,
                self.opts.user.as_deref(),
                &mut self.fix,
            );
            if chosen.is_some() || !self.opts.autocorrect {
                return chosen;
            }
        }
        autocorrect::correct(
            words,
            &self.comp,
            &self.rejected,
            self.opts.user.as_deref(),
            &mut self.fix,
        )
    }

    /// Word choice for a word that the whole-text converter ends without a
    /// Space or punctuation: at a new line or at the end of the text. Only
    /// word choice runs here, not autocorrect, so a word that is still being
    /// typed is not spelled again. On a choice the buffer is cleared.
    fn choose_at_end(&mut self) -> Option<String> {
        let words = self.opts.words.as_ref().filter(|_| self.opts.word_choice)?;
        if self.no_correct || self.latin || self.run_latin || self.reseeded || self.comp.is_empty()
        {
            return None;
        }
        let chosen = autocorrect::choose(
            words,
            &self.comp,
            &self.rejected,
            self.opts.user.as_deref(),
            &mut self.fix,
        )?;
        self.clear_buffer();
        Some(chosen)
    }

    /// The word in `comp` in lower case.
    fn lower_comp(&self) -> String {
        self.comp.iter().map(|&c| to_lower(c)).collect()
    }

    /// The `Used` event for the word in `comp` that Space or punctuation
    /// commits as typed: only with learning on, in the Cyrillic view, and for
    /// a word that is in the word list or the user dictionary. A word that
    /// Backspace reopened is not counted again. The check
    /// reuses a buffer; only an event allocates.
    fn used_event(&mut self, may_correct: bool) -> Option<Learned> {
        if !self.opts.learning
            || !may_correct
            || self.latin
            || self.run_latin
            || (self.reseeded && !self.accepted)
            || self.no_expand
            || self.comp.is_empty()
        {
            return None;
        }
        self.lower.clear();
        self.lower.extend(self.comp.iter().map(|&c| to_lower(c)));
        let known = self
            .opts
            .words
            .as_ref()
            .is_some_and(|w| w.contains(&self.lower))
            || self
                .opts
                .user
                .as_ref()
                .is_some_and(|u| u.is_added(&self.lower));
        known.then(|| Learned::Used(self.lower.clone()))
    }

    /// Remember the word in `comp` as one the user does not want corrected.
    fn reject_current(&mut self) {
        let word = self.lower_comp();
        if self.rejected.len() < autocorrect::MAX_REJECTED {
            self.rejected.insert(word);
        }
    }

    /// Record the finished word in `read` and the symbol `c` in the run, or
    /// stop tracking a run that gets too long.
    fn run_push(&mut self, c: char) {
        if !self.run_ok {
            return;
        }
        if self.run.len() + self.read.len() + 1 > RUN_MAX {
            self.run_ok = false;
            self.run.clear();
            self.run_text.clear();
            return;
        }
        self.run.extend_from_slice(&self.read);
        self.run.push(c);
    }

    /// Live triggers: `@`, `://` and a leading `www.` turn the run Latin at
    /// the key that completes them. The key is consumed.
    fn try_trigger(&mut self, c: char, out: &mut KeyOutcome) -> bool {
        if !self.run_ok || self.run_latin || self.run.len() + self.read.len() + 1 > RUN_MAX {
            return false;
        }
        let hit = match c {
            '@' => true,
            '/' => {
                (self.read.is_empty() && self.run.ends_with(&[':', '/']))
                    || is_domain_trimmed(&self.run, &self.read)
            }
            _ => false,
        };
        if !hit {
            return false;
        }
        let mut raw: String = self.run.iter().chain(self.read.iter()).collect();
        raw.push(c);
        out.consumed = true;
        out.delete_before = self.run_text.chars().count();
        out.commit = Some(raw);
        self.clear_buffer();
        self.prev_gender = None;
        self.reset_run();
        self.run_ok = false;
        self.run_latin = true;
        true
    }

    /// If the run ends here as a domain: its raw keys and the number of
    /// characters already committed for it.
    fn domain_run(&self) -> Option<(String, usize)> {
        if !self.opts.auto_latin || !self.run_ok || self.run_latin {
            return None;
        }
        if self.run.len() + self.read.len() > RUN_MAX || !is_domain_trimmed(&self.run, &self.read) {
            return None;
        }
        let raw = self.run.iter().chain(self.read.iter()).collect();
        Some((raw, self.run_text.chars().count()))
    }

    /// Replace the commit of a run end by the raw run. A Space arms the undo
    /// that puts the Cyrillic run back.
    fn make_latin(&mut self, out: &mut KeyOutcome, raw: String, delete: usize, space: bool) {
        let mut cyrillic = std::mem::take(&mut self.run_text);
        if let Some(tail) = out.commit.take() {
            cyrillic.push_str(&tail);
        }
        self.drop_undo();
        if space {
            let mut buf = std::mem::take(&mut self.spare);
            buf.clear();
            self.undo = Some(Undo {
                raw: buf,
                reseeded: false,
                latin: false,
                delete: raw.chars().count() + 1,
                prev_gender: None,
                restore: Some(cyrillic),
                corrected: false,
            });
        }
        out.delete_before = delete;
        out.commit = Some(raw);
        self.prev_gender = None;
    }

    /// The end of the text (`convert_with`) ends the run like a Space would,
    /// without a key: a domain turns Latin. Returns the raw run and the
    /// characters to delete.
    fn finish_text(&mut self) -> Option<(String, usize)> {
        let found = self.domain_run();
        if found.is_some() {
            self.clear_buffer();
        }
        found
    }

    /// Process one key. Ports `ImeProcessKey` + `ImeToAsciiEx`.
    pub fn process_key(&mut self, key: Key) -> KeyOutcome {
        let accept = self.ghost_on
            && (key == Key::Tab || (key == Key::Right && self.cursor == self.comp.len()));
        let out = if accept {
            self.accept_ghost();
            KeyOutcome {
                consumed: true,
                ..KeyOutcome::default()
            }
        } else {
            // Without a completion Tab is exactly `Other`.
            let key = if key == Key::Tab { Key::Other } else { key };
            self.process_key_inner(key)
        };
        self.refresh_ghost();
        out
    }

    fn process_key_inner(&mut self, key: Key) -> KeyOutcome {
        let mut out = KeyOutcome::default();
        // Without Auto Latin a symbol is just `Other`.
        // Only Space and punctuation correct a word; Symbol may become Other.
        let punctuation = matches!(key, Key::Symbol(_));
        let key = match key {
            Key::Symbol(_) if !self.opts.auto_latin => Key::Other,
            k => k,
        };
        // Only a key right after the expansion can undo it.
        // Shift/CapsLock alone is not such a key.
        // (An armed undo means the composition is empty.)
        if key != Key::Modifier {
            if let Some(mut u) = self.undo.take() {
                if key == Key::Backspace && u.restore.is_some() {
                    // An automatic change: put the Cyrillic run back.
                    self.spare = u.raw;
                    self.prev_gender = None;
                    out.consumed = true;
                    out.delete_before = u.delete;
                    out.commit = u.restore;
                    return out;
                }
                if key == Key::Backspace {
                    // `read` is empty here: swap, so both buffers are kept.
                    std::mem::swap(&mut self.read, &mut u.raw);
                    self.spare = u.raw;
                    self.latin = u.latin;
                    self.no_expand = true;
                    self.reseeded = u.reseeded;
                    self.prev_gender = u.prev_gender;
                    self.recompose();
                    self.cursor = self.comp.len();
                    if u.corrected {
                        self.no_correct = true;
                        self.reject_current();
                        if self.opts.learning {
                            out.learned = Some(Learned::Rejected(self.lower_comp()));
                        }
                    }
                    out.consumed = true;
                    out.delete_before = u.delete;
                    return out;
                }
                self.spare = u.raw;
            }
        }
        if !self.wants_key(key) {
            let auto = self.opts.auto_latin;
            let mut domain = None;
            let mut made_latin = false;
            if auto {
                match key {
                    Key::Symbol(c) => {
                        if self.try_trigger(c, &mut out) {
                            return out;
                        }
                        self.run_push(c);
                    }
                    Key::Space | Key::Other => domain = self.domain_run(),
                    Key::Modifier => {}
                    // Backspace, caret keys, Esc: the text may have changed.
                    _ => self.stop_run(),
                }
            }
            // Not ours: finish any composition, let the key through.
            let (commit, gender, corrected, learned) = self.finish_word(
                key == Key::Space,
                (key == Key::Space || punctuation) && domain.is_none(),
            );
            out.commit = commit;
            out.corrected = corrected;
            out.learned = learned;
            if auto {
                match key {
                    Key::Symbol(c) if self.run_ok => {
                        if let Some(s) = &out.commit {
                            self.run_text.push_str(s);
                        }
                        self.run_text.push(c);
                    }
                    Key::Space | Key::Other => {
                        if let Some((raw, delete)) = domain {
                            made_latin = true;
                            self.make_latin(&mut out, raw, delete, key == Key::Space);
                        }
                        self.reset_run();
                    }
                    _ => {}
                }
            }
            // A space right after a word hands that word's gender to the next
            // one; repeated spaces keep it; any other key forgets it.
            match key {
                Key::Space if out.commit.is_some() => self.prev_gender = gender,
                // Shift/CapsLock alone must not break the context.
                Key::Space | Key::Modifier => {}
                _ => self.prev_gender = None,
            }
            // A run that turned Latin leaves no gender for a particle.
            if made_latin {
                self.prev_gender = None;
            }
            return out;
        }
        out.consumed = true;
        match key {
            Key::Char(c) => {
                if self.read.len() >= MAX_COMP_LEN {
                    out.rejected = true;
                    return out;
                }
                if self.run_latin && self.read.is_empty() {
                    self.latin = true;
                }
                let (back, rc) = self.read_cursor();
                self.read.insert(rc, c);
                self.recompose();
                self.cursor = self.comp.len().saturating_sub(back);
            }
            Key::Backspace => {
                let (back, rc) = self.read_cursor();
                if rc > 0 {
                    self.read.remove(rc - 1);
                    self.recompose();
                    self.cursor = self.comp.len().saturating_sub(back);
                }
            }
            Key::Delete => {
                let (_, rc) = self.read_cursor();
                if rc < self.read.len() {
                    self.read.remove(rc);
                    self.recompose();
                    self.cursor = self.cursor.min(self.comp.len());
                }
            }
            Key::Left if self.cursor > 0 => {
                self.comp_to_read();
                self.cursor -= 1;
            }
            Key::Right if self.cursor < self.comp.len() => {
                self.comp_to_read();
                self.cursor += 1;
            }
            Key::Home if self.cursor > 0 => {
                self.comp_to_read();
                self.cursor = 0;
            }
            // Only reached with keep_latin and a composition (see wants_key).
            // A re-seeded word has no raw keys, so Esc leaves it alone.
            Key::Escape if !self.reseeded => {
                self.latin = !self.latin;
                self.recompose();
                // Without a re-seed the caret is always at the end.
                self.cursor = self.comp.len();
            }
            Key::End if self.cursor < self.comp.len() => {
                self.comp_to_read();
                self.cursor = self.comp.len();
            }
            _ => {}
        }
        if self.comp.is_empty() {
            self.clear_buffer(); // emptied by editing: cancel, as the original does
        }
        out
    }
}

/// Convenience: transliterate a whole string, the way typing it would.
/// Input characters compose; any other character ends the current word and
/// is copied through (`"sain baina uu"` -> `"сайн байна уу"`).
pub fn convert(input: &str) -> String {
    convert_with(input, &Options::default())
}

/// Like [`convert`], with explicit [`Options`] (for example
/// [`Options::classic`]).
pub fn convert_with(input: &str, opts: &Options) -> String {
    let mut c = Composer::with_options(opts.clone());
    let mut out = String::new();
    for ch in input.chars() {
        let key = if is_input_char(ch) {
            Key::Char(ch)
        } else if ch == ' ' {
            Key::Space
        } else if ch.is_alphabetic() || ch.is_whitespace() || ch.is_control() {
            Key::Other
        } else {
            Key::Symbol(ch)
        };
        if ch == '\n' {
            if let Some(s) = c.choose_at_end() {
                out.push_str(&s);
            }
        }
        let r = c.process_key(key);
        for _ in 0..r.delete_before {
            out.pop();
        }
        if let Some(s) = r.commit {
            out.push_str(&s);
        }
        if !r.consumed {
            out.push(ch);
        }
    }
    // The end of the text is not a typed Space or punctuation, so a trailing
    // abbreviation stays as typed (бн until a Space follows), like `commit`.
    if let Some((raw, delete)) = c.finish_text() {
        for _ in 0..delete {
            out.pop();
        }
        out.push_str(&raw);
    }
    if let Some(s) = c.choose_at_end().or_else(|| c.commit()) {
        out.push_str(&s);
    }
    out
}
