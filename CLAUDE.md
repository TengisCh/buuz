# Claude Code instructions

@AGENTS.md

## Claude Code specifics

- Shared settings are in `.claude/settings.json` (permissions, a 400k-token
  auto-compaction window). Personal overrides go in
  `.claude/settings.local.json`, which is git-ignored.
- Project skills are in `.claude/skills/`: `verify`, `fix-typing-bug`,
  `add-dependency`, `new-skill`. Use them instead of improvising those flows.
- Dependency installs (`npm install`, `npx`, `cargo add`, `cargo install`)
  ask for approval on purpose. Follow `add-dependency` before asking.
- Long sessions compact at about 400k tokens. Before a big compaction-prone
  task, write the plan into the PR description or an issue comment, not only
  into the chat, so it survives the summary.
