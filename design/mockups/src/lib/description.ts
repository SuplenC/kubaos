// Facts beside their labels, as a summary reads them: `kubaos-ui`'s `description::description_list`,
// `KubaOS.Components`' `DescriptionList` (`docs/design/components/description-list.md`). One under
// the other, `space.s8` apart: each label in `small`, `on_surface_variant`, a quarter of the width,
// its value in `body` the rest, `space.s8` from it, wrapping, and an optional `mono` line
// `space.s4` under the value for a path, an email or a version, on one line ending in an ellipsis,
// since those break badly wrapped mid-token. The quarter is split in whole device pixels.

import "./lists.css";
import { scaleOf } from "./list.ts";
import { probe } from "./probe.ts";
import { split } from "./table.ts";
import { clamped, wrapped } from "./text.ts";
import { length } from "./tokens.ts";

export interface Fact {
  readonly label: string;
  readonly value: string;
  /** A path, an email or a version under the value, in `mono`. */
  readonly detail?: string;
}

export function descriptionList(facts: readonly Fact[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-facts";
  for (const fact of facts) {
    const value = document.createElement("span");
    value.className = "k-fact-value";
    value.append(wrapped(fact.value, "body"));
    if (fact.detail !== undefined) value.append(clamped(fact.detail, "mono", "on_surface_variant"));
    box.append(wrapped(fact.label, "small", "on_surface_variant"), value);
  }
  new ResizeObserver(() => {
    const scale = scaleOf(box);
    const widths = split(box.clientWidth, length("space-s8") * scale, [1, 3]);
    box.style.setProperty("--columns", widths.map((each) => `${each}px`).join(" "));
  }).observe(box);
  return probe(box, { role: "component", name: "DescriptionList", variant: "", fixed: { width: false, height: false } });
}
