// Text in a type style: `kubaos-ui`'s `text::text`. Its box is as wide as the words' advances,
// rounded up to a device pixel (`.k-text` in `lib.css`), and its style's line tall.
//
// The advances are measured as iced lays text out: plain ASCII unkerned, with no ligatures, and
// anything else shaped with kerning. They are measured at 64 times the size, where the canvas's
// rounding is under a thousandth of a pixel, in logical pixels, which the box scales.

import { probe } from "./probe.ts";

/** A type style, by its name in `design/tokens.toml`. */
export type TypeStyle =
  | "clock" | "hero" | "hero_number" | "display" | "title" | "headline" | "heading" | "subheading"
  | "body_large" | "body" | "body_strong" | "button_large" | "button" | "button_small" | "label"
  | "label_number" | "small" | "caption" | "accent" | "mono" | "keycap" | "mono_large";

const PRECISION = 64;

let canvas: CanvasRenderingContext2D | null = null;
const widths = new Map<string, number>();

const fonts = new Map<TypeStyle, string>();

/**
 * A style's font as the canvas takes it, at `PRECISION` times its size: its tokens read on the
 * root, which is at scale 1, so the size is in logical pixels.
 */
function font(style: TypeStyle): string {
  const known = fonts.get(style);
  if (known !== undefined) return known;
  const meter = document.createElement("div");
  meter.style.cssText = `position:absolute;visibility:hidden;width:var(--type-${style}-size)`;
  document.documentElement.append(meter);
  const size = meter.getBoundingClientRect().width;
  meter.remove();
  const read = (name: string) =>
    getComputedStyle(document.documentElement).getPropertyValue(`--type-${style}-${name}`).trim();
  const made = `${read("weight")} ${size * PRECISION}px ${read("family")}`;
  fonts.set(style, made);
  return made;
}

/** Forgets every width and font measured: the tokens changed. */
export function forgetWidths(): void {
  widths.clear();
  fonts.clear();
}

/** Whether iced shapes `words`: anything but plain ASCII. */
export function shaped(words: string): boolean {
  return /[^\x00-\x7f]/.test(words);
}

/** The width `words` need in `style`, in logical pixels. */
export function advance(style: TypeStyle, words: string): number {
  const key = `${style}|${words}`;
  const known = widths.get(key);
  if (known !== undefined) return known;
  canvas ??= document.createElement("canvas").getContext("2d");
  if (!canvas) throw new Error("no 2D canvas to measure text with");
  canvas.font = font(style);
  const shape = shaped(words);
  canvas.fontKerning = shape ? "normal" : "none";
  canvas.textRendering = shape ? "optimizeLegibility" : "optimizeSpeed";
  const width = canvas.measureText(words).width / PRECISION;
  widths.set(key, width);
  return width;
}

/** Sets `element`'s font to `style`'s. */
export function typeset(element: HTMLElement, style: TypeStyle): void {
  for (const [property, token] of [
    ["--t-size", "size"],
    ["--t-line", "line"],
    ["--t-weight", "weight"],
    ["--t-family", "family"],
    ["--t-features", "features"],
  ] as const) {
    element.style.setProperty(property, `var(--type-${style}-${token})`);
  }
}

/** `words` in `style`, in the colour role `color`. */
export function text(words: string, style: TypeStyle, color = "on_surface"): HTMLElement {
  const element = probe(document.createElement("span"), { role: "text" });
  element.className = "k-text";
  element.textContent = words;
  typeset(element, style);
  element.style.setProperty("--natural", String(advance(style, words)));
  element.style.color = `var(--color-${color})`;
  element.toggleAttribute("data-shaped", shaped(words));
  return element;
}

/**
 * `words` in `style`, in the colour role `color`, broken between words onto as many lines as the
 * room they are given needs, each the style's line tall: a note, a tooltip's body.
 */
export function wrapped(words: string, style: TypeStyle, color = "on_surface"): HTMLElement {
  const element = text(words, style, color);
  element.dataset.wrapped = "";
  return element;
}
