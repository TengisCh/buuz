# Mongolian word list

`mn.txt` holds the most frequent words of Mongolian Wikipedia (about 48,000), one
`word<TAB>count` line per word, best rank first. The first two lines are `#`
comments: the SPDX licence line and the attribution. Buuz uses it for suggestions and autocorrect (issues #1 and #2). The
core parses it with `WordList` and loads it on demand.

- **Source:** Mongolian Wikipedia, dump `mnwiki-20261101`
  (`mnwiki-20261101-pages-articles.xml.bz2`, sha1
  `41e081ae13ea84efef03cf3a2ef09ed1391a2748`), from
  <https://dumps.wikimedia.org/mnwiki/20261101/>.
- **Attribution:** Mongolian Wikipedia contributors.
- **Licence:** [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
  See [LICENSE](LICENSE). This applies to the data in this folder only, not to
  the Buuz code.
- **Changes:** word counts derived; no text kept. Only lower-cased words and
  how often each occurs.
- **How it was built:** `tools/wordlist/build.mjs`, see
  [tools/wordlist/README.md](../../tools/wordlist/README.md). Pages in the
  main namespace, markup stripped, tokens of Mongolian Cyrillic letters only,
  Russian function words and endings dropped, seen at least 3 times, cut at
  a count boundary so the list has at most 50,000 words.
