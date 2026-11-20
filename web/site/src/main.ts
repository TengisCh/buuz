// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

import "@fontsource-variable/nunito/wght.css";
import "./style.css";
import init, { config_keys, config_remove_key, config_set_classic, config_set_option, convert_scheme, convert_with_config, key_table, scheme_for_language, scheme_ids, scheme_is_draft } from "./wasm/buuz.js";
import { TYPING_LANGS, formatOf, formatsOf, groupKeys, sampleKeys, type KeyOf } from "./mapping";
import wasmUrl from "./wasm/buuz_bg.wasm?url";
import { lang, setupLang, t, fmt, issueText, type Key } from "./i18n";
import { MONGOLIAN, loadScheme, saveScheme, schemeAfterLang } from "./scheme";
import { applyImport, browserStore, loadConfigState, parseIssue, saveConfig, switchesOf, type Issue } from "./config";
import phrases from "./phrases.json";
import { ghostPhrases, shuffled } from "./ghost";
import calibrationSentences from "./calibration.json";
import { accept, completionAt, ensureWordList, type Completion } from "./completion";
import wordListUrl from "../../../data/wordlist/mn.txt?url";
import { CalibrationFlow, saveError, type Outcome } from "./calibration";
import { readPaste } from "./paste";
import { addUserWord, clearUserDict, entriesOf, importUserDict, loadUserDict, removeUserWord } from "./userdict";

setupLang();

// Theme: light by default, dark only by choice (saved). The inline script in
// index.html applies a saved choice before first paint.
const themeBtn = document.getElementById("theme") as HTMLButtonElement;
function syncTheme(): void {
  const dark = document.documentElement.dataset.theme === "dark";
  const label = t(dark ? "themeLight" : "themeDark");
  themeBtn.setAttribute("aria-pressed", String(dark));
  themeBtn.setAttribute("aria-label", label);
  themeBtn.title = label;
}
themeBtn.addEventListener("click", () => {
  const root = document.documentElement;
  const dark = root.dataset.theme !== "dark";
  if (dark) root.dataset.theme = "dark";
  else delete root.dataset.theme;
  try {
    localStorage.setItem("buuz.theme", dark ? "dark" : "light");
  } catch {
    /* private mode: don't remember */
  }
  syncTheme();
});
document.addEventListener("buuz:lang", syncTheme);
syncTheme();

const latin = document.getElementById("latin") as HTMLTextAreaElement;
const cyr = document.getElementById("cyr") as HTMLTextAreaElement;
const copyBtn = document.getElementById("copy") as HTMLButtonElement;
const clearBtn = document.getElementById("clear") as HTMLButtonElement;
const status = document.getElementById("status") as HTMLElement;
const mascot = document.getElementById("mascot") as HTMLElement;

// The mascot puffs while you type and hops when you copy. Restarting the
// animation needs the class removed and a reflow before adding it back.
function animate(cls: "puff" | "hop"): void {
  mascot.classList.remove("puff", "hop");
  void mascot.offsetWidth;
  mascot.classList.add(cls);
}

let ready = false;

// Settings: the gear in the header opens a dialog with the mode and options.
const settings = document.getElementById("settings") as HTMLDialogElement;
document.getElementById("settings-btn")?.addEventListener("click", () => settings.showModal());
settings.querySelector(".fb-close")?.addEventListener("click", () => settings.close());
// A click on the backdrop lands on the dialog itself, outside its padding box.
settings.addEventListener("click", (e) => {
  const r = settings.getBoundingClientRect();
  if (e.target === settings && (e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom)) settings.close();
});

// The settings are one portable config text (see docs/decisions/0005). It is
// kept in localStorage as "buuz.config"; the mode and option switches below
// read and write it, and the converter reads it. With storage blocked, the
// page keeps it in memory for this visit.
const store = browserStore();
let config = ""; // loaded once the wasm module is ready (see init below)

// The language to type. "mn" uses the config and the Buuz rules; any other id
// is a core scheme (see src/scheme.ts). It is saved by itself, not in the config.
const schemeSelect = document.getElementById("scheme") as HTMLSelectElement;
let scheme = MONGOLIAN;
const typesMongolian = (): boolean => scheme === MONGOLIAN;

/** Show the scheme in the dialog and tell the style sheet which settings apply. */
function applyScheme(): void {
  schemeSelect.value = scheme;
  document.body.dataset.scheme = scheme;
  (document.getElementById("scheme-draft") as HTMLElement).hidden = typesMongolian() || !scheme_is_draft(scheme);
  // The Words tab is for Mongolian only; leave it when another scheme is picked.
  if (!typesMongolian() && document.getElementById("tab-words")?.getAttribute("aria-selected") === "true") selectSetTab("tab-typing");
  renderMaps();
}

