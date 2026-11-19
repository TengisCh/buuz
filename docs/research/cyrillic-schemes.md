# Cyrillic schemes for other languages

Buuz types Mongolian with Latin keys. Many people type other Cyrillic
languages the same way. This note lists what they type, and which table
Buuz uses for each language. The code is in `core/src/scheme.rs`.

## What "encoding" means here

It is not a character encoding. It is a Latin to Cyrillic **scheme**: the
habit that says `zh` gives `ж`. Mongolian keeps its own engine, with vowel
harmony. The other languages use a plain table, matched longest first.

## Russian

Russian speakers use three kinds of scheme.

| Scheme | Who uses it | Example | In Buuz |
| --- | --- | --- | --- |
| Everyday translit | Chat, SMS, people without a Cyrillic keyboard | `privet` gives привет | `ru` |
| Standard romanisation | Official documents, libraries, street signs | GOST 7.79, ISO 9, the passport table | `ru-gost` (system B only) |
| Volapük (look-alike) | Chat without a Cyrillic keyboard, and to avoid word filters | `npuBeT` gives привет, `4to` gives что | `ru-volapuk` |
| Phonetic keyboard | People who type Russian on a Latin keyboard layout | `q` gives я | Not in Buuz |

### Everyday translit (`ru`)

| Latin | Cyrillic | Note |
| --- | --- | --- |
| `zh` `kh` `ts` `ch` `sh` | ж х ц ч ш | |
| `shch` `sch` `shh` | щ | |
| `yo` `yu` `ya` `ye` | ё ю я е | `jo`, `ju`, `ja`, `je` work too |
| `y` after a vowel | й | `moy` gives мой |
| `y` after a consonant | ы | `krasivyy` gives красивый |
| `j` | й | |
| `e` | е | `e'` gives э. A word that starts with `et` gives эт (это, этот) |
| `'` | ь | After a letter only |
| `''` | ъ | |
| `x` | кс | |
| `c` | ц | |
| `w` | в | |

Known limits. The table has no word list, so it cannot choose between two
readings:

- `ts` is always ц. A word with т and с next to each other (отсутствие)
  comes out wrong. The same is true for `sh`, `zh`, `kh` and `ch`.
- `e` is always е. Write `e'` for э (`e'kran` gives экран).
- `-tsya` is always ця. Write `t'sya` for ться.

A later change can fix these with a Russian word list, as Mongolian does.

### GOST 7.79 system B (`ru-gost`)

System B is the ASCII form of the standard. Buuz reads it backwards. The
table is from the [GOST 7.79 article][gost].

| Cyrillic | Latin |
| --- | --- |
| ё ю я | `yo` `yu` `ya` |
| ж ч ш щ | `zh` `ch` `sh` `shh` |
| ц | `cz`, or `c` before i, e, y, j |
| х | `x` |
| й | `j` |
| ы э | ``y` `` ``e` `` |
| ь ъ | `` ` `` ``` `` ``` |

The apostrophe `'` works for the grave accent. System A (ISO 9) needs
letters with marks (š, ž, č). It is not in Buuz yet.

### Volapük (`ru-volapuk`)

Volapük writes Cyrillic with Latin letters and digits that look the same.
It is the reverse of everyday translit: `p` is р, not п.

| Look-alike | Cyrillic |
| --- | --- |
| `a` `c` `e` `k` `m` `o` `t` `x` | а с е к м о т х |
| `b` `h` `p` `y` | в н р у |
| `n` `u` `r` `w` | п и я ш |
| `3` `4` `6` `9` `0` | з ч б я о |
| `bl` `b\|` `10` `\|0` `lli` `iii` | ы ы ю ю щ ш |
| `>\|<` | ж |

- A digit gives a letter only next to a letter. `2026` stays a number.
- A capital inside a word is a look-alike, not a capital letter. `npuBeT`
  gives привет. Only the first letter of a word keeps its case.
