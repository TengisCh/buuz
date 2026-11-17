# How Buuz turns Latin into Cyrillic

Status: describes `buuz-core` on `main` as of November 2026 (a faithful port
of the original Buuz by Odbayar Nyamtseren). Planned changes are linked at
the end and are **not** implemented yet.

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
`'` and `"`. Anything else (including `ö`, `ü` and Cyrillic letters) ends the
word. A word can hold at most 50 keys.

## 2. Matching: longest rule first

The engine reads the raw keys left to right. At each position it tries the
**longest** sequence in the table first (3 keys, then 2, then 1). So:

- `sh` → ш, not с + х
- `ch` → ч, `kh` → х
- `iii` → ы, `ii` → ий, `i` → и
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
| `i` | и | `ii` | ий | `iii` | ы |
| `ya` | я | `ye` | е | `yo` | ё |
| `yu` | ю | | | | |

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
| `q`, `o'`, `'o` | ө |
| `w`, `u'`, `'u` | ү |
| `o"`, `"o` | о |
| `u"`, `"u` | у |

### Signs

| Keys | Result |
| --- | --- |
| `'` | ь |
| `''` | Ь |
| `"` | ъ |
| `""` | Ъ |

## 4. Vowel harmony today

Every vowel rule carries flags: which kind of word it may appear in, and
whether it decides the word's gender.

- **Make the word masculine:** `a`, `ya`, `yo`, `yu`, `o"`, `u"` (and `o`, `u`
  once the word is already masculine).
- **Make the word feminine:** `e`, `ye`, `q`, `w`, `o'`, `u'`.
- **Neutral:** `i`, `ii`, `iii`, `y`, all consonants and signs.

The engine walks the word **left to right**, starting in **masculine** mode.
Each gender-deciding vowel switches the mode for the letters after it. `o`
and `u` take the form for the current mode.

Examples (all checked against the current core):

| Typed | Result | What happened |
| --- | --- | --- |
| `geree` | гэрээ | no `o`/`u` involved |
| `geree uu` | гэрээ уу | each word starts again in masculine mode (should be үү) |
| `ovoo` | овоо | masculine by default |
| `ovqq` | овөө | `o` was decided before `q` made the word feminine |
| `unee` | унээ | same: `u` decided before `e` |
| `hwn` | хүн | explicit `w` |
| `hun` | хун | no feminine vowel, so masculine |

### What harmony does not cover today

- **Look-ahead:** `o`/`u` before the first deciding vowel are always
  masculine (`unee` → унээ, `huuhed` → хуухэд).
- **ий / ы:** `ii` is always ий. In masculine words the long и after most
  consonants should be ы (`aaviin` → аавийн, should be аавын). Users must
  type `y` themselves.
- **Particles typed as separate words:** `irsen uu` → ирсэн уу; the
  particle should follow the previous word (ирсэн үү).
- **Words with no deciding vowel** (`hun`, `ug`, `huuhdiin`): genuinely
  ambiguous without a word list.

Proposed fixes, with test cases and questions for native speakers:
[issue #7](https://github.com/TengisCh/buuz/issues/7).

## 5. Capital letters

Case follows the keys you press, letter by letter: `Sain` → Сайн,
`SAIN` → САЙН, `sAin` → сАйн. For multi-key letters the first key decides
(`Sh` and `SH` → Ш). `''` and `""` give the capital signs Ь and Ъ.

## 6. Editing inside a word

Backspace and Delete edit the **raw keys** and the word is converted again.
Once you move the caret with the arrow keys, Home or End, the word is frozen
as Cyrillic and further edits act on Cyrillic letters, as in the original.

## 7. Cyrillic to Latin

Not implemented. The core only converts Latin to Cyrillic. A reverse
converter is on the roadmap for the web converter. If added, it should
produce keys that convert back to the same text with the default table
(`ү` → `w`, `ө` → `q`, `х` → `h`, `ц` → `c`, `ы` → `y`, `ь` → `'`), so that
round trips are exact.

## 8. Planned changes

- [#7](https://github.com/TengisCh/buuz/issues/7) smarter vowel harmony
- [#9](https://github.com/TengisCh/buuz/issues/9) accept `ö` and `ü`
- [#8](https://github.com/TengisCh/buuz/issues/8) portable config and
  smarter defaults
- [#3](https://github.com/TengisCh/buuz/issues/3) custom key mappings
- [#13](https://github.com/TengisCh/buuz/issues/13) the full smart typing plan