// Typing language and format. The chip under the Latin field (and the copies
// in "How to type" and the Rules tab) opens one dialog: the language, its
// formats and the full key table. The page language in the header changes
// only the page text; the typing language follows it at first (schemeAfterLang).
// The tables are built from the core rules, so they match the converter.
const typingDlg = document.getElementById("typing") as HTMLDialogElement;
typingDlg.querySelector(".fb-close")?.addEventListener("click", () => typingDlg.close());
typingDlg.addEventListener("click", (e) => e.target === typingDlg && typingDlg.close());
const openTyping = () => {
  if (settings.open) settings.close();
  typingDlg.showModal();
};
for (const chip of document.querySelectorAll<HTMLButtonElement>(".typing-chip")) chip.addEventListener("click", openTyping);
const NOTE: Partial<Record<KeyOf["when"], Key>> = { after_vowel: "mapAfterVowel", word_start: "mapWordStart" };
const FORMAT_EMOJI = { chat: "💬", official: "🏛️", lookalike: "🔢" } as const;
const el = <K extends keyof HTMLElementTagNameMap>(tag: K, props: Partial<HTMLElementTagNameMap[K]> = {}, ...kids: (Node | string)[]) => {
  const e = Object.assign(document.createElement(tag), props);
  e.append(...kids);
  return e;
};
function flag(lang: string): SVGSVGElement {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("class", "flag");
  svg.setAttribute("viewBox", "0 0 24 16");
  svg.setAttribute("aria-hidden", "true");
  const use = document.createElementNS("http://www.w3.org/2000/svg", "use");
  // Only a known language code goes into the href.
  const code = TYPING_LANGS.find(([l]) => l === lang)?.[0] ?? "mn";
  use.setAttribute("href", `#f-${code}`);
  svg.append(use);
  return svg;
}
function formatName(id: string): string {
  const f = formatOf(id);
  return f.name ?? t(f.kind === "lookalike" ? "fmtLookalike" : "fmtChat");
}
function formatSub(id: string): string {
  const kind = formatOf(id).kind;
  return kind === "official" ? t("fmtOfficialSub") : kind === "lookalike" ? "4to → что" : t("fmtChatSub");
}
function renderMaps(): void {
  const { lang: typeLang } = formatOf(scheme);
  const langName = TYPING_LANGS.find(([l]) => l === typeLang)?.[1] ?? typeLang;
  const ids = scheme_ids().split(",");
  for (const chip of document.querySelectorAll<HTMLButtonElement>(".typing-chip")) {
    chip.replaceChildren(
      el("span", { ariaHidden: "true", textContent: "⌨️" }),
      flag(typeLang),
      el("span", { lang: typeLang, textContent: langName }),
      el("span", { className: "fmt", textContent: `· ${FORMAT_EMOJI[formatOf(scheme).kind]} ${formatName(scheme)}` }),
      el("span", { className: "chev", ariaHidden: "true", textContent: "▾" }),
    );
    chip.setAttribute("aria-label", `${t("typingTitle")}: ${langName}, ${formatName(scheme)}`);
  }
  // The dialog: languages, then the formats of the chosen language.
  document.getElementById("typing-langs")?.replaceChildren(
    ...TYPING_LANGS.map(([l, name]) => {
      const b = el("button", { type: "button", lang: l }, flag(l), el("span", { textContent: name }));
      b.setAttribute("aria-pressed", String(l === typeLang));
      b.addEventListener("click", () => ready && setScheme(formatsOf(l, ids)[0]));
      return b;
    }),
  );
  document.getElementById("typing-formats")?.replaceChildren(
    ...formatsOf(typeLang, ids).map((id) => {
      const b = el("button", { type: "button" }, el("b", { textContent: `${FORMAT_EMOJI[formatOf(id).kind]} ${formatName(id)}` }), el("small", { textContent: formatSub(id) }));
      b.setAttribute("aria-pressed", String(id === scheme));
      b.addEventListener("click", () => ready && setScheme(id));
      return b;
    }),
  );
  const rows = groupKeys(key_table(scheme), typeLang);
  // The line under the output: a few keys of the active format, and a link to all.
  const strip = document.getElementById("rule-strip");
  if (strip) {
    const more = el("button", { type: "button", className: "linkish" }, el("span", { ariaHidden: "true", textContent: "📖 " }), t("allRules"));
    more.addEventListener("click", openTyping);
    strip.replaceChildren(...sampleKeys(rows).flatMap((k) => [el("span", { className: "pair", lang: typeLang }, el("kbd", { lang: "en", textContent: k.latin }), ` ${k.out}`)]), more);
  }
  const list = () =>
    el(
      "ul",
      { className: "map", lang: typeLang },
      ...rows.map((r) => {
        const keys = el("span", { className: "in" });
        for (const k of r.keys) {
          keys.append(el("kbd", { lang: "en", textContent: k.latin, className: k.when === "smart" ? "smart" : "", title: k.when === "smart" ? t("mapSmart") : "" }));
          const note = NOTE[k.when];
          if (note) keys.append(el("small", { textContent: t(note) }));
        }
        const out = el("span", { className: "out", textContent: r.out });
        if (r.keys.some((k) => k.when === "harmony")) {
          out.title = t("mapHarmony");
          out.append(el("span", { ariaHidden: "true", textContent: " 🎵" }));
        }
        return el("li", {}, keys, out);
      }),
    );
  for (const id of ["keys-cheat", "keys-rules", "keys-typing"]) document.getElementById(id)?.replaceChildren(list());
  renderMyKeys();
}

/** The user's own keys (the config's [keys], for example from calibration), each with a remove button. */
function renderMyKeys(): void {
  const box = document.getElementById("my-keys") as HTMLElement;
  const ul = document.getElementById("my-keys-list") as HTMLUListElement;
  let lines: string[] = [];
  try {
    lines = config ? config_keys(config).split("\n").filter(Boolean) : [];
  } catch {
    lines = [];
  }
  box.hidden = lines.length === 0;
  ul.replaceChildren(
    ...lines.map((line) => {
      const [key, value] = line.split("\t");
      const rm = el("button", { type: "button", className: "linkish my-key-x", textContent: "✕", ariaLabel: fmt("myKeyRemove", { key }) });
      rm.addEventListener("click", () => {
        setConfig(config_remove_key(config, key));
        cfgSay([t("myKeyRemoved")]);
      });
      return el("li", {}, el("span", { className: "in" }, el("kbd", { lang: "en", textContent: key })), el("span", { className: "out", textContent: value || "∅" }), rm);
    }),
  );
}
document.addEventListener("buuz:lang", () => ready && renderMaps());

// Settings tabs (Typing, Words, Rules): one panel shows at a time. Arrow keys
// move between the visible tabs, as in the WAI-ARIA tabs pattern.
const setTabs = Array.from(document.querySelectorAll<HTMLButtonElement>('.set-tabs [role="tab"]'));
function selectSetTab(id: string, focus = false): void {
  for (const tab of setTabs) {
    const on = tab.id === id;
    tab.setAttribute("aria-selected", String(on));
    tab.tabIndex = on ? 0 : -1;
    (document.getElementById(tab.getAttribute("aria-controls") ?? "") as HTMLElement).hidden = !on;
    if (on && focus) tab.focus();
  }
}
for (const tab of setTabs) {
  tab.addEventListener("click", () => selectSetTab(tab.id));
  tab.addEventListener("keydown", (e) => {
    if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
    const shown = setTabs.filter((t) => t.offsetParent !== null);
    const i = shown.indexOf(tab);
    const next = shown[(i + (e.key === "ArrowRight" ? 1 : shown.length - 1)) % shown.length];
    e.preventDefault();
    selectSetTab(next.id, true);
  });
}

