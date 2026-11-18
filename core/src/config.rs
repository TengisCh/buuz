// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0.

//! The portable config (issue #8): one small text file that carries a user's
//! settings between web, Windows and macOS. The format is a subset of TOML,
//! parsed by hand so the core keeps zero dependencies. See
//! `docs/decisions/0005-portable-config-format.md` for the full format.
//!
//! Parsing runs once, when a shell loads the file. The typing path never
//! touches this module: [`Config::options`] builds the [`Options`] value
//! that the [`Composer`](crate::Composer) uses.
//!
//! ```
//! use buuz_core::{convert_with, Config};
//!
//! let config = Config::parse("buuz_config = 1\n[abbreviations]\nmgl = \"Монгол\"\n").unwrap().config;
//! assert_eq!(convert_with("mgl bn ", &config.options()), "Монгол байна ");
//! ```

use std::collections::HashSet;
use std::fmt;
use std::fmt::Write as _;
use std::sync::Arc;

use crate::abbrev::{check_key, check_value, AbbrevError, Abbreviations};
use crate::composer::{is_input_char, Options};
use crate::keys::{check_mapping, KeyError, KeyMap};
use crate::rules::to_lower;

/// The newest config version this core can read.
pub const CONFIG_VERSION: u64 = 1;

/// The largest text `Config::parse` reads, in bytes.
pub const MAX_CONFIG_BYTES: usize = 1024 * 1024;

/// The most entries in one of `[abbreviations]` and `[keys]`.
pub const MAX_SECTION_ENTRIES: usize = 10_000;

/// The most characters in an abbreviation value or a `[keys]` value.
pub const MAX_VALUE_CHARS: usize = 64;

/// The base settings of a config. `[options]` only overrides single fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Preset {
    /// [`Options::smart`]: every smart feature on. The default.
    #[default]
    Smart,
    /// [`Options::classic`]: the original Buuz behaviour.
    Classic,
}

impl Preset {
    fn name(self) -> &'static str {
        match self {
            Preset::Smart => "smart",
            Preset::Classic => "classic",
        }
    }
}

/// A config. The default is the smart preset with no overrides.
///
/// The fields are private. Read them with the getters and change them with
/// the setters, which apply the same checks as [`Config::parse`]. So
/// `Config::parse(&c.to_text())` gives back a config equal to `c` for every
/// `c`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Config {
    preset: Preset,
    smart_vowels: Option<bool>,
    abbreviations: Option<bool>,
    keep_latin: Option<bool>,
    auto_latin: Option<bool>,
    completion: Option<bool>,
    autocorrect: Option<bool>,
    learning: Option<bool>,
    abbreviation_entries: Vec<(String, String)>,
    keys: Vec<(String, String)>,
}

/// The result of a successful parse: the config and what the parser ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub config: Config,
    pub warnings: Vec<ConfigWarning>,
}

/// Something in the text that this core does not know. It comes from a newer
/// Buuz, or from a typo. The parser ignores it and goes on.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConfigWarning {
    /// A key in `[options]` that does not exist.
    UnknownOption { line: usize, key: String },
    /// A `[section]` that does not exist. Its lines are skipped.
    UnknownSection { line: usize, name: String },
}

impl fmt::Display for ConfigWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigWarning::UnknownOption { line, key } => {
                write!(f, "line {line}: unknown option `{key}`, ignored")
            }
            ConfigWarning::UnknownSection { line, name } => {
                write!(f, "line {line}: unknown section [{name}], ignored")
            }
        }
    }
}