- `b` is в, except in `bl` and `b|`, which are ы. Write `'` for ь.
- `bI` (capital i) is not ы: the capital turns to lower case, so it is `bi`, ви.
- Letters with no look-alike use their sound (`d` д, `f` ф, `z` з).

## Other languages

All tables are modelled on the Russian one, with the letters of the
language added. Only Russian is checked by a native speaker. The other
schemes are **drafts**. The web page marks them as drafts.

| Id | Language | Special keys |
| --- | --- | --- |
| `uk` | Ukrainian | `h` г, `g` ґ, `y` и, `i` і, `yi` ї, `ye` є, `''` ʼ |
| `be` | Belarusian | `i` і, `y` ы, `w` or `u'` ў, `h` г |
| `bg` | Bulgarian | `y` й, `q` я, `sht` щ, `u'` ъ |
| `sr` | Serbian | `lj` љ, `nj` њ, `dj` ђ, `dzh` џ, `ch` ч, `cc` ć ћ, also č ć š ž đ dž |
| `mk` | Macedonian | As Serbian, plus `gj` ѓ, `kj` ќ, `dz` ѕ |
| `kk` | Kazakh | `ae` ә, `gh` ғ, `q` қ, `ng` ң, `oe` ө, `u'` ұ, `ue` ү, `h'` һ, `iy` и |
| `ky` | Kyrgyz | `ng` ң, `oe` ө, `ue` or `u'` ү, `j` ж |

Ukrainian does not read `ia`, `iu` or `ie` as я, ю, є. The letter і is often
followed by a vowel (діалог), so only `ya`, `yu`, `ye` and `yi` are special.

Serbian and Macedonian have a one-to-one Latin alphabet, so these tables are
exact for text in that alphabet. The ASCII digraphs `dj`, `nj` and `lj` can
be wrong for a word that really has д+ј (подјела), as in Serbian Latin.

## Official standards, read backwards

Each language can have two schemes: the casual habit, and the official
romanisation read backwards. The casual one is the default.

| Id | Standard | Source |
| --- | --- | --- |
| `ru-gost` | GOST 7.79 system B | [Wikipedia][gost] |
| `uk-kmu` | Ukrainian national system, 2010 (resolution 55) | [Wikipedia][uk] |
| `mn-mns` | MNS 5217:2012, used on Mongolian passports | [CLDR rules][mns] |

An official table is made to go from Cyrillic to Latin. Read backwards, it
cannot give back every letter.

- `mn-mns` writes и, й, ъ and ь all as `i`, and ш and щ both as `sh`.
  The reverse table gives и and ш. This is the Mongolian official standard.
  The typing rules of Buuz stay the casual way for Mongolian.
- `uk-kmu` writes ї as `i` after the first letter, and drops ь. So
  `Ukraina` gives Украіна, not Україна.

Not added, because no source table was checked: the Bulgarian 2009
streamlined system, the Kazakh 2021 Latin alphabet and the Belarusian 2000
instruction. The Bulgarian system is the casual table without ъ.

## Not in Buuz

- **Wrong layout repair.** Text typed with the Latin layout by mistake
  (`ghbdtn` for привет). This is a different feature.
- **Passport romanisation** (Order 1047, ICAO). It loses information:
  й and и are both `i`, and ь has no letter.
- **Tajik, Tatar, Bashkir, Kalmyk and Buryat.** Add them when a speaker asks.

## How to add a language

1. Open an issue with the language and how people type it.
2. Add a `Scheme` variant, a rule table and tests in `core/`.
3. Ask a native speaker to check the draft, then remove its draft flag.

[gost]: https://en.wikipedia.org/wiki/GOST_7.79-2000
[uk]: https://en.wikipedia.org/wiki/Romanization_of_Ukrainian
[mns]: https://cobalt.googlesource.com/cobalt/+/3f5e51355f2ee9074dbf39b90e7732e31303fb5c%5E/third_party/icu/source/data/translit/mn_mn_Latn_MNS.txt
