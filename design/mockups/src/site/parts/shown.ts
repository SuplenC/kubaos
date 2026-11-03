// A foundation's parts: each shown thing, its name and its words, as a shelf's group, drawn as wide
// as the page as a foundation's tables and grids are laid out for; and what holds the family to its
// rules.

import type { Check, NonEmpty, Shown } from "../page.ts";
import type { Group } from "./shelf.ts";
import { el, label, prose } from "../dom.ts";

export function shownAll(parts: readonly Shown[]): Group {
  return { title: "", items: parts.map((part) => ({ name: part.name, words: part.described, sample: part.sample(), fluid: true })) };
}

export function checks(list: NonEmpty<Check>): HTMLElement {
  return el(
    "dl",
    "page-list",
    ...list.flatMap((check) => [el("dt", "", label(check.check, "body_strong", "on_surface")), el("dd", "", prose(check.holds))]),
  );
}
