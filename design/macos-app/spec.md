<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Buuz macOS app: build spec

This spec is for the SwiftUI implementer. It replaces the 760 × 540 fixed
window in `macos/Settings/` with a resizable window in the style of Wispr
Flow, and replaces the welcome steps with a hands-on tutorial.

- Mockup: open [`mockup.html`](mockup.html) from a local web server at the
  repository root (it loads the logo from `../../brand/export/`). Use the
  bar at the top to pick each screen, state, theme, language and size.
- Layout audit against Wispr Flow, and the list of fixed gaps:
  [`audit.md`](audit.md). Screenshots: [`shots/`](shots/).
- Product rules: [`docs/desktop-onboarding.md`](../../docs/desktop-onboarding.md).
- Names: [`docs/glossary.md`](../../docs/glossary.md).

Only real features appear in the app. Buuz never counts keystrokes, so there
are no word counts, speeds or streaks. The cards show setup progress, the
current keyboard, the switch key, the lessons and the mode.

## 1. Build order

Do each step as one commit. Write the test first where the step has logic
(`macos/Tests/main.swift`).

1. Tokens and fonts: `macos/Settings/Tokens.swift` with the colours in §3.
   Use the `Theme` enum in `Sources/Core/Theme.swift` (`system`, `light`,
   `dark`) for the choice, stored in `Prefs`.
2. Window: `WindowConfigurator` and the root `AppShell` (§4). Remove
   `.frame(width: 760, height: 540)`.
3. `PageColumn` (§4.1), sidebar, top icon buttons and the content panel
   (§5.1 to §5.3).
4. Home (§6.1) with the setup card, hero, status tiles, try box and cheat
   sheet. Then Lessons and Help (§6.2, §6.3).
5. `SettingsSheet` (§6.4) with General, Typing, Shortcuts, Privacy and About.
6. `InputSource` additions: current keyboard, its name, and the
   selected-source notification (§8.1). Test the pure parts.
7. `HotkeyRecorder` and the conflict table (§8.2). Test the conflict lookup.
8. New `TourStep` flow with resume and skip (§7.1). Test `next`, `back` and
   skip in `main.swift`, like `OnboardingFlow` today.
9. `LessonField` and `LessonChecker` (§8.3). Test every pass condition in
   §7.3 with plain strings.
10. Tutorial screens and lessons (§7.2, §7.3).
11. Strings (§9) in `strings.en.json` and `strings.mn.json`.
12. Run the `verify` skill and the e2e test.

## 2. Decisions

| Topic | Decision |
| --- | --- |
| Look | Warm frame, sidebar on the frame, one large rounded panel for the content. Calm spacing in the style of Wispr Flow. See [`audit.md`](audit.md). |
| Layout | One scroll view and one centred column on every page. No side cards. See the "no overlap" rule in §4.1. |
| Settings | A modal sheet with its own list (General, Typing, Shortcuts, Privacy, About), opened from Settings in the sidebar. The main window keeps Home, Lessons and Help. |
| Accent | One accent colour, `accent`. `ember` is only for the brand (logo, Б glyph, hero glow). `success`, `warn` and `danger` are only for states. |
| Theme | Light, Dark, System. Default stays Light (onboarding doc §2 rule 6). |
| Navigation | Custom `HStack`, not `NavigationSplitView`. The split view adds its own material, toolbar and collapse behaviour that we do not want. |
| Icons | SF Symbols in the new layout. New string keys have no emoji. Old keys that have emoji stay for the menu; the new layout drops the emoji (`noEmoji`). |
| Primary button | Khadag Blue with a 2 pt pressed edge. One primary button for each screen. |
| Lessons | A real text field that goes through the real input method. A lesson passes only when the text has the expected Cyrillic. |
| Switch key | Three macOS options (fn, Control + Space, Caps Lock) and one Buuz option (own shortcut), as full-width radio rows. |
| Tutorial | 5 setup steps, 8 lessons, Done. Skip on every step. Replay from Lessons and Settings. |

## 3. Tokens

### 3.1 Colours

Put the colours in `Tokens.swift` as `Color` values made from
`NSColor(name:dynamicProvider:)`, so they follow the window appearance. Do not
use an asset catalog (the build uses `swiftc`).

| Token | Light | Dark | Use |
| --- | --- | --- | --- |
| `frame` | `#F4EFE8` | `#121019` | Window background, sidebar, settings sheet list |
| `panel` | `#FFFFFF` | `#1B1825` | Content panel, sheet, setup card (light) |
| `panelLine` | `#EAE2D7` | white 7 % | 1 pt outline of the panel only |
| `text` | `#1E1838` | `#F3F0F9` | Main text |
| `text2` | `#645D72` | `#A8A2B7` | Secondary text, hints, section titles |
| `text3` | `#9A93A6` | `#6F6981` | Eyebrows, placeholders, empty circles |
| `fill` | `#F7F3EE` | `#221E2E` | Every card and group, try box, neutral banner |
| `fill2` | `#EFE8DF` | `#2B2639` | Pills, segmented control, Latin key chips, tile hover |
| `hover` | Night 5 % | white 5 % | Sidebar row on hover |
| `selected` | Night 7 % | white 8 % | Selected sidebar row |
| `sep` | `#EAE2D7` | white 7 % | Hairline between rows in a group, footer line |
| `track` | `#E6DED3` | `#2E2940` | Progress track, switch off, segmented control in a group |
| `accent` | `#2E6BFF` | `#4D82FF` | Primary button, links, focus ring, radio, switch on, progress |
| `accentEdge` | `#1D4FD1` | `#2D5FD6` | 2 pt edge under the primary button |
| `accentSoft` | `#EAF0FF` | accent 15 % | Focus halo, info banner, "Recommended" pill |
| `btn2` | `#FFFFFF` + 1 pt `#E3DACE` inside | `#2E2940`, no line | Secondary button |
| `ember` | `#FF6A3D` | `#FF6A3D` | Brand only: Б glyph, update dot, hero glow |
| `amber` | `#FFB648` | `#FFB648` | Hero glow only |
| `success` | `#17915F` | `#43D196` | Pass, done checks |
| `successBg` | `#E5F6EE` | success 12 % | Success banner and halo |
| `warn` | `#A85400` | `#FFC266` | Wrong keyboard, conflict |
| `warnBg` | `#FFF2DF` | amber 11 % | Warning banner |
| `danger` | `#D23B3B` | `#FF7B7B` | Add failed, wrong letters, recording dot |
| `hero` | `#1E1838` | `#0D0B14` | Hero card base (both themes are dark) |
| `scrim` | Night 30 % | black 55 % | Behind the settings sheet |

The hero card draws two radial gradients on `hero`: `ember` at 70 % from the
top right and `amber` at 50 % from the bottom right. A `warnHero` variant
swaps the two colours.

No coloured tiles in settings rows. No shadows on cards. The only shadows
are the window, the sheet and the 2 pt primary button edge.

### 3.2 Type

Use the system font. Use `design: .rounded` for titles, numbers and the
expected output. Use `.monospaced` for Latin keys.

| Style | Font |
| --- | --- |
| `pageTitle` | 24 bold rounded (in the sheet: 22) |
| `tourTitle` | 28 heavy rounded, centred (welcome: 34; body under 520 high: 24) |
| `heroTitle` | 21 heavy rounded |
| `tileValue` | 17 bold rounded |
| `lessonOutput` | 22 bold rounded, `text` |
| `practice` | 26 regular (body under 520 high: 22) |
| `body` | 13.5 regular |
| `rowTitle` | 13.5 semibold |
| `hint` | 12.5 regular, `text2` |
| `sectionTitle` | 13 semibold, `text2`, sentence case |
| `eyebrow` | 11 bold, upper case, tracking 0.8, `text3` (tutorial and sheet list only) |
| `key` | 12 semibold monospaced (keycap), 13.5 semibold monospaced (Latin chip) |
| `sidebar` | 13.5 medium, selected semibold |

