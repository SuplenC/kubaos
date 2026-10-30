// Content extremes: the shortest and longest words and a translation 40% longer side by side,
// whatever the toolbar's length, then a missing picture, and none, one and many.

import { translated } from "../../lib/content.ts";
import type { Content } from "../page.ts";
import { el, mockCell } from "../dom.ts";
import { bandedGrid } from "./bands.ts";
import { partOr, subheading } from "./section.ts";
import { variants } from "./variants.ts";

export function content(spec: Content): HTMLElement {
  const text = partOr(spec.text, (rows) =>
    bandedGrid(
      ["Shortest", "Longest", "Translated, 40% longer"],
      rows.map((row) => ({
        name: row.name,
        cells: [mockCell(row.sample(row.words.short)), mockCell(row.sample(row.words.long)), mockCell(row.sample(translated(row.words.long)))],
      })),
    ),
  );
  return el(
    "div",
    "page-part",
    subheading("Text"),
    text,
    subheading("A missing icon or picture"),
    partOr(spec.missingPicture, variants),
    subheading("None, one and many"),
    partOr(spec.counts, variants),
  );
}
