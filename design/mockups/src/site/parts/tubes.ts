// The line a part lights: what each look of lit makes of it, signs on and signs off, then which
// tone each variant's line and glow light in, whether it has a fill (the line inside the fill's
// edge is the tone's core with the signs on), and whether its content, drawn in its tone, burns as
// a tube with it.

import { toneColour } from "../../lib/focus.ts";
import { el, label, prose } from "../dom.ts";
import { type Focus, LOOK_NAMES, LOOKS, type Tube } from "../page.ts";
import { subheading } from "./section.ts";

/** What `tube` is in each look, in words. */
function described(tube: Tube): string {
  const colour = toneColour(tube.tone);
  const tone = tube.tone === "accent" ? "the accent" : `\`${colour}\``;
  const line = tube.filled
    ? `In ${tone}: it has a fill, so lit with the signs on, the line inside the fill's edge is the tone's core, \`stroke.thin\` wide, with the glow outside in \`${colour}\`. With the signs off the same line is unlit glass, and nothing glows; by day that is a dark line.`
    : `In ${tone}: lit with the signs on, the tube inside its edge is \`${colour}\`, \`focus.tube\` wide, with the glow outside it. With the signs off the same tube is unlit glass, the tone unlit, and nothing glows; by day it is the tone's line.`;
  const content = tube.content
    ? ` Its content is drawn in its tone, so with the signs on it burns too: its letters and icon go to the core with a faint near glow round them (\`focus.content_near\` from \`focus.content_glow\`) and no far glow. With them off, it is at its rest colour.`
    : "";
  return line + content;
}

export function tubes(focus: Focus): HTMLElement {
  const list = (entries: readonly (readonly [string, string])[]) =>
    el("dl", "page-list", ...entries.flatMap(([name, words]) => [el("dt", "", label(name, "body_strong", "on_surface")), el("dd", "", prose(words))]));
  return el(
    "div",
    "page-part",
    list(LOOKS.map((look) => [LOOK_NAMES[look], focus.looks[look]] as const)),
    subheading("Each variant's line"),
    list(focus.tubes.map((tube) => [tube.name, described(tube)] as const)),
  );
}