### 3.3 Radius and spacing

All numbers are points at the default window size 1000 × 680.

| Thing | Value |
| --- | --- |
| Spacing grid | 4, 8, 12, 16, 20, 24, 32, 40, 48 |
| Content panel | radius 14, 1 pt `panelLine` outline, inset 44 top, 12 right and bottom |
| Page padding | 40 top, 56 bottom, sides `max(48, 9 % of panel width)` |
| Page column | max width 720, centred (646 at 1000 wide, 548 at 880 wide) |
| Gap between sections | 32 (in the sheet: 28) |
| Section title to its card | 12 |
| Card, group, banner | radius 12, `fill`, no border, no shadow |
| Card padding | 20 (group: 0 top and bottom, 20 sides) |
| Settings row | min height 60, padding 12 top and bottom, 16 between text and control, `sep` hairline between rows |
| Option row (switch key) | min height 64 (tutorial body under 570 high: 56) |
| Cheat sheet row | min height 52 |
| Hero | height 152, radius 14, padding 28 sides |
| Button | radius 8, height 32 (large: 38), padding 14 sides |
| Sidebar row | radius 8, height 32, gap 2 |
| Keycap | radius 6, height 24 (large: 32), bottom border 2.5 |
| Switch | 36 × 20 |
| Pill, keyboard chip | capsule; chip height 26 |
| Tap target | at least 28 × 28 for icon buttons, 32 high for buttons |

## 4. Window

| Rule | Value |
| --- | --- |
| Default size | 1000 × 680 |
| Minimum size | 880 × 600 |
| Resizable | Yes. Content fills the window at every size. No fixed frame on the root. |
| Title bar | Hidden title, transparent title bar, full-size content view. Traffic lights sit on the frame at the top left. |
| Toolbar | None. The top icon buttons are normal views in the top strip. |

macOS 12 is the minimum, so `.defaultSize` and `.windowResizability` are not
always there. Do this:

1. Keep `WindowGroup` with `.windowStyle(.hiddenTitleBar)`.
2. On macOS 13 and later, add `.defaultSize(width: 1000, height: 680)` and
   `.windowResizability(.contentMinSize)`.
3. Add `WindowConfigurator`, an `NSViewRepresentable` in the background of the
   root view. On the first `viewDidMoveToWindow`, set:
   `titleVisibility = .hidden`, `titlebarAppearsTransparent = true`,
   `styleMask.insert(.fullSizeContentView)`, `isMovableByWindowBackground = true`,
   `minSize = 880 × 600`, `setFrameAutosaveName("BuuzMain")`. On macOS 12
   only, call `setContentSize(1000 × 680)` and `center()` when there is no
   saved frame.
4. Root view: `.frame(minWidth: 880, maxWidth: .infinity, minHeight: 600, maxHeight: .infinity)`
   and `.ignoresSafeArea()` so the frame colour goes under the title bar.
5. Apply the theme with `.preferredColorScheme(choice == .system ? nil : …)`.
   Also set `NSApp.appearance` to `nil`, `.aqua` or `.darkAqua`, so the
   dynamic `NSColor` tokens and the traffic lights follow.

Structure:

```text
AppShell (ZStack, background: frame)
├─ HStack(spacing: 0)
│  ├─ Sidebar or TourRail        width 200, padding top 48, leading 12, trailing 8, bottom 12
│  └─ ContentPanel               padding top 44, trailing 12, bottom 12
│     └─ RoundedRectangle 14, fill panel, stroke panelLine
│        └─ page view: PageColumn (app pages) or TourPage (tutorial)
├─ TopIcons                      top 7, trailing 12 (overlay, aligned top trailing)
└─ SettingsSheet                 when open: scrim + centred sheet (§6.4)
```

The 44 pt top strip holds the traffic lights (left) and the top icons
(right). The sidebar width stays 200 at every window size. The panel takes
all the remaining width and height.

### 4.1 No-overlap rule (`PageColumn`)

Build every app page and every settings page with `PageColumn`. Do not
build a page any other way.

```swift
struct PageColumn<Content: View>: View {
    var maxWidth: CGFloat = 720
    @ViewBuilder var content: Content
    var body: some View {
        GeometryReader { geo in
            ScrollView(.vertical) {
                VStack(alignment: .leading, spacing: 32) { content }
                    .frame(maxWidth: maxWidth)
                    .padding(.horizontal, max(48, geo.size.width * 0.09))
                    .padding(.top, 40).padding(.bottom, 56)
                    .frame(maxWidth: .infinity)
            }
        }
    }
}
```

Rules:

1. One vertical `ScrollView` per page. No nested scroll views, no fixed
   heights on cards, no `position` or `offset` for layout.
2. One column. A thing that used to sit in a side column goes into the
   column, or into a row of equal tiles under the hero (`LazyVGrid` with
   flexible columns).
3. Text never sits in a column under 180 pt wide. Option choices are
   full-width rows, never a grid.
4. A trailing control keeps its natural width (`fixedSize()`); the text on
   the left takes the rest and wraps.
5. The bottom padding (56) keeps the last card clear of the panel corner.
6. Check every page at 880 × 600 and 1000 × 680, Light and Dark, Mongolian
   and English. The mockup has a checker for this (§4.2).

### 4.2 Checking the layout

Open `mockup.html?shot=1&size=880x600&theme=dark&lang=en#shortcuts/conflict`
to see one window at one size, without the bar. Open `mockup.html?qa=1` to
run the layout check. It renders every screen and state at 1000 × 680 and
880 × 600 in both languages and lists: elements that go past the panel or
the sheet, siblings that overlap, a clipped sidebar, text that wraps to
three lines in a column under 180 pt, and tutorial bodies that need a
scroll bar. Today the list is empty, except two expected items: the typing
demo on Welcome clips on purpose, and the config path ends in an ellipsis.

## 5. Components

### 5.1 Sidebar (`Sidebar`)

- Logo row: 26 pt Buuz Buddy (`buuz-buddy-256.png`, radius 7), "Buuz" in
  18 heavy rounded, "2.0" in 10.5 bold `text3`.
- Main items: Home `house`, Lessons `book`.
- Spacer, then the setup card (§5.4), a `sep` line, then
  Settings `gearshape` (opens the sheet) and Help `questionmark.circle`.
- `SidebarItem`: 32 high, radius 8, gap 2, symbol 16 pt in `text2`, title
  13.5. Hover: `hover`. Selected: `selected` fill, semibold, symbol in
  `text`. Settings shows as selected while the sheet is open. Full keyboard
  focus with the `accent` focus ring.

### 5.2 Top icons (`TopIcons`)

Three 30 × 30 plain buttons, radius 8, symbol 17 pt in `text2`. Each has a
help tag and an accessibility label.

| Button | Symbol | Action |
| --- | --- | --- |
| Appearance | `sun.max` or `moon` | Menu: Light, Dark, System, with a check on the current one |
| Updates | `bell` | Popover: version, "Check for updates" button and its result. An `ember` dot shows only after a check finds a new version. No automatic network call. |
| Language | Text "МН" or "EN" in a capsule outline | Switches between Mongolian and English |

### 5.3 Content panel

App pages: `PageColumn` (§4.1) with the page header (`pageTitle`, one
`hint` line, an optional trailing button) and the sections. Tutorial: see
§7.2.

### 5.4 Setup card (`SetupChecklist`)

A compact card that does not crowd the navigation.

- On `panel` in Light, `fill` in Dark. Radius 12, padding 12 (6 at the
  bottom). No border.
- Header button: chevron 13 pt, `setup.title` 12.5 bold, "3/4" in `text3`
  at the right. A click folds the card to the header and the bar. Store the
  state in `Prefs` (`setup.folded`).
