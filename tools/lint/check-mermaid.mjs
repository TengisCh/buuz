// SPDX-License-Identifier: MIT OR Apache-2.0
// Parses every ```mermaid block in the given Markdown files with the real
// Mermaid parser (in headless Chromium), so a broken diagram fails CI instead
// of showing "Unable to render rich display" on GitHub.
//
// Usage: node tools/lint/check-mermaid.mjs [files...]   (default: all *.md)
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const require = createRequire(import.meta.url);
const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..", "..");
const mermaidJs = require.resolve("mermaid/dist/mermaid.min.js");

function findMarkdown(dir, out = []) {
  for (const name of readdirSync(dir)) {
    if (["node_modules", "target", ".git"].includes(name)) continue;
    const p = join(dir, name);
    if (statSync(p).isDirectory()) findMarkdown(p, out);
    else if (name.endsWith(".md")) out.push(p);
  }
  return out;
}

const files = process.argv.length > 2 ? process.argv.slice(2) : findMarkdown(root);
const browser = await chromium.launch();
const page = await browser.newPage();
await page.addScriptTag({ path: mermaidJs });

let checked = 0;
let failed = 0;
for (const file of files) {
  const text = readFileSync(file, "utf8");
  const re = /^```mermaid\r?\n([\s\S]*?)^```/gm;
  let m;
  while ((m = re.exec(text))) {
    const line = text.slice(0, m.index).split("\n").length;
    const error = await page.evaluate(async (code) => {
      try {
        await window.mermaid.parse(code);
        return null;
      } catch (e) {
        return String(e?.message ?? e);
      }
    }, m[1]);
    checked++;
    if (error) {
      failed++;
      console.error(`${file}:${line}: Mermaid parse error\n${error}\n`);
    }
  }
  // An unclosed fence swallows the rest of the file and shows up as a parse
  // error above; also flag odd fence counts explicitly.
  const fences = (text.match(/^```/gm) || []).length;
  if (fences % 2 !== 0) {
    failed++;
    console.error(`${file}: unbalanced code fences (${fences} fence lines)`);
  }
}
await browser.close();
console.log(`Mermaid: ${checked} diagram(s) checked, ${failed} problem(s).`);
process.exit(failed ? 1 : 0);
