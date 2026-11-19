// SPDX-License-Identifier: MIT OR Apache-2.0

//! The C ABI, called the way a C++ or Swift shell calls it (issue #72).

use std::ffi::CStr;
use std::ptr;

use buuz_ffi::*;

/// The text of a (pointer, length) pair that the library returned.
unsafe fn text(p: *const std::os::raw::c_char, len: usize) -> String {
    assert!(!p.is_null());
    let p = p.cast::<u8>();
    let s = std::str::from_utf8(std::slice::from_raw_parts(p, len)).unwrap();
    // The library also ends each string with a NUL byte.
    assert_eq!(*p.add(len), 0);
    s.to_string()
}

unsafe fn key(c: *mut BuuzComposer, kind: u32, ch: char) -> BuuzOutcome {
    let mut out = BuuzOutcome::default();
    assert!(buuz_process_key(c, kind, ch as u32, &mut out));
    out
}

unsafe fn preedit(c: *mut BuuzComposer) -> String {
    let mut len = 0;
    let p = buuz_preedit(c, &mut len);
    text(p, len)
}

unsafe fn typed(c: *mut BuuzComposer, s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        let kind = match ch {
            ' ' => BUUZ_KEY_SPACE,
            '.' | ',' => BUUZ_KEY_SYMBOL,
            _ => BUUZ_KEY_CHAR,
        };
        let o = key(c, kind, ch);
        if !o.commit.is_null() {
            out.push_str(&text(o.commit, o.commit_len));
        }
        if !o.consumed {
            out.push(ch);
        }
    }
    out
}

#[test]
fn smart_defaults_without_a_config() {
    unsafe {
        let c = buuz_composer_new(ptr::null(), 0, ptr::null());
        assert!(!c.is_null());
        assert_eq!(typed(c, "sain bain"), "сайн ");
        assert!(buuz_is_composing(c));
        assert_eq!(preedit(c), "байн");
        assert_eq!(buuz_cursor(c), 4);
        let mut len = 0;
        let p = buuz_commit(c, &mut len);
        assert_eq!(text(p, len), "байн");
        assert!(!buuz_is_composing(c));
        // Nothing left to commit.
        assert!(buuz_commit(c, &mut len).is_null());
        assert_eq!(len, 0);
        buuz_composer_free(c);
    }
}

#[test]
fn a_config_text_sets_the_options() {
    let cfg = "buuz_config = 1\npreset = \"classic\"\n";
    unsafe {
        let c = buuz_composer_new(cfg.as_ptr(), cfg.len(), ptr::null());
        assert!(!c.is_null());
        // Classic mode: no abbreviations.
        assert_eq!(typed(c, "bn "), "бн ");
        buuz_composer_free(c);
    }
}

#[test]
fn a_bad_config_gives_null() {
    let cfg = "this is not a config";
    unsafe {
        assert!(buuz_composer_new(cfg.as_ptr(), cfg.len(), ptr::null()).is_null());
        // Not UTF-8.
        let bad = [0xffu8, 0xfe];
        assert!(buuz_composer_new(bad.as_ptr(), bad.len(), ptr::null()).is_null());
    }
}

#[test]
fn the_word_list_reaches_the_composer() {
    let list = "дэлгүүр\t100\n";
    let cfg = "buuz_config = 1\n[options]\nautocorrect = true\n";
    unsafe {
        let w = buuz_word_list_parse(list.as_ptr(), list.len());
        assert!(!w.is_null());
        let c = buuz_composer_new(cfg.as_ptr(), cfg.len(), w);
        // The composer keeps its own reference: the shell may free the list.
        buuz_word_list_free(w);
        let mut out = BuuzOutcome::default();
        for ch in "delgur".chars() {
            assert!(buuz_process_key(c, BUUZ_KEY_CHAR, ch as u32, &mut out));
        }
        assert!(buuz_process_key(c, BUUZ_KEY_SPACE, ' ' as u32, &mut out));
        assert_eq!(text(out.commit, out.commit_len), "дэлгүүр");
        assert!(out.corrected);
        // Backspace right after reopens the word as typed.
        assert!(buuz_process_key(c, BUUZ_KEY_BACKSPACE, 0, &mut out));
        assert_eq!(out.delete_before, 8);
        assert_eq!(preedit(c), "дэлгүр");
        buuz_composer_free(c);
    }
}

#[test]
fn a_bad_word_list_gives_null() {
    let bad = [0xffu8];
    unsafe {
        assert!(buuz_word_list_parse(bad.as_ptr(), bad.len()).is_null());
        assert!(buuz_word_list_parse(ptr::null(), 0).is_null());
    }
}

#[test]
fn bad_arguments_are_refused_not_crashed() {
    unsafe {
        let mut out = BuuzOutcome::default();
        assert!(!buuz_process_key(
            ptr::null_mut(),
            BUUZ_KEY_CHAR,
            'a' as u32,
            &mut out
        ));
        let c = buuz_composer_new(ptr::null(), 0, ptr::null());
        // An unknown key kind, and a code point that is not a char.
        assert!(!buuz_process_key(c, 999, 0, &mut out));
        assert!(!buuz_process_key(c, BUUZ_KEY_CHAR, 0xD800, &mut out));
        assert!(!buuz_process_key(
            c,
            BUUZ_KEY_CHAR,
            'a' as u32,
            ptr::null_mut()
        ));
        let mut len = 7;
        assert!(buuz_preedit(ptr::null_mut(), &mut len).is_null());
        assert_eq!(len, 0);
        assert!(!buuz_is_composing(ptr::null()));
        buuz_cancel(ptr::null_mut());
        buuz_composer_free(ptr::null_mut());
        buuz_word_list_free(ptr::null_mut());
        buuz_composer_free(c);
    }
}

#[test]
fn cancel_drops_the_composition() {
    unsafe {
        let c = buuz_composer_new(ptr::null(), 0, ptr::null());
        typed(c, "sain");
        buuz_cancel(c);
        assert!(!buuz_is_composing(c));
        assert_eq!(preedit(c), "");
        buuz_composer_free(c);
    }
}

#[test]
fn version_is_the_crate_version() {
    let v = unsafe { CStr::from_ptr(buuz_version()) };
    assert_eq!(v.to_str().unwrap(), env!("CARGO_PKG_VERSION"));
}
