// SPDX-License-Identifier: MIT OR Apache-2.0
// Render every mockup screen to PNG, plus one overview sheet per OS.
//   node design/desktop-onboarding/render.js
const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

const here = __dirname;
const out = path.join(here, 'previews');
fs.mkdirSync(out, { recursive: true });
const screens = ['welcome', 'privacy', 'enable', 'enable-done', 'shortcut', 'try', 'harmony', 'import', 'done', 'home', 'typing', 'menu'];
const url = (q) => 'file://' + path.join(here, 'index.html') + '?' + q;

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 860, height: 620 }, deviceScaleFactor: 2 });
  for (const os of ['mac', 'win']) {
    for (const s of screens) {
      await page.goto(url(`os=${os}&lang=mn&screen=${s}`));
      await page.evaluate(() => document.fonts.ready);
      await page.waitForTimeout(150);
      await page.locator('.stage').screenshot({ path: path.join(out, `${os}-${s}.png`) });
    }
  }
  // overview sheets: 3 columns, scaled down
  const sheet = await browser.newPage({ viewport: { width: 2700, height: 1000 }, deviceScaleFactor: 1 });
  for (const [os, lang] of [['mac', 'mn'], ['win', 'mn'], ['mac', 'en']]) {
    await sheet.goto(url(`os=${os}&lang=${lang}`));
    await sheet.addStyleTag({ content: '.gallery{width:2700px}' });
    await sheet.evaluate(() => document.fonts.ready);
    await sheet.waitForTimeout(300);
    await sheet.screenshot({ path: path.join(out, `overview-${os}-${lang}.png`), fullPage: true });
  }
  await browser.close();
})();
