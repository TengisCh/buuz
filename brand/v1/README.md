<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Buuz 2.0 brand concepts

Original, hand-built SVG dumpling mascots (not derived from any other logo). Status: **concepts, awaiting a pick**.

- `concepts/` — four icon concepts, each as `-icon.svg` (app-icon tile) and `-mark.svg` (transparent mascot). Open `concepts/index.html` for the size sheet.
- `ui-mock.html` — look-and-feel mock of the web converter in the recommended direction.
- `generate-concepts.py` — regenerates the SVGs (`python3 generate-concepts.py concepts`).

## Concepts

| # | Name | Idea |
|---|------|------|
| 1 | Classic Buuz | Cream dumpling, orange tile, steam, small face. The mascot. **Recommended.** |
| 2 | Steamer Pal | Dumpling peeking out of a bamboo steamer, blue tile. Most story, busier at 16px. |
| 3 | Cyrillic Б | No face, Б on the belly, deep blue tile. Most grown-up, ties to the IME. |
| 4 | Jelly Pop | Pink clay look, sparkly eyes, lilac tile. Most kawaii. |

## Recommendation

Use **1 (Classic Buuz)** as the mascot and app icon, and borrow **3's Б** as a small-size and tray/menu-bar glyph (a face is unreadable at 16px, a Б is not). Keep the dumpling's pleated top: it is the buuz silhouette and survives small sizes.

## Palette

| Token | Hex | Use |
|-------|-----|-----|
| Cream | `#FFF6E8` | page background |
| Dough | `#FFE9C7` | cards, chips, mascot body |
| Steam Orange | `#FF6A45` | primary brand, highlights |
| Amber | `#FFB35C` | gradient start, warm accents |
| Khökh Blue | `#2F4BFF` | primary buttons, links (nod to Mongolian "sky blue") |
| Ink | `#2B1B17` | text (warm near-black, never pure black) |
| Blush | `#FF8FA3` | cheeks, small delights |
| Muted | `#8A6F60` | secondary text |

Dark mode: Ink `#1B1410` background, Dough text, same orange and blue.

## Typography

- Display and UI: **Nunito** (900 for headlines, 700 for UI). Friendly rounded forms, full Cyrillic support, OFL licensed.
- Alternative display: **Baloo 2** for a chunkier, more playful headline.
- Mono (code, key hints): **JetBrains Mono**.
- All three are open source and cover Mongolian Cyrillic (ө, ү included). Verify before shipping.

## Look and feel

- Soft, chunky and tactile: big radii (24-32px), a solid "pressed" bottom edge on buttons and cards instead of blurry shadows.
- Warm cream canvas, one orange hero surface (the Cyrillic output), blue for the one primary action.
- Mascot reacts: blinks while idle, steam puffs when text converts, a happy bounce on copy.
- Playful copy, plain behaviour: instant, no sign-up, text stays on the device.
- Never copy Bun's look: ours is a pleated Mongolian buuz, warm palette, no shared shapes.

## Open questions for the owner

1. Which concept (or mix)?
2. Wordmark: lowercase `buuz` as in the mock, or `Buuz 2.0`?
3. Should the Mongolian script (Бууз) appear next to the Latin name in the logo lockup?
