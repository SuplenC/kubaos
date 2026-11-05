// The line a part lights, as two tables: what each look of lit makes of it, signs on and signs
// off; then each variant's line, its tone, whether it is the core inside a fill's edge or the tube,
// and whether its content burns with it.

import { toneColour } from "../../lib/focus.ts";
import { type Focus, LOOK_NAMES, LOOKS } from "../page.ts";
import { pair, specTable, titled } from "./table.ts";

export function tubes(focus: Focus): HTMLElement {
  const looks = specTable(
    { heads: ["Look", "What"] },
    LOOKS.map((look) => ({ cells: [LOOK_NAMES[look], focus.looks[look]] })),
  );
  const lines = specTable(
    { heads: ["Variant", "Tone", "Line", "Content"] },
    focus.tubes.map((tube) => ({
      cells: [
        tube.name,
        tube.tone === "accent" ? "the accent" : `\`${toneColour(tube.tone)}\``,
        tube.filled ? "the core, `stroke.thin`" : "the tube, `focus.tube`",
        tube.content ? "burns with it" : "at rest",
      ],
    })),
  );
  return pair(titled("The lit line", looks.element), titled("Each variant's line", lines.element));
}
