// The neon focus on a control: `kubaos-ui`'s `focus_ring` and `focus_ring_on_fill`. One line inside
// the control's edge, at its corner: the tube in the control's tone, or on a control filled with
// its tone's colour, the tone's core inside the fill's edge. The glow outside is that tone too, and
// it is all that reaches past the box. Focused has two looks, which the scene picks: lit, the neon
// burning with its core half way to white, at night at rest; and unlit, the line as unlit glass
// (`_unlit`, the tone toward black) with no glow, in daylight always and at night while a sequence
// has the lights out.
// The glow is `glow` at the middle of every device pixel out from the edge, as
// `kubaos_ui::focus::rings` lays its bands to give. Nothing of it takes layout room.
//
// The glow is one mask, worked out per device pixel from its distance to the edge, in the focus
// colour, so it is rounded to 8 bits once: as twenty translucent elements, one per band, Chromium
// would round each as it composites them, and the glow would draw a tenth dimmer. The mask depends
// on the scale and the control's size, so every ring is drawn again when either changes.
//
// A control whose content is drawn in its own tone (a Text button's label and icon) lights that
// content as a tube too: a copy of it in the tone's core over it, with the same near and far glow
// round its letters and strokes, scaled by the label's stroke over the tube's width. It burns on
// the ring's own `--lit`, so it lights, strikes and goes out with the line; unlit, the copy is out
// and the content is its own tone. Its glow is a mask worked out per device pixel from a distance
// field of the content drawn on a canvas.

import { settings } from "../site/settings.ts";
import type { TypeStyle } from "./text.ts";
import { length, number } from "./tokens.ts";

const rings = new Set<HTMLElement>();

/**
 * The tone a focus tube lights in: the control's own, or the accent for one with none
 * (`docs/DESIGN.md`, "Focus").
 */
export type Tone = "accent" | "cold" | "error" | "success" | "warning" | "sharing";

/**
 * The colour role a tone's tube and glow are drawn in; its core is the role's `_core` at night,
 * `_unlit` in daylight.
 */
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

/**
 * Squared distances along one line of `n` cells from `f`'s, in place: the lower envelope of the
 * parabolas rooted at each cell (Felzenszwalb and Huttenlocher).
 */
function envelope(f: Float64Array, n: number): void {
  const d = new Float64Array(n);
  const v = new Int32Array(n);
  const z = new Float64Array(n + 1);
  let k = 0;
  v[0] = 0;
  z[0] = -Infinity;
  z[1] = Infinity;
  for (let q = 1; q < n; q++) {
    if (f[q] === Infinity) continue;
    if (f[v[k]!] === Infinity) {
      v[k] = q;
      continue;
    }
    let s = (f[q]! + q * q - (f[v[k]!]! + v[k]! * v[k]!)) / (2 * q - 2 * v[k]!);
    while (s <= z[k]!) {
      k--;
      s = (f[q]! + q * q - (f[v[k]!]! + v[k]! * v[k]!)) / (2 * q - 2 * v[k]!);
    }
    k++;
    v[k] = q;
    z[k] = s;
    z[k + 1] = Infinity;
  }
  k = 0;
  for (let q = 0; q < n; q++) {
    while (z[k + 1]! < q) k++;
    const at = v[k]!;
    d[q] = f[at] === Infinity ? Infinity : (q - at) * (q - at) + f[at]!;
  }
  f.set(d);
}

/** The offset of `element` inside `box`, through its offset parents. */
function offsetIn(element: HTMLElement, box: HTMLElement): { left: number; top: number } {
  let left = 0;
  let top = 0;
  for (let at: HTMLElement | null = element; at && at !== box; at = at.offsetParent as HTMLElement | null) {
    left += at.offsetLeft;
    top += at.offsetTop;
  }
  return { left, top };
}

