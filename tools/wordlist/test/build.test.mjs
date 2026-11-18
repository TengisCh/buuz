// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors.

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import {
  countWords,
  cleanText,
  rank,
  render,
  filterRussian,
  HEADER,
  SPDX,
} from "../build.mjs";

const fixture = fileURLToPath(new URL("./fixture.xml", import.meta.url));
const script = fileURLToPath(new URL("../build.mjs", import.meta.url));
const xml = readFileSync(fixture, "utf8");

async function counts() {
  return countWords(xml.split("\n"));
}

test("counts words from namespace 0 pages only", async () => {
  const c = await counts();
  assert.equal(c.get("хэрэглэгчийнтуршилт"), undefined);
});

test("skips redirect pages", async () => {
  const c = await counts();
  assert.equal(c.get("редирект"), undefined);
});

test("strips templates (nested), refs, comments, files, categories, tables, urls", async () => {
  const c = await counts();
  for (const gone of [
    "зохиомол",
    "name",
    "аа",
    "эшлэлтэнэмэгдэл",
    "коммент",
    "ноорог",
    "тайлбар",
    "холбоос",
    "зургийн",
    "ангиллын",
    "нэр",
    "гарчигхэсэг",
    "хүснэгтэнтэг",
    "абв",
    "nbsp",
  ]) {
    assert.equal(c.get(gone), undefined, gone);
  }
});

test("keeps link text, splits hyphens, lower-cases", async () => {
  const c = await counts();
  assert.equal(c.get("хотууд"), 1);
  assert.equal(c.get("хот"), undefined, "link target is not shown text");
  assert.equal(c.get("хоёр"), 1);
  assert.equal(c.get("гурван"), 1);
  assert.equal(c.get("өө"), 2, "ӨӨ lower-cased plus өө");
  assert.equal(c.get("монгол"), 3);
});

test("letter filter: Latin, digits, mixed, 1 letter and over-long tokens are dropped", async () => {
  const c = await counts();
  assert.equal(c.get("abc"), undefined);
  assert.equal(c.get("123"), undefined);
  assert.equal(c.get("mixedлатин"), undefined);
  assert.equal(c.get("ү"), undefined);
  assert.equal(c.get("а"), undefined);
  for (const w of c.keys()) assert.ok(w.length <= 30, w);
});

test("minimum count and ranking with deterministic ties", async () => {
  const c = await counts();
  const ranked = rank(c, { minCount: 3, limit: 50000 });
  assert.deepEqual(ranked, [
    ["байна", 5],
    ["болон", 3],
    ["монгол", 3],
    ["ётой", 3],
  ]);
  // The cut falls on a count boundary: three words tie at 3, so none of them
  // stays when fewer than three places are left.
  assert.deepEqual(rank(c, { minCount: 3, limit: 2 }), [["байна", 5]]);
  assert.deepEqual(rank(c, { minCount: 3, limit: 3 }), [["байна", 5]]);
  assert.equal(rank(c, { minCount: 3, limit: 4 }).length, 4);
});

test("cut keeps every word at the boundary count", () => {
  const c = new Map([
    ["аа", 9],
    ["бб", 5],
    ["вв", 5],
    ["гг", 4],
  ]);
  assert.equal(rank(c, { minCount: 3, limit: 3 }).length, 3);
  assert.deepEqual(rank(c, { minCount: 3, limit: 2 }), [["аа", 9]]);
  assert.deepEqual(rank(c, { minCount: 3, limit: 1 }), [["аа", 9]]);
  assert.equal(rank(c, { minCount: 3, limit: 4 }).length, 4);
  assert.deepEqual(rank(c, { minCount: 5, limit: 100 }), [
    ["аа", 9],
    ["бб", 5],
    ["вв", 5],
  ]);
});

test("output has the header and word<TAB>count lines", async () => {
  const c = await counts();
  const out = render(rank(c, { minCount: 3, limit: 50000 }));
  const lines = out.trimEnd().split("\n");
  assert.equal(lines[0], SPDX);
  assert.equal(lines[0], "# SPDX-License-Identifier: CC-BY-SA-4.0");
  assert.equal(lines[1], HEADER);
  assert.equal(lines[2], "байна\t5");
  assert.equal(out.endsWith("\n"), true);
});