function setScheme(id: string): void {
  scheme = id;
  saveScheme(store, id);
  applyScheme();
  render();
  ghostForScheme();
  updateHint();
}

schemeSelect.addEventListener("change", () => ready && setScheme(schemeSelect.value));
// A page in Russian types Russian, and so on (see schemeAfterLang).
let pageLang: string = lang();
document.addEventListener("buuz:lang-changed", (e) => {
  const next = (e as CustomEvent<string>).detail;
  const previous = pageLang;
  pageLang = next;
  if (ready) setScheme(schemeAfterLang(scheme, next, previous, scheme_for_language));
});

// Mode: "smart" (the default; the feature toggles below apply) or "classic"
// (the original Buuz, every smart feature off, toggles greyed out).
const features = document.getElementById("features") as HTMLFieldSetElement;
const modeRadios = document.querySelectorAll<HTMLInputElement>('input[name="mode"]');
const smartBox = document.getElementById("smart") as HTMLInputElement;
const abbrBox = document.getElementById("abbr") as HTMLInputElement;
const completionBox = document.getElementById("completion") as HTMLInputElement;
const autocorrectBox = document.getElementById("autocorrect") as HTMLInputElement;
const doubleVBox = document.getElementById("double-v") as HTMLInputElement;
const habitKeysBox = document.getElementById("habit-keys") as HTMLInputElement;
const wordChoiceBox = document.getElementById("word-choice") as HTMLInputElement;
// The typewriter demo never expands abbreviations.
let ghostConfig = "";

function applyConfig(): void {
  const s = switchesOf(config);
  for (const r of modeRadios) r.checked = (r.value === "classic") === s.classic;
  // Plain Classic greys the switches out (they show the effective values). If
  // the config turns smart features on in Classic, they stay usable, with a note.
  features.disabled = s.classic && !s.classicWithFeatures;
  (document.getElementById("classic-note") as HTMLElement).hidden = !s.classicWithFeatures;
  smartBox.checked = s.smartVowels;
  abbrBox.checked = s.abbreviations;
  completionBox.checked = s.completion;
  autocorrectBox.checked = s.autocorrect;
  doubleVBox.checked = s.doubleV;
  habitKeysBox.checked = s.habitKeys;
  wordChoiceBox.checked = s.wordChoice;
  // Turned on while the box has focus: load the list now (see loadWords).
  // Autocorrect needs the list for the output, so it loads it at once.
  if (document.activeElement === latin || s.autocorrect) loadWords();
  ghostConfig = config_set_option(config, "abbreviations", false);
}

/** Make `text` the config: save it, show it in the switches, convert again. */
function setConfig(text: string, save = true): void {
  config = text;
  if (save) saveConfig(store, text);
  applyConfig();
  renderMyKeys();
  render();
}

for (const r of modeRadios) {
  r.addEventListener("change", () => {
    if (r.checked && ready) setConfig(config_set_classic(config, r.value === "classic"));
  });
}
// "Smart vowels" (on by default): look-ahead harmony, ий/ы, the уу/үү particle.
// Off gives the classic left-to-right behaviour of the original Buuz.
smartBox.addEventListener("change", () => ready && setConfig(config_set_option(config, "smart_vowels", smartBox.checked)));
// "Abbreviations" (on by default): bn + space or punctuation -> байна (never at the end of the text).
abbrBox.addEventListener("change", () => ready && setConfig(config_set_option(config, "abbreviations", abbrBox.checked)));

// "Finish words with Tab" (on by default): see docs/typing-rules.md section 12.
completionBox.addEventListener("change", () => ready && setConfig(config_set_option(config, "completion", completionBox.checked)));

// "Fix spelling mistakes" (off by default): see docs/typing-rules.md section 13.
autocorrectBox.addEventListener("change", () => ready && setConfig(config_set_option(config, "autocorrect", autocorrectBox.checked)));

// "Type vv as үү" (on by default): see docs/typing-rules.md section 3.
doubleVBox.addEventListener("change", () => ready && setConfig(config_set_option(config, "double_v", doubleVBox.checked)));

// "Type ts as ц and zh as ж" (on by default): see docs/typing-rules.md section 3.
habitKeysBox.addEventListener("change", () => ready && setConfig(config_set_option(config, "habit_keys", habitKeysBox.checked)));
wordChoiceBox.addEventListener("change", () => ready && setConfig(config_set_option(config, "word_choice", wordChoiceBox.checked)));

// Completion hint: under the Latin box, the whole Cyrillic word with the
// ghost part in grey. It shows only while the box has focus, nothing is
// selected, and the caret is at the end of a word. Tab accepts the word only
// while the hint is visible; Esc hides the hint for the current word, so the
// next Tab moves focus. The hint comes back when the word changes. With no
// hint, Tab keeps its normal focus behaviour.
const hintEl = document.getElementById("completion-hint") as HTMLElement;
let pending: Completion | null = null;
let dismissed: string | null = null; // "start:latin word" that Esc hid

