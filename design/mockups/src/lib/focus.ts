// The neon focus on a control: `kubaos-ui`'s `focus_ring` and `focus_ring_on_fill`. The tube is the
// control's own edge, drawn inside its box, in the control's tone, and the glow is that tone too; a
// control filled with its tone's colour shows the tone's hot core just outside its fill instead.
// The glow is `glow` at the middle of every device pixel out from the edge, as
// `kubaos_ui::focus::rings` lays its bands to give. Nothing of it takes layout room.
//
// The glow is one mask, worked out per device pixel from its distance to the edge, in the focus
// colour, so it is rounded to 8 bits once: as twenty translucent elements, one per band, Chromium
// would round each as it composites them, and the glow would draw a tenth dimmer. The mask depends
// on the scale and the control's size, so every ring is drawn again when either changes.

import { settings } from "../site/settings.ts";
import { length, number } from "./tokens.ts";

const rings = new Set<HTMLElement>();

/**
 * The tone a focus tube lights in: the control's own, or the accent for one with none
 * (`docs/DESIGN.md`, "Focus").
 */
export type Tone = "accent" | "cold" | "error" | "success" | "warning" | "sharing";

/** The colour role a tone's tube and glow are drawn in; its core is the role's `_core`. */
export function toneColour(tone: Tone): string {
  return tone === "accent" ? "primary" : tone;
}

/** The glow's opacity `d` logical pixels outside the tube: the near glow and the far one. */
export function glow(d: number): number {
  const fade = (reach: number, opacity: number) => (d >= reach ? 0 : opacity * (1 - d / reach) ** 2);
  const near = fade(length("focus-near"), number("focus-near_opacity"));
  const far = fade(length("focus-far"), number("focus-far_opacity"));
  return 1 - (1 - near) * (1 - far);
}

/** A device pixel's distance outside a rounded box of half sides `hw`, `hh` and corner `r`. */
function outside(x: number, y: number, hw: number, hh: number, r: number): number {
  const qx = Math.abs(x) - (hw - r);
  const qy = Math.abs(y) - (hh - r);
  return Math.hypot(Math.max(qx, 0), Math.max(qy, 0)) + Math.min(Math.max(qx, qy), 0) - r;
}

/** The glow's mask round a box `width` by `height` device pixels with corner `corner`, at `scale`. */
function mask(width: number, height: number, corner: number, scale: number): string {
  const reach = Math.max(0, Math.ceil(length("focus-far") * scale - 1e-3));
  const canvas = document.createElement("canvas");
  canvas.width = width + 2 * reach;
  canvas.height = height + 2 * reach;
  const context = canvas.getContext("2d");
  if (!context || reach === 0) return "none";
  const image = context.createImageData(canvas.width, canvas.height);
  const r = Math.min(corner, width / 2, height / 2);
  for (let py = 0; py < canvas.height; py++) {
    for (let px = 0; px < canvas.width; px++) {
      const d = outside(px + 0.5 - canvas.width / 2, py + 0.5 - canvas.height / 2, width / 2, height / 2, r);
      if (d <= 0) continue;
      image.data[(py * canvas.width + px) * 4 + 3] = Math.round(glow(d / scale) * 255);
    }
  }
  context.putImageData(image, 0, 0);
  return `url(${canvas.toDataURL()})`;
}

/** Draws `ring`'s glow for its control's size now, at `scale`. */
function draw(ring: HTMLElement, scale: number): void {
  const glowing = ring.querySelector<HTMLElement>(".k-glow");
  if (!glowing || !ring.isConnected) return;
  const inset = ring.hasAttribute("data-filled") ? Math.max(1, Math.round(length("stroke-thin") * scale)) : 0;
  const width = ring.offsetWidth + 2 * inset;
  const height = ring.offsetHeight + 2 * inset;
  const corner = Number.parseFloat(getComputedStyle(ring).borderTopLeftRadius) + inset;
  const key = `${width}x${height}r${corner}@${scale}`;
  if (glowing.dataset.drawn === key) return;
  glowing.dataset.drawn = key;
  glowing.style.maskImage = mask(width, height, corner, scale);
}

/**
 * Adds the focus ring in `tone` to `control`, whose corner is the radius token `corner`; `filled`
 * for a control filled with its tone's colour. Its glow is drawn once the control is laid out.
 */
export function focusRing(control: HTMLElement, corner: string, tone: Tone, filled: boolean): void {
  const ring = document.createElement("span");
  ring.className = "k-ring";
  ring.toggleAttribute("data-filled", filled);
  ring.style.setProperty("--corner", `var(--radius-${corner})`);
  ring.style.setProperty("--tube", `var(--color-${toneColour(tone)})`);
  ring.style.setProperty("--core", `var(--color-${toneColour(tone)}_core)`);
  ring.setAttribute("aria-hidden", "true");
  const glowing = document.createElement("span");
  glowing.className = "k-glow";
  const tube = document.createElement("span");
  tube.className = "k-tube";
  ring.append(glowing, tube);
  control.append(ring);
  rings.add(ring);
}

/** Draws every ring on the page for its control's size at the scale picked, and lets go of those
 *  no longer on it. */
export function redrawRings(): void {
  for (const ring of rings) {
    if (ring.isConnected) draw(ring, settings().scale);
    else rings.delete(ring);
  }
}
