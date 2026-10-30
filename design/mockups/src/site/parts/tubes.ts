// The focus line: what each look makes of it, then which tone each variant's tube and glow light
// in, whether its fill is the tube with the tone's core inside the fill's edge, and whether its
// content, drawn in its tone, lights as a tube with it.

import { toneColour } from "../../lib/focus.ts";
import { el, label, prose } from "../dom.ts";
import { type Focus, LOOK_NAMES, LOOKS, type Tube } from "../page.ts";
import { subheading } from "./section.ts";

/** What `tube` looks like in each look, in words. */
function described(tube: Tube): string {
  const colour = toneColour(tube.tone);
  const tone = tube.tone === "accent" ? "the accent" : `the ${tube.tone} tone`;
  const content = tube.content
    ? ` Its content is drawn in \`${colour}\`, so it lights with the line: lit, its letters and icon are \`${colour}_core\` with a faint near glow round them in \`${colour}\` (\`focus.content_near\` from \`focus.content_glow\`) and no far glow, striking and going out with the tube; unlit, they are \`${colour}\`, with no glow.`
    : "";
  const line = tube.filled
    ? `In ${tone}: filled with \`${colour}\`, so its fill is the tube and the line inside the fill's edge is its core. Lit, the core is \`${colour}_core\` with the glow outside in \`${colour}\`; unlit, it is \`${colour}_unlit\`, with no glow.`
    : `In ${tone}: the tube inside its edge. Lit, it is \`${colour}\` with the glow outside it in the same colour; unlit, it is glass, \`${colour}_unlit\` at \`focus.glass\` over the control's container, with no glow.`;
  return line + content;
}

export function tubes(focus: Focus): HTMLElement {
  const list = (entries: readonly (readonly [string, string])[]) =>
    el("dl", "page-list", ...entries.flatMap(([name, words]) => [el("dt", "", label(name, "body_strong", "on_surface")), el("dd", "", prose(words))]));
  return el(
    "div",
    "page-part",
    list(LOOKS.map((look) => [LOOK_NAMES[look], focus.looks[look]] as const)),
    subheading("Each variant's tube"),
    list(focus.tubes.map((tube) => [tube.name, described(tube)] as const)),
  );
}
