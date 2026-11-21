# 0009: Finish the macOS app first, pause Windows

Status: Accepted
Date: 2026-11-21

## Context

Buuz has a macOS app and a Windows app. The first version of the Windows
text service, settings app and installer is on `main` (issue #76). The
maintainer has not yet tested the macOS app in full (issue #84). Work on two
platforms at once spreads review and testing too thin.

## Decision

- The macOS app comes first. Make it correct and polished before more
  Windows work.
- Windows development is paused. Agents do not edit files under `windows/`.
- `core/`, `web/` and the macOS app stay open for work.
- Only the maintainer lifts the pause, by editing `.claude/project-focus.json`.
  An agent never edits that file.

## Enforcement

- `.claude/project-focus.json` lists the paused paths.
- The hook `.claude/hooks/enforce-focus.sh` denies Edit, Write and MultiEdit
  on a paused path. It also denies edits to the focus file and to the hook.
- `AGENTS.md` hard rule 7 repeats the rule for agents that do not use hooks.
- The workflow `.github/workflows/windows.yml` stays editable, so a broken CI
  run can be fixed.

## Consequences

- Issue #76 stays open. The Windows follow-up work waits.
- A Windows security fix needs the maintainer to lift the pause first.
- Shared code in `core/` and `ffi/` must still keep the Windows build green.
  Run the Windows CI before merge when `ffi/` changes.

## Lift the pause

1. The maintainer sets `"paused_paths": []` in `.claude/project-focus.json`.
2. The maintainer changes the status of this decision to
   `Superseded by NNNN`.
