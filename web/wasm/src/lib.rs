// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

//! WebAssembly wrapper: exposes `buuz-core` to the website so the page gives
//! exactly the same output as the Windows and macOS input methods.

use wasm_bindgen::prelude::*;

/// Transliterate Latin text to Mongolian Cyrillic ("sain baina uu" -> "сайн байна уу"),
/// with smart vowel harmony on.
#[wasm_bindgen]
pub fn convert(input: &str) -> String {
    buuz_core::convert(input)
}

/// Like `convert`, with smart vowel harmony and abbreviation expansion switched
/// on or off (`bn` -> `байна` needs `abbreviations`). Links and emails stay
/// Latin here whatever the flags are; `false, false` is not the classic
/// behaviour. Use `convert_classic` for the original Buuz.
#[wasm_bindgen]
pub fn convert_with(input: &str, smart_vowels: bool, abbreviations: bool) -> String {
    let opts = buuz_core::Options {
        smart_vowels,
        abbreviations: abbreviations.then(buuz_core::Abbreviations::builtin),
        ..Default::default()
    };
    buuz_core::convert_with(input, &opts)
}

/// Classic mode: the original Buuz rules with every smart feature off,
/// including any added later (autocorrect, completion). The "Classic" switch
/// on the site uses this rather than `convert_with(.., false, false)`.
#[wasm_bindgen]
pub fn convert_classic(input: &str) -> String {
    buuz_core::convert_with(input, &buuz_core::Options::classic())
}

/// The built-in abbreviation list as text, one `abbreviation = expansion` per line.
#[wasm_bindgen]
pub fn abbreviation_list() -> String {
    buuz_core::Abbreviations::builtin().to_text()
}

// --- Portable config (see docs/decisions/0005-portable-config-format.md) -----
//
// The page keeps the config as text. Errors and warnings go to JavaScript as
// machine-readable strings so the page can show them in its own language:
// `code<TAB>number<TAB>detail`. `number` is a line number (0 when there is
// none); `detail` is a key or section name. For `too_new` the number is the
// version found and the detail is the version supported. For `too_large` the
// detail is the limit.

fn tag(code: &str, number: usize, detail: &str) -> String {
    format!(
        "{code}\t{number}\t{}",
        detail.replace(['\t', '\n', '\r'], " ")
    )
}

fn error_tag(e: &buuz_core::ConfigError) -> String {
    use buuz_core::{AbbrevError, ConfigError as E};
    match e {
        E::MissingVersion { line } => tag("missing_version", *line, ""),
        E::TooNew { found, supported } => tag("too_new", *found as usize, &supported.to_string()),
        E::Syntax { line } => tag("syntax", *line, ""),
        E::UnknownKey { line, key } => tag("unknown_key", *line, key),
        E::BadValue { line, key } => tag("bad_value", *line, key),
        E::BadEscape { line } => tag("bad_escape", *line, ""),
        E::SingleQuotes { line } => tag("single_quotes", *line, ""),
        E::DuplicateKey { line, key } => tag("duplicate_key", *line, key),
        E::DuplicateSection { line, name } => tag("duplicate_section", *line, name),
        E::Abbreviation(AbbrevError::BadKey { line, key })
        | E::Abbreviation(AbbrevError::BadValue { line, key }) => {
            tag("bad_abbreviation", *line, key)
        }
        E::Abbreviation(AbbrevError::MissingEquals { line }) => tag("syntax", *line, ""),
        E::BadKeys { line, key } => tag("bad_keys", *line, key),
        E::TooLarge { line: 0, .. } => tag("too_large", 0, "1 MiB"),
        E::TooLarge { line, limit } => tag("too_many_entries", *line, &limit.to_string()),
        other => tag("other", 0, &other.to_string()),
    }
}

fn warning_tag(w: &buuz_core::ConfigWarning) -> String {
    use buuz_core::ConfigWarning as W;
    match w {
        W::UnknownOption { line, key } => tag("unknown_option", *line, key),
        W::UnknownSection { line, name } => tag("unknown_section", *line, name),
        other => tag("other", 0, &other.to_string()),
    }
}

fn parse_config(text: &str) -> Result<buuz_core::Parsed, String> {
    buuz_core::Config::parse(text).map_err(|e| error_tag(&e))
}

/// A config that was read and written back in canonical form.
#[wasm_bindgen]
pub struct ConfigReport {
    text: String,
    warnings: Vec<String>,
    classic: bool,
    smart_vowels: bool,
    abbreviations: bool,
    completion: bool,
    autocorrect: bool,
    double_v: bool,
    habit_keys: bool,
    word_choice: bool,
}

#[wasm_bindgen]
impl ConfigReport {
    /// The canonical config text.
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }
    /// What the parser ignored, as `code<TAB>line<TAB>name` strings.
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
    /// The preset is classic.
    #[wasm_bindgen(getter)]
    pub fn classic(&self) -> bool {
        self.classic
    }
    /// Smart vowels are on after the preset and overrides.
    #[wasm_bindgen(getter)]
    pub fn smart_vowels(&self) -> bool {
        self.smart_vowels
    }
    /// Abbreviation expansion is on after the preset and overrides.
    #[wasm_bindgen(getter)]
    pub fn abbreviations(&self) -> bool {
        self.abbreviations
    }
    /// Completion is on after the preset and overrides. It needs the word
    /// list (`load_word_list`) to show anything.
    #[wasm_bindgen(getter)]
    pub fn completion(&self) -> bool {
        self.completion
    }
    /// Autocorrect is on after the preset and overrides. It needs the word
    /// list (`load_word_list`) to change anything.
    #[wasm_bindgen(getter)]
    pub fn autocorrect(&self) -> bool {
        self.autocorrect
    }
    /// Double V (`vv` is үү) is on after the preset and overrides.
    #[wasm_bindgen(getter)]
    pub fn double_v(&self) -> bool {
        self.double_v
    }
    /// Habit keys (`ts` is ц, `zh` is ж) are on after the preset and overrides.
    #[wasm_bindgen(getter)]
    pub fn habit_keys(&self) -> bool {
        self.habit_keys
    }

    /// Word choice (о/ө and у/ү by the word list) is on after the preset and overrides.
    #[wasm_bindgen(getter)]
    pub fn word_choice(&self) -> bool {
        self.word_choice
    }
}

/// The default config (smart preset, no overrides) as text.
#[wasm_bindgen]
pub fn default_config_text() -> String {
    buuz_core::Config::default().to_text()
}

