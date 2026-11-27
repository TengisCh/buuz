// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stable C ABI around `buuz-core` for the Windows (C++ TSF) and macOS
//! (Swift IMK) input methods. See `include/buuz.h` for the C view and
//! `docs/architecture.md` section 6.
//!
//! Rules for every function:
//!
//! - Strings in and out are UTF-8 with a byte length. A returned string also
//!   ends with a NUL byte, so C code can use it as a C string.
//! - A returned string belongs to the handle. It stays valid until the next
//!   call on that handle, or until the handle is freed.
//! - A null handle or a bad argument gives an error value (`false`, null or
//!   0). No function crashes on it.
//! - A Rust panic never crosses the C boundary: it gives the error value.
//! - The functions on one handle are not thread-safe. Use one handle from
//!   one thread at a time.

use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::slice;
use std::sync::Arc;

use buuz_core::{Composer, Config, Key, WordList};

/// A typed letter or input key. `ch` holds the character.
pub const BUUZ_KEY_CHAR: u32 = 0;
/// Backspace.
pub const BUUZ_KEY_BACKSPACE: u32 = 1;
/// Delete (forward delete).
pub const BUUZ_KEY_DELETE: u32 = 2;
/// Left arrow.
pub const BUUZ_KEY_LEFT: u32 = 3;
/// Right arrow.
pub const BUUZ_KEY_RIGHT: u32 = 4;
/// Home.
pub const BUUZ_KEY_HOME: u32 = 5;
/// End.
pub const BUUZ_KEY_END: u32 = 6;
/// Shift, Caps Lock and other modifiers: they do not end the composition.
pub const BUUZ_KEY_MODIFIER: u32 = 7;
/// Space.
pub const BUUZ_KEY_SPACE: u32 = 8;
/// Esc.
pub const BUUZ_KEY_ESCAPE: u32 = 9;
/// A printable character that is not a letter and not a space. `ch` holds it.
pub const BUUZ_KEY_SYMBOL: u32 = 10;
/// Tab.
pub const BUUZ_KEY_TAB: u32 = 11;
/// Enter and every other key: it ends the composition.
pub const BUUZ_KEY_OTHER: u32 = 12;

/// A composer and the buffers of the strings it returns.
pub struct BuuzComposer {
    composer: Composer,
    /// The last commit, preedit or other string, with a NUL byte at the end.
    out: Vec<u8>,
}

/// A parsed word list. A composer keeps its own reference to it.
pub struct BuuzWordList(Arc<WordList>);

/// The result of one key. Read it in this order: delete `delete_before`
/// characters before the caret, insert `commit`, then show the preedit.
#[repr(C)]
#[derive(Debug)]
pub struct BuuzOutcome {
    /// The input method used the key. When false, pass the key on to the
    /// application after the commit.
    pub consumed: bool,
    /// The composition is full and the key was refused.
    pub rejected: bool,
    /// The commit holds a word that word choice, autocorrect, suffix choice or rounding harmony changed.
    /// Backspace right after the Space undoes the change.
    pub corrected: bool,
    /// Characters (Unicode scalars) before the caret that the application
    /// must delete first.
    pub delete_before: u32,
    /// UTF-8 text to insert now, or null. It ends with a NUL byte and stays
    /// valid until the next call on the composer.
    pub commit: *const c_char,
    /// Length of `commit` in bytes, without the NUL byte.
    pub commit_len: usize,
}

impl Default for BuuzOutcome {
    fn default() -> Self {
        BuuzOutcome {
            consumed: false,
            rejected: false,
            corrected: false,
            delete_before: 0,
            commit: ptr::null(),
            commit_len: 0,
        }
    }
}

/// Run `f`, and give `err` if it panics.
fn guard<T>(err: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(err)
}

/// The UTF-8 text at (`p`, `len`). Null with length 0 is the empty text.
///
/// # Safety
///
/// `p` must point to `len` readable bytes, or be null.
unsafe fn utf8<'a>(p: *const u8, len: usize) -> Option<&'a str> {
    if p.is_null() {
        return (len == 0).then_some("");
    }
    std::str::from_utf8(slice::from_raw_parts(p, len)).ok()
}

