// Facts beside their labels, as a summary reads them: `kubaos-ui`'s `description::description_list`,
// `KubaOS.Components`' `DescriptionList` (`docs/design/components/description-list.md`). One under
// the other, `space.s8` apart: each label in `small`, `on_surface_variant`, a quarter of the width,
// its value in `body` the rest, `space.s8` from it, wrapping, and an optional `mono` line
// `space.s4` under the value for a path, an email or a version, on one line ending in an ellipsis,
// since those break badly wrapped mid-token. The quarter is split in whole device pixels, and
// never narrower than a label's longest word.
//
// A line may carry one action at its right end (an IconButton, Text, sm: copy this fact). A list
// with any action keeps a column `size.control_sm` wide at the right of every line, `space.s8`
// from the values, so the action never covers a value and a value too long for what is left
// wraps. The action shows while the pointer is on its line or the keyboard is on it, fading over
// `short` on `standard`; hidden, it takes no press. It is centred down its line and adds nothing
// to the line's height.

import "./lists.css";
import { split } from "./layout.ts";
import { scaleOf } from "./list.ts";
import { probe } from "./probe.ts";
import { clamped, wrapped } from "./text.ts";
import { length } from "./tokens.ts";

export interface Fact {
  readonly label: string;
  readonly value: string;
  /** A path, an email or a version under the value, in `mono`. */
  readonly detail?: string;
  /** One action at the line's end, shown while the line is pointed at or holds the keyboard: an IconButton (Text, sm). */
  readonly action?: HTMLElement;
}

export function descriptionList(facts: readonly Fact[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-facts";
  const acting = facts.some((fact) => fact.action !== undefined);
  box.toggleAttribute("data-actions", acting);
  for (const fact of facts) {
    const value = document.createElement("span");
    value.className = "k-fact-value";
    value.append(wrapped(fact.value, "body"));
    if (fact.detail !== undefined) value.append(clamped(fact.detail, "mono", "on_surface_variant"));
    const label = wrapped(fact.label, "small", "on_surface_variant");
    if (!acting) {
      box.append(label, value);
      continue;
    }
    // A line of its own, laid out by the list's grid, so pointing at any of it is pointing at the line.
    const line = document.createElement("span");
    line.className = "k-fact-line";
    const end = document.createElement("span");
    end.className = "k-fact-action";
    if (fact.action) end.append(fact.action);
    line.append(label, value, end);
    box.append(line);
  }
  new ResizeObserver(() => {
    const scale = scaleOf(box);
    const widths = split(box.clientWidth, length("space-s8") * scale, [1, 3]);
    // A label never narrower than its longest word, so a narrow list wraps it between words.
    box.style.setProperty("--columns", `minmax(min-content, ${widths[0]}px) minmax(0, 1fr)${acting ? " var(--size-control_sm)" : ""}`);
  }).observe(box);
  return probe(box, { role: "component", name: "DescriptionList", variant: acting ? "actions" : "", fixed: { width: false, height: false } });
}

/** The action cell of `value`'s line, where a line's action goes once it has one (a fact shown late). */
export function actionCell(value: HTMLElement): HTMLElement | null {
  return value.parentElement?.querySelector<HTMLElement>(":scope > .k-fact-action") ?? null;
}
