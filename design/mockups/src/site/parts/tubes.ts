// The focus tubes: which tone each variant's tube and glow light in, and whether its fill is the
// tube with the tone's core outside it.

import { toneColour } from "../../lib/focus.ts";
import type { NonEmpty, Tube } from "../page.ts";
import { el, label, prose } from "../dom.ts";

/** What `tube` looks like, in words. */
function described(tube: Tube): string {
  const colour = toneColour(tube.tone);
  const tone = tube.tone === "accent" ? "the accent" : `the ${tube.tone} tone`;
  return tube.filled
    ? `In ${tone}: filled with \`${colour}\`, so its fill is the tube, \`${colour}_core\` rides just outside it, and the glow beyond is \`${colour}\`.`
    : `In ${tone}: the tube on its edge and the glow outside it are \`${colour}\`.`;
}

export function tubes(list: NonEmpty<Tube>): HTMLElement {
  return el(
    "dl",
    "page-list",
    ...list.flatMap((tube) => [el("dt", "", label(tube.name, "body_strong", "on_surface")), el("dd", "", prose(described(tube)))]),
  );
}
