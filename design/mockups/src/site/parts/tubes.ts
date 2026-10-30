// The line a part lights: what lit makes of it, then which tone each variant's line and glow light
// in, whether it has a fill (the line inside the fill's edge is the tone's core by night), and
// whether its content, drawn in its tone, burns as a tube with it.

import { toneColour } from "../../lib/focus.ts";
import { el, label, prose } from "../dom.ts";
import type { Focus, Tube } from "../page.ts";
import { subheading } from "./section.ts";

/** What `tube` is lit, in words. */
function described(tube: Tube): string {
  const colour = toneColour(tube.tone);
  const tone = tube.tone === "accent" ? "the accent" : `\`${colour}\``;
  const line = tube.filled
    ? `In ${tone}: it has a fill, so lit, the line inside the fill's edge is the tone's core at night, \`stroke.thin\` wide, with the glow outside in \`${colour}\`; by day it is a dark line, and nothing glows.`
    : `In ${tone}: lit, the tube inside its edge is \`${colour}\`, \`focus.tube\` wide, with the glow outside it at night; by day the same line, and nothing glows.`;
  const content = tube.content
    ? ` Its content is drawn in its tone, so lit at night it burns too: its letters and icon go to the core with a faint near glow round them (\`focus.content_near\` from \`focus.content_glow\`) and no far glow.`
    : "";
  return line + content;
}

export function tubes(focus: Focus): HTMLElement {
  return el(
    "div",
    "page-part",
    prose(focus.lit),
    subheading("Each variant's line"),
    el("dl", "page-list", ...focus.tubes.flatMap((tube) => [el("dt", "", label(tube.name, "body_strong", "on_surface")), el("dd", "", prose(described(tube)))])),
  );
}
