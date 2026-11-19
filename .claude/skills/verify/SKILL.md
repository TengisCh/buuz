---
name: verify
description: Run the same checks CI runs (Rust core, web build and tests, Markdown lint) for the parts of Buuz a change touched. Use before saying a change is done, before committing, or when asked to check, test or verify.
---

# Verify a change like CI does

1. See what changed: `git diff --name-only origin/main...HEAD` plus
   `git status --short`.
2. Run only the groups that apply, in this order, and stop at the first
   failure to fix it:

   | Changed paths | Run |
   | --- | --- |
   | `core/**` | `cd core && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test --release --locked` |
   | `core/**`, `web/**` | `cd web/site && npm ci && npm run build && npm test` |
   | `**/*.md`, `tools/lint/**` | `cd tools/lint && npm ci && cd ../.. && tools/lint/node_modules/.bin/markdownlint-cli2` |
   | Mermaid blocks in Markdown | `cd tools/lint && npx --no-install playwright install chromium && node check-mermaid.mjs` |
   | `**/package*.json` | `npm audit` in that folder; report anything moderate or worse |

3. `cargo fmt --check` failing is fixed with `cargo fmt`, never by hand.
4. Report what ran and the result of each group. Say plainly which groups you
   skipped and why. Do not call the change verified if any group failed.
