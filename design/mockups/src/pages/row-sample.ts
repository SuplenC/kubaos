// A sample shared by Field's and Select's pages: their small size in a list row's control slot,
// in a well with the controls it sits beside in Settings: a compact slider, a switch and a choice.

import { field } from "../lib/field.ts";
import { within } from "../lib/layout.ts";
import { list, listRow, ROW_CONTROL } from "../lib/list.ts";
import { select } from "../lib/select.ts";
import { slider } from "../lib/slider.ts";
import { tabs } from "../lib/tabs.ts";

/** The control column a settings row ends in: a menu wide. */
const COLUMN = "var(--size-menu)";

/** A row that ends in `control`, held to the control column. */
function ending(title: string, subtitle: string, control: HTMLElement): HTMLElement {
  const made = listRow({ title, subtitle });
  made.firstElementChild?.append(within(COLUMN, control));
  return made;
}

/** A well of settings rows: a slider, a switch, a choice, then a field and a select at the row's size. */
export function rowOfControls(): HTMLElement {
  const names = ["Off", "Slow", "Normal", "Fast", "Fastest"];
  return within(
    "var(--size-column)",
    list([
      ending("Repeat speed", "How fast a held key types once it starts.", slider({ icon: "repeat", size: "Compact", stops: names.length, index: 3, names, widest: "Fastest" })),
      { title: "Mac-style accents", subtitle: "Left Alt with e, then a letter: é.", toggle: true },
      ending("Workspace numbers", "How the bar writes them.", tabs({ choices: [{ label: "123" }, { label: "一二三" }], picked: 0 })),
      ending("Try it here", "Hold a key to feel the repeat.", field({ size: ROW_CONTROL, placeholder: "Hold a key here" })),
      ending("New windows open", "Where a new window goes.", select({ size: ROW_CONTROL, options: ["Right or below", "Left or above"], picked: 0 })),
    ]),
  );
}