/// Read a config text and give back its canonical form, the warnings and the
/// effective switches. The error is a tagged string (see above).
#[wasm_bindgen]
pub fn config_normalize(text: &str) -> Result<ConfigReport, String> {
    let parsed = parse_config(text)?;
    let o = parsed.config.options();
    Ok(ConfigReport {
        text: parsed.config.to_text(),
        warnings: parsed.warnings.iter().map(warning_tag).collect(),
        classic: parsed.config.preset() == buuz_core::Preset::Classic,
        smart_vowels: o.smart_vowels,
        abbreviations: o.abbreviations.is_some(),
        completion: o.completion,
        autocorrect: o.autocorrect,
        double_v: o.double_v,
        habit_keys: o.habit_keys,
        word_choice: o.word_choice,
    })
}

/// Set the preset of a config and return the new canonical text. Overrides,
/// abbreviations and keys stay.
#[wasm_bindgen]
pub fn config_set_classic(text: &str, classic: bool) -> Result<String, String> {
    let mut config = parse_config(text)?.config;
    config.set_preset(if classic {
        buuz_core::Preset::Classic
    } else {
        buuz_core::Preset::Smart
    });
    Ok(config.to_text())
}

/// Switch `smart_vowels`, `abbreviations`, `completion`, `autocorrect`, `double_v`, `habit_keys` or `word_choice` and return the new canonical
/// text. A value equal to the preset's own is not written, so the default
/// config stays free of overrides.
#[wasm_bindgen]
pub fn config_set_option(text: &str, name: &str, value: bool) -> Result<String, String> {
    let mut config = parse_config(text)?.config;
    let base = if config.preset() == buuz_core::Preset::Classic {
        buuz_core::Options::classic()
    } else {
        buuz_core::Options::smart()
    };
    match name {
        "smart_vowels" => config.set_smart_vowels((value != base.smart_vowels).then_some(value)),
        "abbreviations" => {
            config.set_abbreviations((value != base.abbreviations.is_some()).then_some(value))
        }
        "completion" => config.set_completion((value != base.completion).then_some(value)),
        "autocorrect" => config.set_autocorrect((value != base.autocorrect).then_some(value)),
        "double_v" => config.set_double_v((value != base.double_v).then_some(value)),
        "habit_keys" => config.set_habit_keys((value != base.habit_keys).then_some(value)),
        "word_choice" => config.set_word_choice((value != base.word_choice).then_some(value)),
        _ => return Err(tag("unknown_option", 0, name)),
    }
    Ok(config.to_text())
}

thread_local! {
    // The options of the last two config texts (the page's config and its
    // typewriter-demo config), so typing does not parse either again.
    static RECENT: std::cell::RefCell<Vec<(String, buuz_core::Options)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// The options of a config text, from the two-entry cache.
fn options_of(config_text: &str) -> Result<buuz_core::Options, String> {
    RECENT.with(|recent| {
        let mut recent = recent.borrow_mut();
        let at = match recent.iter().position(|(t, _)| t == config_text) {
            Some(i) => i,
            None => {
                let opts = parse_config(config_text)?.config.options();
                if recent.len() >= 2 {
                    recent.remove(0);
                }
                recent.push((config_text.to_string(), opts));
                recent.len() - 1
            }
        };
        Ok(recent[at].1.clone())
    })
}

/// Transliterate with the options of a config text. The error is a tagged
/// string (see above). With autocorrect on, the loaded word list (`load_word_list`) goes into the
/// options, so a word that Space or punctuation ends is corrected as in the
/// core (section 13). A newline does not correct. Without a list, nothing
/// changes.
#[wasm_bindgen]
pub fn convert_with_config(input: &str, config_text: &str) -> Result<String, String> {
    let mut opts = options_of(config_text)?;
    if opts.autocorrect {
        opts.words = WORDS.with(|w| w.borrow().clone());
        opts.user = user_snapshot();
    }
    opts.learning = false;
    Ok(buuz_core::convert_with(input, &opts))
}

// --- Completion (issue #2) ---------------------------------------------------
//
// The page converts the whole Latin box at once, so it has no composer of its
// own. For the word at the caret it asks here: the core composer runs the
// word with the word list and gives the ghost text, so the core stays the one
// source of the rules. An accepted word goes into the Latin box as Cyrillic:
// the converter copies Cyrillic through unchanged (a test holds this).

thread_local! {
    // The word list, parsed once by `load_word_list`.
    static WORDS: std::cell::RefCell<Option<std::sync::Arc<buuz_core::WordList>>> =
        const { std::cell::RefCell::new(None) };
}

/// Parse the word list text (`data/wordlist/mn.txt`) once and keep it. Gives
/// the number of words. The error is a tagged string `bad_word_list`, with
/// the line number.
#[wasm_bindgen]
pub fn load_word_list(text: &str) -> Result<usize, String> {
    let list = buuz_core::WordList::parse(text).map_err(|e| tag("bad_word_list", e.line(), ""))?;
    let n = list.len();
    WORDS.with(|w| *w.borrow_mut() = Some(std::sync::Arc::new(list)));
    Ok(n)
}

/// Whether `load_word_list` has kept a list.
#[wasm_bindgen]
pub fn word_list_loaded() -> bool {
    WORDS.with(|w| w.borrow().is_some())
}

/// The word at the end of `before_caret`: its trailing run of input
/// characters (Latin letters, `'`, `"`, `ö`, `ü`).
fn trailing_word(before_caret: &str) -> &str {
    let start = before_caret
        .char_indices()
        .rev()
        .take_while(|&(_, c)| buuz_core::is_input_char(c))
        .last()
        .map_or(before_caret.len(), |(i, _)| i);
    &before_caret[start..]
}

/// A completion for the word at the caret.
#[wasm_bindgen]
pub struct Completion {
    rest: String,
    word: String,
    replaces: usize,
}

#[wasm_bindgen]
impl Completion {
    /// The ghost text: the Cyrillic rest of the word.
    #[wasm_bindgen(getter)]
    pub fn rest(&self) -> String {
        self.rest.clone()
    }
    /// The whole completed word in Cyrillic (what Tab puts in the box).
    #[wasm_bindgen(getter)]
    pub fn word(&self) -> String {
        self.word.clone()
    }
    /// How many UTF-16 units of Latin before the caret the word replaces.
    #[wasm_bindgen(getter)]
    pub fn replaces(&self) -> usize {
        self.replaces
    }
}

/// Complete the word at the caret. `before_caret` and `after_caret` are the
/// text on each side. The word is the run of input characters that ends at
/// the caret; a run after Cyrillic, a digit or any other character starts a
/// new word. Gives nothing when the caret is inside a word (an input
/// character follows it), completion is off, no list is loaded, or no word is
/// a clear winner. One `Composer` runs the word, so the rest and the whole
/// word always agree.
#[wasm_bindgen]
pub fn complete_at(before_caret: &str, after_caret: &str, config_text: &str) -> Option<Completion> {
    if after_caret
        .chars()
        .next()
        .is_some_and(buuz_core::is_input_char)
    {
        return None;
    }
    let word = trailing_word(before_caret);
    if word.is_empty() {
        return None;
    }
    let mut opts = options_of(config_text).ok()?;
    if !opts.completion {
        return None;
    }
    opts.words = WORDS.with(|w| w.borrow().clone());
    opts.words.as_ref()?;
    opts.user = user_snapshot();
    opts.learning = false;
    let mut c = buuz_core::Composer::with_options(opts);
    for ch in word.chars() {
        c.process_key(buuz_core::Key::Char(ch));
    }
    let rest = c.completion()?.to_string();
    Some(Completion {
        word: c.preedit() + &rest,
        rest,
        replaces: word.encode_utf16().count(),
    })
}

// --- Personal dictionary (issue #12, docs/typing-rules.md section 14) ---------
//
// The page keeps the dictionary as text in localStorage. These functions read
// and change that text; none of them stores anything. `set_user_dict` hands
// the text to the converter and to completion. The web converter never
// learns: it converts the whole box again on each input, so a use count would
// count one word many times, and it has no Backspace undo to reject from.

thread_local! {
    // The text last given to `set_user_dict` and the snapshot built from it.
    static USER: std::cell::RefCell<(String, Option<std::sync::Arc<buuz_core::UserSnapshot>>)> =
        const { std::cell::RefCell::new((String::new(), None)) };
}

fn user_snapshot() -> Option<std::sync::Arc<buuz_core::UserSnapshot>> {
    USER.with(|u| u.borrow().1.clone())
}

fn user_tag(p: buuz_core::UserDictProblem) -> &'static str {
    use buuz_core::UserDictProblem as P;
    match p {
        P::BadLine => "ud_bad_line",
        P::UnknownKind => "ud_unknown_kind",
        P::BadCount => "ud_bad_count",
        P::EmptyWord => "ud_empty_word",
        P::BadWord => "ud_bad_word",
        P::WordTooLong => "ud_word_too_long",
        P::Duplicate => "ud_duplicate",
    }
}

