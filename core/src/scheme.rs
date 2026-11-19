// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Latin to Cyrillic schemes for languages other than Mongolian.
//!
//! A [`Scheme`] is a table of Latin patterns and Cyrillic output. It is
//! matched longest first, one word at a time, with no look-ahead and no word
//! list. Mongolian does not use this module: it has its own composer
//! ([`crate::Composer`]) with vowel harmony.
//!
//! The tables follow how people type each language in Latin letters. They
//! are not the official romanisation standards, except `ru-gost`, which is
//! GOST 7.79 system B read backwards. See `docs/research/cyrillic-schemes.md`.

use std::fmt;

/// When a rule may match.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cond {
    /// Always.
    Any,
    /// The last output letter is a vowel of the language.
    AfterVowel,
    /// At the start of a word.
    WordStart,
    /// The last output character is a letter.
    AfterLetter,
    /// The last output character is a letter, or a letter follows.
    /// Digits use it, so a number (`2026`) stays a number.
    TouchesLetter,
}

/// One row: a lower-case Latin pattern, its Cyrillic output (lower case) and
/// the condition. The first rule wins among rules of the same length.
type Rule = (&'static str, &'static str, Cond);

use Cond::{AfterLetter, AfterVowel, Any, TouchesLetter, WordStart};

const RU_VOWELS: &str = "аеёиоуыэюя";
const UK_VOWELS: &str = "аеєиіїоуюя";
const BE_VOWELS: &str = "аеёіоуыэюяў";
const BG_VOWELS: &str = "аъеиоуюя";
const KK_VOWELS: &str = "аәеёиіоөұүыэюя";
const KY_VOWELS: &str = "аеёиоөуүыэюя";

/// Everyday Russian translit: the habit of most Russian speakers.
#[rustfmt::skip]
const RU: &[Rule] = &[
    ("shch", "щ", Any), ("sch", "щ", Any), ("shh", "щ", Any),
    ("zh", "ж", Any), ("kh", "х", Any), ("ts", "ц", Any), ("ch", "ч", Any), ("sh", "ш", Any),
    ("yo", "ё", Any), ("jo", "ё", Any), ("yu", "ю", Any), ("ju", "ю", Any),
    ("ya", "я", Any), ("ja", "я", Any), ("ye", "е", Any), ("je", "е", Any),
    // `et` at the start of a word is это, этот, эти. `e'` is э anywhere.
    ("et", "эт", WordStart), ("e'", "э", Any), ("e`", "э", Any), ("e’", "э", Any),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("g", "г", Any), ("d", "д", Any),
    ("e", "е", Any), ("z", "з", Any), ("i", "и", Any), ("j", "й", Any), ("k", "к", Any),
    ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any), ("p", "п", Any),
    ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any), ("f", "ф", Any),
    ("h", "х", Any), ("c", "ц", Any), ("w", "в", Any), ("x", "кс", Any),
    // `y` after a vowel is й (ай, ой), after a consonant ы.
    ("y", "й", AfterVowel), ("y", "ы", Any),
    ("''", "ъ", AfterLetter), ("'", "ь", AfterLetter),
    ("`", "ь", AfterLetter), ("’", "ь", AfterLetter),
];

/// GOST 7.79 system B (ASCII), read backwards. Only the letters of system B.
#[rustfmt::skip]
const RU_GOST: &[Rule] = &[
    ("shh", "щ", Any), ("sh", "ш", Any), ("ch", "ч", Any), ("zh", "ж", Any),
    ("cz", "ц", Any), ("yo", "ё", Any), ("yu", "ю", Any), ("ya", "я", Any),
    ("y`", "ы", Any), ("e`", "э", Any), ("``", "ъ", AfterLetter), ("`", "ь", AfterLetter),
    ("y'", "ы", Any), ("e'", "э", Any), ("''", "ъ", AfterLetter), ("'", "ь", AfterLetter),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("g", "г", Any), ("d", "д", Any),
    ("e", "е", Any), ("z", "з", Any), ("i", "и", Any), ("j", "й", Any), ("k", "к", Any),
    ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any), ("p", "п", Any),
    ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any), ("f", "ф", Any),
    ("x", "х", Any), ("c", "ц", Any), ("y", "ы", Any),
];

