// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { convert_with_config, default_config_text } from "../src/wasm/buuz.js";
import { ALPHABET, CalibrationFlow, coverage, saveError, uncovered } from "../src/calibration.ts";
import sentences from "../src/calibration.json" with { type: "json" };
import { parseIssue } from "../src/config.ts";
import { issueText, strings } from "../src/i18n.ts";

await init({ module_or_path: await readFile(new URL("../src/wasm/buuz_bg.wasm", import.meta.url)) });

test("calibration.json covers the alphabet except п, ф and щ", () => {
  const missing = uncovered(sentences);
  console.log(`covered ${coverage(sentences).length} of ${ALPHABET.length} letters; not covered: ${missing.join(" ")}`);
  assert.deepEqual(missing, ["п", "ф", "щ"]);
  assert.ok(sentences.length >= 8 && sentences.length <= 12);
  for (const s of sentences) assert.ok(!/\p{Extended_Pictographic}/u.test(s), s);
});

test("a typist who follows the default table gets nothing to accept", () => {
  const flow = new CalibrationFlow(["гэр", "сайн"]);
  flow.setAnswer(0, "ger");
  flow.setAnswer(1, "sain");
  const r = flow.finish();
  assert.equal(r.suggestions.length, 0);
  assert.deepEqual(r.unmatchedSentences, []);
});

test("accepting a subset changes only those keys", () => {
  const flow = new CalibrationFlow(["гүү", "үнэ", "цай", "цай"]);
  flow.setAnswer(0, "gvv");
  flow.setAnswer(1, "vne");
  flow.setAnswer(2, "hai");
  flow.setAnswer(3, "hai");
  const r = flow.finish();
  const at = (letter) => r.suggestions.findIndex((s) => s.letter === letter);
  assert.ok(at("ү") >= 0 && at("ц") >= 0);
  const cfg = default_config_text();
  const onlyV = flow.save(cfg, [at("ү")]);
  assert.equal(convert_with_config("v", onlyV), "ү");
  assert.equal(convert_with_config("h", onlyV), "х");
  const both = flow.save(cfg, [at("ү"), at("ц")]);
  assert.equal(convert_with_config("h", both), "ц");
  // Nothing accepted: the config is the same text.
  assert.equal(flow.save(cfg, []), cfg);
  // The suggestion says what it displaces.
  assert.equal(r.suggestions[at("ү")].displaces, "в");
});

test("sentences left empty are not sent, and unmatched ones are named by their number", () => {
  const flow = new CalibrationFlow(["гэр", "сайн", "бууз"]);
  flow.setAnswer(0, "ger");
  flow.setAnswer(2, "qqqqqqqqqqqqqqqqqqqqqqqq");
  const r = flow.finish();
  assert.deepEqual(r.unmatchedSentences, [3]);
  assert.deepEqual(r.notTyped, [2]);
});

test("cancel leaves the config unchanged and drops what was typed", () => {
  const cfg = default_config_text();
  const flow = new CalibrationFlow(["гүү"]);
  flow.setAnswer(0, "gvv");
  assert.equal(flow.finish().suggestions.length, 1);
  assert.equal(flow.answerCount(), 1);
  flow.cancel();
  assert.equal(flow.answerCount(), 0);
  assert.equal(flow.getAnswer(0), "");
  assert.throws(() => flow.finish());
  // After cancel nothing can be saved, so the page's config text stays as it was.
  assert.throws(() => flow.save(cfg, [0]));
  assert.equal(convert_with_config("v", cfg), "в");
});

test("a new flow is independent of an older one that was cancelled", () => {
  const old = new CalibrationFlow(["гүү"]);
  old.setAnswer(0, "gvv");
  const fresh = new CalibrationFlow(["гүү"]);
  fresh.setAnswer(0, "gvv");
  old.cancel(); // a late "close" for the old flow
  assert.equal(fresh.answerCount(), 1);
  assert.equal(fresh.finish().suggestions.length, 1);
});

test("all answers empty: nothing to suggest, every sentence not typed, nothing usable", () => {
  const flow = new CalibrationFlow(["гэр", "сайн", "бууз"]);
  const r = flow.finish();
  assert.equal(r.suggestions.length, 0);
  assert.deepEqual(r.notTyped, [1, 2, 3]);
  assert.equal(r.usable, 0);
});

test("usable counts typed sentences that matched", () => {
  const flow = new CalibrationFlow(["гэр", "сайн", "бууз"]);
  flow.setAnswer(0, "ger");
  flow.setAnswer(1, "qqqqqqqqqqqqqqqqqqqqqqqq");
  const r = flow.finish();
  assert.equal(r.usable, 1);
  assert.deepEqual(r.unmatchedSentences, [2]);
  const none = new CalibrationFlow(["гэр"]);
  none.setAnswer(0, "qqqqqqqqqqqqqqqqqqqqqqqq");
  assert.equal(none.finish().usable, 0);
});

test("save errors: tagged strings are translated, JS errors get the fallback", () => {
  const tr = (raw) => issueText(parseIssue(raw));
  assert.equal(saveError("bad_keys\t4\tv", tr, "fallback"), issueText({ code: "bad_keys", number: 4, detail: "v" }));
  assert.equal(saveError(new Error("bad suggestion index"), tr, "fallback"), "fallback");
  assert.equal(saveError(new TypeError("x"), tr, "fallback"), "fallback");
});

test("typed text is dropped by cancel after a save too", () => {
  const flow = new CalibrationFlow(["гүү"]);
  flow.setAnswer(0, "gvv");
  flow.finish();
  flow.save(default_config_text(), [0]);
  flow.cancel();
  assert.equal(flow.answerCount(), 0);
});

test("a bad accepted index is an error, not a silent change", () => {
  const flow = new CalibrationFlow(["гүү"]);
  flow.setAnswer(0, "gvv");
  flow.finish();
  assert.throws(() => flow.save(default_config_text(), [9]));
});

test("calibration strings have the same placeholders in both languages", () => {
  const keys = Object.keys(strings.mn).filter((k) => k.startsWith("cal"));
  assert.ok(keys.length > 10);
  const holes = (text) => (text.match(/\{\w+\}/g) ?? []).sort().join(",");
  for (const k of keys) {
    assert.ok(strings.en[k] && strings.mn[k], k);
    assert.equal(holes(strings.mn[k]), holes(strings.en[k]), k);
    // No Mongolian suffix glued to a placeholder, which would be wrong for some values.
    assert.ok(!/\}-/.test(strings.mn[k]), `${k}: suffix after a placeholder`);
  }
});