/** Draws `lit`'s content on `context`, `reach` in from its corner, as its glyphs cover pixels. */
async function rasterise(lit: HTMLElement, context: CanvasRenderingContext2D, reach: number): Promise<void> {
  for (const words of lit.querySelectorAll<HTMLElement>(".k-text")) {
    const style = getComputedStyle(words);
    const { left, top } = offsetIn(words, lit);
    context.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
    context.fontKerning = words.hasAttribute("data-shaped") ? "normal" : "none";
    const metrics = context.measureText(words.textContent ?? "");
    const ascent = metrics.fontBoundingBoxAscent;
    const line = Number.parseFloat(style.lineHeight);
    // CSS's half-leading: the font's box in the middle of the line, the baseline its ascent down.
    const baseline = top + (line - ascent - metrics.fontBoundingBoxDescent) / 2 + ascent;
    context.fillText(words.textContent ?? "", reach + left, reach + baseline);
  }
  for (const symbol of lit.querySelectorAll<SVGSVGElement>(".k-icon")) {
    const slot = symbol.parentElement;
    if (!slot) continue;
    const style = getComputedStyle(symbol);
    const size = Number.parseFloat(style.width);
    const { left, top } = offsetIn(slot, lit);
    const copy = symbol.cloneNode(true) as SVGSVGElement;
    copy.removeAttribute("style");
    copy.removeAttribute("class");
    copy.setAttribute("xmlns", "http://www.w3.org/2000/svg");
    copy.setAttribute("width", String(size));
    copy.setAttribute("height", String(size));
    copy.setAttribute("stroke", "#000");
    copy.setAttribute("stroke-width", style.strokeWidth);
    const picture = new Image();
    picture.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(copy.outerHTML)}`;
    await picture.decode();
    context.drawImage(picture, reach + left, reach + top + Number.parseFloat(style.top), size, size);
  }
}

/**
 * The glow's mask round `lit`'s content, `reach` device pixels past its box on every side, at
 * `scale`, its lengths scaled by `stretch`: each pixel's glow at its distance from the nearest
 * pixel the content covers, less half a pixel to that pixel's edge.
 */
async function contentMask(lit: HTMLElement, reach: number, scale: number, stretch: number): Promise<string> {
  const canvas = document.createElement("canvas");
  canvas.width = lit.offsetWidth + 2 * reach;
  canvas.height = lit.offsetHeight + 2 * reach;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context || reach === 0) return "none";
  await rasterise(lit, context, reach);
  const { width, height } = canvas;
  const covered = context.getImageData(0, 0, width, height).data;
  const field = new Float64Array(width * height);
  for (let i = 0; i < field.length; i++) field[i] = covered[i * 4 + 3]! >= 128 ? 0 : Infinity;
  const column = new Float64Array(height);
  for (let x = 0; x < width; x++) {
    for (let y = 0; y < height; y++) column[y] = field[y * width + x]!;
    envelope(column, height);
    for (let y = 0; y < height; y++) field[y * width + x] = column[y]!;
  }
  for (let y = 0; y < height; y++) envelope(field.subarray(y * width, (y + 1) * width), width);
  const image = context.createImageData(width, height);
  for (let i = 0; i < field.length; i++) {
    if (field[i] === 0) continue;
    const d = Math.sqrt(field[i]!) - 0.5;
    image.data[i * 4 + 3] = Math.round(glow(d / (scale * stretch)) * 255);
  }
  context.putImageData(image, 0, 0);
  return `url(${canvas.toDataURL()})`;
}

/** How much longer a label's glow is than the line's: its style's symbol stroke over the tube. */
function stretch(style: TypeStyle): number {
  return (length(`type-${style}-icon`) * number("stroke-symbol")) / 24 / length("focus-tube");
}

/** Draws the glow round `ring`'s lit content for its size now, at `scale`. */
function drawContent(ring: HTMLElement, scale: number): void {
  const lit = ring.querySelector<HTMLElement>(".k-lit");
  const glowing = lit?.querySelector<HTMLElement>(".k-lit-glow");
  if (!lit || !glowing) return;
  const longer = stretch(lit.dataset.style as TypeStyle);
  const reach = Math.max(0, Math.ceil(length("focus-far") * longer * scale - 1e-3));
  const key = `${lit.offsetWidth}x${lit.offsetHeight}+${reach}@${scale}`;
  if (glowing.dataset.drawn === key) return;
  glowing.dataset.drawn = key;
  glowing.style.inset = `${-reach}px`;
  void contentMask(lit, reach, scale, longer).then((made) => {
    if (glowing.dataset.drawn === key) glowing.style.maskImage = made;
  });
}

/** Draws `ring`'s glow for its control's size now, at `scale`. */
function draw(ring: HTMLElement, scale: number): void {
  const glowing = ring.querySelector<HTMLElement>(".k-glow");
  if (!glowing || !ring.isConnected) return;
  drawContent(ring, scale);
  const width = ring.offsetWidth;
  const height = ring.offsetHeight;
  const corner = Number.parseFloat(getComputedStyle(ring).borderTopLeftRadius);
  const key = `${width}x${height}r${corner}@${scale}`;
  if (glowing.dataset.drawn === key) return;
  glowing.dataset.drawn = key;
  glowing.style.maskImage = mask(width, height, corner, scale);
}

/** A control's content drawn in its own tone, which lights as a tube with the line. */
export interface LitContent {
  readonly content: HTMLElement;
  /** The label's type style, whose symbol stroke scales the glow. */
  readonly style: TypeStyle;
}

/**
 * Adds the focus ring in `tone` to `control`, whose corner is the radius token `corner`; `filled`
 * for a control filled with its tone's colour; `lit`, the content drawn in its tone, to light as
 * a tube with it. Its glows are drawn once the control is laid out.
 */
export function focusRing(control: HTMLElement, corner: string, tone: Tone, filled: boolean, lit?: LitContent): void {
  const ring = document.createElement("span");
  ring.className = "k-ring";
  ring.toggleAttribute("data-filled", filled);
  ring.style.setProperty("--corner", `var(--radius-${corner})`);
  const colour = toneColour(tone);
  ring.style.setProperty("--tube-lit", `var(--color-${colour})`);
  ring.style.setProperty("--core-lit", `var(--color-${colour}_core)`);
  // Unlit glass over the control's own container: on a fill, the unlit core is the line itself.
  ring.style.setProperty("--tube-unlit", `rgb(from var(--color-${colour}_unlit) r g b / var(--focus-glass))`);
  ring.style.setProperty("--core-unlit", `var(--color-${colour}_unlit)`);
  ring.setAttribute("aria-hidden", "true");
  const glowing = document.createElement("span");
  glowing.className = "k-glow";
  const tube = document.createElement("span");
  tube.className = "k-tube";
  ring.append(glowing, tube);
  if (lit) {
    // A picture of the content, not more of it: no probe, nothing read out twice.
    const copy = lit.content.cloneNode(true) as HTMLElement;
    for (const each of [copy, ...copy.querySelectorAll<HTMLElement>("[data-probe]")]) delete each.dataset.probe;
    const burning = document.createElement("span");
    burning.className = "k-lit";
    burning.dataset.style = lit.style;
    const around = document.createElement("span");
    around.className = "k-lit-glow";
    burning.append(around, copy);
    ring.append(burning);
  }
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
