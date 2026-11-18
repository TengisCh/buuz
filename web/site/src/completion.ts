// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// Tab completion for the Latin box (docs/typing-rules.md section 12). The
// page has no key-by-key composer, so it asks the core (through wasm) about
// the word before the caret. The word list is a separate static asset, loaded
// once from this site's own origin, and only when completion is on. If the
// load fails, completion stays off and nothing is shown. No DOM here, so the
// node tests run it.

import { complete_at, load_word_list, word_list_loaded } from "./wasm/buuz.js";

/** The part of `fetch` this module uses. */
export type Fetcher = (url: string, init: { credentials: "omit" }) => Promise<{ ok: boolean; text(): Promise<string> }>;

let loading: Promise<boolean> | null = null;
let failed = false;

/** Forget a failed load, so the next `ensureWordList` tries again (for tests). */
export function resetWordListState(): void {
  loading = null;
  failed = false;
}

/** True when `url` is on the same origin as `base`. */
export function sameOrigin(url: string, base: string): boolean {
  try {
    return new URL(url, base).origin === new URL(base).origin;
  } catch {
    return false;
  }
}

/**
 * Fetch and parse the word list once. Gives true when a list is loaded.
 * Fails silently (gives false) on a network error, a bad answer, a bad list
 * or a URL from another origin. A failure stays for the rest of the visit:
 * the page does not try again on every key.
 */
export function ensureWordList(url: string, base: string, fetcher: Fetcher): Promise<boolean> {
  if (word_list_loaded()) return Promise.resolve(true);
  if (failed) return Promise.resolve(false);
  if (loading) return loading;
  if (!sameOrigin(url, base)) return Promise.resolve(false);
  loading = (async () => {
    try {
      const res = await fetcher(url, { credentials: "omit" });
      if (!res.ok) throw new Error("bad answer");
      load_word_list(await res.text());
      return true;
    } catch {
      failed = true;
      return false;
    } finally {
      loading = null;
    }
  })();
  return loading;
}

/** A completion for the word before the caret. */
export interface Completion {
  /** The ghost text: the Cyrillic rest of the word. */
  rest: string;
  /** The whole completed word in Cyrillic. */
  word: string;
  /** How many UTF-16 units of Latin before the caret the word replaces. */
  replaces: number;
}

/**
 * The completion for the word that ends at `caret`, or null. The core gives
 * none when an input character follows the caret (the caret is inside a
 * word).
 */
export function completionAt(value: string, caret: number, config: string): Completion | null {
  // A 64-unit window keeps the call cheap on long text. If it splits a
  // surrogate pair, the lone half is not an input character, so it is harmless.
  const before = value.slice(Math.max(0, caret - 64), caret);
  const after = value.slice(caret, caret + 2);
  const c = complete_at(before, after, config);
  if (!c) return null;
  const out = { rest: c.rest, word: c.word, replaces: c.replaces };
  c.free();
  return out;
}

/** The text and caret after Tab accepts `c` at `caret`. */
export function accept(value: string, caret: number, c: Completion): { value: string; caret: number } {
  const start = caret - c.replaces;
  return { value: value.slice(0, start) + c.word + value.slice(caret), caret: start + c.word.length };
}
