// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

export type PasteResult = { kind: "text"; text: string } | { kind: "empty" } | { kind: "blocked" };

/** Read the clipboard. Never throws: a refusal (permission, insecure page, old browser) gives "blocked". */
export async function readPaste(clip: { readText?: () => Promise<string> } | undefined): Promise<PasteResult> {
  if (!clip?.readText) return { kind: "blocked" };
  try {
    const text = await clip.readText();
    return text ? { kind: "text", text } : { kind: "empty" };
  } catch {
    return { kind: "blocked" };
  }
}