fn parse_user(text: &str) -> Result<buuz_core::ParsedUserDict, String> {
    buuz_core::UserDict::parse(text).map_err(|_| tag("too_large", 0, "1 MiB"))
}

/// A dictionary text that was read and written back in canonical form.
#[wasm_bindgen]
pub struct UserDictReport {
    text: String,
    warnings: Vec<String>,
}

#[wasm_bindgen]
impl UserDictReport {
    /// The canonical text.
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// The skipped lines, as `code<TAB>line<TAB>` strings.
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

/// Read a dictionary text (a saved one or an imported file). A bad line is
/// skipped and reported; a text over 1 MiB gives the error `too_large`.
#[wasm_bindgen]
pub fn userdict_normalize(text: &str) -> Result<UserDictReport, String> {
    let parsed = parse_user(text)?;
    Ok(UserDictReport {
        text: parsed.dict.to_text(),
        warnings: parsed
            .warnings
            .iter()
            .map(|w| tag(user_tag(w.problem), w.line, ""))
            .collect(),
    })
}

/// Add a word (Cyrillic letters, `-` and `'` only) and give the new text. The
/// error is `ud_empty_word`, `ud_bad_word`, `ud_word_too_long` or
/// `ud_not_cyrillic`.
#[wasm_bindgen]
pub fn userdict_add(text: &str, word: &str) -> Result<String, String> {
    let word = word.trim();
    let mut dict = parse_user(text)?.dict;
    let code = if word.is_empty() {
        "ud_empty_word"
    } else if word.chars().count() > buuz_core::MAX_USERDICT_WORD_CHARS {
        "ud_word_too_long"
    } else if word.chars().any(|c| c.is_whitespace() || c.is_control()) {
        "ud_bad_word"
    } else if !word
        .chars()
        .all(|c| ('\u{400}'..='\u{4ff}').contains(&c) || c == '-' || c == '\'')
    {
        "ud_not_cyrillic"
    } else if dict.add(word) {
        return Ok(dict.to_text());
    } else {
        "ud_bad_word"
    };
    Err(tag(code, 0, ""))
}

/// Delete every entry of a word and give the new text.
#[wasm_bindgen]
pub fn userdict_remove(text: &str, word: &str) -> String {
    match parse_user(text) {
        Ok(mut p) => {
            p.dict.remove(word);
            p.dict.to_text()
        }
        Err(_) => String::new(),
    }
}

/// The text of an empty dictionary.
#[wasm_bindgen]
pub fn userdict_clear() -> String {
    String::new()
}

/// The entries as `kind<TAB>word<TAB>count` strings, the least recently used
/// first.
#[wasm_bindgen]
pub fn userdict_entries(text: &str) -> Vec<String> {
    parse_user(text).map_or_else(
        |_| Vec::new(),
        |p| p.dict.to_text().lines().map(str::to_string).collect(),
    )
}

/// Use this dictionary text for autocorrect and completion. The snapshot is
/// built again only when the text changes. A text that is too large is an
/// error and changes nothing.
#[wasm_bindgen]
pub fn set_user_dict(text: &str) -> Result<(), String> {
    if USER.with(|u| u.borrow().0 == text) {
        return Ok(());
    }
    let dict = parse_user(text)?.dict;
    let snap = (!dict.is_empty()).then(|| std::sync::Arc::new(dict.snapshot()));
    USER.with(|u| *u.borrow_mut() = (text.to_string(), snap));
    Ok(())
}

// --- Calibration (issue #4) ---------------------------------------------------
//
// The page sends the sentences and what the user typed as JSON, once, when the
// test ends. Nothing is stored. There is no serde, so the JSON is read and
// written by hand. Errors are tagged strings: `bad_pairs` for input that is
// not `[["cyrillic", "typed"], ...]`, `bad_indexes` for a bad index list.

/// The most bytes of JSON read, the most pairs, the longest sentence or typed
/// line (in characters), and the most accepted indexes.
const MAX_JSON_BYTES: usize = 64 * 1024;
const MAX_PAIRS: usize = 50;
const MAX_STRING_CHARS: usize = 500;
const MAX_INDEXES: usize = 64;

/// A tiny reader for the two JSON shapes the page sends.
struct Json<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Json<'a> {
    fn new(text: &'a str) -> Self {
        Json {
            chars: text.chars().peekable(),
        }
    }

    fn ws(&mut self) {
        while matches!(self.chars.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.chars.next();
        }
    }

