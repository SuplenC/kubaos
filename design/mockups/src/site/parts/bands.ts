// A grid of samples, a row per name and a column per heading, that wraps onto bands of columns when
// it is wider than its room: each band a grid of its own with the names again and its columns'
// headings, as few bands as fit, their columns shared out evenly in order.

import { el, label } from "../dom.ts";

interface Banded {
  readonly heads: readonly HTMLElement[];
  readonly names: readonly string[];
  /** A row's cells, a column each. */
  readonly cells: readonly (readonly HTMLElement[])[];
  /** How many columns each band holds. */
  split: readonly number[];
}

const grids = new WeakMap<HTMLElement, Banded>();

/** A grid of `heads` across and `rows` down, each row its name and a cell per heading. */
export function bandedGrid(heads: readonly string[], rows: readonly { name: string; cells: readonly HTMLElement[] }[]): HTMLElement {
  const banded: Banded = {
    heads: heads.map((head) => el("div", "page-cell", label(head, "label"))),
    names: rows.map((row) => row.name),
    cells: rows.map((row) => row.cells),
    split: [heads.length],
  };
  const element = el("div", "page-bands");
  grids.set(element, banded);
  draw(element, banded);
  return element;
}

/** Lays `banded`'s cells out in `element` as its split says. */
function draw(element: HTMLElement, banded: Banded): void {
  let first = 0;
  element.replaceChildren(
    ...banded.split.map((count) => {
      const columns = range(first, count);
      first += count;
      const grid = el("div", "page-matrix");
      grid.style.setProperty("--columns", String(count));
      grid.append(el("div", "page-cell"), ...columns.map((column) => banded.heads[column]!));
      banded.names.forEach((name, row) => {
        grid.append(el("div", "page-cell", label(name, "label")), ...columns.map((column) => banded.cells[row]![column]!));
      });
      return grid;
    }),
  );
}

function range(from: number, count: number): number[] {
  return Array.from({ length: count }, (_, i) => from + i);
}

/** `total` columns shared out over `bands` in order, the first bands taking any left over. */
function evenly(total: number, bands: number): number[] {
  return Array.from({ length: bands }, (_, i) => Math.floor(total / bands) + (i < total % bands ? 1 : 0));
}

/**
 * The fewest bands, their columns shared out evenly, in which every band fits `room`: a band is
 * the names' column, then each of its columns after a gap.
 */
function splitFor(widths: readonly number[], names: number, gap: number, room: number): number[] {
  for (let bands = 1; bands < widths.length; bands++) {
    const split = evenly(widths.length, bands);
    let first = 0;
    const fits = split.every((count) => {
      const width = range(first, count).reduce((sum, column) => sum + gap + widths[column]!, names);
      first += count;
      return width <= room;
    });
    if (fits) return split;
  }
  return widths.map(() => 1);
}

/**
 * Bands every banded grid on `page` for its room. A column is as wide as its widest cell whatever
 * band it is in, so the widths read from the grid as it is drawn hold for every split.
 */
export function reflowBands(page: HTMLElement): void {
  for (const element of page.querySelectorAll<HTMLElement>(".page-bands")) {
    const banded = grids.get(element);
    const band = element.firstElementChild;
    if (!banded || !(band instanceof HTMLElement)) continue;
    // Read in the viewport's pixels, which the page's zoom shrinks from its CSS pixels.
    const zoom = element.getBoundingClientRect().width / element.offsetWidth || 1;
    const gap = Number.parseFloat(getComputedStyle(band).columnGap) * zoom;
    const names = band.firstElementChild?.getBoundingClientRect().width ?? 0;
    const widths = banded.heads.map((head) => head.getBoundingClientRect().width);
    const split = splitFor(widths, names, gap, element.getBoundingClientRect().width);
    if (split.join() === banded.split.join()) continue;
    banded.split = split;
    draw(element, banded);
  }
}
