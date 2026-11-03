// Variants and sizes: each one live, its name, and what it is for.

import type { Variant } from "../page.ts";
import { el, label, mockCell, prose } from "../dom.ts";

export function variants(rows: readonly Variant[]): HTMLElement {
  const table = el("div", "page-variants");
  for (const row of rows) {
    table.append(el("div", "page-cell", label(row.name, "label")), mockCell(row.sample()), prose(row.for));
  }
  stackWhenNarrow(table);
  return table;
}

// A sample as wide as a window leaves the words beside it a sliver, one word a line: when the
// column left for them is narrower than `size.rail`, every row's words go under its sample.
export function stackWhenNarrow(table: HTMLElement): void {
  const fit = (): void => {
    const cells = [...table.children] as HTMLElement[];
    const names = cells.filter((_, i) => i % 3 === 0);
    const mocks = cells.filter((_, i) => i % 3 === 1);
    if (mocks.length === 0) return;
    const widest = (list: HTMLElement[]): number => Math.max(...list.map((cell) => cell.firstElementChild?.getBoundingClientRect().width ?? 0));
    const style = getComputedStyle(table);
    const gap = parseFloat(style.columnGap) || 0;
    const probe = el("div", "");
    probe.style.width = "var(--size-rail)";
    table.append(probe);
    const least = probe.getBoundingClientRect().width;
    probe.remove();
    const left = table.getBoundingClientRect().width - widest(names) - widest(mocks) - 2 * gap;
    table.toggleAttribute("data-stacked", left < least);
  };
  new ResizeObserver(fit).observe(table);
}
