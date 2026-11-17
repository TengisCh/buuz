// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

import "@fontsource-variable/nunito/wght.css";
import "./style.css";
import init, { convert_with } from "./wasm/buuz.js";
import wasmUrl from "./wasm/buuz_bg.wasm?url";
import { setupLang, t, type Key } from "./i18n";
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

// "Smart vowels" (on by default): look-ahead harmony, ий/ы, the уу/үү particle.
// Off gives the classic left-to-right behaviour of the original Buuz.
const smartBox = document.getElementById("smart") as HTMLInputElement;
let smart = true;
try {
  smart = localStorage.getItem("buuz.smart") !== "off";
} catch {
  /* private mode: default on */
}
smartBox.checked = smart;
smartBox.addEventListener("change", () => {
  smart = smartBox.checked;
  try {
    localStorage.setItem("buuz.smart", smart ? "on" : "off");
  } catch {
    /* private mode: don't remember */
  }
  render();
});

// "Abbreviations" (on by default): bn + space or punctuation -> байна (never at the end of the text).
const abbrBox = document.getElementById("abbr") as HTMLInputElement;
let abbr = true;
try {
  abbr = localStorage.getItem("buuz.abbr") !== "off";
} catch {
  /* private mode: default on */
}
abbrBox.checked = abbr;
abbrBox.addEventListener("change", () => {
  abbr = abbrBox.checked;
  try {
    localStorage.setItem("buuz.abbr", abbr ? "on" : "off");
  } catch {
    /* private mode: don't remember */
  }
  render();
});

function render(): void {
  // convert_with() is microseconds even for long pastes, so run it synchronously
  // on every input event: no debounce, no frame delay.
  cyr.value = ready ? convert_with(latin.value.normalize("NFC"), smart, abbr) : "";
}

const toast = document.getElementById("toast") as HTMLElement;
const clearX = document.getElementById("clear-x") as HTMLButtonElement;
const copyX = document.getElementById("copy-x") as HTMLButtonElement;

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
const pool = (phrases as [string, string][]).map(([l]) => l);
for (let i = pool.length - 1; i > 0; i--) {
  const j = Math.floor(Math.random() * (i + 1));
  [pool[i], pool[j]] = [pool[j], pool[i]];
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
  cyr.placeholder = ready ? convert_with(text, smart, false) : "";
}
function typewriter(): void {
  let p = 0;
  let chars: string[] = [];
  let n = 0;
  let phase: "type" | "hold" | "erase" | "gap" = "type";
  let blinks = 0;
  let paused = false;
  const next = (ms: number) => window.setTimeout(step, ms);
  function step(): void {
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
  ready = true;
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
