---
name: plan-issue
description: Turn a Buuz GitHub issue into an agreed, test-first plan and then run it with the right model for each phase (Opus plans and reviews, Sonnet implements, Haiku does lookups). Use when asked to work on, pick up, start or implement an issue number.
---

# From issue to merged change

1. **Read.** `gh issue view <n> --comments`. Check dependencies in the epic
   (#13). If a blocking issue is open, say so and stop.
2. **Grill** (Opus). For anything labelled `needs-design`, or that adds a
   setting, run the `grill` skill with the maintainer. Small, clear bugs go
   straight to `fix-typing-bug` instead.
3. **Plan** (Opus). Use the `planner` subagent, or plan mode (the project
   model `opusplan` switches to Opus there). The plan lists failing tests
   first. Post the plan as an issue comment or the PR description before
   coding, so it survives compaction (`CLAUDE.md`).
4. **Branch.** `issue-<n>-<slug>` from an up-to-date `main`.
5. **Implement** (Sonnet). Follow the `tdd` skill step by step, or hand the
   plan to the `implementer` subagent. One concern per PR.
6. **Review** (Opus). Run the `reviewer` subagent on the diff and fix what
   it finds. Then the `verify` skill.
7. **Docs.** Update `docs/typing-rules.md`, `docs/glossary.md` and, for a
   decision, add an ADR in `docs/decisions/`. Tick the box in epic #13.
8. **Ship** only when the maintainer asks: commit `<area>: <what>`, PR
   with `Closes #<n>`.

Use the `scout` subagent (Haiku) for lookups along the way: where a rule
lives, which tests cover a word, what a CI log says.
