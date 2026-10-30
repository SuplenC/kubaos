// The layout primitives a mock is built from: `kubaos-ui`'s `layout::row` and `stack`. Each child
// is its own box, already whole device pixels wide, so every child starts on one.

import { type Align, probe } from "./probe.ts";
import { length } from "./tokens.ts";

/** Children side by side, `gap` apart (a spacing token's name, `s8`, or 0), lined up by `align`. */
export function row(gap: string | 0, align: Align, ...children: Element[]): HTMLElement {
  return layout("x", gap, align, children);
}

/** Children one under another, `gap` apart, lined up by `align`. */
export function stack(gap: string | 0, align: Align, ...children: Element[]): HTMLElement {
  return layout("y", gap, align, children);
}

function layout(axis: "x" | "y", gap: string | 0, align: Align, children: Element[]): HTMLElement {
  const element = document.createElement("span");
  element.className = axis === "x" ? "k-row" : "k-stack";
  element.dataset.align = align;
  element.dataset.layout = "";
  element.style.setProperty("--gap", gap === 0 ? "0px" : `var(--space-${gap})`);
  element.append(...children);
  return probe(element, { role: "layout", axis, gap: gap === 0 ? 0 : length(`space-${gap}`), align });
}
