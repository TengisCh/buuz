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
| Word list | The most frequent Mongolian words (up to 50,000) with their counts, by rank, built from Mongolian Wikipedia (CC BY-SA 4.0). Loaded on demand, never on the per-key path. In code: `WordList`, `data/wordlist/mn.txt` |

## Modes and features

| Term | Meaning | In code |
| --- | --- | --- |
| Classic mode | The original 2009 Buuz behaviour, left to right, every smart feature off. Its output never changes | `Options::classic()` |
| Smart mode | The default: classic rules plus the smart features, all on by default, each with its own switch | `Options::smart()` |
| Smart feature | One improvement inside smart mode with its own switch. Smart vowels, abbreviations, keep Latin and completion are on by default (opt-out). Autocorrect is off by default (opt-in) | fields of `Options` |
| Smart vowels | Look-ahead vowel harmony, ий/ы and the particle rule | `Options::smart_vowels` |
| Abbreviation | A short Latin key sequence expanded at commit (`bn` to байна). Backspace right after undoes it | `Abbreviations` |
| Undo | Backspace right after an automatic change restores what was typed | `KeyOutcome::delete_before` |
| Keep Latin | Esc while composing shows the raw Latin keys instead of Cyrillic; Esc again switches back | `Options::keep_latin`, `Key::Escape` |
| Reopen | Backspace right after a word and a space puts the word back in the preedit with its raw keys | `Options::keep_latin` |
| Auto Latin | A link or an email turns Latin by itself (`@`, `://`, a `/` after a domain, or a domain at the end of a run). Separate switch from Keep Latin | `Options::auto_latin` |
| Run | Everything typed since the last Space, Enter or Tab: words and the symbols between them. A caret move stops tracking until the next one. Auto Latin looks at the run | `docs/typing-rules.md` §9 |
| Symbol key | A printable key that is not a letter and not a space (`.` `@` `/` digits). Acts like `Key::Other` when Auto Latin is off | `Key::Symbol` |
| Config | The portable settings file shared by web, Windows and macOS. A small TOML subset: a preset, option overrides, user abbreviations and custom keys. Versioned with `buuz_config = 1`. Unknown options and sections give warnings, not errors. See ADR 0005 | `Config` |
| Preset | The base of a config: `smart` (default) or `classic`. Options in the config only override it | `Preset` |
| Key map | The built-in rule table with the user's `[keys]` changes applied. Built once and shared. Without one, the composer uses the built-in table. See `docs/typing-rules.md` §10 | `KeyMap`, `Options::keys` |
| Completion | Ghost text after the preedit: the rest of a clear winner from the word list. Tab, or Right at the end, accepts it. See `docs/typing-rules.md` §12 | `Composer::completion`, `Options::completion`, `Key::Tab` |
| Autocorrect | At Space or punctuation, a word that is not in the word list becomes its one clear neighbour that is (`хуухэд` to `хүүхэд`). Off by default. Backspace right after undoes it. See `docs/typing-rules.md` §13 | `Options::autocorrect`, `KeyOutcome::corrected` |
| Rejected set | The words whose correction the user undid in this session, at most 256. Autocorrect skips them. Not saved | `Composer` |
| Target | The Cyrillic value of a `[keys]` entry (`ү`, or a pair such as `у/ү`). The new key copies the rules of the built-in key that gives it. Empty removes the key | `KeyMap::from_pairs`, `KeyError::UnknownTarget` |
| Calibration | A short test where the user types Cyrillic sentences in Latin. It suggests `[keys]` entries for the keys the built-in table does not give. Offline, not on the typing path. See `docs/typing-rules.md` §11 | `calibrate`, `Calibration` |
| Suggestion | One key proposed by calibration: a chunk typed for a letter (or a gender pair), how often, and the letter it displaces | `Suggestion` |
| Displaces | The letter a built-in key gives that a suggestion would take away (`v` for ү displaces в) | `Suggestion::displaces` |

## Project

| Term | Meaning |
| --- | --- |
| Core | The Rust engine in `core/`, shared by every platform |
| Shell | A platform front end around the core: web converter, Windows (TSF), macOS (InputMethodKit) |
| Original Buuz | The 2009 input method by Odbayar Nyamtseren that the core is ported from |
