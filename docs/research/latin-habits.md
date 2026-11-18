# Latin typing habits: first measurements

Status: draft for review. Measured on synthetic pairs. Issue
[#42](https://github.com/TengisCh/buuz/issues/42).

## Summary

- Smart mode already handles `kh` and `x` for х, `ü` for ү, and `ö` for ө.
  The table has these keys today.
- Smart mode fails on every habit that needs a key the table does not have:
  `ts`, `zh`, `c` for ч, `u` for ө, `8` for ө, `ia`, `iu`, `io`, `y` for ү.
- A `[keys]` entry fixes `ts`, `zh` and `ia` with few breaks. It cannot fix
  `u` for ө, because `u` already has a meaning.
- A word-list rule fixes most `u` for ө words in this test. The estimate is
  too good, because the test words come from the same list.
- The `mixed` chat profile scores 77.7% in smart mode and 74.7% in classic
  mode.

## Method

1. Take the 3,000 most frequent words of `data/wordlist/mn.txt`.
2. For each profile, keep the words that hold the changed letter. Take every
   Nth word by rank, at most 300 words. The `mixed` profile keeps every word.
3. Turn each word into Latin with the profile habits
   (`core/tests/benchmark.rs`). Write the pairs to
   `data/benchmark/habits.tsv`.
4. Convert each Latin text and compare it with the Cyrillic word. A pair
   scores only on an exact match.
5. Convert with a Space after the word, as in real typing. The Space lets
   abbreviations and autocorrect run. The Space is cut from the result.

Three configurations run on every pair:

| Name | Options |
| --- | --- |
| smart | `Options::smart()` |
| classic | `Options::classic()` |
| autocorrect | `Options::smart()` with the full word list and `autocorrect` on |

## Profiles

The base spelling follows the table: х is `h`, ц is `c`, ч is `ch`, ө is `q`,
ү is `w`, е is `ye`, ё is `yo`, ю is `yu`, я is `ya`, ь is `'`, ий is `ii'`.
A й after a vowel is `i`. The base uses `ii'` because it is always right. This
means the `ii_for_ий` profile measures the habit of typing `ii`.

| Profile | Habit |
| --- | --- |
| `kh`, `x_for_h` | х as `kh`, х as `x` |
| `ts` | ц as `ts` |
| `zh_for_ж`, `c_for_ч` | ж as `zh`, ч as `c` |
| `u_for_ү`, `v_for_ү`, `y_for_ү`, `umlaut_ü` | ү as `u`, `v`, `y`, `ü` |
| `o_for_ө`, `umlaut_ö`, `8_for_ө`, `u_for_ө` | ө as `o`, `ö`, `8`, `u` |
| `ia_for_я`, `iu_for_ю`, `io_for_ё` | я as `ia`, ю as `iu`, ё as `io` |
| `e_for_е` | е as `e` |
| `y_for_й` | й after a vowel as `y` |
| `ii_for_ы`, `i_for_ы` | ы as `ii`, ы as `i` |
| `ii_for_ий` | ий as `ii` |
| `drop_soft_sign`, `i_for_soft_sign` | ь dropped, ь as `i` |
| `single_long_vowel` | aa, оо, уу, үү, өө, ээ typed with one letter |
| `mixed` | ц as `ts`, ү as `u`, ө as `o`, ь dropped, ий as `ii` |

The plan lists `ya`, `yu`, `ye` and `yo` as habits. These are the base
spelling, so the profiles are the other spelling (`ia`, `iu`, `e`, `io`). The
base also types й as `i`, so `i_for_й` has no profile.

## Results

Share of pairs with an exact match. Pairs are 5,075 in total, with 5 manual
pairs.

| Profile | Pairs | Smart | Classic | Autocorrect |
| --- | --- | --- | --- | --- |
| base | 300 | 98.7% | 99.0% | 99.7% |
| kh | 245 | 99.6% | 99.6% | 100.0% |
| x_for_h | 245 | 99.6% | 99.6% | 100.0% |
| ts | 136 | 0.0% | 0.0% | 0.0% |
| zh_for_ж | 288 | 0.0% | 0.0% | 0.0% |
| c_for_ч | 214 | 0.0% | 0.0% | 0.0% |
| u_for_ү | 200 | 56.0% | 11.5% | 87.5% |
| v_for_ү | 200 | 30.0% | 0.0% | 30.0% |
| y_for_ү | 200 | 0.0% | 0.0% | 0.0% |
| umlaut_ü | 200 | 100.0% | 100.0% | 100.0% |
| o_for_ө | 232 | 11.6% | 4.7% | 62.1% |
| umlaut_ö | 232 | 99.6% | 99.6% | 100.0% |
| 8_for_ө | 232 | 0.0% | 0.0% | 0.0% |
| u_for_ө | 232 | 0.0% | 0.0% | 0.0% |
| ia_for_я | 83 | 0.0% | 0.0% | 0.0% |
| iu_for_ю | 19 | 0.0% | 0.0% | 0.0% |
| e_for_е | 135 | 0.0% | 0.0% | 81.5% |
| io_for_ё | 18 | 0.0% | 0.0% | 0.0% |
| y_for_й | 161 | 0.0% | 0.0% | 0.0% |
| ii_for_ы | 275 | 92.4% | 0.0% | 96.0% |
| i_for_ы | 275 | 0.0% | 0.0% | 0.0% |
| ii_for_ий | 191 | 89.0% | 99.0% | 99.0% |
| drop_soft_sign | 83 | 0.0% | 0.0% | 61.4% |
| i_for_soft_sign | 83 | 1.2% | 0.0% | 1.2% |
| single_long_vowel | 291 | 0.0% | 0.0% | 71.1% |
| mixed | 300 | 77.7% | 74.7% | 88.3% |
| manual u_for_ө | 4 | 0.0% | 0.0% | 0.0% |
| manual v_for_ү | 1 | 100.0% | 0.0% | 100.0% |

Observations:

- The base profile scores 98.7%. The four misses are words that the table
  cannot type in smart mode: `boljee` (бөлжээ, not болжээ), `profyessor`,
  `yevryei` and `molyekulyn`. The `ye` key does not decide the gender.
- Autocorrect uses the word list that holds every test word. It is an upper
  bound. Do not read it as the gain for real text.
- `ii_for_ий` is better in classic mode (99.0%) than in smart mode (89.0%).
  The smart rule turns `ii` into ы after л, as in `surguuliin` (сургуулын,
  not сургуулийн).
- `ii_for_ы` is the other side. Smart mode gets 92.4%, classic mode gets 0%.
- `v_for_ү` scores 30% in smart mode because `vv` gives үү. A single `v`
  stays в.

## Candidate rules: fixed and broken

Each rule runs on all 5,075 pairs. "Target" is the profile the rule is for.
"Broken (base)" counts pairs of the base profile that were right before and
are wrong after.

### Key map rules

These are exact. The core does them today with a `[keys]` entry.

| Rule | Target before | Target after | Fixed (all) | Broken (all) | Broken (base) |
| --- | --- | --- | --- | --- | --- |
| `ts = "ц"` | 0.0% | 100.0% | 149 | 3 | 0 |
| `zh = "ж"` | 0.0% | 97.6% | 281 | 0 | 0 |
| `c = "ч"` | 0.0% | 99.5% | 213 | 71 | 13 |
| `ia = "я"` | 0.0% | 75.9% | 63 | 10 | 3 |
| `iu = "ю"` | 0.0% | 47.4% | 9 | 5 | 2 |
| `io = "ё"` | 0.0% | 38.9% | 7 | 15 | 3 |
| `v = "ү"` | 30.0% | 93.0% | 129 | 178 | 23 |
| `y = "ү"` | 0.0% | 99.5% | 199 | 78 | 22 |

- `ts` and `zh` break almost nothing. Both are good defaults.
- `c` takes the key from ц, so every word with ц breaks.
- `v` and `y` take the key from в and ы. They are a user choice, not a
  default.
- `ia`, `iu` and `io` break words with и + а, у or о (`сургуулиар`,
  `ариун`, `радио`).

### Latin pre-processing

This is an estimate: the Latin text is changed before the core sees it.

| Rule | Target before | Target after | Fixed (all) | Broken (all) | Broken (base) |
| --- | --- | --- | --- | --- | --- |
| `8` becomes `q` | 0.0% | 99.6% | 231 | 0 | 0 |

The core treats `8` as a digit and ends the word, so this needs a core change.

### Word-list rules

These are estimates. The rule converts several Latin spellings and keeps a
listed word. "Best by count" takes the most frequent listed word.
"Only if unlisted" tries the other spellings only when the plain result is
not a listed word. The test words come from the same list, so the numbers are
too good for real text.

| Rule | Target before | Target after | Fixed (all) | Broken (all) | Broken (base) |
| --- | --- | --- | --- | --- | --- |
| `u` may be ө, best by count | 0.0% | 97.5% | 230 | 1 | 0 |
| `u` may be ө, only if unlisted | 0.0% | 93.2% | 220 | 0 | 0 |
| `u` may be ү or ө, best by count | 25.7% | 97.0% | 324 | 1 | 0 |
| `o` may be ө, best by count | 11.6% | 97.8% | 220 | 0 | 0 |
| chat: `u` ү or ө, `o` ө, best by count | 77.7% | 88.7% | 544 | 1 | 0 |
| add ь, only if unlisted | 0.0% | 65.1% | 59 | 0 | 0 |
| `i` may be ь, only if unlisted | 1.2% | 88.0% | 72 | 0 | 0 |
| `e` may be ye, only if unlisted | 0.0% | 69.6% | 94 | 0 | 0 |
| double a vowel, only if unlisted | 0.0% | 85.6% | 249 | 0 | 0 |

The one break for `u` is `uuriin` (уурын), which becomes өөрийн. Both are
common words. The wrong word wins by count.

## Limitations

- The pairs are synthetic. A real person mixes habits and makes typing errors.
- The words come from Mongolian Wikipedia. Chat uses other words, and short
  words, names and loanwords are rare in the sample.
- The word-list rules and autocorrect are tested on words from the list
  itself. The real rate is lower.
- The profiles use one habit each. The `mixed` profile is a guess about chat.
- The fix and break counts depend on how many pairs each profile has. A
  profile with few words (`iu_for_ю`, `io_for_ё`) gives weak numbers.
- The base profile uses `ii'` for ий, which nobody types. See `ii_for_ий`.
- No real chat data is in this test yet.

## Public evidence

Research date 2026-11-18. Public sources only. No source gives a frequency
for each variant of a letter, so the strength below says only how well a
variant is shown to exist.

| Habit | Variants seen | Strength | Main source |
| --- | --- | --- | --- |
| х | `kh`, `h`, `x` | Strong that all three exist. No frequency | S1, S4 |
| ц | `ts`, `c` (`c` is the 2003 standard) | Some | S1 |
| ч | `ch`, also `c`, `j`, `ts` | Some | S1 |
| ү | `u` (main), `v`, `y`; `ü` is rare | Strong for `u`. Some for `v`, `y`. Anecdotal that `v` is "nearly universal" | S1, S2, S4, S5 |
| ө | `o` (main), `u`; `ö` is rare; `o'` is the 2003 standard | Strong for `o`. Some for `u` | S1, S2, S4 |
| е, ё, я, ю | `ye`, `e`, `y`, `i` (е); `ya`, `y` (я); `yu`, `y` (ю) | Some | S1 |
| й | `i`, `y` in chat words | Some | S1 |
| ы | `y`, `i`, `ii` | Some | S1 |
| ий | `ii` | Anecdotal | S1, S6 |
| ь | `i`, `e`, `'` | Some for `i` and `e`. Anecdotal for `'` | S1, S5 |
| Long vowel | Doubled in the standard. Both doubled and single in chat | Strong that both exist. Anecdotal that single is common | S1, S5 |

No public evidence was found for these profiles:

- `8` or `q` for ө.
- `zh` for ж. Sources give `j`.
- `ia`, `iu` or `io` for я, ю, ё.
- ь dropped. Sources show ь as `i`, `e` or `'`.
- `w` or `x` for ш. There is no Buuz profile for it.

Numbers from the sources:

- **Why Latin.** A 2018 survey of 74 Facebook users (quoted in S1) gives
  these reasons: faster (61.7%), no spelling rules (60.9%), Latin
  keyboards (56.4%). 85% did not know the standard romanization.
- **Hard letters.** In the S4 survey, 60% said some standard letters are
  missing on devices. 35% said they "write by following others". The hardest
  letters were Ө (34%), Ү (28%), Х (20%) and В (18%).
- **Spread.** S1 lists more than one Latin choice for 17 of the 35 letters.
  One name has seven spellings in S1: `Munkhsukh`, `Munksvkh`, `Monkhsukh`,
  `Monhsuh`, `Munhsuh`, `Munxsvx`, `Monhsvh`.
- **Normalization accuracy.** S2 trained on 2,200 real social media
  sentences and tested on 200 sentences (1,663 words). The best seq2seq model
  gets 86.04% word accuracy and 94.25% character accuracy. The statistical
  baseline gets 82.08% and 92.97%. On unknown words the figures are 73.16% and
  62.19%. A later paper (S3) reports a word error rate of 13.41%. The
  datasets are not public.
- **MNS 5217:2012.** The 2012 revision of the standard gives ө = `ö`,
  ү = `ü`, х = `kh`, ц = `ts`, ч = `ch`, ш = `sh`, ж = `j`, й = `i`,
  ы = `y`, ь = `i`, ъ = `i`. The 2003 version used `x` for х, `c` for ц,
  `o'` and `u'`. Both S1 and S9 say the standard cannot be reversed, because
  several Cyrillic letters share `i` or `sh`. Most people do not follow it.
  No public passport rule was found.

Limits of the evidence: S1 is one thesis and its corpus is not public. The
`v` claim in S5 is a blog post and S1 ranks `v` third. S3 and S12 were read
through search snippets only.

### Sources

| ID | Source | URL |
| --- | --- | --- |
| S1 | Byambadorj Zolzaya, doctoral thesis, Tokushima University, 2022, chapter 2 and appendices A, B | <https://tokushima-u.repo.nii.ac.jp/record/2009963/files/k3634_fulltext.pdf> |
| S2 | Byambadorj et al., normalization of transliterated words, NLP Annual Meeting 2020 | <https://anlp.jp/proceedings/annual_meeting/2020/pdf_dir/E5-3.pdf> |
| S3 | Byambadorj et al., ACM TALLIP 2021 (abstract only) | <https://aminer.org/pub/619788eb5244ab9dcb0dfacb/normalization-of-transliterated-mongolian-words-using-seq-seq-model-with-limited-data> |
| S4 | Translation Studies IX, National University of Mongolia, article on letter-by-letter transliteration | <https://journal.num.edu.mn/TranslationStudies/article/download/1011/923/3530> |
| S5 | Blog post "Transliterating Mongolian", Eye of Tengri, 2015 | <https://eye-of-tengri.blogspot.com/2015/06/transliterating-mongolian.html> |
| S6 | Wikipedia, Naming conventions (Mongolian) | <https://en.wikipedia.org/wiki/Wikipedia:RMN> |
| S7 | ICU transliteration data `mn_mn_Latn_MNS.txt` | <https://cobalt.googlesource.com/cobalt/+/3f5e51355f2ee9074dbf39b90e7732e31303fb5c%5E/third_party/icu/source/data/translit/mn_mn_Latn_MNS.txt> |
| S8 | BGN/PCGN 1964 romanization of Mongolian | <https://assets.publishing.service.gov.uk/media/5d288d91e5274a14efbe5032/ROMANIZATION_OF_MONGOLIAN.pdf> |
| S9 | Interscript, MNS 5217:2012 system page | <https://interscript.org/systems/masm-mon-Cyrl-Latn-5217-2012> |
| S10 | Transliteration between Mongolian scripts, Cyrillic and Latin | <https://multilingual-technologies.netlify.app/article/mongolian_transliteration> |
| S12 | r/mongolia thread on Mongolian in Latin letters (snippet only) | <https://reddit.birdcat.cafe/r/mongolia/comments/1l1a176/why_mongolian_in_alphabet_isnt_translatable> |

## Proposed changes

Ranked by public evidence first, then by fixed and broken counts from the
tables above. Each change starts with the pairs in the last column. Add the
pairs as failing tests (`tdd` skill) before the change.

Group (a): accept now. A key rule in smart mode, with its own switch or as
part of an existing one.
Group (b): needs the word list. These belong with autocorrect (more
neighbours for a typed word) or a later stage.
Group (c): reject.

| # | Change | Group | Evidence | Fixed / broken | First test pairs |
| --- | --- | --- | --- | --- | --- |
| 1 | `ts` gives ц (also in the word `хүртсэн`, see the break). Done in #63: `habit_keys` with a word list check | (a) own switch for habit keys | Some, and the 2012 standard | 149 / 3 | `tsagaan` → цагаан, `tsereg` → цэрэг. Known break: `hwrtsen` → хүртсэн |
| 2 | `zh` gives ж. Done in #63 | (a) same switch as 1, low priority | None found | 281 / 0 | `gezh` → гэж, `baizhee` → байжээ |
| 3 | `o` may be ө, chosen with the word list | (b) | Strong | 220 / 0 | `tov` → төв, `mon` → мөн, `torii'n` → төрийн |
| 4 | `u` may be ө or ү, chosen with the word list | (b) | Strong for `u` as ү, some as ө | 324 / 1 | `uvul` → өвөл, `udur` → өдөр, `tuv` → төв, `uruu` → өрөө. Guard: `uuriin` → уурын |
| 5 | ий or ы after л chosen with the word list | (b), also a bug | Some | see the bug below | `surguuliin` → сургуулийн. Done in #62: `ii_for_ий` smart mode with a list goes from 89.0% to 97.4% |
| 6 | `e` may be ye | (b) | Some | 94 / 0 | `wed` → үед, `erqnhii'` → ерөнхий |
| 7 | A single vowel may be a long vowel | (b) | Anecdotal | 249 / 0 | `terer` → тэрээр, `zwn` → зүүн |
| 8 | `i` or a missing letter may be ь | (b) | Some for `i`. None found for dropped | 72 / 0 (`i`), 59 / 0 (add) | `huuli` → хууль, `surguul` → сургууль |
| 9 | `8` gives ө | (c) for now | None found | 231 / 0 | `t8v` → төв (only if evidence appears) |
| 10 | `ia`, `iu`, `io` give я, ю, ё | (c) | None found | 63 / 10, 9 / 5, 7 / 15 | none |
| 11 | `c` gives ч | (c) | Some, but `c` is ц in the 2003 standard | 213 / 71 | none |
| 12 | `v` or `y` gives ү as a built-in key | (c) | Some for `v` and `y` | 129 / 178, 199 / 78 | none. Stays a user `[keys]` choice |

Notes:

- Rows 3 and 4 have the best evidence. Autocorrect already scores 62.1% on
  `o_for_ө` and 87.5% on `u_for_ү`. The estimates above are too good,
  because the test words come from the list itself. Measure again on
  held-out words before the build.
- Row 1: the 3 breaks are words with тс typed as `ts`. The word list check
  in #63 undoes them. After #63, smart mode gets 99.3% on `ts` (was 0.0%),
  97.6% on `zh_for_ж` (was 0.0%) and 82.0% on `mixed` (was 77.7%).
- Rows 10 to 12 break base-profile words. Row 12 stays a user choice. Calibration
  already offers it.

### Bug: ий typed as `ii` turns into ы after л

Smart mode gives 89.0% on `ii_for_ий` and classic mode gives 99.0%. The
failing pair is `surguuliin`, which gives сургуулын. Want: сургуулийн. The
other misses are `huviin` (хувын), `niitiin`, `baigaliin` and `britaniin`.
21 of 191 pairs fail.

This is not a one-line fix. The documented rule gives `uuliin` → уулын
(typing-rules section 4.2), and уулын is a real word. The word list tells
the two apart: сургуулийн is listed (count 6,194) and сургуулын is not.
Fix it in group (b): when the ы result is not a listed word and the ий
result is, use ий. First test pairs: `surguuliin` → сургуулийн and
`huviin` → хувийн, with the guard `uuliin` → уулын.

### Not measured yet

- `y` for й after a vowel (`bayna` → байна). S1 shows chat words with it.
  The profile scores 0% today and no candidate is measured.
- Dropped vowels in chat (`hgd` for хийгээд). Anecdotal only.
