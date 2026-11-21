#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# PreToolUse hook for Edit, Write and MultiEdit. It enforces
# .claude/project-focus.json: a file under a paused path (windows/) cannot be
# edited. It reads the hook JSON on stdin. It allows the call (exit 0, no
# output) or prints a deny decision. If jq or the config is missing, it allows.
set -u
command -v jq >/dev/null 2>&1 || exit 0
root="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "$0")/../.." && pwd)}"
cfg="$root/.claude/project-focus.json"
[ -f "$cfg" ] || exit 0
input=$(cat)
file=$(printf '%s' "$input" | jq -r '.tool_input.file_path // ""')
[ -n "$file" ] || exit 0
rel="${file#"$root"/}"

# The focus file itself is the maintainer's. Agents never edit it.
case "$rel" in
  .claude/project-focus.json|.claude/hooks/enforce-focus.sh)
    jq -n '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:"Focus: only the maintainer edits .claude/project-focus.json and the focus hook."}}'
    exit 0 ;;
esac

while IFS= read -r ok; do
  [ -n "$ok" ] && [ "$rel" = "$ok" ] && exit 0
done < <(jq -r '.allowed_in_paused[]?' "$cfg")

while IFS= read -r p; do
  [ -n "$p" ] || continue
  case "$rel" in
    "$p"*)
      msg=$(jq -r '.message' "$cfg")
      jq -n --arg r "Focus: $msg ($rel)" '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:$r}}'
      exit 0 ;;
  esac
done < <(jq -r '.paused_paths[]?' "$cfg")
exit 0
