---
name: new-skill
description: Capture a repeated Buuz workflow or a maintainer correction as a new project skill or an AGENTS.md rule. Use when the same multi-step task comes up a second time, when the maintainer says "always do X this way", or when asked to add or update a skill.
---

# Grow the agent setup over time

Decide where it belongs:

- **A rule** (one line, always true, for example "never log keystrokes"):
  add it to `AGENTS.md` under the right heading. Keep that file short.
- **A procedure** (steps someone follows for a kind of task): a skill.
- **A one-off fact** about the current task: neither. Leave it out.

To add a skill:

1. Check `.claude/skills/` for one to extend instead.
2. Create `.claude/skills/<kebab-name>/SKILL.md` with front matter:
   - `name`: same as the folder.
   - `description`: what it does **and when to use it**, with the words a
     person would actually type ("release", "add a rule", "screenshot"). This
     is all an agent sees when deciding whether to load it.
3. Body: numbered steps, exact commands, the checks that prove it worked,
   and what to report. Under about 60 lines; link to docs rather than copy.
4. Add the skill to the list in `CLAUDE.md`.
5. Commit as `agents: add <name> skill` in its own PR, with one line on the
   situation that prompted it.

Candidates already in sight: cutting a release, adding a site language
string (`web/site/src/i18n.ts`), writing an ADR in `docs/decisions/`,
rendering brand or design previews.
