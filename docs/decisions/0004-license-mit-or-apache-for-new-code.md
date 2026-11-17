# 0004: MIT OR Apache-2.0 for new code, Apache-2.0 for the ported core

Status: Accepted
Date: 2026-11-17
Amends: [0002](0002-apache-2-open-core.md)

## Context

We wanted a popular open source license that keeps typing free and trusted,
and that never gets in the way of paid add-ons kept in a separate private
repository. One constraint is fixed: `core/` is ported from the original Buuz
(rule table, vowel harmony and composition logic), which is Apache-2.0. We
cannot relicense someone else's code, so that part stays Apache-2.0 unless the
original author agrees otherwise.

## Options considered

| License | Popularity | What others may do | Patent grant | Fits paid add-ons | Fits app stores |
| --- | --- | --- | --- | --- | --- |
| **MIT** | Most used on GitHub | Anything, keep the notice | No explicit grant | Yes | Yes |
| **Apache-2.0** | Second most used; standard for Android, Kubernetes, Swift | Anything, keep notice and NOTICE file, mark changes | Yes, plus patent retaliation | Yes | Yes |
| **MIT OR Apache-2.0** | The Rust ecosystem default | User picks either | Available via Apache side | Yes | Yes |
| **MPL-2.0** | Moderate (Firefox) | Changes to our files must stay open; can combine with closed code | Yes | Yes, if add-ons are separate files | Yes |
| **GPL-3.0** | Common for apps | Anything that includes it must be GPL too | Yes | Only if we own all copyright and dual-license; blocks SDK customers | Conflicts with Apple App Store terms |
| **AGPL-3.0** | Niche, mostly servers | Like GPL, also for network use | Yes | Same problems as GPL; scares enterprises | Same as GPL |

Copyleft licenses (MPL, GPL, AGPL) would stop a competitor from shipping a
closed fork, but they make embedding and selling an SDK harder, and GPL
conflicts with the Mac and iOS App Stores. For an input method, where trust
and wide adoption matter more than blocking forks, a permissive license fits
better.

## Decision

- **New code** in this repository (FFI layer, WebAssembly wrapper, Windows and
  macOS input methods, website, tools) is licensed **MIT OR Apache-2.0**, at the
  user's option. Root files: `LICENSE-MIT`, `LICENSE-APACHE`, and `LICENSE`
  explaining the split.
- **`core/`** stays **Apache-2.0 only**, with `core/LICENSE` and `core/NOTICE`
  crediting Odbayar Nyamtseren. Its `Cargo.toml` says `license = "Apache-2.0"`.
  Calling the whole crate dual-licensed would be inaccurate, because nearly all
  of it is ported code.
- If the original author agrees to MIT for his code, `core/` can move to
  MIT OR Apache-2.0 too. The courtesy email suggested in `docs/research.md` is a
  good moment to ask.

## Consequences

- **Paid add-ons are not affected.** Both licenses are permissive. A closed,
  paid product may link to `buuz-core` and the other crates, as long as it
  ships the license texts and the `NOTICE` file. Neither license requires the
  add-on's own code to be open.
- Anyone, including competitors, may also build closed products on this code.
  We accept that; our advantages are the brand, quality and the paid extras.
- Shipped apps must include `LICENSE-APACHE`, `LICENSE-MIT` and `NOTICE` (for
  example in an "About" or "Licenses" screen).
- Contributions to `core/` are accepted under Apache-2.0; elsewhere under
  MIT OR Apache-2.0 (see `CONTRIBUTING.md`).
- New source files carry an SPDX header: `SPDX-License-Identifier: Apache-2.0`
  inside `core/`, `SPDX-License-Identifier: MIT OR Apache-2.0` elsewhere.