/** Load the word list (a separate asset) the first time it is needed. */
function loadWords(): void {
  if (!ready || !typesMongolian()) return;
  const s = switchesOf(config);
  if (!s.completion && !s.autocorrect && !s.wordChoice) return;
  // After the list loads, convert again so autocorrect and word choice show in the output.
  void ensureWordList(wordListUrl, location.href, fetch).then((ok) => {
    updateHint();
    const now = switchesOf(config);
    if (ok && (now.autocorrect || now.wordChoice)) render();
  });
}
function wordKey(c: Completion): string {
  const end = latin.selectionStart;
  return `${end - c.replaces}:${latin.value.slice(end - c.replaces, end)}`;
}
function currentCompletion(): Completion | null {
  if (!ready || !typesMongolian() || latin.selectionStart !== latin.selectionEnd) return null;
  const c = completionAt(latin.value, latin.selectionStart, config);
  return c && wordKey(c) !== dismissed ? c : null;
}
function updateHint(): void {
  pending = document.activeElement === latin ? currentCompletion() : null;
  hintEl.hidden = !pending;
  hintEl.replaceChildren();
  if (!pending) return;
  const head = pending.word.slice(0, pending.word.length - pending.rest.length);
  const word = document.createElement("span");
  word.lang = "mn";
  const ghostEl = document.createElement("span");
  ghostEl.className = "ghost-rest";
  ghostEl.textContent = pending.rest;
  word.append(head, ghostEl);
  hintEl.append(`${t("completionHintLabel")} `, word);
}
document.addEventListener("selectionchange", updateHint);
document.addEventListener("buuz:lang", updateHint);
latin.addEventListener("focus", loadWords);
latin.addEventListener("input", loadWords);
latin.addEventListener("blur", updateHint);
latin.addEventListener("keydown", (e) => {
  if (e.isComposing || e.ctrlKey || e.altKey || e.metaKey) return;
  if (e.key === "Escape" && pending) {
    dismissed = wordKey(pending);
    updateHint();
    return;
  }
  if (e.key !== "Tab" || e.shiftKey || !pending) return;
  const c = pending;
  e.preventDefault();
  const caret = latin.selectionStart;
  latin.setSelectionRange(caret - c.replaces, caret);
  // insertText keeps the browser's undo history and fires "input".
  if (!document.execCommand("insertText", false, c.word)) {
    const next = accept(latin.value, caret, c);
    latin.value = next.value;
    latin.setSelectionRange(next.caret, next.caret);
    latin.dispatchEvent(new Event("input"));
  }
});

function render(): void {
  // convert_with_config() is microseconds even for long pastes (the core reads
  // the config once per change), so run it synchronously on every input
  // event: no debounce, no frame delay.
  const text = latin.value.normalize("NFC");
  cyr.value = !ready ? "" : typesMongolian() ? convert_with_config(text, config) : convert_scheme(scheme, text);
}

// Settings file: export the config as text or a .buuz file, import it by
// paste or by opening a file. A bad import shows an error and changes nothing.
const cfgText = document.getElementById("cfg-text") as HTMLTextAreaElement;
const cfgMsg = document.getElementById("cfg-msg") as HTMLElement; // role="status": success, polite
const cfgErr = document.getElementById("cfg-err") as HTMLElement; // role="alert": errors, assertive
const cfgFile = document.getElementById("cfg-file") as HTMLInputElement;
const udMsg = document.getElementById("ud-msg") as HTMLElement;
const udErr = document.getElementById("ud-err") as HTMLElement;
let sayFrame = 0;
/**
 * Show a message. The region is emptied first and filled on the next frame, so
 * a screen reader announces the same message again. `lines[0]` comes first,
 * then the issue list, then the other lines.
 */
function cfgSay(lines: string[], err = false, issues: Issue[] = [], regions: [HTMLElement, HTMLElement] = [cfgMsg, cfgErr]): void {
  cfgClear();
  const on = err ? regions[1] : regions[0];
  sayFrame = requestAnimationFrame(() => {
    const line = (text: string) => {
      const div = document.createElement("div");
      div.textContent = text;
      return div;
    };
    on.append(line(lines[0]));
    if (issues.length > 0) {
      const ul = document.createElement("ul");
      for (const i of issues) {
        const li = document.createElement("li");
        li.textContent = issueText(i);
        ul.append(li);
      }
      on.append(ul);
    }
    for (const l of lines.slice(1)) on.append(line(l));
  });
}
/** Empty both message regions (language change, dialog closed). */
function cfgClear(): void {
  cancelAnimationFrame(sayFrame);
  for (const el of [cfgMsg, cfgErr, udMsg, udErr]) el.replaceChildren();
}
settings.addEventListener("close", cfgClear);
document.addEventListener("buuz:lang", cfgClear);

