---
name: fix-typing-bug
description: Fix a wrong Latin-to-Cyrillic conversion in Buuz test-first. Use when a user or issue reports that typing some Latin keys gives the wrong Mongolian output, or when changing a rule in core/src/rules.rs or core/src/composer.rs.
---

# Fix a typing bug, test first

1. **Pin down the case.** Exact Latin keys, expected Cyrillic, actual
   Cyrillic. Reproduce it with a quick test or the web converter. If the
   expected output is unclear, ask; do not guess Mongolian spelling.
2. **Check it is a bug, not the original behaviour.** Read the matching
   section of `docs/typing-rules.md`. Buuz must match the original Buuz unless
   the issue explicitly changes a rule (issues labelled `typing-rules` or
   `smart-typing`).
3. **Write the failing test** in `core/tests/rules.rs` with the `eq(input,
   want)` helper, inside the test function for that rule area or a new
   `#[test] fn` named after the behaviour. Add neighbouring cases that must
   not change (similar words, upper case, word start and end).
4. Run `cd core && cargo test --locked` and confirm the new test fails for
   the reason you expect.
5. **Fix** in `core/src/rules.rs` (table) or `core/src/composer.rs` (engine).
   Keep the per-key path allocation-free; no logging.
6. Run the `verify` skill (core and web groups: the site embeds the core).
7. If the rule itself changed, update `docs/typing-rules.md` and, when it is
   user-visible, add a case to `web/site/test/convert.test.mjs`.
8. Commit as `core: <behaviour>`, for example
   `core: ий after a back vowel becomes ы`, and link the issue.