- Progress bar: 4 high, `track`, fill `accent`.
- Four rows, 26 high, 12.5, one line each (cut with an ellipsis). Circle
  14 pt. Done: `success` fill with a white check. Current: `accent` ring
  with a dot. Not started: `text3` ring.
- Rows and their real check:

| Row | Done when |
| --- | --- |
| `setup.add` | `InputSource.isEnabled` |
| `setup.key` | `Prefs.switchKey` is set |
| `setup.switch` | `Prefs.switchedOnce` is true (set the first time the current keyboard is Buuz) |
| `setup.lessons` | All 8 lessons passed. Show "5/8" at the right. |

- Click a row: go to that tutorial step.
- When all four are done, hide the card.

### 5.5 Hero card (`HeroCard`)

Height 152 (Wispr: 133 to 167). Padding 28 at the sides, content centred
vertically: title `heroTitle` in white, one line in white 78 %, one primary
button (large). No eyebrow. The text keeps 150 pt free at the right for the
Buddy image (122 pt, bottom right, rotated −6°, clipped by the card). A
24 pt close button (white 14 %) at the top right hides the card for this
launch. Under 500 pt column width, hide the image and let the text use the
full width.

| Variant | When | Title | Button |
| --- | --- | --- | --- |
| Lessons | Lessons not finished | `home.hero.lessons.title` | `home.hero.lessons.continue`, opens the next lesson |
| Off | `InputSource.isEnabled` is false | `home.hero.off.title` (`warnHero`), no close button | `enable.button`, calls `addBuuz()` |
| Tip | Setup done | One tip from the tip list | `home.hero.tip.button`, opens that lesson |

### 5.6 Status tiles (`StatusTile`)

A row of three equal tiles right under the hero (12 pt gap, 12 pt above).
Each tile: `fill`, radius 12, padding 14/16, a `tileValue` line and a
`hint` label, both on one line. Hover: `fill2`. The whole tile is a button.

| Tile | Value | Label | Click |
| --- | --- | --- | --- |
| Keyboard | Б glyph (22 pt `ember` tile) and "Buuz", or a gray "A" and the current keyboard name. Live. | `home.card.keyboard` | Settings › Shortcuts |
| Switch key | "🌐 fn", "⌃ Space", "⇪ Caps Lock" or the recorded keys | `home.card.switchKey` | Settings › Shortcuts |
| Mode | `typing.mode.smart` or `typing.mode.classic` | `home.card.mode` | Settings › Typing |

### 5.7 Keyboard chip (`KeyboardChip`)

Capsule, 26 high, on `fill2`. An 18 pt round glyph (Б on `ember` for Buuz,
the first letter on `text3` for others), the name, and a 6 pt dot:
`success` for Buuz, `warn` for others. The dot pulses slowly; no pulse with
Reduce Motion. It reads `InputSource.current` (§8.1).

### 5.8 Banner (`StatusBanner`)

Row: SF Symbol 18 pt, a text column (bold title line in the kind colour and
a body line in `text`), and either a small trailing item (a keycap, a
Buddy, one short button) or a button row under the text, 10 pt below. Use
the button row when there are two buttons or a long button title, so the
text never gets squeezed. Radius 12, padding 14/16, no border.

| Kind | Colours | Symbol |
| --- | --- | --- |
| `neutral` | `fill`, `text2` | `info.circle` |
| `info` | `accentSoft`, `accent` | `lightbulb` or `info.circle` |
| `ok` | `successBg`, `success` | `checkmark` |
| `warn` | `warnBg`, `warn` | `exclamationmark.triangle` |
| `danger` | `dangerBg`, `danger` | `exclamationmark.triangle` |

### 5.9 Settings row and group (`SettingsGroup`, `SettingsRow`)

Group: `fill`, radius 12, padding 0/20, a `sep` hairline between rows.
Row: title (`rowTitle`) and hint (`hint`) on the left, the control on the
right (`Toggle(.switch)`, segmented `Picker`, keycaps or a secondary
button). Min height 60. No icon tiles. Coming-soon rows: 55 % opacity, a
"coming soon" pill, no control. A section title (`sectionTitle`) sits 12 pt
above a group.

### 5.10 Keycap and Latin chip

`Keycap`: `panel`-like key with a 1 pt border and a 2.5 pt darker bottom
border. Large keycap (32 high) in the recorder. Lit keycap (`accent` fill,
white text) for keys held down while recording. `LatinChip`: monospaced text
on `fill2`, radius 6, for Latin input such as `qndqr`.

### 5.11 Hotkey picker (`SwitchKeyPicker`)

A group of four full-width radio rows (not a grid). Each row: radio 18 pt,
title (`rowTitle`) with the "Recommended" pill (`accentSoft`) after it for
fn, the note (`hint`) under the title, and the keycaps at the right.
Selected: the radio fills with `accent`. No borders or halos on the row.

| Option | Keys shown | How Buuz applies it |
| --- | --- | --- |
| Globe key | 🌐 fn | macOS setting "Press 🌐 key to: Change Input Source". Show the `neutral` banner `hotkey.system` with the button `shortcut.manual.button` under the text (opens Keyboard settings). |
| Control + Space | ⌃ Space | macOS shortcut "Select the previous input source". Same banner. |
| Caps Lock | ⇪ caps | macOS setting "Use Caps Lock to switch to and from ABC". Same banner. |
| My own shortcut | recorded keys | Buuz registers it (§8.2). Note `hotkey.record` until selected, then `hotkey.custom.note`. Shows the recorder. |

Under the group, when "My own shortcut" is selected, show `HotkeyRecorder`
(full width, min height 56, radius 12):

| State | Look | Text |
| --- | --- | --- |
| Empty | Dashed border on `fill` | `hotkey.record` |
| Recording | `accent` border on `accentSoft`, red dot pulsing, lit keycaps for held modifiers | `hotkey.recording`, under it `hotkey.needsModifier` |
| Conflict | No recorder. The keys show in the option row. A `warn` banner with a button row | `hotkey.conflict.title` and `hotkey.conflict.body`, buttons `hotkey.conflict.another` (primary, starts recording again) and `hotkey.conflict.useAnyway` (secondary) |
| Saved | `success` border on `successBg`, keycaps, check | `hotkey.saved`, and a `hotkey.clear` button |

Esc while recording cancels and keeps the old shortcut. A key with no ⌃, ⌥
or ⌘ is not accepted (fn alone and Caps Lock alone are only the macOS options).

## 6. App pages

Every page is a `PageColumn` (§4.1). Numbers are at 1000 × 680.

### 6.1 Home

Top to bottom, 32 pt between items (12 between the hero and the tiles):

1. Header: greeting by the local hour (before 12 `home.greeting.morning`,
   before 18 `home.greeting.afternoon`, else `home.greeting.evening`) and 👋,
   then `home.subtitle`.
2. Hero card (§5.5).
3. Status tiles (§5.6).
4. Try box: section title `home.try.title` with the `KeyboardChip` at the
   right, then a 48 pt `LessonField` (§8.3), radius 12, placeholder
   `home.try.placeholder`. Focused: `accent` border and a 4 pt halo. Not
   saved.
5. Cheat sheet: section title `home.cheat.title` and a search button (28 pt)
   that shows a filter field. A group with one row for each entry. Columns:
   keycaps 64, example (`LatinChip` → bold Cyrillic), description at the
   right (`hint`, one line, ellipsis). Under 470 pt column width, hide the
   description.

| Keys | Example | Description key | Smart |
| --- | --- | --- | --- |
| q | `qndqr` → өндөр | `cheat.q` | |
| w | `wg` → үг | `cheat.w` | |
| ' | `mor'` → морь | `cheat.soft` | |
| " | `"` → ъ | `cheat.hard` | |
| o u | `eruul` → эрүүл | `cheat.harmony` | yes |
| v v | `svv` → сүү | `cheat.vv` | yes |
| t s | `tsai` → цай | `cheat.ts` | yes |
| z h | `zhims` → жимс | `cheat.zh` | yes |
| esc | `zoom ⎋` → zoom | `cheat.esc` | yes |
| ⌫ | `tov ␣ ⌫` → тов | `cheat.backspace` | yes |
| b n | `bn` → байна | `cheat.abbr` | yes |

