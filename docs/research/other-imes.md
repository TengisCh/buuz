# Features from other input methods

Status: draft. Researched 2026-11-19. Updates the table in issue #6.

## Summary

- Buuz has most of what mature input methods do well. It has ghost
  completion, word choice, autocorrect with undo, abbreviations, a personal
  dictionary, Keep Latin, Auto Latin and calibration.
- The gaps are:
  - choosing between two good words
  - hiding a wrong suggestion
  - undo after punctuation
  - learning names in bulk
- Five proposals follow. Each one runs on the per-word path or offline. None
  adds work to the per-key path.
- Cloud candidates stay out. A 2024 Citizen Lab report found that 8 of 9
  cloud pinyin keyboards could leak keystrokes.

## Method

- Read the docs and the core code for the current state.
- Read the Mozc config schema and the Mozc dictionary importer.
- Searched for Rime, Sogou, Gboard, Varnam, Microsoft Indic tools, Korean
  IMEs and Citizen Lab reports.
- Limits: not confirmed whether Gboard Hindi shows the typed Latin as a
  candidate. Not confirmed from Mozc docs: the undo-commit key. A Firefox bug
  confirms the feature.

## Feature table

Fit "Yes" means the feature keeps the privacy rule (no network, no keystroke
logs) and the speed rule (no allocation on the per-key path).

| Feature | Who does it | Buuz today | Fit |
| --- | --- | --- | --- |
| Candidate list, number keys | Mozc, Rime, Sogou, Microsoft Indic | None. Word choice picks one word | Yes, on demand and per word. Proposal 3 |
| Suggestion strip, ghost text | Mozc, Gboard | Completion with Tab (section 12) | Done |
| History reorders the choice | Mozc, Rime, Gboard | Use counts feed completion only, not word choice | Yes. Proposal 1 |
| Hide a suggestion | Mozc suppression, Gboard | A rejected list word still shows as ghost | Yes. Proposal 2 |
| Undo the last commit | Mozc, Gboard, iOS | Backspace after Space only | Yes. Proposal 4 |
| Learn words from a text file | Varnam `learn-from-file` | Add one word at a time (web settings) | Yes, user started. Proposal 5 |
| Learning levels, incognito | Mozc, Android, SwiftKey | `Options::learning` false. Shells switch it off for passwords | Core done. A "Pause learning" menu item is shell work |
| Dictionary import and export | Mozc, Rime | Web has `.txt` import and export | Done on web. Desktop pending |
| Tolerant spelling | Rime, Sogou, Gboard Hindi | Habit keys, Double V, word choice | Done |
| Phrases from short codes | Rime, Sogou | Abbreviations (section 7) | Done |
| Temporary Latin | Rime, Microsoft Pinyin | Esc (Keep Latin), Auto Latin | Done |
| Mode errors | Korean study, CHI 2019 | Preedit is the preview | Per-app memory is shell work |
| Reconversion | Mozc, MS-IME | None | Later |
| Next-word prediction | Mozc, Gboard | None | Later. Needs bigram data |
| Emoji by keyword | Gboard, Rime | None | Low priority |
| Cloud candidates | Sogou, Baidu, others | None | **No.** Breaks the privacy promise |

## Principles that carry over

- Run everything on the device.
- Keep the per-key path small. Put new work on the per-word path or offline.
- Give every automatic change a one-key undo.
- Let the user see, hide and clear what Buuz learned.
- Show the user what Buuz will commit before the commit.

## Proposals, ranked

All five are smart-mode only. Classic mode output does not change.

### 1. Personal word choice

- **What:** The user's own use counts break a tie in word choice. Today
  `uruu` stays уруу, because уруу (399) and өрөө (333) are both common. A
  user who often commits өрөө should get өрөө.
- **Cost:** Per-word path. `choose()` in `core/src/autocorrect.rs` reads only
  the added and rejected words. It needs a lookup by word.
