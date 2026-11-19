// SPDX-License-Identifier: MIT OR Apache-2.0
// Node checks for logic.js with the real Buuz converter.
// Run: node macos/tools/strings-editor/test.mjs (after build.sh built web/site/src/wasm)
import { readFileSync, writeFileSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..", "..", "..");
const require = createRequire(import.meta.url);
const logic = require("./logic.js");
const glue = await import(join(root, "web/site/src/wasm/buuz.js"));
glue.initSync({ module: readFileSync(join(root, "web/site/src/wasm/buuz_bg.wasm")) });
const config = glue.default_config_text();
glue.load_word_list(readFileSync(join(root, "data/wordlist/mn.txt"), "utf8"));
const convert = (s) => glue.convert_with_config(s, config);

// Type text key by key like the page does. Returns the field value.
function type(text, start = "") {
  const typer = logic.createTyper(convert);
  let value = start;
  let caret = value.length;
  for (const ch of text) {
    const ev = ch === "\n" ? { inputType: "insertLineBreak", data: null } : { inputType: "insertText", data: ch };
    const r = typer.handle({ value, start: caret, end: caret }, ev);
    if (r) ({ value, caret } = r);
    else {
      value = value.slice(0, caret) + ch + value.slice(caret);
      caret += ch.length;
    }
  }
  return { value, caret, typer };
}
let n = 0;
const ok = (name, fn) => (fn(), n++, console.log("ok", name));

ok("converter works", () => assert.equal(convert("geree"), "гэрээ"));
ok("typing sain baina uu", () => assert.equal(type("sain baina uu").value, "сайн байна уу"));
ok("typing matches whole-text conversion", () => {
  for (const s of ["geree", "sh", "ts", "zh", "bayarlalaa", "nom", "yamar uu", "ödör", "hun"])
    assert.equal(type(s).value, convert(s), s);
});
ok("sh becomes one letter", () => assert.equal(type("sh").value, convert("sh")));
ok("placeholder stays Latin", () => assert.equal(type("Алхам {step} / {total}").value, "Алхам {step} / {total}"));
ok("braces typed in Latin stay", () => assert.equal(type("sain {key} baina").value, "сайн {key} байна"));
ok("emoji and Cyrillic pass", () => assert.equal(type("🎉 Сайн bai").value, "🎉 Сайн бай"));
ok("backspace removes one Latin letter", () => {
  const t = type("sai");
  const r = t.typer.handle({ value: t.value, start: t.caret, end: t.caret }, { inputType: "deleteContentBackward" });
  assert.equal(r.value, convert("sa"));
});
ok("paste is not converted", () => {
  const typer = logic.createTyper(convert);
  assert.equal(typer.handle({ value: "", start: 0, end: 0 }, { inputType: "insertFromPaste", data: null }), null);
});
ok("punctuation in an English-like sentence", () => assert.equal(type("sain. ").value, convert("sain. ")));
ok("placeholder helpers", () => {
  assert.deepEqual(logic.placeholders("a {b} {a} {b}"), ["{a}", "{b}"]);
  assert.ok(logic.samePlaceholders("x {a}", "{a} y"));
  assert.ok(!logic.samePlaceholders("{a}", "{b}"));
});

// JSON round trip: the output of an unedited model equals the file bytes.
const mnText = readFileSync(join(root, "macos/App/Strings/strings.mn.json"), "utf8");
const enText = readFileSync(join(root, "macos/App/Strings/strings.en.json"), "utf8");
const model = logic.createModel(JSON.parse(mnText), JSON.parse(enText));
ok("round trip mn", () => assert.equal(model.toJson("mn"), mnText));
ok("round trip en", () => assert.equal(model.toJson("en"), enText));
ok("changes list and revert", () => {
  const k = model.order[0];
  const before = model.value("mn", k);
  model.set("mn", k, before + "!");
  assert.ok(model.isChanged("mn", k));
  assert.match(model.changesList(), new RegExp(k.replace(".", "\\.") + " \\[mn\\]"));
  assert.notEqual(model.toJson("mn"), mnText);
  model.revert(k);
  assert.equal(model.toJson("mn"), mnText);
  assert.equal(model.changesList(), "");
});

// Alternatives: pure logic.
ok("normalizeAlternatives cleans data", () => {
  assert.deepEqual(logic.normalizeAlternatives({ a: ["x", "x", "", 3, "y"], b: [], c: "no" }), { a: ["x", "y"] });
  assert.deepEqual(logic.normalizeAlternatives(null), {});
  assert.deepEqual(logic.normalizeAlternatives([1]), {});
});
ok("alternativeItems: original first, chosen marked", () => {
  const items = logic.alternativeItems("A {n} 🎉", ["B {n}", "A {n} 🎉", "C"], "B {n}");
  assert.deepEqual(items.map((i) => i.text), ["A {n} 🎉", "B {n}", "C"]);
  assert.deepEqual(items.map((i) => i.chosen), [false, true, false]);
  assert.deepEqual(items.map((i) => i.original), [true, false, false]);
  assert.equal(logic.alternativeCount("A {n} 🎉", ["B {n}", "A {n} 🎉", "C"]), 2);
});
ok("alternativeItems: none gives no menu", () => {
  assert.deepEqual(logic.alternativeItems("A", [], "A"), []);
  assert.deepEqual(logic.alternativeItems("A", undefined, "A"), []);
  assert.deepEqual(logic.alternativeItems("A", ["A"], "A"), []);
});
ok("original is chosen when the cell is unchanged", () => {
  const items = logic.alternativeItems("A", ["B"], "A");
  assert.deepEqual(items.map((i) => i.chosen), [true, false]);
});
ok("moveActive wraps", () => {
  assert.equal(logic.moveActive(-1, 3, 1), 0);
  assert.equal(logic.moveActive(-1, 3, -1), 2);
  assert.equal(logic.moveActive(2, 3, 1), 0);
  assert.equal(logic.moveActive(0, 3, -1), 2);
  assert.equal(logic.moveActive(0, 0, 1), -1);
});
ok("picking sets the cell, marks it changed, and the original reverts it", () => {
  const m = logic.createModel({ k: "A" }, { k: "a" });
  const items = logic.alternativeItems(m.orig.mn.k, ["B"], m.value("mn", "k"));
  assert.equal(logic.pickAlternative(m, "k", items, 1), "B");
  assert.equal(m.value("mn", "k"), "B");
  assert.ok(m.isChanged("mn", "k"));
  const again = logic.alternativeItems(m.orig.mn.k, ["B"], m.value("mn", "k"));
  assert.ok(again[1].chosen);
  logic.pickAlternative(m, "k", again, 0);
  assert.ok(!m.isChanged("mn", "k"));
  assert.equal(logic.pickAlternative(m, "k", again, 9), null);
});

// Build with a missing alternatives file: a note, and "{}" in the page.
ok("build without alternatives file", () => {
  const dir = mkdtempSync(join(tmpdir(), "se-"));
  const out = join(dir, "o.html");
  const log = execFileSync("node", [join(here, "build.mjs"), out], { env: { ...process.env, ALT_FILE: join(dir, "missing.json") } }).toString();
  assert.match(log, /not found, building without alternatives/);
  assert.match(readFileSync(out, "utf8"), /id="data-alt" type="application\/json">\{\}<\/script>/);
});
ok("build with an alternatives file", () => {
  const dir = mkdtempSync(join(tmpdir(), "se-"));
  const alt = join(dir, "alt.json");
  writeFileSync(alt, JSON.stringify({ k: ["a <b>", "c"], z: [] }));
  const out = join(dir, "o.html");
  execFileSync("node", [join(here, "build.mjs"), out], { env: { ...process.env, ALT_FILE: alt } });
  assert.match(readFileSync(out, "utf8"), /"a \\u003cb>"/);
});
console.log(`${n} checks passed`);
