// Content extremes: the shortest and longest words and a translation 40% longer side by side,
// whatever the toolbar's length, then a missing picture, and none, one and many.

import { translated } from "../../lib/content.ts";
import { applies, type Content } from "../page.ts";
import { mockCell } from "../dom.ts";
import { bandedGrid } from "./bands.ts";
import type { Group, Item } from "./shelf.ts";
import { variants } from "./variants.ts";

const LENGTHS = ["Shortest", "Longest", "Translated, 40% longer"] as const;

/** The content's groups for a shelf: the text at each length, a missing picture, the counts. */
export function content(spec: Content): Group[] {
  const text: Group = {
    title: "Text",
    items: applies(spec.text)
      ? spec.text.flatMap((row) =>
          [row.words.short, row.words.long, translated(row.words.long)].map((words, i): Item => ({
            name: `${row.name} · ${LENGTHS[i]!}`,
            sample: row.sample(words),
            make: () => row.sample(words),
            at: [row.name, LENGTHS[i]!],
          })),
        )
      : spec.text,
    small: (items) =>
      bandedGrid(
        [...LENGTHS],
        applies(spec.text) ? spec.text.map((row, r) => ({ name: row.name, cells: [0, 1, 2].map((i) => mockCell(items[r * 3 + i]!.sample)) })) : [],
      ),
  };
  return [text, variants("A missing icon or picture", spec.missingPicture), variants("None, one and many", spec.counts)];
}
