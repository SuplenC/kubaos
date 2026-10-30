// The description block: how the part looks and behaves, in words, beside the live thing.

import type { Sample } from "../page.ts";
import { el, prose } from "../dom.ts";

export function description(live: Sample, paragraphs: readonly string[]): HTMLElement {
  return el("div", "page-description", el("div", "page-live", live()), el("div", "page-description-words", ...paragraphs.map((p) => prose(p))));
}
