// SPDX-License-Identifier: MIT OR Apache-2.0
// Render a preview sheet of every icon option at real sizes.
//   node brand/render.js brand/icons brand/previews
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const [iconDir, outDir] = process.argv.slice(2);
fs.mkdirSync(outDir, { recursive: true });
const opts = [
  ['a-buuz-buddy', 'A. Buuz Buddy', 'The mascot. A real buuz: spiral pleats, open steam hole, a happy face.'],
  ['b-keycap', 'B. Keycap', 'The product. A buuz on a key with both legends, B and Б.'],
  ['c-rosette', 'C. Rosette', 'The mark. A buuz from above: spiral pleats around the steam hole.'],
  ['d-steam-b', 'D. Steam Б', 'The magic. Steam from the buuz curls into a Cyrillic Б.'],
];
const read = (f) => 'data:image/svg+xml;base64,' + fs.readFileSync(path.join(iconDir, f)).toString('base64');

const card = ([slug, name, line]) => {
  const big = read(`${slug}-icon.svg`), sm = read(`${slug}-small.svg`);
  return `<section><div class="hero"><img src="${big}" width="300" height="300" class="ico"></div>
  <div class="meta"><h2>${name}</h2><p>${line}</p>
  <div class="row">
    <figure><img src="${big}" width="128" class="ico"><figcaption>128</figcaption></figure>
    <figure><img src="${big}" width="64" class="ico"><figcaption>64</figcaption></figure>
    <figure><img src="${sm}" width="32"><figcaption>32</figcaption></figure>
    <figure><img src="${sm}" width="16"><figcaption>16</figcaption></figure>
    <figure class="dark"><img src="${sm}" width="16"> <img src="${sm}" width="22"><figcaption>tray (dark)</figcaption></figure>
    <figure class="tab"><span><img src="${sm}" width="16"> buuz</span><figcaption>browser tab</figcaption></figure>
  </div></div></section>`;
};

const html = `<!doctype html><html><head><meta charset="utf-8">
<link href="https://fonts.googleapis.com/css2?family=Nunito:wght@600;800;900&display=block" rel="stylesheet">
<style>
body{margin:0;padding:48px;background:#FFF7EC;font-family:Nunito,Inter,sans-serif;color:#24190F;width:1904px}
h1{font-size:56px;font-weight:900;margin:0 0 6px}
.sub{font-size:22px;color:#7A6455;margin:0 0 36px}
.grid{display:grid;grid-template-columns:1fr 1fr;gap:28px}
section{background:#fff;border-radius:36px;padding:32px;display:flex;gap:32px;box-shadow:0 6px 0 #F0DFC8}
.ico{filter:drop-shadow(0 10px 18px rgba(60,30,10,.25));border-radius:22%}
h2{font-size:34px;font-weight:900;margin:8px 0 4px}
p{font-size:20px;color:#7A6455;margin:0 0 22px;max-width:520px}
.row{display:flex;gap:22px;align-items:flex-end;flex-wrap:wrap}
figure{margin:0;display:flex;flex-direction:column;align-items:center;gap:8px}
figcaption{font-size:14px;color:#9A8475}
.dark{background:#1E1E22;padding:10px 14px;border-radius:10px;flex-direction:row;align-items:center}
.dark figcaption{color:#aaa}
.tab span{display:flex;align-items:center;gap:8px;background:#ECEFF3;padding:8px 14px;border-radius:10px 10px 0 0;font:600 14px Inter,sans-serif;color:#333}
img{image-rendering:auto}
</style></head><body>
<h1>Buuz 2.0 · icon options v2</h1>
<p class="sub">Derived from what buuz means (family, Tsagaan Sar, the pleats and the steam hole) and what the app does (Latin keys in, Cyrillic out).</p>
<div class="grid">${opts.map(card).join('')}</div></body></html>`;