function importText(raw: string): void {
  if (!ready) return;
  if (raw.trim() === "") return cfgSay([t("importEmpty")], true);
  const r = applyImport(store, raw);
  if (!r.ok) return cfgSay([issueText(r.error)], true);
  setConfig(r.text, false);
  cfgText.value = "";
  const notSaved = r.saved ? [] : [t("importNotSaved")];
  if (r.warnings.length > 0) cfgSay([t("importIgnored"), ...notSaved], false, r.warnings);
  else cfgSay([t("importOk"), ...notSaved]);
}
document.getElementById("cfg-copy")?.addEventListener("click", async () => {
  if (!ready) return;
  try {
    await navigator.clipboard.writeText(config);
  } catch {
    // No clipboard access: show the text in the box and say to copy it by hand.
    cfgText.value = config;
    cfgText.focus();
    cfgText.select();
    return cfgSay([t("copyByHand")], true);
  }
  cfgSay([t("exportCopied")]);
});
document.getElementById("cfg-download")?.addEventListener("click", () => {
  if (!ready) return;
  const url = URL.createObjectURL(new Blob([config], { type: "text/plain;charset=utf-8" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = "my-typing.buuz";
  document.body.append(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  cfgSay([t("exportDownloaded")]);
});
document.getElementById("cfg-apply")?.addEventListener("click", () => importText(cfgText.value));
document.getElementById("cfg-open")?.addEventListener("click", () => cfgFile.click());
cfgFile.addEventListener("change", () => {
  const file = cfgFile.files?.[0];
  cfgFile.value = ""; // so the same file can be opened again
  if (!file) return;
  if (file.size > 1024 * 1024) return cfgSay([issueText({ code: "too_large", number: 0, detail: "1 MiB" })], true);
  file.text().then(importText, () => cfgSay([t("importFileError")], true));
});

// Personal dictionary (docs/typing-rules.md section 14): the words the user
// added, kept as text in localStorage under "buuz.userdict". It stays on this
// device and is not part of the config file. The page never learns by itself.
// The list is built with textContent only.
let userText = ""; // loaded once the wasm module is ready (see init below)
const udList = document.getElementById("ud-list") as HTMLUListElement;
const udEmpty = document.getElementById("ud-empty") as HTMLElement;
const udWord = document.getElementById("ud-word") as HTMLInputElement;
const udFile = document.getElementById("ud-file") as HTMLInputElement;
const udSay = (lines: string[], err = false, issues: Issue[] = []) => cfgSay(lines, err, issues, [udMsg, udErr]);
const KIND_KEY = { add: "udKindAdd", use: "udKindUse", reject: "udKindReject" } as const;

function showUserDict(): void {
  const entries = entriesOf(userText).reverse(); // the newest first
  udEmpty.hidden = entries.length > 0;
  (document.getElementById("ud-count") as HTMLElement).textContent = entries.length > 0 ? String(entries.length) : "";
  udList.hidden = entries.length === 0;
  document.getElementById("ud-clear")?.toggleAttribute("disabled", entries.length === 0);
  document.getElementById("ud-export")?.toggleAttribute("disabled", entries.length === 0);
  udList.replaceChildren(
    ...entries.map((e) => {
      const li = document.createElement("li");
      const word = document.createElement("span");
      word.className = "ud-word";
      word.textContent = e.word;
      const meta = document.createElement("span");
      meta.className = "ud-meta";
      meta.lang = document.documentElement.lang;
      meta.textContent = e.kind === "add" ? t(KIND_KEY.add) : `${t(KIND_KEY[e.kind])}, ${fmt("udCount", { n: e.count })}`;
      const del = document.createElement("button");
      del.type = "button";
      del.className = "ghost";
      del.textContent = t("udDelete");
      del.setAttribute("aria-label", fmt("udDeleteLabel", { word: e.word }));
      del.addEventListener("click", () => {
        const r = removeUserWord(store, userText, e.word);
        changeUserDict(r.text, [t("udRemoved")], r.saved);
      });
      li.append(word, meta, del);
      return li;
    }),
  );
}
/** Make `text` the dictionary (already saved), show it, convert again, and say `lines`. */
function changeUserDict(text: string, lines: string[], saved: boolean, issues: Issue[] = []): void {
  userText = text;
  showUserDict();
  render();
  updateHint();
  udSay(saved ? lines : [...lines, t("udNotSaved")], false, issues);
}
document.addEventListener("buuz:lang", showUserDict);
function addWord(): void {
  if (!ready) return;
  const r = addUserWord(store, userText, udWord.value);
  if (!r.ok) return udSay([issueText(r.error)], true);
  udWord.value = "";
  changeUserDict(r.text, [t("udAdded")], r.saved);
}
document.getElementById("ud-add")?.addEventListener("click", addWord);
udWord.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.isComposing) {
    e.preventDefault();
    addWord();
  }
});
document.getElementById("ud-clear")?.addEventListener("click", () => {
  if (!ready || !window.confirm(t("udClearConfirm"))) return;
  const r = clearUserDict(store);
  changeUserDict(r.text, [t("udCleared")], r.saved);
});
document.getElementById("ud-export")?.addEventListener("click", () => {
  if (!ready) return;
  const url = URL.createObjectURL(new Blob([userText], { type: "text/plain;charset=utf-8" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = "buuz-words.txt";
  document.body.append(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  udSay([t("udExported")]);
});
document.getElementById("ud-open")?.addEventListener("click", () => udFile.click());
udFile.addEventListener("change", () => {
  const file = udFile.files?.[0];
  udFile.value = "";
  if (!file || !ready) return;
  if (file.size > 1024 * 1024) return udSay([issueText({ code: "too_large", number: 0, detail: "1 MiB" })], true);
  file.text().then(
    (raw) => {
      const r = importUserDict(store, userText, raw);
      if (!r.ok) return udSay([issueText(r.error)], true);
      changeUserDict(r.text, [r.warnings.length > 0 ? t("udImportIgnored") : t("udImportOk")], r.saved, r.warnings);
    },
    () => udSay([t("importFileError")], true),
  );
});

// Teach Buuz how you type (issue #4): a second dialog on top of the settings.
// The user types each sentence in Latin with conversion off (a plain input),
// then sees the suggestions and chooses which to keep. What was typed lives
// only in `flow` and in the input; closing the dialog in any way (Esc, Cancel,
// Save, Use the default) drops both. Text is set with textContent only.
const cal = document.getElementById("calibrate") as HTMLDialogElement;
const calStep = document.getElementById("cal-step") as HTMLElement;
const calResult = document.getElementById("cal-result") as HTMLElement;
const calInput = document.getElementById("cal-input") as HTMLInputElement;
const calProgress = document.getElementById("cal-progress") as HTMLElement;
const calSentence = document.getElementById("cal-sentence") as HTMLElement;
const calBack = document.getElementById("cal-back") as HTMLButtonElement;
const calNext = document.getElementById("cal-next") as HTMLButtonElement;
const calList = document.getElementById("cal-list") as HTMLUListElement;
const calNotes = document.getElementById("cal-notes") as HTMLUListElement;
const calLead = document.getElementById("cal-lead") as HTMLElement;
const calRetry = document.getElementById("cal-retry") as HTMLButtonElement;
const calSaveBtn = document.getElementById("cal-save") as HTMLButtonElement;
const calErr = document.getElementById("cal-err") as HTMLElement; // role="alert"
let flow: CalibrationFlow | null = null;
let calIndex = 0;
let outcome: Outcome | null = null;
let calChecks: boolean[] = [];

function calFail(message: string): void {
  calErr.replaceChildren();
  requestAnimationFrame(() => calErr.replaceChildren(document.createTextNode(message)));
}

function calShowStep(): void {
  if (!flow) return;
  calStep.hidden = false;
  calResult.hidden = true;
  const last = calIndex === flow.sentences.length - 1;
  calProgress.textContent = fmt("calProgress", { n: calIndex + 1, total: flow.sentences.length });
  calSentence.textContent = flow.sentences[calIndex];
  calInput.value = flow.getAnswer(calIndex);
  calBack.disabled = calIndex === 0;
  calNext.textContent = t(last ? "calFinish" : "calNext");
  calInput.focus();
}

function calLine(parent: HTMLElement, text: string, tag: "li" | "p" = "li", cls = ""): void {
  const el = document.createElement(tag);
  el.textContent = text;
  if (cls) el.className = cls;
  parent.append(el);
}

function calShowResult(): void {
  if (!flow || !outcome) return;
  calStep.hidden = true;
  calResult.hidden = false;
  calList.replaceChildren();
  calNotes.replaceChildren();
  const o = outcome;
  const none = o.suggestions.length === 0;
  const nothing = o.usable === 0; // nothing typed matched: "already fine" would be wrong
  // Nothing to save because a key was used for more than one letter: say so,
  // not "already fine" (word choice picks ө/ү for such a key).
  const ambiguous = none && o.conflicts.length > 0;
  calLead.textContent = t(nothing ? "calNothing" : ambiguous ? "calAmbiguous" : none ? "calNone" : "calSuggestLead");
  calSaveBtn.hidden = none;
  calRetry.hidden = !nothing;
  o.suggestions.forEach((s, i) => {
    const li = document.createElement("li");
    const label = document.createElement("label");
    label.className = "toggle";
    const box = document.createElement("input");
    box.type = "checkbox";
    box.checked = calChecks[i] ?? true;
    box.addEventListener("change", () => (calChecks[i] = box.checked));
    const text = document.createElement("span");
    text.textContent = fmt("calSuggestion", { letter: s.target, chunk: s.chunk, count: s.count });
    label.append(box, text);
    li.append(label);
    if (s.pair) calLine(li, fmt("calPair", { target: s.target }), "p", "hint");
    if (s.displaces) calLine(li, fmt("calDisplaces", { chunk: s.chunk, other: s.displaces }), "p", "hint");
    if (s.tie) calLine(li, t("calTie"), "p", "hint");
    calList.append(li);
  });
  for (const c of o.conflicts) calLine(calNotes, fmt("calConflict", { ...c }));
  for (const u of o.unsupported) calLine(calNotes, fmt("calUnsupported", { ...u }));
  for (const n of o.unmatchedSentences) calLine(calNotes, fmt("calUnmatched", { n }));
  for (const n of o.notTyped) calLine(calNotes, fmt("calNotTyped", { n }));
  if (o.skippedLetters.length > 0) calLine(calNotes, fmt("calSkipped", { letters: o.skippedLetters.join(", ") }));
  (nothing ? calRetry : none ? document.getElementById("cal-default") : calSaveBtn)?.focus();
}

function calGo(delta: number): void {
  if (!flow) return;
  flow.setAnswer(calIndex, calInput.value);
  if (delta > 0 && calIndex === flow.sentences.length - 1) {
    try {
      outcome = flow.finish();
    } catch {
      return calFail(t("calError"));
    }
    calChecks = outcome.suggestions.map(() => true);
    return calShowResult();
  }
  calIndex = Math.min(Math.max(calIndex + delta, 0), flow.sentences.length - 1);
  calShowStep();
}

/** Forget what was typed and clear the screens. Safe to call twice. */
function calDiscard(): void {
  flow?.cancel();
  flow = null;
  outcome = null;
  calChecks = [];
  calInput.value = "";
  calList.replaceChildren();
  calNotes.replaceChildren();
  calErr.replaceChildren();
}

/** Start from the first sentence with a fresh flow. */
function calStart(): void {
  calDiscard();
  flow = new CalibrationFlow(calibrationSentences);
  calIndex = 0;
  calShowStep();
}

document.getElementById("cal-open")?.addEventListener("click", () => {
  if (!ready) return cfgSay([t("calNotReady")], true);
  // A "close" event from an earlier dialog can still be queued; calStart
  // always builds new state, and the close handler below ignores an open dialog.
  if (!cal.open) cal.showModal();
  calStart();
});
calRetry.addEventListener("click", calStart);
calNext.addEventListener("click", () => calGo(1));
calBack.addEventListener("click", () => calGo(-1));
calInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.isComposing) {
    e.preventDefault();
    calGo(1);
  }
});
for (const id of ["cal-cancel", "cal-close", "cal-default"]) {
  document.getElementById(id)?.addEventListener("click", () => {
    cal.close();
    calDiscard();
  });
}
calSaveBtn.addEventListener("click", () => {
  if (!flow || !outcome) return;
  const accepted = calChecks.flatMap((on, i) => (on ? [i] : []));
  if (accepted.length === 0) return cal.close();
  let text: string;
  try {
    text = flow.save(config, accepted);
  } catch (e) {
    return calFail(saveError(e, (raw) => issueText(parseIssue(raw)), t("calError")));
  }
  setConfig(text);
  cal.close();
  calDiscard();
  cfgSay([accepted.length === 1 ? t("calSavedOne") : fmt("calSaved", { n: accepted.length })]);
});
// Every way out lands here (Esc, buttons, close() from script): forget what
// was typed. The event is queued after close(), so a dialog that was already
// reopened is left alone; calStart and calDiscard cover that case.
cal.addEventListener("close", () => {
  if (!cal.open) calDiscard();
});
document.addEventListener("buuz:lang", () => {
  if (!cal.open || !flow) return;
  if (!outcome) flow.setAnswer(calIndex, calInput.value); // keep what was typed
  if (outcome) calShowResult();
  else calShowStep();
});

const toast = document.getElementById("toast") as HTMLElement;
const clearX = document.getElementById("clear-x") as HTMLButtonElement;
const copyX = document.getElementById("copy-x") as HTMLButtonElement;
const pasteBtn = document.getElementById("paste") as HTMLButtonElement;
const pasteX = document.getElementById("paste-x") as HTMLButtonElement;

// Paste shows while the Latin box is empty; Clear takes its place once there is text.
function syncPaste(): void {
  const empty = latin.value === "";
  pasteBtn.hidden = !empty;
  pasteX.hidden = !empty;
  clearBtn.hidden = empty;
}

async function paste(): Promise<void> {
  const before = latin.value;
  const r = await readPaste(navigator.clipboard);
  if (r.kind === "text") {
    if (latin.value !== before) return; // the person typed while the read was pending
    latin.value = r.text;
    render();
    fit();
    syncCaret();
    latin.focus();
    animate("puff");
  } else {
    say(t(r.kind === "empty" ? "pasteEmpty" : "pasteBlocked"));
    latin.focus(); // the person can press Ctrl+V / long-press instead
  }
}
pasteBtn.addEventListener("click", () => void paste());
pasteX.addEventListener("click", () => void paste());

// Phones: Google Translate-style. Both boxes grow with their text (capped), so
// input, result and the copy button stay above the on-screen keyboard.
const phone = matchMedia("(max-width: 640px)");
function fit(): void {
  for (const el of [latin, cyr]) {
    el.rows = phone.matches ? 2 : 7;
    if (!phone.matches) {
      el.style.height = "";
      continue;
    }
    el.style.height = "auto";
    const cap = Math.round((window.visualViewport?.height ?? innerHeight) * (el === latin ? 0.3 : 0.32));
    el.style.height = Math.min(el.scrollHeight + 2, Math.max(cap, 96)) + "px";
  }
  clearX.hidden = latin.value === "";
  syncPaste();
}
phone.addEventListener("change", fit);
window.visualViewport?.addEventListener("resize", () => {
  fit();
  // Keep the toast just above the keyboard.
  const vv = window.visualViewport!;
  toast.style.bottom = Math.max(16, innerHeight - vv.height - vv.offsetTop + 16) + "px";
});

let statusTimer: number | undefined;
function say(msg: string): void {
  status.textContent = msg;
  toast.textContent = "✓ " + msg;
  toast.classList.add("show");
  window.clearTimeout(statusTimer);
  statusTimer = window.setTimeout(() => {
    status.textContent = "";
    toast.classList.remove("show");
  }, 1800);
}

async function copy(): Promise<void> {
  const text = cyr.value;
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    cyr.select(); // older browsers / non-secure contexts
    document.execCommand("copy");
    cyr.setSelectionRange(0, 0);
  }
  say(t("copied"));
  animate("hop");
}

latin.addEventListener("input", () => {
  render();
  fit();
  syncCaret();
  if (!mascot.classList.contains("puff")) animate("puff");
});
mascot.addEventListener("animationend", () => mascot.classList.remove("puff", "hop"));
copyBtn.addEventListener("click", () => void copy());
copyX.addEventListener("click", () => void copy());
clearX.addEventListener("click", () => clearBtn.click());
clearBtn.addEventListener("click", () => {
  latin.value = "";
  render();
  syncCaret();
  fit();
  latin.focus();
});
document.addEventListener("keydown", (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
    e.preventDefault();
    void copy();
  }
});

