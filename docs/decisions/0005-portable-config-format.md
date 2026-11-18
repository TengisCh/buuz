# 0005: Portable config format, a small TOML subset

Status: Accepted
Date: 2026-11-17

## Context

Issue #8 asks for one settings file that moves between web, Windows and
macOS. A user sets up Buuz once and takes the same settings to each shell.
The file must be easy to read and edit by hand. It must be easy to share as
text. It must not break when Buuz adds new settings.

The core has no dependencies and does no I/O (see
[0003](0003-native-speed-over-portability.md)). Each shell reads the text
and gives it to the core. Parsing runs once at load. The typing path does not
change.

## Options considered

| Option | Easy to edit | New dependency | Size cost | Notes |
| --- | --- | --- | --- | --- |
| **TOML subset, parsed by hand** | Yes: comments, sections | None | A small parser in Rust | A file in the subset is valid TOML |
| Full TOML with a crate | Yes | `toml` and its tree | Large in WASM | Supports features we do not need |
| JSON with `serde` | No: no comments, strict commas | `serde`, `serde_json` | Large in WASM | Easy for programs, hard for people |
| Custom line format | Yes | None | Small | Other tools cannot read it, and we must document and test every rule |

The core needs only flags, a few strings and two tables of pairs. A hand
parser for a small subset covers this. The `add-dependency` checklist says to
prefer no dependency, and a keyboard app has a high supply-chain risk. A custom
format has no advantage over a subset of a known one.

## Decision

Use a **small subset of TOML**, parsed by hand in `core/src/config.rs`.

The subset has:

- `key = value` lines and `[section]` headers.
- `#` comments, on their own line or after a value.
- Double-quoted strings with the escapes `\"`, `\\` and `\n`.
- Bare keys (`A-Z a-z 0-9 _ -`) and quoted keys (`"o'"`).
- `true`, `false` and whole numbers (no leading zeros, at most `i64::MAX`).

The parser follows TOML here, so a file it accepts is valid TOML:

- Only space and tab are whitespace. Other Unicode whitespace is an error.
- Control characters (except tab) are an error in strings and comments.
- A byte order mark at the start of the text is ignored.

The subset has no arrays, inline tables, dates, multi-line strings or single
quotes. A single-quoted string gives a clear error ("use double quotes").
Other input is a parse error with a line number.

The rules:

- **Version.** `buuz_config = 1` must be the first line with content. A
  higher number gives `ConfigError::TooNew { found, supported }`. The check
  runs before the rest of the file is read.
- **Errors.** Bad syntax, a wrong value type, a bad abbreviation or key
  entry, a duplicate key or section, an unknown top-level key, a missing
  version and a too-new version are errors.
- **Warnings.** An unknown key in `[options]` and an unknown `[section]` are
  not errors. `Config::parse` returns them as `ConfigWarning` values with a
  line number and goes on. It skips the lines of an unknown section and the
  value of an unknown option. An old Buuz can then read a file from a newer
  Buuz. The shell can show the warnings, so a typo does not pass without
  notice.
- **Layers.** `preset` is `"smart"` (default) or `"classic"`. `[options]`
  holds only overrides. A missing option keeps the preset value.
- **Abbreviations.** `[abbreviations]` adds entries to the built-in list. The
  core lowercases the key and trims the value. An entry with an empty value
  removes a built-in entry. The core checks entries with the same rules as
  `Abbreviations::from_pairs`.
- **Keys.** `[keys]` maps 1 to 3 input keys to Cyrillic text. Keys are not
  case sensitive: the core stores them in lowercase, and `O` and `o` are a
  duplicate. The value is a target: Cyrillic text that a built-in key gives
  (`ү`), a pair (`у/ү`), or an empty value that removes the key. The new key
  copies the rules of the built-in key that gives the target. A target that no
  built-in key gives is `ConfigError::UnknownTarget`. `Config::options()`
  applies the mappings (issue #3, `docs/typing-rules.md` section 10).
- **Limits.** The text is at most 1 MiB. A section has at most 10,000
  entries. An abbreviation value or a `[keys]` value has at most 64
  characters. Above a limit, the parser gives `ConfigError::TooLarge`.
- **Learned words** are never in the file. Issue #12 can add an opt-in export.

API in `buuz_core`:

| Item | Use |
| --- | --- |
| `Config::parse(&str)` | Read a text. Gives `Result<Parsed, ConfigError>` |
| `Parsed` | `config: Config` and `warnings: Vec<ConfigWarning>` |
| `Config::to_text()` | Write the canonical text |
| `Config::options()` | Build the `Options` for the composer. It loads no data: `words` stays `None` |
| `Config::needs_word_list()` | True when a feature needs the word list. The shell then loads it and sets `Options::words` |
| `Config::default()` | Smart preset, no overrides |
| Getters, setters | `preset()`, `keys()` and so on; `set_abbreviation(key, value)`, `set_key(key, value)` and the other setters |

The fields of `Config` are private. The setters apply the same checks and
normal form as `parse`. So `parse(to_text(c)).config` equals `c` for every
`Config`. Writing the text of a parsed file again gives the same text. The
canonical text has no comments. It keeps the order of abbreviations and keys.
`Config`, `Preset`, `ConfigError` and `ConfigWarning` are `#[non_exhaustive]`.

### Full example

```toml
# Buuz settings. Save as my-typing.buuz.

# Required, first line with content. Newer files are refused.
buuz_config = 1

# Base settings: "smart" (default) or "classic".
# "classic" is the original Buuz and has every smart feature off.
preset = "smart"

# Only overrides. Leave out a key to keep the preset value.
[options]
smart_vowels = true     # look-ahead vowel harmony
abbreviations = true    # false turns off all abbreviations, also your own
keep_latin = true       # Esc shows the raw keys
auto_latin = true       # true: links and emails turn Latin by themselves
completion = true       # Tab accepts the rest of a likely word (needs the word list)
autocorrect = false     # fixes a misspelled word at Space (needs the word list)

# Added to the built-in list. Keys are not case sensitive.
[abbreviations]
mgl = "Монгол"          # new entry
bn = "байна аа"         # replaces the built-in entry
tgd = ""                # empty value removes the built-in entry

# Custom key mappings. The value is a Cyrillic letter that a built-in key gives.
[keys]
x = "ш"
kh = "ү"
```

## Consequences

- The core keeps zero dependencies. The parser is tested in
  `core/tests/config.rs`. A test parses the example above from
  `core/tests/data/example.buuz`. A generated-input test checks that the
  parser never panics and that the round trip holds.
- Other tools can read a config file as TOML. A text editor can edit it.
- The subset is small on purpose. A new value type needs a new ADR.
- Bump `buuz_config` only when the meaning of existing content changes: a
  rename, a changed default or a new value type. A new key or a new section
  needs no bump. An older Buuz warns and ignores it.
- Shells carry the text as copy and paste, or as a `.buuz` file. URL
  fragments and `buuz://` links wait until the desktop apps exist.
- The web shell (the second pull request of issue #8) adds import, export and
  a `localStorage` copy. Windows and macOS use the same core functions.
