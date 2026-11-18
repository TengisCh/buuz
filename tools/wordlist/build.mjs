#!/usr/bin/env node
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors.
//
// Builds data/wordlist/mn.txt from a Mongolian Wikipedia dump.
//
//   bzcat mnwiki-20261101-pages-articles.xml.bz2 | node tools/wordlist/build.mjs > data/wordlist/mn.txt
//
// Reads the decompressed XML from stdin line by line, keeps the text of
// namespace 0 pages (no redirects), strips wiki markup roughly, and counts
// Mongolian Cyrillic words. See tools/wordlist/README.md.

import { createInterface } from "node:readline";
import { pathToFileURL } from "node:url";

// Input contract: like real dumps, the XML has one tag per line
// (`<page>`, `<ns>`, `<redirect`, `<text ...>`), and page text may span lines.

export const SPDX = "# SPDX-License-Identifier: CC-BY-SA-4.0";
export const HEADER =
  "# Mongolian word frequencies from Mongolian Wikipedia (mnwiki-20261101), CC BY-SA 4.0. See data/wordlist/README.md.";
export const MIN_COUNT = 3;
export const LIMIT = 50000;
const MIN_LEN = 2;
const MAX_LEN = 30;
// An unclosed span ends at the next blank line or after this many characters.
const MAX_SPAN = 64 * 1024;

