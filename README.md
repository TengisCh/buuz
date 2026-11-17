<!-- markdownlint-disable MD041 -->
<p align="center">
  <a href="https://buuz.thetengis.com">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="docs/readme/hero-dark.png">
      <img src="docs/readme/hero-light.png" alt="Buuz 2.0: sain baina uu becomes сайн байна уу" width="520">
    </picture>
  </a>
</p>

<p align="center"><b>Type Mongolian Cyrillic on any Latin keyboard.</b><br>You type how it sounds. Buuz turns it into Cyrillic as you go.</p>

<p align="center">
  <a href="https://buuz.thetengis.com"><img src="https://img.shields.io/badge/try_it-buuz.thetengis.com-FFB648?style=flat&logo=googlechrome&logoColor=white" alt="Try it live"></a>
  <a href="https://github.com/TengisCh/buuz/stargazers"><img src="https://img.shields.io/github/stars/TengisCh/buuz?style=flat&logo=github&color=FF6A3D" alt="GitHub stars"></a>
  <a href="https://github.com/TengisCh/buuz/actions/workflows/core.yml"><img src="https://img.shields.io/github/actions/workflow/status/TengisCh/buuz/core.yml?branch=main&label=core%20tests" alt="Core tests"></a>
  <a href="#-license"><img src="https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-2E6BFF" alt="License: MIT or Apache-2.0"></a>
  <a href="https://github.com/sponsors/TengisCh"><img src="https://img.shields.io/badge/sponsor-%E2%99%A5-FF8E8E?style=flat&logo=githubsponsors&logoColor=white" alt="Sponsor"></a>
</p>

<p align="center">
  <img src="docs/readme/demo.webp" alt="Typing sain baina uu, buuz id'ye and chi yuu hiij baina? and getting сайн байна уу, бууз идье and чи юу хийж байна?" width="800">
</p>

## 🚀 Try it

