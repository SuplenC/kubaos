// Variants and sizes: each one live, its name, and what it is for.

import type { Variant } from "../page.ts";
import { el, label, mockCell, prose } from "../dom.ts";

export function variants(rows: readonly Variant[]): HTMLElement {
  const table = el("div", "page-variants");
  for (const row of rows) {
    table.append(el("div", "page-cell", label(row.name, "label")), mockCell(row.sample()), prose(row.for));
  }
  return table;
}
