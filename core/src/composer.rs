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

use std::sync::Arc;

use crate::abbrev::Abbreviations;
use crate::rules::{rules, to_lower, Rule, II_FINAL_STEMS, KEEP_II, X_F, X_M, X_MF, X_MM};

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
    /// Anything else (space, Enter, Tab, digits, punctuation, Esc...).
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
    /// application must delete before showing the preedit. Non-zero only when
    /// Backspace undoes an abbreviation expansion.
    pub delete_before: usize,
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
}

impl Default for Options {
    fn default() -> Self {
        Options {
            smart_vowels: true,
            abbreviations: None,
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
        }
    }

    /// Classic mode: the original left-to-right Buuz behaviour, with every
    /// smart rule off. New smart features must leave this output unchanged.
    pub fn classic() -> Self {
        Options {
            smart_vowels: false,
            abbreviations: None,
        }
    }
}

/// What Backspace needs to take back an expansion committed by Space.
#[derive(Debug, Clone)]
struct Undo {
    /// The raw keys of the abbreviation.
    raw: Vec<char>,
    /// Characters to delete: the expansion plus the space.
    delete: usize,
    /// The previous-word gender the abbreviation was typed with.
    prev_gender: Option<u8>,
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
    cands: &'static [Rule],
}

/// What `read_to_comp` needs to know besides the raw keys.
#[derive(Debug, Clone, Copy)]
struct Context {
    smart: bool,
    /// Gender of the word before a space, if the last key was a space.
    prev_gender: Option<u8>,
}

/// True if the lowercased keys of `tok` are exactly `pat` (lower case ASCII).
fn token_is(read: &[char], tok: &Token, pat: &str) -> bool {
    let keys = &read[tok.start..tok.start + tok.len];
    keys.len() == pat.len() && keys.iter().zip(pat.chars()).all(|(&k, p)| to_lower(k) == p)
}

