# 0007: Table schemes for other Cyrillic languages

Status: Accepted
Date: 2026-11-19

## Context

T asked for Russian and then other Cyrillic languages. Mongolian needs a
composer with vowel harmony. Russian and its relatives do not have that.
People type them with simple Latin habits (`zh` gives `ж`).

## Decision

- Add a `Scheme` type to `core/`. A scheme is a table of Latin patterns,
  matched longest first, with four small conditions (after a vowel, at the
  start of a word, after a letter, any).
- Mongolian keeps `Composer`. Schemes do not use the composer, the portable
  config or the word list.
- The core exposes `Scheme::convert`, which converts a whole text. A shell
  that wants live typing can call it for the current word.
- A scheme has an id (`ru`, `uk`, ...). The page stores the id by itself.
  It is not in the portable config yet.
- Only Russian is checked by a native speaker. Other schemes are drafts, and
  the page says so.

## Consequences

- A new language costs one table and its tests.
- A scheme cannot choose by word. Russian `e` is always е, and `ts` is always
  ц. A word list can fix this later, as it does for Mongolian.
- The desktop shells call `Scheme::convert` through the C ABI when they add
  other languages. That is a separate change.