/// Russian "Volapük": Latin letters and digits that look like Cyrillic
/// letters (`npuBeT` is привет, `4to` is что). A digit gives a letter only
/// next to a letter. Letters with no look-alike use their sound.
#[rustfmt::skip]
const RU_VOLAPUK: &[Rule] = &[
    ("lli", "щ", Any), ("iii", "ш", Any), (">|<", "ж", Any), ("}|{", "ж", Any),
    ("bl", "ы", Any), ("10", "ю", TouchesLetter),
    ("e'", "э", Any), ("€", "э", Any), ("ë", "ё", Any),
    ("a", "а", Any), ("b", "в", Any), ("c", "с", Any), ("d", "д", Any), ("e", "е", Any),
    ("f", "ф", Any), ("g", "г", Any), ("h", "н", Any), ("i", "и", Any), ("j", "й", Any),
    ("k", "к", Any), ("l", "л", Any), ("m", "м", Any), ("n", "п", Any), ("o", "о", Any),
    ("p", "р", Any), ("r", "я", Any), ("s", "с", Any), ("t", "т", Any), ("u", "и", Any),
    ("v", "в", Any), ("w", "ш", Any), ("x", "х", Any), ("y", "у", Any), ("z", "з", Any),
    ("3", "з", TouchesLetter), ("4", "ч", TouchesLetter), ("6", "б", TouchesLetter),
    ("9", "я", TouchesLetter), ("0", "о", TouchesLetter),
    ("''", "ъ", AfterLetter), ("'", "ь", AfterLetter),
];

/// Everyday Ukrainian translit. `h` is г and `g` is ґ, as in the 2010
/// national romanisation. `i` followed by a vowel stays two letters (діалог).
#[rustfmt::skip]
const UK: &[Rule] = &[
    ("shch", "щ", Any), ("sch", "щ", Any),
    ("zh", "ж", Any), ("kh", "х", Any), ("ts", "ц", Any), ("ch", "ч", Any), ("sh", "ш", Any),
    ("yi", "ї", Any), ("ji", "ї", Any), ("ye", "є", Any), ("je", "є", Any),
    ("yu", "ю", Any), ("ju", "ю", Any), ("ya", "я", Any), ("ja", "я", Any),
    ("yo", "йо", Any), ("jo", "йо", Any),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("h", "г", Any), ("g", "ґ", Any),
    ("d", "д", Any), ("e", "е", Any), ("z", "з", Any), ("i", "і", Any), ("j", "й", Any),
    ("k", "к", Any), ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any),
    ("p", "п", Any), ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any),
    ("f", "ф", Any), ("c", "ц", Any), ("w", "в", Any), ("x", "кс", Any),
    ("y", "й", AfterVowel), ("y", "и", Any),
    // `''` is the apostrophe (U+02BC), `'` the soft sign.
    ("''", "ʼ", AfterLetter), ("'", "ь", AfterLetter),
    ("`", "ь", AfterLetter), ("’", "ь", AfterLetter),
];

/// Belarusian translit. No и: `i` is і and `y` is ы. `w` and `u'` give ў.
#[rustfmt::skip]
const BE: &[Rule] = &[
    ("shch", "шч", Any), ("sch", "шч", Any),
    ("zh", "ж", Any), ("kh", "х", Any), ("dz", "дз", Any), ("ts", "ц", Any),
    ("ch", "ч", Any), ("sh", "ш", Any),
    ("yo", "ё", Any), ("jo", "ё", Any), ("yu", "ю", Any), ("ju", "ю", Any),
    ("ya", "я", Any), ("ja", "я", Any), ("ye", "е", Any), ("je", "е", Any),
    ("e'", "э", Any), ("e`", "э", Any), ("u'", "ў", Any),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("h", "г", Any), ("g", "г", Any),
    ("d", "д", Any), ("e", "е", Any), ("z", "з", Any), ("i", "і", Any), ("j", "й", Any),
    ("k", "к", Any), ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any),
    ("p", "п", Any), ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any),
    ("f", "ф", Any), ("c", "ц", Any), ("w", "ў", Any), ("x", "х", Any),
    ("y", "й", AfterVowel), ("y", "ы", Any),
    ("''", "ʼ", AfterLetter), ("'", "ь", AfterLetter),
    ("`", "ь", AfterLetter), ("’", "ь", AfterLetter),
];

