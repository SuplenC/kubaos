// The state matrix: a row per variant, a column per state, every cell live, then what each state
// is and which states don't apply, with why.

import { STATE_NAMES, type State, type States } from "../page.ts";
import { el, label, mockCell, prose } from "../dom.ts";
import { subheading } from "./section.ts";

export function matrix<S extends State>(states: States<S>): HTMLElement {
  const grid = el("div", "page-matrix");
  grid.style.setProperty("--columns", String(states.shown.length));
  grid.append(el("div", "page-cell"));
  for (const state of states.shown) grid.append(el("div", "page-cell", label(STATE_NAMES[state], "label")));
  for (const row of states.rows) {
    grid.append(el("div", "page-cell", label(row.name, "label")));
    for (const state of states.shown) grid.append(mockCell(row.cell(state)));
  }
  const described = el(
    "dl",
    "page-list",
    ...states.shown.flatMap((state) => [
      el("dt", "", label(STATE_NAMES[state], "body_strong", "on_surface")),
      el("dd", "", prose(states.described[state])),
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
    grid,
    subheading("What each state is"),
    described,
    ...(absent.length ? [subheading("States it doesn't have"), missing] : []),
  );
}
