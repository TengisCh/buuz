// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors
import test from "node:test";
import assert from "node:assert/strict";

// Transpile-free: Node 22.6+ can strip types with --experimental-strip-types.
const { POST } = await import("../api/feedback.ts");

const req = (body, headers = {}) =>
  new Request("https://buuz.test/api/feedback", {
    method: "POST",
    headers: { host: "buuz.test", origin: "https://buuz.test", ...headers },
    body: JSON.stringify(body),
  });

test("rejects short messages", async () => {
  const r = await POST(req({ message: "hi" }));
  assert.equal(r.status, 400);
});
test("honeypot pretends success without calling GitHub", async () => {
  let called = false;
  globalThis.fetch = async () => ((called = true), new Response("{}"));
  const r = await POST(req({ message: "hello there", website: "x" }));
  assert.equal(r.status, 200);
  assert.equal(called, false);
});
test("503 until the token exists", async () => {
  delete process.env.GITHUB_FEEDBACK_TOKEN;
  const r = await POST(req({ message: "hello there" }));
  assert.equal(r.status, 503);
});
test("cross-origin is refused", async () => {
  const r = await POST(req({ message: "hello there" }, { origin: "https://evil.test" }));
  assert.equal(r.status, 403);
});
test("creates a labeled issue, defangs mentions, rate limits", async () => {
  process.env.GITHUB_FEEDBACK_TOKEN = "secret-token";
  let seen;
  globalThis.fetch = async (url, init) => (
    (seen = { url, init }),
    new Response(JSON.stringify({ html_url: "https://github.com/TengisCh/buuz/issues/42", number: 42 }), { status: 201 })
  );
  const h = { "x-forwarded-for": "9.9.9.9" };
  for (let i = 0; i < 5; i++) {
    const r = await POST(req({ message: "hello @octocat\nline2", category: "bug", contact: "me@x.mn" }, h));
    assert.equal(r.status, 200);
    if (i === 0) assert.deepEqual(await r.json(), { ok: true, url: "https://github.com/TengisCh/buuz/issues/42", number: 42 });
  }
  const payload = JSON.parse(seen.init.body);
  assert.deepEqual(payload.labels, ["feedback", "bug"]);
  assert.ok(!payload.body.includes("@octocat"));
  assert.match(seen.url, /TengisCh\/buuz\/issues$/);
  const r6 = await POST(req({ message: "hello again" }, h));
  assert.equal(r6.status, 429);
  assert.ok(!(await r6.text()).includes("secret-token"));
});
