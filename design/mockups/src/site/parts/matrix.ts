// The state matrix: a row per variant and a column per state, once by night and once by day, each
// grid live and painted in its mode in the theme picked, then what each state is and which states
// don't apply, with why. Wider than the page, the columns wrap onto bands.

import { el, label, mockCell, prose } from "../dom.ts";
import { STATE_NAMES, type State, type States } from "../page.ts";
import { type Mode, settings } from "../settings.ts";
import { bandedGrid } from "./bands.ts";
import { subheading } from "./section.ts";
import { painted } from "./swatches.ts";

const MODES: readonly Mode[] = ["dark", "light"];
const MODE_NAMES: Record<Mode, string> = { dark: "Night", light: "Day" };

export function matrix<S extends State>(states: States<S>): HTMLElement {
  const theme = settings().theme;
  const grids = MODES.map((mode) => {
    const grid = bandedGrid(
      states.shown.map((state) => STATE_NAMES[state]),
      states.rows.map((row) => ({ name: row.name, cells: states.shown.map((state) => mockCell(row.cell(state))) })),
    );
    return painted(theme, mode, label(MODE_NAMES[mode], "body_strong", "on_surface"), grid);
  });
  const described = el(
    "dl",
    "page-list",
    ...states.shown.flatMap((state) => [el("dt", "", label(STATE_NAMES[state], "body_strong", "on_surface")), el("dd", "", prose(states.described[state]))]),
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
