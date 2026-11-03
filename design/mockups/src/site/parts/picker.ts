// A row of choices, one picked: the library's toggling Chips in a row that wraps, the picked one
// on and the rest Neutral. The site picks with it: the page's sections, a stage's samples, a
// player's moves. Chips rather than Tabs, since a sample's name runs to a sentence and a row of
// them must wrap where a Tabs well can't.

import { chip } from "../../lib/chip.ts";
import { el } from "../dom.ts";

export interface Picker {
  readonly element: HTMLElement;
  /** Marks `index` picked without telling `onPick`: the page's scroll lit another section. */
  set(index: number): void;
}

/** `names` as chips, `picked` on; a press picks one and calls `onPick` with it. */
export function picker(names: readonly string[], picked: number, onPick: (index: number) => void, label: string): Picker {
  const chips = names.map((name) => chip({ label: name, kind: "Selected", press: "toggle", on: false }));
  const set = (index: number): void => {
    chips.forEach((each, i) => {
      // A chip toggles itself on a press; a picker's chip is on only while it is the pick.
      each.setAttribute("aria-pressed", String(i === index));
      each.style.setProperty("--checked", i === index ? "1" : "0");
    });
  };
  chips.forEach((each, i) => {
    each.addEventListener("click", () => {
      set(i);
      onPick(i);
    });
    each.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      event.preventDefault();
      each.click();
    });
  });
  set(picked);
  const element = el("div", "page-picker", ...chips);
  element.setAttribute("role", "group");
  element.setAttribute("aria-label", label);
  return { element, set };
}