/// Copy `s` into the handle buffer with a NUL byte, and give a pointer to it.
fn keep(out: &mut Vec<u8>, s: &str) -> *const u8 {
    out.clear();
    out.extend_from_slice(s.as_bytes());
    out.push(0);
    out.as_ptr()
}

/// Parse a word list (`data/wordlist/mn.txt` format). Gives null when the
/// text is not UTF-8, is not a valid list, or `text` is null. Free the
/// result with `buuz_word_list_free`.
///
/// # Safety
///
/// `text` must point to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn buuz_word_list_parse(text: *const u8, len: usize) -> *mut BuuzWordList {
    guard(ptr::null_mut(), || {
        if text.is_null() {
            return ptr::null_mut();
        }
        match utf8(text, len).map(WordList::parse) {
            Some(Ok(w)) => Box::into_raw(Box::new(BuuzWordList(Arc::new(w.with_chat_words())))),
            _ => ptr::null_mut(),
        }
    })
}

/// Free a word list. Composers made with it keep working. Null does nothing.
///
/// # Safety
///
/// `words` must come from `buuz_word_list_parse` and not be freed before.
#[no_mangle]
pub unsafe extern "C" fn buuz_word_list_free(words: *mut BuuzWordList) {
    if !words.is_null() {
        guard((), || drop(Box::from_raw(words)));
    }
}

/// Make a composer from a portable config text (`docs/decisions/0005`).
/// A null `config` gives the smart defaults. `words` may be null; the
/// features that need words (completion, word choice, suffix choice, rounding harmony, autocorrect) are then
/// off. Gives null when the config is not UTF-8 or not valid. Free the result
/// with `buuz_composer_free`.
///
/// # Safety
///
/// `config` must point to `len` readable bytes, or be null. `words` must be
/// null or a live word list.
#[no_mangle]
pub unsafe extern "C" fn buuz_composer_new(
    config: *const u8,
    len: usize,
    words: *const BuuzWordList,
) -> *mut BuuzComposer {
    guard(ptr::null_mut(), || {
        let cfg = if config.is_null() {
            Config::default()
        } else {
            match utf8(config, len).map(Config::parse) {
                Some(Ok(parsed)) => parsed.config,
                _ => return ptr::null_mut(),
            }
        };
        let mut opts = cfg.options();
        if let Some(w) = words.as_ref() {
            opts.words = Some(Arc::clone(&w.0));
        }
        Box::into_raw(Box::new(BuuzComposer {
            composer: Composer::with_options(opts),
            out: Vec::new(),
        }))
    })
}

/// Free a composer. Null does nothing.
///
/// # Safety
///
/// `c` must come from `buuz_composer_new` and not be freed before.
#[no_mangle]
pub unsafe extern "C" fn buuz_composer_free(c: *mut BuuzComposer) {
    if !c.is_null() {
        guard((), || drop(Box::from_raw(c)));
    }
}

/// The core key for a key kind and a character.
fn to_key(kind: u32, ch: u32) -> Option<Key> {
    Some(match kind {
        BUUZ_KEY_CHAR => Key::Char(char::from_u32(ch)?),
        BUUZ_KEY_BACKSPACE => Key::Backspace,
        BUUZ_KEY_DELETE => Key::Delete,
        BUUZ_KEY_LEFT => Key::Left,
        BUUZ_KEY_RIGHT => Key::Right,
        BUUZ_KEY_HOME => Key::Home,
        BUUZ_KEY_END => Key::End,
        BUUZ_KEY_MODIFIER => Key::Modifier,
        BUUZ_KEY_SPACE => Key::Space,
        BUUZ_KEY_ESCAPE => Key::Escape,
        BUUZ_KEY_SYMBOL => Key::Symbol(char::from_u32(ch)?),
        BUUZ_KEY_TAB => Key::Tab,
        BUUZ_KEY_OTHER => Key::Other,
        _ => return None,
    })
}

