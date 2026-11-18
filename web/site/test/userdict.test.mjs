// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { convert_with_config, complete_at, config_set_option, default_config_text, load_word_list } from "../src/wasm/buuz.js";
import { USERDICT_KEY, addUserWord, clearUserDict, entriesOf, importUserDict, loadUserDict, removeUserWord } from "../src/userdict.ts";
import { strings } from "../src/i18n.ts";

await init({ module_or_path: await readFile(new URL("../src/wasm/buuz_bg.wasm", import.meta.url)) });

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

test("an empty or blocked store gives an empty dictionary", () => {
  assert.equal(loadUserDict(fakeStore()), "");
  assert.equal(loadUserDict(fakeStore({}, true)), "");
  assert.equal(loadUserDict(null), "");
});

test("a saved dictionary loads, bad lines are dropped", () => {
  const store = fakeStore({ [USERDICT_KEY]: "add\tтэнгис\t1\nnonsense\nuse\tбайна\t5\n" });
  const text = loadUserDict(store);
  assert.deepEqual(entriesOf(text), [
    { kind: "add", word: "тэнгис", count: 1 },
    { kind: "use", word: "байна", count: 5 },
  ]);
});

test("add, list, remove and clear keep the store in step", () => {
  const store = fakeStore();
  let r = addUserWord(store, "", "Тэнгис");
  assert.ok(r.ok && r.saved);
  assert.equal(store.data.get(USERDICT_KEY), "add\tтэнгис\t1\n");
  assert.deepEqual(entriesOf(r.text), [{ kind: "add", word: "тэнгис", count: 1 }]);
  const bad = addUserWord(store, r.text, "tengis");
  assert.deepEqual(bad, { ok: false, error: { code: "ud_not_cyrillic", number: 0, detail: "" } });
  assert.equal(addUserWord(store, r.text, "  ").error.code, "ud_empty_word");
  const gone = removeUserWord(store, r.text, "тэнгис");
  assert.equal(gone.text, "");
  assert.equal(store.data.has(USERDICT_KEY), false);
  r = addUserWord(store, "", "бууз");
  const cleared = clearUserDict(store);
  assert.deepEqual(cleared, { text: "", saved: true });
  assert.equal(store.data.has(USERDICT_KEY), false);
});

test("blocked storage still changes the dictionary for this visit", () => {
  const store = fakeStore({}, true);
  const r = addUserWord(store, "", "бууз");
  assert.ok(r.ok);
  assert.equal(r.saved, false);
  assert.equal(entriesOf(r.text).length, 1);
});

test("import merges, keeps what is there and reports warnings", () => {
  const store = fakeStore();
  const have = "add\tтэнгис\t1\nuse\tбайна\t7\n";
  const r = importUserDict(store, have, "use\tбайна\t2\nadd\tбууз\t1\nbad\n");
  assert.ok(r.ok);
  assert.deepEqual(r.warnings.map((w) => [w.code, w.number]), [["ud_bad_line", 3]]);
  assert.deepEqual(entriesOf(r.text).map((e) => [e.kind, e.word, e.count]).sort(), [
    ["add", "бууз", 1],
    ["add", "тэнгис", 1],
    ["use", "байна", 7],
  ]);
  assert.equal(store.data.get(USERDICT_KEY), r.text);
  const big = importUserDict(store, have, "a".repeat(1024 * 1024 + 1));
  assert.equal(big.ok, false);
  assert.equal(big.error.code, "too_large");
});

test("the dictionary reaches autocorrect and completion", () => {
  const cfg = config_set_option(default_config_text(), "autocorrect", true);
  load_word_list("дэлгүүр\t100\n");
  const store = fakeStore();
  loadUserDict(store);
  assert.equal(convert_with_config("delgur ", cfg), "дэлгүүр ");
  const r = addUserWord(store, "", "дэлгүр");
  assert.equal(convert_with_config("delgur ", cfg), "дэлгүр ");
  assert.ok(r.ok);
  clearUserDict(store);
  assert.equal(convert_with_config("delgur ", cfg), "дэлгүүр ");
  addUserWord(store, "", "тэнгис");
  assert.equal(complete_at("tengi", "", cfg)?.word, "тэнгис");
  clearUserDict(store);
});

test("every dictionary message has a Mongolian and an English text", () => {
  const keys = ["bad_line", "unknown_kind", "bad_count", "empty_word", "bad_word", "word_too_long", "duplicate", "not_cyrillic"];
  for (const lang of ["mn", "en"]) {
    for (const k of keys) assert.ok(strings[lang][`cfg_ud_${k}`], `${lang} cfg_ud_${k}`);
    for (const k of ["udTitle", "udHint", "udEmpty", "udAdd", "udDelete", "udClear", "udClearConfirm", "udExport", "udImport", "udKindAdd", "udKindUse", "udKindReject"]) {
      assert.ok(strings[lang][k], `${lang} ${k}`);
    }
  }
});
