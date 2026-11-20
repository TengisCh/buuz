// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";

const { GET, POST, PROMPTS } = await import("../api/samples.ts");

const req = (body, headers = {}) =>
  new Request("https://buuz.test/api/samples", {
    method: "POST",
    headers: { host: "buuz.test", origin: "https://buuz.test", ...headers },
    body: JSON.stringify(body),
  });
const good = { samples: [{ id: 1, latin: "snu yu bn" }, { id: 3, latin: " uchigdur  boro orsnu? " }, { id: 4, latin: "" }], device: "phone", consent: true };

test("GET lists the prompts", async () => {
  const r = GET();
  const { prompts } = await r.json();
  assert.equal(prompts.length, Object.keys(PROMPTS).length);
  assert.deepEqual(prompts[0], { id: 1, text: "Сайн уу, юу байна?" });
});
test("refuses bad input", async () => {
  process.env.GITHUB_FEEDBACK_TOKEN = "secret-token";
  for (const [body, code] of [
    [{ ...good, samples: [{ id: 999, latin: "x" }] }, "invalid"],
    [{ ...good, samples: [{ id: 1, latin: "hi @octocat" }] }, "bad_text"],
    [{ ...good, samples: [{ id: 1, latin: "a" }, { id: 1, latin: "b" }] }, "invalid"],
    [{ ...good, samples: [{ id: 1, latin: "  " }] }, "empty"],
    [{ ...good, consent: false }, "no_consent"],
  ]) {
    const r = await POST(req(body));
    assert.equal(r.status, 400);
    assert.equal((await r.json()).error, code);
  }
});
test("cross-origin is refused", async () => {
  const r = await POST(req(good, { origin: "https://evil.test" }));
  assert.equal(r.status, 403);
});
test("honeypot pretends success without calling GitHub", async () => {
  let called = false;
  globalThis.fetch = async () => ((called = true), new Response("{}"));
  const r = await POST(req({ ...good, website: "x" }));
  assert.equal(r.status, 200);
  assert.equal(called, false);
});
test("posts a labelled issue with a TSV block, then rate limits", async () => {
  process.env.GITHUB_FEEDBACK_TOKEN = "secret-token";
  let seen;
  globalThis.fetch = async (url, init) => (
    (seen = { url, init }),
    new Response(JSON.stringify({ html_url: "https://github.com/TengisCh/buuz/issues/7" }), { status: 201 })
  );
  const h = { "x-forwarded-for": "8.8.8.8" };
  const r = await POST(req(good, h));
  assert.deepEqual(await r.json(), { ok: true, url: "https://github.com/TengisCh/buuz/issues/7" });
  const payload = JSON.parse(seen.init.body);
  assert.deepEqual(payload.labels, ["typing-samples"]);
  assert.equal(payload.title, "[Typing samples] 2 sentences");
  assert.ok(payload.body.includes("1\tСайн уу, юу байна?\tsnu yu bn"));
  assert.ok(payload.body.includes("3\tӨчигдөр бороо орсон уу?\tuchigdur boro orsnu?"));
  assert.ok(payload.body.includes("Device: phone"));
  assert.ok(!payload.body.includes("8.8.8.8"));
  for (let i = 0; i < 4; i++) await POST(req(good, h));
  const r6 = await POST(req(good, h));
  assert.equal(r6.status, 429);
});
