// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
//
// Writes one static page per page language: dist/en/index.html and
// dist/ru/index.html, made from the built dist/index.html (Mongolian). Search
// engines read these pages without running any script, so each page carries
// its own language, title, description, canonical link, hreflang links and
// FAQ data. The page script then takes over, as it does on the Mongolian page.
//
// Run by `npm run build` after `vite build`. No dependencies: the text comes
// from src/i18n.ts, and the page is changed with small string rewrites.

import { readFile, writeFile, mkdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { ALL_LANGS, PATHS as LANG_PATHS, strings } from "../src/i18n.ts";

export const ORIGIN = "https://buuz.thetengis.com";
export const PATHS = LANG_PATHS;
const LOCALES = { mn: "mn_MN", en: "en_US", ru: "ru_RU", uk: "uk_UA", be: "be_BY", bg: "bg_BG", sr: "sr_RS", mk: "mk_MK", kk: "kk_KZ", ky: "ky_KG" };
// One example of each language, for the social description.
const EXAMPLE = { uk: "pryvit → привіт", be: "pryvitanne → прывітанне", bg: "zdravey → здравей", sr: "hvala → хвала", mk: "zdravo → здраво", kk: "saelem → сәлем", ky: "jakshy → жакшы" };
const ALT = {
  uk: "Buuz 2.0: pryvit → привіт",
  be: "Buuz 2.0: pryvitanne → прывітанне",
  bg: "Buuz 2.0: zdravey → здравей",
  sr: "Buuz 2.0: hvala → хвала",
  mk: "Buuz 2.0: zdravo → здраво",
  kk: "Buuz 2.0: saelem → сәлем",
  ky: "Buuz 2.0: jakshy → жакшы",
  mn: "Buuz 2.0: sain baina uu гэж бичихэд сайн байна уу болно",
  en: "Buuz 2.0: sain baina uu becomes сайн байна уу",
  ru: "Buuz 2.0: sain baina uu превращается в сайн байна уу",
};

const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const escAttr = (s) => esc(s).replace(/"/g, "&quot;");
/** The text of a string with markup: everything outside `<...>`. A scan, not a pattern, so no tag can be rebuilt from what is left. */
function plain(s) {
  let out = "";
  let depth = 0;
  for (const c of s) {
    if (c === "<") depth++;
    else if (c === ">" && depth > 0) depth--;
    else if (depth === 0) out += c;
  }
  return out;
}

/** Set (or add) an attribute in one opening tag. */
function setAttr(tag, name, value) {
  const re = new RegExp(`(\\s${name}=)"[^"]*"`);
  return re.test(tag) ? tag.replace(re, `$1"${escAttr(value)}"`) : tag.replace(/\s*\/?>$/, (end) => ` ${name}="${escAttr(value)}"${end}`);
}

/** The strings of a language. A key it lacks is English. */
function textOf(lang) {
  return { ...strings.en, ...strings[lang] };
}

/** The FAQPage data of one language, built from the faq strings. */
export function faqJson(lang) {
  const s = textOf(lang);
  const mainEntity = [];
  for (let i = 0; s[`faq${i}q`]; i++) {
    mainEntity.push({
      "@type": "Question",
      name: s[`faq${i}q`],
      acceptedAnswer: { "@type": "Answer", text: plain(s[`faq${i}a`]) },
    });
  }
  // "<" is escaped so the data can never close its own <script> element.
  return JSON.stringify({ "@context": "https://schema.org", "@type": "FAQPage", inLanguage: lang, mainEntity }, null, 2).replace(/</g, "\\u003c");
}

/** The head links that say which page is which language. */
export function hreflangLinks() {
  const links = ALL_LANGS.map((l) => `<link rel="alternate" hreflang="${l}" href="${ORIGIN}${PATHS[l]}" />`);
  links.push(`<link rel="alternate" hreflang="x-default" href="${ORIGIN}/" />`);
  return links.join("\n    ");
}

/** Make the page of language `lang` from the Mongolian page `html`. */
export function render(html, lang) {
  const s = textOf(lang);
  const url = ORIGIN + PATHS[lang];
  const ogTitle = s.title.replace(/^Buuz:/, "Buuz 2.0:");
  const ogDescription = `${EXAMPLE[lang] ?? "sain baina uu → сайн байна уу"}. ${s.description}`;
  let out = html;

  out = out.replace(/<html lang="[^"]*"/, `<html lang="${lang}"`);
  out = out.replace(/<title>[\s\S]*?<\/title>/, `<title>${esc(s.title)}</title>`);
  out = out.replace(/(<meta\s+name="description"\s+content=")[^"]*(")/, `$1${escAttr(s.description)}$2`);
  out = out.replace(/(<link rel="canonical" href=")[^"]*(")/, `$1${url}$2`);
  out = out.replace(/(<meta property="og:url" content=")[^"]*(")/, `$1${url}$2`);
  out = out.replace(/(<meta property="og:title" content=")[^"]*(")/, `$1${escAttr(ogTitle)}$2`);
  out = out.replace(/(<meta property="og:description" content=")[^"]*(")/, `$1${escAttr(ogDescription)}$2`);
  out = out.replace(/(<meta property="og:image:alt" content=")[^"]*(")/, `$1${escAttr(ALT[lang])}$2`);
  out = out.replace(/(<meta name="twitter:title" content=")[^"]*(")/, `$1${escAttr(ogTitle)}$2`);
  out = out.replace(/(<meta name="twitter:description" content=")[^"]*(")/, `$1${escAttr(ogDescription)}$2`);
  out = out.replace(/<meta property="og:locale" content="[^"]*" \/>(?:\s*<meta property="og:locale:alternate"[^>]*\/>)+/, [
    `<meta property="og:locale" content="${LOCALES[lang]}" />`,
    ...Object.keys(LOCALES).filter((l) => l !== lang).map((l) => `<meta property="og:locale:alternate" content="${LOCALES[l]}" />`),
  ].join("\n    "));
  out = out.replace(
    /<script type="application\/ld\+json">\s*\{\s*"@context": "https:\/\/schema\.org",\s*"@type": "FAQPage"[\s\S]*?<\/script>/,
    `<script type="application/ld+json">\n${faqJson(lang)}\n    </script>`,
  );

  // Text, placeholders and labels: the same keys the page script uses.
  const el = /<([a-z0-9]+)\b([^>]*?)\sdata-i18n="(\w+)"([^>]*)>([\s\S]*?)<\/\1>/g;
  out = out.replace(el, (m, tag, a, key, b, _inner) => (key in s ? `<${tag}${a} data-i18n="${key}"${b}>${esc(s[key])}</${tag}>` : m));
  out = out.replace(/<([a-z0-9]+)\b([^>]*?)\sdata-i18n-html="(\w+)"([^>]*)>([\s\S]*?)<\/\1>/g, (m, tag, a, key, b) => (key in s ? `<${tag}${a} data-i18n-html="${key}"${b}>${s[key]}</${tag}>` : m));
  out = out.replace(/<[a-z0-9]+\b[^>]*\sdata-i18n-ph="(\w+)"[^>]*>/g, (tag, key) => (key in s ? setAttr(tag, "placeholder", s[key]) : tag));
  out = out.replace(/<[a-z0-9]+\b[^>]*\sdata-i18n-aria="(\w+)"[^>]*>/g, (tag, key) => (key in s ? setAttr(setAttr(tag, "aria-label", s[key]), "title", s[key]) : tag));

  // The language menu shows this language: the button and the list agree.
  out = out.replace(/(<ul id="langlist"[\s\S]*?<\/ul>)/, (ul) =>
    ul.replace(/ aria-selected="true"/g, "").replace(new RegExp(`(<li [^>]*data-lang="${lang}"[^>]*)>`), '$1 aria-selected="true">'));
  const chosen = out.match(new RegExp(`<li [^>]*data-lang="${lang}"[^>]*>([\\s\\S]*?)</li>`));
  if (chosen) out = out.replace(/(<span class="langcur">)[\s\S]*?(<\/span><svg class="chev")/, (m, a, b) => a + chosen[1] + b);
  // The link-preview image is the one of this language.
  out = out.split("/og/mn.png").join(`/og/${lang}.png`);
  // The footer list of languages marks this one.
  out = out.replace(/(<nav aria-label="Language" id="langs">[\s\S]*?<\/nav>)/, (nav) =>
    nav.replace(/ aria-current="true"/g, "").replace(new RegExp(`(<a href="${PATHS[lang]}" hreflang="${lang}" lang="${lang}")`), '$1 aria-current="true"'));
  return out;
}

async function main() {
  const dist = fileURLToPath(new URL("../dist/", import.meta.url));
  const html = await readFile(dist + "index.html", "utf8");
  for (const lang of ALL_LANGS.filter((l) => l !== "mn")) {
    await mkdir(dist + lang, { recursive: true });
    await writeFile(`${dist}${lang}/index.html`, render(html, lang));
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await main();
