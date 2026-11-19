// SPDX-License-Identifier: MIT OR Apache-2.0
// Puts the template, the two string files, the word list, the wasm module
// and the scripts into one HTML file. Usage: node build.mjs OUT.html
import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..", "..", "..");
const wasmDir = join(root, "web", "site", "src", "wasm");
const out = process.argv[2];
if (!out) throw new Error("usage: node build.mjs OUT.html");

const read = (p) => readFileSync(p, "utf8");
const safe = (text, what) => {
  if (/<\/script/i.test(text)) throw new Error(`${what} contains "</script"`);
  return text;
};
// JSON inside a script tag: "<" is escaped so no text can close the tag.
const jsonData = (file) => {
  const text = read(file);
  JSON.parse(text); // fail early on a broken file
  return text.replace(/</g, "\\u003c");
};

// Alternative wordings for the Mongolian cells. A missing file means none.
// ALT_FILE overrides the path (used by tests).
const altFile = process.env.ALT_FILE || join(here, "alternatives.mn.json");
let altText = "{}";
if (existsSync(altFile)) {
  altText = jsonData(altFile);
  const n = Object.values(JSON.parse(altText)).reduce((a, v) => a + (Array.isArray(v) ? v.length : 0), 0);
  console.log(`strings-editor: ${n} alternatives from ${altFile}`);
} else {
  console.log(`strings-editor: note: ${altFile} not found, building without alternatives`);
}

// The wasm-bindgen glue is an ES module. Make it a plain script: no export,
// no import.meta. The page calls initSync with the embedded bytes.
let glue = read(join(wasmDir, "buuz.js"));
glue = glue.replace(/^export \{[^}]*\};?\s*$/gm, "").replace(/^export /gm, "").replace(/import\.meta\.url/g, "location.href");

const parts = {
  MN: jsonData(join(root, "macos", "App", "Strings", "strings.mn.json")),
  EN: jsonData(join(root, "macos", "App", "Strings", "strings.en.json")),
  ALT: altText,
  WORDS: safe(read(join(root, "data", "wordlist", "mn.txt")), "word list"),
  WASM: readFileSync(join(wasmDir, "buuz_bg.wasm")).toString("base64"),
  GLUE: safe(glue, "glue"),
  LOGIC: safe(read(join(here, "logic.js")), "logic.js"),
  APP: safe(read(join(here, "app.js")), "app.js"),
};

let html = read(join(here, "template.html"));
for (const [k, v] of Object.entries(parts)) html = html.split(`/*@${k}@*/`).join("\u0000" + k + "\u0000");
for (const [k, v] of Object.entries(parts)) html = html.split("\u0000" + k + "\u0000").join(v);
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, html);
console.log(`strings-editor: wrote ${out} (${(html.length / 1024).toFixed(0)} KiB)`);