    fn eat(&mut self, c: char) -> Option<()> {
        self.ws();
        (self.chars.next() == Some(c)).then_some(())
    }

    /// `,` or the closing bracket: true when the list goes on.
    fn more(&mut self, close: char) -> Option<bool> {
        self.ws();
        match self.chars.next()? {
            ',' => Some(true),
            c if c == close => Some(false),
            _ => None,
        }
    }

    fn at_close(&mut self, close: char) -> bool {
        self.ws();
        if self.chars.peek() == Some(&close) {
            self.chars.next();
            true
        } else {
            false
        }
    }

    fn hex4(&mut self) -> Option<u32> {
        let mut n = 0;
        for _ in 0..4 {
            n = n * 16 + self.chars.next()?.to_digit(16)?;
        }
        Some(n)
    }

    fn string(&mut self) -> Option<String> {
        self.eat('"')?;
        let mut out = String::new();
        loop {
            match self.chars.next()? {
                '"' => return Some(out),
                '\\' => match self.chars.next()? {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let hi = self.hex4()?;
                        let code = if (0xD800..0xDC00).contains(&hi) {
                            if self.chars.next()? != '\\' || self.chars.next()? != 'u' {
                                return None;
                            }
                            let lo = self.hex4()?;
                            if !(0xDC00..0xE000).contains(&lo) {
                                return None;
                            }
                            0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                        } else {
                            hi
                        };
                        out.push(char::from_u32(code)?);
                    }
                    _ => return None,
                },
                c if (c as u32) < 0x20 => return None,
                c => out.push(c),
            }
            if out.chars().count() > MAX_STRING_CHARS {
                return None;
            }
        }
    }

    fn index(&mut self) -> Option<usize> {
        self.ws();
        let mut n: usize = 0;
        let mut any = false;
        while let Some(d) = self.chars.peek().and_then(|c| c.to_digit(10)) {
            n = n.checked_mul(10)?.checked_add(d as usize)?;
            any = true;
            self.chars.next();
        }
        any.then_some(n)
    }

    fn end(&mut self) -> Option<()> {
        self.ws();
        self.chars.next().is_none().then_some(())
    }
}

fn read_pairs(text: &str) -> Option<Vec<(String, String)>> {
    if text.len() > MAX_JSON_BYTES {
        return None;
    }
    let mut j = Json::new(text);
    j.eat('[')?;
    let mut pairs = Vec::new();
    if !j.at_close(']') {
        loop {
            j.eat('[')?;
            let a = j.string()?;
            j.eat(',')?;
            let b = j.string()?;
            j.eat(']')?;
            pairs.push((a, b));
            if pairs.len() > MAX_PAIRS {
                return None;
            }
            if !j.more(']')? {
                break;
            }
        }
    }
    j.end()?;
    Some(pairs)
}

fn read_indexes(text: &str) -> Option<Vec<usize>> {
    if text.len() > MAX_JSON_BYTES {
        return None;
    }
    let mut j = Json::new(text);
    j.eat('[')?;
    let mut out = Vec::new();
    if !j.at_close(']') {
        loop {
            let i = j.index()?;
            if out.contains(&i) || out.len() >= MAX_INDEXES {
                return None;
            }
            out.push(i);
            if !j.more(']')? {
                break;
            }
        }
    }
    j.end()?;
    Some(out)
}

fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn push_char(out: &mut String, c: char) {
    push_json_string(out, c.encode_utf8(&mut [0; 4]));
}

fn calibrate_pairs(pairs_json: &str) -> Result<buuz_core::Calibration, String> {
    let pairs = read_pairs(pairs_json).ok_or_else(|| tag("bad_pairs", 0, ""))?;
    let refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    Ok(buuz_core::calibrate(&refs))
}

/// Compare what the user typed with the sentences and suggest keys. `pairs_json`
/// is `[["cyrillic", "typed"], ...]`. The result is JSON:
/// `{"suggestions":[{"letter","chunk","count","default_count","tie","displaces","pair","target"}],
/// "conflicts":[{"letter","chunk","count","other","other_count"}],
/// "unsupported":[{"letter","chunk","count"}],"unmatched":[index],"skipped_letters":[letter]}`.
/// `displaces` is a letter or null. Sentence indexes in `unmatched` start at 0.
#[wasm_bindgen]
pub fn calibrate_json(pairs_json: &str) -> Result<String, String> {
    let c = calibrate_pairs(pairs_json)?;
    let mut o = String::from("{\"suggestions\":[");
    for (i, s) in c.suggestions.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str("{\"letter\":");
        push_char(&mut o, s.letter);
        o.push_str(",\"chunk\":");
        push_json_string(&mut o, &s.chunk);
        o.push_str(&format!(
            ",\"count\":{},\"default_count\":{},\"tie\":{},\"displaces\":",
            s.count, s.default_count, s.tie
        ));
        match s.displaces {
            Some(d) => push_char(&mut o, d),
            None => o.push_str("null"),
        }
        o.push_str(&format!(",\"pair\":{},\"target\":", s.pair));
        push_json_string(&mut o, &s.target());
        o.push('}');
    }
    o.push_str("],\"conflicts\":[");
    for (i, k) in c.conflicts.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str("{\"letter\":");
        push_char(&mut o, k.letter);
        o.push_str(",\"chunk\":");
        push_json_string(&mut o, &k.chunk);
        o.push_str(&format!(",\"count\":{},\"other\":", k.count));
        push_char(&mut o, k.other);
        o.push_str(&format!(",\"other_count\":{}}}", k.other_count));
    }
    o.push_str("],\"unsupported\":[");
    for (i, u) in c.unsupported.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str("{\"letter\":");
        push_char(&mut o, u.letter);
        o.push_str(",\"chunk\":");
        push_json_string(&mut o, &u.chunk);
        o.push_str(&format!(",\"count\":{}}}", u.count));
    }
    o.push_str("],\"unmatched\":[");
    for (i, u) in c.unmatched.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str(&u.to_string());
    }
    o.push_str("],\"skipped_letters\":[");
    for (i, l) in c.skipped_letters.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        push_char(&mut o, *l);
    }
    o.push_str("]}");
    Ok(o)
}

