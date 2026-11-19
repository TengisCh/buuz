// SPDX-License-Identifier: MIT OR Apache-2.0
// Render the link-preview images, one per page language: web/site/public/og/<lang>.png
// (1200x630). The tagline comes from the built page, so run the site build first.
//   cd web/site && npm run build
//   NODE_PATH=tools/lint/node_modules node brand/og-images.js
// Needs Playwright (and network for the Nunito font from Google Fonts).
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..', 'web', 'site');
const icon = 'data:image/svg+xml;base64,' +
  fs.readFileSync(path.join(__dirname, 'icons', 'buuz-buddy-icon.svg')).toString('base64');

// [Latin input, Cyrillic output, "Free · Open source"]. Languages other than
// Mongolian, English and Russian are drafts: a native speaker must check them.
const PAGES = {
  mn: ['sain baina uu', 'сайн байна уу', 'Үнэгүй · Нээлттэй эх'],
  en: ['sain baina uu', 'сайн байна уу', 'Free · Open source'],
  ru: ['privet', 'привет', 'Бесплатно · Открытый код'],
  uk: ['pryvit', 'привіт', 'Безкоштовно · Відкритий код'],
  be: ['pryvitanne', 'прывітанне', 'Бясплатна · Адкрыты код'],
  bg: ['zdravey', 'здравей', 'Безплатно · Отворен код'],
  sr: ['hvala', 'хвала', 'Бесплатно · Отворени код'],
  mk: ['zdravo', 'здраво', 'Бесплатно · Отворен код'],
  kk: ['saelem', 'сәлем', 'Тегін · Ашық бастапқы код'],
  ky: ['jakshy', 'жакшы', 'Акысыз · Ачык булак'],
};

function tagline(lang) {
  const file = lang === 'mn' ? 'index.html' : path.join(lang, 'index.html');
  const html = fs.readFileSync(path.join(root, 'dist', file), 'utf8');
  const m = html.match(/<p class="tag"[^>]*>([\s\S]*?)<\/p>/);
  if (!m) throw new Error(`no tagline in dist/${file}`);
  return m[1].replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>');
}

const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;');

function page(lang) {
  const [from, to, free] = PAGES[lang];
  return `<!doctype html><html lang="${lang}"><head><meta charset="utf-8">
<link href="https://fonts.googleapis.com/css2?family=Nunito:wght@800;900&display=block" rel="stylesheet">
<style>
*{box-sizing:border-box}body{margin:0;width:1200px;height:630px;position:relative;overflow:hidden;
background:radial-gradient(circle at 1010px 130px,#ffeeb3 0,#fff7dd 200px,#fffdf8 520px);font-family:Nunito,sans-serif}
.lock{position:absolute;left:70px;top:70px;display:flex;align-items:center;gap:36px}
.lock img{width:150px;height:150px;border-radius:34px;box-shadow:0 8px 24px #ff6a3d40}
.wm{font:900 150px/1 Nunito;letter-spacing:-.02em;color:#1e1838}
.wm sup{font-size:48px;color:#ff6a3d;margin-left:6px;vertical-align:80px}
.tag{position:absolute;left:70px;top:262px;width:1060px;font:900 38px/1.2 Nunito;color:#6b5b4e;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.in,.out{position:absolute;top:350px;height:130px;border-radius:30px;display:flex;align-items:center;padding:0 34px;font:900 44px/1 Nunito;color:#1e1838;white-space:nowrap}
.in{left:70px;width:480px;background:#fdf7ec;border:3px solid #f1e2c6;box-shadow:0 8px 0 #ecdcbc}
.out{left:650px;width:480px;background:linear-gradient(135deg,#ffd54a,#ffb347);box-shadow:0 8px 0 #e8a63a}
.arrow{position:absolute;left:550px;width:100px;top:350px;height:130px;display:flex;align-items:center;justify-content:center;font:900 44px/1 Nunito;color:#ff6a3d}
.url{position:absolute;left:70px;bottom:46px;font:800 26px/1 Nunito;color:#3d7bff}
.free{position:absolute;right:70px;bottom:46px;font:800 26px/1 Nunito;color:#6b5b4e}
</style></head><body>
<div class="lock"><img src="${icon}"><span class="wm">Buuz<sup>2.0</sup></span></div>
<div class="tag">${esc(tagline(lang))} ✨</div>
<div class="in">${esc(from)}</div><div class="arrow">→</div><div class="out">${esc(to)}</div>
<div class="url">buuz.thetengis.com</div><div class="free">🔒 ${esc(free)}</div>
</body></html>`;
}

(async () => {
  const outDir = path.join(root, 'public', 'og');
  fs.mkdirSync(outDir, { recursive: true });
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 1200, height: 630 }, deviceScaleFactor: 1 });
  for (const lang of Object.keys(PAGES)) {
    await p.setContent(page(lang), { waitUntil: 'networkidle' });
    await p.evaluate(() => document.fonts.ready);
    await p.screenshot({ path: path.join(outDir, `${lang}.png`) });
  }
  await b.close();
})();
