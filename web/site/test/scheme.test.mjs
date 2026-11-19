// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { convert_scheme, scheme_for_language, scheme_ids, scheme_is_draft } from "../src/wasm/buuz.js";
import { SCHEME_KEY, loadScheme, saveScheme, schemeAfterLang } from "../src/scheme.ts";
import { strings } from "../src/i18n.ts";
import { MORE } from "../src/i18n-more.ts";

await init({ module_or_path: await readFile(new URL("../src/wasm/buuz_bg.wasm", import.meta.url)) });

const ids = scheme_ids().split(",");

function fakeStore(initial = {}, blocked = false) {
  const data = new Map(Object.entries(initial));
  const guard = () => {
    if (blocked) throw new DOMException("blocked", "SecurityError");
  };
  return {
    data,
    getItem: (k) => (guard(), data.get(k) ?? null),
    setItem: (k, v) => (guard(), void data.set(k, String(v))),
    removeItem: (k) => (guard(), void data.delete(k)),
  };
}

test("the core lists the schemes the page offers", () => {
  assert.deepEqual(ids, ["ru", "ru-gost", "ru-volapuk", "uk", "uk-kmu", "be", "bg", "sr", "mk", "kk", "ky", "mn-mns"]);
  assert.equal(convert_scheme("ru", "Privet, mir!"), "Привет, мир!");
  assert.equal(convert_scheme("sr", "ljubav"), "љубав");
  assert.equal(scheme_for_language("ru"), "ru");
  assert.equal(scheme_for_language("en"), "");
  assert.equal(scheme_is_draft("ru"), false);
  assert.equal(scheme_is_draft("uk"), true);
});

test("index.html offers every scheme", async () => {
  const html = await readFile(new URL("../index.html", import.meta.url), "utf8");
  for (const id of ["mn", ...ids]) assert.match(html, new RegExp(`<option value="${id}"`), id);
});

test("the saved scheme is read back only when it is known", () => {
  const store = fakeStore();
  assert.equal(loadScheme(store, ids), null);
  saveScheme(store, "uk");
  assert.equal(store.data.get(SCHEME_KEY), "uk");
  assert.equal(loadScheme(store, ids), "uk");
  saveScheme(store, "mn");
  assert.equal(loadScheme(store, ids), "mn");
  assert.equal(loadScheme(fakeStore({ [SCHEME_KEY]: "klingon" }), ids), null);
});

test("a blocked store does not throw", () => {
  const store = fakeStore({}, true);
  assert.equal(loadScheme(store, ids), null);
  saveScheme(store, "ru");
  assert.equal(loadScheme(null, ids), null);
  saveScheme(null, "ru");
});

test("a page language with a scheme types it; leaving it drops back to Mongolian", () => {
  const f = scheme_for_language;
  assert.equal(schemeAfterLang("mn", "ru", "mn", f), "ru");
  assert.equal(schemeAfterLang("ru", "uk", "ru", f), "uk");
  assert.equal(schemeAfterLang("ru", "mn", "ru", f), "mn");
  assert.equal(schemeAfterLang("ru", "en", "ru", f), "mn");
  assert.equal(schemeAfterLang("uk", "en", "ru", f), "uk");
  assert.equal(schemeAfterLang("uk", "en", "uk", f), "mn");
  assert.equal(schemeAfterLang("mn", "en", "mn", f), "mn");
});

test("mn, en and ru have every string; the others only keys that exist", () => {
  const mn = Object.keys(strings.mn).sort();
  for (const lang of ["en", "ru"]) assert.deepEqual(Object.keys(strings[lang]).sort(), mn, lang);
  for (const lang of MORE) {
    for (const key of Object.keys(strings[lang])) assert.ok(key in strings.mn, `${lang}.${key}`);
  }
});

test("each more language has the text of the main page", () => {
  const needed = ["title", "description", "tag", "howToBody", "faqTitle", "faq0q", "faq0a", "faq1q", "faq1a", "faq2q", "faq2a", "faq3q", "faq3a", "faq4q", "faq4a", "faq5q", "faq5a", "faq6q", "faq6a", "footProduct", "feedbackTitle", "schemeLabel", "schemeHint"];
  for (const lang of MORE) {
    for (const key of needed) assert.ok(strings[lang][key], `${lang}.${key}`);
    assert.notEqual(strings[lang].title, strings.en.title, lang);
    assert.equal(scheme_for_language(lang), lang, `${lang} has its own scheme`);
  }
});
