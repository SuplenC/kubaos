// The page's own tables, which carry its words: a header of labels, then a row per entry of a few
// words each, a `backticked` token set as code. Two short tables sit side by side where the width
// holds both (`pair`).

import { el, label, prose } from "../dom.ts";

/** A cell: the page's words, or anything drawn (a keycap, a swatch). */
export type TableCell = string | Element;

export interface Row {
  readonly cells: readonly TableCell[];
  /** A row that says what doesn't apply, drawn muted. */
  readonly muted?: boolean;
}

/** A column whose cells are values, set in mono at its end, so their digits line up. */
export interface Columns {
  readonly heads: readonly string[];
  readonly numeric?: readonly number[];
}

function cell(content: TableCell, numeric: boolean, muted: boolean): HTMLTableCellElement {
  const box = el("td", numeric ? "page-spec-value" : "");
  box.append(typeof content === "string" ? prose(content, numeric ? "mono" : "body", muted ? "on_surface_variant" : "on_surface") : content);
  return box;
}

/** A table of `columns` and `rows`, each row's element in the result's `rows`, in order. */
export function specTable(columns: Columns, rows: readonly Row[]): { readonly element: HTMLElement; readonly rows: HTMLElement[] } {
  const numeric = new Set(columns.numeric ?? []);
  const head = el("tr", "", ...columns.heads.map((words, i) => el("th", numeric.has(i) ? "page-spec-value" : "", label(words, "label"))));
  const drawn = rows.map((row) => {
    const line = el("tr", "", ...row.cells.map((content, i) => cell(content, numeric.has(i), row.muted ?? false)));
    line.toggleAttribute("data-muted", row.muted ?? false);
    return line;
  });
  const element = el("div", "page-spec-box", el("table", "page-spec-table", el("thead", "", head), el("tbody", "", ...drawn)));
  return { element, rows: drawn };
}

/** Two or more short tables side by side while the width holds a column for each, else stacked. */
export function pair(...tables: Element[]): HTMLElement {
  return el("div", "page-pair", ...tables);
}

/** A table under a small heading of its own: what a pair's tables are each about. */
export function titled(title: string, table: Element): HTMLElement {
  return el("div", "page-titled", label(title, "subheading", "on_surface"), table);
}
