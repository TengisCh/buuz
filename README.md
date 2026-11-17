# Buuz 2.0

**Type Mongolian Cyrillic with a Latin keyboard.** Buuz is a phonetic input
method: you type how a word sounds in Latin letters and get Cyrillic as you
type.

```
buuz id'ye        → бууз идье
sain baina uu     → сайн байна уу
```

Buuz 2.0 is a community continuation of the original
[Buuz IME](https://github.com/odbayar/buuz) by Odbayar Nyamtseren (Windows,
last released 2015). It keeps the original typing rules so existing users feel
at home, and rebuilds the engine so it can run natively on today's systems.

> **Status: early development.** The shared transliteration core is done and
> tested. The Windows and macOS input methods and the web converter are
> planned; see the [architecture and roadmap](docs/architecture.md).

## What is in this repository

| Path | What it is | Status |
| --- | --- | --- |
| [`core/`](core/) | `buuz-core`, the Rust transliteration engine shared by every platform | Working, tested |
| [`docs/architecture.md`](docs/architecture.md) | Long-term architecture and roadmap | Current plan |
| [`docs/research.md`](docs/research.md) | Name, license and cost research | Reference |
| `platforms/windows/` | Windows input method (Text Services Framework) | Planned |
| `platforms/macos/` | macOS input method (InputMethodKit) | Planned |
| `web/` | Browser converter: type Latin, copy Cyrillic | Planned |

## Principles

- **Native and instant.** Every keystroke is handled by compiled native code
  on your device. No lag, no web views in the typing path.
- **Offline and private.** The input method never touches the network and
  never stores what you type.
- **Faithful.** The rules match the original Buuz, including vowel harmony
  (`o`/`u` become `ө`/`ү` after front vowels).
- **Free and open.** The core and basic Windows and macOS typing are open
  source under Apache 2.0.

## Building the core

```sh
cd core
cargo test
```

Requires a stable Rust toolchain (1.70 or newer).

## Contributing

Bug reports with the exact Latin input, what you expected and what you got are
the most useful thing you can send. See [CONTRIBUTING.md](CONTRIBUTING.md).
Security issues: see [SECURITY.md](SECURITY.md).

## License

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE). Portions are
derived from the original Buuz, Copyright 2009 Odbayar Nyamtseren.

"Buuz 2.0" is an independent continuation and is not affiliated with or
endorsed by the original author unless stated otherwise.
