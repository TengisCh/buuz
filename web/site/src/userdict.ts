// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// The personal dictionary (docs/typing-rules.md section 14) as text in
// localStorage under "buuz.userdict". No DOM here, so the node tests run it.
// The text stays on this device and is not part of the config export. Every
// storage access is in try/catch: with storage blocked, the dictionary lives
// for this visit only. Each change also goes to the converter with
// `set_user_dict`. The page never learns words on its own: see section 14.

import { set_user_dict, userdict_add, userdict_clear, userdict_entries, userdict_normalize, userdict_remove } from "./wasm/buuz.js";
import { parseIssue, type Issue, type Store } from "./config.ts";

export const USERDICT_KEY = "buuz.userdict";

export interface UserEntry {
  kind: "add" | "use" | "reject";
  word: string;
  count: number;
}

function use(text: string): string {
  try {
    set_user_dict(text);
  } catch {
    /* too large: keep the old one */
  }
  return text;
}

function save(store: Store | null, text: string): boolean {
  try {
    if (!store) return false;
    if (text === "") store.removeItem(USERDICT_KEY);
    else store.setItem(USERDICT_KEY, text);
    return true;
  } catch {
    return false;
  }
}

/** The saved dictionary text, made current for the converter. Empty when none, blocked or unreadable. */
export function loadUserDict(store: Store | null): string {
  let raw: string | null = null;
  try {
    raw = store ? store.getItem(USERDICT_KEY) : null;
  } catch {
    raw = null;
  }
  if (raw === null) return use("");
  try {
    const r = userdict_normalize(raw);
    const text = r.text;
    r.free();
    return use(text);
  } catch {
    return use("");
  }
}

/** The entries of a dictionary text, the least recently used first. */
export function entriesOf(text: string): UserEntry[] {
  return Array.from(userdict_entries(text), (line) => {
    const [kind, word, count] = line.split("\t");
    return { kind: kind as UserEntry["kind"], word, count: Number(count) };
  });
}

export type AddResult = { ok: true; text: string; saved: boolean } | { ok: false; error: Issue };

/** Add a word (Cyrillic). The core checks it. */
export function addUserWord(store: Store | null, text: string, word: string): AddResult {
  try {
    const next = userdict_add(text, word);
    return { ok: true, text: use(next), saved: save(store, next) };
  } catch (e) {
    return { ok: false, error: parseIssue(e) };
  }
}

/** Delete every entry of a word. */
export function removeUserWord(store: Store | null, text: string, word: string): { text: string; saved: boolean } {
  const next = userdict_remove(text, word);
  return { text: use(next), saved: save(store, next) };
}

/** Forget everything and remove the stored key. */
export function clearUserDict(store: Store | null): { text: string; saved: boolean } {
  const next = userdict_clear();
  return { text: use(next), saved: save(store, next) };
}

export type UserImportResult = { ok: true; text: string; warnings: Issue[]; saved: boolean } | { ok: false; error: Issue };

/**
 * Merge an imported file into the dictionary. Entries already there stay as
 * they are. Bad lines of the file come back as warnings.
 */
export function importUserDict(store: Store | null, text: string, raw: string): UserImportResult {
  try {
    const file = userdict_normalize(raw);
    const warnings = Array.from(file.warnings, parseIssue);
    const incoming = file.text;
    file.free();
    // A word that is already there is a duplicate here, and the first line wins.
    const merged = userdict_normalize(text + incoming);
    const next = merged.text;
    merged.free();
    return { ok: true, text: use(next), warnings, saved: save(store, next) };
  } catch (e) {
    return { ok: false, error: parseIssue(e) };
  }
}
