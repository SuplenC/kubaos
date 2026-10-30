// The words a mocked element shows, at the length the toolbar asks for: the shortest, the longest,
// or a translation that runs 40% longer than the longest.

import { settings } from "../site/settings.ts";

/** One thing said, as short as it goes and as long as it gets in English. */
export interface Words {
  readonly short: string;
  readonly long: string;
}

/** The same words at every length: a name, or what a sample shows on purpose. */
export function fixed(words: string): Words {
  return { short: words, long: words };
}

/** How much longer than English a translation runs, as the mockup skill shows it. */
const TRANSLATION = 1.4;

const ACCENTED: Record<string, string> = {
  a: "á", e: "é", i: "í", o: "ö", u: "ü", c: "ç", n: "ñ", s: "š", y: "ý", z: "ž",
  A: "Á", E: "É", I: "Í", O: "Ö", U: "Ü", C: "Ç", N: "Ñ", S: "Š", Y: "Ý", Z: "Ž",
};

/**
 * `english` as a stand-in translation: its letters accented, so the text is shaped as a
 * translation's is, and its words run on until it is 40% longer.
 */
export function translated(english: string): string {
  const accent = (text: string) => [...text].map((ch) => ACCENTED[ch] ?? ch).join("");
  const words = english.split(" ");
  let out = accent(english);
  for (let i = 0; [...out].length < english.length * TRANSLATION; i++) {
    out += ` ${accent(words[i % words.length] ?? "")}`;
  }
  return out;
}

/** `words` at the length the toolbar asks for. */
export function shown(words: Words): string {
  switch (settings().text) {
    case "short":
      return words.short;
    case "long":
      return words.long;
    case "translated":
      return translated(words.long);
  }
}
