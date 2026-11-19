// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { key_table, scheme_ids } from "../src/wasm/buuz.js";
import { groupKeys } from "../src/mapping.ts";

await init({ module_or_path: await readFile(new URL("../src/wasm/buuz_bg.wasm", import.meta.url)) });

const find = (rows, out) => rows.find((r) => r.out === out);

test("Buuz rows group every key of one letter", () => {
  const rows = groupKeys(key_table("mn"), "mn");
  assert.deepEqual(find(rows, "ө").keys.map((k) => k.latin), ["q", "ö", "o'", "'o"]);
  assert.deepEqual(find(rows, "ц").keys, [{ latin: "c", when: "always" }, { latin: "ts", when: "smart" }]);
  assert.deepEqual(find(rows, "о/ө").keys, [{ latin: "o", when: "harmony" }]);
});

test("rows follow the alphabet of the language", () => {
  const outs = groupKeys(key_table("ru"), "ru").map((r) => r.out);
  assert.ok(outs.indexOf("а") < outs.indexOf("б"));
  assert.ok(outs.indexOf("е") < outs.indexOf("ж"));
  assert.ok(outs.indexOf("ж") < outs.indexOf("я"));
});

test("a key with a condition keeps it", () => {
  const rows = groupKeys(key_table("ru"), "ru");
  assert.ok(find(rows, "й").keys.some((k) => k.latin === "y" && k.when === "after_vowel"));
});

test("every scheme has rows, and capitals are left out", () => {
  for (const id of ["mn", ...scheme_ids().split(",")]) {
    const rows = groupKeys(key_table(id), id.split("-")[0]);
    assert.ok(rows.length > 15, id);
    assert.ok(rows.every((r) => r.out === r.out.toLowerCase()), id);
  }
});

test("an empty table gives no rows", () => {
  assert.deepEqual(groupKeys("", "mn"), []);
});

import { formatOf, formatsOf, sampleKeys } from "../src/mapping.ts";

test("each scheme has a language and a kind of format", () => {
  assert.deepEqual(formatOf("mn"), { lang: "mn", kind: "chat", name: "Buuz" });
  assert.deepEqual(formatOf("mn-mns"), { lang: "mn", kind: "official", name: "MNS 5217" });
  assert.deepEqual(formatOf("ru-gost"), { lang: "ru", kind: "official", name: "ГОСТ 7.79" });
  assert.equal(formatOf("ru-volapuk").kind, "lookalike");
  assert.deepEqual(formatOf("kk"), { lang: "kk", kind: "chat", name: null });
});

test("formats of a language put the chat format first", () => {
  const ids = scheme_ids().split(",");
  assert.deepEqual(formatsOf("ru", ids), ["ru", "ru-gost", "ru-volapuk"]);
  assert.deepEqual(formatsOf("mn", ids), ["mn", "mn-mns"]);
  assert.deepEqual(formatsOf("kk", ids), ["kk"]);
});

test("sample keys are short multi-letter keys", () => {
  const s = sampleKeys(groupKeys(key_table("ru"), "ru"));
  assert.equal(s.length, 6);
  assert.ok(s.some((k) => k.latin === "zh" && k.out === "ж"));
});