/// Add the accepted suggestions to a config and return the new canonical text.
/// `accepted_indexes_json` is a list of positions in `calibrate_json`'s
/// `suggestions`, such as `[0,2]`. The pairs are calibrated again, so the
/// positions match. The config changes only when every accepted entry fits.
#[wasm_bindgen]
pub fn calibrate_apply(
    config_text: &str,
    pairs_json: &str,
    accepted_indexes_json: &str,
) -> Result<String, String> {
    let mut config = parse_config(config_text)?.config;
    let mut c = calibrate_pairs(pairs_json)?;
    let wanted = read_indexes(accepted_indexes_json).ok_or_else(|| tag("bad_indexes", 0, ""))?;
    if wanted.iter().any(|i| *i >= c.suggestions.len()) {
        return Err(tag("bad_indexes", 0, ""));
    }
    c.suggestions = c
        .suggestions
        .into_iter()
        .enumerate()
        .filter(|(i, _)| wanted.contains(i))
        .map(|(_, s)| s)
        .collect();
    c.apply(&mut config).map_err(|e| error_tag(&e))?;
    Ok(config.to_text())
}

#[cfg(test)]
mod tests {
    const LIST: &str = "# test\nгэрлэх\t100\nгэрлэлт\t10\nбайна\t500\n";

    #[test]
    fn userdict_normalize_reports_warnings_and_too_large() {
        let r = super::userdict_normalize("add\tТэнгис\t1\nbad line\nadd\tтэнгис\t1\n").unwrap();
        assert_eq!(r.text(), "add\tтэнгис\t1\n");
        assert_eq!(r.warnings(), vec!["ud_bad_line\t2\t", "ud_duplicate\t3\t"]);
        let big = "a".repeat(1024 * 1024 + 1);
        assert!(super::userdict_normalize(&big)
            .err()
            .unwrap()
            .starts_with("too_large\t0\t"));
    }

    #[test]
    fn userdict_add_remove_list_clear() {
        let t = super::userdict_add("", "Тэнгис").unwrap();
        assert_eq!(t, "add\tтэнгис\t1\n");
        for (w, code) in [
            ("", "ud_empty_word"),
            ("a b", "ud_bad_word"),
            ("tengis", "ud_not_cyrillic"),
        ] {
            let e = super::userdict_add(&t, w).err().unwrap();
            assert!(e.starts_with(code), "{w}: {e}");
        }
        let long = "а".repeat(65);
        assert!(super::userdict_add(&t, &long)
            .err()
            .unwrap()
            .starts_with("ud_word_too_long"));
        let t = format!("{t}use\tтэнгис\t4\n");
        assert_eq!(
            super::userdict_entries(&t),
            vec!["add\tтэнгис\t1", "use\tтэнгис\t4"]
        );
        assert_eq!(super::userdict_remove(&t, "Тэнгис"), "");
        assert_eq!(super::userdict_clear(), "");
    }

    #[test]
    fn user_dictionary_feeds_autocorrect_and_completion() {
        let cfg =
            super::config_set_option(&super::default_config_text(), "autocorrect", true).unwrap();
        super::load_word_list("дэлгүүр\t100\n").unwrap();
        super::set_user_dict("").unwrap();
        assert_eq!(
            super::convert_with_config("delgur ", &cfg).unwrap(),
            "дэлгүүр "
        );
        // A rejected word is not corrected; an added word is not either.
        super::set_user_dict("reject\tдэлгүр\t1\n").unwrap();
        assert_eq!(
            super::convert_with_config("delgur ", &cfg).unwrap(),
            "дэлгүр "
        );
        // An added word shows up in completion.
        super::set_user_dict("add\tтэнгис\t1\n").unwrap();
        let c = super::complete_at("tengi", "", &cfg).expect("completion");
        assert_eq!(c.word(), "тэнгис");
        // Same text again keeps the cached snapshot; a new text replaces it.
        super::set_user_dict("add\tтэнгис\t1\n").unwrap();
        super::set_user_dict("").unwrap();
        assert!(super::complete_at("tengi", "", &cfg).is_none());
        assert!(super::set_user_dict(&"a".repeat(1024 * 1024 + 1)).is_err());
    }

    #[test]
    fn completion_option_is_switchable_and_reported() {
        let cfg = super::default_config_text();
        assert!(super::config_normalize(&cfg).unwrap().completion());
        let off = super::config_set_option(&cfg, "completion", false).unwrap();
        assert!(off.contains("completion"));
        assert!(!super::config_normalize(&off).unwrap().completion());
        let on = super::config_set_option(&off, "completion", true).unwrap();
        assert_eq!(on, cfg);
        let classic = super::config_set_classic(&cfg, true).unwrap();
        assert!(!super::config_normalize(&classic).unwrap().completion());
    }

    #[test]
    fn double_v_option_is_switchable_and_reported() {
        let cfg = super::default_config_text();
        assert!(super::config_normalize(&cfg).unwrap().double_v());
        let off = super::config_set_option(&cfg, "double_v", false).unwrap();
        assert!(off.contains("double_v = false"));
        assert!(!super::config_normalize(&off).unwrap().double_v());
        let on = super::config_set_option(&off, "double_v", true).unwrap();
        assert_eq!(on, cfg);
        let classic = super::config_set_classic(&cfg, true).unwrap();
        assert!(!super::config_normalize(&classic).unwrap().double_v());
        let classic_on = super::config_set_option(&classic, "double_v", true).unwrap();
        assert!(super::config_normalize(&classic_on).unwrap().double_v());
    }

    #[test]
    fn word_choice_option_is_switchable_and_reported() {
        let cfg = super::default_config_text();
        assert!(super::config_normalize(&cfg).unwrap().word_choice());
        let off = super::config_set_option(&cfg, "word_choice", false).unwrap();
        assert!(off.contains("word_choice = false"));
        assert!(!super::config_normalize(&off).unwrap().word_choice());
        let classic = super::config_set_classic(&cfg, true).unwrap();
        assert!(!super::config_normalize(&classic).unwrap().word_choice());
    }

    #[test]
    fn habit_keys_option_is_switchable_and_reported() {
        let cfg = super::default_config_text();
        assert!(super::config_normalize(&cfg).unwrap().habit_keys());
        let off = super::config_set_option(&cfg, "habit_keys", false).unwrap();
        assert!(off.contains("habit_keys = false"));
        assert!(!super::config_normalize(&off).unwrap().habit_keys());
        let on = super::config_set_option(&off, "habit_keys", true).unwrap();
        assert_eq!(on, cfg);
        let classic = super::config_set_classic(&cfg, true).unwrap();
        assert!(!super::config_normalize(&classic).unwrap().habit_keys());
    }

