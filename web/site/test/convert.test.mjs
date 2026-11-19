// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { convert, convert_classic, convert_with } from "../src/wasm/buuz.js";

await init({ module_or_path: await readFile(new URL("../src/wasm/buuz_bg.wasm", import.meta.url)) });

const cases = [
  ["sain baina uu", "сайн байна уу"],
  ["buuz id'ye", "бууз идье"],
  ["geree", "гэрээ"],
  ["gar", "гар"],
  ["mongol", "монгол"],
  ["wnen", "үнэн"],
  ["qvqr", "өвөр"],
  ["hün", "хүн"],
  ["öndör", "өндөр"],
  ["shine", "шинэ"],
  ["chi", "чи"],
  ["yo", "ё"],
  ["unee", "үнээ"],
  ["irsen uu", "ирсэн үү"],
  ["aaviin", "аавын"],
  ["ajlahgui", "ажлахгүй"],
  ["", ""],
  ["hello, 123!\nsain", "хэллө, 123!\nсайн"],
  ["github.com", "github.com"],
  ["bat@gmail.com", "bat@gmail.com"],
];
for (const [input, expected] of cases) {
  test(JSON.stringify(input), () => assert.equal(convert(input), expected));
}

test("the Classic switch turns every smart feature off", () => {
  assert.equal(convert_classic("unee"), "унээ");
  assert.equal(convert_classic("aaviin"), "аавийн");
  assert.equal(convert_classic("bn uu "), "бн уу ");
  assert.equal(convert_classic("sain baina uu"), "сайн байна уу");
  assert.equal(convert_classic("ajlahgui"), "ажлахгуй");
});

test("classic mode keeps the original left-to-right rules", () => {
  assert.equal(convert_with("unee", false, false), "унээ");
  assert.equal(convert_with("aaviin", false, false), "аавийн");
  assert.equal(convert_with("irsen uu", false, false), "ирсэн уу");
  assert.equal(convert_with("unee", true, false), "үнээ");
});

test("abbreviations expand only when switched on", () => {
  assert.equal(convert_with("bn uu", true, true), "байна уу");
  assert.equal(convert_with("bn ", true, false), "бн ");
});

test("an abbreviation expands only after a typed space or punctuation", () => {
  assert.equal(convert_with("bn", true, true), "бн");
  assert.equal(convert_with("bn ", true, true), "байна ");
  assert.equal(convert_with("bn.", true, true), "байна.");
});

test("long paste stays fast", () => {
  const text = "sain baina uu ".repeat(20000);
  // The best of 5 runs: one slow run on a shared CI machine is not a slow converter.
  let best = Infinity;
  for (let i = 0; i < 5; i++) {
    const t = performance.now();
    convert(text);
    best = Math.min(best, performance.now() - t);
  }
  assert.ok(best < 200, `280k chars should convert in well under 200 ms (best of 5: ${best.toFixed(0)} ms)`);
});

// Every typewriter phrase on the page must convert exactly as written.
const phrases = JSON.parse(await readFile(new URL("../src/phrases.json", import.meta.url), "utf8"));
for (const [latin, cyrillic] of phrases) {
  test(`phrase ${JSON.stringify(latin)}`, () => assert.equal(convert(latin), cyrillic));
}
