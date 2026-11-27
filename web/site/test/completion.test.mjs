// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { config_set_classic, convert_with_config, config_set_option, default_config_text, load_word_list } from "../src/wasm/buuz.js";
import { accept, completionAt, ensureWordList, resetWordListState, sameOrigin } from "../src/completion.ts";
import { switchesOf } from "../src/config.ts";

await init({ module_or_path: await readFile(new URL("../src/wasm/buuz_bg.wasm", import.meta.url)) });

const BASE = "https://buuz.example/";
const cfg = default_config_text();
const LIST = "# test\nгэрлэх\t100\nгэрлэлт\t10\nбайна\t500\n";

test("a failed load keeps completion off and says nothing", async () => {
  const bad = async () => {
    throw new Error("offline");
  };
  assert.equal(await ensureWordList("/assets/mn.txt", BASE, bad), false);
  resetWordListState();
  assert.equal(await ensureWordList("/assets/mn.txt", BASE, async () => ({ ok: false, text: async () => "" })), false);
  resetWordListState();
  assert.equal(await ensureWordList("/assets/mn.txt", BASE, async () => ({ ok: true, text: async () => "no tab here" })), false);
  assert.equal(completionAt("gerl", 4, cfg), null);
});

test("the list loads only from the site's own origin", async () => {
  assert.equal(sameOrigin("/assets/mn.txt", BASE), true);
  assert.equal(sameOrigin("https://evil.example/mn.txt", BASE), false);
  let called = false;
  const f = async () => {
    called = true;
    return { ok: true, text: async () => LIST };
  };
  assert.equal(await ensureWordList("https://evil.example/mn.txt", BASE, f), false);
  assert.equal(called, false);
});

test("a failed load stays failed for the visit", async () => {
  resetWordListState();
  let calls = 0;
  const f = async () => {
    calls++;
    throw new Error("offline");
  };
  assert.equal(await ensureWordList("/assets/mn.txt", BASE, f), false);
  assert.equal(await ensureWordList("/assets/mn.txt", BASE, f), false);
  assert.equal(calls, 1);
});

test("a good load is parsed once and completion works", async () => {
  resetWordListState();
  let calls = 0;
  const f = async (url, init) => {
    calls++;
    assert.equal(init.credentials, "omit");
    return { ok: true, text: async () => LIST };
  };
  assert.equal(await ensureWordList("/assets/mn.txt", BASE, f), true);
  assert.equal(await ensureWordList("/assets/mn.txt", BASE, f), true);
  assert.equal(calls, 1);
  const c = completionAt("sain gerl", 9, cfg);
  assert.deepEqual(c, { rest: "эх", word: "гэрлэх", replaces: 4 });
  assert.equal(completionAt("gerl", 4, config_set_option(cfg, "completion", false)), null);
});

test("accepting replaces the Latin word and the box stays convertible", () => {
  load_word_list(LIST);
  const value = "sain gerl baina";
  const c = completionAt(value, 9, cfg);
  const r = accept(value, 9, c);
  assert.equal(r.value, "sain гэрлэх baina");
  assert.equal(r.caret, 11);
  assert.equal(convert_with_config(r.value, cfg), "сайн гэрлэх байна");
});

test("no completion in the middle of a non-word", () => {
  load_word_list(LIST);
  assert.equal(completionAt("gerl ", 5, cfg), null);
  assert.equal(completionAt("", 0, cfg), null);
});

test("no completion when the caret is inside a word", () => {
  load_word_list("# t\nмонгол\t50\n");
  assert.equal(completionAt("mongxyz", 4, cfg), null);
  assert.equal(completionAt("mongol", 4, cfg), null);
  assert.deepEqual(completionAt("mong", 4, cfg), { rest: "ол", word: "монгол", replaces: 4 });
  assert.deepEqual(completionAt("mong xyz", 4, cfg), { rest: "ол", word: "монгол", replaces: 4 });
});

