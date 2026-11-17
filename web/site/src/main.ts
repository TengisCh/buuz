// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

import "./style.css";
import init, { convert } from "./wasm/buuz.js";
import wasmUrl from "./wasm/buuz_bg.wasm?url";

const latin = document.getElementById("latin") as HTMLTextAreaElement;
const cyr = document.getElementById("cyr") as HTMLTextAreaElement;
const copyBtn = document.getElementById("copy") as HTMLButtonElement;
const clearBtn = document.getElementById("clear") as HTMLButtonElement;
const status = document.getElementById("status") as HTMLElement;

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
  say("Copied");
}

latin.addEventListener("input", render);
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

init({ module_or_path: wasmUrl }).then(() => {
  ready = true;
  render(); // text typed while the module was loading
});