/// Process one key and write the result to `out`. `kind` is a `BUUZ_KEY_*`
/// value. `ch` is the Unicode code point for `BUUZ_KEY_CHAR` and
/// `BUUZ_KEY_SYMBOL`, and is ignored for the other kinds. Gives false (and
/// leaves the composer as it was) for a null argument, an unknown kind or a
/// bad code point.
///
/// # Safety
///
/// `c` must be a live composer and `out` must point to a `BuuzOutcome`.
#[no_mangle]
pub unsafe extern "C" fn buuz_process_key(
    c: *mut BuuzComposer,
    kind: u32,
    ch: u32,
    out: *mut BuuzOutcome,
) -> bool {
    guard(false, || {
        let (Some(c), Some(out)) = (c.as_mut(), out.as_mut()) else {
            return false;
        };
        let Some(key) = to_key(kind, ch) else {
            return false;
        };
        let r = c.composer.process_key(key);
        *out = BuuzOutcome {
            consumed: r.consumed,
            rejected: r.rejected,
            corrected: r.corrected,
            delete_before: u32::try_from(r.delete_before).unwrap_or(u32::MAX),
            commit: ptr::null(),
            commit_len: 0,
        };
        if let Some(text) = r.commit {
            out.commit = keep(&mut c.out, &text).cast();
            out.commit_len = text.len();
        }
        true
    })
}

/// The preedit (the underlined text being composed) as UTF-8. Writes its
/// byte length to `len`. Gives null for a null composer.
///
/// # Safety
///
/// `c` must be null or a live composer. `len` must be null or writable.
#[no_mangle]
pub unsafe extern "C" fn buuz_preedit(c: *mut BuuzComposer, len: *mut usize) -> *const c_char {
    let (p, n) = guard((ptr::null(), 0), || match c.as_mut() {
        Some(c) => {
            let s = c.composer.preedit();
            (keep(&mut c.out, &s), s.len())
        }
        None => (ptr::null(), 0),
    });
    if let Some(len) = len.as_mut() {
        *len = n;
    }
    p.cast()
}

/// The caret inside the preedit, in characters. Mongolian Cyrillic is in
/// the Basic Multilingual Plane, so this is also the caret in UTF-16 units.
/// Gives 0 for a null composer.
///
/// # Safety
///
/// `c` must be null or a live composer.
#[no_mangle]
pub unsafe extern "C" fn buuz_cursor(c: *const BuuzComposer) -> u32 {
    guard(0, || {
        c.as_ref().map_or(0, |c| {
            u32::try_from(c.composer.cursor()).unwrap_or(u32::MAX)
        })
    })
}

/// True while a composition is in progress.
///
/// # Safety
///
/// `c` must be null or a live composer.
#[no_mangle]
pub unsafe extern "C" fn buuz_is_composing(c: *const BuuzComposer) -> bool {
    guard(false, || {
        c.as_ref().is_some_and(|c| c.composer.is_composing())
    })
}

/// End the composition and give its text, for example on focus loss.
/// Gives null (and 0 in `len`) when there is nothing to commit.
///
/// # Safety
///
/// `c` must be null or a live composer. `len` must be null or writable.
#[no_mangle]
pub unsafe extern "C" fn buuz_commit(c: *mut BuuzComposer, len: *mut usize) -> *const c_char {
    let (p, n) = guard((ptr::null(), 0), || {
        let Some(c) = c.as_mut() else {
            return (ptr::null(), 0);
        };
        match c.composer.commit() {
            Some(s) => (keep(&mut c.out, &s), s.len()),
            None => (ptr::null(), 0),
        }
    });
    if let Some(len) = len.as_mut() {
        *len = n;
    }
    p.cast()
}

/// Drop the composition without a commit.
///
/// # Safety
///
/// `c` must be null or a live composer.
#[no_mangle]
pub unsafe extern "C" fn buuz_cancel(c: *mut BuuzComposer) {
    guard((), || {
        if let Some(c) = c.as_mut() {
            c.composer.cancel();
        }
    });
}

/// The library version, as a static NUL-terminated string.
#[no_mangle]
pub extern "C" fn buuz_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}
