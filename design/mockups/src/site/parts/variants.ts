// Variants and sizes: each one live, its name, and what it is for, as a shelf's group.

import type { DoesntApply, Variant } from "../page.ts";
import { applies, drawsScreen } from "../page.ts";
import type { Group, Item } from "./shelf.ts";

/** `rows` as the items of a shelf, each drawn now, or a whole screen when it is first shown. */
export function variantItems(rows: readonly Variant[]): Item[] {
  return rows.map((row) => ({ name: row.name, words: row.for, sample: drawsScreen(row.sample) ? () => row.sample() : row.sample(), make: row.sample }));
}

/**
 * A group of variants under `title`, or why it doesn't apply; `chip` names it on a stage's switcher
 * where the section's own heading already says its title.
 */
export function variants(title: string, rows: readonly Variant[] | DoesntApply, chip = title): Group {
  return { title, chip, items: applies(rows) ? variantItems(rows) : rows };
}
