// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors.

import { test } from "node:test";
import assert from "node:assert/strict";
import {
  parseList,
  findPairs,
  stemClass,
  gender,
  group,
  format,
  SPDX,
} from "../suffixes.mjs";

const LIST = `# SPDX-License-Identifier: CC-BY-SA-4.0
# header
гэр\t100
гэрийн\t30
гэрт\t5
улс\t80
улсын\t60
монгол\t50
монголын\t20
ном\t40
номын\t10
номтой\t4
хот\t30
хотод\t3
хотын\t12
ази\t20
азийн\t3
азигийг\t2
газар\t10
гараас\t9
гар\t7
гарыг\t7
`;

const words = parseList(LIST);

test("parseList skips header lines and keeps counts", () => {
  assert.equal(words.get("гэр"), 100);
  assert.equal(words.has("# header"), false);
  assert.equal(words.size, 20);
});

test("findPairs picks the longest listed variant, one per family", () => {
  const pairs = findPairs(words);
  const gen = pairs.filter((p) => p.family === "genitive" && p.word === "гэрийн");
  assert.equal(gen.length, 1);
  assert.equal(gen[0].stem, "гэр");
  assert.equal(gen[0].variant, "ийн");
  assert.equal(gen[0].count, 30);
});

test("findPairs needs a listed stem of two letters or more", () => {
  const pairs = findPairs(words);
  assert.equal(pairs.some((p) => p.word === "хотод"), false);
  assert.equal(pairs.some((p) => p.word === "гэрт"), false);
  assert.equal(findPairs(parseList("а\t5\nан\t5\n")).length, 0);
});

test("guard G2: a г form follows a vowel letter only", () => {
  const pairs = findPairs(words);
  assert.ok(pairs.some((p) => p.word === "азигийг" && p.variant === "гийг"));
  assert.ok(pairs.some((p) => p.word === "гараас"));
  assert.equal(
    findPairs(parseList("гэр\t5\nгэргийг\t5\nхот\t5\nхотгийн\t5\n")).length,
    0,
  );
});

test("stemClass", () => {
  assert.equal(stemClass("ази"), "short_vowel");
  assert.equal(stemClass("гуураа"), "long_vowel");
  assert.equal(stemClass("хуаи"), "diphthong");
  assert.equal(stemClass("улаан"), "n");
  assert.equal(stemClass("монг"), "g");
  assert.equal(stemClass("дель"), "soft_sign");
  assert.equal(stemClass("малай"), "y");
  assert.equal(stemClass("хуаж"), "palatal");
  assert.equal(stemClass("гэр"), "consonant");
});

test("gender uses the first deciding vowel", () => {
  assert.equal(gender("номт"), "masculine_o");
  assert.equal(gender("өвөл"), "feminine_ö");
  assert.equal(gender("улс"), "masculine");
  assert.equal(gender("гэр"), "feminine");
  assert.equal(gender("хүн"), "feminine");
  assert.equal(gender("хил"), "neutral");
});

test("group sums pairs and weight and computes shares", () => {
  const rows = group(findPairs(words));
  const back = rows.filter(
    (x) => x.family === "genitive" && x.stemClass === "consonant" && x.gender === "masculine",
  );
  assert.deepEqual(back.map((x) => x.variant), ["ын"]);
  assert.equal(back[0].pairs, 1);
  assert.equal(back[0].weight, 60);
  assert.equal(back[0].share, 1);
  const front = rows.filter(
    (x) => x.family === "genitive" && x.stemClass === "consonant" && x.gender === "feminine",
  );
  assert.equal(front[0].variant, "ийн");
});

test("shares add up within a group and examples are by count", () => {
  const rows = group(
    findPairs(parseList("улс\t9\nулсын\t6\nулсны\t2\nулсаас\t1\nбар\t1\nбарын\t1\n")),
  );
  const gen = rows.filter((x) => x.family === "genitive");
  assert.deepEqual(gen.map((x) => x.variant), ["ын", "ны"]);
  assert.equal(gen[0].weight, 7);
  assert.equal(gen[0].share, 7 / 9);
  assert.equal(gen[1].share, 2 / 9);
  assert.equal(gen[0].examples[0], "улс+ын");
});

test("order is deterministic", () => {
  const reversed = new Map([...words].reverse());
  assert.equal(format(group(findPairs(words))), format(group(findPairs(reversed))));
});

test("format writes the header, the column names and 2-decimal shares", () => {
  const lines = format(group(findPairs(words))).split("\n");
  assert.equal(lines[0], SPDX);
  assert.equal(SPDX, "# SPDX-License-Identifier: CC-BY-SA-4.0");
  assert.ok(lines[1].startsWith("# "));
  assert.equal(
    lines[2],
    "family\tstem_class\tgender\tvariant\tpairs\tweight\tshare\texamples",
  );
  const cols = lines[3].split("\t");
  assert.equal(cols.length, 8);
  assert.match(cols[6], /^\d\.\d\d$/);
});

test("guard G1: the one-letter variants н and г are never parsed", () => {
  assert.equal(findPairs(parseList("гэр\t5\nгэрн\t5\nасуу\t5\nасууг\t5\n")).length, 0);
});

test("guard G3: a г form is ambiguous when the stem plus г is listed", () => {
  const pairs = findPairs(parseList("ба\t5\nбаг\t8\nбагийн\t20\n"));
  assert.equal(pairs.length, 1);
  assert.equal(pairs[0].stem, "ба");
  assert.equal(pairs[0].variant, "гийн");
  assert.equal(pairs[0].ambiguous, true);
  const rows = group(pairs);
  assert.equal(rows.length, 1);
  assert.equal(rows[0].stemClass, "ambiguous");
  assert.equal(rows[0].family, "genitive");
  assert.equal(rows[0].variant, "гийн");
});

test("guard G3: an н form is ambiguous when the stem plus н is listed", () => {
  const pairs = findPairs(parseList("ус\t5\nусн\t4\nуснаас\t9\n"));
  assert.equal(pairs.length, 1);
  assert.equal(pairs[0].stem, "ус");
  assert.equal(pairs[0].variant, "наас");
  assert.equal(pairs[0].ambiguous, true);
});

test("an unambiguous longest variant is a normal pair", () => {
  const pairs = findPairs(parseList("ус\t5\nуснаас\t9\n"));
  assert.equal(pairs.length, 1);
  assert.equal(pairs[0].variant, "наас");
  assert.equal(pairs[0].ambiguous, false);
});

test("ambiguous words stay out of the shares of the real groups", () => {
  const rows = group(findPairs(parseList("ба\t5\nбаг\t8\nбагийн\t20\nгэр\t9\nгэрийн\t3\n")));
  const real = rows.filter((r) => r.stemClass !== "ambiguous");
  assert.deepEqual(real.map((r) => r.variant), ["ийн"]);
  assert.equal(real[0].share, 1);
});
