// SPDX-License-Identifier: MIT OR Apache-2.0
// Example phrases for the typewriter placeholder of each typing language. They are
// Latin text: the page converts them with the scheme of the page, so the example
// result always matches the rule that is on. Languages other than Mongolian,
// English and Russian are drafts for a native speaker to check.

const OTHER: Record<string, string[]> = {
  ru: ["privet, mir", "kak dela?", "spasibo bolshoe", "dobroe utro", "ya lyublyu buuz", "do svidaniya"],
  uk: ["pryvit, svite", "yak spravy?", "dyakuyu", "dobryi den", "do pobachennya", "ya lyublyu buuz"],
  be: ["pryvitanne", "dziakuj", "dobry dzien", "jak spravy?", "da pabachennia"],
  bg: ["zdravey, svyat", "kak si?", "blagodarya", "dobro utro", "do skoro"],
  sr: ["zdravo, svete", "hvala", "dobar dan", "kako si?", "prijatelj"],
  mk: ["zdravo, svete", "blagodaram", "dobar den", "kako si?", "prijatel"],
  kk: ["saelem, jaqsy", "rahmet", "qosh keldiniz", "qalaysyn?"],
  ky: ["salam, jakshy", "rahmat", "kandaysyn?", "kosh keliniz"],
};

/** The language of a scheme id: "ru-gost" is Russian, "mn-mns" is Mongolian. */
export const schemeLanguage = (scheme: string): string => scheme.split("-")[0];

/** The example phrases for a scheme, or null when the Mongolian ones apply. */
export function ghostPhrases(scheme: string): string[] | null {
  return OTHER[schemeLanguage(scheme)] ?? null;
}

/** The phrases in random order. */
export function shuffled(list: string[]): string[] {
  const out = [...list];
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}
