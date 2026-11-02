// A status message's box and its lit edge (`docs/DESIGN.md`, "States": fixed signals are always
// lit), shared by `Banner` and `ErrorBlock`. The box lies flat in the page: a fill, the sunken
// soft hairline (`.k-sunken`), the content radius and `space.s16` inside. Its lit edge is the
// box's own left edge, the thin stroke wide in whole device pixels, tapering into both corners over
// the same length: the tone's core with the tone's near glow at night, while the signs are on, and
// the tone with no glow by day. `feedback.css` draws both.

import "./feedback.css";

/** The tones a status message lights its edge in, by their colour role. */
export type StatusTone = "primary" | "warning" | "error";

/** A status message's box in `tone`, filled with `fill` (a colour's CSS), holding `content`. */
export function statusBox(tone: StatusTone, fill: string, ...content: Element[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-status k-sunken";
  box.style.setProperty("--status-fill", fill);
  box.style.setProperty("--status-tone", `var(--color-${tone})`);
  box.style.setProperty("--status-core", `var(--color-${tone}_core)`);
  const edge = document.createElement("span");
  edge.className = "k-status-edge";
  box.append(...content, edge);
  return box;
}
