// A status message's box and its rail (`docs/DESIGN.md`, "States": fixed signals are always lit),
// shared by `Banner` and `ErrorBlock`. The box lies flat in the page: a fill, the sunken soft
// hairline (`.k-sunken`), the content radius and `space.s16` inside. Its tone is a rail of neon
// down its start side, inside the box from the top padding to the bottom one with round ends, the
// tabs' and the collapsible's rail: the tone's core with its near glow at night while the signs are
// on, glass while they are off, and the tone with no glow by day. `feedback.css` draws both.

import "./feedback.css";

/** The tones a status message lights its rail in, by their colour role. */
export type StatusTone = "primary" | "warning" | "error";

/** A status message's box in `tone`, filled with `fill` (a colour's CSS), holding `content`. */
export function statusBox(tone: StatusTone, fill: string, ...content: Element[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-status k-sunken";
  box.style.setProperty("--status-fill", fill);
  box.style.setProperty("--status-tone", `var(--color-${tone})`);
  box.style.setProperty("--status-core", `var(--color-${tone}_core)`);
  box.style.setProperty("--status-glass", `var(--color-${tone}_glass)`);
  const rail = document.createElement("span");
  rail.className = "k-status-rail";
  box.append(...content, rail);
  return box;
}
