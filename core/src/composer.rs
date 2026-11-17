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

use crate::rules::{rules, X_F, X_M, X_MF, X_MM};

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
}

/// True for characters that start/extend a composition: ASCII letters,
/// `'`, `"` and the umlauts `ö ü Ö Ü` (typed for ө and ү).
pub fn is_input_char(ch: char) -> bool {
    ch == '\'' || ch == '"' || ch.is_ascii_alphabetic() || matches!(ch, 'ö' | 'ü' | 'Ö' | 'Ü')
}

/// Transliterate a raw (possibly mixed Latin/Cyrillic) string as one word.
fn read_to_comp(read: &[char]) -> Vec<char> {
    let rs = rules();
    let mut word = X_M;
    let mut out = Vec::with_capacity(read.len());
    let mut i = 0;
    'outer: while i < read.len() {
        for &len in &rs.lengths {
            if len > read.len() - i {
                continue;
            }
            if let Some(cands) = rs.get(&read[i..i + len]) {
                for rule in cands {
                    if word & rule.flags == word {
                        if rule.flags & X_MF != 0 {
                            word = X_F;
                        } else if rule.flags & X_MM != 0 {
                            word = X_M;
                        }
                        out.extend_from_slice(&rule.to);
                        i += len;
                        continue 'outer;
                    }
                }
            }
        }
        out.push(read[i]);
        i += 1;
    }
    out
}

/// Stateful composition buffer.
#[derive(Debug, Clone, Default)]
pub struct Composer {
    /// Raw input; becomes the Cyrillic text after cursor movement.
    read: Vec<char>,
    comp: Vec<char>,
    /// Cursor position within `comp`, in characters.
    cursor: usize,
}

impl Composer {
    pub fn new() -> Self {
        Self::default()
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

    /// Commit the composition (e.g. on focus loss or Space/Enter).
    pub fn commit(&mut self) -> Option<String> {
        if self.comp.is_empty() {
            return None;
        }
        let s = self.preedit();
        self.reset();
        Some(s)
    }

    /// Discard the composition without inserting anything.
    pub fn cancel(&mut self) {
        self.reset();
    }

    pub fn reset(&mut self) {
        self.read.clear();
        self.comp.clear();
        self.cursor = 0;
    }

    fn recompose(&mut self) {
        self.comp = read_to_comp(&self.read);
    }

    fn comp_to_read(&mut self) {
        self.read = self.comp.clone();
    }

    /// (back cursor distance, cursor index within `read`)
    fn read_cursor(&self) -> (usize, usize) {
        let back = self.comp.len().saturating_sub(self.cursor);
        (back, self.read.len().saturating_sub(back))
    }

    /// Process one key. Ports `ImeProcessKey` + `ImeToAsciiEx`.
    pub fn process_key(&mut self, key: Key) -> KeyOutcome {
        let mut out = KeyOutcome::default();
        if !self.wants_key(key) {
            // Not ours: finish any composition, let the key through.
            out.commit = self.commit();
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
            self.reset(); // emptied by editing: cancel, as the original does
        }
        out
    }
}

/// Convenience: transliterate a whole string, the way typing it would.
/// Input characters compose; any other character ends the current word and
/// is copied through (`"sain baina uu"` -> `"сайн байна уу"`).
pub fn convert(input: &str) -> String {
    let mut c = Composer::new();
    let mut out = String::new();
    for ch in input.chars() {
        let key = if is_input_char(ch) {
            Key::Char(ch)
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
    if let Some(s) = c.commit() {
        out.push_str(&s);
    }
    out
}
