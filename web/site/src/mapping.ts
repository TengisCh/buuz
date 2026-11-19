// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// The key table people see under "How to type" and in the Rules tab. The core
// gives one row per Latin key (`key_table` in web/wasm). This module groups
// the keys by the Cyrillic they give, so each letter is one short row:
// ө  q ö o' 'o. No DOM here, so it runs in the node tests.

export type When = "always" | "after_vowel" | "word_start" | "harmony" | "smart";

export interface KeyOf {
  latin: string;
  when: When;
}

export interface Row {
  out: string;
  keys: KeyOf[];
}

/** Short keys first, plain ASCII before ö and ü, a leading apostrophe last. */
function keyOrder(a: string, b: string): number {
  const rank = (s: string) => [s.length, /^['"]/.test(s) ? 1 : 0, /^[\x20-\x7e]*$/.test(s) ? 0 : 1];
  const [x, y] = [rank(a), rank(b)];
  for (let i = 0; i < x.length; i++) if (x[i] !== y[i]) return x[i] - y[i];
  return a < b ? -1 : a > b ? 1 : 0;
}

/**
 * Group the core's `latin<TAB>cyrillic<TAB>when` lines by the Cyrillic.
 * Rows follow the alphabet of `language`. Capital outputs ('' gives Ь) are
 * left out: they are the same key typed twice, which the rules page explains.
 */
export function groupKeys(table: string, language: string): Row[] {
  const byOut = new Map<string, KeyOf[]>();
  for (const line of table.split("\n")) {
    const [latin, out, when] = line.split("\t");
    if (!latin || !out || out !== out.toLowerCase()) continue;
    const keys = byOut.get(out) ?? [];
    if (!keys.some((k) => k.latin === latin)) keys.push({ latin, when: when as When });
    byOut.set(out, keys);
  }
  const collator = new Intl.Collator(language);
  return [...byOut]
    .map(([out, keys]) => ({ out, keys: keys.sort((a, b) => keyOrder(a.latin, b.latin)) }))
    .sort((a, b) => collator.compare(a.out, b.out));
}
