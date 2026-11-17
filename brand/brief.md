<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Buuz 2.0 brand brief (v2, first principles)

## 1. What buuz means to Mongolians

- **The food.** Buuz (бууз) are steamed dumplings, usually minced mutton or beef with onion, garlic and salt, wrapped in a thin wheat dough. The name comes from Chinese *baozi* by way of Buryat/Mongolian. They are eaten by hand, and the first bite is a careful one: you nibble or tilt it to sip the hot juice (шүүс) before eating the rest.
- **The shape.** A Mongolian buuz is round and soft with its pleats pinched in a spiral toward the top, and the top is **left open**: a small steam hole where you can see the filling. That open top is what tells a buuz apart from a closed bun. It is the single most recognisable detail, so the icon keeps it.
- **Tsagaan Sar.** At Lunar New Year (Tsagaan Sar, the "White Month") households make buuz by the hundreds or thousands in the days before, often freezing them outside in the cold. Making them is a family job: elders roll and fill, kids learn to pleat and get teased for lopsided ones. Guests visiting during the holiday are served buuz in every home.
- **Every day.** Buuz are also ordinary comfort food: the гуанз (canteen) lunch, the thing grandma makes, the smell of steam on a winter day.
- **What it feels like:** home, warmth in the cold, something made together by hand, a little bit of a joke about how many you can eat.

## 2. What the product does

Buuz 2.0 lets you type Mongolian Cyrillic phonetically on an ordinary US/Latin keyboard: `sain baina uu` → `сайн байна уу`, `q` → `ө`, `w` → `ү`. It starts as a web converter (type, copy), then becomes a native input method on macOS and Windows. It is fast, offline, open source and collects nothing.

The audience is young Mongolians at home and abroad who chat in Latin script because their keyboard has no Cyrillic layout, or because it is faster.

## 3. From meaning to brand

| From the buuz | Brand principle | Where it shows |
|---|---|---|
| Open steam hole | **Something comes out transformed.** Latin goes in, Cyrillic comes out, like steam from the top. | Steam is the conversion moment: the mascot puffs when text converts and the steam rises from Buuz Buddy's open top. |
| Spiral pleats | **Hand-made, careful, a little imperfect.** | Pleats are the logo's texture on Buuz Buddy's dome. |
| Made together at Tsagaan Sar | **Community and open source.** Built by and for Mongolians, a continuation of Odbayar Nyamtseren's original Buuz. | Warm, inclusive copy; credit the original in the About screen. |
| Comfort food | **Soft, warm, friendly, never corporate.** | Cream canvas, rounded type, chunky "pressed" buttons. |
| Blue khadag, Eternal Blue Sky | **One trusted, cool colour for the action.** | Khadag Blue is the only colour for the primary button and links. |
| Typing | **It's a keyboard tool.** | Key hints in JetBrains Mono (q → ө, w → ү). |

The tone is Bun.js-like in spirit (a cute food mascot for a fast developer-grade tool) but nothing is borrowed: our shape is a pleated Mongolian buuz with an open top, our palette is warm dough and ember, and our hook is steam turning into Cyrillic.

## 4. Size rules

- 128 px and up: full icon with mascot, steam and shading.
- 32 px and below: the face and steam vanish, so `icons/buuz-buddy-small.svg` is a simplified buuz with bigger eyes. Use that for favicons, the Windows tray and the macOS menu bar.
- The macOS menu-bar icon will need a single-colour template version of the small glyph; it is drawn with the macOS app.