/// The gender this token proves for the word, if any.
fn evidence(read: &[char], tokens: &[Token], idx: usize) -> Option<u8> {
    let tok = &tokens[idx];
    // `ye` (е) is mostly a loanword vowel and says nothing about the word.
    if token_is(read, tok, "ye") {
        return None;
    }
    for rule in tok.cands {
        if rule.flags & X_MF != 0 {
            // An explicit ү right after г is the suffix -гүй (болохгүй).
            let explicit_u = rule.to.first().map(|&c| to_lower(c)) == Some('ү');
            if explicit_u && idx > 0 && token_is(read, &tokens[idx - 1], "g") {
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

/// Transliterate a raw (possibly mixed Latin/Cyrillic) string as one word.
/// Returns the text and the gender the word leaves for a following particle.
fn read_to_comp(read: &[char], ctx: Context, tokens: &mut Vec<Token>) -> (Vec<char>, u8) {
    let rs = rules();
    // Pass 0: split the keys into longest-match tokens.
    tokens.clear();
    let mut i = 0;
    while i < read.len() {
        let mut len = 1;
        let mut cands: &'static [Rule] = &[];
        for &l in &rs.lengths {
            if l <= read.len() - i {
                if let Some(c) = rs.get(&read[i..i + l]) {
                    len = l;
                    cands = c;
                    break;
                }
            }
        }
        tokens.push(Token {
            start: i,
            len,
            cands,
        });
        i += len;
    }
    // Pass 1: the word's gender from the first (and last) deciding vowel.
    let mut initial = X_M;
    let mut last_ev = None;
    if ctx.smart {
        let mut first_ev = None;
        for idx in 0..tokens.len() {
            if let Some(g) = evidence(read, tokens, idx) {
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
    for tok in tokens.iter() {
        let chosen = tok.cands.iter().find(|r| word & r.flags == word);
        match chosen {
            Some(rule) => {
                if rule.flags & X_MF != 0 {
                    word = X_F;
                } else if rule.flags & X_MM != 0 {
                    word = X_M;
                }
                if ctx.smart && token_is(read, tok, "ii") && ii_is_y(&out, word) {
                    let upper = rule.to[0] != to_lower(rule.to[0]);
                    out.push(if upper { 'Ы' } else { 'ы' });
                } else {
                    out.extend_from_slice(&rule.to);
                }
            }
            None => out.extend_from_slice(&read[tok.start..tok.start + tok.len]),
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
    /// Backspace just undid an expansion: this word will not expand again.
    no_expand: bool,
    /// Set right after an expansion committed by Space.
    undo: Option<Undo>,
    /// Scratch buffer reused across keystrokes.
    tokens: Vec<Token>,
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
            no_expand: false,
            undo: None,
            tokens: Vec::new(),
        }
    }

    /// Change the options; takes effect from the next recomposition.
    pub fn set_options(&mut self, opts: Options) {
        self.opts = opts;
    }

    /// Forget the previous word (the particle `uu` then stays masculine) and
    /// any pending undo. Shells must call this on any caret move or focus
    /// change the composer did not see (a click, a window switch, an arrow key
    /// sent while nothing was composed and handled elsewhere).
    pub fn clear_context(&mut self) {
        self.prev_gender = None;
        self.undo = None;
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
        self.undo = None;
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
        self.undo = None;
    }

    /// Drop the composition and any pending undo (but keep the previous-word
    /// context, as before).
    pub fn reset(&mut self) {
        self.clear_buffer();
        self.undo = None;
    }

    /// Empty the buffer; the previous-word context and the undo stay.
    fn clear_buffer(&mut self) {
        self.reseeded = false;
        self.no_expand = false;
        self.read.clear();
        self.comp.clear();
        self.cursor = 0;
    }

    fn recompose(&mut self) {
        let ctx = Context {
            smart: self.opts.smart_vowels,
            prev_gender: self.prev_gender,
        };
        let (comp, gender) = read_to_comp(&self.read, ctx, &mut self.tokens);
        self.comp = comp;
        self.word_gender = gender;
    }

    fn comp_to_read(&mut self) {
        self.reseeded = true;
        self.read = self.comp.clone();
    }

    /// (back cursor distance, cursor index within `read`)
    fn read_cursor(&self) -> (usize, usize) {
        let back = self.comp.len().saturating_sub(self.cursor);
        (back, self.read.len().saturating_sub(back))
    }

    /// Commit the word at Space or punctuation, expanding an abbreviation if
    /// one matches. Returns the text and the gender it leaves for the next
    /// word. After a Space expansion the undo is armed (it needs the
    /// previous-word gender, so this runs before the caller updates it).
    fn finish_word(&mut self, space: bool) -> (Option<String>, Option<u8>) {
        let expansion = match &self.opts.abbreviations {
            Some(abbr) if !self.reseeded && !self.no_expand && !self.comp.is_empty() => {
                abbr.lookup(&self.read)
            }
            _ => None,
        };
        match expansion {
            Some(text) => {
                let gender = text_gender(&text);
                if space {
                    self.undo = Some(Undo {
                        raw: self.read.clone(),
                        delete: text.chars().count() + 1,
                        prev_gender: self.prev_gender,
                    });
                }
                self.clear_buffer();
                (Some(text), gender)
            }
            None => {
                let gender = Some(self.word_gender);
                (self.take_composition(), gender)
            }
        }
    }

    /// Process one key. Ports `ImeProcessKey` + `ImeToAsciiEx`.
    pub fn process_key(&mut self, key: Key) -> KeyOutcome {
        let mut out = KeyOutcome::default();
        // Only a key right after the expansion can undo it.
        // Shift/CapsLock alone is not such a key.
        let undo = self.undo.take();
        if key == Key::Modifier {
            self.undo = undo.clone();
        }
        if let (Key::Backspace, true, Some(u)) = (key, self.comp.is_empty(), undo) {
            self.read = u.raw;
            self.no_expand = true;
            self.prev_gender = u.prev_gender;
            self.recompose();
            self.cursor = self.comp.len();
            out.consumed = true;
            out.delete_before = u.delete;
            return out;
        }
        if !self.wants_key(key) {
            // Not ours: finish any composition, let the key through.
            let (commit, gender) = self.finish_word(key == Key::Space);
            out.commit = commit;
            // A space right after a word hands that word's gender to the next
            // one; repeated spaces keep it; any other key forgets it.
            match key {
                Key::Space if out.commit.is_some() => self.prev_gender = gender,
                // Shift/CapsLock alone must not break the context.
                Key::Space | Key::Modifier => {}
                _ => self.prev_gender = None,
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
        } else {
            Key::Other
        };
        let r = c.process_key(key);
        if let Some(s) = r.commit {
            out.push_str(&s);
        }
        if !r.consumed {
            out.push(ch);
        }
    }
    // The end of the text is not a typed Space or punctuation, so a trailing
    // abbreviation stays as typed (бн until a Space follows), like `commit`.
    if let Some(s) = c.commit() {
        out.push_str(&s);
    }
    out
}