Hide a smart row when its switch is off or the mode is Classic. There is no
"smart" pill. I checked every example with `buuz-core` and the word list
(smart mode).

States: setup not done (setup card, lessons hero), all done (no setup card,
tip hero), Buuz not added (off hero, gray keyboard tile, gray chip, try box
not focused).

### 6.2 Lessons

Header `help.lessons`, `home.hero.lessons.body`, primary button
`help.replayAll` at the right. One group, one row for each lesson: number
or done check (22 pt circle), lesson title, keys → output on one line, ghost
button `help.replay`. Replay opens that lesson in the tutorial and returns
to Lessons after it.

### 6.3 Help

Header `help.title`, `help.subtitle`.

1. Section `help.missing`: a card (padding 20) with the System Settings
   path and a button row: `enable.button` (primary), `enable.manual.button`.
2. A group: feedback (`settings.feedback.title`, `settings.feedback.body`,
   `settings.feedback.button`) and tutorial (`general.tutorial`,
   `general.tutorial.hint`, `settings.shortcuts.rerunTutorial`).

### 6.4 Settings sheet (`SettingsSheet`)

Opened by Settings in the sidebar, by the status tiles, and by ⌘,. A modal
sheet over the whole window, in the style of the Wispr Flow settings.

- Scrim: `scrim` over the window. A click on the scrim or Esc closes the
  sheet.
- Sheet: width `min(760, window width − 64)`, height
  `min(560, window height − 56)`, centred, radius 14, `panel`, shadow.
- Left list: 184 wide on `frame`, padding 20/10. Eyebrow `nav.settings`,
  then rows like `SidebarItem`: General `slider.horizontal.3`
  (`settings.general.title`), Typing `pencil.line` (`nav.typing`), Shortcuts
  `command` (`nav.shortcuts`), Privacy `lock` (`nav.privacy`), About
  `info.circle` (`nav.about`). At the bottom, `settings.about.version` in
  `text3`.
- Right: close button 30 pt at the top right (`common.close`), and a
  `PageColumn` with padding 32 top, 36 sides, 40 bottom, no max width, 28
  between sections, `pageTitle` 22.
- Open on the page the user came from (Typing from the mode tile,
  Shortcuts from the key tiles), else on the last page used.

Why a sheet: settings are rare visits. The sheet keeps the main sidebar to
four items, so the setup card has room. Each settings page gets the full
sheet width for one column, and the user sees that Home is still there
behind it.

#### General

| Group | Row | Control |
| --- | --- | --- |
| 1 | `settings.general.language` | Segmented: Монгол, English |
| 1 | `settings.general.theme` | Segmented: Light, Dark, `general.theme.system` |
| 2 | `settings.general.openAtLogin` | Switch. `SMAppService.mainApp` on macOS 13 and later; hide the row on macOS 12. |
| 2 | `settings.general.checkUpdates` | `general.updates.hint`, secondary button |
| 2 | `general.tutorial` | `general.tutorial.hint`, secondary button `settings.shortcuts.rerunTutorial` (restarts at Welcome) |

#### Typing

Header `settings.typing.title`, `typing.subtitle`.

1. Mode group: `typing.mode`, `typing.mode.hint`, segmented Smart / Classic.
   This writes `preset` in the config. It needs `ConfigEditor.preset` and
   `setPreset` (small, test-first). In Classic, show the `info` banner
   `typing.classicNote` under it and dim the switches.
2. Groups and switches, each with its section title. Each switch writes one
   key in `[options]` with `setFlag`:

| Group | Row | Config key | Default |
| --- | --- | --- | --- |
| `typing.group.vowels` | `typing.smartVowels` | `smart_vowels` | on |
| | `typing.autocorrect` | `autocorrect` | off |
| `typing.group.keys` | `typing.doubleV` | `double_v` | on |
| | `typing.habitKeys` | `habit_keys` | on |
| | `typing.abbreviations` | `abbreviations` | on |
| `typing.group.latin` | `typing.keepLatin` | `keep_latin` | on |
| | `typing.autoLatin` | `auto_latin` | on |
| `typing.group.soon` | auto capitals, own keys, per app | none | coming soon (last section) |

Preview section (before Coming soon): `typing.preview`, the hint
`typing.preview.hint`, then a group of five 44 pt rows, Latin at the left
and the result at the right: `eruul`, `svv`, `tsai`, `bn`, `zoom ⎋`.
   Compute the result with the engine (`Engine` with the current config
   text), not with a fixed table.

#### Shortcuts

Header `settings.shortcuts.title`, `shortcuts.switchKey.hint`.

1. Section `settings.shortcuts.switchKey`: `SwitchKeyPicker` (§5.11) and its
   banner or recorder.
2. A group with one row: `shortcuts.test.title` and `shortcuts.test.body`
   with the chosen key at the left; two `KeyboardChip`s (other ⇄ Buuz) at
   the right. The current one updates live. After one switch to Buuz and
   back, show `shortcut.works` (without its emoji) in `success` under the
   body.
3. Section `shortcuts.typingKeys`: rows with the action at the left and the
   keycaps at the right: `shortcuts.key.commit` (Space ⏎ .),
   `shortcuts.key.esc` (esc), `shortcuts.key.backspace` (⌫).

#### Privacy

1. Header `settings.privacy.title`, `privacy.subtitle`.
2. `ok` banner with `wifi.slash`: `settings.privacy.noNetwork`, and the link
   `settings.privacy.source` → under it.
3. Section `settings.privacy.promises`: a group of four rows, the
   `privacy.card.*` titles (without emoji) and bodies.
4. Section `privacy.keeps.title`: three rows. The config row shows the real
   config path (`ConfigStore.defaultConfigURL`, with `~` for the home
   folder, one line with an ellipsis in the middle) and a
   `privacy.keeps.reveal` button (`NSWorkspace.activateFileViewerSelecting`).
   Then `privacy.keeps.prefs` and `privacy.keeps.never`.

#### About

The wordmark centred (44 high; `buuz-wordmark.png` in Light,
`buuz-wordmark-light.png` in Dark; add both to the bundle) and
`settings.about.version` under it. Then a group: the credit with
`settings.about.original`, the website, the licences, and feedback with
`settings.feedback.button`.

## 7. Tutorial

### 7.1 Flow

Replace `Step` with `TourStep`:

```swift
enum TourStep: String, CaseIterable {
    case welcome, privacy, add, key, switchTo
    case lesson1, lesson2, lesson3, lesson4, lesson5, lesson6, lesson7, lesson8
    case done
}
```

- Store the step as its raw value in `Prefs` (`tour.step`). Quitting resumes
  on the same step. Store passed lessons as a comma list (`tour.passed`).
- `next` skips `add` when Buuz is already enabled (as today). `back` does the
  same in reverse.
- Every step except Welcome and Done has a Skip button. Skip moves to the
  next step and does not mark the step as passed.
- "Skip the tutorial" in the rail: go to Done with the real counts.
- Done: `finishOnboarding()`. The app shows Home after that.
- Lessons › Replay starts at one lesson with `returnTo = .lessons`.
  Settings › General › Tutorial and Help › Tutorial start at Welcome.
- With Reduce Motion: no hop, no confetti, no pulse. Use a fade.

### 7.2 Layout

The same window and frame as the app. One clear task for each screen.

- `TourRail` instead of the sidebar (width 200): logo, eyebrow `tour.setup`
  and five rows (28 high, 13), eyebrow `tour.lessons` and eight small rows
  (26 high, 12.5, one line, cut with an ellipsis), the Done row, then "Skip
  the tutorial" at the bottom. Row states as in §5.4; the current row has
  the `selected` fill. A click on a done or current row goes there. The rail
  fits in 600 pt height without a scroll bar.
