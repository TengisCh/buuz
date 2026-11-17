# Claude Code instructions

@AGENTS.md

## Claude Code specifics

- Shared settings are in `.claude/settings.json` (permissions, a 400k-token
  auto-compaction window). Personal overrides go in
  `.claude/settings.local.json`, which is git-ignored.
- Models: the project default is `opusplan`, which runs Sonnet for normal
  work and switches to Opus in plan mode. Subagents in `.claude/agents/`
  pin their own model: `planner` and `reviewer` (Opus), `implementer`
  (Sonnet), `scout` (Haiku). Skills `grill` (Opus) and `triage-issue`
  (Haiku) do the same. Override for yourself with `/model` or
  `settings.local.json`.
- Project skills are in `.claude/skills/`: `plan-issue`, `grill`, `tdd`,
  `verify`, `fix-typing-bug`, `triage-issue`, `add-dependency`,
  `new-skill`. Use them instead of improvising those flows.
- A `PostToolUse` hook runs `rustfmt` on every `.rs` file Claude edits
  (`.claude/hooks/format-rust.sh`), so `cargo fmt --check` stays green.
- Dependency installs (`npm install`, `npx`, `cargo add`, `cargo install`)
  ask for approval on purpose. Follow `add-dependency` before asking.
- Long sessions compact at about 400k tokens. Before a big compaction-prone
  task, write the plan into the PR description or an issue comment, not only
  into the chat, so it survives the summary.
