# buuz-core

Offline Rust core for Buuz 2.0: phonetic Latin → Mongolian Cyrillic
transliteration (`buuz id'ye` → `бууз идье`), behaviour-compatible with the
original [Buuz IME](https://github.com/odbayar/buuz) (Apache 2.0).

- `Composer` is a composition buffer (preedit text, cursor, commit/cancel)
  meant to be driven by thin platform front ends (Windows TSF, macOS
  InputMethodKit): feed `Key`s via `process_key`, render `preedit()`/`cursor()`,
  insert returned `commit` text.
- `convert(&str)` transliterates a whole string.
- No dependencies, no I/O, no network, no keystroke collection.

Ported faithfully from the original `Composer.cpp`: rule table, longest-match,
vowel-harmony word state (o/u → ө/ү after e, q, w, ye, `o'`, `u'`; reset by
a, ya, yo, yu, `"o`, `"u`), per-letter case expansion (including quirks such as
`cH` → `ч`), 50-char composition limit and cursor editing (Cyrillic re-seeding
after Left/Right/Home/End).

`cargo test` runs the test suite. Expected values were derived by hand from the
original rules and `docs/table.html`; a differential test against a compiled
build of the original algorithm is a recommended follow-up.

Licensed under Apache-2.0 only; see `LICENSE` and `NOTICE` (original copyright
Odbayar Nyamtseren). Unlike the rest of the repository, which is
MIT OR Apache-2.0, this crate is ported from the original Buuz and keeps its
license (see `docs/decisions/0004-license-mit-or-apache-for-new-code.md`).
