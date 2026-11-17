# Buuz 2.0: name, license and cost research

Status: draft for review. Not legal advice. Researched 2026-11-17.

## Summary and recommendations

1. **Original project.** Buuz is a Windows IME written in C++ by Odbayar Nyamtseren (copyright 2009-2012), published under Apache 2.0 at [odbayar/buuz](https://github.com/odbayar/buuz). The last commit is 2017-11-27 (VS2017 update); the last changelog entry is v1.0.4 (2012-11-27).
2. **License.** Apache 2.0 lets us fork, modify, rewrite and commercialize, including under a different license for new code, as long as we keep the notices (see below). It grants no trademark rights.
3. **Name.** I found no registered "Buuz" mark for software in the sources I could reach, but I found one pending US application for BUUZ in software classes (details below) and the word is a common Mongolian food, so it is a weak, shared brand. Recommendation: ship as **"Buuz 2.0"** only as a clearly credited, community continuation for the free open-source IME, and **do not invest in registering "Buuz" as a trademark**. If a paid/enterprise brand is planned, pick a distinctive name for that product (for example "Buuz Pro" is still risky; an unrelated coined name is safer) and run a proper clearance search before spending on a registration.
4. **Before using the name publicly,** email the original author (odbayar.n@gmail.com, per the source headers) to ask for blessing or hand-off. It costs nothing and removes the main reputational risk.
5. **Realistic year-one cost for free open-source desktop distribution: roughly USD 100 to 400**, dominated by the Apple Developer Program fee and a domain. Windows signing can be free (SignPath Foundation for qualifying open-source projects, or Microsoft Store) but eligibility needs checking. A trademark registration would add roughly USD 350+ per class per jurisdiction before any attorney fees.

## 1. Who holds the original project

| Item | Finding |
| --- | --- |
| Author / copyright holder | Odbayar Nyamtseren, per copyright headers in the source (2009, 2009-2012) |
| Original home | code.google.com/p/buuz (exported to GitHub) |
| Canonical repo | [odbayar/buuz](https://github.com/odbayar/buuz), Apache-2.0 |
| Last commit | 2017-11-27, "Let's use VS2017" |
| Last changelog entry | v1.0.4, 2012-11-27 |
| Other copies | Several automatic exports of the Google Code project (for example `tulga/buuz`, `Tseku111/buuz`), also Apache-2.0 |
| Other projects using the name | Several unrelated repos on GitHub use "buuz" (a keyboard repo, a "buuz counter", etc.). None looked like an established commercial product. |

Note: the repo has a `NOTICE` file that only says "Buuz - Input Method Editor / Copyright 2009 Odbayar Nyamtseren". There is no other trademark policy in the repo.

## 2. What Apache 2.0 allows for a revival

- **Allowed:** use, copy, modify, fork, redistribute, relicense *your own* additions, sell binaries or support, and build a closed-source product on top (section 2 grants a perpetual, worldwide, royalty-free license; section 4 sets conditions).
- **Conditions when redistributing (section 4):**
  - include a copy of the Apache 2.0 license text;
  - keep copyright, patent, trademark and attribution notices in files you reuse;
  - mark modified files with prominent notices that you changed them;
  - carry over the contents of the `NOTICE` file in a NOTICE file or in documentation or UI if you ship one, and you may add your own attributions to it.
- **Trademarks (section 6):** the license does *not* let you use the licensor's trade names or marks except to describe the origin of the work. So Apache 2.0 neither grants nor blocks use of the name "Buuz"; it only means the author has not licensed it to us.
- **Rewrites:** if the Rust core is a clean rewrite from the documented mapping rules rather than copied code, it is not a derivative of the C++ code, but the mapping table and docs came from the original, so keep crediting it. Simplest safe path: keep Apache 2.0 for the whole repo and include the original NOTICE.
- **Practical to-do:** add `LICENSE` (Apache 2.0), a `NOTICE` that lists "Original Buuz Copyright 2009-2012 Odbayar Nyamtseren" plus the new copyright line, and a short "Derived from Buuz" note in the README.

## 3. Trademark findings for "Buuz"

What I could check from here (public web only, no paid database access):

- **US:** a [BUUZ application, serial 98523382](https://trademark.justia.com/985/23/buuz-98523382.html), filed 2024-04-28, classes 9 (downloadable middleware for software application integration) and 42 (application service provider featuring API software). Status when I read it: new application, not yet examined, no registration. Owner name was not shown; the correspondent is an individual. This is software-adjacent, so it matters if we sell an SDK/API under the same name in the US. It is not clearly aimed at keyboards or IMEs. Re-check its status before any commercial launch.
- **Mongolia, EU, WIPO:** I could not run a reliable search. These registers need their own databases (Mongolian IPO, EUIPO TMview, WIPO Global Brand Database), and their search pages are interactive. **Open item:** someone should run a search for "Buuz/Бууз" in classes 9, 41 and 42 in TMview and the Mongolian IPO, or pay a local agent for a short clearance report.
- **Weakness of the word:** "buuz" is a common Mongolian steamed dumpling. Marks that are the ordinary word for something are weak for unrelated goods, and in Mongolia a registration for software could still be challenged or narrowly construed. Also the name only means something to Mongolian speakers, which is the target audience anyway.
- **Common-law use:** the original project has used the name since 2009 for exactly this product. That gives its author (not us) the strongest claim, which is another reason to ask them first.

Mongolia notes (from a [law-firm guide](https://ip-coster.com/IPGuides/trademark-mongolia)): examination plus publication, 3-month opposition window, about 9-12 months to registration, valid 10 years from filing and renewable, cancellable for non-use after 5 years. The page does not publish fee amounts.

### Name options

| Option | Pros | Cons |
| --- | --- | --- |
| **Buuz 2.0** (continuation) | Existing recognition among users; matches the plan | Name not owned by us; possible conflict if the pending US application matures; need the author's blessing |
| **Buuz, new mark claimed by us** | Full control if it registers | Weak word; the original author has prior use; cost and risk; not recommended |
| **Distinct new name** (for example a coined word, with "formerly Buuz" in the README) | Registrable, clean for paid products and SDK | Loses name recognition; needs clearance |

Recommended: start as "Buuz 2.0" under Apache 2.0 with the author's blessing, keep the project name flexible in the code (single constant, installer strings), and decide on a distinctive commercial brand only if the paid tier becomes real.

## 4. Cost estimate

Figures are list prices at the time of research and should be rechecked; items marked (verify) came from general knowledge or a page that did not state the price.

| Item | Cost | Notes |
| --- | --- | --- |
| Apple Developer Program (needed for Developer ID signing and notarization on macOS) | about USD 99/year (verify at developer.apple.com/programs) | Required for notarized macOS distribution. |
| Windows signing, option A: [Azure Artifact Signing](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options) | about USD 9.99/month | Organizations in US, Canada, EU, UK; individuals only in US and Canada. May not be available depending on where the publisher is based. |
| Windows signing, option B: SignPath Foundation | free | For qualifying open-source projects; check eligibility. |
| Windows signing, option C: OV certificate | USD 150-300/year | Worldwide; needs hardware token or HSM. EV (USD 400+) no longer bypasses SmartScreen, so not worth it. |
| Windows signing, option D: Microsoft Store (MSIX) | free | Microsoft re-signs; no SmartScreen warning. IMEs may have packaging limits, to be checked in the TSF prototype step. |
| Domain | about USD 10-25/year | Plus optional `.mn` (price varies by registrar, verify). |
| Hosting website and downloads | USD 0 | GitHub Pages and GitHub Releases. |
| CI | USD 0 | GitHub Actions free for public repos, including Windows and macOS runners. |
| Trademark (optional) | US: USD 350 per class government fee ([USPTO](https://www.uspto.gov/trademarks/fees-payment-information/summary-2025-trademark-fee-changes)); Mongolia: fees not published on the page I read, agent quote needed | Add attorney or agent fees. Not recommended for "Buuz" at this stage. |
| Clearance search by a local agent (optional) | quote needed | Worth it only before a paid launch. |

**Year-one total for free distribution:** about USD 110 (Apple plus domain) if Windows signing is free via SignPath or the Store; about USD 230-420 if buying an OV certificate or Artifact Signing. Adding one US trademark class would be about USD 350 more, plus fees.

## 5. Open items for the owner

1. Email Odbayar Nyamtseren to ask for blessing, hand-off or a "Buuz 2.0" naming permission.
2. Search TMview and the Mongolian IPO for Buuz/Бууз in classes 9, 41 and 42, and re-check US serial 98523382.
3. Decide the signing path for Windows (depends on where the publisher is registered and whether SignPath accepts the project).
4. Confirm the Apple price on Apple's site when enrolling.

## Sources

- [odbayar/buuz](https://github.com/odbayar/buuz) (license, NOTICE, source headers, readme)
- [Microsoft: Windows code signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
- [Justia: BUUZ, serial 98523382](https://trademark.justia.com/985/23/buuz-98523382.html)
- [USPTO: trademark fee summary](https://www.uspto.gov/trademarks/fees-payment-information/summary-2025-trademark-fee-changes)
- [IP-Coster: Mongolia trademark guide](https://ip-coster.com/IPGuides/trademark-mongolia)
