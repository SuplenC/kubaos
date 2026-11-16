// Content extremes: the shortest and longest words and a translation 40% longer side by side,
// whatever the toolbar's length, then a missing picture, and none, one and many.

import { translated } from "../../lib/content.ts";
import { applies, type Content, type DoesntApply, drawsScreen } from "../page.ts";
import { mockCell } from "../dom.ts";
import { bandedGrid } from "./bands.ts";
import { type Group, type Item, sampleOf } from "./shelf.ts";
import { variants } from "./variants.ts";

const LENGTHS = ["Shortest", "Longest", "Translated, 40% longer"] as const;

/** One text sample at one length, undrawn: its name, its place on a switcher, how to draw it. */
export interface TextSample {
  readonly name: string;
  readonly at: readonly [string, string];
  readonly make: () => Element;
  readonly screen: boolean;
}

/** Every row of `text` at each length, in the order the content's grid shows them. */
export function textSamples(text: Exclude<Content["text"], DoesntApply>): TextSample[] {
  return text.flatMap((row) =>
    [row.words.short, row.words.long, translated(row.words.long)].map((words, i): TextSample => ({
      name: `${row.name} · ${LENGTHS[i]!}`,
      at: [row.name, LENGTHS[i]!],
      make: () => row.sample(words),
      screen: drawsScreen(row.sample),
    })),
  );
}

/** The content's groups for a shelf: the text at each length, a missing picture, the counts. */
export function content(spec: Content): Group[] {
  const text: Group = {
    title: "Text",
    items: applies(spec.text)
      ? textSamples(spec.text).map(
          (each): Item => ({ name: each.name, sample: each.screen ? each.make : each.make(), make: each.make, at: each.at }),
        )
      : spec.text,
    small: (items) =>
      bandedGrid(
        [...LENGTHS],
        applies(spec.text) ? spec.text.map((row, r) => ({ name: row.name, cells: [0, 1, 2].map((i) => mockCell(sampleOf(items[r * 3 + i]!))) })) : [],
      ),
  };
  return [text, variants("A missing icon or picture", spec.missingPicture), variants("None, one and many", spec.counts)];
}
