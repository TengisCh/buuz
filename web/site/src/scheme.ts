// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// The language the converter types. "mn" (Mongolian, the default) uses the
// Buuz rules and the portable config. Any other id is a core scheme
// (docs/research/cyrillic-schemes.md) and does not use the config. The
// choice is kept by itself in localStorage; it is not in the config file.

import type { Store } from "./config.ts";

export const SCHEME_KEY = "buuz.scheme";
export const MONGOLIAN = "mn";

/** The saved scheme id if it is Mongolian or one of `known`, else null. */
export function loadScheme(store: Store | null, known: readonly string[]): string | null {
  try {
    const id = store?.getItem(SCHEME_KEY) ?? null;
    return id === MONGOLIAN || (id !== null && known.includes(id)) ? id : null;
  } catch {
    return null;
  }
}

/** Save the id. A blocked store is fine: the page keeps it for this visit. */
export function saveScheme(store: Store | null, id: string): void {
  try {
    store?.setItem(SCHEME_KEY, id);
  } catch {
    /* private mode etc. */
  }
}

/**
 * The scheme after the page language changes. A page in a language with its
 * own scheme (Russian, Ukrainian, ...) types that language. Leaving such a page
 * drops back to Mongolian only if the scheme is still that page's own scheme;
 * any other choice is kept.
 *
 * `defaultFor(page)` is the scheme id of a page language, or "" (Mongolian and
 * English have none).
 */
export function schemeAfterLang(current: string, page: string, previous: string, defaultFor: (lang: string) => string): string {
  const own = defaultFor(page);
  if (own) return own;
  const before = defaultFor(previous);
  return before && current === before ? MONGOLIAN : current;
}
