// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// The page keeps its settings as one portable config text (see
// docs/decisions/0005-portable-config-format.md) in localStorage under
// "buuz.config". This module holds the storage, migration and import logic,
// with no DOM, so it runs in the node tests. Every storage access is wrapped
// in try/catch: with storage blocked, the page keeps its config in memory.

import { config_normalize, config_set_classic, config_set_option, default_config_text } from "./wasm/buuz.js";

export const CONFIG_KEY = "buuz.config";
const LEGACY_KEYS = ["buuz.mode", "buuz.smart", "buuz.abbr"] as const;

/** The part of localStorage this module uses. */
export interface Store {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

/** A parse error or warning from the core: a code, a line (or number) and a name. */
export interface Issue {
  code: string;
  number: number;
  detail: string;
}

export type ImportResult = { ok: true; text: string; warnings: Issue[] } | { ok: false; error: Issue };

/** Split the core's `code<TAB>number<TAB>detail` string. */
export function parseIssue(raw: unknown): Issue {
  const [code = "other", number = "0", ...rest] = String(raw).split("\t");
  return { code, number: Number(number) || 0, detail: rest.join("\t") };
}

/** The browser's localStorage, or null when it is blocked. */
export function browserStore(): Store | null {
  try {
    return localStorage;
  } catch {
    return null;
  }
}

function read(store: Store | null, key: string): string | null {
  try {
    return store ? store.getItem(key) : null;
  } catch {
    return null;
  }
}

/** Save the config text. Returns false when storage is blocked. */
export function saveConfig(store: Store | null, text: string): boolean {
  try {
    if (!store) return false;
    store.setItem(CONFIG_KEY, text);
    return true;
  } catch {
    return false;
  }
}

/** Build a config from the old per-switch keys (buuz.mode, buuz.smart, buuz.abbr). */
export function migrateLegacy(get: (key: string) => string | null): string {
  let text = default_config_text();
  // Options first, while the preset is smart, so an "off" is always an override.
  if (get("buuz.smart") === "off") text = config_set_option(text, "smart_vowels", false);
  if (get("buuz.abbr") === "off") text = config_set_option(text, "abbreviations", false);
  if (get("buuz.mode") === "classic") text = config_set_classic(text, true);
  return text;
}

/**
 * The config text for this visit: the saved one, else one migrated from the
 * old keys (saved, and the old keys removed), else the default. A saved text
 * that no longer parses is ignored.
 */
export function loadConfig(store: Store | null): string {
  return loadConfigState(store).text;
}

export const BAD_CONFIG_KEY = "buuz.config.bad";

/**
 * Like `loadConfig`, and says when a saved text was unreadable. That text is
 * copied to "buuz.config.bad" before the default is used, so it is not lost.
 */
export function loadConfigState(store: Store | null): { text: string; recovered: boolean } {
  const saved = read(store, CONFIG_KEY);
  if (saved !== null) {
    const r = importConfig(saved);
    if (r.ok) return { text: r.text, recovered: false };
    try {
      store?.setItem(BAD_CONFIG_KEY, saved);
    } catch {
      /* ignore */
    }
    return { text: default_config_text(), recovered: true };
  }
  return { text: loadWithoutSaved(store), recovered: false };
}

function loadWithoutSaved(store: Store | null): string {
  if (LEGACY_KEYS.some((k) => read(store, k) !== null)) {
    const text = migrateLegacy((k) => read(store, k));
    if (saveConfig(store, text)) {
      for (const k of LEGACY_KEYS) {
        try {
          store?.removeItem(k);
        } catch {
          /* ignore */
        }
      }
    }
    return text;
  }
  return default_config_text();
}

/** The switches the dialog shows, as the config gives them. */
export function switchesOf(text: string): {
  classic: boolean;
  smartVowels: boolean;
  abbreviations: boolean;
  completion: boolean;
  /** Classic preset, but overrides turn a smart feature on. */
  classicWithFeatures: boolean;
} {
  const r = config_normalize(text);
  const out = {
    classic: r.classic,
    smartVowels: r.smart_vowels,
    abbreviations: r.abbreviations,
    completion: r.completion,
    classicWithFeatures: r.classic && (r.smart_vowels || r.abbreviations || r.completion),
  };
  r.free();
  return out;
}

export type ApplyResult =
  | { ok: true; text: string; warnings: Issue[]; saved: boolean }
  | { ok: false; error: Issue };

/** Import text: on success save it; on a parse error change nothing. */
export function applyImport(store: Store | null, raw: string): ApplyResult {
  const r = importConfig(raw);
  if (!r.ok) return r;
  return { ok: true, text: r.text, warnings: r.warnings, saved: saveConfig(store, r.text) };
}

/** Check imported text. On success it gives the canonical text and the warnings. */
export function importConfig(raw: string): ImportResult {
  try {
    const report = config_normalize(raw);
    const warnings = Array.from(report.warnings, parseIssue);
    const text = report.text;
    report.free();
    return { ok: true, text, warnings };
  } catch (e) {
    return { ok: false, error: parseIssue(e) };
  }
}
