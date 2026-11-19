# Strings editor

A local page to review and correct the UI strings of the macOS app. It shows
English and Mongolian side by side. It runs offline in any browser.

## Build and open

Run these commands from the repository root. Node is needed.

```sh
macos/tools/strings-editor/build.sh
open macos/build/strings-editor.html
```

The script builds the Buuz converter (`web/wasm`) with
`web/site/scripts/build-wasm.sh` when `web/site/src/wasm` is missing. Set
`REBUILD_WASM=1` to build it again. The output is one file with the strings,
the converter and the word list inside.

## Edit

1. Type in a Mongolian cell. Latin letters become Cyrillic as you type, with the
   same converter as the web site.
2. Use the "Latin → Cyrillic" button under a cell to turn this off for that cell.
   The two checkboxes at the top set the default for all cells.
3. Text inside `{braces}` stays Latin. Paste is never converted.
4. Click a `+ {name}` button to insert a placeholder that the other language has.
5. Yellow cells differ from the original. "Revert" restores a row.
6. A red note means the placeholders differ between the two languages.

7. Click "✨ Alternatives (n)" under a Mongolian cell to see other wordings.
   Click one to use it. The first row is the original, so you can go back.
   A check marks the row that the cell has now.
8. Keyboard: press Alt+Down in a Mongolian cell to open the menu. Use the
   Up and Down keys, Enter to pick, Esc to close. A click outside also closes it.
9. Clear "Show alternatives" at the top to hide the buttons.

The wordings come from `alternatives.mn.json` in this folder. It is an object
`{"key": ["wording", ...]}` with the same keys as `strings.mn.json`. The build
reads it when it exists. If it is missing, the build prints a note and the
page has no alternatives. The page never changes this file.

The page keeps your edits in the browser storage, so a reload keeps them.

## Download and replace the files

1. Click "Download strings.mn.json" and "Download strings.en.json".
2. Copy both files into `macos/App/Strings/`, and replace the old files.
3. Run `git diff` to check the changes.
4. Click "Copy changes as a list" to paste the changes into an issue or a message.

The key order, the 2-space indent and the final newline stay the same as in the
originals.

## Test

Run `node macos/tools/strings-editor/test.mjs` after the build. It checks the
typing, the placeholders, the alternatives menu logic and the JSON round trip with the real converter.
