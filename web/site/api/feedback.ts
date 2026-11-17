// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// Vercel Function: turns the site's feedback form into a GitHub issue in
// TengisCh/buuz. The token comes from the GITHUB_FEEDBACK_TOKEN environment
// variable (fine-grained PAT, Issues: read & write on this one repo) and is
// never logged or returned. Nothing from the converter box ever reaches this
// function: only what the person types into the feedback form.

const REPO = "TengisCh/buuz";
const CATEGORIES = new Set(["bug", "idea", "other"]);
const MIN_MESSAGE = 5;
const MAX_MESSAGE = 2000;
const MAX_CONTACT = 200;

// Best-effort rate limit. Memory is per function instance, so this stops casual
// floods, not a determined attacker; GitHub's own limits are the backstop.
const PER_IP_LIMIT = 5;
const GLOBAL_LIMIT = 60;
const WINDOW_MS = 60 * 60 * 1000;
const hits = new Map<string, number[]>();
let globalHits: number[] = [];

function recent(list: number[], now: number): number[] {
  return list.filter((t) => now - t < WINDOW_MS);
}

function limited(ip: string): boolean {
  const now = Date.now();
  globalHits = recent(globalHits, now);
  const mine = recent(hits.get(ip) ?? [], now);
  if (globalHits.length >= GLOBAL_LIMIT || mine.length >= PER_IP_LIMIT) {
    hits.set(ip, mine);
    return true;
  }
  mine.push(now);
  globalHits.push(now);
  hits.set(ip, mine);
  if (hits.size > 5000) hits.clear();
  return false;
}

function json(body: Record<string, unknown>, status: number): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json", "cache-control": "no-store" },
  });
}

// Issues are public: stop @mentions from pinging people.
function defang(s: string): string {
  return s.replace(/@/g, "@​");
}

function str(v: unknown, max: number): string {
  return typeof v === "string" ? v.trim().slice(0, max) : "";
}

export async function POST(request: Request): Promise<Response> {
  // Same-origin only.
  const origin = request.headers.get("origin");
  const host = request.headers.get("host");
  if (origin) {
    try {
      if (new URL(origin).host !== host) return json({ error: "forbidden" }, 403);
    } catch {
      return json({ error: "forbidden" }, 403);
    }
  }

  let data: Record<string, unknown>;
  try {
    data = (await request.json()) as Record<string, unknown>;
  } catch {
    return json({ error: "invalid" }, 400);
  }

  // Honeypot: real people never fill this hidden field. Pretend success.
  if (str(data.website, 100) !== "") return json({ ok: true }, 200);

  const message = str(data.message, MAX_MESSAGE + 1);
  if (message.length < MIN_MESSAGE) return json({ error: "too_short" }, 400);
  if (message.length > MAX_MESSAGE) return json({ error: "too_long" }, 400);
  const category = CATEGORIES.has(str(data.category, 10)) ? str(data.category, 10) : "other";
  const contact = str(data.contact, MAX_CONTACT);
  const lang = str(data.lang, 2) === "en" ? "en" : "mn";

  const token = process.env.GITHUB_FEEDBACK_TOKEN;
  if (!token) return json({ error: "not_configured" }, 503);

  const ip =
    (request.headers.get("x-forwarded-for") ?? "").split(",")[0].trim() ||
    request.headers.get("x-real-ip") ||
    "unknown";
  if (limited(ip)) return json({ error: "rate_limited" }, 429);

  const firstLine = message.split("\n")[0].replace(/\s+/g, " ");
  const title = `[Feedback] ${defang(firstLine).slice(0, 70)}`;
  const body = [
    defang(message),
    "",
    "---",
    `Category: ${category}`,
    `Contact (given by the sender, public): ${contact ? defang(contact) : "none"}`,
    `Site language: ${lang}`,
    "_Sent from the feedback form on the Buuz website._",
  ].join("\n");

  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}/issues`, {
      method: "POST",
      headers: {
        authorization: `Bearer ${token}`,
        accept: "application/vnd.github+json",
        "x-github-api-version": "2022-11-28",
        "user-agent": "buuz-feedback-form",
        "content-type": "application/json",
      },
      body: JSON.stringify({ title, body, labels: ["feedback"] }),
    });
    if (!res.ok) {
      // Status only: never log the token or the response headers.
      console.error("feedback: GitHub responded", res.status);
      return json({ error: "upstream" }, 502);
    }
    return json({ ok: true }, 200);
  } catch {
    console.error("feedback: GitHub request failed");
    return json({ error: "upstream" }, 502);
  }
}

export function GET(): Response {
  return json({ error: "method_not_allowed" }, 405);
}