// Link targets in these namespaces are dropped with their caption.
const DROP_LINK = /^\[\[\s*(?:file|image|media|category|файл|зураг|медиа|ангилал)\s*:/;
// Tags whose content is dropped (the rest of the tags lose only the tag).
const DROP_TAG = /<(ref|math|gallery|source|syntaxhighlight|score|timeline|imagemap|nowiki|pre|code|references)\b[^>]{0,1000}>/g;

/** End (exclusive) of an unclosed span that starts at `from`: next blank line or the cap. */
function unclosedEnd(s, from) {
  const window = s.slice(from, from + MAX_SPAN);
  const blank = window.indexOf("\n\n");
  return from + (blank >= 0 ? blank : window.length);
}

/** Remove balanced `open ... close` spans (nested ones included). */
function stripBalanced(s, open, close, keep) {
  if (!s.includes(open)) return s;
  let out = "";
  let i = 0;
  while (i < s.length) {
    const at = s.indexOf(open, i);
    if (at < 0) break;
    out += s.slice(i, at);
    const limit = Math.min(s.length, at + MAX_SPAN);
    let depth = 0;
    let end = -1;
    for (let j = at; j < limit; j++) {
      if (s.startsWith(open, j)) {
        depth++;
        j += open.length - 1;
      } else if (s.startsWith(close, j)) {
        depth--;
        j += close.length - 1;
        if (depth === 0) {
          end = j + 1;
          break;
        }
      }
    }
    const closed = end >= 0;
    if (!closed) end = unclosedEnd(s, at); // an unclosed span is dropped
    if (closed && keep && keep(s.slice(at, end))) out += s.slice(at, end);
    i = Math.max(end, at + open.length);
  }
  return out + s.slice(i);
}

/** Remove `<!-- ... -->` comments. */
function stripComments(s) {
  if (!s.includes("<!--")) return s;
  let out = "";
  let i = 0;
  for (;;) {
    const at = s.indexOf("<!--", i);
    if (at < 0) break;
    out += s.slice(i, at) + " ";
    const window = s.slice(at + 4, at + 4 + MAX_SPAN);
    const close = window.indexOf("-->");
    i = close >= 0 ? at + 4 + close + 3 : unclosedEnd(s, at + 4);
  }
  return out + s.slice(i);
}

/** Remove tags that carry content (refs, math, ...) with that content; other tags are dropped later. */
function stripTagContent(s) {
  let out = "";
  let i = 0;
  DROP_TAG.lastIndex = 0;
  for (let m; (m = DROP_TAG.exec(s)); ) {
    if (m.index < i) continue;
    out += s.slice(i, m.index) + " ";
    i = m.index + m[0].length;
    if (m[0].endsWith("/>")) continue;
    const window = s.slice(i, i + MAX_SPAN);
    const close = window.indexOf(`</${m[1]}`);
    if (close >= 0) {
      const gt = window.indexOf(">", close);
      i = gt >= 0 ? i + gt + 1 : i + close;
    } else {
      i = unclosedEnd(s, i);
    }
    DROP_TAG.lastIndex = i;
  }
  return out + s.slice(i);
}

/** Strip wiki markup from one page's text. Rough on purpose. Expects lower-case text. */
export function cleanText(raw) {
  let s = raw
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&amp;/g, "&");
  s = stripComments(s);
  s = stripTagContent(s);
  s = stripBalanced(s, "{{", "}}");
  s = stripBalanced(s, "{|", "|}");
  // Files and categories first (captions hold nested links), then plain links.
  s = stripBalanced(s, "[[", "]]", (span) => !DROP_LINK.test(span));
  s = s.replace(/\[\[([^\]|]*)\|([^\]]*)\]\]/g, "$2").replace(/\[\[([^\]]*)\]\]/g, "$1");
  s = s.replace(/https?:\/\/\S+/g, " ").replace(/www\.\S+/g, " ");
  s = s.replace(/^=+.*=+[ \t]*$/gm, " "); // heading lines
  s = s.replace(/'{2,}/g, ""); // bold and italic marks
  s = s.replace(/<[^>]*>/g, " ");
  s = s.replace(/&(?:#\d+|#x[0-9a-f]+|[a-z][a-z0-9]*);/g, " ");
  return s;
}

// Split on whitespace and punctuation, but keep hyphens and apostrophes inside a token.
const TOKEN_SEPARATOR = /[^\p{L}\p{M}\p{N}\-'’]+/u;
const PART_SEPARATOR = /[-'’]+/;
const MONGOLIAN_WORD = /^[а-яёөү]+$/;

function addWords(text, counts) {
  for (const token of cleanText(text.toLowerCase()).split(TOKEN_SEPARATOR)) {
    for (const part of token.split(PART_SEPARATOR)) {
      if (part === "") continue;
      // `2010-нд`, `obama-гийн`: a part that is not Cyrillic takes the rest of
      // the token with it, because what follows is only its suffix.
      if (!MONGOLIAN_WORD.test(part)) break;
      if (part.length < MIN_LEN || part.length > MAX_LEN) continue;
      counts.set(part, (counts.get(part) ?? 0) + 1);
    }
  }
}

/** Count words in the text of namespace 0 pages. `lines` is any (async) iterable of lines. */
export async function countWords(lines) {
  const counts = new Map();
  let ns = null;
  let redirect = false;
  let inText = false;
  let buf = [];
  const finish = () => {
    if (ns === "0" && !redirect) addWords(buf.join("\n"), counts);
    buf = [];
    inText = false;
  };
  for await (const line of lines) {
    if (inText) {
      const end = line.indexOf("</text>");
      if (end >= 0) {
        buf.push(line.slice(0, end));
        finish();
      } else {
        buf.push(line);
      }
      continue;
    }
    const t = line.trimStart();
    if (t.startsWith("<page>")) {
      ns = null;
      redirect = false;
    } else if (t.startsWith("<ns>")) {
      ns = t.slice(4, t.indexOf("</ns>"));
    } else if (t.startsWith("<redirect")) {
      redirect = true;
    } else if (t.startsWith("<text")) {
      if (t.slice(0, t.indexOf(">") + 1).endsWith("/>")) continue; // empty text
      const start = t.indexOf(">") + 1;
      const rest = t.slice(start);
      const end = rest.indexOf("</text>");
      if (end >= 0) {
        buf = [rest.slice(0, end)];
        finish();
      } else {
        buf = [rest];
        inText = true;
      }
    }
  }
  return counts;
}

// Common Russian function words. Russian text sits inside Mongolian pages
// (names, quotes). Mongolian look-alikes (за, их, да, он) are left out on purpose.
const RUSSIAN_STOP = new Set(
  (
    "на при по или для не из от его что это как но все был была было были она они бы же " +
    "также этот эта эти того году года после между только который которые которая которое " +
    "быть есть может будет под без над уже еще ещё её"
  ).split(" "),
);
const RUSSIAN_SUFFIX = /(?:ского|ский|ская|ское|ские|ство)$/;

/**
 * Drop Russian function words and typical Russian word endings.
 * `removed` counts the words that would have passed `minCount`.
 */
export function filterRussian(counts, minCount = MIN_COUNT) {
  const out = new Map();
  const removed = { stoplist: 0, suffix: 0 };
  for (const [word, n] of counts) {
    if (RUSSIAN_STOP.has(word)) {
      if (n >= minCount) removed.stoplist++;
    } else if (RUSSIAN_SUFFIX.test(word)) {
      if (n >= minCount) removed.suffix++;
    } else {
      out.set(word, n);
    }
  }
  return { counts: out, removed };
}

/**
 * Words seen at least `minCount` times, best first (ties in code-point order).
 * The cut falls on a count boundary: all words with count >= N, where N is the
 * smallest count for which the total is at most `limit`.
 */
export function rank(counts, { minCount = MIN_COUNT, limit = LIMIT } = {}) {
  const entries = [];
  for (const [word, n] of counts) if (n >= minCount) entries.push([word, n]);
  // Only Cyrillic BMP letters survive, so `<` equals code-point order.
  entries.sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
  if (entries.length <= limit) return entries;
  const last = entries[limit - 1]?.[1] ?? Infinity;
  const next = entries[limit][1];
  const boundary = next === last ? last + 1 : last;
  return entries.filter(([, n]) => n >= boundary);
}

export function render(ranked) {
  return SPDX + "\n" + HEADER + "\n" + ranked.map(([w, n]) => `${w}\t${n}\n`).join("");
}

async function main() {
  const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
  const all = await countWords(rl);
  const { counts, removed } = filterRussian(all);
  const ranked = rank(counts);
  process.stderr.write(
    `unique tokens ${all.size}, removed by stoplist ${removed.stoplist}, ` +
      `by Russian suffix ${removed.suffix}, kept ${ranked.length}\n`,
  );
  process.stdout.write(render(ranked));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
