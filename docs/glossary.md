# Glossary

The words Buuz uses for its own ideas. Code, tests, issues, docs and the
settings UI use these names, so a term means one thing everywhere (the
"ubiquitous language" idea from domain-driven design). Add a term here
when a new concept appears; rename here first when a name changes.

## Typing

| Term | Meaning | In code |
| --- | --- | --- |
| Key | One key event the platform hands to the core, already resolved to a character or an action (Backspace, Space, Other) | `Key` |
| Composition | The word being typed: Latin keys not yet committed | `Composer` |
| Raw input | The Latin keys of the composition, as typed | `Composer::raw()` |
| Preedit | The Cyrillic shown (underlined) for the composition before it is committed | `Composer::preedit()` |
| Commit | Turning the composition into final text in the document, at Space, punctuation or any other key | `Composer::commit()`, `KeyOutcome::commit` |
| Rule | One row of the table: Latin pattern to Cyrillic, with gender variants for vowels | `Rule`, `core/src/rules.rs` |
| Longest match | The table is matched longest rule first (`sh` before `s`) | `docs/typing-rules.md` §2 |
| Per-key path | Code that runs on every keystroke: under 10 µs, no allocation, no logging, no network | `Composer::process_key` |
| Per-word path | Code that runs once at commit: under 1 ms (abbreviations, later autocorrect) | |

## Mongolian

| Term | Meaning |
| --- | --- |
| Gender | Vowel harmony class of a word: masculine (а, о, у) or feminine (э, ө, ү). Decides which Cyrillic vowel a Latin `o` or `u` becomes |
| Deciding vowel | The first vowel in a word that fixes its gender |
| Particle | The question word уу/үү, which takes the gender of the previous word |
| Long vowel | A doubled vowel (оо, уу, ээ, өө, үү) |

## Modes and features

| Term | Meaning | In code |
| --- | --- | --- |
| Classic mode | The original 2009 Buuz behaviour, left to right, every smart feature off. Its output never changes | `Options::classic()` |
| Smart mode | The default: classic rules plus the smart features, all on by default, each with its own switch | `Options::smart()` |
| Smart feature | One opt-out improvement inside smart mode: smart vowels, abbreviations, keep Latin, later autocorrect and completion | fields of `Options` |
| Smart vowels | Look-ahead vowel harmony, ий/ы and the particle rule | `Options::smart_vowels` |
| Abbreviation | A short Latin key sequence expanded at commit (`bn` to байна). Backspace right after undoes it | `Abbreviations` |
| Undo | Backspace right after an automatic change restores what was typed | `KeyOutcome::delete_before` |
| Keep Latin | Esc while composing shows the raw Latin keys instead of Cyrillic; Esc again switches back | `Options::keep_latin`, `Key::Escape` |
| Reopen | Backspace right after a word and a space puts the word back in the preedit with its raw keys | `Options::keep_latin` |
| Auto Latin | A link or an email turns Latin by itself (`@`, `://`, a `/` after a domain, or a domain at the end of a run). Separate switch from Keep Latin | `Options::auto_latin` |
| Run | Everything typed since the last Space, Enter or Tab: words and the symbols between them. A caret move stops tracking until the next one. Auto Latin looks at the run | `docs/typing-rules.md` §9 |
| Symbol key | A printable key that is not a letter and not a space (`.` `@` `/` digits). Acts like `Key::Other` when Auto Latin is off | `Key::Symbol` |
| Portable config | The one settings file shared by web, Windows and macOS (issue #8) | |

## Project

| Term | Meaning |
| --- | --- |
| Core | The Rust engine in `core/`, shared by every platform |
| Shell | A platform front end around the core: web converter, Windows (TSF), macOS (InputMethodKit) |
| Original Buuz | The 2009 input method by Odbayar Nyamtseren that the core is ported from |
