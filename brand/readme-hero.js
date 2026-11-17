// SPDX-License-Identifier: MIT OR Apache-2.0
// Render the README hero (docs/readme/hero.png): icon, "Buuz 2.0" and the subtitle on a
// transparent background. The colours are chosen to read on both GitHub themes
// (>= 3:1 on #ffffff and on #0d1117), so one image serves light and dark mode.
//   NODE_PATH=tools/lint/node_modules node brand/readme-hero.js
// Needs Playwright (and network for the Nunito font from Google Fonts).
// The committed PNG is this render downscaled to 1040px wide (`sips -Z 1040`, 2x of the
// README's width="520"), so a fresh render is 1846px wide and should be downscaled the same way.
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const icon = 'data:image/svg+xml;base64,' +
  fs.readFileSync(path.join(__dirname, 'icons', 'buuz-buddy-icon.svg')).toString('base64');
const out = path.join(__dirname, '..', 'docs', 'readme', 'hero.png');

// Ember (#FF6A3D) is only 2.85:1 on white, so the text uses slightly deeper tints of it.
const WORDMARK = '#E8542A'; // 3.66:1 on #fff, 5.17:1 on #0d1117
const SUP = '#F25C2E';      // 3.31:1 on #fff, 5.72:1 on #0d1117
const SUBTITLE = '#3D7BFF'; // Khadag Blue, lightened: 3.84:1 on #fff, 4.93:1 on #0d1117

const html = `<!doctype html><html><head><meta charset="utf-8">
<link href="https://fonts.googleapis.com/css2?family=Nunito:wght@900&display=block" rel="stylesheet">
<style>body{margin:0;background:transparent}
.hero{display:inline-flex;flex-direction:column;align-items:center;padding:24px 40px 28px}
.lock{display:flex;align-items:center;gap:40px}
.lock img{width:220px;height:220px}
.wm{font:900 200px/1 Nunito,sans-serif;letter-spacing:-.02em;color:${WORDMARK}}
.wm sup{font-size:72px;color:${SUP};margin-left:10px;vertical-align:120px}
.sub{font:900 54px/1 Nunito,sans-serif;color:${SUBTITLE};margin-top:40px;white-space:pre}</style></head>
<body><div class="hero"><div class="lock"><img src="${icon}"><span class="wm">Buuz<sup>2.0</sup></span></div>
<div class="sub">sain baina uu  →  сайн байна уу</div></div></body></html>`;

(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 1400, height: 700 }, deviceScaleFactor: 2 });
  await p.setContent(html, { waitUntil: 'networkidle' });
  await p.evaluate(() => document.fonts.ready);
  await p.locator('.hero').screenshot({ path: out, omitBackground: true });
  await b.close();
})();