/// Bulgarian translit. `y` is й, `q` is я, `sht` is щ and `u'` is ъ.
#[rustfmt::skip]
const BG: &[Rule] = &[
    ("sht", "щ", Any), ("zh", "ж", Any), ("ts", "ц", Any), ("ch", "ч", Any), ("sh", "ш", Any),
    ("yu", "ю", Any), ("ju", "ю", Any), ("ya", "я", Any), ("ja", "я", Any),
    ("u'", "ъ", Any), ("u`", "ъ", Any), ("ŭ", "ъ", Any),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("g", "г", Any), ("d", "д", Any),
    ("e", "е", Any), ("z", "з", Any), ("i", "и", Any), ("y", "й", Any), ("j", "й", Any),
    ("k", "к", Any), ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any),
    ("p", "п", Any), ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any),
    ("f", "ф", Any), ("h", "х", Any), ("c", "ц", Any), ("q", "я", Any), ("w", "в", Any),
    ("x", "кс", Any),
    ("'", "ь", AfterLetter), ("`", "ь", AfterLetter), ("’", "ь", AfterLetter),
];

/// Serbian Cyrillic from the Latin alphabet (Gaj), with ASCII digraphs.
#[rustfmt::skip]
const SR: &[Rule] = &[
    ("dzh", "џ", Any), ("dž", "џ", Any),
    ("lj", "љ", Any), ("nj", "њ", Any), ("dj", "ђ", Any), ("đ", "ђ", Any),
    ("ch", "ч", Any), ("č", "ч", Any), ("c'", "ћ", Any), ("cc", "ћ", Any), ("ć", "ћ", Any),
    ("sh", "ш", Any), ("š", "ш", Any), ("zh", "ж", Any), ("ž", "ж", Any),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("g", "г", Any), ("d", "д", Any),
    ("e", "е", Any), ("z", "з", Any), ("i", "и", Any), ("j", "ј", Any), ("k", "к", Any),
    ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any), ("p", "п", Any),
    ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any), ("f", "ф", Any),
    ("h", "х", Any), ("c", "ц", Any),
];

/// Macedonian Cyrillic from the Latin alphabet: Serbian plus ѓ ќ ѕ.
#[rustfmt::skip]
const MK: &[Rule] = &[
    ("dzh", "џ", Any), ("dž", "џ", Any),
    ("gj", "ѓ", Any), ("kj", "ќ", Any), ("dz", "ѕ", Any),
    ("lj", "љ", Any), ("nj", "њ", Any),
    ("ch", "ч", Any), ("č", "ч", Any), ("sh", "ш", Any), ("š", "ш", Any),
    ("zh", "ж", Any), ("ž", "ж", Any),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("g", "г", Any), ("d", "д", Any),
    ("e", "е", Any), ("z", "з", Any), ("i", "и", Any), ("j", "ј", Any), ("k", "к", Any),
    ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any), ("p", "п", Any),
    ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any), ("f", "ф", Any),
    ("h", "х", Any), ("c", "ц", Any),
];

/// Kazakh translit (draft). Marks: `ae` ә, `gh` ғ, `q` қ, `ng` ң, `oe` ө,
/// `u'` ұ, `ue` ү, `h'` һ. `i` is і, `iy` is и and `y` is ы.
#[rustfmt::skip]
const KK: &[Rule] = &[
    ("shch", "щ", Any), ("sch", "щ", Any),
    ("ae", "ә", Any), ("a'", "ә", Any), ("ä", "ә", Any),
    ("gh", "ғ", Any), ("g'", "ғ", Any), ("ğ", "ғ", Any),
    ("ng", "ң", Any), ("n'", "ң", Any), ("ñ", "ң", Any),
    ("oe", "ө", Any), ("o'", "ө", Any), ("ö", "ө", Any),
    ("ue", "ү", Any), ("ü", "ү", Any), ("u'", "ұ", Any), ("ū", "ұ", Any),
    ("h'", "һ", Any), ("iy", "и", Any),
    ("zh", "ж", Any), ("kh", "х", Any), ("ts", "ц", Any), ("ch", "ч", Any), ("sh", "ш", Any),
    ("yo", "ё", Any), ("yu", "ю", Any), ("ya", "я", Any), ("ye", "е", Any),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("g", "г", Any), ("d", "д", Any),
    ("e", "е", Any), ("z", "з", Any), ("i", "і", Any), ("j", "ж", Any), ("k", "к", Any),
    ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any), ("p", "п", Any),
    ("q", "қ", Any), ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any),
    ("f", "ф", Any), ("h", "х", Any), ("c", "ц", Any), ("w", "в", Any), ("x", "кс", Any),
    ("y", "й", AfterVowel), ("y", "ы", Any),
    ("''", "ъ", AfterLetter), ("'", "ь", AfterLetter),
    ("`", "ь", AfterLetter), ("’", "ь", AfterLetter),
];

