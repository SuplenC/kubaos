// A foundation's parts: each shown thing, its name and its note, as a shelf's group, drawn as wide
// as the page as a foundation's tables and grids are laid out for; and what holds the family to its
// rules, as a table.

import type { Check, NonEmpty, Shown } from "../page.ts";
import type { Group } from "./shelf.ts";
import { specTable } from "./table.ts";

export function shownAll(parts: readonly Shown[]): Group {
  return { title: "", items: parts.map((part) => ({ name: part.name, words: part.note, sample: part.sample(), fluid: true })) };
}

export function checks(list: NonEmpty<Check>): HTMLElement {
  return specTable({ heads: ["Check", "Holds"] }, list.map((check) => ({ cells: [/^[a-z]/.test(check.check) ? `\`${check.check}\`` : check.check, check.holds] }))).element;
}
