// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

import "@fontsource-variable/nunito/wght.css";
import "./style.css";
import init, { convert } from "./wasm/buuz.js";
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
  syncCaret();
  if (!mascot.classList.contains("puff")) animate("puff");
});
mascot.addEventListener("animationend", () => mascot.classList.remove("puff", "hop"));
copyBtn.addEventListener("click", () => void copy());
clearBtn.addEventListener("click", () => {
  latin.value = "";
  render();
  syncCaret();
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
function syncCaret(): void {
  latin.classList.toggle("demo", latin.value === "" && !reducedMotion);
}
function showGhost(text: string, caret: boolean): void {
  syncCaret();
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
    const data = (await res.json().catch(() => ({}))) as { error?: string };
    if (res.ok) {
      fb.reset();
      fbSay(t("fbThanks"), true);
    } else {
      fbSay(t(FB_ERRORS[data.error ?? ""] ?? "fbError"), false);
    }
  } catch {
    fbSay(t("fbError"), false);
  } finally {
    fbSend.disabled = false;
  }
});
