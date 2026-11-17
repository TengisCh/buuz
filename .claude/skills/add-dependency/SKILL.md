---
name: add-dependency
description: Vet and add or upgrade an npm package or Rust crate in Buuz with supply-chain checks. Use before any npm install <pkg>, cargo add, version bump, or when reviewing a Dependabot PR.
---

# Add or upgrade a dependency safely

Buuz is a keyboard: it sees everything people type. Every dependency is
code that could ship in it, so the default answer is "do we need it?".

1. **Justify it.** Can the standard library, the browser, or 20 lines of our
   own code do this? If yes, stop and do that instead.
2. **Confirm the identity.** Exact package name (watch for typosquats such
   as `react-dom` vs `reactdom`), the source repo link matches the registry
   page, real maintainers, sensible download count and history.
3. **Pick a version at least 7 days old.**
   `npm view <pkg> time --json` or the crates.io versions page. `.npmrc` sets
   `min-release-age=7`; do not override it except for an urgent security fix
   the maintainer asked for.
4. **Read install scripts.** `npm view <pkg>@<v> scripts`. A `preinstall`,
   `install` or `postinstall` script, or a Rust `build.rs` that downloads
   anything, needs the maintainer's OK.
5. **Licence** must be compatible with MIT OR Apache-2.0 (MIT, Apache-2.0,
   BSD, ISC, Zlib, Unicode, MPL-2.0 for tools only). No GPL or AGPL.
6. **Install exactly**, from the package folder:
   `npm install --save-exact <pkg>@<version>` (dev tools: `--save-dev`), or
   `cargo add <crate>@=<version>`. Commit the lockfile with it.
7. **Check** `npm audit` / `cargo tree -d`, then run the `verify` skill.
8. **PR text** says why the dependency is needed, what you checked in steps
   2 to 5, and its size impact for anything shipped to users.

Reviewing a Dependabot PR: do steps 2, 4, 5 and 7 for every major bump, and
read the changelog for breaking changes. Lockfile-only patch bumps need only
step 7.