    #[test]
    fn autocorrect_option_is_switchable_and_reported() {
        let cfg = super::default_config_text();
        assert!(!super::config_normalize(&cfg).unwrap().autocorrect());
        let on = super::config_set_option(&cfg, "autocorrect", true).unwrap();
        assert!(on.contains("autocorrect = true"));
        assert!(super::config_normalize(&on).unwrap().autocorrect());
        // Completion stays reported on its own.
        let no_c = super::config_set_option(&cfg, "completion", false).unwrap();
        assert!(!super::config_normalize(&no_c).unwrap().completion());
        assert!(!super::config_normalize(&no_c).unwrap().autocorrect());
        assert!(super::config_normalize(&on).unwrap().completion());
        // Back to the preset value: the override goes away.
        assert_eq!(
            super::config_set_option(&on, "autocorrect", false).unwrap(),
            cfg
        );
    }

    #[test]
    fn convert_autocorrects_with_a_list_and_the_switch() {
        let cfg = super::default_config_text();
        let on = super::config_set_option(&cfg, "autocorrect", true).unwrap();
        // No list loaded: unchanged.
        assert_eq!(
            super::convert_with_config("delgur ", &on).unwrap(),
            "дэлгүр "
        );
        super::load_word_list("дэлгүүр\t100\n").unwrap();
        assert_eq!(
            super::convert_with_config("delgur ", &on).unwrap(),
            "дэлгүүр "
        );
        assert_eq!(
            super::convert_with_config("delgur.", &on).unwrap(),
            "дэлгүүр."
        );
        // Switch off: unchanged.
        assert_eq!(
            super::convert_with_config("delgur ", &cfg).unwrap(),
            "дэлгүр "
        );
        // A newline, or the end of the text, does not correct.
        assert_eq!(
            super::convert_with_config("delgur\n", &on).unwrap(),
            "дэлгүр\n"
        );
        assert_eq!(super::convert_with_config("delgur", &on).unwrap(), "дэлгүр");
    }

    /// (rest, word, replaces) of a completion.
    fn at(before: &str, after: &str, cfg: &str) -> Option<(String, String, usize)> {
        super::complete_at(before, after, cfg).map(|c| (c.rest(), c.word(), c.replaces()))
    }

    fn got(rest: &str, word: &str, n: usize) -> Option<(String, String, usize)> {
        Some((rest.to_string(), word.to_string(), n))
    }

    #[test]
    fn complete_needs_a_list_and_the_switch() {
        let cfg = super::default_config_text();
        assert_eq!(at("gerl", "", &cfg), None);
        assert!(!super::word_list_loaded());
        assert_eq!(super::load_word_list(LIST), Ok(3));
        assert!(super::word_list_loaded());
        assert_eq!(at("gerl", "", &cfg), got("эх", "гэрлэх", 4));
        let off = super::config_set_option(&cfg, "completion", false).unwrap();
        assert_eq!(at("gerl", "", &off), None);
        let classic = super::config_set_classic(&cfg, true).unwrap();
        assert_eq!(at("gerl", "", &classic), None);
    }

    #[test]
    fn complete_uses_only_the_word_at_the_caret() {
        let cfg = super::default_config_text();
        super::load_word_list(LIST).unwrap();
        assert_eq!(at("sain gerl", "", &cfg), got("эх", "гэрлэх", 4));
        assert_eq!(at("gerl ", "", &cfg), None);
        assert_eq!(at("", "", &cfg), None);
        assert_eq!(at("ge", "", &cfg), None); // under 3 letters
        assert_eq!(at("zzzz", "", &cfg), None);
        // Text after a non-letter is fine.
        assert_eq!(at("gerl", " baina", &cfg), got("эх", "гэрлэх", 4));
        assert_eq!(at("gerl", "\n", &cfg), got("эх", "гэрлэх", 4));
        assert_eq!(at("gerl", "123", &cfg), got("эх", "гэрлэх", 4));
        assert_eq!(at("gerl", "эх", &cfg), got("эх", "гэрлэх", 4));
    }

    #[test]
    fn no_completion_when_the_caret_is_inside_a_word() {
        let cfg = super::default_config_text();
        super::load_word_list(LIST).unwrap();
        for after in ["x", "xyz", "ö", "'", "\"", "Ü"] {
            assert_eq!(at("gerl", after, &cfg), None, "{after}");
        }
    }

    #[test]
    fn a_latin_run_after_cyrillic_or_a_digit_is_a_new_word() {
        let cfg = super::default_config_text();
        super::load_word_list(LIST).unwrap();
        assert_eq!(at("гэр gerl", "", &cfg), got("эх", "гэрлэх", 4));
        assert_eq!(at("1gerl", "", &cfg), got("эх", "гэрлэх", 4));
        assert_eq!(at("монголgerl", "", &cfg), got("эх", "гэрлэх", 4));
        assert_eq!(at("12", "", &cfg), None);
    }

    #[test]
    fn complete_gives_the_whole_cyrillic_word() {
        let cfg = super::default_config_text();
        super::load_word_list("# t\nулаанбаатар\t50\nулаан\t5\n").unwrap();
        assert_eq!(at("Ulaanb", "", &cfg), got("аатар", "Улаанбаатар", 6));
        // A tie of two close candidates gives no ghost.
        super::load_word_list("гэрлэх\t10\nгэрлэлт\t9\n").unwrap();
        assert_eq!(at("gerl", "", &cfg), None);
    }

    #[test]
    fn accepted_cyrillic_passes_through_the_converter() {
        let cfg = super::default_config_text();
        for (input, expected) in [
            ("гэрлэх baina", "гэрлэх байна"),
            ("sain гэрлэх, Монгол uu", "сайн гэрлэх, Монгол уу"),
            ("гэрлэх", "гэрлэх"),
        ] {
            assert_eq!(super::convert_with_config(input, &cfg).unwrap(), expected);
        }
    }

    #[test]
    fn bad_word_list_gives_a_tagged_error() {
        let e = super::load_word_list("# c\nbad line without tab\n").unwrap_err();
        assert!(e.starts_with("bad_word_list\t2\t"), "{e}");
    }

    #[test]
    fn default_config_is_smart_with_abbreviations() {
        let cfg = super::default_config_text();
        for input in ["unee", "irsen uu", "bn uu ", "aaviin", "github.com"] {
            assert_eq!(
                super::convert_with_config(input, &cfg).unwrap(),
                super::convert_with(input, true, true),
                "{input}"
            );
        }
    }

    #[test]
    fn config_classic_and_overrides_change_output() {
        let classic = super::config_set_classic(&super::default_config_text(), true).unwrap();
        assert_eq!(
            super::convert_with_config("unee", &classic).unwrap(),
            "унээ"
        );
        assert_eq!(super::convert_with_config("bn ", &classic).unwrap(), "бн ");
        let t = super::config_set_option(&super::default_config_text(), "abbreviations", false)
            .unwrap();
        assert!(t.contains("abbreviations = false"));
        assert_eq!(super::convert_with_config("bn ", &t).unwrap(), "бн ");
        // Back to the preset value: the override goes away.
        let t = super::config_set_option(&t, "abbreviations", true).unwrap();
        assert_eq!(t, super::default_config_text());
        assert!(super::config_set_option(&t, "nope", true).is_err());
    }