// Typewriter placeholder: while the Latin box is empty, a fun phrase types
// itself, pauses, deletes, and the next one starts. The Cyrillic box shows the
// live conversion as its placeholder. It only ever touches placeholders, so it
// sits still the moment there is real text, and resumes when the box is empty.
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)").matches;
const mongolianPool = (phrases as [string, string][]).map(([l]) => l);
let pool = shuffled(mongolianPool);
/** The typewriter shows examples of the typing language that is on. */
function ghostForScheme(): void {
  pool = shuffled(ghostPhrases(scheme) ?? mongolianPool);
  if (latin.value === "" && !isFocused()) showGhost("", false);
}
// While the demo types, hide the real caret so there's only one cursor.
// The moment there is real text, the native caret comes back.
// While the box has focus the ghost text is hidden entirely: it is distracting
// when you are about to type. It comes back on blur if the box is still empty.
// Always read the live focus state: autofocus can land before or after this
// script runs (it differs between browsers), so a cached flag can go stale.
const isFocused = (): boolean => document.activeElement === latin;
function syncCaret(): void {
  latin.classList.toggle("demo", latin.value === "" && !reducedMotion && !isFocused());
}
latin.addEventListener("focus", () => {
  latin.placeholder = "";
  cyr.placeholder = "";
  syncCaret();
});
latin.addEventListener("blur", () => {
  syncCaret();
  if (reducedMotion && latin.value === "" && ready) showGhost(pool[0], false);
});
// Autofocus can have happened before this script ran: nothing to show then.
if (isFocused()) {
  latin.placeholder = "";
  cyr.placeholder = "";
}
function showGhost(text: string, caret: boolean): void {
  if (isFocused()) {
    latin.placeholder = "";
    cyr.placeholder = "";
    return syncCaret();
  }
  syncCaret();
  latin.placeholder = text + (caret ? "|" : "");
  cyr.placeholder = !ready ? "" : typesMongolian() ? convert_with_config(text, ghostConfig) : convert_scheme(scheme, text);
}
function typewriter(): void {
  let p = 0;
  let chars: string[] = [];
  let n = 0;
  let phase: "type" | "hold" | "erase" | "gap" = "type";
  let blinks = 0;
  let paused = false;
  const next = (ms: number) => window.setTimeout(step, ms);
  let usedPool = pool;
  function step(): void {
    if (pool !== usedPool) {
      // The typing language changed: start a fresh phrase of the new language.
      usedPool = pool;
      chars = [];
      n = 0;
      p = 0;
      phase = "type";
    }
    if (latin.value !== "" || document.hidden || isFocused()) {
      // Paused (focus, text, hidden tab): the next run restarts from the first
      // letter of a fresh phrase, with the boxes empty, like real typing.
      paused = true;
      return void next(600);
    }
    if (paused) {
      paused = false;
      n = 0;
      chars = [];
      phase = "type";
      blinks = 0;
      p++;
      showGhost("", true);
    }
    if (chars.length === 0) chars = Array.from(pool[p % pool.length]);
    switch (phase) {
      case "type":
        n++;
        showGhost(chars.slice(0, n).join(""), true);
        if (n >= chars.length) { phase = "hold"; blinks = 0; }
        return void next(55 + Math.random() * 70);
      case "hold":
        showGhost(chars.join(""), blinks % 2 === 1);
        if (++blinks > 5) phase = "erase";
        return void next(420);
      case "erase":
        n = Math.max(0, n - 1);
        showGhost(chars.slice(0, n).join(""), true);
        if (n === 0) phase = "gap";
        return void next(28);
      case "gap":
        p++;
        chars = [];
        phase = "type";
        return void next(350);
    }
  }
  step();
}

