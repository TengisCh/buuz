// SPDX-License-Identifier: MIT OR Apache-2.0
// Export the final Buuz Buddy icon and the "Buuz 2.0" wordmark as PNG, JPEG and WebP.
//   python3 brand/generate.py brand/icons && node brand/export.js && python3 brand/webp.py
// Needs Playwright (and network for the Nunito font from Google Fonts).
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const dir = path.join(__dirname, 'icons');
const out = path.join(__dirname, 'export');
fs.mkdirSync(out, { recursive: true });
const svg = (f) => 'data:image/svg+xml;base64,' + fs.readFileSync(path.join(dir, f)).toString('base64');
const icon = svg('buuz-buddy-icon.svg');
const small = svg('buuz-buddy-small.svg');

const page1 = (src, size, bg) =>
  `<body style="margin:0;background:${bg}"><img src="${src}" width="${size}" height="${size}" style="display:block"></body>`;

const wordmark = (ink, bg) => `<!doctype html><html><head><meta charset="utf-8">
<link href="https://fonts.googleapis.com/css2?family=Nunito:wght@900&display=block" rel="stylesheet">
<style>body{margin:0;background:${bg}}
.lock{display:inline-flex;align-items:center;gap:40px;padding:48px 64px}
.lock img{width:220px;height:220px}
.wm{font:900 200px/1 Nunito,sans-serif;letter-spacing:-.02em;color:${ink}}
.wm sup{font-size:72px;color:#FF6A3D;margin-left:10px;vertical-align:120px}</style></head>
<body><div class="lock"><img src="${icon}"><span class="wm">Buuz<sup>2.0</sup></span></div></body></html>`;

(async () => {
  const b = await chromium.launch();
  const p = await b.newPage();
  const shot = async (html, size, file, opts) => {
    await p.setViewportSize({ width: size, height: size });
    await p.setContent(html);
    await p.screenshot({ path: path.join(out, file), clip: { x: 0, y: 0, width: size, height: size }, ...opts });
  };
  for (const s of [16, 32, 48, 64, 128, 256, 512, 1024]) {
    await shot(page1(s <= 48 ? small : icon, s, 'transparent'), s, `buuz-buddy-${s}.png`, { omitBackground: true });
  }
  for (const s of [512, 1024]) {
    await shot(page1(icon, s, '#fff'), s, `buuz-buddy-${s}.jpg`, { type: 'jpeg', quality: 95 });
  }
  // Wordmark lockups: transparent PNG (dark ink and light ink), white JPEG.
  await p.setViewportSize({ width: 1400, height: 400 });
  for (const [file, ink, bg, opts] of [
    ['buuz-wordmark.png', '#1E1838', 'transparent', { omitBackground: true }],
    ['buuz-wordmark-light.png', '#FFFFFF', 'transparent', { omitBackground: true }],
    ['buuz-wordmark.jpg', '#1E1838', '#fff', { type: 'jpeg', quality: 95 }],
  ]) {
    await p.setContent(wordmark(ink, bg), { waitUntil: 'networkidle' });
    await p.evaluate(() => document.fonts.ready);
    await p.locator('.lock').screenshot({ path: path.join(out, file), ...opts });
  }
  await b.close();
})();
