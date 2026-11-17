// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { convert } from "../src/wasm/buuz.js";

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
  ["", ""],
  ["hello, 123!\nsain", "хэллө, 123!\nсайн"],
];
for (const [input, expected] of cases) {
  test(JSON.stringify(input), () => assert.equal(convert(input), expected));
}

test("long paste stays fast", () => {
  const text = "sain baina uu ".repeat(20000);
  const t = performance.now();
  convert(text);
  assert.ok(performance.now() - t < 200, "280k chars should convert in well under 200 ms");
});

// Every typewriter phrase on the page must convert exactly as written.
const phrases = JSON.parse(await readFile(new URL("../src/phrases.json", import.meta.url), "utf8"));
for (const [latin, cyrillic] of phrases) {
  test(`phrase ${JSON.stringify(latin)}`, () => assert.equal(convert(latin), cyrillic));
}
