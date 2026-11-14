// Every key a surface has, in a popover over the button that opens it
// (`docs/design/components/shortcuts.md`): grouped by what they do, each group's name in `label`
// over its rows, each row what the keys do in `body` and the keys at its end as keycaps, so a
// surface can show no key hints and still say every key it takes.

import { keycap, keys } from "./keycap.ts";
import { row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { text } from "./text.ts";
import { popover } from "./tooltip.ts";

/** A group of keys: what they are for, then each combination and what it does. */
export interface ShortcutGroup {
  readonly name: string;
  readonly keys: readonly (readonly [combination: string, does: string])[];
}

/**
 * The popover: `size.panel` wide on the ladder's step 2, `space.s16` between groups, `space.s4`
 * from a group's name to its rows, each row `size.row_compact` tall at least, its words at its
 * start and its keycaps `space.s4` apart in a column at its end.
 */
export function shortcuts(groups: readonly ShortcutGroup[]): HTMLElement {
  const made = popover(
    2,
    stack(
      "s16",
      "start",
      ...groups.map((group) =>
        stack(
          "s4",
          "start",
          text(group.name, "label", "on_surface_variant"),
          ...group.keys.map(([combination, does]) => {
            const caps = row("s4", "centre", ...keys(combination).map(keycap));
            caps.classList.add("k-shortcut-keys");
            const line = row("s16", "centre", text(does, "body"), caps);
            line.classList.add("k-shortcut");
            return line;
          }),
        ),
      ),
    ),
  );
  made.classList.add("k-shortcuts");
  // The groups and their rows take the popover's whole width, so every row's keys end at its edge.
  const body = made.firstElementChild as HTMLElement;
  body.classList.add("k-shortcut-groups");
  for (const group of body.children) group.classList.add("k-shortcut-group");
  return probe(made, { role: "component", name: "Shortcuts", variant: "", fixed: { width: true, height: false } });
}
