# 0006: Word list data under CC BY-SA 4.0, as a separate asset

Status: Accepted
Date: 2026-11-18
Amends: [0004](0004-license-mit-or-apache-for-new-code.md)

## Context

Suggestions and autocorrect (issues #1 and #2) need a Mongolian word list
with frequencies. The maintainer chose the Mongolian Wikipedia dump as the
source (issue #11). Its text is CC BY-SA 4.0. Whether a list of word counts
is a derivative of the text is a legal judgement that no source we found
settles. Others who publish Wikipedia-derived frequency lists (wordfreq,
FrequencyWords) license them CC BY-SA 4.0 with attribution.

## Decision

- `data/wordlist/mn.txt`, its README and its builder output are licensed
  **CC BY-SA 4.0**, with attribution to Mongolian Wikipedia contributors. It
  holds derived counts only; no article text is kept.
- The list stays a separate data file. It is loaded as a separate asset at run
  time and is not compiled into app binaries (no `include_str!` in shipped
  code; tests may read it).
- Shells show the attribution and the licence link in their licence screens
  once they ship the file.
- The builder (`tools/wordlist/`) and the loader (`core/src/wordlist.rs`) keep
  their code licences.

## Consequences

- Share-alike applies to the data file, and to adaptations of it. It does not
  reach the app code, because the code only reads the file.
- Every shell that bundles `mn.txt` must ship `data/wordlist/LICENSE` and the
  attribution, and must keep the file separable.
- The first line of `mn.txt` carries `SPDX-License-Identifier: CC-BY-SA-4.0`,
  so the licence travels with the file.
- If a lawyer decides counts are not a derivative, the licence can be relaxed
  by a later record. Until then, the safe path stands.
