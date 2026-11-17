---
name: grill
description: Interrogate a Buuz feature idea, issue or plan one hard question at a time until the design is unambiguous, before any code is written. Use when an issue is labelled needs-design, when the maintainer says "grill me" or "poke holes in this", or at the start of plan-issue.
model: opus
---

# Grill the design before building it

Goal: reach a shared understanding with no hidden assumptions. You ask,
the maintainer answers. Ask **one question at a time**, wait for the
answer, then pick the next question from what you learned. Offer your
recommended answer with each question so the maintainer can just say yes.

Before asking anything, read the issue (`gh issue view <n> --comments`),
`docs/glossary.md`, and the code it touches. Do not ask what the code or
docs already answer.

Walk these branches, skipping the ones that do not apply:

1. **Behaviour.** Exact Latin keys in, exact Cyrillic out, for the happy
   path and three edge cases (upper case, word boundary, punctuation).
   Never guess Mongolian spelling.
2. **Classic mode.** Is the change smart-mode only? It must be, unless
   the issue explicitly changes original Buuz behaviour.
3. **Undo.** How does Backspace right after an automatic change restore
   what was typed?
4. **Speed.** Per keystroke (must stay under 10 µs, no allocation) or once
   per word on commit (under 1 ms)?
5. **Privacy.** Does anything leave the device or get stored? It must not
   leave the device.
6. **Settings.** Which toggle controls it, its default, and its name in the
   portable config (#8).
7. **Shells.** Anything web, Windows or macOS must do differently?
8. **Done.** Which tests prove it, and what is explicitly out of scope?

Stop when every branch has an answer or a recorded open question. Then
write the result as a short spec (goal, decisions, tests, out of scope) and
offer to post it as an issue comment, so it survives context compaction.
Record any new domain term in `docs/glossary.md`.
