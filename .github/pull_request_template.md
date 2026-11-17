<!-- markdownlint-disable-file MD041 -->
## What and why

Closes #

## How it was tested

- [ ] `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` in `core/`
- [ ] New behaviour has a test that failed before the change (test-first)
- [ ] Classic mode output is unchanged (`Options::classic()`)

## Checklist

- [ ] No network access, keystroke logging or new heavy allocation in the typing path
- [ ] New files carry an SPDX license line (Apache-2.0 in `core/`, MIT OR Apache-2.0 elsewhere)