- Panel (`TourPage`): a body and a footer.
  - Body: a vertical `ScrollView` with padding 28 top, 20 bottom,
    `max(40, 8 % of panel width)` at the sides. The content is one column,
    at most 580 wide, centred horizontally, and centred vertically when it
    fits (top-aligned when it scrolls). 20 between items.
  - Footer: 60 high, pinned to the bottom, a `sep` line above, padding 0/20.
- Header in the body, centred: eyebrow (`tour.eyebrow` "Step n of 5" or
  `tour.lessonEyebrow` "Lesson n of 8"), `tourTitle`, one line in `text2`
  (14.5, at most 480 wide). No trailing chip and no lesson dots: the rail
  and the footer already show the progress.
- Footer: ghost "← Back" (not on Welcome), spacer, the step count in
  `text3`, ghost "Skip", primary button. The primary button is disabled until
  the step passes, where the step has a check.
- Short windows: when the body is under 570 high, the gap is 16 and option
  rows are 56 high. Under 520 high, the gap is 14, the title is 24, the
  practice field is 68 high with 22 pt text, and the Welcome Buddy is 96.
  With these rules, every step fits at 880 × 600 without a scroll bar.

### 7.3 Steps and pass conditions

| # | Step | Content | Passes when | States |
| --- | --- | --- | --- | --- |
| 1 | Welcome | Buddy 120 (hops once), `welcome.title`, `welcome.subtitle`, a demo line that types `sain baina uu` → сайн байна уу, `tour.welcome.time` | Always | one |
| 2 | Privacy | `privacy.title`, a group of four promise rows (`privacy.card.*` without emoji), `ok` banner `settings.privacy.noNetwork` with the link `privacy.githubPill` under it | Always | one |
| 3 | Add the keyboard | Status row in a group (Buddy 36, `enable.status.name`, `enable.status.kind` · `tour.add.live`, waiting or added pill, `enable.button`), `enable.note` centred, manual card with the path and `enable.manual.button`. | `InputSource.isEnabled` (enabled-sources notification) | waiting, added (title `enable.success`, green tick), failed (`danger` banner `enable.error`, manual card with a primary button) |
| 4 | Switch key | `tour.key.title`, `SwitchKeyPicker` | A switch key is chosen and not in a conflict that the user did not accept | choose, recording, conflict, saved |
| 5 | Switch to Buuz | Two tiles (140 × 96) ABC ⇄ Buuz with "NOW" on the current one, keycap of the key between. Two check rows side by side. | Current keyboard became Buuz, then another keyboard, then Buuz again | other (`tour.switch.subtitle`, `neutral` banner `tour.switch.menu` with a small menu bar picture), buuz (`tour.switch.backHint`, second row spins), done (title `tour.switch.done`, both rows checked) |
| 6–13 | Lessons | See below | See below | waiting, wrong keyboard, Latin only, almost, success |
| 14 | Done | Confetti behind the content, Buddy 96, `done.title`, `done.subtitle`, a group of three rows (switch key, tricks, lessons n/8 with `tour.done.replay`), switch `done.login` (on) | Always | one; the primary button is `done.button` |

On step 5, when the user picked "My own shortcut", the key works only while
the settings app runs (§8.2). Say so under the tiles if the switch is not
seen within 10 seconds.

Lesson layout (top to bottom, centred column):

1. Header: eyebrow, title, `lessonN.body`.
2. Task strip: `fill`, radius 12, padding 16/20, content centred on one
   line: the keys (Latin chips on `panel`, 16 pt, and keycaps), an arrow,
   and the expected output in `lessonOutput`. It wraps only when the column
   is too narrow.
3. The `lessonN.why` line, centred, 12.5, `text3`, 10 pt under the strip.
4. The `LessonField` (§8.3): full column width, at least 84 high, 26 pt
   text, radius 14, 1.5 pt border by state, a 4 pt halo. It gets focus when
   the lesson opens. No card or eyebrow around it.
5. A tool row 8 pt under the field: `KeyboardChip`, the sub-checks
   (lessons 6 and 7 only: small capsules that turn green), a spacer, and a
   ghost `lesson.clear` button.
6. Status banner (§5.8), full width.

Rules for all lessons. `full` is the field text. `committed` is the text
without the marked range. Before a comparison, change both to lower case, trim
them, and collapse runs of spaces.

