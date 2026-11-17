<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Buuz 2.0 brand (v2)

Status: **four options, awaiting a pick.** The first round (v1) is archived in `v1/`.

- [`brief.md`](brief.md): research on what buuz means and what the app does, and how the brand follows from it.
- `icons/`: each option as `<slug>-icon.svg` (app icon, 128 px and up) and `<slug>-small.svg` (16-32 px glyph).
- `previews/options-sheet.png`: all four options at 300, 128, 64, 32 and 16 px, in a dark tray and a browser tab.
- `previews/brand-sheet.png`: wordmark options, palette, type and the web converter's look.
- `generate.py` writes the SVGs; `render.js` renders the previews (needs Playwright):
  `python3 brand/generate.py brand/icons && node brand/render.js brand/icons brand/previews`

## Options

| | Name | Idea | 16 px |
|---|---|---|---|
| A | Buuz Buddy | The mascot: a real buuz with spiral pleats, open steam hole, happy face, on ember orange. Evolution of v1 #1. | simplified buuz |
| B | Keycap | The product: the buuz sitting on a keyboard key with the legends B and Б. | Б on a key |
| C | Rosette | The mark: a buuz seen from above, spiral pleats around the hole. Calm, geometric, works as a logo anywhere. | rosette |
| D | Steam Б | The magic: steam rising from the buuz becomes a Cyrillic Б. | Б |

Recommendation: **A as the app icon and mascot, with D's Б as the 16 px tray and favicon glyph.** A carries the warmth and cuteness; at 16 px a face is unreadable, while a Б is crisp and says what the app does.

## Palette

| Token | Hex | Use |
|---|---|---|
| Dough | `#FFF3E0` | page, cards |
| Ember | `#FF6A3D` | brand, hero output, mascot tile |
| Amber | `#FFB648` | gradient start, warm accents |
| Khadag Blue | `#2E6BFF` | the one primary action, links |
| Night | `#1E1838` | text; dark-mode background |
| Filling | `#8C4A2B` | tiny accents |
| Blush | `#FF8E8E` | cheeks, small delights |
| Steam | `#FFFFFF` | highlights, motion |

## Type

Nunito (900 headlines, 700 UI), rounded and warm, with full Mongolian Cyrillic (Ө ө Ү ү, checked in the brand sheet). JetBrains Mono for key hints. Both OFL.

## Open questions

1. Which option (or mix)?
2. Wordmark: `buuz`, `Buuz 2.0`, or bilingual `buuz бууз`?
