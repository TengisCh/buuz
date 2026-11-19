// SPDX-License-Identifier: MIT OR Apache-2.0
// Pure logic of the strings editor. No DOM here, so node can test it.

const LETTER = /^[A-Za-z'"öüÖÜ]$/;
const PLACEHOLDER = /\{[A-Za-z0-9_.]+\}/g;

/** True when `pos` is inside an open `{...}` in `text`. */
function insideBrace(text, pos) {
  const open = text.lastIndexOf("{", pos - 1);
  return open !== -1 && text.lastIndexOf("}", pos - 1) < open;
}

/** The sorted, unique placeholders of a text, such as `{step}`. */
function placeholders(text) {
  return [...new Set((text || "").match(PLACEHOLDER) || [])].sort();
}

function samePlaceholders(a, b) {
  const x = placeholders(a);
  const y = placeholders(b);
  return x.length === y.length && x.every((p, i) => p === y[i]);
}

/**
 * Live Latin to Cyrillic typing for one text field.
 * `convert` is the Buuz converter (string to string).
 * The typer keeps the Latin letters of the word that is being typed, so
 * "s" then "h" gives "ш" and not "сх". Call `handle` from `beforeinput`.
 */
function createTyper(convert) {
  let pending = null; // { start, raw, end }: the word in the field
  return {
    reset() {
      pending = null;
    },
    /**
     * st: { value, start, end } is the field before the edit.
     * ev: { inputType, data }.
     * Returns null to let the browser edit, or { value, caret } to replace it.
     */
    handle(st, ev) {
      if (pending && !(st.start === st.end && st.end === pending.end)) pending = null;
      const pd = pending;
      let ch = null;
      if (ev.inputType === "insertText" && ev.data && ev.data.length === 1) ch = ev.data;
      else if (ev.inputType === "insertLineBreak") ch = "\n";

      if (ch !== null) {
        if (insideBrace(st.value, st.start)) {
          pending = null;
          return null;
        }
        if (ch.charCodeAt(0) > 127 && !LETTER.test(ch)) {
          pending = null;
          return null;
        }
        let value = st.value;
        let pos = st.start;
        let word = pd;
        let replaced = false;
        if (st.start !== st.end) {
          value = value.slice(0, st.start) + value.slice(st.end);
          word = null;
          replaced = true;
        }
        const from = word ? word.start : pos;
        const rawBefore = word ? word.raw : "";
        const tail = value.slice(pos);
        if (LETTER.test(ch)) {
          const raw = rawBefore + ch;
          const out = convert(raw);
          pending = { start: from, raw, end: from + out.length };
          return { value: value.slice(0, from) + out + tail, caret: from + out.length };
        }
        const out = convert(rawBefore + ch);
        pending = null;
        if (!word && !replaced && out === ch) return null;
        return { value: value.slice(0, from) + out + tail, caret: from + out.length };
      }

      if (ev.inputType === "deleteContentBackward" && pd && st.start === st.end) {
        const raw = pd.raw.slice(0, -1);
        const out = raw ? convert(raw) : "";
        pending = raw ? { start: pd.start, raw, end: pd.start + out.length } : null;
        return {
          value: st.value.slice(0, pd.start) + out + st.value.slice(st.end),
          caret: pd.start + out.length,
        };
      }
      pending = null;
      return null;
    },
  };
}

/** The strings model: originals, current values, order. */
function createModel(mn, en) {
  const order = [...Object.keys(mn)];
  for (const k of Object.keys(en)) if (!(k in mn)) order.push(k);
  const orig = { mn: { ...mn }, en: { ...en } };
  const cur = { mn: { ...mn }, en: { ...en } };
  return {
    order,
    orig,
    cur,
    value(lang, key) {
      return key in cur[lang] ? cur[lang][key] : "";
    },
    isChanged(lang, key) {
      return (cur[lang][key] ?? "") !== (orig[lang][key] ?? "") || key in cur[lang] !== key in orig[lang];
    },
    isMissing(key) {
      return !this.value("mn", key).trim() || !this.value("en", key).trim();
    },
    placeholderMismatch(key) {
      return !samePlaceholders(this.value("mn", key), this.value("en", key));
    },
    set(lang, key, text) {
      cur[lang][key] = text;
    },
    revert(key) {
      for (const l of ["mn", "en"]) {
        if (key in orig[l]) cur[l][key] = orig[l][key];
        else delete cur[l][key];
      }
    },
    /** Saved edits from an older visit. Unknown keys are ignored. */
    restore(saved) {
      for (const l of ["mn", "en"]) {
        const s = saved && saved[l];
        if (!s || typeof s !== "object") continue;
        for (const k of this.order) if (typeof s[k] === "string") cur[l][k] = s[k];
      }
    },
    /** JSON text, same key order, 2 spaces, trailing newline. */
    toJson(lang) {
      const o = {};
      for (const k of this.order) {
        if (k in cur[lang] && (k in orig[lang] || cur[lang][k] !== "")) o[k] = cur[lang][k];
      }
      return JSON.stringify(o, null, 2) + "\n";
    },
    changesList() {
      const lines = [];
      for (const k of this.order) {
        for (const l of ["mn", "en"]) {
          if (!this.isChanged(l, k)) continue;
          lines.push(`${k} [${l}]`);
          lines.push(`  before: ${JSON.stringify(orig[l][k] ?? "")}`);
          lines.push(`  after:  ${JSON.stringify(cur[l][k] ?? "")}`);
        }
      }
      return lines.join("\n") + (lines.length ? "\n" : "");
    },
  };
}

/**
 * Clean the alternatives data: { key: [text, ...] }. Anything that is not a
 * list of non-empty strings is dropped. Missing or broken data gives {}.
 */
function normalizeAlternatives(data) {
  const out = {};
  if (!data || typeof data !== "object" || Array.isArray(data)) return out;
  for (const [k, v] of Object.entries(data)) {
    if (!Array.isArray(v)) continue;
    const seen = new Set();
    const list = v.filter((t) => typeof t === "string" && t.trim() !== "" && !seen.has(t) && seen.add(t));
    if (list.length) out[k] = list;
  }
  return out;
}

/**
 * The rows of the quick-pick menu: the original value first, then the
 * alternatives that differ from it. `chosen` marks the row equal to `current`.
 * Returns [] when there are no alternatives.
 */
function alternativeItems(original, alts, current) {
  const list = (alts || []).filter((t) => t !== original);
  if (!list.length) return [];
  return [{ text: original, original: true }, ...list.map((text) => ({ text, original: false }))].map((it) => ({
    ...it,
    chosen: it.text === current,
  }));
}

/** Number of alternatives shown on the button. */
function alternativeCount(original, alts) {
  return (alts || []).filter((t) => t !== original).length;
}

/** Move the active row in a menu of `n` rows. Wraps. `dir` is 1 or -1. */
function moveActive(active, n, dir) {
  if (n <= 0) return -1;
  if (active < 0) return dir > 0 ? 0 : n - 1;
  return (active + dir + n) % n;
}

/** Pick row `i`: the cell gets its text. Returns null for a bad index. */
function pickAlternative(model, key, items, i) {
  if (!items[i]) return null;
  model.set("mn", key, items[i].text);
  return items[i].text;
}

if (typeof module !== "undefined") {
  module.exports = {
    insideBrace,
    placeholders,
    samePlaceholders,
    createTyper,
    createModel,
    normalizeAlternatives,
    alternativeItems,
    alternativeCount,
    moveActive,
    pickAlternative,
  };
}