👉 **[buuz.thetengis.com](https://buuz.thetengis.com)**: type, copy, paste anywhere. No sign-up, nothing leaves your device.

[![Windows: coming soon](https://img.shields.io/badge/Windows-coming_soon-0078D4?logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI%2BPHBhdGggZmlsbD0iI2ZmZiIgZD0iTTAgMGgxMS4zNzd2MTEuMzcySDB6TTEyLjYyMyAwSDI0djExLjM3MkgxMi42MjN6TTAgMTIuNjIzaDExLjM3N1YyNEgwek0xMi42MjMgMTIuNjIzSDI0VjI0SDEyLjYyM3oiLz48L3N2Zz4%3D)](#-whats-inside) [![macOS: coming soon](https://img.shields.io/badge/macOS-coming_soon-000000?logo=apple&logoColor=white)](#-whats-inside)

Keyboards for Windows and macOS are on the way, so you can type Mongolian in any app. ⭐ Star the repo to hear when they land.

| Light | Dark |
| --- | --- |
| ![Buuz web converter, light theme](docs/readme/screenshot-light.png) | ![Buuz web converter, dark theme](docs/readme/screenshot-dark.png) |

## ⌨️ How to type

Most letters are what you'd guess (`b` → б, `d` → д, `sain` → сайн). The handful worth learning:

| Vowels | | Y-vowels | | Consonants | | Signs | |
| :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| `e` | э | `ye` | е | `h` | х | `'` | ь |
| `w` | ү | `yo` | ё | `c` | ц | `"` | ъ |
| `q` | ө | `yu` | ю | `ch` | ч | | |
| `y` | ы | `ya` | я | `sh` | ш | | |
| `ii` | ий / ы | | | `j` | ж | | |
| `ii'` | ий | | | | | | |
| | | | | `z` | з | | |

✨ **Vowel harmony:** `o` and `u` pick о/ө and у/ү for you, so `geree` → гэрээ and `gar` → гар. It looks ahead in the word (`unee` → үнээ), writes ы after masculine words (`aaviin` → аавын) and lets `uu` follow the word before it (`irsen uu` → ирсэн үү). Type `ii'` to force ий. All of this can be switched off for the original behaviour.

✂️ **Abbreviations:** short forms expand when you type a space or punctuation after them, so `bn ` → байна, `bnu ` → байна уу and `sn ` → сайн. On the web, deleting that space brings the abbreviation back; the upcoming desktop keyboards undo it with Backspace right after the space. On the web this is a checkbox that turns it off; the core has it off by default.

## 🥟 Why Buuz

- ⚡ **Instant.** Native compiled code on every keystroke. No lag.
- 🔒 **Private.** Works offline and never stores or sends what you type.
- 🥟 **Familiar.** Same rules as the original [Buuz](https://github.com/odbayar/buuz) by Odbayar Nyamtseren, which many Mongolians have typed with since 2009. Buuz 2.0 is its community continuation.
- 💛 **Free and open source.**

## 📦 What's inside

| Path | What it is | Status |
| --- | --- | --- |
| [`core/`](core/) | ![Rust](https://img.shields.io/badge/-Rust-B7410E?logo=rust&logoColor=white) `buuz-core`, the engine every platform shares | ✅ Working, tested |
| [`web/`](web/) | ![WebAssembly](https://img.shields.io/badge/-WebAssembly-654FF0?logo=webassembly&logoColor=white) ![Vite](https://img.shields.io/badge/-Vite-646CFF?logo=vite&logoColor=white) web converter on ![Vercel](https://img.shields.io/badge/-Vercel-000000?logo=vercel&logoColor=white) | ✅ [Live](https://buuz.thetengis.com) |
| `windows/` | ![Windows](https://img.shields.io/badge/Windows-keyboard-0078D4?logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI%2BPHBhdGggZmlsbD0iI2ZmZiIgZD0iTTAgMGgxMS4zNzd2MTEuMzcySDB6TTEyLjYyMyAwSDI0djExLjM3MkgxMi42MjN6TTAgMTIuNjIzaDExLjM3N1YyNEgwek0xMi42MjMgMTIuNjIzSDI0VjI0SDEyLjYyM3oiLz48L3N2Zz4%3D) Text Services Framework | 🛠️ Next |
| `macos/` | ![macOS](https://img.shields.io/badge/macOS-keyboard-000000?logo=apple&logoColor=white) InputMethodKit | 🛠️ Next |
| [`brand/`](brand/) | Mascot, icons, palette | 🎨 |
| [`docs/`](docs/) | [Architecture](docs/architecture.md) and [decisions](docs/decisions/) | 📚 |

Build and test the core:

```sh
cd core && cargo test
```

## 💬 Feedback

[![Open an issue](https://img.shields.io/badge/Open_an_issue-181717?logo=github&logoColor=white)](https://github.com/TengisCh/buuz/issues/new/choose)

Found a wrong conversion, a bug, or have a fun idea? **[Open an issue](https://github.com/TengisCh/buuz/issues/new/choose)**. For a typing bug, the exact Latin you typed, what you expected and what you got is perfect. Pull requests are welcome too; see [CONTRIBUTING.md](CONTRIBUTING.md). Security issues: [SECURITY.md](SECURITY.md).

## 💛 Support Buuz

[![Sponsor on GitHub](https://img.shields.io/badge/Sponsor-EA4AAA?logo=githubsponsors&logoColor=white)](https://github.com/sponsors/TengisCh)

Buuz is free. Donations pay for things like the Apple developer account that lets the macOS keyboard install without warnings. If Buuz saves you time, **[sponsor it](https://github.com/sponsors/TengisCh)** 💛

## 📜 License

New code is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. The transliteration core in [`core/`](core/) is ported from the original Buuz (Copyright 2009 Odbayar Nyamtseren) and is Apache-2.0 only. See [LICENSE](LICENSE), [NOTICE](NOTICE) and [decision 0004](docs/decisions/0004-license-mit-or-apache-for-new-code.md).

"Buuz 2.0" is an independent continuation and is not affiliated with or endorsed by the original author unless stated otherwise.
