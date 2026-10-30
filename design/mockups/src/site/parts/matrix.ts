// The state matrix: a row per variant, a column per state, every cell live, then what each state
// is and which states don't apply, with why. A part with a focus line has a column after Focused
// for its unlit look, pinned as Focused is. Wider than the page, the columns wrap onto bands.

import { applies, type DoesntApply, type Focus, STATE_NAMES, type State, type States } from "../page.ts";
import { el, label, mockCell, prose } from "../dom.ts";
import { bandedGrid } from "./bands.ts";
import { subheading } from "./section.ts";

const UNLIT = "Focused · unlit";

export function matrix<S extends State>(states: States<S>, focus: Focus | DoesntApply): HTMLElement {
  const unlit = applies(focus) && (states.shown as readonly State[]).includes("focused") ? focus : null;
  const columns = states.shown.flatMap((state): (S | "unlit")[] => (unlit && state === "focused" ? [state, "unlit"] : [state]));
  const name = (column: S | "unlit") => (column === "unlit" ? UNLIT : STATE_NAMES[column]);
  const grid = bandedGrid(
    columns.map(name),
    states.rows.map((row, index) => ({
      name: row.name,
      cells: columns.map((column) => mockCell(column === "unlit" ? unlit!.unlit(index) : row.cell(column))),
    })),
  );
  const described = el(
    "dl",
    "page-list",
    ...columns.flatMap((column) => [
      el("dt", "", label(name(column), "body_strong", "on_surface")),
      el("dd", "", prose(column === "unlit" ? unlit!.looks.unlit : states.described[column])),
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
