// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { ORIGIN, PATHS, faqJson, hreflangLinks, render } from "../scripts/prerender.mjs";
import { langOfPath, strings } from "../src/i18n.ts";

const html = await readFile(new URL("../index.html", import.meta.url), "utf8");

for (const lang of ["en", "ru"]) {
  test(`the ${lang} page is complete without any script`, () => {
    const page = render(html, lang);
    assert.match(page, new RegExp(`<html lang="${lang}"`));
    assert.ok(page.includes(`<title>${strings[lang].title}</title>`));
    assert.ok(page.includes(`<link rel="canonical" href="${ORIGIN}${PATHS[lang]}" />`));
    assert.ok(page.includes(`<meta property="og:url" content="${ORIGIN}${PATHS[lang]}" />`));
    assert.ok(page.includes(strings[lang].description));
    // Every text key of the page is written in this language.
    for (const m of page.matchAll(/data-i18n="(\w+)"/g)) {
      assert.ok(page.includes(strings[lang][m[1]].replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")), m[1]);
    }
    for (const m of page.matchAll(/data-i18n-html="(\w+)"/g)) assert.ok(page.includes(strings[lang][m[1]]), m[1]);
    // No Mongolian page text is left in the title, tag line or FAQ.
    assert.ok(!page.includes(strings.mn.tag));
    assert.ok(!page.includes(strings.mn.faq0q));
  });
}

test("every page links to every language, and to itself", () => {
  const links = hreflangLinks();
  for (const l of ["mn", "en", "ru"]) assert.ok(links.includes(`hreflang="${l}" href="${ORIGIN}${PATHS[l]}"`));
  assert.ok(links.includes(`hreflang="x-default" href="${ORIGIN}/"`));
  for (const page of [html, render(html, "en"), render(html, "ru")]) {
    for (const l of ["mn", "en", "ru", "x-default"]) assert.ok(page.includes(`hreflang="${l}"`), l);
  }
});

test("the FAQ data of each language is valid and in that language", () => {
  for (const lang of ["mn", "en", "ru"]) {
    const data = JSON.parse(faqJson(lang));
    assert.equal(data.inLanguage, lang);
    assert.equal(data.mainEntity.length, 7);
    assert.equal(data.mainEntity[0].name, strings[lang].faq0q);
    assert.ok(!data.mainEntity[2].acceptedAnswer.text.includes("<kbd>"));
  }
  assert.ok(render(html, "ru").includes(`"inLanguage": "ru"`));
});

test("the language button of each page names the next language", () => {
  assert.match(render(html, "en"), /id="lang"[^>]*>РУ</);
  assert.match(render(html, "ru"), /id="lang"[^>]*>МН</);
});

test("a path belongs to one language", () => {
  assert.equal(langOfPath("/"), "mn");
  assert.equal(langOfPath("/en/"), "en");
  assert.equal(langOfPath("/en"), "en");
  assert.equal(langOfPath("/ru/"), "ru");
  assert.equal(langOfPath("/privacy"), null);
});

test("the sitemap lists every language page", async () => {
  const sitemap = await readFile(new URL("../public/sitemap.xml", import.meta.url), "utf8");
  for (const p of Object.values(PATHS)) assert.ok(sitemap.includes(`<loc>${ORIGIN}${p}</loc>`), p);
  assert.ok(!sitemap.includes("buuz.thetengis.com/?lang=en"), "no query-string alternate for the converter");
});
