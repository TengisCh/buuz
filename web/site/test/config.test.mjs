// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { config_set_classic, config_set_option, convert_with, convert_with_config, default_config_text } from "../src/wasm/buuz.js";
import { BAD_CONFIG_KEY, CONFIG_KEY, applyImport, importConfig, loadConfigState, loadConfig, migrateLegacy, saveConfig, switchesOf } from "../src/config.ts";
import { issueText, strings } from "../src/i18n.ts";

await init({ module_or_path: await readFile(new URL("../src/wasm/buuz_bg.wasm", import.meta.url)) });

/** A fake localStorage; `blocked` makes every call throw, like a blocked browser. */
function fakeStore(initial = {}, blocked = false) {
  const data = new Map(Object.entries(initial));
  const guard = () => {
    if (blocked) throw new DOMException("blocked", "SecurityError");
  };
  return {
    data,
    getItem: (k) => (guard(), data.get(k) ?? null),
    setItem: (k, v) => (guard(), void data.set(k, String(v))),
    removeItem: (k) => (guard(), void data.delete(k)),
  };
}

test("the default config converts exactly like the current smart output", () => {
  const cfg = default_config_text();
  for (const input of ["unee", "irsen uu", "bn uu ", "aaviin", "sain baina uu", "github.com", "geree\nbn."]) {
    assert.equal(convert_with_config(input, cfg), convert_with(input, true, true), input);
  }
});

test("a first visit with no saved keys gets the default config", () => {
  const store = fakeStore();
  assert.equal(loadConfig(store), default_config_text());
  assert.equal(store.data.size, 0);
});

test("old keys migrate into a config, which is saved; the old keys go", () => {
  const store = fakeStore({ "buuz.mode": "classic", "buuz.smart": "off", "buuz.abbr": "off" });
  const cfg = loadConfig(store);
  assert.equal(store.getItem(CONFIG_KEY), cfg);
  assert.equal(store.data.has("buuz.mode") || store.data.has("buuz.smart") || store.data.has("buuz.abbr"), false);
  assert.deepEqual(switchesOf(cfg), { classic: true, smartVowels: false, abbreviations: false, completion: false, autocorrect: false, doubleV: false, habitKeys: false, classicWithFeatures: false });
  assert.equal(convert_with_config("unee", cfg), "унээ");
});

test("each old key migrates on its own", () => {
  assert.deepEqual(switchesOf(migrateLegacy((k) => (k === "buuz.smart" ? "off" : null))), {
    classic: false,
    smartVowels: false,
    abbreviations: true,
    completion: true,
    autocorrect: false,
    doubleV: true,
    habitKeys: true,
    classicWithFeatures: false,
  });
  const abbr = migrateLegacy((k) => (k === "buuz.abbr" ? "off" : null));
  assert.equal(convert_with_config("bn ", abbr), "бн ");
  assert.equal(convert_with_config("unee", abbr), "үнээ");
  // "on" values and an empty store give the default.
  assert.equal(migrateLegacy((k) => (k === "buuz.abbr" ? "on" : null)), default_config_text());
});

test("a saved config wins over old keys", () => {
  const saved = default_config_text();
  const store = fakeStore({ [CONFIG_KEY]: saved, "buuz.mode": "classic" });
  assert.equal(loadConfig(store), saved);
});

test("a corrupt saved config falls back to the default", () => {
  assert.equal(loadConfig(fakeStore({ [CONFIG_KEY]: "garbage" })), default_config_text());
});

test("blocked storage keeps the page working", () => {
  const store = fakeStore({}, true);
  assert.equal(loadConfig(store), default_config_text());
  assert.equal(saveConfig(store, default_config_text()), false);
  assert.equal(loadConfig(null), default_config_text());
  assert.equal(saveConfig(null, "x"), false);
});

test("export then import gives the same config, and the same output", () => {
  const custom = "buuz_config = 1\npreset = \"classic\"\n\n[options]\nkeep_latin = true\n\n[abbreviations]\nzz = \"заавал\"\n";
  const exported = importConfig(custom);
  assert.ok(exported.ok);
  const again = importConfig(exported.text);
  assert.ok(again.ok);
  assert.equal(again.text, exported.text);
  assert.deepEqual(again.warnings, []);
  assert.equal(convert_with_config("unee", exported.text), convert_with_config("unee", again.text));
  // The store round trip keeps it too.
  const store = fakeStore();
  saveConfig(store, exported.text);
  assert.equal(loadConfig(store), exported.text);
});

test("a bad import gives an error and no text", () => {
  const r = importConfig("buuz_config = 1\nnot a line\n");
  assert.equal(r.ok, false);
  assert.deepEqual(r.error, { code: "syntax", number: 2, detail: "" });
  assert.equal(importConfig("hello").error.code, "missing_version");
  assert.deepEqual(importConfig("buuz_config = 99").error, { code: "too_new", number: 99, detail: "1" });
});

