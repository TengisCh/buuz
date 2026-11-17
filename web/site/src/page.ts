// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// Shared script for the simple pages (privacy, cookies, 404). Each page ships
// both languages; this shows the one the visitor chose on the main page
// (same "buuz.lang" key) and lets them switch.
import "@fontsource-variable/nunito/wght.css";
import "./style.css";
import { apply } from "./i18n";

type Lang = "mn" | "en";
const KEY = "buuz.lang";

function saved(): Lang {
  const q = new URLSearchParams(location.search).get("lang");
  if (q === "mn" || q === "en") return q;
  try {
    return localStorage.getItem(KEY) === "en" ? "en" : "mn";
  } catch {
    return "mn";
  }
}

function show(lang: Lang): void {
  const desc = document.querySelector('meta[name="description"]')?.getAttribute("content") ?? "";
  apply(lang); // footer and shared labels
  document.querySelector('meta[name="description"]')?.setAttribute("content", desc);
  for (const el of document.querySelectorAll<HTMLElement>("[data-lang]")) el.hidden = el.dataset.lang !== lang;
  const title = document.querySelector<HTMLElement>(`[data-title-${lang}]`);
  if (title) document.title = title.dataset[`title${lang === "mn" ? "Mn" : "En"}`] ?? document.title;
}

let lang = saved();
show(lang);
document.getElementById("lang")?.addEventListener("click", () => {
  lang = lang === "mn" ? "en" : "mn";
  try {
    localStorage.setItem(KEY, lang);
  } catch {
    /* ignore */
  }
  show(lang);
});