test("Ulaanb gives Улаанбаатар, and a Latin run after Cyrillic or a digit is a new word", () => {
  load_word_list("# t\nулаанбаатар\t50\nулаан\t5\n");
  assert.deepEqual(completionAt("Ulaanb", 6, cfg), { rest: "аатар", word: "Улаанбаатар", replaces: 6 });
  assert.equal(completionAt("1Ulaanb", 7, cfg)?.replaces, 6);
  assert.equal(completionAt("гэрUlaanb", 9, cfg)?.replaces, 6);
});

test("autocorrect: the converter corrects a finished word once the list is loaded", () => {
  const on = config_set_option(cfg, "autocorrect", true);
  assert.equal(switchesOf(on).autocorrect, true);
  assert.equal(switchesOf(cfg).autocorrect, false);
  load_word_list("# t\nдэлгүүр\t100\n");
  // After the load, the same input converts again with the correction.
  assert.equal(convert_with_config("delgur ", on), "дэлгүүр ");
  // A newline does not correct. Word choice is off here, so only autocorrect could.
  const acOnly = config_set_option(on, "word_choice", false);
  assert.equal(convert_with_config("delgur\n", acOnly), "дэлгүр\n");
  assert.equal(convert_with_config("delgur", acOnly), "дэлгүр");
  // Word choice (on by default) also finds it: у may be ү, and a vowel may be long.
  assert.equal(convert_with_config("delgur ", cfg), "дэлгүүр ");
  const neither = config_set_option(cfg, "word_choice", false);
  assert.equal(convert_with_config("delgur ", neither), "дэлгүр ");
  // Plain Classic never corrects.
  const classic = config_set_classic(cfg, true);
  assert.equal(convert_with_config("delgur ", classic), "дэлгүр ");
});

test("word choice: the converter picks ө or ү with the list, and the switch turns it off", () => {
  assert.equal(switchesOf(cfg).wordChoice, true);
  load_word_list("# t\nтөв\t100\n");
  assert.equal(convert_with_config("tov ", cfg), "төв ");
  // Word choice also runs before a newline and on the last word of the text.
  assert.equal(convert_with_config("tov\n", cfg), "төв\n");
  assert.equal(convert_with_config("tov", cfg), "төв");
  const off = config_set_option(cfg, "word_choice", false);
  assert.equal(switchesOf(off).wordChoice, false);
  assert.equal(convert_with_config("tov ", off), "тов ");
  const classic = config_set_classic(cfg, true);
  assert.equal(switchesOf(classic).wordChoice, false);
  assert.equal(convert_with_config("tov ", classic), "тов ");
  const classicOn = config_set_option(classic, "word_choice", true);
  assert.equal(switchesOf(classicOn).classicWithFeatures, true);
});

test("suffix choice: the converter fixes the case ending with the list, and the switch turns it off", () => {
  assert.equal(switchesOf(cfg).suffixChoice, true);
  load_word_list("# t\nус\t3038\nусны\t3131\nусын\t12\n");
  const base = config_set_option(cfg, "word_choice", false);
  assert.equal(convert_with_config("usiin ", base), "усны ");
  const off = config_set_option(base, "suffix_choice", false);
  assert.equal(switchesOf(off).suffixChoice, false);
  assert.equal(convert_with_config("usiin ", off), "усын ");
  const classic = config_set_classic(cfg, true);
  assert.equal(switchesOf(classic).suffixChoice, false);
  const classicOn = config_set_option(classic, "suffix_choice", true);
  assert.equal(switchesOf(classicOn).classicWithFeatures, true);
});

test("rounding harmony: the converter rounds the suffix with the list, and the switch turns it off", () => {
  assert.equal(switchesOf(cfg).roundingHarmony, true);
  load_word_list("# t\nболсон\t500\nболсан\t3\n");
  assert.equal(convert_with_config("bolsan ", cfg), "болсон ");
  const off = config_set_option(cfg, "rounding_harmony", false);
  assert.equal(switchesOf(off).roundingHarmony, false);
  assert.equal(convert_with_config("bolsan ", off), "болсан ");
  const classic = config_set_classic(cfg, true);
  assert.equal(switchesOf(classic).roundingHarmony, false);
  const classicOn = config_set_option(classic, "rounding_harmony", true);
  assert.equal(switchesOf(classicOn).classicWithFeatures, true);
});
