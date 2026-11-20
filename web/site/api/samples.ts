// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// Vercel Function for the "help improve Buuz" page (/help, issue #101).
// GET gives the Cyrillic prompts. POST takes how a person typed some of them
// in Latin and posts the batch as a public GitHub issue labelled
// `typing-samples`. It uses the same token as the feedback form
// (GITHUB_FEEDBACK_TOKEN). It keeps no name, email or IP address: the IP is
// held in memory for at most one hour for the rate limit, like feedback.

const REPO = "TengisCh/buuz";
const LABELS = ["typing-samples"];

/** Everyday chat sentences. Never change or reuse an id: old samples point to it. */
export const PROMPTS: Record<number, string> = {
  1: "Сайн уу, юу байна?",
  2: "Өнөөдөр их хүйтэн байна, өвөл ирлээ.",
  3: "Өчигдөр бороо орсон уу?",
  4: "Чи хаана байгаа юм бэ?",
  5: "Би одоо гэртээ харьж байна.",
  6: "Ээж минь хоол хийж байна.",
  7: "Хүүхдүүд гадаа тоглож байна.",
  8: "Маргааш хэдэн цагт уулзах вэ?",
  9: "Есөн цагт уулзъя, болох уу?",
  10: "Ёстой гоё кино байсан шүү.",
  11: "Мэдэхгүй ээ, бодоод үзье.",
  12: "Баярлалаа, дараа ярья.",
  13: "Зүгээр дээ, санаа зовох хэрэггүй.",
  14: "Өглөөний цайгаа уусан уу?",
  15: "Дэлгүүрээс талх, сүү авчраарай.",
  16: "Мөнгө байхгүй ш дээ.",
  17: "Чиний утасны дугаар хэд вэ?",
  18: "Ах нь их сургуульд сурдаг.",
  19: "Үнэхээр сонирхолтой ном байна.",
  20: "Ажлаасаа гараад залгаарай, хүлээж байя.",
  21: "Би одоо ажил дээрээ байна.",
  22: "Хоол идсэн үү?",
  23: "Өнөө орой кино үзэх үү?",
  24: "Чамд баярлалаа, их тус боллоо.",
  25: "Энэ долоо хоногт завгүй байна.",
  26: "Хаана уулзах вэ, төв талбай дээр үү?",
  27: "Ээжийнхээ төрсөн өдөрт бэлэг авна.",
  28: "Өрөөндөө ороод унтлаа.",
  29: "Үнэхээр ядарч байна, амрах хэрэгтэй.",
  30: "Түрүүн залгасан, утсаа аваагүй.",
  31: "Чи юу хийж байгаа юм?",
  32: "Хүүхдүүд сургуульдаа явсан.",
  33: "Үдээс хойш цас орно гэсэн.",
  34: "Ямар үнэтэй юм бэ?",
  35: "Гэрийн даалгавраа хийчихсэн үү?",
  36: "Шинэ оны мэнд хүргэе!",
  37: "Өвөө эмээ хоёр хөдөө байгаа.",
  38: "Битгий санаа зов, бүх юм зүгээр.",
  39: "Ойлголоо, удахгүй хариу бичье.",
  40: "Хөөе, хаачсан бэ?",
};

const MAX_SAMPLES = 20;
const MAX_LATIN = 200;
// Letters (also ö ü), digits, space and plain punctuation. No @, backtick, |
// or < >, so the issue body needs no escaping.
const LATIN_OK = /^[A-Za-z0-9öüÖÜ .,!?'"\-():;]+$/;
const DEVICES = new Set(["phone", "computer"]);

// Best-effort rate limit, like the feedback form.
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

function json(body: Record<string, unknown>, status: number, cache = "no-store"): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json", "cache-control": cache },
  });
}

type Sample = { id: number; latin: string };

/** The valid samples of a request body, or a short error code. */
export function parseSamples(data: Record<string, unknown>): Sample[] | string {
  if (!Array.isArray(data.samples)) return "invalid";
  if (data.samples.length > MAX_SAMPLES) return "too_many";
  const seen = new Set<number>();
  const out: Sample[] = [];
  for (const raw of data.samples) {
    if (typeof raw !== "object" || raw === null) return "invalid";
    const { id, latin } = raw as Record<string, unknown>;
    if (typeof id !== "number" || !(id in PROMPTS) || seen.has(id)) return "invalid";
    if (typeof latin !== "string") return "invalid";
    const text = latin.replace(/\s+/g, " ").trim();
    if (text === "") continue;
    if (text.length > MAX_LATIN || !LATIN_OK.test(text)) return "bad_text";
    seen.add(id);
    out.push({ id, latin: text });
  }
  return out.length ? out : "empty";
}

/** The issue title and body for a batch. */
export function issueFor(samples: Sample[], device: string): { title: string; body: string } {
  const where = DEVICES.has(device) ? device : "not given";
  const rows = samples.map((s) => `${s.id}\t${PROMPTS[s.id]}\t${s.latin}`);
  const body = [
    "Typing samples sent from https://buuz.thetengis.com/help.",
    "",
    "```tsv",
    "id\tcyrillic\tlatin",
    ...rows,
    "```",
    "",
    `Device: ${where}`,
    "_The sender agreed to share these samples under CC0. No name, email or IP address is kept._",
  ].join("\n");
  const n = samples.length;
  return { title: `[Typing samples] ${n} sentence${n === 1 ? "" : "s"}`, body };
}

export function GET(): Response {
  const prompts = Object.entries(PROMPTS).map(([id, text]) => ({ id: Number(id), text }));
  return json({ prompts }, 200, "public, max-age=3600");
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
  if (typeof data.website === "string" && data.website.trim() !== "") return json({ ok: true }, 200);

  const samples = parseSamples(data);
  if (typeof samples === "string") return json({ error: samples }, 400);
  if (data.consent !== true) return json({ error: "no_consent" }, 400);
  const device = typeof data.device === "string" ? data.device : "";

  const token = process.env.GITHUB_FEEDBACK_TOKEN;
  if (!token) return json({ error: "not_configured" }, 503);

  const ip =
    (request.headers.get("x-forwarded-for") ?? "").split(",")[0].trim() ||
    request.headers.get("x-real-ip") ||
    "unknown";
  if (limited(ip)) return json({ error: "rate_limited" }, 429);

  const { title, body } = issueFor(samples, device);
  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}/issues`, {
      method: "POST",
      headers: {
        authorization: `Bearer ${token}`,
        accept: "application/vnd.github+json",
        "x-github-api-version": "2022-11-28",
        "user-agent": "buuz-typing-samples",
        "content-type": "application/json",
      },
      body: JSON.stringify({ title, body, labels: LABELS }),
    });
    if (!res.ok) {
      // Status only: never log the token or the response headers.
      console.error("samples: GitHub responded", res.status);
      return json({ error: "upstream" }, 502);
    }
    const issue = (await res.json().catch(() => ({}))) as { html_url?: unknown };
    const url =
      typeof issue.html_url === "string" && issue.html_url.startsWith(`https://github.com/${REPO}/issues/`)
        ? issue.html_url
        : undefined;
    return json({ ok: true, url }, 200);
  } catch {
    console.error("samples: GitHub request failed");
    return json({ error: "upstream" }, 502);
  }
}