| State | When (first match wins) | Field border | Banner |
| --- | --- | --- | --- |
| success | The lesson pass condition is true. It stays true for this lesson after that. | `success` + halo | `ok`: `lesson.success` and `lessonN.success`, a 40 pt Buddy that hops twice. The primary button turns on. Return also goes on. |
| wrong keyboard | The current keyboard is not Buuz | `warn` | `warn`: `lesson.wrong.title` with the keyboard name, `lesson.wrong.body` with the switch key, a keycap of the key at the right |
| Latin only | `full` has 3 or more ASCII letters and no Cyrillic, and the lesson is not lesson 7 | `warn` | `info`: `lesson.latin.title`, `lesson.latin.body` |
| almost | `committed` has Cyrillic, has the same word count as the target or more, and does not pass | `accent` | `info`: `lesson.close.title` and `lesson.close.body` with what they got (or the lesson's own `lessonN.close`). Show the wrong letters in `danger`, underlined. |
| waiting | Anything else | `accent` (focused) | `neutral`: `lesson.waiting` |

| Lesson | Keys | Target | Pass condition |
| --- | --- | --- | --- |
| 1 First words | `sain baina uu` | сайн байна уу | `committed` contains "сайн байна уу" |
| 2 Vowels match | `eruul mend` | эрүүл мэнд | `committed` contains "эрүүл" and "мэнд" |
| 3 q and w | `qndqr wg` | өндөр үг | `committed` contains "өндөр" and "үг" |
| 4 Soft sign | `mor' hon'` | морь хонь | `committed` contains "морь" and "хонь" |
| 5 Space and Enter | `nom` ⏎ | ном | Return was pressed in the field, `committed` contains "ном", and there is no marked text. The almost state is "ном" still marked: show `lesson5.close`. |
| 6 Backspace undo | `tov` ␣ ⌫ | тов | Sub-check 1: `committed` contained "төв" at some time in this lesson. Sub-check 2: after that, `full` contains "тов" and does not contain "төв". Pass when both are true. The almost state is sub-check 1 only: show `lesson6.close`. |
| 7 Esc keeps Latin | `zoom` esc ␣ | zoom | Sub-check 1: the marked text was "зоом" at some time in this lesson. Sub-check 2: after that, `committed` contains "zoom". Pass when both are true. The almost state is "зоом" committed: show `lesson7.close`. Lesson 7 has no Latin-only state, because Latin is the goal. |
| 8 Short forms | `bi zgr bn` | би зүгээр байна | `committed` contains "би зүгээр байна". When the `abbreviations` switch is off, show the `warn` banner `lesson8.off` with a `lesson8.turnOn` button that calls `setFlag("abbreviations", true)`. |

I checked all targets with `buuz-core`, smart mode and the word list. With
the word list, `unuudur` also gives өнөөдөр, so the old "trap" chip from the
Try step is gone.

## 8. Platform parts

### 8.1 Current keyboard

Add to `InputSource`:

- `current: (id: String, name: String)?` from
  `TISCopyCurrentKeyboardInputSource` with `kTISPropertyInputSourceID` and
  `kTISPropertyLocalizedName`.
- `isBuuzCurrent`: true when the id starts with
  `com.thetengis.inputmethod.Buuz`.
- `observeSelected(_:)`: observer for the distributed notification
  `com.apple.Carbon.TISNotifySelectedKeyboardInputSourceChanged`.

`AppModel` keeps `@Published var currentKeyboard`, sets `switchedOnce` the
first time it is Buuz, and counts the round trip for tutorial step 5. This
needs no permission. No key is read here.

### 8.2 Own shortcut

- Record with a local `NSEvent` monitor (`.keyDown`, `.flagsChanged`) that
  runs only while the recorder is in the recording state, and only in the
  Buuz window. This needs no permission.
- Use the types in `Sources/Core/Hotkey.swift` (`HotkeyMods` and the key
  value) and `Sources/App/HotkeyManager.swift` where they already cover this.
- Store as `custom:<keyCode>:<modifierFlags>` in `Prefs.switchKey`. The other
  values are `fn`, `controlSpace`, `capsLock`.
- Register with Carbon `RegisterEventHotKey` in the settings app. When it
  fires, select Buuz with `TISSelectInputSource`, or the last other keyboard
  when Buuz is current. `eventHotKeyExistsErr` means another app has it:
  show it as a conflict with the action "another app".
- The settings app must run for this. When the user saves an own shortcut,
  turn on "open at login" and run the app without a Dock icon when no window
  is open. Ask the maintainer before this ships (see §10).
- Conflict check: look up the recorded keys in this table, and in the user's
  enabled `AppleSymbolicHotKeys` (read `com.apple.symbolichotkeys` with
  `UserDefaults(suiteName:)`; read only).

| Keys | macOS action | String key |
| --- | --- | --- |
| ⌃ Space | Select the previous input source | `hotkey.action.prevSource` |
| ⌃ ⌥ Space | Select the next input source | `hotkey.action.nextSource` |
| ⌘ Space | Spotlight | `hotkey.action.spotlight` |
| ⌥ ⌘ Space | Spotlight (Finder search) | `hotkey.action.spotlight` |
| ⌃ ⌘ Space | Emoji & Symbols | `hotkey.action.emoji` |
| ⌘ Tab, ⌘ \` | Switch apps | `hotkey.action.appSwitcher` |
| ⇧ ⌘ 3, 4, 5 | Screenshot | `hotkey.action.screenshot` |

### 8.3 Lesson field (`LessonField`)

A SwiftUI `TextField` does not report marked text, so use an
`NSViewRepresentable` around a one-line `NSTextView` subclass:

- Override `setMarkedText`, `unmarkText`, `insertText` and
  `doCommand(by:)`. After each, call back with
  `LessonText(full: String, marked: NSRange?)`.
- `insertNewline(_:)` calls `onSubmit` and does not add a line.
- No logging, no saving. The text lives only in the view state of the
  current lesson and is cleared when the lesson changes.
- `LessonChecker` is a pure struct in `Sources/Core`: it takes the stream of
  `LessonText` values, the submit events and `isBuuzCurrent`, and gives the
  state and the sub-checks. Test it without AppKit.

## 9. Strings

Add each key to `macos/App/Strings/strings.en.json` and `strings.mn.json`.
The mockup reads the same table, so the mockup text and this list are the
same. T reviews the Mongolian text before the release.

Keys that stay as they are and are used again: `common.*`, `welcome.*`,
`privacy.*`, `enable.*`, `shortcut.subtitle`, `shortcut.manual.button`,
`shortcut.works`, `done.*`, all `settings.*` keys. The new layout does not
use `settings.home`, `settings.typing` (the sidebar labels with emoji),
`settings.home.*`, `try.*` and `tip.*`. Keep them until the menu stops using
them.

New keys (202):

| Key | English | Mongolian |
| --- | --- | --- |
| `nav.home` | Home | Нүүр |
| `nav.typing` | Typing | Бичих |
| `nav.shortcuts` | Shortcuts | Товчлол |
| `nav.privacy` | Privacy | Нууцлал |
| `nav.about` | About | Тухай |
| `nav.settings` | Settings | Тохиргоо |
| `nav.help` | Help | Тусламж |
| `nav.lessons` | Lessons | Хичээл |
| `top.theme` | Appearance | Харагдах байдал |
| `top.whatsNew` | Updates | Шинэчлэл |
| `top.language` | Language | Хэл |
| `setup.title` | Setup | Тохируулга |
| `setup.count` | {done}/{total} | {done}/{total} |
| `setup.add` | Add the keyboard | Гарт нэмэх |
| `setup.key` | Switch key | Солих товч |
| `setup.switch` | Switch to Buuz | Buuz руу шилжих |
| `setup.lessons` | Lessons | Хичээлүүд |
| `setup.open` | Open | Нээх |
| `home.greeting.morning` | Good morning | Өглөөний мэнд |
| `home.greeting.afternoon` | Good afternoon | Өдрийн мэнд |
| `home.greeting.evening` | Good evening | Оройн мэнд |
| `home.subtitle` | Type Latin, get Cyrillic in every app. | Латинаар бич, бүх апп дотор кирилл болоод гарна. |
| `home.hero.lessons.title` | Learn Buuz in 3 minutes | Buuz-ийг 3 минутад сур |
| `home.hero.lessons.body` | Eight short lessons. You type, Buuz checks. | Найман богино хичээл. Та бичнэ, Buuz шалгана. |
| `home.hero.lessons.continue` | Continue · {done}/{total} | Үргэлжлүүлэх · {done}/{total} |
| `home.hero.off.title` | Buuz is not in your keyboards | Buuz гарын жагсаалтад алга |
| `home.hero.off.body` | Add it back with one click. Your settings stay. | Нэг товшоод буцааж нэм. Тохиргоо тань хэвээрээ. |
| `home.hero.tip.title` | Esc keeps a word in Latin | Esc дарвал үг латинаараа үлдэнэ |
| `home.hero.tip.body` | Type zoom, press Esc, then Space. | zoom гэж бичээд Esc, дараа нь Space дар. |
| `home.hero.tip.button` | Try it | Туршаад үз |
| `home.card.keyboard` | is your keyboard now | одоогийн гар |
| `home.card.switchKey` | switch key | солих товч |
| `home.card.mode` | mode | горим |
| `home.card.notAdded` | Not added | Нэмээгүй |
| `home.try.title` | Try it here | Энд туршаад үз |
| `home.try.placeholder` | Type in Latin… | Латинаар бичээрэй… |
| `home.cheat.title` | Cheat sheet | Товчны хүснэгт |
| `home.cheat.find` | Find a key | Товч хайх |
| `cheat.q` | ө, anywhere in a word | ө, үгийн хаана ч |
| `cheat.w` | ү, anywhere in a word | ү, үгийн хаана ч |
| `cheat.soft` | soft sign ь | зөөлний тэмдэг ь |
| `cheat.hard` | hard sign ъ | хатуугийн тэмдэг ъ |
| `cheat.harmony` | o and u follow the word | o, u үгэндээ зохицно |
| `cheat.vv` | vv gives үү | vv нь үү |
| `cheat.ts` | ts gives ц | ts нь ц |
| `cheat.zh` | zh gives ж | zh нь ж |
| `cheat.esc` | keep the word in Latin | үгийг латинаар үлдээх |
| `cheat.backspace` | undo the last automatic change | сүүлийн автомат өөрчлөлтийг буцаах |
| `cheat.abbr` | abbreviation | товчилсон үг |
| `typing.subtitle` | Changes apply the next time you click in a text field. | Дараагийн удаа бичих талбарт дарахад өөрчлөлт хэрэгжинэ. |
| `typing.mode` | Mode | Горим |
| `typing.mode.smart` | Smart | Ухаалаг |
| `typing.mode.classic` | Classic | Сонгодог |
| `typing.mode.hint` | Classic types exactly like the original Buuz. | Сонгодог горим анхны Buuz-тэй яг адилхан бичнэ. |
| `typing.classicNote` | Classic mode turns off the smart switches below. | Сонгодог горимд доорх ухаалаг тохиргоо ажиллахгүй. |
| `typing.group.vowels` | Vowels and words | Эгшиг ба үг |
| `typing.group.keys` | Extra keys | Нэмэлт товч |
| `typing.group.latin` | Latin words | Латин үг |
| `typing.group.soon` | Coming soon | Удахгүй |
| `typing.smartVowels` | Smart vowels | Ухаалаг эгшиг |
| `typing.smartVowels.hint` | o and u match the word: eruul → эрүүл | o, u үгэндээ зохицно: eruul → эрүүл |
| `typing.autocorrect` | Autocorrect | Автомат засвар |
| `typing.autocorrect.hint` | хуухэд → хүүхэд at Space. Off by default. | Space дарахад хуухэд → хүүхэд. Анхдагчаар унтраалттай. |
| `typing.doubleV` | vv for үү | vv-ээр үү бичих |
| `typing.doubleV.hint` | svv → сүү | svv → сүү |
| `typing.habitKeys` | ts for ц, zh for ж | ts-ээр ц, zh-ээр ж |
| `typing.habitKeys.hint` | tsai → цай, zhims → жимс | tsai → цай, zhims → жимс |
| `typing.abbreviations` | Abbreviations | Товчилсон үг |
| `typing.abbreviations.hint` | bn → байна, zgr → зүгээр. Backspace right after undoes it. | bn → байна, zgr → зүгээр. Шууд Backspace дарвал буцна. |
| `typing.keepLatin` | Esc keeps Latin | Esc дарвал латинаар үлдэнэ |
| `typing.keepLatin.hint` | Press Esc while you type a word to keep it in Latin. | Үг бичиж байхдаа Esc дарвал латинаараа үлдэнэ. |
| `typing.autoLatin` | Links and emails stay Latin | Холбоос, имэйл латинаараа үлдэнэ |
| `typing.autoLatin.hint` | `name@mail.com` and `https://…` do not change. | `name@mail.com`, `https://…` өөрчлөгдөхгүй. |
| `typing.preview` | Preview | Урьдчилан харах |
| `typing.preview.hint` | What these keys give with your settings. | Таны тохиргоогоор эдгээр товч юу гаргахыг харуулна. |
| `shortcuts.switchKey.hint` | Press it to switch between Buuz and your Latin keyboard. | Buuz болон латин гарын хооронд шилжинэ. |
| `shortcuts.test.title` | Test your key | Товчоо шалга |
| `shortcuts.test.body` | Press {key}. The keyboard below changes. | {key} дар. Доорх гар солигдоно. |
| `shortcuts.typingKeys` | Keys while you type | Бичих үеийн товч |
| `shortcuts.key.commit` | Finish the word | Үгийг дуусгах |
| `shortcuts.key.esc` | Keep the word in Latin | Үгийг латинаар үлдээх |
| `shortcuts.key.backspace` | Undo the last automatic change | Сүүлийн автомат өөрчлөлтийг буцаах |
| `hotkey.fn` | Globe key | Глобус товч |
| `hotkey.fn.note` | On newer Macs. Set in macOS. | Шинэ Mac-д. macOS-д тохируулна. |
| `hotkey.controlSpace.note` | The macOS input source shortcut | macOS-ийн оролтын хэл солих товчлол |
| `hotkey.capsLock.note` | Switches between Buuz and ABC | Buuz, ABC хоёрын хооронд солино |
| `hotkey.custom` | My own shortcut | Өөрийн товчлол |
| `hotkey.custom.note` | Record any key combination | Дурын товчлол бичүүлэх |
| `hotkey.record` | Click to record a shortcut | Товчлол бичүүлэхийн тулд дарна уу |
| `hotkey.recording` | Press your shortcut… | Товчлолоо дарна уу… |
| `hotkey.needsModifier` | Add ⌃, ⌥ or ⌘ to the key. | Товч дээр ⌃, ⌥ эсвэл ⌘ нэмж дарна уу. |
| `hotkey.clear` | Clear | Арилгах |
| `hotkey.saved` | Saved | Хадгалагдлаа |
| `hotkey.system` | macOS owns this key. Buuz opens the right settings page for you. | Энэ товчийг macOS удирддаг. Buuz тохирох хуудсыг нь нээж өгнө. |
| `hotkey.conflict.title` | macOS already uses {keys} | {keys}-г macOS ашигладаг |
| `hotkey.conflict.body` | It is the shortcut for “{action}”. | Энэ нь «{action}» товчлол. |
| `hotkey.conflict.another` | Try another | Өөрийг сонгох |
| `hotkey.conflict.useAnyway` | Use anyway | Ингээд ашиглах |
| `hotkey.action.prevSource` | Select the previous input source | Өмнөх оролтын хэлийг сонгох |
| `hotkey.action.nextSource` | Select the next input source | Дараагийн оролтын хэлийг сонгох |
| `hotkey.action.spotlight` | Spotlight | Spotlight хайлт |
| `hotkey.action.emoji` | Emoji & Symbols | Эможи ба тэмдэгт |
| `hotkey.action.appSwitcher` | Switch apps | Апп солих |
| `hotkey.action.screenshot` | Screenshot | Дэлгэцийн зураг |
| `privacy.keeps.title` | What Buuz keeps on this Mac | Энэ Mac дээр Buuz юу хадгалдаг вэ |
| `privacy.keeps.config` | Your typing settings, in one small text file | Бичих тохиргоо тань, нэг жижиг текст файлд |
| `privacy.keeps.prefs` | Language, appearance and lesson progress | Хэл, харагдах байдал, хичээлийн явц |
| `privacy.keeps.never` | Never what you type | Таны бичсэн зүйлийг хэзээ ч хадгалахгүй |
| `privacy.keeps.reveal` | Show in Finder | Finder-т харуулах |
| `general.theme.system` | System | Системийн |
| `general.tutorial` | Tutorial | Заавар |
| `general.tutorial.hint` | Setup and all eight lessons, from the start. | Тохируулга болон найман хичээл, эхнээс нь. |
| `general.updates.hint` | Buuz asks only when you click. It never sends what you type. | Зөвхөн таныг дарахад асууна. Бичсэн зүйлийг тань илгээхгүй. |
| `about.website` | Website | Вэбсайт |
| `help.title` | Help | Тусламж |
| `help.subtitle` | Fix the keyboard, or tell us what is wrong. | Гараа засах эсвэл бидэнд хэлэх. |
| `help.lessons` | Lessons | Хичээлүүд |
| `help.replay` | Replay | Дахин |
| `help.replayAll` | Start all again | Бүгдийг дахин эхлэх |
| `help.missing` | Buuz is not in the keyboard list? | Buuz гарын жагсаалтад алга уу? |
| `tour.setup` | Setup | Тохируулга |
| `tour.lessons` | Lessons | Хичээлүүд |
| `tour.step.welcome` | Welcome | Тавтай морил |
| `tour.step.privacy` | Privacy | Нууцлал |
| `tour.step.add` | Add the keyboard | Гарт нэмэх |
| `tour.step.key` | Switch key | Солих товч |
| `tour.step.switch` | Switch to Buuz | Buuz руу шилжих |
| `tour.step.done` | Done | Дууслаа |
| `tour.skipAll` | Skip the tutorial | Зааврыг алгасах |
| `tour.skipStep` | Skip | Алгасах |
| `tour.eyebrow` | Step {n} of {total} | Алхам {n}/{total} |
| `tour.lessonEyebrow` | Lesson {n} of {total} | Хичээл {n}/{total} |
| `tour.welcome.time` | About 3 minutes. You can skip any step. | 3 минут орчим. Аль ч алхмыг алгасаж болно. |
| `tour.add.live` | Buuz checks this by itself | Buuz өөрөө шалгаж байна |
| `tour.key.title` | Choose your switch key | Солих товчоо сонго |
| `tour.switch.title` | Switch to Buuz | Buuz руу шилж |
| `tour.switch.subtitle` | Press {key}. The keyboard below changes by itself. | {key} дар. Доорх гар өөрөө солигдоно. |
| `tour.switch.now` | Now | Одоо |
| `tour.switch.press` | press {key} | {key} дар |
| `tour.switch.toBuuz` | Switch to Buuz | Buuz руу шилжих |
| `tour.switch.back` | Switch back to {name} | {name} руу буцах |
| `tour.switch.backHint` | Now press {key} again to go back to {name}. Then come back to Buuz. | Одоо {key}-г дахин дарж {name} руу буц. Тэгээд Buuz руу эргэж ир. |
| `tour.switch.done` | You can switch both ways | Та хоёр тийш шилжиж чаддаг боллоо |
| `tour.switch.menu` | Key does not work? Click the input menu in the menu bar and choose Buuz. | Товч ажиллахгүй байна уу? Дээд мөрний оролтын цэснээс Buuz-ийг сонго. |
| `lesson.type` | Type | Бич |
| `lesson.get` | You get | Гарна |
| `lesson.field` | Practice | Дасгал |
| `lesson.waiting` | Type the keys above. Buuz checks as you type. | Дээрх товчнуудыг бич. Бичих явцад Buuz шалгана. |
| `lesson.wrong.title` | Your keyboard is {name} | Таны гар {name} байна |
| `lesson.wrong.body` | Press {key} to switch to Buuz, then type again. | {key} дарж Buuz руу шилжээд дахин бич. |
| `lesson.latin.title` | Only Latin letters | Зөвхөн латин үсэг гарлаа |
| `lesson.latin.body` | Did you press Esc? Press it again while you type. Or check that Б is in the menu bar. | Esc дарсан уу? Бичиж байхдаа дахин дар. Эсвэл дээд мөрөнд Б байгаа эсэхийг шалга. |
| `lesson.close.title` | Almost | Бараг боллоо |
| `lesson.close.body` | You got {got}. Look at the hint and try again. | {got} гарлаа. Зөвлөгөөг хараад дахин оролд. |
| `lesson.success` | Correct! | Зөв! |
| `lesson.clear` | Clear | Арилгах |
| `lesson.next` | Next lesson | Дараагийн хичээл |
| `lesson.finish` | Finish | Дуусгах |
| `lesson1.title` | Your first words | Анхны үгс |
| `lesson1.body` | Type in Latin. The word is underlined until you press Space. | Латинаар бич. Space дартал үг доогуур зураастай байна. |
| `lesson1.why` | The underline means Buuz is still working on the word. | Доогуур зураас нь Buuz үгээ боловсруулж байна гэсэн үг. |
| `lesson1.success` | You typed your first Mongolian words. | Та анхны монгол үгсээ бичлээ. |
| `lesson2.title` | Vowels match the word | Эгшиг үгэндээ зохицно |
| `lesson2.body` | In a word with э, ө or ү, o becomes ө and u becomes ү. | э, ө, ү-тэй үгэнд o нь ө, u нь ү болно. |
| `lesson2.why` | You type o and u. Buuz picks о or ө, у or ү. | Та o, u л бичнэ. Buuz о эсвэл ө, у эсвэл ү-г сонгоно. |
| `lesson2.success` | Buuz matched the vowels for you. | Buuz эгшгийг тань тааруулж өглөө. |
| `lesson3.title` | q for ө, w for ү | q нь ө, w нь ү |
| `lesson3.body` | Use q and w when you want ө or ү for sure, also at the start of a word. | ө, ү заавал хэрэгтэй бол q, w ашигла. Үгийн эхэнд ч болно. |
| `lesson3.why` | q and w are free keys in Mongolian, so Buuz uses them. | Монголд q, w үсэг байхгүй тул Buuz тэдгээрийг ашиглана. |
| `lesson3.success` | ө and ү whenever you want them. | ө, ү хүссэн үедээ. |
| `lesson4.title` | ' for ь, " for ъ | ' нь ь, " нь ъ |
| `lesson4.body` | The apostrophe gives the soft sign. Shift and apostrophe give the hard sign. | Апостроф зөөлний тэмдэг ь, Shift + апостроф хатуугийн тэмдэг ъ өгнө. |
| `lesson4.why` | The ' key is next to Return on most keyboards. | ' товч ихэнх гар дээр Return-ий хажууд байдаг. |
| `lesson4.success` | Soft sign done. | Зөөлний тэмдэг боллоо. |
| `lesson5.title` | Space and Enter finish a word | Space, Enter үгийг дуусгана |
| `lesson5.body` | Type nom. It stays underlined. Press Enter to finish it. | nom гэж бич. Доогуур зураастай үлдэнэ. Enter дарж дуусга. |
| `lesson5.why` | Space, Enter, Tab and punctuation all finish the word. | Space, Enter, Tab, цэг таслал бүгд үгийг дуусгана. |
| `lesson5.close` | The word is still underlined. Press Enter. | Үг доогуур зураастай хэвээр байна. Enter дар. |
| `lesson5.success` | The word is finished and sent. | Үг дуусаж, илгээгдлээ. |
| `lesson6.title` | Backspace undoes a change | Backspace өөрчлөлтийг буцаана |
| `lesson6.body` | Type tov and Space. Buuz makes it төв. Press Backspace right after to get тов. | tov гээд Space дар. Buuz төв болгоно. Шууд Backspace дарвал тов болно. |
| `lesson6.why` | This works for word choice, abbreviations and autocorrect. | Үг сонголт, товчилсон үг, автомат засварт ажиллана. |
| `lesson6.step1` | tov + Space gives төв | tov + Space → төв |
| `lesson6.step2` | Backspace gives тов | Backspace → тов |
| `lesson6.close` | Press Backspace right after Space, before you type anything else. | Space-ийн дараа өөр юм бичихээс өмнө шууд Backspace дар. |
| `lesson6.success` | Buuz gave your word back. | Buuz үгийг тань буцааж өглөө. |
| `lesson7.title` | Esc keeps Latin | Esc дарвал латинаараа үлдэнэ |
| `lesson7.body` | For a name like zoom, press Esc before Space. The word stays in Latin. | zoom гэх мэт нэрийг бичээд Space-ээс өмнө Esc дар. Үг латинаараа үлдэнэ. |
| `lesson7.why` | Press Esc again to get the Cyrillic back. | Esc-ийг дахин дарвал кирилл буцаж гарна. |
| `lesson7.step1` | zoom shows as зоом | zoom → зоом |
| `lesson7.step2` | Esc and Space give zoom | Esc, Space → zoom |
| `lesson7.close` | Press Esc before Space. | Space-ээс өмнө Esc дар. |
| `lesson7.success` | Latin when you need it. | Хэрэгтэй үед латин. |
| `lesson8.title` | Short forms | Товчилсон үг |
| `lesson8.body` | Some short forms grow into full words at Space. | Зарим товчлол Space дарахад бүтэн үг болно. |
| `lesson8.why` | See all short forms in Typing › Abbreviations. | Бүх товчлолыг Бичих › Товчилсон үг хэсгээс хар. |
| `lesson8.off` | Abbreviations are off. Turn them on in Typing, or skip this lesson. | Товчилсон үг унтраалттай байна. Бичих хэсэгт асаах эсвэл энэ хичээлийг алгас. |
| `lesson8.turnOn` | Turn on | Асаах |
| `lesson8.success` | Three keys, one word. | Гурван товч, нэг үг. |
| `tour.done.lessons` | {done} of {total} lessons | {done}/{total} хичээл |
| `tour.done.tricks` | Tricks | Заль |
| `tour.done.replay` | Replay any lesson from Lessons. | Хичээлийг Хичээл хэсгээс дахин үзэж болно. |

## 10. Open questions for the maintainer

1. Own shortcut needs the settings app to run at login without a Dock icon
   (§8.2). Is that OK, or do we ship only the three macOS options first?
2. Default theme: Light (current rule) or System?
3. Preset switch (Smart / Classic) in Typing needs a small `ConfigEditor`
   change. Ship it in the same PR or later?