/// Kyrgyz translit (draft). Marks: `ng` ң, `oe` ө, `ue` ү (also `u'`).
#[rustfmt::skip]
const KY: &[Rule] = &[
    ("shch", "щ", Any), ("sch", "щ", Any),
    ("ng", "ң", Any), ("n'", "ң", Any), ("ñ", "ң", Any),
    ("oe", "ө", Any), ("o'", "ө", Any), ("ö", "ө", Any),
    ("ue", "ү", Any), ("u'", "ү", Any), ("ü", "ү", Any),
    ("zh", "ж", Any), ("kh", "х", Any), ("ts", "ц", Any), ("ch", "ч", Any), ("sh", "ш", Any),
    ("yo", "ё", Any), ("yu", "ю", Any), ("ya", "я", Any), ("ye", "е", Any),
    ("a", "а", Any), ("b", "б", Any), ("v", "в", Any), ("g", "г", Any), ("d", "д", Any),
    ("e", "е", Any), ("z", "з", Any), ("i", "и", Any), ("j", "ж", Any), ("k", "к", Any),
    ("l", "л", Any), ("m", "м", Any), ("n", "н", Any), ("o", "о", Any), ("p", "п", Any),
    ("q", "к", Any), ("r", "р", Any), ("s", "с", Any), ("t", "т", Any), ("u", "у", Any),
    ("f", "ф", Any), ("h", "х", Any), ("c", "ц", Any), ("w", "в", Any), ("x", "кс", Any),
    ("y", "й", AfterVowel), ("y", "ы", Any),
    ("''", "ъ", AfterLetter), ("'", "ь", AfterLetter),
    ("`", "ь", AfterLetter), ("’", "ь", AfterLetter),
];

/// A Latin to Cyrillic scheme for a language other than Mongolian.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scheme {
    /// Russian, everyday translit (`privet` gives `привет`).
    Russian,
    /// Russian, GOST 7.79 system B read backwards.
    RussianGost,
    /// Russian, look-alike letters and digits (`4to` gives `что`).
    RussianVolapuk,
    /// Ukrainian, everyday translit.
    Ukrainian,
    /// Belarusian, everyday translit.
    Belarusian,
    /// Bulgarian, everyday translit.
    Bulgarian,
    /// Serbian Cyrillic from the Latin alphabet.
    Serbian,
    /// Macedonian Cyrillic from the Latin alphabet.
    Macedonian,
    /// Kazakh, draft translit.
    Kazakh,
    /// Kyrgyz, draft translit.
    Kyrgyz,
}

/// The id of a scheme is not known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownScheme(pub String);

impl fmt::Display for UnknownScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown scheme `{}`", self.0)
    }
}

impl std::error::Error for UnknownScheme {}

impl Scheme {
    /// Every scheme, in the order a settings list shows them.
    pub const ALL: [Scheme; 10] = [
        Scheme::Russian,
        Scheme::RussianGost,
        Scheme::RussianVolapuk,
        Scheme::Ukrainian,
        Scheme::Belarusian,
        Scheme::Bulgarian,
        Scheme::Serbian,
        Scheme::Macedonian,
        Scheme::Kazakh,
        Scheme::Kyrgyz,
    ];

