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

/** The languages of the typing picker, in its order, with their own names. */
export const TYPING_LANGS = [
  ["mn", "Монгол"],
  ["ru", "Русский"],
  ["uk", "Українська"],
  ["be", "Беларуская"],
  ["bg", "Български"],
  ["sr", "Српски"],
  ["mk", "Македонски"],
  ["kk", "Қазақша"],
  ["ky", "Кыргызча"],
] as const;

export type FormatKind = "chat" | "official" | "lookalike";

/** What a scheme id means in the picker: its language and its kind of format. */
export function formatOf(id: string): { lang: string; kind: FormatKind; name: string | null } {
  const lang = id.split("-")[0];
  const official: Record<string, string> = { "mn-mns": "MNS 5217", "ru-gost": "ГОСТ 7.79", "uk-kmu": "КМУ 2010" };
  if (official[id]) return { lang, kind: "official", name: official[id] };
  if (id === "ru-volapuk") return { lang, kind: "lookalike", name: null };
  return { lang, kind: "chat", name: id === "mn" ? "Buuz" : null };
}

/** The scheme ids of one language: the chat format first. */
export function formatsOf(lang: string, ids: readonly string[]): string[] {
  return ["mn", ...ids].filter((id) => id.split("-")[0] === lang);
}

/** A few multi-letter keys for the line under the output, in table order. */
export function sampleKeys(rows: Row[], max = 6): { latin: string; out: string }[] {
  const out: { latin: string; out: string }[] = [];
  for (const r of rows) {
    // A letter with a one-key way (х: h) says less than one without (ж: zh).
    if (r.keys.some((k) => k.latin.length === 1)) continue;
    const ok = r.keys.filter((k) => /^[a-z]+$/.test(k.latin) && (k.when === "always" || k.when === "smart"));
    const k = ok.find((k) => !k.latin.startsWith("j")) ?? ok[0];
    if (k && out.length < max) out.push({ latin: k.latin, out: r.out });
  }
  return out;
}
