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
    "<kbd>o</kbd>, <kbd>u</kbd> хоёр эгшиг зохицох ёсоор о/ө, у/ү-гийн алийг нь өөрөө сонгоно. Жишээ нь <kbd>geree</kbd> гэвэл гэрээ, <kbd>gar</kbd> гэвэл гар болно. Заавал ө, ү гаргах бол <kbd>q</kbd>, <kbd>w</kbd>-г (эсвэл <kbd>ö</kbd>, <kbd>ü</kbd>-г) ашиглаарай. Үгийн хойно орох эгшгийг ч харж сонгодог тул <kbd>unee</kbd> гэвэл үнээ болно. Эр үгэнд <kbd>ii</kbd> нь ы болно (<kbd>aaviin</kbd> гэвэл аавын). Заавал ий гаргах бол <kbd>ii'</kbd>.",
  abbreviations: "Товчлол (bn → байна)",
  abbreviationsHint:
    "Товчилсон үгийн дараа зай эсвэл цэг, таслал бичихэд бүтнээр нь бичигдэнэ (bn + зай → байна). Энэ сайт дээр тэр зайг арилгавал товчлол буцна. Удахгүй гарах компьютерийн гарт зайны дараа Backspace дарвал буцаана. Унтраавал хэзээ ч өргөжихгүй.",
  mode: "Горим",
  modeSmart: "Ухаалаг (үндсэн)",
  modeClassic: "Сонгодог (анхны Buuz)",
  modeHint:
    "Ухаалаг горимд доорх нэмэлт боломжууд ажиллана. Сонгодог горим нь анхны Buuz-ийн дүрмээр яг хэвээр нь, нэмэлт боломжгүйгээр бичнэ.",
  smartVowels: "Ухаалаг эгшиг",
  smartVowelsHint:
    "Үгийн эгшгийг урагш харж сонгоно (unee → үнээ, aaviin → аавын). Унтраавал анхны Buuz-ийн дүрмээр бичнэ.",
  appTitle: "Ямар ч апп дээр монголоор бич",
  appBody:
    "Buuz удахгүй Windows, macOS-ийн гар болж гарна. Энэ хуудастай яг ижил дүрмээр, хүссэн апп дотроо шууд бичнэ.",
  soon: "удахгүй",
  github: "GitHub дээр нээлттэй эх",
  credit:
    'Buuz 2.0 бол Нямцэрэнгийн Одбаярын бүтээсэн <span class="nw"><a href="https://github.com/odbayar/buuz">Buuz</a>-ийн</span> үргэлжлэл.',
  skip: "Хөрвүүлэгч рүү очих",
  close: "Хаах",
  footProduct: "Бүтээгдэхүүн",
  footWeb: "Вэб хөрвүүлэгч",
  footCommunity: "Нээлттэй эх",
  footSponsor: "Дэмжих",
  footLegal: "Эрх зүй",
  footPrivacy: "Нууцлалын мэдэгдэл",
  footCookies: "Күүкийн бодлого",
  footLicense: "Лиценз",
  tabFeedback: "Санал",
  tabContact: "Холбоо барих",
  fbThanksCat: "Талархал",
  fbCollab: "Хамтран ажиллах",
  contactBody: "Хувийн асуулт, хамтын ажиллагаа, хэвлэлийн хүсэлт байвал шууд имэйлээр бичээрэй. Энэ нь нийтэд харагдахгүй.",
  fbIssue: "Явцыг нь эндээс хараарай →",
  switchTo: "EN",
  switchLabel: "EN: Switch to English",
  vowels: "Эгшиг",
  yVowels: "Й-тэй эгшиг",
  consonants: "Гийгүүлэгч",
  signs: "Тэмдэг",
  faqTitle: "Түгээмэл асуулт",
  faq0q: "Латинаар кирилл хэрхэн бичих вэ?",
  faq0a: "Дээрх нүдэнд латин үсгээр бичихэд шууд кирилл болно. Жишээ нь <kbd>sain baina uu</kbd> гэж бичвэл «сайн байна уу» болно. Гарсан кирилл текстээ «Хуулах» товчоор хуулаад хүссэн газраа буулгаарай.",
  faq1q: "Латин кирилл хөрвүүлэгч үнэгүй юу?",
  faq1a: "Тийм, үнэгүй бөгөөд нээлттэй эх. Бүртгүүлэх, нэвтрэх шаардлагагүй.",
  faq2q: "Ө, Ү үсгийг яаж бичих вэ?",
  faq2a: "<kbd>q</kbd>, <kbd>ö</kbd> нь ө, <kbd>w</kbd>, <kbd>ü</kbd> нь ү болно. <kbd>o</kbd>, <kbd>u</kbd> нь эгшиг зохицох ёсоор о/ө, у/ү-гийн алийг нь өөрөө сонгоно: <kbd>gar</kbd> → гар, <kbd>geree</kbd> → гэрээ.",
  faq3q: "Бичсэн зүйл маань хаа нэгтээ илгээгдэх үү?",
  faq3a: "Үгүй. Хөрвүүлэлт бүхэлдээ таны төхөөрөмж дээр ажиллана, бичсэн текст хаашаа ч илгээгдэхгүй.",
  faq4q: "Утсан дээр ашиглаж болох уу?",
  faq4a: "Болно. Утасны хөтөч дээр шууд ажиллана. Windows, macOS-ийн гар удахгүй гарна.",
  faq5q: "Уламжлалт монгол бичгээр (босоо) бичиж болох уу?",
  faq5a: "Одоохондоо болохгүй, зөвхөн кирилл үсгээр бичнэ.",
  faq6q: "Buuz гэж юу вэ?",
  faq6a: "Нямцэрэнгийн Одбаярын бүтээсэн Buuz хөтөлбөрийн үргэлжлэл. Дүрэм нь адилхан, одоо нээлттэй эхээр шинэчлэгдэж байна.",
  feedbackTitle: "Санал хүсэлт",
  feedbackBody: "Алдаа олсон уу, эсвэл гоё санаа байна уу? Доор бичээд илгээчихээрэй.",
  feedbackBtn: "Илгээх",
  fbMessage: "Таны санал",
  fbMessagePh: "Юу бодож байна, бидэнд хэлээрэй...",
  fbCategory: "Төрөл",
  fbBug: "Алдаа",
  fbIdea: "Санаа",
  fbOther: "Бусад",
  fbContact: "Холбоо барих (заавал биш)",
  fbContactPh: "Имэйл эсвэл GitHub (заавал биш)",
  fbPublic: "Санал тань GitHub дээр нийтэд харагдана.",
  fbSending: "Илгээж байна...",
  fbThanks: "Баярлалаа! Санал тань илгээгдлээ 🎉",
  fbTooShort: "Арай дэлгэрэнгүй бичээрэй.",
  fbTooLong: "Хэт урт байна, 2000 тэмдэгтээс бага байлгаарай.",
  fbRate: "Хэт олон удаа илгээлээ. Түр хүлээгээд дахин оролдоорой.",
  fbNotReady: "Санал илгээх хэсэг одоохондоо ажиллахгүй байна. GitHub холбоосоор бичээрэй.",
  fbError: "Илгээж чадсангүй. Дахин оролдоорой.",
  fbGithub: "GitHub дээр бичих",
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
    "<kbd>o</kbd> and <kbd>u</kbd> pick о/ө and у/ү by vowel harmony, so <kbd>geree</kbd> gives гэрээ and <kbd>gar</kbd> gives гар. To force a letter, use <kbd>q</kbd> or <kbd>w</kbd> (or <kbd>ö</kbd> or <kbd>ü</kbd>) for ө and ү. Buuz also looks ahead in the word, so <kbd>unee</kbd> gives үнээ, and <kbd>ii</kbd> becomes ы in masculine words (<kbd>aaviin</kbd> gives аавын). To force ий, type <kbd>ii'</kbd>.",
  abbreviations: "Abbreviations (bn → байна)",
  abbreviationsHint:
    "A short form expands when you type a space or punctuation after it (bn then a space gives байна). On this website, deleting that space brings the abbreviation back; in the upcoming desktop keyboards, Backspace right after the space undoes it. Turn the checkbox off to never expand.",
  mode: "Mode",
  modeSmart: "Smart (default)",
  modeClassic: "Classic (original Buuz)",
  modeHint:
    "Smart mode adds the features below. Classic types exactly like the original Buuz, with every smart feature off.",
  smartVowels: "Smart vowels",
  smartVowelsHint:
    "Picks vowels by looking ahead in the word (unee gives үнээ, aaviin gives аавын). Turn off for the original Buuz rules.",
  appTitle: "Type Mongolian in every app",
  appBody:
    "Buuz is also coming as a keyboard for Windows and macOS, with the same rules as this page, so you can type straight into any app.",
  soon: "coming soon",
  github: "Open source on GitHub",
  credit: 'Buuz 2.0 continues <a href="https://github.com/odbayar/buuz">Buuz</a> by Odbayar Nyamtseren.',
  skip: "Skip to the converter",
  close: "Close",
  footProduct: "Product",
  footWeb: "Web converter",
  footCommunity: "Open source",
  footSponsor: "Sponsor",
  footLegal: "Legal",
  footPrivacy: "Privacy notice",
  footCookies: "Cookie policy",
  footLicense: "License",
  tabFeedback: "Feedback",
  tabContact: "Contact",
  fbThanksCat: "Thanks",
  fbCollab: "Collaborate",
  contactBody: "Private questions, collaboration or press? Email us directly. This isn't public.",
  fbIssue: "Follow it here →",
  switchTo: "МН",
  switchLabel: "МН: Монгол хэл рүү шилжих",
  vowels: "Vowels",
  yVowels: "Y-vowels",
  consonants: "Consonants",
  signs: "Signs",
  faqTitle: "Frequently asked questions",
  faq0q: "How do I type Mongolian Cyrillic with a Latin keyboard?",
  faq0a: "Type Latin letters in the top box and Cyrillic appears right away. For example <kbd>sain baina uu</kbd> becomes «сайн байна уу». Press Copy and paste the result wherever you like.",
  faq1q: "Is the Latin to Cyrillic converter free?",
  faq1a: "Yes, it is free and open source. No sign-up or login.",
  faq2q: "How do I type Ө and Ү?",
  faq2a: "<kbd>q</kbd> or <kbd>ö</kbd> gives ө and <kbd>w</kbd> or <kbd>ü</kbd> gives ү. <kbd>o</kbd> and <kbd>u</kbd> choose о/ө and у/ү by vowel harmony: <kbd>gar</kbd> → гар, <kbd>geree</kbd> → гэрээ.",
  faq3q: "Is what I type sent anywhere?",
  faq3a: "No. Conversion runs entirely on your device and the text never leaves it.",
  faq4q: "Can I use it on my phone?",
  faq4a: "Yes, it works in your phone's browser. Keyboards for Windows and macOS are coming soon.",
  faq5q: "Can it write the traditional vertical Mongol script?",
  faq5a: "Not yet. For now it writes Cyrillic only.",
  faq6q: "What is Buuz?",
  faq6a: "A continuation of Buuz by Odbayar Nyamtseren, with the same typing rules, now being renewed as open source.",
  feedbackTitle: "Feedback",
  feedbackBody: "Found a bug or have a fun idea? Write it below and send it.",
  feedbackBtn: "Send",
  fbMessage: "Your feedback",
  fbMessagePh: "What's on your mind?",
  fbCategory: "Type",
  fbBug: "Bug",
  fbIdea: "Idea",
  fbOther: "Other",
  fbContact: "Contact (optional)",
  fbContactPh: "Email or GitHub (optional)",
  fbPublic: "Your feedback is posted publicly on GitHub.",
  fbSending: "Sending...",
  fbThanks: "Thank you! Your feedback was sent 🎉",
  fbTooShort: "Please write a little more.",
  fbTooLong: "Too long, please keep it under 2000 characters.",
  fbRate: "Too many messages. Please wait a bit and try again.",
  fbNotReady: "The feedback form isn't switched on yet. Please use the GitHub link instead.",
  fbError: "Couldn't send. Please try again.",
  fbGithub: "Write on GitHub",
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
  for (const el of document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>("[data-i18n-ph]")) {
    el.placeholder = t(el.dataset.i18nPh as Key);
  }
  for (const el of document.querySelectorAll<HTMLElement>("[data-i18n-aria]")) {
    const label = t(el.dataset.i18nAria as Key);
    el.setAttribute("aria-label", label);
    el.title = label;
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
