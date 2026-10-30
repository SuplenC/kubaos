// The state matrix: a row per variant and a column per state, once by night and once by day, each
// grid live and painted in its mode in the theme picked, then what each state is and which states
// don't apply, with why. A part with a lit line has a column after Lit at night for its signs-off
// look, pinned as Lit is; by day lit is always signs off, so the day grid has no such column. Wider
// than the page, the columns wrap onto bands.

import { el, label, mockCell, prose } from "../dom.ts";
import { applies, type DoesntApply, type Focus, LOOK_NAMES, STATE_NAMES, type State, type States } from "../page.ts";
import { type Mode, settings } from "../settings.ts";
import { bandedGrid } from "./bands.ts";
import { subheading } from "./section.ts";
import { painted } from "./swatches.ts";

const MODES: readonly Mode[] = ["dark", "light"];
const MODE_NAMES: Record<Mode, string> = { dark: "Night", light: "Day" };

/** A column of the matrix: a state, or the lit state's signs-off look. */
type Column<S> = S | "signsOff";

export function matrix<S extends State>(states: States<S>, focus: Focus | DoesntApply): HTMLElement {
  const theme = settings().theme;
  const off = applies(focus) && (states.shown as readonly State[]).includes("lit") ? focus : null;
  const columns = (mode: Mode) =>
    states.shown.flatMap((state): Column<S>[] => (off && mode === "dark" && state === "lit" ? [state, "signsOff"] : [state]));
  const name = (column: Column<S>) => (column === "signsOff" ? LOOK_NAMES.signsOff : STATE_NAMES[column]);
  const grids = MODES.map((mode) => {
    const grid = bandedGrid(
      columns(mode).map(name),
      states.rows.map((row, index) => ({
        name: row.name,
        cells: columns(mode).map((column) => mockCell(column === "signsOff" ? off!.signsOff(index) : row.cell(column))),
      })),
    );
    const note = off && mode === "light" ? [prose("By day the signs are off, so Lit is always its signs-off look: it has no column of its own.", "body", "on_surface_variant")] : [];
    return painted(theme, mode, label(MODE_NAMES[mode], "body_strong", "on_surface"), ...note, grid);
  });
  const described = el(
    "dl",
    "page-list",
    ...columns("dark").flatMap((column) => [
      el("dt", "", label(name(column), "body_strong", "on_surface")),
      el("dd", "", prose(column === "signsOff" ? off!.looks.signsOff : states.described[column])),
    ]),
  );
  const absent = Object.entries(states.absent) as [State, string][];
  const missing = el(
    "dl",
    "page-list",
    ...absent.flatMap(([state, why]) => [
      el("dt", "", label(STATE_NAMES[state], "body_strong", "on_surface")),
      el("dd", "", prose(`Doesn't apply: ${why}`, "body", "on_surface_variant")),
    ]),
  );
  return el(
    "div",
    "page-part",
    ...grids,
    subheading("What each state is"),
    described,
    ...(absent.length ? [subheading("States it doesn't have"), missing] : []),
  );
}
