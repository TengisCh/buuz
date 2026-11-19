// SPDX-License-Identifier: MIT OR Apache-2.0
// Page code of the strings editor. The glue (wasm-bindgen), logic.js and
// the data are put in front of this file by build.mjs.

(function () {
  const $ = (id) => document.getElementById(id);
  const read = (id) => JSON.parse($(id).textContent);
  const b64 = $("wasm-b64").textContent.trim();
  const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
  initSync({ module: bytes });
  const config = default_config_text();
  try {
    load_word_list($("wordlist").textContent);
  } catch (e) {
    console.warn("word list not loaded", e);
  }
  const convert = (s) => convert_with_config(s, config);

  const model = createModel(read("data-mn"), read("data-en"));
  const STORE = "buuz.stringsEditor.v1";
  const settings = { typeMn: true, typeEn: false, showAlt: true, search: "", filter: "all" };
  let altData = {};
  try {
    altData = normalizeAlternatives(read("data-alt"));
  } catch (e) {
    console.warn("alternatives not loaded", e);
  }
  const cellMode = new Map(); // "lang:key" -> boolean, an override of the global toggle

  function load() {
    try {
      const saved = JSON.parse(localStorage.getItem(STORE) || "null");
      if (saved) {
        model.restore(saved);
        if (saved.settings) Object.assign(settings, saved.settings, { search: "", filter: "all" });
      }
    } catch (e) {
      /* storage blocked or bad data: start from the files */
    }
  }
  let saveTimer = 0;
  function save() {
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      try {
        const diff = { mn: {}, en: {} };
        for (const l of ["mn", "en"])
          for (const k of model.order) if (model.isChanged(l, k)) diff[l][k] = model.value(l, k);
        localStorage.setItem(STORE, JSON.stringify({ ...diff, settings: { typeMn: settings.typeMn, typeEn: settings.typeEn, showAlt: settings.showAlt } }));
        status("Saved in this browser.");
      } catch (e) {
        status("Could not save in this browser. Download your work.");
      }
    }, 200);
  }
  function status(text) {
    $("status").textContent = text;
  }

  const rows = new Map(); // key -> { tr, cells: {mn, en}, warn }
  function typing(lang, key) {
    const o = cellMode.get(lang + ":" + key);
    return o !== undefined ? o : lang === "mn" ? settings.typeMn : settings.typeEn;
  }

  function makeCell(lang, key) {
    const td = document.createElement("td");
    td.className = "cell " + lang;
    const ta = document.createElement("textarea");
    ta.value = model.value(lang, key);
    ta.rows = 1;
    ta.spellcheck = false;
    ta.lang = lang;
    ta.setAttribute("aria-label", `${key} (${lang === "mn" ? "Mongolian" : "English"})`);
    const typer = createTyper(convert);
    ta.addEventListener("beforeinput", (e) => {
      if (e.isComposing || !typing(lang, key)) {
        typer.reset();
        return;
      }
      const r = typer.handle(
        { value: ta.value, start: ta.selectionStart, end: ta.selectionEnd },
        { inputType: e.inputType, data: e.data },
      );
      if (!r) return;
      e.preventDefault();
      ta.value = r.value;
      ta.setSelectionRange(r.caret, r.caret);
      onEdit(lang, key, ta);
    });
    ta.addEventListener("input", () => onEdit(lang, key, ta));
    ta.addEventListener("blur", () => typer.reset());

    const tools = document.createElement("div");
    tools.className = "tools";
    const mode = document.createElement("button");
    mode.type = "button";
    mode.className = "chip mode";
    mode.addEventListener("click", () => {
      cellMode.set(lang + ":" + key, !typing(lang, key));
      typer.reset();
      refreshTools(lang, key);
      ta.focus();
    });
    const ph = document.createElement("span");
    ph.className = "phs";
    const alt = document.createElement("button");
    alt.type = "button";
    alt.className = "chip alt";
    alt.hidden = true;
    alt.setAttribute("aria-haspopup", "listbox");
    alt.title = "Choose another wording (Alt+Down in the cell)";
    alt.addEventListener("click", (e) => {
      e.stopPropagation();
      if (menu.open && menu.key === key) closeMenu(false);
      else openMenu(key, alt);
    });
    tools.append(mode, ph, alt);
    td.append(ta, tools);
    if (lang === "mn") {
      ta.addEventListener("keydown", (e) => {
        if (e.altKey && e.key === "ArrowDown" && !e.ctrlKey && !e.metaKey && hasAlternatives(key)) {
          e.preventDefault();
          openMenu(key, ta);
        }
      });
    }
    return { td, ta, mode, ph, typer, alt };
  }

  function refreshTools(lang, key) {
    const r = rows.get(key);
    const c = r.cells[lang];
    const on = typing(lang, key);
    if (lang === "mn") {
      const n = alternativeCount(model.orig.mn[key] ?? "", altData[key]);
      c.alt.hidden = !settings.showAlt || n === 0;
      c.alt.textContent = `✨ Alternatives (${n})`;
    }
    c.mode.textContent = on ? "Latin → Cyrillic: on" : "Latin → Cyrillic: off";
    c.mode.setAttribute("aria-pressed", String(on));
    c.mode.classList.toggle("on", on);
    // Placeholder buttons: the ones of the other language that this cell lacks.
    const other = lang === "mn" ? "en" : "mn";
    const have = placeholders(model.value(lang, key));
    const want = placeholders(model.value(other, key)).filter((p) => !have.includes(p));
    c.ph.replaceChildren(
      ...want.map((p) => {
        const b = document.createElement("button");
        b.type = "button";
        b.className = "chip ph";
        b.textContent = "+ " + p;
        b.title = "Insert " + p + " at the cursor";
        b.addEventListener("click", () => {
          c.ta.focus();
          const s = c.ta.selectionStart;
          c.ta.setRangeText(p, s, c.ta.selectionEnd, "end");
          c.typer.reset();
          onEdit(lang, key, c.ta);
        });
        return b;
      }),
    );
  }

  // The quick-pick menu: one layer on the page, so no table can clip it.
  const menu = { open: false, key: null, items: [], active: -1, anchor: null, el: null };
  const hasAlternatives = (key) => settings.showAlt && alternativeCount(model.orig.mn[key] ?? "", altData[key]) > 0;

  function renderMenu() {
    menu.el.replaceChildren(
      ...menu.items.map((it, i) => {
        const li = document.createElement("li");
        li.setAttribute("role", "option");
        li.setAttribute("aria-selected", String(it.chosen));
        li.classList.toggle("chosen", it.chosen);
        li.classList.toggle("active", i === menu.active);
        li.append(document.createTextNode(it.text));
        if (it.original) {
          const small = document.createElement("small");
          small.textContent = "Original";
          li.append(small);
        }
        li.addEventListener("mousemove", () => {
          if (menu.active !== i) {
            menu.active = i;
            for (const [j, c] of [...menu.el.children].entries()) c.classList.toggle("active", j === i);
          }
        });
        li.addEventListener("mousedown", (e) => e.preventDefault());
        li.addEventListener("click", (e) => {
          e.stopPropagation();
          choose(i);
        });
        return li;
      }),
    );
    const act = menu.el.children[menu.active];
    if (act) act.scrollIntoView({ block: "nearest" });
  }

  function placeMenu() {
    const r = menu.anchor.getBoundingClientRect();
    const el = menu.el;
    el.style.left = "0px";
    el.style.top = "0px";
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    let left = Math.min(Math.max(4, r.left), innerWidth - w - 4);
    let top = r.bottom + 4;
    if (top + h > innerHeight - 4 && r.top - h - 4 > 4) top = r.top - h - 4;
    el.style.left = Math.max(4, left) + "px";
    el.style.top = Math.max(4, top) + "px";
  }

  function openMenu(key, anchor) {
    closeMenu(false);
    const items = alternativeItems(model.orig.mn[key] ?? "", altData[key], model.value("mn", key));
    if (!items.length) return;
    if (!menu.el) {
      menu.el = document.createElement("ul");
      menu.el.id = "altmenu";
      menu.el.setAttribute("role", "listbox");
      menu.el.setAttribute("aria-label", "Alternative wordings");
      document.body.append(menu.el);
    }
    menu.open = true;
    menu.key = key;
    menu.items = items;
    menu.anchor = anchor;
    const chosen = items.findIndex((it) => it.chosen);
    menu.active = chosen >= 0 ? chosen : 0;
    menu.el.hidden = false;
    renderMenu();
    placeMenu();
  }

  function closeMenu(refocus) {
    if (!menu.open) return;
    menu.open = false;
    menu.el.hidden = true;
    const anchor = menu.anchor;
    menu.anchor = null;
    if (refocus && anchor) anchor.focus();
  }

  function choose(i) {
    const key = menu.key;
    const text = pickAlternative(model, key, menu.items, i);
    if (text === null) return;
    const c = rows.get(key).cells.mn;
    c.ta.value = text;
    c.typer.reset();
    closeMenu(false);
    c.ta.focus();
    onEdit("mn", key, c.ta);
  }

  document.addEventListener(
    "keydown",
    (e) => {
      if (!menu.open) return;
      const n = menu.items.length;
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        e.stopPropagation();
        menu.active = moveActive(menu.active, n, e.key === "ArrowDown" ? 1 : -1);
        renderMenu();
      } else if (e.key === "Enter") {
        e.preventDefault();
        e.stopPropagation();
        choose(menu.active);
      } else if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        closeMenu(true);
      } else if (e.key === "Tab") {
        closeMenu(false);
      }
    },
    true,
  );
  document.addEventListener("mousedown", (e) => {
    if (menu.open && !menu.el.contains(e.target) && !(e.target.closest && e.target.closest("button.alt"))) closeMenu(false);
  });
  addEventListener("resize", () => closeMenu(false));
  addEventListener("scroll", () => closeMenu(false));

  function refreshRow(key) {
    const r = rows.get(key);
    for (const l of ["mn", "en"]) {
      r.cells[l].td.classList.toggle("changed", model.isChanged(l, key));
      r.cells[l].td.classList.toggle("empty", !model.value(l, key).trim());
      refreshTools(l, key);
    }
    const bad = model.placeholderMismatch(key);
    r.warn.hidden = !bad;
    if (bad) {
      r.warn.textContent =
        "Placeholders differ. English: " +
        (placeholders(model.value("en", key)).join(" ") || "none") +
        ". Mongolian: " +
        (placeholders(model.value("mn", key)).join(" ") || "none") +
        ".";
    }
    const ch = model.isChanged("mn", key) || model.isChanged("en", key);
    r.revert.hidden = !ch;
    r.tr.dataset.changed = ch ? "1" : "";
  }

  function onEdit(lang, key, ta) {
    model.set(lang, key, ta.value);
    refreshRow(key);
    updateCounts();
    applyFilter();
    save();
  }

  function build() {
    const body = $("rows");
    let group = null;
    for (const key of model.order) {
      const prefix = key.split(".")[0];
      if (prefix !== group) {
        group = prefix;
        const h = document.createElement("tbody");
        h.className = "group";
        h.dataset.group = prefix;
        const tr = document.createElement("tr");
        const th = document.createElement("th");
        th.colSpan = 3;
        th.scope = "colgroup";
        th.textContent = prefix;
        tr.append(th);
        h.append(tr);
        body.append(h);
      }
      const tb = body.lastElementChild;
      const tr = document.createElement("tr");
      tr.dataset.key = key;
      tr.dataset.group = prefix;
      const k = document.createElement("td");
      k.className = "key";
      const code = document.createElement("code");
      code.textContent = key;
      const warn = document.createElement("div");
      warn.className = "warn";
      warn.hidden = true;
      const revert = document.createElement("button");
      revert.type = "button";
      revert.className = "chip revert";
      revert.textContent = "Revert";
      revert.hidden = true;
      revert.addEventListener("click", () => {
        model.revert(key);
        for (const l of ["mn", "en"]) {
          rows.get(key).cells[l].ta.value = model.value(l, key);
          rows.get(key).cells[l].typer.reset();
        }
        refreshRow(key);
        updateCounts();
        applyFilter();
        save();
      });
      k.append(code, warn, revert);
      const en = makeCell("en", key);
      const mn = makeCell("mn", key);
      tr.append(k, en.td, mn.td);
      tb.append(tr);
      rows.set(key, { tr, cells: { en, mn }, warn, revert });
    }
    for (const key of model.order) refreshRow(key);
  }

  function updateCounts() {
    let changed = 0;
    let missing = 0;
    let warn = 0;
    for (const k of model.order) {
      if (model.isChanged("mn", k) || model.isChanged("en", k)) changed++;
      if (model.isMissing(k)) missing++;
      if (model.placeholderMismatch(k)) warn++;
    }
    $("counts").textContent = `${model.order.length} keys, ${changed} changed, ${missing} missing, ${warn} placeholder warnings`;
  }

  function applyFilter() {
    const q = settings.search.trim().toLowerCase();
    const visibleGroups = new Set();
    for (const key of model.order) {
      const r = rows.get(key);
      let show = true;
      if (settings.filter === "changed") show = r.tr.dataset.changed === "1";
      else if (settings.filter === "missing") show = model.isMissing(key);
      else if (settings.filter === "warn") show = model.placeholderMismatch(key);
      if (show && q) {
        show = (key + "\n" + model.value("en", key) + "\n" + model.value("mn", key)).toLowerCase().includes(q);
      }
      r.tr.hidden = !show;
      if (show) visibleGroups.add(key.split(".")[0]);
    }
    for (const g of document.querySelectorAll("tbody.group")) g.hidden = !visibleGroups.has(g.dataset.group);
    $("empty").hidden = visibleGroups.size > 0;
  }

  function download(lang) {
    const blob = new Blob([model.toJson(lang)], { type: "application/json;charset=utf-8" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `strings.${lang}.json`;
    document.body.append(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  }

  async function copyChanges() {
    const text = model.changesList() || "No changes.\n";
    try {
      await navigator.clipboard.writeText(text);
    } catch (e) {
      const t = document.createElement("textarea");
      t.value = text;
      document.body.append(t);
      t.select();
      document.execCommand("copy");
      t.remove();
    }
    status("Copied the list of changes.");
  }

  const head = () => document.documentElement.style.setProperty("--head", document.querySelector("header").offsetHeight + "px");
  head();
  addEventListener("resize", head);
  load();
  $("typeMn").checked = settings.typeMn;
  $("typeEn").checked = settings.typeEn;
  $("showAlt").checked = settings.showAlt;
  build();
  updateCounts();
  $("typeMn").addEventListener("change", (e) => {
    settings.typeMn = e.target.checked;
    for (const k of model.order) refreshTools("mn", k);
    save();
  });
  $("typeEn").addEventListener("change", (e) => {
    settings.typeEn = e.target.checked;
    for (const k of model.order) refreshTools("en", k);
    save();
  });
  $("showAlt").addEventListener("change", (e) => {
    settings.showAlt = e.target.checked;
    closeMenu(false);
    for (const k of model.order) refreshTools("mn", k);
    save();
  });
  $("search").addEventListener("input", (e) => {
    settings.search = e.target.value;
    applyFilter();
  });
  $("filter").addEventListener("change", (e) => {
    settings.filter = e.target.value;
    applyFilter();
  });
  $("dl-mn").addEventListener("click", () => download("mn"));
  $("dl-en").addEventListener("click", () => download("en"));
  $("copy").addEventListener("click", copyChanges);
  $("reset").addEventListener("click", () => {
    if (!confirm("Remove all your edits and go back to the original strings?")) return;
    for (const k of model.order) {
      model.revert(k);
      for (const l of ["mn", "en"]) {
        rows.get(k).cells[l].ta.value = model.value(l, k);
        rows.get(k).cells[l].typer.reset();
      }
      refreshRow(k);
    }
    updateCounts();
    applyFilter();
    save();
  });
  applyFilter();
})();
