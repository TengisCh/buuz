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

- New source files carry the Apache 2.0 header used in `core/src/`.

## Licensing of contributions

By submitting a pull request you agree that your contribution is licensed
under the Apache License 2.0 (section 5 of the license). You keep your
copyright. Please only submit code you wrote or have the right to submit.

## Conduct

Be kind and assume good intent. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
