---
name: tdd
description: Red-green-refactor loop for Buuz. Use for any behaviour change in core/, web/wasm/ or the web converter's logic - new rule, new option, new API - not only typing bugs. Writes the failing test first, then the smallest fix, then cleans up.
---

# Test-driven development in Buuz

One behaviour per loop. Repeat until the plan's test list is done.

## Red

Write one test that describes the behaviour in the glossary's words.

| What changes | Where the test goes |
| --- | --- |
| Conversion rule | `core/tests/rules.rs` (`eq(input, want)`) |
| Vowel harmony | `core/tests/harmony.rs` |
| Abbreviations | `core/tests/abbrev.rs` |
| Options, modes, composer API | `core/tests/<feature>.rs`, a new file is fine |
| WASM signature | `#[cfg(test)]` in `web/wasm/src/lib.rs` |
| What the site shows | `web/site/test/*.test.mjs` |

Run `cd core && cargo test --locked <name>` and read the failure. It must
fail **for the reason you expect** (wrong output, missing function), not
because of a typo. A test that passes straight away is not a red test:
fix the test or the assumption.

Add guard cases next to it: similar words, upper case, start and end of
word, and the same input in `Options::classic()` if the change is smart-only.

## Green

Make the smallest change that passes. Hard-code if that is honestly the
smallest step; the next test will force the general rule. Keep the
per-key path allocation-free.

## Refactor

With everything green, remove duplication and improve names (see
`docs/glossary.md`). Run the full `cargo test --locked` and clippy. Never
refactor and change behaviour in the same step.

## Done

All tests in the plan exist and pass, `verify` is green, and every test
would fail if its fix were reverted.
