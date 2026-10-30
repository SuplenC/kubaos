// The focus tubes: which tone each variant's tube and glow light in, and whether its fill is the
// tube with the tone's core inside the fill's edge.

import { toneColour } from "../../lib/focus.ts";
import { el, label, prose } from "../dom.ts";
import type { NonEmpty, Tube } from "../page.ts";

/** What `tube` looks like, in words. */
function described(tube: Tube): string {
  const colour = toneColour(tube.tone);
  const tone = tube.tone === "accent" ? "the accent" : `the ${tube.tone} tone`;
  return tube.filled
    ? `In ${tone}: filled with \`${colour}\`, so its fill is the tube and the line inside the fill's edge is its core, \`${colour}_core\` at night, \`${colour}_unlit\` in daylight; the glow outside, at night only, is \`${colour}\`.`
    : `In ${tone}: the tube inside its edge is \`${colour}\` at night, with the glow outside it in the same colour; in daylight it is unlit glass, \`${colour}_unlit\` at \`focus.glass\` over the control's container, with no glow.`;
}

export function tubes(list: NonEmpty<Tube>): HTMLElement {
  return el(
    "dl",
    "page-list",
    ...list.flatMap((tube) => [el("dt", "", label(tube.name, "body_strong", "on_surface")), el("dd", "", prose(described(tube)))]),
  );
}
