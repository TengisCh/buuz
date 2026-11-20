// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// The "help improve Buuz" page (/help, issue #101). It shows a few Cyrillic
// prompts, takes how the visitor types them in Latin, and sends the batch to
// /api/samples only when they press Send. Nothing is converted or stored here.

const COUNT = 10;

type Prompt = { id: number; text: string };

const MSG = {
  mn: {
    noConsent: "☝️ Эхлээд зөвшөөрлийн нүдийг чагтлаарай.",
    empty: "✍️ Ядаж нэг өгүүлбэр бичээрэй.",
    badText: "🔤 Зөвхөн латин үсэг, тоо, энгийн цэг таслал ашиглаарай.",
    sending: "⏳ Илгээж байна…",
    sent: "🎉 Баярлалаа! Таны жишээ нэмэгдлээ.",
    view: "Харах",
    limited: "⏳ Түр хүлээгээд дахин оролдоорой.",
    failed: "😕 Илгээж чадсангүй. Дараа дахин оролдоорой.",
    loadFailed: "😕 Өгүүлбэрүүдийг ачаалж чадсангүй. Хуудсаа дахин ачаална уу.",
    placeholder: "Латинаар бичих…",
  },
  en: {
    noConsent: "☝️ Please tick the consent box first.",
    empty: "✍️ Please type at least one sentence.",
    badText: "🔤 Please use only Latin letters, digits and plain punctuation.",
    sending: "⏳ Sending…",
    sent: "🎉 Thank you! Your samples were added.",
    view: "View",
    limited: "⏳ Please wait a while and try again.",
    failed: "😕 Could not send. Please try again later.",
    loadFailed: "😕 Could not load the sentences. Please reload the page.",
    placeholder: "Type in Latin…",
  },
};

function msg(): (typeof MSG)["mn"] {
  return document.documentElement.lang === "mn" ? MSG.mn : MSG.en;
}

const form = document.getElementById("samples") as HTMLFormElement;
const list = document.getElementById("prompts") as HTMLOListElement;
const note = document.getElementById("status") as HTMLParagraphElement;
const send = document.getElementById("send") as HTMLButtonElement;
const more = document.getElementById("more") as HTMLButtonElement;
const consent = document.getElementById("consent") as HTMLInputElement;
let all: Prompt[] = [];

function say(text: string, link?: string): void {
  note.textContent = text;
  if (link) {
    const a = document.createElement("a");
    a.href = link;
    a.target = "_blank";
    a.rel = "noopener";
    a.textContent = ` ${msg().view} ↗`;
    note.append(a);
  }
}

function pick(): Prompt[] {
  const p = [...all];
  for (let i = p.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [p[i], p[j]] = [p[j], p[i]];
  }
  return p.slice(0, COUNT);
}

function render(): void {
  list.replaceChildren();
  for (const p of pick()) {
    const li = document.createElement("li");
    const label = document.createElement("label");
    label.htmlFor = `p${p.id}`;
    label.lang = "mn";
    label.className = "prompt";
    label.textContent = p.text;
    const input = document.createElement("input");
    input.id = `p${p.id}`;
    input.dataset.id = String(p.id);
    input.type = "text";
    input.maxLength = 200;
    input.placeholder = msg().placeholder;
    input.setAttribute("autocomplete", "off");
    input.setAttribute("autocorrect", "off");
    input.setAttribute("autocapitalize", "off");
    input.spellcheck = false;
    li.append(label, input);
    list.append(li);
  }
  send.disabled = false;
  more.hidden = true;
}

async function load(): Promise<void> {
  try {
    const res = await fetch("/api/samples");
    all = ((await res.json()) as { prompts: Prompt[] }).prompts;
    document.getElementById("loading")?.remove();
    render();
  } catch {
    say(msg().loadFailed);
  }
}

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  const samples = [...list.querySelectorAll<HTMLInputElement>("input[data-id]")]
    .map((i) => ({ id: Number(i.dataset.id), latin: i.value.trim() }))
    .filter((s) => s.latin !== "");
  if (!samples.length) return say(msg().empty);
  if (!consent.checked) return say(msg().noConsent);
  const device = (form.querySelector<HTMLInputElement>('input[name="device"]:checked')?.value) ?? "";
  const website = (form.querySelector<HTMLInputElement>('input[name="website"]')?.value) ?? "";
  send.disabled = true;
  say(msg().sending);
  try {
    const res = await fetch("/api/samples", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ samples, device, consent: true, website }),
    });
    const body = (await res.json().catch(() => ({}))) as { url?: string; error?: string };
    if (res.ok) {
      say(msg().sent, body.url);
      more.hidden = false;
      return;
    }
    say(res.status === 429 ? msg().limited : body.error === "bad_text" ? msg().badText : msg().failed);
  } catch {
    say(msg().failed);
  }
  send.disabled = false;
});

more.addEventListener("click", () => {
  render();
  note.textContent = "";
  list.querySelector("input")?.focus();
});

// The placeholders follow the page language when the visitor switches it.
document.getElementById("lang")?.addEventListener("click", () => {
  for (const i of list.querySelectorAll("input")) i.placeholder = msg().placeholder;
});

void load();
