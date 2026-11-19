# Working on Buuz with a coding agent

Instructions for any AI coding agent (Claude Code, Codex, Cursor, Copilot and
others) working in this repository. Humans: see
[CONTRIBUTING.md](CONTRIBUTING.md). Both say the same thing; this file is the
short, strict version.

## What Buuz is

A phonetic Latin to Mongolian Cyrillic input method (`geree` becomes
`гэрээ`). One Rust core, native shells per platform, a web converter.

| Path | What | Language |
| --- | --- | --- |
| `core/` | Transliteration rules and composition engine. Apache-2.0, ported from the original Buuz | Rust |
| `ffi/` | C ABI around `core/` for the Windows and macOS shells (`include/buuz.h`) | Rust, C |
| `web/wasm/` | WebAssembly wrapper around `core/` | Rust |
| `web/site/` | Website and converter at buuz.thetengis.com (Vite, Vercel) | TypeScript |
| `docs/` | Architecture, typing rules, decisions (ADRs) | Markdown |
| `tools/lint/` | Markdown and Mermaid lint | Node |
| `brand/`, `design/` | Logo, mockups and their render scripts | |

Read [docs/architecture.md](docs/architecture.md) before structural changes,
[docs/typing-rules.md](docs/typing-rules.md) before touching rules, and use
the names in [docs/glossary.md](docs/glossary.md) in code, tests and issues.

## Hard rules

1. **The typing path stays private and fast.** No network calls, no logging of
   keystrokes, no heavy allocation per key. This is a security promise
   ([SECURITY.md](SECURITY.md)), not a preference.
2. **Behaviour changes are test-first.** Every typing bug and every new
   feature starts as a failing test (`tdd` skill). Classic mode
   (`Options::classic()`) stays identical to the original Buuz; new
   behaviour goes into smart mode, on by default, with its own toggle.
3. **SPDX header on new source files.** `// SPDX-License-Identifier: Apache-2.0`
   in `core/`, `// SPDX-License-Identifier: MIT OR Apache-2.0` elsewhere.
   Data files under `data/wordlist/` and `data/benchmark/` are CC BY-SA 4.0
   and take no code-style header; the header lines of `mn.txt` and the
   `.tsv` files carry it.
4. **No new dependency without the add-dependency checklist** (below).
5. **Never** commit secrets or `.env` files, force-push `main`, publish
   packages, or change GitHub or Vercel settings. Ask the maintainer.

## Workflow

Plan, then test-first build, then review. For an issue use the
`plan-issue` skill, which runs these phases:

| Phase | How | Model |
| --- | --- | --- |
| Question the design | `grill` skill (issues labelled `needs-design`, new settings) | Opus |
| Plan | plan mode or the `planner` subagent; post the plan on the issue or PR | Opus |
| Implement | `implementer` subagent, which follows the `tdd` skill (also for review fixes) | Sonnet |
| Review | `reviewer` subagent, then the `verify` skill | Opus |
| Lookups, triage | `scout` subagent, `triage-issue` skill | Haiku |

Small, clear typing bugs skip grill and plan: use `fix-typing-bug`.

## Commands

Run the checks for what you touched before you say a change is done.

```sh
# core/ (also run when web/wasm changes)
cd core && cargo fmt && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked

# ffi/ (also run when core/src changes)
cd ffi && cargo fmt && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked

# web/site/ (builds core to wasm first; needs Rust + wasm32 target)
cd web/site && npm ci && npm run build && npm test

# any Markdown
cd tools/lint && npm ci && cd ../.. && tools/lint/node_modules/.bin/markdownlint-cli2
```

## Supply-chain rules

Dependencies are the easiest way into a keyboard app, so:

- Install from lockfiles only: `npm ci`, `cargo build --locked`. Do not run
  `npm install` or `cargo update` unless the task is a dependency change.
- `.npmrc` refuses npm versions younger than 7 days (`min-release-age=7`).
  Do not bypass it unless the maintainer asks, for a security fix.
- Adding or upgrading a dependency: prefer none; check the package is the
  real one (name, repo, maintainers, downloads), that its version is at least
  7 days old, has no install script you have not read, and has a licence
  compatible with MIT OR Apache-2.0. Pin exact versions. Say why in the PR.
- Never pipe a download into a shell, and never `npx` a package that is not
  already in a lockfile.
- GitHub Actions are pinned to full commit SHAs with a `# vX.Y.Z` comment.
  Keep it that way; Dependabot updates them.
- Dependabot patch and minor updates merge themselves once every other check
  on the PR has passed (`.github/workflows/dependabot-automerge.yml`). Major
  updates stay open for review with the add-dependency checklist.

## Writing

Many readers of this repository read English as a second language. Write
text that people read later (docs, skills, PR descriptions, commit messages,
issue comments) in a subset of ASD-STE100 Simplified Technical English.
Chat replies and the reasoning in ADRs and `grill` sessions are exempt.

- Use the active voice. Put the instruction first: "Run `cargo test`", not
  "Tests should be run".
- Write one instruction per sentence.
- Keep sentences short: at most 20 words in procedures, 25 in descriptions.
  Keep paragraphs to 6 sentences or fewer.
- Give each word one meaning, and use one word for each meaning. Do not
  change words for variety.
- Use the terms in [docs/glossary.md](docs/glossary.md) as the approved
  technical names. Write code names in backticks.
- Use simple words: "use", not "utilize"; "make sure", not "ensure";
  "start", not "commence"; "help", not "facilitate"; "about", not
  "approximately".
- Write lists for steps and tables for comparisons. Do not hide steps in a
  paragraph.
- Leave out filler: no "comprehensive", "robust", "seamless", "leverage",
  and no summary that repeats the text above it.

## Git and PRs

- Branch from `main`: `issue-<n>-<slug>` for an issue, otherwise
  `<area>/<slug>`. Never commit to `main` directly.
- Commit subject: `<area>: <what changed>` in lower case, for example
  `core: accept ö/ü as input for ө/ү`. Areas: `core`, `web`, `docs`,
  `brand`, `design`, `lint`, `ci`, `deps`, `agents`.
- One concern per PR. Link the issue (`Closes #n`). PRs are squash-merged.
- Only commit, push or open a PR when the maintainer asks.

## Growing the agent setup

Project skills live in `.claude/skills/<name>/SKILL.md`. When you notice the
same multi-step workflow for the second time, or the maintainer corrects you
on how something is done here, propose a new skill or a rule in this file
(see the `new-skill` skill). Keep this file short: rules here, procedures in
skills.
