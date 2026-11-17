---
name: triage-issue
description: Label and sort new Buuz issues, including feedback-form issues. Use when asked to triage, label or go through new issues, or when a new feedback issue arrives.
model: haiku
---

# Triage issues

1. List untriaged issues: `gh issue list --search "no:label OR label:feedback -label:priority:high -label:priority:medium -label:priority:low"`.
2. For each, read it (`gh issue view <n>`) and propose labels from
   `.github/labels.yml` only: one type (`bug`, `typing-rules`,
   `enhancement`, `feedback`, `thanks`, `collaboration`, `documentation`,
   `question`, `research`), areas (`area:*`), and for planned work a
   priority. Add `needs-design` if behaviour or settings are undecided,
   `good first issue` if it is a small, well-specified change.
3. A typing bug reported as feedback becomes `typing-rules` with the Latin
   input and expected output copied into the issue body.
4. Show the proposed labels for all issues as one table and wait for the
   maintainer's OK before running `gh issue edit`. Closing an issue, or
   commenting on it, always needs an explicit OK: it notifies the reporter.
5. Link new smart-typing work into epic #13.