    #[test]
    fn normalize_gives_canonical_text_and_warnings() {
        let r = super::config_normalize(
            "# mine\nbuuz_config = 1\n[options]\nsmart_vowels = false\nfuture = true\n",
        )
        .unwrap();
        assert_eq!(
            r.text,
            "buuz_config = 1\npreset = \"smart\"\n\n[options]\nsmart_vowels = false\n"
        );
        assert_eq!(r.warnings, vec!["unknown_option\t5\tfuture".to_string()]);
        assert!(!r.classic && !r.smart_vowels && r.abbreviations);
        assert_eq!(super::config_normalize(&r.text).unwrap().text, r.text);
    }

    #[test]
    fn two_configs_alternate_with_the_right_output() {
        let a = super::default_config_text();
        let b = super::config_set_classic(&a, true).unwrap();
        for _ in 0..3 {
            assert_eq!(super::convert_with_config("unee", &a).unwrap(), "үнээ");
            assert_eq!(super::convert_with_config("unee", &b).unwrap(), "унээ");
        }
        let c = super::config_set_option(&a, "smart_vowels", false).unwrap();
        assert_eq!(super::convert_with_config("unee", &c).unwrap(), "унээ");
        assert_eq!(super::convert_with_config("unee", &a).unwrap(), "үнээ");
    }

    #[test]
    fn too_large_tags() {
        let big = "#".repeat(1024 * 1024 + 1);
        assert_eq!(
            super::config_normalize(&big).err().unwrap(),
            "too_large\t0\t1 MiB"
        );
        let mut many = String::from("buuz_config = 1\n[abbreviations]\n");
        for i in 0..10_001 {
            let k: String = [i / 676, i / 26 % 26, i % 26]
                .iter()
                .map(|d| (b'a' + *d as u8) as char)
                .collect();
            many.push_str(&format!("{k} = \"x\"\n"));
        }
        assert!(super::config_normalize(&many)
            .err()
            .unwrap()
            .starts_with("too_many_entries\t"));
    }

    #[test]
    fn bad_config_gives_tagged_errors() {
        assert_eq!(
            super::config_normalize("hello").err().unwrap(),
            "missing_version\t1\t"
        );
        assert_eq!(
            super::config_normalize("buuz_config = 9").err().unwrap(),
            "too_new\t9\t1"
        );
        assert!(super::convert_with_config("x", "buuz_config = 1\nzzz = 1").is_err());
        assert!(super::config_set_classic("", true).is_err());
    }

    #[test]
    fn matches_core() {
        assert_eq!(super::convert("sain baina uu"), "сайн байна уу");
    }

    #[test]
    fn links_and_emails_stay_latin_except_in_classic() {
        assert_eq!(super::convert("github.com"), "github.com");
        assert_eq!(super::convert("bat@gmail.com"), "bat@gmail.com");
        assert_ne!(super::convert_classic("github.com"), "github.com");
    }

    #[test]
    fn convert_with_switches_smart_vowels() {
        assert_eq!(super::convert_with("unee", true, false), "үнээ");
        assert_eq!(super::convert_with("unee", false, false), "унээ");
        assert_eq!(super::convert_with("irsen uu", true, false), "ирсэн үү");
        assert_eq!(super::convert_with("irsen uu", false, false), "ирсэн уу");
    }

    #[test]
    fn convert_classic_is_the_original_buuz() {
        assert_eq!(super::convert_classic("unee"), "унээ");
        assert_eq!(super::convert_classic("irsen uu"), "ирсэн уу");
        assert_eq!(super::convert_classic("bn uu "), "бн уу ");
        assert_eq!(super::convert_classic("sain baina"), "сайн байна");
    }

    #[test]
    fn convert_with_switches_abbreviations() {
        assert_eq!(super::convert_with("bn uu", true, true), "байна уу");
        // Only a typed Space or punctuation expands; the end of the text does not.
        assert_eq!(super::convert_with("bn ", true, true), "байна ");
        assert_eq!(super::convert_with("bn.", true, true), "байна.");
        assert_eq!(super::convert_with("bn", true, true), "бн");
        assert_eq!(super::convert_with("bn ", true, false), "бн ");
        assert!(super::abbreviation_list().contains("bn = байна\n"));
    }

    /// Every typewriter phrase on the page must convert exactly as written.
    #[test]
    fn site_phrases_convert_as_written() {
        let json = include_str!("../../site/src/phrases.json");
        // phrases.json is a flat list of string pairs; read its strings (with \" and \\).
        let mut strings = Vec::new();
        let mut cur: Option<String> = None;
        let mut chars = json.chars();
        while let Some(c) = chars.next() {
            match (&mut cur, c) {
                (None, '"') => cur = Some(String::new()),
                (None, _) => {}
                (Some(_), '"') => strings.push(cur.take().unwrap()),
                (Some(s), '\\') => s.extend(chars.next()),
                (Some(s), c) => s.push(c),
            }
        }
        assert!(strings.len() >= 2 && strings.len() % 2 == 0);
        for pair in strings.chunks(2) {
            assert_eq!(super::convert(&pair[0]), pair[1], "phrase {:?}", pair[0]);
        }
    }

    // "v" for ү, typed in a sentence with ү in it. "v" gives в by default.
    const V_PAIRS: &str = r#"[["гүн","gvn"],["үнэ","vne"]]"#;