    /// The short id used in settings and in the portable config: `ru`,
    /// `ru-gost`, `ru-volapuk`, `uk`, `be`, `bg`, `sr`, `mk`, `kk`, `ky`.
    pub fn id(self) -> &'static str {
        match self {
            Scheme::Russian => "ru",
            Scheme::RussianGost => "ru-gost",
            Scheme::RussianVolapuk => "ru-volapuk",
            Scheme::Ukrainian => "uk",
            Scheme::Belarusian => "be",
            Scheme::Bulgarian => "bg",
            Scheme::Serbian => "sr",
            Scheme::Macedonian => "mk",
            Scheme::Kazakh => "kk",
            Scheme::Kyrgyz => "ky",
        }
    }

    /// The scheme with this id (see [`Scheme::id`]).
    pub fn from_id(id: &str) -> Result<Scheme, UnknownScheme> {
        Scheme::ALL
            .into_iter()
            .find(|s| s.id() == id)
            .ok_or_else(|| UnknownScheme(id.to_string()))
    }

    /// The ISO 639-1 code of the language: `ru`, `uk`, `be`, ...
    pub fn language(self) -> &'static str {
        match self {
            Scheme::RussianGost | Scheme::RussianVolapuk => "ru",
            other => other.id(),
        }
    }

    /// The scheme is a draft: its habits are not checked by a native speaker.
    pub fn is_draft(self) -> bool {
        !matches!(self, Scheme::Russian | Scheme::RussianGost)
    }

    /// The scheme a language uses by default, for a page in that language.
    pub fn for_language(language: &str) -> Option<Scheme> {
        Scheme::ALL.into_iter().find(|s| s.id() == language)
    }

    fn table(self) -> (&'static [Rule], &'static str) {
        match self {
            Scheme::Russian => (RU, RU_VOWELS),
            Scheme::RussianGost => (RU_GOST, RU_VOWELS),
            Scheme::RussianVolapuk => (RU_VOLAPUK, ""),
            Scheme::Ukrainian => (UK, UK_VOWELS),
            Scheme::Belarusian => (BE, BE_VOWELS),
            Scheme::Bulgarian => (BG, BG_VOWELS),
            Scheme::Serbian => (SR, ""),
            Scheme::Macedonian => (MK, ""),
            Scheme::Kazakh => (KK, KK_VOWELS),
            Scheme::Kyrgyz => (KY, KY_VOWELS),
        }
    }

    /// Convert Latin text to Cyrillic. Characters without a rule (digits,
    /// punctuation, Cyrillic, other letters) stay as they are.
    ///
    /// ```
    /// use buuz_core::Scheme;
    /// assert_eq!(Scheme::Russian.convert("Privet, mir!"), "Привет, мир!");
    /// ```
    pub fn convert(self, input: &str) -> String {
        let (rules, vowels) = self.table();
        let mut chars: Vec<char> = input.chars().collect();
        if self == Scheme::RussianVolapuk {
            // A capital inside a word is a look-alike (the B of npuBeT), not a
            // capital letter. Only the first letter of a word keeps its case.
            for i in 1..chars.len() {
                if chars[i - 1].is_alphabetic() {
                    chars[i] = lower(chars[i]);
                }
            }
        }
        let lower: Vec<char> = chars.iter().map(|c| lower(*c)).collect();
        let mut out = String::with_capacity(input.len());
        let mut last: Option<char> = None; // last output char, lower case
        let mut i = 0;
        while i < chars.len() {
            let word_start = i == 0 || !(chars[i - 1].is_alphabetic() || is_mark(chars[i - 1]));
            let mut best: Option<(usize, &str)> = None;
            for (pat, to, cond) in rules {
                let n = pat.chars().count();
                if best.is_some_and(|(len, _)| len >= n) || i + n > chars.len() {
                    continue;
                }
                if !pat.chars().zip(&lower[i..]).all(|(p, c)| p == *c) {
                    continue;
                }
                let ok = match cond {
                    Cond::Any => true,
                    Cond::AfterVowel => last.is_some_and(|c| vowels.contains(c)),
                    Cond::WordStart => word_start,
                    Cond::AfterLetter => last.is_some_and(char::is_alphabetic),
                    Cond::TouchesLetter => {
                        last.is_some_and(char::is_alphabetic)
                            || chars.get(i + n).is_some_and(|c| c.is_alphabetic())
                    }
                };
                if ok {
                    best = Some((n, to));
                }
            }
            let Some((n, to)) = best else {
                out.push(chars[i]);
                last = Some(lower[i]);
                i += 1;
                continue;
            };
            push_cased(&mut out, to, &chars[i..i + n], i, &chars);
            last = to.chars().last();
            i += n;
        }
        out
    }
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn is_mark(c: char) -> bool {
    matches!(c, '\'' | '`' | '’')
}

/// Write `to` with the case of the Latin `src` that gave it. An all-capital
/// pattern of two or more letters, or one next to another capital, gives
/// capitals. A capital first letter gives a capital first letter.
fn push_cased(out: &mut String, to: &str, src: &[char], at: usize, all: &[char]) {
    let letters: Vec<char> = src.iter().copied().filter(|c| c.is_alphabetic()).collect();
    let upper = |c: &char| c.is_uppercase();
    let any_lower = letters.iter().any(|c| c.is_lowercase());
    let first_upper = letters.first().is_some_and(upper);
    let near_upper = (at > 0 && all[at - 1].is_uppercase())
        || all.get(at + src.len()).is_some_and(|c| c.is_uppercase());
    if letters.is_empty() || (!first_upper && !letters.iter().any(upper)) {
        out.push_str(to);
    } else if !any_lower && (letters.len() > 1 || near_upper) {
        out.extend(to.chars().flat_map(char::to_uppercase));
    } else if first_upper {
        let mut it = to.chars();
        if let Some(c) = it.next() {
            out.extend(c.to_uppercase());
        }
        out.extend(it);
    } else {
        out.push_str(to);
    }
}
