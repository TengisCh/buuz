# 0008: One URL for each page language

Status: Accepted
Date: 2026-11-19

## Context

The site switched language with JavaScript and `localStorage`, on one URL.
Google reads the page without a saved choice, so it saw only Mongolian.
Google says to use one URL for each language and to link them with
`hreflang`. It says not to switch language by cookie or browser setting.

## Decision

- Each page language has its own URL: `/` (Mongolian), `/en/` and `/ru/`.
- `npm run build` writes `dist/en/index.html` and `dist/ru/index.html` from
  the Mongolian page (`web/site/scripts/prerender.mjs`). Each file has its
  own text, title, description, canonical link, `og:` tags and FAQ data.
- Every page has `hreflang` links to all languages and an `x-default` link
  to `/`. The sitemap lists all three with the same links.
- The language button moves to the other URL with `history.pushState`. The
  URL decides the language. Nothing redirects by guessing the language.
- A page language needs full text before it gets a URL. A partly translated
  page hurts search results. The scheme picker covers the other languages.

## Consequences

- A new page language costs one string table in `i18n.ts` and one entry in
  `PATHS`, the sitemap and `vercel.json`.
- The privacy and cookie pages have no Russian text. Russian visitors see
  English there.
- Ukrainian, Belarusian and other pages need a native speaker to check the
  text first.
