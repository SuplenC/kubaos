// The state matrix: a row per variant and a column per state, once by night and once by day, each
// grid live and painted in its mode in the theme picked. A part with a lit line has a column after Lit at night for its signs-off
// look, pinned as Lit is; by day lit is always signs off, so the day grid has no such column. Wider
// than the page, the columns wrap onto bands.

import { el, label, mockCell } from "../dom.ts";
import { applies, type DoesntApply, type Focus, drawsScreen, LOOK_NAMES, STATE_NAMES, type State, type States } from "../page.ts";
import { type Mode, settings } from "../settings.ts";
import { bandedGrid } from "./bands.ts";
import { type Group, type Item, sampleOf } from "./shelf.ts";
import { painted } from "./swatches.ts";

const MODES: readonly Mode[] = ["dark", "light"];
const MODE_NAMES: Record<Mode, string> = { dark: "Night", light: "Day" };

/** A column of the matrix: a state, or the lit state's signs-off look. */
type Column<S> = S | "signsOff";

/** `draw(value)`, drawn now, or when it is first shown if `draw` draws a whole screen. */
function later<T>(draw: (value: T) => Element, value: T): Element | (() => Element) {
  return drawsScreen(draw) ? () => draw(value) : draw(value);
}

/**
 * The matrix as a shelf's group: its cells night then day, a row per variant and a column per
 * state, each cell painted in its mode.
 */
export function matrix<S extends State>(states: States<S>, focus: Focus | DoesntApply): Group {
  const theme = settings().theme;
  const off = applies(focus) && (states.shown as readonly State[]).includes("lit") ? focus : null;
  const columns = (mode: Mode) =>
    states.shown.flatMap((state): Column<S>[] => (off && mode === "dark" && state === "lit" ? [state, "signsOff"] : [state]));
  const name = (column: Column<S>) => (column === "signsOff" ? LOOK_NAMES.signsOff : STATE_NAMES[column]);
  const words = (column: Column<S>) => (column === "signsOff" ? off!.looks.signsOff : states.looks[column]);
  const items: Item[] = MODES.flatMap((mode) =>
    states.rows.flatMap((row, index) =>
      columns(mode).map((column) => ({
        name: `${row.name} · ${name(column)} · ${MODE_NAMES[mode]}`,
        words: words(column),
        sample: column === "signsOff" ? later(off!.signsOff, index) : later(row.cell, column),
        make: () => (column === "signsOff" ? off!.signsOff(index) : row.cell(column)),
        at: [MODE_NAMES[mode], row.name, name(column)],
        frame: (sample: Element) => painted(theme, mode, sample),
      })),
    ),
  );
  const grids = (all: readonly Item[]): HTMLElement => {
    let next = 0;
    return el(
      "div",
      "page-part",
      ...MODES.map((mode) => {
        const grid = bandedGrid(
          columns(mode).map(name),
          states.rows.map((row) => ({ name: row.name, cells: columns(mode).map(() => mockCell(sampleOf(all[next++]!))) })),
        );
        return painted(theme, mode, label(MODE_NAMES[mode], "label", "on_surface_variant"), grid);
      }),
    );
  };
  return { title: "", items, small: grids };
}
