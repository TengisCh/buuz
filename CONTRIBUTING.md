# Contributing to Buuz 2.0

Thanks for helping. Buuz is small on purpose, so the bar for changes is
"does it keep typing fast, offline and faithful to the original rules?"

## Reporting a typing bug

Open an issue with:

1. The exact Latin keys you typed (for example `geree`).
2. What you expected (`гэрээ`) and what you got.
3. Platform and version (Windows 11, macOS 15, web).

A rule change can affect many words, so we turn every confirmed bug into a
test in `core/tests/` before fixing it.

## Code changes

- Open an issue first for anything larger than a small fix.
- Keep the typing path free of network calls, logging of keystrokes, and
  heavy allocations. See `docs/architecture.md` for the performance budget.
- Run before pushing:

  ```sh
  cd core
  cargo fmt
  cargo clippy --all-targets -- -D warnings
  cargo test
  ```

- New source files start with an SPDX line: `// SPDX-License-Identifier: Apache-2.0`
  inside `core/`, `// SPDX-License-Identifier: MIT OR Apache-2.0` elsewhere.
  Data files under `data/wordlist/` are CC BY-SA 4.0 and take no code-style
  header; the first line of `mn.txt` carries `SPDX-License-Identifier: CC-BY-SA-4.0`.

## Writing docs

Many readers read English as a second language, so keep docs plain: active
voice, short sentences, one instruction per sentence, and the terms in
`docs/glossary.md`. The full list is under "Writing" in
[AGENTS.md](AGENTS.md).

## Licensing of contributions

Unless you say otherwise, any contribution you submit:

- to `core/` is licensed under the Apache License 2.0, because that code is
  derived from the original Buuz;
- to `data/wordlist/` is licensed under CC BY-SA 4.0, like the Wikipedia text
  it is derived from (see [decision 0006](docs/decisions/0006-word-list-data-licence.md));
- anywhere else is dual-licensed under MIT OR Apache-2.0, as described in
  `LICENSE`, without any additional terms or conditions.

You keep your copyright. Please only submit code you wrote or have the right
to submit.

## Conduct

Be kind and assume good intent. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
