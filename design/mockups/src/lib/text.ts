// Text in a type style: `kubaos-ui`'s `text::text`. Its box is as wide as the words' advances,
// rounded up to a device pixel (`.k-text` in `lib.css`), and its style's line tall.
//
// The advances are measured as iced lays text out: plain ASCII unkerned, with no ligatures, and
// anything else shaped with kerning. They are measured at 64 times the size, where the canvas's
// rounding is under a thousandth of a pixel, in logical pixels, which the box scales.

import { probe } from "./probe.ts";
import { length, names } from "./tokens.ts";

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

/**
 * `words` in `style` kept to `lines` lines in the room they are given, the last one ending in an
 * ellipsis where the words don't fit: `kubaos-ui`'s `text::line` for one, QML's `maximumLines`.
 */
export function clamped(words: string, style: TypeStyle, color = "on_surface", lines = 1): HTMLElement {
  const element = text(words, style, color);
  element.dataset.clamped = lines === 1 ? "line" : "lines";
  element.style.setProperty("--lines", String(lines));
  return element;
}

/** A run of a text's characters, from `start` up to `end`, drawn in another colour. */
export interface Run {
  readonly start: number;
  readonly end: number;
}

/**
 * `words` with `runs` in the colour role `marked`: the characters a Command menu query matched, as
 * QML's `markup`. The runs are drawn over the measured words, so the box is the plain text's.
 */
export function marked(words: string, style: TypeStyle, color: string, runs: readonly Run[], marked = "primary"): HTMLElement {
  const element = text(words, style, color);
  const characters = [...words];
  let at = 0;
  const parts: Node[] = [];
  for (const run of [...runs].sort((a, b) => a.start - b.start)) {
    if (run.start > at) parts.push(document.createTextNode(characters.slice(at, run.start).join("")));
    const span = document.createElement("span");
    span.textContent = characters.slice(run.start, run.end).join("");
    span.style.color = `var(--color-${marked})`;
    parts.push(span);
    at = run.end;
  }
  if (at < characters.length) parts.push(document.createTextNode(characters.slice(at).join("")));
  element.replaceChildren(...parts);
  return element;
}

/** Every type style, largest first, then by its line. */
export function styles(): TypeStyle[] {
  const measure = (style: TypeStyle) => [length(`type-${style}-size`), length(`type-${style}-line`)] as const;
  return names("type-")
    .filter((name) => name.endsWith("-size"))
    .map((name) => name.slice("type-".length, -"-size".length) as TypeStyle)
    .sort((a, b) => measure(b)[0] - measure(a)[0] || measure(b)[1] - measure(a)[1]);
}

/** Real words for each style, as the surfaces that use it show them. */
export const STYLE_WORDS: Record<TypeStyle, string> = {
  clock: "21:47",
  hero: "Welcome",
  hero_number: "64%",
  display: "That's it",
  title: "Pick a disk",
  headline: "Erase the disk?",
  heading: "Network",
  subheading: "Couldn't connect",
  body_large: "Everything on this disk will be erased.",
  body: "KubaOS takes the whole disk and encrypts it.",
  body_strong: "Samsung 990 Pro",
  button_large: "Install",
  button: "Continue",
  button_small: "Show log",
  label: "Passphrase",
  label_number: "72%",
  small: "At least 12 characters",
  caption: "Version 0.4",
  accent: "ようこそ",
  mono: "/dev/nvme0n1p2",
  keycap: "Ctrl",
  mono_large: "kubaos update",
};