- **Test first:** In `core/tests/word_choice.rs`, add
  `user_uses_break_a_word_choice_tie`. Give a snapshot with `use өрөө 5`.
  Type `uruu` and Space. Expect the commit өрөө. With no snapshot, expect the commit уруу.

### 2. Hide a ghost

- **What:** A wrong ghost goes away for good. Completion skips a rejected
  list word. A key hides the ghost that shows.
- **Cost:** One set lookup in `refresh_ghost` in `core/src/composer.rs`.
  Measure it against the per-key budget.
- **Test first:** In `core/tests/completion.rs`, add
  `rejected_list_word_shows_no_ghost`. Give a snapshot with
  `reject баярлалаа`. Type `bayarl`. Expect `completion()` is `None`.

### 3. Alternatives on demand

- **What:** A small list, opened only when the user asks. It shows up to 3
  forms that word choice considered. Nothing shows while the user types
  normally.
- **Cost:** Per-word path, on demand. It reuses the word choice search.
- **Test first:** In `core/tests/word_choice.rs`, add
  `alternatives_list_both_forms`. Type `uruu`. Expect `alternatives()` to be
  `["уруу", "өрөө"]`.

### 4. Undo after punctuation

- **What:** Backspace right after punctuation that ended a changed word
  undoes the change. Today only Space arms the undo.
- **Cost:** Per-word path. It uses the same `delete_before` mechanism.
- **Test first:** In `core/tests/abbrev.rs`, add
  `backspace_after_punctuation_undoes_expansion`. Type `bn.` and Backspace.
  The exact result is an open question.

### 5. Learn words from the user's own text

- **What:** The user pastes their own Cyrillic text in the settings. The core
  counts the words that are not in the word list. The user confirms each
  proposed word. This helps names and chat words.
- **Cost:** Offline, started by the user. Buuz keeps only the confirmed
  entries.
- **Test first:** In `core/tests/userdict.rs`, add
  `words_from_text_proposes_unknown_words`. Use a text with an unlisted word
  twice, plus `сайн`. Expect only the unlisted word, with count 2.

## Not proposed now

- Multi-word smart toggle (Korean study): look at it after per-app memory.
- Per-app memory: shell work. The 78% finding in the Korean study makes it
  more important.
- Reconversion, next-word prediction, emoji: see the table.
- Cloud anything: never.

## Open questions for the maintainer

1. Proposal 1: may use counts also beat a clear list winner, or only break a
   tie?
2. Proposal 1: how many uses make a user word count? Completion uses 3.
3. Proposal 2: reuse `reject`, or add a new entry kind such as `hide`? Which
   key hides the ghost?
4. Proposal 3: which key opens the list? Do number keys pick, given that chat
   uses digits?
5. Proposal 4: does one Backspace after `bn.` undo the expansion and drop the
   `.`? Does Enter get the same undo?
6. Proposal 5: web only first, or desktop too? What minimum count makes a
   word a proposal?

## Sources

- [Mozc `config.proto`](https://raw.githubusercontent.com/google/mozc/master/src/protocol/config.proto)
- [Mozc user dictionary importer](https://code.googlesource.com/mozc/+/HEAD/src/dictionary/user_dictionary_importer.h)
- [Mozilla bug 1176959: Kakutei-Undo of Mozc](https://bugzilla.mozilla.org/show_bug.cgi?id=1176959)
- [Varnam: learning words](https://varnamproject.github.io/docs/learning)
- [Google Research: the machine intelligence behind Gboard](https://research.google/blog/the-machine-intelligence-behind-gboard/)
- [Microsoft Indic phonetic keyboards](https://news.microsoft.com/source/asia/2018/12/10/microsoft-indic-phonetic-keyboards-10-languages-windows-insider-program/)
- [Citizen Lab: keyboard apps](https://citizenlab.ca/topic/keyboard-apps/)
- [CHI 2019: mode errors in a Korean-English keyboard](https://www.springerprofessional.de/doi/10.1145/3290605.3300255)