test("command line reads stdin and is deterministic", () => {
  const run = () => execFileSync("node", [script], { input: xml, encoding: "utf8" });
  const a = run();
  assert.equal(a, run());
  assert.match(a, /^# SPDX-License-Identifier: CC-BY-SA-4.0\n# Mongolian word frequencies from Mongolian Wikipedia \(mnwiki-20261101\)/);
  assert.match(a, /\nболон\t3\n/);
});

const words = (text) => countWords(["<page>", "<ns>0</ns>", `<text>${text}</text>`, "</page>"]);

test("fragments: a part that is not Cyrillic drops that part and all later parts", async () => {
  const c = await words("2010-нд Obama-гийн хоёр-гурван o'Нил-ын");
  assert.equal(c.get("нд"), undefined);
  assert.equal(c.get("гийн"), undefined);
  assert.equal(c.get("ын"), undefined);
  assert.equal(c.get("хоёр"), 1);
  assert.equal(c.get("гурван"), 1);
});

test("fragments: Cyrillic compounds with a hyphen or apostrophe stay split", async () => {
  const c = await words("хар-цагаан дэлхийн’хүн");
  assert.equal(c.get("хар"), 1);
  assert.equal(c.get("цагаан"), 1);
  assert.equal(c.get("дэлхийн"), 1);
  assert.equal(c.get("хүн"), 1);
});

test("bold and italic quotes do not hide or glue words", async () => {
  const c = await words("'''монгол''' ''улс''");
  assert.equal(c.get("монгол"), 1);
  assert.equal(c.get("улс"), 1);
});

test("headings are dropped", async () => {
  const c = await countWords([
    "<page>",
    "<ns>0</ns>",
    "<text>== Гарчигтүүхэн ==",
    "биеийнтекст",
    "=== Дэд гарчиг ===</text>",
    "</page>",
  ]);
  assert.equal(c.get("гарчигтүүхэн"), undefined);
  assert.equal(c.get("дэд"), undefined);
  assert.equal(c.get("биеийнтекст"), 1);
});

test("unclosed spans end at the next blank line", () => {
  for (const open of ["{{", "[[", "<!--", "<ref>", "{|"]) {
    const out = cleanText(`before ${open} скрытое слово\n\nafter`);
    assert.ok(!out.includes("скрытое"), open);
    assert.ok(out.includes("before") && out.includes("after"), open);
  }
});

test("unclosed spans stop after 64 KB without a blank line", () => {
  const out = cleanText("{{ " + "a ".repeat(40000) + " tail");
  assert.ok(out.includes("tail"));
});

test("many unclosed openers stay fast", () => {
  const t0 = Date.now();
  cleanText("<ref>x ".repeat(20000));
  cleanText("{{x ".repeat(20000));
  cleanText("<!--x ".repeat(20000));
  assert.ok(Date.now() - t0 < 5000);
});

test("closed spans keep working with nested and multi-line content", () => {
  const out = cleanText("a {{x\n\n{{y}}\nz}} b <ref>one\n\ntwo</ref> c");
  assert.ok(out.includes("a") && out.includes("b") && out.includes("c"));
  assert.ok(!out.includes("one") && !out.includes("two") && !out.includes("z"));
});

test("deleted text elements are skipped and do not swallow the next page", async () => {
  const c = await countWords([
    "<page>",
    "<ns>0</ns>",
    '<text bytes="0" deleted="deleted" />',
    "</page>",
    "<page>",
    "<ns>0</ns>",
    "<text>сүүлийнхэн</text>",
    "</page>",
  ]);
  assert.equal(c.get("сүүлийнхэн"), 1);
  assert.equal(c.size, 1);
});

test("double-escaped entities are removed, not turned into tags", async () => {
  const c = await words("эхлэл &amp;lt;ref&amp;gt;эшлэлийнтекст &amp;nbsp; төгсгөл");
  assert.equal(c.get("эшлэлийнтекст"), 1);
  assert.equal(c.get("lt"), undefined);
  assert.equal(c.get("nbsp"), undefined);
  assert.equal(c.get("төгсгөл"), 1);
});

test("length limit: 30 letters are kept, 31 are not", async () => {
  const c = await words(`${"а".repeat(30)} ${"а".repeat(31)} ${"б".repeat(2)} б`);
  assert.equal(c.get("а".repeat(30)), 1);
  assert.equal(c.get("а".repeat(31)), undefined);
  assert.equal(c.get("бб"), 1);
  assert.equal(c.get("б"), undefined);
});

test("Russian filter: stoplist and suffixes, but not Mongolian look-alikes", () => {
  const c = new Map(
    [
      "на", "при", "для", "что", "российского", "министерство", "русский", "русская",
      "за", "нь", "дэлхий", "харъяат", "гавъяат", "байна", "болон", "хий", "ний", "их",
    ].map((w) => [w, 9]),
  );
  const { counts, removed } = filterRussian(c, 3);
  for (const gone of ["на", "при", "для", "что", "российского", "министерство", "русский", "русская"]) {
    assert.equal(counts.has(gone), false, gone);
  }
  for (const kept of ["за", "нь", "дэлхий", "харъяат", "гавъяат", "байна", "болон", "хий", "ний", "их"]) {
    assert.equal(counts.has(kept), true, kept);
  }
  assert.equal(removed.stoplist, 4);
  assert.equal(removed.suffix, 4);
});

test("Russian suffix rule keeps Mongolian words that end in ого or ных", () => {
  const c = new Map(
    ["бодлого", "тооллого", "ажиллагааных", "лого", "монгольского", "российский"].map((w) => [w, 9]),
  );
  const { counts, removed } = filterRussian(c, 3);
  for (const kept of ["бодлого", "тооллого", "ажиллагааных", "лого"]) {
    assert.equal(counts.has(kept), true, kept);
  }
  assert.equal(counts.has("монгольского"), false);
  assert.equal(counts.has("российский"), false);
  assert.equal(removed.suffix, 2);
});