init({ module_or_path: wasmUrl }).then(() => {
  const state = loadConfigState(store); // needs the wasm module (migration, parsing)
  config = state.text;
  (document.getElementById("cfg-notice") as HTMLElement).hidden = !state.recovered;
  // The page language decides first (a visitor on /uk/ types Ukrainian); then the saved choice.
  scheme = scheme_for_language(lang()) || (loadScheme(store, scheme_ids().split(",")) ?? MONGOLIAN);
  applyScheme();
  ghostForScheme();
  userText = loadUserDict(store); // also gives it to the converter
  showUserDict();
  applyConfig();
  ready = true;
  if (switchesOf(config).autocorrect) loadWords(); // applyConfig ran before ready
  render(); // text typed while the module was loading
  fit();
  if (reducedMotion) showGhost(pool[0], false);
  else typewriter();
});

// Feedback form: posts only what is typed into this form to /api/feedback.
// The converter text is never included.
const fb = document.getElementById("fb") as HTMLFormElement;
const fbSend = document.getElementById("fb-send") as HTMLButtonElement;
const fbStatus = document.getElementById("fb-status") as HTMLElement;
const FB_ERRORS: Record<string, Key> = {
  too_short: "fbTooShort",
  too_long: "fbTooLong",
  rate_limited: "fbRate",
  not_configured: "fbNotReady",
};
function fbSay(msg: string, ok: boolean): void {
  fbStatus.textContent = msg;
  fbStatus.className = "fb-status " + (ok ? "ok" : "err");
}
fb.addEventListener("submit", async (e) => {
  e.preventDefault();
  const f = new FormData(fb);
  const message = String(f.get("message") ?? "").trim();
  if (message.length < 5) return fbSay(t("fbTooShort"), false);
  fbSend.disabled = true;
  fbSay(t("fbSending"), true);
  try {
    const res = await fetch("/api/feedback", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        message,
        category: f.get("category"),
        contact: f.get("contact"),
        website: f.get("website"),
        lang: document.documentElement.lang,
      }),
    });
    const data = (await res.json().catch(() => ({}))) as { error?: string; url?: string; number?: number };
    if (res.ok) {
      fb.reset();
      fbSay(t("fbThanks"), true);
      if (data.url && data.url.startsWith("https://github.com/TengisCh/buuz/issues/")) {
        const a = document.createElement("a");
        a.href = data.url;
        a.target = "_blank";
        a.rel = "noopener";
        a.textContent = ` ${t("fbIssue")}${data.number ? ` #${data.number}` : ""}`;
        fbStatus.append(a);
      }
    } else {
      fbSay(t(FB_ERRORS[data.error ?? ""] ?? "fbError"), false);
    }
  } catch {
    fbSay(t("fbError"), false);
  } finally {
    fbSend.disabled = false;
  }
});

