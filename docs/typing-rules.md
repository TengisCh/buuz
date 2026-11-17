# How Buuz turns Latin into Cyrillic

Status: describes `buuz-core` with smart vowel harmony (issue #7), which is
on by default; the original left-to-right behaviour is kept as the "classic"
option. Buuz is a port of the original by Odbayar Nyamtseren. Planned changes
are linked at the end and are **not** implemented yet.

Source: [`core/src/rules.rs`](../core/src/rules.rs) (the table) and
[`core/src/composer.rs`](../core/src/composer.rs) (the engine).

## 1. The big picture

1. While you type a word, Buuz keeps the **raw Latin keys** in a buffer and
   shows the converted word as underlined preedit text.
2. On **every key**, the whole word is converted again from the raw keys.
   This is why a later key can still change earlier letters (for example
   `'` after `o` turns о into ө).
3. A space, digit, punctuation mark, Enter, Tab or any other non-letter key
   **commits** the word and is then passed to the app unchanged.

Word characters are the ASCII letters `a`–`z`, `A`–`Z` and the two sign keys
`'` and `"`, plus the umlauts `ö`, `ü`, `Ö`, `Ü` (see below). Anything else
(including Cyrillic letters) ends the word. A word can hold at most 50 keys.

## 2. Matching: longest rule first

The engine reads the raw keys left to right. At each position it tries the
**longest** sequence in the table first (3 keys, then 2, then 1). So:

- `sh` → ш, not с + х
- `ch` → ч, `kh` → х
- `iii` → ы, `ii'` → ий, `ii` → ий or ы (see section 4), `i` → и
- `sxc` → щ

A key that matches nothing is copied as is.

## 3. The table

### Letters that never depend on the word

| Keys | Cyrillic | Keys | Cyrillic | Keys | Cyrillic |
| --- | --- | --- | --- | --- | --- |
| `a` | а | `j` | ж | `s` | с |
| `b` | б | `k` | к | `sh` | ш |
| `c` | ц | `kh` | х | `sxc` | щ |
| `ch` | ч | `l` | л | `t` | т |
| `d` | д | `m` | м | `v` | в |
| `e` | э | `n` | н | `w` | ү |
| `f` | ф | `p` | п | `x` | х |
| `g` | г | `q` | ө | `y` | ы |
| `h` | х | `r` | р | `z` | з |
| `i` | и | `ii` | ий or ы (smart, section 4) | `iii` | ы |
| `ya` | я | `ye` | е | `yo` | ё |
| `yu` | ю | `ii'` | ий | | |

A vowel followed by `i` becomes vowel + й: `ai` ай, `ei` эй, `oi` ой/өй,
`ui` уй/үй, `qi` өй, `wi` үй.

### Vowels that follow vowel harmony

| Keys | Masculine word | Feminine word |
| --- | --- | --- |
| `o` | о | ө |
| `u` | у | ү |

To force one form regardless of the word:

| Keys | Result |
| --- | --- |
| `q`, `ö`, `o'`, `'o` | ө |
| `w`, `ü`, `u'`, `'u` | ү |
| `o"`, `"o` | о |
| `u"`, `"u` | у |

### Signs

| Keys | Result |
| --- | --- |
| `'` | ь |
| `''` | Ь |
| `"` | ъ |
| `""` | Ъ |

## 4. Vowel harmony

Every vowel rule carries flags: which kind of word it may appear in, and
whether it decides the word's gender.

- **Masculine:** `a`, `ya`, `yo`, `yu`, `o"`, `u"`.
- **Feminine:** `e`, `q`, `w`, `ö`, `ü`, `o'`, `u'` (and `ye`, see below).
- **Neutral:** `i`, `ii`, `iii`, `y`, all consonants and signs. Plain `o` and
  `u` do not decide anything; they follow the word.

"Smart vowels" is on by default. It can be switched off (the "Smart vowels"
checkbox on the website, `Options::classic()` in the core) to get the
original behaviour, described at the end of this section.

### 4.1 Look ahead for the gender

The whole word is converted again on every key, so the core first scans the
word for its **first deciding vowel** and starts in that gender. `o` and `u`
before it follow it. A word with no deciding vowel is masculine.

| Typed | Result |
| --- | --- |
| `unee` | үнээ |
| `huuhed` | хүүхэд |
| `ovqq` | өвөө |
| `hun` | хун (no deciding vowel: masculine, type `w` for хүн) |

On screen, earlier letters can change when a later vowel arrives (`у` → `ү`
while typing `unee`). After the first deciding vowel the word still switches
gender left to right, as before.

Two keys are **not** evidence for the look-ahead (they still switch the
gender for the letters after them):

- `ye` (е): mostly a loanword vowel. `kofye` → кофе, `akadyemiig` →
  академийг.
- An explicit ү (`w`, `wi`, `u'`, `ü`, ...) right after `g`: it is the suffix
  -гүй. `bolohgwi` → болохгүй, `ugwi` → угүй.

### 4.2 ий or ы

`ii` becomes **ы** only when all of these hold; otherwise it is ий:

1. the word already has a vowel (`bii` → бий, `iim` → ийм, `chiig` → чийг);
2. the word is masculine at that point (`geriin` → гэрийн);
3. the letter before is a consonant other than ж ч ш щ ь ъ г к й
   (`bagshiig` → багшийг, `tuujiig` → туужийг, `achiin` → ачийн,
   `angiin` → ангийн, `airgiig` → айргийг);
4. the word so far is not one of the и-final stems салх, такс, компан
   (`salhiig` → салхийг).

So `aaviin` → аавын, `nomiig` → номыг, `ohiniig` → охиныг, `uuliin` → уулын,
`hotiig` → хотыг. Capitals are kept: `AAVIIN` → ААВЫН, `Nomiig` → Номыг.

To override: `ii'` always gives ий (`aavii'n` → аавийн), and `y` or `iii`
always give ы. After г the rule keeps ий because back г (чарга → чаргыг) and
front г (айраг → айргийг) are typed the same.

### 4.3 The particle уу / үү

A word typed as just `u` or `uu` straight after a **space** takes the gender
of the word before it: `irsen uu` → ирсэн үү, `baina uu` → байна уу,
`sain uu` → сайн уу. Any other key in between (punctuation, Enter, a digit,
cancelling, or a caret move while nothing is being composed) forgets the
previous word, so `irsen. uu` → ирсэн. уу. Repeated spaces keep it, and so do
Shift and CapsLock (`Key::Modifier`). A caret move while a word is being
composed is consumed by the composer and does not forget it. Front ends send
the space as `Key::Space`; `Composer::commit()` (focus loss) and
`Composer::clear_context()` forget it on demand.

### 4.4 Classic behaviour

With smart vowels off, the engine walks the word left to right starting in
masculine mode; each gender-deciding vowel switches the mode for the letters
after it. `unee` → унээ, `ovqq` → овөө, `aaviin` → аавийн, `irsen uu` →
ирсэн уу.

### 4.5 What harmony does not cover

- **Words with no deciding vowel** (`hun`, `ug`, `huuhdiin`): genuinely
  ambiguous without a word list.
- **Other particles** typed as separate words.
- **ий vs ы after г**, and stems that end in и beyond the short list above.
- **Suffix spelling** (dropped vowels, ь before a suffix: `har'iin` stays
  харьийн).

Background and sources: [issue #7](https://github.com/TengisCh/buuz/issues/7).

## 5. Capital letters

Case follows the keys you press, letter by letter: `Sain` → Сайн,
`SAIN` → САЙН, `sAin` → сАйн. For multi-key letters the first key decides
(`Sh` and `SH` → Ш). `''` and `""` give the capital signs Ь and Ъ.

## 6. Editing inside a word

Backspace and Delete edit the **raw keys** and the word is converted again.
Once you move the caret with the arrow keys, Home or End, the word is frozen
as Cyrillic and further edits act on Cyrillic letters, as in the original.
Frozen Cyrillic letters still count as evidence for the gender of the word:
`у` and `о` (like `а`) are masculine evidence, `ү`, `ө` and `э` feminine.

## 7. Cyrillic to Latin

Not implemented. The core only converts Latin to Cyrillic. A reverse
converter is on the roadmap for the web converter. If added, it should
produce keys that convert back to the same text with the default table
(`ү` → `w`, `ө` → `q`, `х` → `h`, `ц` → `c`, `ы` → `y`, `ь` → `'`), so that
round trips are exact.

## 8. Planned changes

- [#8](https://github.com/TengisCh/buuz/issues/8) portable config and
  smarter defaults
- [#3](https://github.com/TengisCh/buuz/issues/3) custom key mappings
- [#13](https://github.com/TengisCh/buuz/issues/13) the full smart typing plan
