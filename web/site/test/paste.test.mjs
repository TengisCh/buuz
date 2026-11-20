// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readPaste } from "../src/paste.ts";
import { ALL_LANGS, strings } from "../src/i18n.ts";

test("a paste returns the clipboard text", async () => {
  assert.deepEqual(await readPaste({ readText: async () => "geree" }), { kind: "text", text: "geree" });
});

test("an empty clipboard is reported as empty", async () => {
  assert.deepEqual(await readPaste({ readText: async () => "" }), { kind: "empty" });
});

test("a refused or missing clipboard is reported as blocked, never thrown", async () => {
  const denied = { readText: async () => Promise.reject(new DOMException("no", "NotAllowedError")) };
  assert.deepEqual(await readPaste(denied), { kind: "blocked" });
  assert.deepEqual(await readPaste(undefined), { kind: "blocked" });
  assert.deepEqual(await readPaste({}), { kind: "blocked" });
});

test("the paste strings exist in Mongolian, English and Russian", () => {
  for (const l of ["mn", "en", "ru"]) {
    for (const k of ["paste", "pasteBlocked", "pasteEmpty"]) assert.ok(strings[l][k], `${l}.${k}`);
  }
  assert.ok(ALL_LANGS.includes("mn"));
});
