// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

import "@fontsource-variable/nunito/wght.css";
import "./style.css";
import init, { convert } from "./wasm/buuz.js";
import wasmUrl from "./wasm/buuz_bg.wasm?url";
import { setupLang, t } from "./i18n";
import phrases from "./phrases.json";

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

function render(): void {
  // convert() is microseconds even for long pastes, so run it synchronously
  // on every input event: no debounce, no frame delay.
  cyr.value = ready ? convert(latin.value) : "";
}

let statusTimer: number | undefined;
function say(msg: string): void {
  status.textContent = msg;
  window.clearTimeout(statusTimer);
  statusTimer = window.setTimeout(() => (status.textContent = ""), 1800);
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
  if (!mascot.classList.contains("puff")) animate("puff");
});
mascot.addEventListener("animationend", () => mascot.classList.remove("puff", "hop"));
copyBtn.addEventListener("click", () => void copy());
clearBtn.addEventListener("click", () => {
  latin.value = "";
  render();
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
const pool = (phrases as [string, string][]).map(([l]) => l);
for (let i = pool.length - 1; i > 0; i--) {
  const j = Math.floor(Math.random() * (i + 1));
  [pool[i], pool[j]] = [pool[j], pool[i]];
}
function showGhost(text: string, caret: boolean): void {
  latin.placeholder = text + (caret ? "|" : "");
  cyr.placeholder = ready ? convert(text) : "";
}
function typewriter(): void {
  let p = 0;
  let chars: string[] = [];
  let n = 0;
  let phase: "type" | "hold" | "erase" | "gap" = "type";
  let blinks = 0;
  const next = (ms: number) => window.setTimeout(step, ms);
  function step(): void {
    if (latin.value !== "" || document.hidden) return void next(600);
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
  ready = true;
  render(); // text typed while the module was loading
  if (matchMedia("(prefers-reduced-motion: reduce)").matches) showGhost(pool[0], false);
  else typewriter();
});