test("unknown options come back as warnings, not errors", () => {
  const r = importConfig("buuz_config = 1\n[options]\nfuture_thing = true\n[extra]\nx = 1\n");
  assert.ok(r.ok);
  assert.deepEqual(
    r.warnings.map((w) => [w.code, w.number, w.detail]),
    [
      ["unknown_option", 3, "future_thing"],
      ["unknown_section", 4, "extra"],
    ],
  );
});

test("every error and warning code has a Mongolian and an English message", () => {
  for (const code of [
    "missing_version", "too_new", "syntax", "unknown_key", "bad_value", "bad_escape", "single_quotes",
    "duplicate_key", "too_many_entries", "duplicate_section", "bad_abbreviation", "bad_keys", "too_large", "other",
    "unknown_option", "unknown_section",
  ]) {
    for (const lang of ["mn", "en"]) {
      assert.ok(strings[lang][`cfg_${code}`], `${lang} cfg_${code}`);
    }
  }
  // The page starts in Mongolian; the message carries the line and the name.
  const msg = issueText({ code: "unknown_option", number: 3, detail: "a$&b" });
  assert.ok(msg.includes("3") && msg.includes("a$&b"));
});

test("migration keeps the old keys when the config cannot be saved", () => {
  const store = fakeStore({ "buuz.mode": "classic", "buuz.abbr": "off" });
  store.setItem = () => {
    throw new DOMException("full", "QuotaExceededError");
  };
  const cfg = loadConfig(store);
  assert.equal(switchesOf(cfg).classic, true);
  assert.equal(store.data.get("buuz.mode"), "classic");
  assert.equal(store.data.get("buuz.abbr"), "off");
});

test("a bad import leaves the saved config unchanged", () => {
  const store = fakeStore();
  const good = default_config_text();
  saveConfig(store, good);
  const r = applyImport(store, "buuz_config = 1\nnot a line\n");
  assert.equal(r.ok, false);
  assert.equal(store.getItem(CONFIG_KEY), good);
  const ok = applyImport(store, 'buuz_config = 1\npreset = "classic"\n');
  assert.ok(ok.ok && ok.saved);
  assert.equal(store.getItem(CONFIG_KEY), ok.text);
  // Blocked storage: the import still works, and says it was not saved.
  const blocked = applyImport(fakeStore({}, true), "buuz_config = 1\n");
  assert.ok(blocked.ok && !blocked.saved);
});

test("a corrupt saved config is kept in buuz.config.bad before the default is used", () => {
  const store = fakeStore({ [CONFIG_KEY]: "garbage" });
  assert.deepEqual(loadConfigState(store), { text: default_config_text(), recovered: true });
  assert.equal(store.getItem(BAD_CONFIG_KEY), "garbage");
  assert.equal(loadConfigState(fakeStore()).recovered, false);
});

test("classic with overrides that turn smart features on is reported", () => {
  const s = switchesOf('buuz_config = 1\npreset = "classic"\n[options]\nsmart_vowels = true\n');
  assert.deepEqual(s, { classic: true, smartVowels: true, abbreviations: false, completion: false, autocorrect: false, doubleV: false, habitKeys: false, classicWithFeatures: true });
});

test("the size limit has one wording, and entry counts have their own message", () => {
  assert.deepEqual(importConfig("#".repeat(1024 * 1024 + 1)).error, { code: "too_large", number: 0, detail: "1 MiB" });
  assert.ok(issueText({ code: "too_many_entries", number: 7, detail: "10000" }).includes("7"));
});

test("double v is a switch: on in smart, off in classic, and it types vv as үү", () => {
  const cfg = default_config_text();
  assert.equal(switchesOf(cfg).doubleV, true);
  assert.equal(convert_with_config("svv", cfg), "сүү");
  const off = config_set_option(cfg, "double_v", false);
  assert.equal(switchesOf(off).doubleV, false);
  assert.equal(convert_with_config("svv", off), "свв");
  const classic = config_set_classic(cfg, true);
  assert.equal(switchesOf(classic).doubleV, false);
  // Only double v on in Classic counts as a smart feature.
  const classicOn = config_set_option(classic, "double_v", true);
  assert.equal(switchesOf(classicOn).classicWithFeatures, true);
});

test("habit keys are a switch: on in smart, off in classic, and they type ts as ц", () => {
  const cfg = default_config_text();
  assert.equal(switchesOf(cfg).habitKeys, true);
  assert.equal(convert_with_config("tsagaan", cfg), "цагаан");
  assert.equal(convert_with_config("gezh", cfg), "гэж");
  const off = config_set_option(cfg, "habit_keys", false);
  assert.equal(switchesOf(off).habitKeys, false);
  assert.equal(convert_with_config("tsagaan", off), "тсагаан");
  const classic = config_set_classic(cfg, true);
  assert.equal(switchesOf(classic).habitKeys, false);
  const classicOn = config_set_option(classic, "habit_keys", true);
  assert.equal(switchesOf(classicOn).classicWithFeatures, true);
});
