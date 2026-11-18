// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// "Teach Buuz how you type" (issue #4), without the DOM so the node tests can
// run it. The user types a few Cyrillic sentences in Latin; the core aligns
// them and suggests `[keys]` entries; the accepted ones go into the config.
// The typed text lives only in a CalibrationFlow, in memory, and is dropped
// by `cancel()`, which the page calls whenever the flow closes. It is never written to storage or sent anywhere.

import { calibrate_apply, calibrate_json } from "./wasm/buuz.js";

/** The Mongolian Cyrillic alphabet, as the core lists it. */
export const ALPHABET = Array.from("абвгдеёжзийклмнопөрстуүфхцчшщъыьэюя");

export interface Suggestion {
  letter: string;
  chunk: string;
  count: number;
  default_count: number;
  tie: boolean;
  /** The letter the key gives today, which it will stop giving. */
  displaces: string | null;
  pair: boolean;
  /** The `[keys]` target, such as "ү" or "у/ү". */
  target: string;
}

export interface Conflict {
  letter: string;
  chunk: string;
  count: number;
  other: string;
  other_count: number;
}

export interface Unsupported {
  letter: string;
  chunk: string;
  count: number;
}

/** What the core found, with sentence numbers counted from 1 as the user sees them. */
export interface Outcome {
  suggestions: Suggestion[];
  conflicts: Conflict[];
  unsupported: Unsupported[];
  /** Sentences that were typed but could not be matched, so were skipped. */
  unmatchedSentences: number[];
  /** Sentences left empty. */
  notTyped: number[];
  /** Letters the sentences do not contain, so they were not tested. */
  skippedLetters: string[];
  /** Typed sentences that matched. With 0, there is nothing to learn from. */
  usable: number;
}

/**
 * The message for an error from `save`. The wasm module throws tagged strings
 * (a config problem), which `translate` turns into text (the page passes
 * `parseIssue` then `issueText`); a JS `Error` comes
 * from `CalibrationFlow` itself, and gets `fallback`.
 */
export function saveError(e: unknown, translate: (tagged: string) => string, fallback: string): string {
  return e instanceof Error ? fallback : translate(String(e));
}

/** The letters of the alphabet that the sentences contain. */
export function coverage(sentences: readonly string[]): string[] {
  const seen = new Set(Array.from(sentences.join(" ").toLowerCase()));
  return ALPHABET.filter((l) => seen.has(l));
}

/** The letters of the alphabet that the sentences do not contain. */
export function uncovered(sentences: readonly string[]): string[] {
  const have = new Set(coverage(sentences));
  return ALPHABET.filter((l) => !have.has(l));
}

export class CalibrationFlow {
  readonly sentences: readonly string[];
  private answers: string[];
  /** The `[cyrillic, typed]` pairs of the last `finish()`; cleared with the answers. */
  private pairsJson = "";
  private suggestionCount = 0;

  constructor(sentences: readonly string[]) {
    this.sentences = sentences;
    this.answers = sentences.map(() => "");
  }

  getAnswer(i: number): string {
    return this.answers[i] ?? "";
  }

  setAnswer(i: number, typed: string): void {
    if (i >= 0 && i < this.answers.length) this.answers[i] = typed;
  }

  /** How many sentences have typed text. */
  answerCount(): number {
    return this.answers.filter((a) => a.trim() !== "").length;
  }

  /** Align the typed text with the sentences and list what to suggest. */
  finish(): Outcome {
    if (this.sentences.length === 0 || this.answers.length === 0) throw new Error("calibration was cancelled");
    const pairs: [string, string][] = [];
    const numbers: number[] = []; // pair index -> sentence number from 1
    const notTyped: number[] = [];
    this.sentences.forEach((s, i) => {
      const typed = this.answers[i].trim();
      if (typed === "") notTyped.push(i + 1);
      else {
        pairs.push([s, typed]);
        numbers.push(i + 1);
      }
    });
    this.pairsJson = JSON.stringify(pairs);
    const raw = JSON.parse(calibrate_json(this.pairsJson));
    this.suggestionCount = raw.suggestions.length;
    return {
      suggestions: raw.suggestions,
      conflicts: raw.conflicts,
      unsupported: raw.unsupported,
      unmatchedSentences: (raw.unmatched as number[]).map((p) => numbers[p] ?? p + 1),
      notTyped,
      skippedLetters: raw.skipped_letters,
      usable: pairs.length - raw.unmatched.length,
    };
  }

  /**
   * The config text with the accepted suggestions (positions in the last
   * `finish()` result) added. Returns the new text; the caller makes it the
   * config. A bad position or a config that does not
   * parse throws and changes nothing.
   */
  save(configText: string, accepted: readonly number[]): string {
    if (this.pairsJson === "") throw new Error("finish() first");
    if (accepted.some((i) => !Number.isInteger(i) || i < 0 || i >= this.suggestionCount)) {
      throw new Error("bad suggestion index");
    }
    return calibrate_apply(configText, this.pairsJson, JSON.stringify(accepted));
  }

  /** Drop everything that was typed. The config is not touched. */
  cancel(): void {
    this.answers = [];
    this.pairsJson = "";
    this.suggestionCount = 0;
    (this as { sentences: readonly string[] }).sentences = [];
  }
}
