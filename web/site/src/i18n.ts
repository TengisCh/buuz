// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// Tiny two-language switch. Mongolian is the default and is also what
// index.html ships, so first paint and search engines see Mongolian.
// English is opt-in: the toggle button or ?lang=en. The choice is saved.
// We deliberately do NOT follow navigator.language: many Mongolians run
// English-language phones and browsers, and they are this site's audience.

export type Lang = "mn" | "en";

const mn = {
  title: "Buuz: Латин үсгээр монголоор бич",
  description:
    "Латинаар бичихэд шууд кирилл болно. Үнэгүй, бичсэн зүйл тань хаашаа ч илгээгдэхгүй.",
  tag: "Латинаар бич, кирилл болоод гарчихна.",
  latin: "Латин",
  cyr: "Кирилл",
  clear: "Цэвэрлэх",
  copy: "Хуулах",
  copied: "Хуулагдлаа",
  privacy:
    "Бүх зүйл таны төхөөрөмж дээр ажиллана. Бичсэн зүйл тань хаашаа ч илгээгдэхгүй.",
  howTo: "Яаж бичих вэ?",
  howToBody:
    "<kbd>o</kbd>, <kbd>u</kbd> хоёр эгшиг зохицох ёсоор о/ө, у/ү-гийн алийг нь өөрөө сонгоно. Жишээ нь <kbd>geree</kbd> гэвэл гэрээ, <kbd>gar</kbd> гэвэл гар болно. Заавал ө, ү гаргах бол <kbd>q</kbd>, <kbd>w</kbd>-г ашиглаарай.",
  appTitle: "Ямар ч апп дээр монголоор бич",
  appBody:
    "Buuz удахгүй Windows, macOS-ийн гар болж гарна. Энэ хуудастай яг ижил дүрмээр, хүссэн апп дотроо шууд бичнэ.",
  soon: "удахгүй",
  github: "GitHub дээр нээлттэй эх",
  credit:
    'Buuz 2.0 бол Нямцэрэнгийн Одбаярын бүтээсэн <span class="nw"><a href="https://github.com/odbayar/buuz">Buuz</a>-ийн</span> үргэлжлэл.',
  switchTo: "EN",
  switchLabel: "Switch to English",
  vowels: "Эгшиг",
  yVowels: "Й-тэй эгшиг",
  consonants: "Гийгүүлэгч",
  signs: "Тэмдэг",
  feedbackTitle: "Санал хүсэлт",
  feedbackBody: "Алдаа олсон уу, эсвэл гоё санаа байна уу? GitHub дээр бидэнд бичээрэй.",
  feedbackBtn: "Санал илгээх",
  themeDark: "Харанхуй горим",
  themeLight: "Цайвар горим",
};

export type Key = keyof typeof mn;

// Same keys as mn, enforced by the type.
const en: Record<Key, string> = {
  title: "Buuz: type Mongolian with a Latin keyboard",
  description:
    "Type Latin letters, get Mongolian Cyrillic instantly. Free, and nothing you type leaves your device.",
  tag: "Type Mongolian with a Latin keyboard.",
  latin: "Latin",
  cyr: "Cyrillic",
  clear: "Clear",
  copy: "Copy",
  copied: "Copied",
  privacy:
    "Everything runs on your device. What you type is never sent anywhere.",
  howTo: "How to type",
  howToBody:
    "<kbd>o</kbd> and <kbd>u</kbd> pick о/ө and у/ү by vowel harmony, so <kbd>geree</kbd> gives гэрээ and <kbd>gar</kbd> gives гар. To force a letter, use <kbd>q</kbd> or <kbd>w</kbd> for ө and ү.",
  appTitle: "Type Mongolian in every app",
  appBody:
    "Buuz is also coming as a keyboard for Windows and macOS, with the same rules as this page, so you can type straight into any app.",
  soon: "coming soon",
  github: "Open source on GitHub",
  credit: 'Buuz 2.0 continues <a href="https://github.com/odbayar/buuz">Buuz</a> by Odbayar Nyamtseren.',
  switchTo: "МН",
  switchLabel: "Монгол хэл рүү шилжих",
  vowels: "Vowels",
  yVowels: "Y-vowels",
  consonants: "Consonants",
  signs: "Signs",
  feedbackTitle: "Feedback",
  feedbackBody: "Found a bug or have a fun idea? Tell us on GitHub.",
  feedbackBtn: "Send feedback",
  themeDark: "Dark mode",
  themeLight: "Light mode",
};

const strings: Record<Lang, Record<Key, string>> = { mn, en };

const STORE = "buuz.lang";
let current: Lang = "mn";

export function t(key: Key): string {
  return strings[current][key];
}

function save(lang: Lang): void {
  try {
    localStorage.setItem(STORE, lang);
  } catch {
    /* private mode etc.: just don't remember */
  }
}

function initial(): Lang {
  const q = new URLSearchParams(location.search).get("lang");
  if (q === "mn" || q === "en") {
    save(q);
    return q;
  }
  try {
    const s = localStorage.getItem(STORE);
    if (s === "mn" || s === "en") return s;
  } catch {
    /* ignore */
  }
  return "mn";
}

export function apply(lang: Lang): void {
  current = lang;
  const root = document.documentElement;
  root.lang = lang;
  document.title = t("title");
  document
    .querySelector('meta[name="description"]')
    ?.setAttribute("content", t("description"));
  // Text-only keys use textContent; keys with markup (<kbd>, links) are ours.
  for (const el of document.querySelectorAll<HTMLElement>("[data-i18n]")) {
    el.textContent = t(el.dataset.i18n as Key);
  }
  for (const el of document.querySelectorAll<HTMLElement>("[data-i18n-html]")) {
    el.innerHTML = t(el.dataset.i18nHtml as Key);
  }
  const btn = document.getElementById("lang") as HTMLButtonElement | null;
  if (btn) {
    btn.textContent = t("switchTo");
    btn.setAttribute("aria-label", t("switchLabel"));
    btn.title = t("switchLabel");
    btn.lang = lang === "mn" ? "en" : "mn";
  }
  document.dispatchEvent(new Event("buuz:lang"));
}

export function setupLang(): void {
  const lang = initial();
  if (lang !== "mn") apply(lang); // HTML already ships Mongolian
  else current = "mn";
  document.getElementById("lang")?.addEventListener("click", () => {
    const next: Lang = current === "mn" ? "en" : "mn";
    save(next);
    apply(next);
  });
}
