<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Layout audit: Wispr Flow and the Buuz mockup

The maintainer said the mockup is too cramped. This file measures how Wispr
Flow uses space, lists every gap in the old mockup, and gives the fix. The
fixes are in [`mockup.html`](mockup.html) and [`spec.md`](spec.md).

## 1. Reference screenshots

The screenshots are at 2× scale. All numbers below are points (pixels ÷ 2).

| File | What it shows |
| --- | --- |
| `1.png` | The current Buuz settings app (760 × 540, not Wispr) |
| `2.webp` | Wispr Home (Dictation) |
| `3.webp` | **The maintainer's screenshot of the cramped Buuz Shortcuts page** (old mockup) |
| `4.webp` | Wispr Notetaker, with an empty state and a detail pane |
| `5.webp` | Wispr Insights |
| `6.webp` | Wispr Dictionary |
| `7.webp` | Wispr Style |
| `8.webp` | Wispr Transforms |
| `9.webp` | Wispr Scratchpad |
| `10.webp` | Wispr Settings › General, the full window with the sheet |
| `11.png` | Wispr Settings › System (crop, zoomed about 1.36×) |
| `12.png` | Wispr Settings › Connectors (crop, zoomed about 1.36×) |
| `13.png` | Wispr Settings › Account (crop, zoomed about 1.36×) |

## 2. How Wispr uses space

Window in `2.webp` to `10.webp`: 1000 × 634.

| Page | Measurements | What we take |
| --- | --- | --- |
| Shell (all) | Top strip 40. Sidebar 158 wide. Content panel 823 wide, radius about 16, inset 6 from the right and bottom edges. | Inset panel on a frame; a short top strip |
| Sidebar (all) | Item pitch 29, selected item 26 high, radius 8, icon 16. Setup card 140 wide, 171 high, radius 12, a 4 pt progress bar, four rows 25 apart. Footer items after a hairline. | 32 high items, a compact setup card with a thin bar |
| Home `2.webp` | Title 40 below the panel top. Two columns: main 563, side 181, gap 19, side gutter 30. Hero 133 high, radius 12, one title, one line, one button 26 high. History list rows 39 high, hairline separators. | Hero height under 167. **Not** the side column: at our panel width it is the cause of the overlap |
| Notetaker `4.webp` | Empty state card 173 high: icon, one line, one button, all centred. Tabs with a 2 pt underline. List rows 47 apart. | Empty and error states as calm centred blocks |
| Insights `5.webp` | Stat cards in a row of 3, gap 18, padding 12, radius 8. | A row of equal tiles under the hero |
| Dictionary `6.webp` | **Centred column 586 wide in an 823 panel: 119 gutters each side (14.5 %).** Title to tabs 60. Banner 161 high. Rows 39 high with hairlines. | The centred max-width column |
| Style `7.webp` | Column 585, centred. Tabs, banner 95 high, three cards in a row. | Same column on every page |
| Transforms `8.webp` | Column 590. Setting row: title and description on the left, control on the right. Section heading 20 pt serif. | Settings rows with the control at the right |
| Scratchpad `9.webp` | Column 588. Banner 167 high. Note cards: padding 12, radius 12, 24 apart. | Card padding and spacing |
| Settings `10.webp` | **Modal sheet 709 × 472, centred over the whole window, scrim behind.** Left list 154 wide with eyebrows "SETTINGS" and "ACCOUNT", rows 26.5 apart, version at the bottom. Content padding 36. Title 46 below the sheet top. One card, rows 64 high, hairlines, 15 side padding. Buttons 121 × 26 at the right. | The settings sheet with its own list |
| System `11.png` | Section titles in sentence case above each card. Rows about 48 high with a switch at the right. | Sentence-case section titles |
| Connectors `12.png` | One card for each item, about 62 high, 12 apart, button at the right. | Not used: a group with hairlines is calmer for us |
| Account `13.png` | Rows with a text field at the right. Buttons under the card: Sign out, Delete (text), Save at the right. | Buttons under a group, not in it |

Wispr rules in short:

1. One centred column, about 71 % of the panel. Side gutters are 14 % each.
2. Every page has one column. Only Home and Notetaker have a side column,
   and both are at least 180 wide.
3. Cards have no border in dark mode and no shadow. Separators are
   hairlines inside one card.
4. Each control sits at the right end of its row.
5. One accent colour for state. Everything else is gray.
6. Rare things (settings) live in a sheet, not in the sidebar.

## 3. Gaps in the old mockup

I rendered every screen and state at 1000 × 680 and 880 × 600, Light and
Dark, with headless Chrome. "Fix" says what the new mockup does.

### All pages

| Gap | Fix |
| --- | --- |
| Side gutters 32 at every size. Content not centred. | Gutters `max(48, 9 % of panel)`, column max 720, centred |
| Two columns on Home, Typing, Shortcuts, Help, About, with a 240 to 248 side card | One column on every page |
| Container queries changed the layout at odd widths, and the side card kept its width while the main column shrank | No side cards, so nothing to collapse |
| Page clipped at the bottom by the panel radius (32 bottom padding) | 56 bottom padding |
| Borders and shadows on every card, coloured tiles on every settings row | `fill` cards, no border, no shadow, no tiles |
| Five accent colours (blue, ember, amber, green, violet) | One accent; ember only for the brand |
| Uppercase eyebrows over every section | Sentence-case section titles, 13 semibold `text2` |
| Title 26, section gap 16 to 22 | Title 24, section gap 32 |