    #[test]
    fn calibrate_json_reports_suggestions() {
        let j = super::calibrate_json(V_PAIRS).unwrap();
        assert!(
            j.starts_with("{\"suggestions\":[{\"letter\":\"ү\",\"chunk\":\"v\","),
            "{j}"
        );
        assert!(j.contains("\"displaces\":\"в\""), "{j}");
        assert!(j.contains("\"target\":\"ү\""), "{j}");
        assert!(j.contains("\"unmatched\":[]"), "{j}");
        assert!(j.contains("\"skipped_letters\":["), "{j}");
        // Default typing: no suggestions.
        let j = super::calibrate_json(r#"[["гэр","ger"]]"#).unwrap();
        assert!(j.starts_with("{\"suggestions\":[],"), "{j}");
    }

    #[test]
    fn calibrate_json_reads_escapes_and_reports_unmatched() {
        // \u escapes, a surrogate pair and a quote in the typed text parse.
        let j = super::calibrate_json(r#"[["\u0433\u044d\u0440 \"\ud83d\ude00\"","ger \"x\""]]"#)
            .unwrap();
        assert!(j.starts_with("{\"suggestions\":[],"), "{j}");
        // A sentence that does not match is listed by index (from 0).
        let j =
            super::calibrate_json(r#"[["гэр","ger"],["сайн","qqqqqqqqqqqqqqqqqqqq"]]"#).unwrap();
        assert!(j.contains("\"unmatched\":[1]"), "{j}");
    }

    #[test]
    fn calibrate_json_rejects_bad_input() {
        for bad in [
            "",
            "{}",
            "[1]",
            "[[\"a\"]]",
            "[[\"a\",\"b\"]",
            "[[\"a\",\"b\"]] x",
            "[[\"\\x\",\"b\"]]",
        ] {
            assert_eq!(
                super::calibrate_json(bad).err().unwrap(),
                "bad_pairs\t0\t",
                "{bad}"
            );
        }
        assert!(super::calibrate_json("[]")
            .unwrap()
            .contains("\"suggestions\":[]"));
    }

    #[test]
    fn calibrate_apply_takes_only_accepted_suggestions() {
        let cfg = super::default_config_text();
        let all = super::calibrate_apply(&cfg, V_PAIRS, "[0]").unwrap();
        assert_eq!(super::convert_with_config("v", &all).unwrap(), "ү");
        assert_eq!(super::convert_with_config("v", &cfg).unwrap(), "в");
        // Nothing accepted: the config is unchanged.
        assert_eq!(super::calibrate_apply(&cfg, V_PAIRS, "[]").unwrap(), cfg);
        // Two suggestions, only the second accepted.
        let pairs = r#"[["гүн","gvn"],["үнэ","vne"],["цай","hai"],["цай","hai"]]"#;
        let j = super::calibrate_json(pairs).unwrap();
        let n = j.matches("\"chunk\"").count();
        assert!(n >= 2, "{j}");
        let one = super::calibrate_apply(&cfg, pairs, "[0]").unwrap();
        let two = super::calibrate_apply(&cfg, pairs, "[1]").unwrap();
        assert_eq!(super::convert_with_config("h", &two).unwrap(), "ц");
        assert_eq!(super::convert_with_config("h", &one).unwrap(), "х");
        assert_eq!(super::convert_with_config("v", &one).unwrap(), "ү");
        assert_ne!(one, two);
        assert_ne!(one, cfg);
    }

    #[test]
    fn calibrate_apply_rejects_bad_input() {
        let cfg = super::default_config_text();
        assert_eq!(
            super::calibrate_apply(&cfg, V_PAIRS, "[5]").err().unwrap(),
            "bad_indexes\t0\t"
        );
        assert_eq!(
            super::calibrate_apply(&cfg, V_PAIRS, "nope").err().unwrap(),
            "bad_indexes\t0\t"
        );
        assert_eq!(
            super::calibrate_apply(&cfg, "x", "[]").err().unwrap(),
            "bad_pairs\t0\t"
        );
        assert!(super::calibrate_apply("junk", V_PAIRS, "[]").is_err());
    }

    #[test]
    fn json_escapes_decode_to_the_same_text() {
        let plain = super::calibrate_json(r#"[["гүн","gvn"],["гэр 😀","ger 😀"]]"#).unwrap();
        // \u escapes (lower and upper case hex), a surrogate pair, \/ and \"
        let esc = super::calibrate_json(r#"[["гүн","gvn"],["гэр 😀","ger 😀"]]"#).unwrap();
        assert_eq!(plain, esc);
        let slash = super::calibrate_json(r#"[["гэр","ger\/"],["гэр","ger"]]"#).unwrap();
        assert!(slash.contains("\"suggestions\":[]"), "{slash}");
    }

    #[test]
    fn json_rejects_bad_strings() {
        for bad in [
            r#"[["\ud83d","a"]]"#,  // lone high surrogate
            r#"[["\ude00","a"]]"#,  // lone low surrogate
            r#"[["\ud83dA","a"]]"#, // high surrogate, then not a low one
            r#"[["\u00zz","a"]]"#,  // bad hex
            r#"[["\u00","a"]]"#,    // short hex
            "[[\"a\nb\",\"a\"]]",   // raw control character
        ] {
            assert_eq!(
                super::calibrate_json(bad).err().unwrap(),
                "bad_pairs\t0\t",
                "{bad}"
            );
        }
    }

    #[test]
    fn json_input_is_bounded() {
        let pair = r#"["а","a"]"#;
        let many = |n: usize| format!("[{}]", vec![pair; n].join(","));
        assert!(super::calibrate_json(&many(50)).is_ok());
        assert_eq!(
            super::calibrate_json(&many(51)).err().unwrap(),
            "bad_pairs\t0\t"
        );
        let long = "а".repeat(501);
        let one = format!("[[\"{long}\",\"a\"]]");
        assert_eq!(super::calibrate_json(&one).err().unwrap(), "bad_pairs\t0\t");
        let ok = "а".repeat(500);
        assert!(super::calibrate_json(&format!("[[\"{ok}\",\"a\"]]")).is_ok());
        let big = format!("[[\"а\",\"a\"]{}]", " ".repeat(64 * 1024));
        assert_eq!(super::calibrate_json(&big).err().unwrap(), "bad_pairs\t0\t");
        let cfg = super::default_config_text();
        assert_eq!(
            super::calibrate_apply(&cfg, &big, "[]").err().unwrap(),
            "bad_pairs\t0\t"
        );
        let bigi = format!("[0{}]", " ".repeat(64 * 1024));
        assert_eq!(
            super::calibrate_apply(&cfg, V_PAIRS, &bigi).err().unwrap(),
            "bad_indexes\t0\t"
        );
    }

    #[test]
    fn index_list_is_bounded_and_has_no_duplicates() {
        let cfg = super::default_config_text();
        for bad in [
            "[0,0]",
            "[99999999999999999999999999]",
            &format!("[{}]", vec!["0"; 65].join(",")),
            "[-1]",
            "[1.5]",
        ] {
            assert_eq!(
                super::calibrate_apply(&cfg, V_PAIRS, bad).err().unwrap(),
                "bad_indexes\t0\t",
                "{bad}"
            );
        }
        let many: Vec<String> = (0..64).map(|i| i.to_string()).collect();
        // 64 entries are read; they are then out of range here, still bad_indexes.
        assert_eq!(
            super::calibrate_apply(&cfg, V_PAIRS, &format!("[{}]", many.join(",")))
                .err()
                .unwrap(),
            "bad_indexes\t0\t"
        );
    }
}
