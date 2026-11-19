#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# PreToolUse hook for the Agent tool. It enforces .claude/model-routing.json:
#  1. A pinned agent (planner, reviewer, translator, implementer, scout) keeps
#     its model. A different `model` argument is refused.
#  2. Language work (translation, Mongolian wording) runs on Opus. A
#     translation must use the `translator` agent.
#  3. A translation between two non-English languages is refused. It must go
#     through English.
#  4. A general-purpose agent needs an explicit `model`. Design, mockup and
#     review work needs `opus`. Built-in helper agents are exempt.
# Reads the hook JSON on stdin. Allows the call (exit 0, no output) or prints a
# deny decision. If jq or the config is missing, it allows the call.
set -u
command -v jq >/dev/null 2>&1 || exit 0
root="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "$0")/../.." && pwd)}"
cfg="$root/.claude/model-routing.json"
[ -f "$cfg" ] || exit 0
input=$(cat)

deny() {
  jq -n --arg r "$1" '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:$r}}'
  exit 0
}

agent=$(printf '%s' "$input" | jq -r '.tool_input.subagent_type // "general-purpose"')
model=$(printf '%s' "$input" | jq -r '.tool_input.model // ""')
text=$(printf '%s' "$input" | jq -r '[.tool_input.description // "", .tool_input.prompt // ""] | join(" ")' | tr '[:upper:]' '[:lower:]')

pinned=$(jq -r --arg a "$agent" '.pinned_agents[$a] // ""' "$cfg")
lang_model=$(jq -r '.language.model' "$cfg")
trans_agent=$(jq -r '.language.translation_agent' "$cfg")
pivot_rule=$(jq -r '.language.rule' "$cfg")
trans_re=$(jq -r '.language.translation_pattern' "$cfg")
word_re=$(jq -r '.language.wording_pattern' "$cfg")
opus_re=$(jq -r '.opus_work_pattern' "$cfg")

# 1. Pinned agents keep their model.
if [ -n "$pinned" ] && [ -n "$model" ] && [ "$model" != "$pinned" ]; then
  deny "Routing: agent '$agent' is pinned to $pinned in .claude/model-routing.json. Remove the model argument."
fi
effective="${pinned:-$model}"

is_translation=0
printf '%s' "$text" | grep -Eq "$trans_re" && is_translation=1
is_language=$is_translation
printf '%s' "$text" | grep -Eq "$word_re" && is_language=1

# 2. Language work runs on Opus. Translation uses the translator agent.
if [ "$is_translation" = 1 ] && [ "$agent" != "$trans_agent" ]; then
  deny "Routing: translation must use the '$trans_agent' agent (Opus). $pivot_rule"
fi
if [ "$is_language" = 1 ] && [ "$effective" != "$lang_model" ]; then
  deny "Routing: language work must run on $lang_model. Use the '$trans_agent' agent or set model: $lang_model. ($pivot_rule)"
fi

# 3. No direct translation between two non-English languages. Only a clear
#    "X to Y", "X into Y" or "between X and Y" pattern counts.
if [ "$is_translation" = 1 ]; then
  langs=$(jq -r '.language.non_english_languages | join("|")' "$cfg")
  if printf '%s' "$text" | grep -Eq "($langs)[a-z ]{0,12}(to|into|->|→) ($langs)|between ($langs) and ($langs)"; then
    deny "Routing: $pivot_rule Split this into two steps, source to English, then English to the target."
  fi
fi

# 4. A general-purpose agent needs an explicit model. Design and review need
#    Opus. Built-in helper agents (Explore, Plan, ...) are exempt.
if [ -z "$pinned" ]; then
  if [ -z "$model" ] && jq -e --arg a "$agent" '.explicit_model_required_for | index($a)' "$cfg" >/dev/null; then
    deny "Routing: set an explicit model for agent '$agent' (opus for plan, design, review and language work; sonnet for implementation; haiku for lookups). See .claude/model-routing.json."
  fi
  if printf '%s' "$text" | grep -Eq "$opus_re" && [ -n "$model" ] && [ "$model" != "opus" ]; then
    deny "Routing: design, mockup and review work must run on opus."
  fi
fi
exit 0