/// Why a config text could not be read. `line` is 1-based.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConfigError {
    /// The first line with content is not `buuz_config = <number>`.
    MissingVersion { line: usize },
    /// The file comes from a newer Buuz.
    TooNew { found: u64, supported: u64 },
    /// A line that is not a comment, a `[section]` or `key = value`.
    Syntax { line: usize },
    /// A top-level key that does not exist.
    UnknownKey { line: usize, key: String },
    /// A value of the wrong type or with a wrong content.
    BadValue { line: usize, key: String },
    /// A string escape other than `\"`, `\\` and `\n`.
    BadEscape { line: usize },
    /// A single-quoted string or key. Only double quotes are allowed.
    SingleQuotes { line: usize },
    /// The same key twice in one place.
    DuplicateKey { line: usize, key: String },
    /// The same `[section]` twice.
    DuplicateSection { line: usize, name: String },
    /// A bad entry in `[abbreviations]` (the error has its own line).
    Abbreviation(AbbrevError),
    /// A bad entry in `[keys]`.
    BadKeys { line: usize, key: String },
    /// A `[keys]` value that no built-in key gives.
    UnknownTarget {
        line: usize,
        key: String,
        value: String,
    },
    /// A `[keys]` value with upper case letters. Targets are lower case.
    UpperCaseTarget {
        line: usize,
        key: String,
        value: String,
    },
    /// The text or a section is over the size limit. `line` is 0 when the
    /// whole text is too long.
    TooLarge { line: usize, limit: usize },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingVersion { line } => write!(
                f,
                "line {line}: the first line must be `buuz_config = {CONFIG_VERSION}`"
            ),
            ConfigError::TooNew { found, supported } => write!(
                f,
                "config version {found} is newer than this Buuz reads (version {supported})"
            ),
            ConfigError::Syntax { line } => {
                write!(f, "line {line}: expected `key = value` or `[section]`")
            }
            ConfigError::UnknownKey { line, key } => write!(f, "line {line}: unknown key {key:?}"),
            ConfigError::BadValue { line, key } => {
                write!(f, "line {line}: wrong value for {key:?}")
            }
            ConfigError::BadEscape { line } => {
                write!(f, "line {line}: use only \\\", \\\\ and \\n in strings")
            }
            ConfigError::SingleQuotes { line } => {
                write!(f, "line {line}: use double quotes, not single quotes")
            }
            ConfigError::DuplicateKey { line, key } => {
                write!(f, "line {line}: key {key:?} is already set")
            }
            ConfigError::DuplicateSection { line, name } => {
                write!(f, "line {line}: section [{name}] is already there")
            }
            ConfigError::Abbreviation(e) => e.fmt(f),
            ConfigError::BadKeys { line: 0, key } => write!(
                f,
                "invalid key mapping {key:?} (use 1 to 3 input keys and Cyrillic text, a pair such as \"у/ү\", or an empty value to remove the key)"
            ),
            ConfigError::BadKeys { line, key } => write!(
                f,
                "line {line}: invalid key mapping {key:?} (use 1 to 3 input keys and Cyrillic text, a pair such as \"у/ү\", or an empty value to remove the key)"
            ),
            ConfigError::UnknownTarget { line, key, value } => {
                if *line > 0 {
                    write!(f, "line {line}: ")?;
                }
                write!(
                    f,
                    "key {key:?}: no built-in key gives {value:?} (use a letter the built-in keys give, or a pair such as \"у/ү\")"
                )
            }
            ConfigError::UpperCaseTarget { line, key, value } => {
                if *line > 0 {
                    write!(f, "line {line}: ")?;
                }
                write!(
                    f,
                    "key {key:?}: the target {value:?} must be lower case (upper case follows the key you press)"
                )
            }
            ConfigError::TooLarge { line: 0, limit } => {
                write!(f, "config is larger than {limit} bytes")
            }
            ConfigError::TooLarge { line, limit } => {
                write!(f, "line {line}: more than {limit} entries in one section")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

enum Value {
    Bool(bool),
    Int(u64),
    Str(String),
    Other,
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Top,
    Options,
    Abbreviations,
    Keys,
    /// A section this core does not know. Its lines are skipped.
    Unknown,
}

fn is_bare(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// TOML whitespace: space and tab only.
fn is_ws(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// Control characters TOML forbids in strings and comments (tab is fine).
fn is_control(c: char) -> bool {
    matches!(c, '\0'..='\u{8}' | '\u{a}'..='\u{1f}' | '\u{7f}')
}

fn trim_ws(s: &str) -> &str {
    s.trim_matches(is_ws)
}

/// Read a string body after the opening quote. Returns the text and the rest
/// of the line after the closing quote.
fn parse_string(s: &str, line: usize) -> Result<(String, &str), ConfigError> {
    let mut out = String::new();
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Ok((out, &s[i + 1..])),
            '\\' => match chars.next() {
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some((_, 'n')) => out.push('\n'),
                Some(_) => return Err(ConfigError::BadEscape { line }),
                None => break,
            },
            c if is_control(c) => return Err(ConfigError::Syntax { line }),
            c => out.push(c),
        }
    }
    Err(ConfigError::Syntax { line })
}

fn parse_key(s: &str, line: usize) -> Result<(String, &str), ConfigError> {
    if let Some(body) = s.strip_prefix('"') {
        return parse_string(body, line);
    }
    if s.starts_with('\'') {
        return Err(ConfigError::SingleQuotes { line });
    }
    let end = s.find(|c| !is_bare(c)).unwrap_or(s.len());
    if end == 0 {
        return Err(ConfigError::Syntax { line });
    }
    Ok((s[..end].to_string(), &s[end..]))
}

fn parse_value(s: &str, line: usize) -> Result<(Value, &str), ConfigError> {
    if let Some(body) = s.strip_prefix('"') {
        let (text, rest) = parse_string(body, line)?;
        return Ok((Value::Str(text), rest));
    }
    if s.starts_with('\'') {
        return Err(ConfigError::SingleQuotes { line });
    }
    let end = s.find(|c: char| is_ws(c) || c == '#').unwrap_or(s.len());
    if end == 0 {
        return Err(ConfigError::Syntax { line });
    }
    let token = &s[..end];
    let value = match token {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        t if t.bytes().all(|b| b.is_ascii_digit()) && !(t.len() > 1 && t.starts_with('0')) => {
            // Above i64::MAX is not a valid TOML integer.
            t.parse::<i64>()
                .map_or(Value::Other, |n| Value::Int(n as u64))
        }
        _ => Value::Other,
    };
    Ok((value, &s[end..]))
}

/// A `#` comment must not hold control characters (tab is fine).
fn comment_ok(comment: &str, line: usize) -> Result<(), ConfigError> {
    if comment.chars().any(is_control) {
        Err(ConfigError::Syntax { line })
    } else {
        Ok(())
    }
}

/// Only whitespace or a `#` comment may follow a value or a header.
fn end_of_line(rest: &str, line: usize) -> Result<(), ConfigError> {
    let rest = rest.trim_start_matches(is_ws);
    if rest.is_empty() {
        Ok(())
    } else if rest.starts_with('#') {
        comment_ok(rest, line)
    } else {
        Err(ConfigError::Syntax { line })
    }
}

/// Read `[name]` and the end of the line. A name may have dots, so a newer
/// Buuz can add `[a.b]` sections that this core skips.
fn parse_header(raw: &str, line: usize) -> Result<&str, ConfigError> {
    let head = raw.strip_prefix('[').ok_or(ConfigError::Syntax { line })?;
    let close = head.find(']').ok_or(ConfigError::Syntax { line })?;
    let name = trim_ws(&head[..close]);
    end_of_line(&head[close + 1..], line)?;
    if name.is_empty() || !name.chars().all(|c| is_bare(c) || c == '.') {
        return Err(ConfigError::Syntax { line });
    }
    Ok(name)
}

fn bad_value(line: usize, key: &str) -> ConfigError {
    ConfigError::BadValue {
        line,
        key: key.to_string(),
    }
}

fn want_bool(value: Value, line: usize, key: &str) -> Result<bool, ConfigError> {
    match value {
        Value::Bool(b) => Ok(b),
        _ => Err(bad_value(line, key)),
    }
}

fn want_str(value: Value, line: usize, key: &str) -> Result<String, ConfigError> {
    match value {
        Value::Str(s) => Ok(s),
        _ => Err(bad_value(line, key)),
    }
}

fn dup(line: usize, key: &str) -> ConfigError {
    ConfigError::DuplicateKey {
        line,
        key: key.to_string(),
    }
}

fn is_cyrillic(c: char) -> bool {
    ('\u{0400}'..='\u{04FF}').contains(&c)
}

/// Check an abbreviation entry and give its normal form: a lowercase key and
/// a trimmed value. An empty value stays empty (it removes a built-in entry).
fn norm_abbreviation(key: &str, value: &str, line: usize) -> Result<(String, String), ConfigError> {
    let lower = check_key(key, line).map_err(ConfigError::Abbreviation)?;
    if value.is_empty() {
        return Ok((lower, String::new()));
    }
    let value = trim_ws(value);
    check_value(key, value, line).map_err(ConfigError::Abbreviation)?;
    if value.chars().count() > MAX_VALUE_CHARS || value.chars().any(is_control) {
        return Err(ConfigError::Abbreviation(AbbrevError::BadValue {
            line,
            key: key.to_string(),
        }));
    }
    Ok((lower, value.to_string()))
}

/// Check a `[keys]` entry and give its normal form: a lowercase key.
fn norm_mapping(key: &str, value: &str, line: usize) -> Result<(String, String), ConfigError> {
    let n = key.chars().count();
    let key_ok = (1..=3).contains(&n) && key.chars().all(is_input_char);
    let value_n = value.chars().count();
    let value_ok = value_n <= MAX_VALUE_CHARS && value.chars().all(|c| is_cyrillic(c) || c == '/');
    if !key_ok || !value_ok {
        return Err(ConfigError::BadKeys {
            line,
            key: key.to_string(),
        });
    }
    check_mapping(key, value).map_err(|e| match e {
        KeyError::BadKey { key } => ConfigError::BadKeys { line, key },
        KeyError::UnknownTarget { key, value } => ConfigError::UnknownTarget { line, key, value },
        KeyError::UpperCaseTarget { key, value } => {
            ConfigError::UpperCaseTarget { line, key, value }
        }
    })?;
    Ok((key.chars().map(to_lower).collect(), value.to_string()))
}

fn too_large(line: usize) -> ConfigError {
    ConfigError::TooLarge {
        line,
        limit: MAX_SECTION_ENTRIES,
    }
}

/// Add or replace an entry by key. A new entry goes last.
fn put(
    list: &mut Vec<(String, String)>,
    entry: (String, String),
    line: usize,
) -> Result<(), ConfigError> {
    if let Some(slot) = list.iter_mut().find(|(k, _)| *k == entry.0) {
        slot.1 = entry.1;
    } else if list.len() >= MAX_SECTION_ENTRIES {
        return Err(too_large(line));
    } else {
        list.push(entry);
    }
    Ok(())
}

fn take(list: &mut Vec<(String, String)>, key: &str) -> bool {
    let before = list.len();
    list.retain(|(k, _)| k != key);
    list.len() != before
}

impl Config {
    /// The base settings.
    pub fn preset(&self) -> Preset {
        self.preset
    }

    /// Override for [`Options::smart_vowels`]. `None` keeps the preset value.
    pub fn smart_vowels(&self) -> Option<bool> {
        self.smart_vowels
    }

    /// Override for the abbreviation switch. `Some(false)` gives no table.
    pub fn abbreviations(&self) -> Option<bool> {
        self.abbreviations
    }

    /// Override for [`Options::keep_latin`].
    pub fn keep_latin(&self) -> Option<bool> {
        self.keep_latin
    }

    /// Override for [`Options::auto_latin`].
    pub fn auto_latin(&self) -> Option<bool> {
        self.auto_latin
    }

    /// Override for [`Options::completion`].
    pub fn completion(&self) -> Option<bool> {
        self.completion
    }

    /// Override for [`Options::autocorrect`].
    pub fn autocorrect(&self) -> Option<bool> {
        self.autocorrect
    }

    /// Override for [`Options::learning`].
    pub fn learning(&self) -> Option<bool> {
        self.learning
    }

    /// Whether the shell must load the word list and set [`Options::words`]:
    /// true when completion or autocorrect is on (the override, else the
    /// preset; autocorrect is off in both presets). `options()` cannot load
    /// data, so shells read this first.
    pub fn needs_word_list(&self) -> bool {
        self.completion.unwrap_or(self.preset == Preset::Smart) || self.autocorrect.unwrap_or(false)
    }

    /// User abbreviations in file order, with lowercase keys. They go on top
    /// of the built-in list. An empty value removes a built-in entry.
    pub fn abbreviation_entries(&self) -> &[(String, String)] {
        &self.abbreviation_entries
    }

    /// Custom key mappings from the `[keys]` section: a lowercase Latin key
    /// of 1 to 3 input characters and a target: Cyrillic text that a built-in
    /// key gives, or an empty value that removes the key. `options()` applies
    /// them.
    pub fn keys(&self) -> &[(String, String)] {
        &self.keys
    }

    /// Set the base settings.
    pub fn set_preset(&mut self, preset: Preset) {
        self.preset = preset;
    }

    /// Set or clear the smart vowels override.
    pub fn set_smart_vowels(&mut self, value: Option<bool>) {
        self.smart_vowels = value;
    }

    /// Set or clear the abbreviations override.
    pub fn set_abbreviations(&mut self, value: Option<bool>) {
        self.abbreviations = value;
    }

    /// Set or clear the keep Latin override.
    pub fn set_keep_latin(&mut self, value: Option<bool>) {
        self.keep_latin = value;
    }

    /// Set or clear the auto Latin override.
    pub fn set_auto_latin(&mut self, value: Option<bool>) {
        self.auto_latin = value;
    }

    /// Set or clear the learning override.
    pub fn set_learning(&mut self, value: Option<bool>) {
        self.learning = value;
    }

    /// Set or clear the autocorrect override.
    pub fn set_autocorrect(&mut self, value: Option<bool>) {
        self.autocorrect = value;
    }

    /// Set or clear the completion override.
    pub fn set_completion(&mut self, value: Option<bool>) {
        self.completion = value;
    }

    /// Add or replace a user abbreviation. The key goes to lowercase and the
    /// value is trimmed. An empty value removes a built-in entry.
    pub fn set_abbreviation(&mut self, key: &str, value: &str) -> Result<(), ConfigError> {
        let entry = norm_abbreviation(key, value, 0)?;
        put(&mut self.abbreviation_entries, entry, 0)
    }

    /// Remove a user abbreviation. Gives `false` when there was none. To
    /// remove a built-in entry, call `set_abbreviation(key, "")`.
    pub fn remove_abbreviation(&mut self, key: &str) -> bool {
        let lower: String = key.chars().map(to_lower).collect();
        take(&mut self.abbreviation_entries, &lower)
    }

    /// Add or replace a key mapping. The key goes to lowercase.
    pub fn set_key(&mut self, key: &str, value: &str) -> Result<(), ConfigError> {
        let entry = norm_mapping(key, value, 0)?;
        put(&mut self.keys, entry, 0)
    }

    /// Remove a key mapping. Gives `false` when there was none.
    pub fn remove_key(&mut self, key: &str) -> bool {
        let lower: String = key.chars().map(to_lower).collect();
        take(&mut self.keys, &lower)
    }

    /// Read a config text. See the ADR for the format. A leading byte order
    /// mark is ignored. Unknown options and sections give warnings. Every
    /// other fault is an error.
    pub fn parse(text: &str) -> Result<Parsed, ConfigError> {
        if text.len() > MAX_CONFIG_BYTES {
            return Err(ConfigError::TooLarge {
                line: 0,
                limit: MAX_CONFIG_BYTES,
            });
        }
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut config = Config::default();
        let mut warnings = Vec::new();
        let mut section = Section::Top;
        let mut seen = [false; 3];
        let mut version_seen = false;
        let mut preset_seen = false;
        let mut abbrev_keys: HashSet<String> = HashSet::new();
        let mut map_keys: HashSet<String> = HashSet::new();
        let mut lines = 0;

        for (i, raw) in text.lines().enumerate() {
            let line = i + 1;
            lines = line;
            let raw = trim_ws(raw);
            if raw.is_empty() {
                continue;
            }
            if raw.starts_with('#') {
                comment_ok(raw, line)?;
                continue;
            }

            if raw.starts_with('[') {
                let name = match parse_header(raw, line) {
                    Ok(name) => name,
                    Err(_) if section == Section::Unknown => continue,
                    Err(e) => {
                        return Err(if version_seen {
                            e
                        } else {
                            ConfigError::MissingVersion { line }
                        })
                    }
                };
                if !version_seen {
                    return Err(ConfigError::MissingVersion { line });
                }
                let (next, slot) = match name {
                    "options" => (Section::Options, 0),
                    "abbreviations" => (Section::Abbreviations, 1),
                    "keys" => (Section::Keys, 2),
                    _ => {
                        warnings.push(ConfigWarning::UnknownSection {
                            line,
                            name: name.to_string(),
                        });
                        section = Section::Unknown;
                        continue;
                    }
                };
                if seen[slot] {
                    return Err(ConfigError::DuplicateSection {
                        line,
                        name: name.to_string(),
                    });
                }
                seen[slot] = true;
                section = next;
                continue;
            }

            if section == Section::Unknown {
                continue;
            }

            let (key, rest) = parse_key(raw, line)?;
            let rest = rest
                .trim_start_matches(is_ws)
                .strip_prefix('=')
                .ok_or(if version_seen {
                    ConfigError::Syntax { line }
                } else {
                    ConfigError::MissingVersion { line }
                })?;
            let rest = rest.trim_start_matches(is_ws);

            // An unknown option is skipped before its value is read, so a
            // newer Buuz can use value types that this core does not know.
            if section == Section::Options
                && !matches!(
                    key.as_str(),
                    "smart_vowels"
                        | "abbreviations"
                        | "keep_latin"
                        | "auto_latin"
                        | "completion"
                        | "autocorrect"
                        | "learning"
                )
            {
                warnings.push(ConfigWarning::UnknownOption { line, key });
                continue;
            }

            let (value, rest) = parse_value(rest, line)?;
            end_of_line(rest, line)?;

            if !version_seen {
                if key != "buuz_config" {
                    return Err(ConfigError::MissingVersion { line });
                }
                match value {
                    Value::Int(found) if found > CONFIG_VERSION => {
                        return Err(ConfigError::TooNew {
                            found,
                            supported: CONFIG_VERSION,
                        })
                    }
                    Value::Int(found) if found >= 1 => version_seen = true,
                    _ => return Err(bad_value(line, &key)),
                }
                continue;
            }

            match section {
                Section::Top => match key.as_str() {
                    "buuz_config" => return Err(dup(line, &key)),
                    "preset" => {
                        if std::mem::replace(&mut preset_seen, true) {
                            return Err(dup(line, &key));
                        }
                        let name = want_str(value, line, &key)?;
                        config.preset = match name.as_str() {
                            "smart" => Preset::Smart,
                            "classic" => Preset::Classic,
                            _ => return Err(bad_value(line, &key)),
                        };
                    }
                    _ => return Err(ConfigError::UnknownKey { line, key }),
                },
                Section::Options => {
                    let slot = match key.as_str() {
                        "smart_vowels" => &mut config.smart_vowels,
                        "abbreviations" => &mut config.abbreviations,
                        "keep_latin" => &mut config.keep_latin,
                        "auto_latin" => &mut config.auto_latin,
                        "completion" => &mut config.completion,
                        "autocorrect" => &mut config.autocorrect,
                        _ => &mut config.learning,
                    };
                    if slot.is_some() {
                        return Err(dup(line, &key));
                    }
                    *slot = Some(want_bool(value, line, &key)?);
                }
                Section::Abbreviations => {
                    let given = want_str(value, line, &key)?;
                    let (lower, expansion) = norm_abbreviation(&key, &given, line)?;
                    if !abbrev_keys.insert(lower.clone()) {
                        return Err(dup(line, &key));
                    }
                    if config.abbreviation_entries.len() >= MAX_SECTION_ENTRIES {
                        return Err(too_large(line));
                    }
                    config.abbreviation_entries.push((lower, expansion));
                }
                Section::Keys => {
                    let mapping = want_str(value, line, &key)?;
                    let (lower, mapping) = norm_mapping(&key, &mapping, line)?;
                    if !map_keys.insert(lower.clone()) {
                        return Err(dup(line, &key));
                    }
                    if config.keys.len() >= MAX_SECTION_ENTRIES {
                        return Err(too_large(line));
                    }
                    config.keys.push((lower, mapping));
                }
                Section::Unknown => {}
            }
        }

        if version_seen {
            Ok(Parsed { config, warnings })
        } else {
            Err(ConfigError::MissingVersion {
                line: first_content_line(text).unwrap_or(lines.max(1)),
            })
        }
    }

    /// The canonical text: parsing it gives back an equal `Config`, and
    /// writing that again gives the same text.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "buuz_config = {CONFIG_VERSION}");
        let _ = writeln!(s, "preset = \"{}\"", self.preset.name());

        let flags = [
            ("smart_vowels", self.smart_vowels),
            ("abbreviations", self.abbreviations),
            ("keep_latin", self.keep_latin),
            ("auto_latin", self.auto_latin),
            ("completion", self.completion),
            ("autocorrect", self.autocorrect),
            ("learning", self.learning),
        ];
        if flags.iter().any(|(_, v)| v.is_some()) {
            s.push_str("\n[options]\n");
            for (name, value) in flags {
                if let Some(v) = value {
                    let _ = writeln!(s, "{name} = {v}");
                }
            }
        }
        for (name, entries) in [
            ("abbreviations", &self.abbreviation_entries),
            ("keys", &self.keys),
        ] {
            if entries.is_empty() {
                continue;
            }
            let _ = write!(s, "\n[{name}]\n");
            for (k, v) in entries {
                push_key(&mut s, k);
                s.push_str(" = ");
                push_quoted(&mut s, v);
                s.push('\n');
            }
        }
        s
    }

    /// The [`Options`] for the composer: the preset, then the overrides, then
    /// the abbreviation table (built-in list plus the user's additions, minus
    /// the user's removals). `abbreviations = false` gives `None`. With
    /// `[keys]` entries this builds a full rule table, so call it once per
    /// config change and keep the result, not once per key press.
    pub fn options(&self) -> Options {
        let base = match self.preset {
            Preset::Smart => Options::smart(),
            Preset::Classic => Options::classic(),
        };
        let enabled = self.abbreviations.unwrap_or(base.abbreviations.is_some());
        let abbreviations = enabled.then(|| {
            if self.abbreviation_entries.is_empty() {
                Abbreviations::builtin()
            } else {
                Arc::new(Abbreviations::builtin_with(&self.abbreviation_entries))
            }
        });
        // A config without keys keeps the shared built-in table.
        let keys = if self.keys.is_empty() {
            None
        } else {
            Some(Arc::new(KeyMap::from_pairs(&self.keys).expect(
                "keys were checked when they were added to the config",
            )))
        };
        Options {
            keys,
            smart_vowels: self.smart_vowels.unwrap_or(base.smart_vowels),
            abbreviations,
            keep_latin: self.keep_latin.unwrap_or(base.keep_latin),
            auto_latin: self.auto_latin.unwrap_or(base.auto_latin),
            words: None,
            completion: self.completion.unwrap_or(base.completion),
            autocorrect: self.autocorrect.unwrap_or(base.autocorrect),
            user: None,
            learning: self.learning.unwrap_or(base.learning),
        }
    }
}

fn first_content_line(text: &str) -> Option<usize> {
    text.lines()
        .position(|l| {
            let l = trim_ws(l);
            !l.is_empty() && !l.starts_with('#')
        })
        .map(|i| i + 1)
}

fn push_quoted(s: &mut String, text: &str) {
    s.push('"');
    for c in text.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            c => s.push(c),
        }
    }
    s.push('"');
}

fn push_key(s: &mut String, key: &str) {
    if !key.is_empty() && key.chars().all(is_bare) {
        s.push_str(key);
    } else {
        push_quoted(s, key);
    }
}