### Sidebar

| Gap | Fix |
| --- | --- |
| Setup card 180 high with a border, a shadow and a gradient bar; title "Buuz-ийг тохируулах" next to "3/4" | Card without border, 4 pt `accent` bar, 26 pt rows, collapsible; title shortened to "Тохируулга" |
| Row "Хичээлүүдээ дуусгах" wrapped to two lines | Short labels: "Солих товч", "Buuz руу шилжих", "Хичээлүүд" |
| Seven nav items plus the card crowd a 600 pt window | Home and Lessons in the main nav, Settings and Help at the bottom; settings pages moved to a sheet |

### Home

| Gap | Fix |
| --- | --- |
| Hero 196 to 250 high, with an eyebrow, dots and a 150 pt Buddy; title wrapped to two lines | Hero 152 high, no eyebrow, no dots, Buddy 122 and clipped |
| Side column with three cards (status, mode, tip) | A row of three tiles under the hero: keyboard, switch key, mode |
| Tip card repeated the hero tip | Removed |
| "smart" pills on cheat sheet rows competed with the text | Removed; smart rows hide when off |

### Shortcuts (`3.webp`)

| Gap | Fix |
| --- | --- |
| "Test your key" card floated at the right and covered the Control + Space card | The test is a row in the column, with the two chips at the right |
| 2 × 2 option cards: "Шинэ Mac-д. macOS-д тохируулна." wrapped in a 150 pt column | Four full-width radio rows: radio, title, note, keycaps at the right |
| "Товчлол бичүүлэхийн тулд дарна уу" pill as keycaps | The note says it; the keycaps column is empty until recorded |
| Info banner squeezed the button next to three lines of text | Button row under the text |
| Conflict state showed the keys twice (recorder and option) | Keys only in the option row, then the warn banner |
| "Keys while you type" table put keycaps on the left | Action at the left, keycaps at the right |

### Typing, Privacy, General, About, Help

| Gap | Fix |
| --- | --- |
| Typing preview card at the right | Preview section in the column, before "Coming soon" |
| Privacy promises in 2 × 2 cards | One group of four rows |
| General content left-aligned at 720 in a wider panel | Lives in the settings sheet, one column |
| About: two columns with a tall logo card | Wordmark centred above one group |
| Help mixed the lesson list, the keyboard fix and feedback in two columns | Lessons is its own page; Help has the fix card and one group |

### Tutorial

| Gap | Fix |
| --- | --- |
| Left-aligned header with lesson dots and a chip at the right | Centred header; progress is in the rail and footer |
| Task card with two eyebrows and a "why" row inside | One centred task strip, the "why" line under it |
| Practice card with its own toolbar and eyebrow around a 76 pt field | One large field (84 high, 26 pt) and a tool row under it |
| Step 2 promises 2 × 2, step 5 two-column checks plus a header chip | Rows in one group; the chip is gone (the tiles show "NOW") |
| Done: three recap cards in a row | One group of three rows; confetti behind the content |
| At 880 × 600 several lessons and step 4 needed a scroll bar | Compact rules under 570 and 520 pt body height; all steps fit |

### Checked and found fine

Keyboard navigation (← → in the mockup), the size switcher, both themes,
both languages, and the rail at 600 pt height (no clipping).

## 4. Settings: sheet or pages?

**Decision: a sheet, like Wispr.** Settings in the sidebar opens a modal
sheet with its own list: General, Typing, Shortcuts, Privacy, About. The
main window keeps Home, Lessons and Help.

| | Sheet | Pages in the sidebar |
| --- | --- | --- |
| Sidebar | 4 items, room for the setup card | 7 items; the setup card crowds them at 600 pt |
| Width for a page | 504 (sheet column at both sizes) | 548 to 646 |
| Context | Home stays visible behind the scrim | The user leaves Home |
| macOS habit | Matches ⌘, settings | Needs a second "Settings" item anyway |
| Cost | One extra view (`SettingsSheet`) | None |

The sheet column is slightly narrower than a page column, but every
settings page is a list of rows, which fits 504 well. The status tiles on
Home open the sheet on the right page, so Typing and Shortcuts are still
one click away.

## 5. Final numbers

| Thing | 1000 × 680 | 880 × 600 |
| --- | --- | --- |
| Sidebar | 200 | 200 |
| Panel | 788 × 624 | 668 × 544 |
| Page gutters | 71 | 60 |
| Page column | 646 (max 720) | 548 |
| Settings sheet | 760 × 560 | 760 × 544 |
| Sheet list / column | 184 / 504 | 184 / 504 |
| Tutorial column | 580 (max) | 562 |

Spacing: section gap 32 (sheet 28), section title to card 12, card padding
20, settings row min 60, option row 64, cheat row 52, hero 152, button 32
(large 38), sidebar row 32, setup row 26. The full list is in `spec.md` §3.3.

## 6. Screenshots

In [`shots/`](shots/), named `<screen>-<theme>-<size>.png`: Home, Settings
› Shortcuts, Settings › General and Lesson 1 (wrong keyboard), each in Light
and Dark at 1000 × 680 and 880 × 600.
