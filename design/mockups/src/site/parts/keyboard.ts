// The keyboard as two tables: every key and what it does, then where focus goes and lands.

import { keycap, keys as split } from "../../lib/keycap.ts";
import { el } from "../dom.ts";
import type { Keyboard } from "../page.ts";
import { pair, specTable, titled } from "./table.ts";

/** A row's keys as keycaps, one cap per key, the combinations after one another. */
function caps(combinations: readonly string[]): HTMLElement {
  return el("span", "page-keys", ...combinations.map((combination) => el("span", "page-combination", ...split(combination).map(keycap))));
}

export function keyboard(spec: Keyboard): HTMLElement {
  const keys = specTable(
    { heads: ["Key", "Does"] },
    spec.keys.map((row) => ({ cells: [caps(row.keys), row.does] })),
  );
  const focus = specTable(
    { heads: ["Focus", "Where"] },
    [
      { cells: ["Tab order", spec.order] },
      { cells: ["Lands", spec.lands] },
    ],
  );
  return pair(titled("Keys", keys.element), titled("Focus", focus.element));
}
