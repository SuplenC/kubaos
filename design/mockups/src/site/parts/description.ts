// The description block: how the part looks and behaves, in words, beside the live thing, or
// under it where a text column doesn't fit beside it.

import type { Sample } from "../page.ts";
import { el, prose, sampleBox } from "../dom.ts";

export function description(live: Sample, paragraphs: readonly string[]): HTMLElement {
  return el(
    "div",
    "page-description",
    sampleBox(el("div", "page-live page-view", live())),
    el("div", "page-description-words", ...paragraphs.map((p) => prose(p))),
  );
}
