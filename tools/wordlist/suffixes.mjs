#!/usr/bin/env node
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors.
//
// Prints the case-suffix patterns mined from data/wordlist/mn.txt, for review
// by a native speaker (issue #52).
//
//   node tools/wordlist/suffixes.mjs data/wordlist/mn.txt > suffix-review.tsv
//
// A pair is a listed word that equals a listed stem (2 letters or more) plus a
// suffix variant. Pairs are grouped by family, stem class and gender. See
// tools/wordlist/README.md.

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export const SPDX = "# SPDX-License-Identifier: CC-BY-SA-4.0";
export const SOURCE =
  "# Case-suffix patterns mined from data/wordlist/mn.txt by tools/wordlist/suffixes.mjs. Not committed. For native-speaker review.";
export const COLUMNS = [
  "family",
  "stem_class",
  "gender",
  "variant",
  "pairs",
  "weight",
  "share",
  "examples",
];

// These family tables must stay the same as core/src/suffix_choice.rs.
export const FAMILIES = {
  genitive: ["гийн", "ийн", "ын", "ний", "ны", "н"],
  accusative: ["гийг", "ийг", "ыг", "г"],
  ablative: [
    "аас", "оос", "ээс", "өөс",
    "наас", "ноос", "нээс", "нөөс",
    "гаас", "гоос", "гээс", "гөөс",
  ],
  comitative: ["тай", "той", "тэй"],
};

const MIN_STEM = 2;
const VOWELS = new Set("аэиоуүөяеёюый");
const BACK = new Set("ауяюы");
const FRONT = new Set("эүе");
const PALATAL = new Set("жчшщ");

// Longest variant first inside each family.
const ORDERED = Object.entries(FAMILIES).map(([family, variants]) => [
  family,
  [...variants].sort((a, b) => b.length - a.length || (a < b ? -1 : 1)),
]);

/** Parse `word<TAB>count` lines into a Map. Lines that start with `#` are skipped. */
export function parseList(text) {
  const words = new Map();
  for (const line of text.split("\n")) {
    if (!line || line.startsWith("#")) continue;
    const [word, count] = line.split("\t");
    const n = Number.parseInt(count, 10);
    if (word && Number.isFinite(n)) words.set(word, (words.get(word) ?? 0) + n);
  }
  return words;
}

/**
 * Find the (stem, family, variant) pairs. Per family, one word gets at most
 * one parse: the longest variant whose stem is listed. Guard G2: a variant
 * that starts with г needs a stem that ends in a vowel letter.
 */
export function findPairs(words) {
  const pairs = [];
  for (const [word, count] of words) {
    for (const [family, variants] of ORDERED) {
      for (const variant of variants) {
        if (!word.endsWith(variant)) continue;
        const stem = word.slice(0, word.length - variant.length);
        if (stem.length < MIN_STEM || !words.has(stem)) continue;
        if (variant.startsWith("г") && !VOWELS.has(stem.at(-1))) continue;
        pairs.push({ word, stem, family, variant, count });
        break;
      }
    }
  }
  return pairs;
}

/** Class of the last letter of the stem. */
export function stemClass(stem) {
  const last = stem.at(-1);
  const prev = stem.at(-2) ?? "";
  if (last === "й") return "y";
  if (last === "ь") return "soft_sign";
  if (VOWELS.has(last)) {
    if (prev === last) return "long_vowel";
    if (VOWELS.has(prev)) return "diphthong";
    return "short_vowel";
  }
  if (last === "н") return "n";
  if (last === "г") return "g";
  if (PALATAL.has(last)) return "palatal";
  return "consonant";
}

/** Gender of the stem: its first deciding vowel. */
export function gender(stem) {
  for (const ch of stem) {
    if (ch === "о" || ch === "ё") return "o";
    if (ch === "ө") return "ö";
    if (BACK.has(ch)) return "back";
    if (FRONT.has(ch)) return "front";
  }
  return "neutral";
}

function cmp(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

/** Group pairs into sorted rows with pair count, weight, share and examples. */
export function group(pairs) {
  const groups = new Map();
  for (const p of pairs) {
    const sc = stemClass(p.stem);
    const g = gender(p.stem);
    const gkey = `${p.family}\t${sc}\t${g}`;
    let entry = groups.get(gkey);
    if (!entry) groups.set(gkey, (entry = { family: p.family, stemClass: sc, gender: g, variants: new Map(), total: 0 }));
    let v = entry.variants.get(p.variant);
    if (!v) entry.variants.set(p.variant, (v = { pairs: [], weight: 0 }));
    v.pairs.push(p);
    v.weight += p.count;
    entry.total += p.count;
  }
  const rows = [];
  for (const e of groups.values()) {
    for (const [variant, v] of e.variants) {
      const examples = [...v.pairs]
        .sort((a, b) => b.count - a.count || cmp(a.word, b.word))
        .slice(0, 3)
        .map((p) => `${p.stem}+${p.variant}`);
      rows.push({
        family: e.family,
        stemClass: e.stemClass,
        gender: e.gender,
        variant,
        pairs: v.pairs.length,
        weight: v.weight,
        share: e.total > 0 ? v.weight / e.total : 0,
        examples,
      });
    }
  }
  rows.sort(
    (a, b) =>
      cmp(a.family, b.family) ||
      cmp(a.stemClass, b.stemClass) ||
      cmp(a.gender, b.gender) ||
      b.weight - a.weight ||
      cmp(a.variant, b.variant),
  );
  return rows;
}

/** Render rows as TSV with a header. */
export function format(rows) {
  const lines = [SPDX, SOURCE, COLUMNS.join("\t")];
  for (const r of rows) {
    lines.push(
      [
        r.family,
        r.stemClass,
        r.gender,
        r.variant,
        r.pairs,
        r.weight,
        r.share.toFixed(2),
        r.examples.join(", "),
      ].join("\t"),
    );
  }
  return lines.join("\n") + "\n";
}

function main(argv) {
  const path = argv[2];
  if (!path) {
    process.stderr.write("usage: node tools/wordlist/suffixes.mjs data/wordlist/mn.txt > suffix-review.tsv\n");
    process.exit(2);
  }
  const rows = group(findPairs(parseList(readFileSync(path, "utf8"))));
  process.stdout.write(format(rows));
  process.stderr.write(`${rows.length} rows\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv);
}
