// Readings in columns: `kubaos-ui`'s `table::table` and `table::table_row`, `KubaOS.Components`'
// `Table` and `TableRow` (`docs/design/components/table.md`). A header in `label`, then a compact
// row per entry: a line of body text with `space.s4` above and below (`size.row_compact`), so many
// fit. Columns take set shares of the width, `space.s12` apart and `space.s12` in from the row's
// ends, split in whole device pixels; words sit at their column's start on one line, ending in an
// ellipsis; numbers at its end in `mono_large`, so their digits line up, as their header does.
//
// The whole row is one control in the state model (`.k-control` in `lib.css`), as a Collapsible's
// header is: lit (the pointer on it, or the keyboard, which the pointer moves) its tube burns
// inside its edge; pressed, the tube runs white-hot. The picked row is `secondary_container`, its
// words `on_secondary_container`, and lit it keeps its tube on that, as a Tonal button does.

import "./lists.css";
import { focusRing } from "./focus.ts";
import { scaleOf } from "./list.ts";
import { split } from "./layout.ts";
import { FIXED_HEIGHT, probe } from "./probe.ts";
import { clamped, text } from "./text.ts";
import { length } from "./tokens.ts";

export interface Column {
  readonly title: string;
  /** Its share of the width, against the other columns'. */
  readonly weight: number;
  /** A number: at its column's end, in `mono_large`. */
  readonly numeric?: boolean;
}

export interface TableRowSpec {
  readonly cells: readonly string[];
  readonly picked?: boolean;
}

/** A state a row is pinned in, as the specimens pin them. */
export type TablePinned = "lit" | "signs-off" | "pressed";

/** One cell: words at its column's start, or a number at its end. */
function cell(words: string, column: Column, header: boolean): HTMLElement {
  const style = header ? "label" : column.numeric ? "mono_large" : "body";
  const shown = column.numeric ? text(words, style) : clamped(words, style);
  shown.style.color = header ? "var(--color-on_surface_variant)" : column.numeric ? "var(--muted)" : "var(--ink)";
  const box = document.createElement("span");
  box.className = "k-table-cell";
  box.toggleAttribute("data-numeric", column.numeric ?? false);
  box.append(shown);
  return box;
}

export function tableRow(columns: readonly Column[], spec: TableRowSpec, pinned?: TablePinned): HTMLElement {
  const item = document.createElement("span");
  item.className = "k-table-row k-control";
  item.setAttribute("role", "row");
  item.toggleAttribute("data-picked", spec.picked ?? false);
  item.setAttribute("aria-selected", String(spec.picked ?? false));
  item.toggleAttribute("data-hot", !spec.picked);
  if (pinned) item.dataset.pinned = pinned;
  item.append(...columns.map((column, index) => cell(spec.cells[index] ?? "", column, false)));
  focusRing(item, "row", "accent", false).dataset.pressable = "";
  return probe(item, { role: "component", name: "TableRow", variant: "", fixed: FIXED_HEIGHT });
}

export interface TableOptions {
  /** The row the keyboard is on, by its index. */
  readonly current?: number;
  readonly pinned?: TablePinned;
  /** Picking a row: the owner draws it picked. */
  readonly onPick?: (index: number) => void;
}

export function table(columns: readonly Column[], rows: readonly TableRowSpec[], options: TableOptions = {}): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-table";
  box.setAttribute("role", "grid");
  box.tabIndex = 0;
  const header = document.createElement("span");
  header.className = "k-table-header";
  header.setAttribute("role", "row");
  header.append(...columns.map((column) => cell(column.title, column, true)));
  const items = rows.map((spec, index) => tableRow(columns, spec, index === options.current ? options.pinned : undefined));
  box.append(header, ...items);
  let current = options.current ?? -1;
  items[current]?.setAttribute("data-current", "");

  new ResizeObserver(() => {
    const scale = scaleOf(box);
    const inside = box.clientWidth - 2 * length("space-s12") * scale;
    const widths = split(inside, length("space-s12") * scale, columns.map((column) => column.weight));
    box.style.setProperty("--columns", widths.map((each) => `${each}px`).join(" "));
  }).observe(box);

  function move(index: number): void {
    items[current]?.removeAttribute("data-current");
    current = index;
    items[current]?.setAttribute("data-current", "");
  }

  items.forEach((item, index) => {
    item.addEventListener("pointermove", () => {
      if (current !== index) move(index);
    });
    item.addEventListener("click", () => options.onPick?.(index));
  });
  box.addEventListener("pointerleave", () => {
    if (!box.matches(":focus")) move(-1);
  });
  box.addEventListener("focus", () => move(current >= 0 ? current : Math.max(0, rows.findIndex((each) => each.picked))));
  box.addEventListener("blur", () => move(-1));
  box.addEventListener("keydown", (event) => {
    const last = items.length - 1;
    const key = event.key.toLowerCase();
    if (event.key === "ArrowDown" || (!event.ctrlKey && key === "j") || (event.ctrlKey && (key === "j" || key === "n"))) move((current + 1) % items.length);
    else if (event.key === "ArrowUp" || (!event.ctrlKey && key === "k") || (event.ctrlKey && (key === "k" || key === "p"))) move((current - 1 + items.length) % items.length);
    else if (event.key === "Home") move(0);
    else if (event.key === "End") move(last);
    else if (event.key === "Enter") {
      if (event.repeat || current < 0) return;
      const item = items[current]!;
      item.dataset.keyed = "";
      setTimeout(() => delete item.dataset.keyed, 100);
      options.onPick?.(current);
    } else return;
    event.preventDefault();
  });
  return probe(box, { role: "component", name: "Table", variant: "", fixed: { width: false, height: false } });
}
