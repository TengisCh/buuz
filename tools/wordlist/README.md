# Word list builder

Builds `data/wordlist/mn.txt` (Mongolian word frequencies, issue #11) from a
Mongolian Wikipedia dump. Node only, no dependency. The dump is never
committed.

## Rebuild

```sh
# 1. Download the dump and its checksums into a scratch folder (not the repo).
mkdir -p /tmp/mnwiki && cd /tmp/mnwiki
curl -LO https://dumps.wikimedia.org/mnwiki/20261101/mnwiki-20261101-pages-articles.xml.bz2
curl -LO https://dumps.wikimedia.org/mnwiki/20261101/mnwiki-20261101-sha1sums.txt

# 2. Check the sha1 against Wikimedia's published value. Stop if it differs.
grep pages-articles.xml.bz2 mnwiki-20261101-sha1sums.txt
shasum -a 1 mnwiki-20261101-pages-articles.xml.bz2

# 3. Build from the repository root.
cd /path/to/buuz
bzcat /tmp/mnwiki/mnwiki-20261101-pages-articles.xml.bz2 \
  | node tools/wordlist/build.mjs > data/wordlist/mn.txt
```

The published sha1 of the 20261101 dump is
`41e081ae13ea84efef03cf3a2ef09ed1391a2748`. The output is deterministic: the
same dump gives the same file. A new dump needs a new date in the header
(`HEADER` in `build.mjs`) and in `data/wordlist/README.md`.

## What it does

- Reads the decompressed XML from stdin line by line. It expects one tag per
  line, as real dumps have.
- Keeps the text of pages in namespace 0, skips redirects.
- Lower-cases, then strips templates (nested too), references and other tags,
  tables, file and category links, comments, headings, bold and italic marks,
  URLs and HTML entities. Link syntax keeps the shown text. An unclosed `{{`,
  `[[`, `<!--` or `<ref` ends at the next blank line or after 64 KB.
- Splits on whitespace and punctuation, but keeps `-`, `'` and `’` inside a
  token, then splits the token on them. A part that is not pure Cyrillic
  (`2010-нд`, `Obama-гийн`) is dropped together with every later part, since
  what follows is only its suffix. Cyrillic parts (`хоёр-гурван`) stay split.
  Keeps parts made only of а-я, ё, ө, ү, 2 to 30 letters long.
- Drops Russian function words (a short stoplist: на, при, по, для, что ...)
  and words ending in `ского`, `ский`, `ская`, `ское`, `ские` or `ство`.
  Mongolian look-alikes such as `за`, `их` and `да` are kept, `ъ` and `ий` are
  not filtered, and `ого` and `ных` are not used as suffixes because they hit
  real Mongolian words (`бодлого`, `ажиллагааных`). The suffix rule still
  removes a few real words such as `ханство`; the printed counts show its size.
- Keeps words seen at least 3 times, then cuts on a count boundary: all words
  with count N or more, where N is the smallest count for which the total is at
  most 50,000. So the file can hold a little fewer than 50,000 words.
- Writes `# SPDX-License-Identifier: CC-BY-SA-4.0`, the attribution comment,
  then `word<TAB>count` lines by rank. It prints a summary to stderr.

Known noise: a Cyrillic suffix after a Cyrillic abbreviation (`АНУ-ын`) counts
as the word `ын`. The list holds surface forms, not stems.

## Test

```sh
node --test tools/wordlist/test
```

It runs the builder on `test/fixture.xml`, a tiny made-up dump.

## Suffix review table

`suffixes.mjs` mines case-suffix patterns from `mn.txt` (issue #52). A native
speaker reviews the result. Node only, no dependency.

```sh
node tools/wordlist/suffixes.mjs data/wordlist/mn.txt > suffix-review.tsv
```

The script finds listed words that are a listed stem plus a suffix variant. It
covers the genitive, accusative, ablative and comitative families. It groups
the pairs by family, stem class and gender. Each row gives the variant, the
number of pairs, the weight (sum of word counts), the share of the weight in
the group and up to three examples.

Do not commit the TSV. Make a new one when you need it. The family tables in
`suffixes.mjs` must stay the same as `core/src/suffix_choice.rs`.