const sw = (hex, name, use, dark) => `<div class="sw"><div style="background:${hex};${dark ? 'color:#fff' : ''}">${hex}</div><b>${name}</b><span>${use}</span></div>`;
const mascot = read('a-buuz-buddy-icon.svg');
const brandHtml = `<!doctype html><html><head><meta charset="utf-8">
<link href="https://fonts.googleapis.com/css2?family=Nunito:wght@600;700;800;900&family=JetBrains+Mono:wght@500&display=block" rel="stylesheet">
<style>
body{margin:0;padding:48px;background:#FFF7EC;font-family:Nunito,sans-serif;color:#1E1838;width:1904px}
h1{font-size:56px;font-weight:900;margin:0 0 30px} h3{font-size:16px;letter-spacing:.12em;text-transform:uppercase;color:#9A8475;margin:0 0 16px}
.grid{display:grid;grid-template-columns:1fr 1fr;gap:28px}
.card{background:#fff;border-radius:36px;padding:32px;box-shadow:0 6px 0 #F0DFC8}
.lock{display:flex;align-items:center;gap:22px;margin:0 0 22px}
.lock img{width:88px;height:88px;border-radius:22%}
.wm{font-weight:900;font-size:76px;letter-spacing:-.02em;line-height:1}
.wm small{font-size:30px;color:#FF6A3D;margin-left:8px;vertical-align:super}
.mn{font-weight:800;font-size:40px;color:#2E6BFF;margin-left:6px}
.label{font-size:15px;color:#9A8475;margin:-12px 0 26px 110px}
.swatches{display:grid;grid-template-columns:repeat(4,1fr);gap:16px}
.sw div{height:96px;border-radius:20px;display:flex;align-items:flex-end;padding:10px 14px;font:500 14px 'JetBrains Mono';box-shadow:inset 0 0 0 1px rgba(0,0,0,.06)}
.sw b{display:block;margin-top:8px;font-size:18px} .sw span{font-size:14px;color:#7A6455}
.type p{margin:0 0 10px}
.ui{background:#FFF3E0;border-radius:30px;padding:28px}
.in,.out{border-radius:24px;padding:20px 24px;font-size:30px;font-weight:700}
.in{background:#fff;box-shadow:0 5px 0 #EBD7BC;color:#7A6455;margin-bottom:14px}
.out{background:linear-gradient(160deg,#FFB648,#FF6A3D);color:#fff;box-shadow:0 6px 0 #D9502B;display:flex;justify-content:space-between;align-items:center}
.btn{background:#2E6BFF;color:#fff;font-size:20px;font-weight:800;padding:12px 22px;border-radius:999px;box-shadow:0 5px 0 #1638B8}
.chips{display:flex;gap:10px;margin-top:16px} .chip{background:#fff;border-radius:999px;padding:8px 14px;font:500 15px 'JetBrains Mono';box-shadow:0 3px 0 #EBD7BC}
.bar{display:flex;align-items:center;gap:12px;margin-bottom:18px;font-weight:900;font-size:28px}.bar img{width:44px;border-radius:22%}
</style></head><body>
<h1>Buuz 2.0 · palette, type and feel</h1>
<div class="grid">
<div class="card"><h3>Wordmark options (pick one)</h3>
<div class="lock"><img src="${mascot}"><span class="wm">buuz</span></div><div class="label">1 · lowercase, friendly. Version lives in copy, not the logo.</div>
<div class="lock"><img src="${mascot}"><span class="wm">Buuz<small>2.0</small></span></div><div class="label">2 · says "continuation" loudly. Ages when there's a 3.0.</div>
<div class="lock"><img src="${mascot}"><span class="wm">buuz</span><span class="mn">бууз</span></div><div class="label">3 · bilingual. The app's whole job in one lockup: Latin in, Cyrillic out.</div>
</div>
<div class="card"><h3>Web converter feel</h3>
<div class="ui"><div class="bar"><img src="${mascot}">buuz <span class="mn" style="font-size:24px">бууз</span></div>
<div class="in">sain baina uu? margaash uulzah uu</div>
<div class="out"><span>сайн байна уу? маргааш уулзах уу</span><span class="btn">Copy</span></div>
<div class="chips"><span class="chip">q → ө</span><span class="chip">w → ү</span><span class="chip">h → х</span><span class="chip">ts → ц</span><span class="chip">sh → ш</span></div></div>
<p style="font-size:17px;color:#7A6455;margin:18px 4px 0">Cream canvas, chunky "pressed" cards, one orange hero (the Cyrillic), one blue action. The mascot puffs steam each time text converts and does a small hop on Copy.</p></div>
<div class="card"><h3>Palette</h3><div class="swatches">
${sw('#FFF3E0','Dough','page and cards')}${sw('#FF6A3D','Ember','brand, hero, mascot tile',1)}${sw('#FFB648','Amber','gradients, warm accents')}${sw('#2E6BFF','Khadag Blue','primary action, links',1)}
${sw('#1E1838','Night','text, dark mode bg',1)}${sw('#8C4A2B','Filling','tiny accents, the hole',1)}${sw('#FF8E8E','Blush','cheeks, delight')}${sw('#FFFFFF','Steam','highlights, motion')}
</div></div>
<div class="card type"><h3>Type</h3>
<p style="font-weight:900;font-size:64px;line-height:1">Сайн уу, найзаа!</p>
<p style="font-weight:900;font-size:40px">Өглөөний мэнд · Үнэхээр гоё</p>
<p style="font-weight:700;font-size:22px;color:#7A6455">Nunito 900 for headlines, 700 for UI. Rounded, warm, full Mongolian Cyrillic (Ө ө Ү ү). Open source (OFL).</p>
<p style="font:500 20px 'JetBrains Mono';color:#7A6455">JetBrains Mono for key hints: q → ө · w → ү · h → х</p></div>
</div></body></html>`;

(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 2000, height: 1000 }, deviceScaleFactor: Number(process.env.DSF || 1) });
  await p.setContent(html, { waitUntil: 'networkidle' });
  await p.evaluate(() => document.fonts.ready);
  await p.screenshot({ path: path.join(outDir, 'options-sheet.png'), fullPage: true });
  await p.setContent(brandHtml, { waitUntil: 'networkidle' });
  await p.evaluate(() => document.fonts.ready);
  await p.screenshot({ path: path.join(outDir, 'brand-sheet.png'), fullPage: true });
  // single large PNGs per option
  for (const [slug] of opts) {
    await p.setContent(`<body style="margin:0;background:transparent"><img src="${read(slug + '-icon.svg')}" width="1024"></body>`);
    await p.setViewportSize({ width: 1024, height: 1024 });
    await p.screenshot({ path: path.join(outDir, `${slug}-1024.png`), omitBackground: true, clip: { x: 0, y: 0, width: 1024, height: 1024 } });
  }
  await b.close();
})();
