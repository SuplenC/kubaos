// Work that doesn't know how far it is: `kubaos-ui`'s `progress::spinner`, `KubaOS.Components`'
// `Spinner` (`docs/design/components/spinner.md`). An icon size across, a lit stretch of neon
// running round an unlit ring, both `stroke.thin` in whole device pixels; `.k-spinner` in
// `lib.css` draws it and turns it every `spinner.turn`.

import { probe } from "./probe.ts";
import { CHROME } from "../site/settings.ts";

/** The icon sizes a spinner takes: 16, 20 and 24 beside text, 32 and 48 for a page's one. */
export const SPINNER_SIZES = ["s16", "s20", "s24", "s32", "s48"] as const;
export type SpinnerSize = (typeof SPINNER_SIZES)[number];

const SVG = "http://www.w3.org/2000/svg";

/** A spinner `size` across in the tone `color` (its core lit at night), turning; held still in a shot. */
export function spinner(size: SpinnerSize, color = "primary"): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-spinner";
  box.style.setProperty("--spinner-size", `var(--icon-${size})`);
  box.style.setProperty("--tone", `var(--color-${color})`);
  box.style.setProperty("--core", `var(--color-${color}_core)`);
  box.toggleAttribute("data-held", !CHROME);
  const drawing = document.createElementNS(SVG, "svg");
  for (const part of ["k-spinner-ring", "k-spinner-arc"]) {
    const circle = document.createElementNS(SVG, "circle");
    circle.setAttribute("class", part);
    // The arc's length in degrees, so its dash is its sweep.
    circle.setAttribute("pathLength", "360");
    drawing.append(circle);
  }
  box.append(drawing);
  return probe(box, { role: "component", name: "Spinner", variant: size.slice(1), fixed: { width: true, height: true } });
}
