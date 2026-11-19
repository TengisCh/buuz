// SPDX-License-Identifier: MIT OR Apache-2.0
// Render the settings panel mockups to PNG: node design/settings-panel/render.js <out-dir>
const { chromium } = require("playwright");
const path = require("path");
const out = process.argv[2] || path.join(__dirname, "previews");
const page = "file://" + path.join(__dirname, "index.html");
(async () => {
  const b = await chromium.launch();
  for (const [v, w, h] of [["a", 390, 844], ["b", 390, 844], ["c", 1280, 820], ["docs", 1280, 820]]) {
    const p = await b.newPage({ viewport: { width: w, height: h }, deviceScaleFactor: 2 });
    await p.goto(`${page}?v=${v}`, { waitUntil: "networkidle" });
    await p.screenshot({ path: path.join(out, `${v}.png`) });
    await p.close();
  }
  await b.close();
})();