// Floating feedback widget: a chat button bottom-right opens a small panel.
// Non-modal: the page stays usable; Esc or a click outside closes it.
const fab = document.getElementById("fb-fab") as HTMLButtonElement;
const panel = document.getElementById("fb-panel") as HTMLElement;
const fbMsg = document.getElementById("fb-msg") as HTMLTextAreaElement;
let lastFocus: HTMLElement | null = null;
function openPanel(): void {
  if (!panel.hidden) return;
  lastFocus = document.activeElement as HTMLElement | null;
  panel.hidden = false;
  fab.setAttribute("aria-expanded", "true");
  fab.classList.add("open");
  requestAnimationFrame(() => panel.classList.add("show"));
  (panel.querySelector('[role="tab"][aria-selected="true"]')?.id === "tab-fb" ? fbMsg : (panel.querySelector(".mail") as HTMLElement)).focus();
}
function closePanel(restore = true): void {
  if (panel.hidden) return;
  panel.classList.remove("show");
  panel.hidden = true;
  fab.setAttribute("aria-expanded", "false");
  fab.classList.remove("open");
  if (restore) (lastFocus ?? fab).focus();
}
fab.addEventListener("click", () => (panel.hidden ? openPanel() : closePanel()));
panel.querySelector(".fb-close")?.addEventListener("click", () => closePanel());
for (const el of document.querySelectorAll("[data-open-feedback]")) el.addEventListener("click", openPanel);
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && !panel.hidden) closePanel();
});
document.addEventListener("pointerdown", (e) => {
  const target = e.target as Node;
  if (!panel.hidden && !panel.contains(target) && !fab.contains(target)) closePanel(false);
});

// Tabs: Feedback (public GitHub issue) / Contact (private email).
const tabs = Array.from(panel.querySelectorAll<HTMLButtonElement>('[role="tab"]'));
function selectTab(tab: HTMLButtonElement, focus = false): void {
  for (const t of tabs) {
    const on = t === tab;
    t.setAttribute("aria-selected", String(on));
    t.tabIndex = on ? 0 : -1;
    (document.getElementById(t.getAttribute("aria-controls") ?? "") as HTMLElement).hidden = !on;
  }
  if (focus) tab.focus();
}
for (const tab of tabs) {
  tab.addEventListener("click", () => selectTab(tab));
  tab.addEventListener("keydown", (e) => {
    const i = tabs.indexOf(tab);
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
      e.preventDefault();
      selectTab(tabs[(i + (e.key === "ArrowRight" ? 1 : tabs.length - 1)) % tabs.length], true);
    }
  });
}

// Footer links on other pages point to /?feedback=1 to open the panel here.
if (new URLSearchParams(location.search).has("feedback")) {
  history.replaceState(null, "", location.pathname);
  openPanel();
}
