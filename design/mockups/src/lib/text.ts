// Text in a type style: `kubaos-ui`'s `text::text`. Its box is as wide as the words' advances,
// rounded up to a device pixel (`.k-text` in `lib.css`), and its style's line tall.
//
// The advances are measured as iced lays text out: plain ASCII unkerned, with no ligatures, and
// anything else shaped with kerning. They are measured at 64 times the size, or as many times as a
// font may be large, where the canvas's rounding is under a thousandth of a pixel, in logical
// pixels, which the box scales. A style with tabular figures is measured laid out in the page, as
// the canvas has no font features.

import { probe } from "./probe.ts";
import { length, names } from "./tokens.ts";

/** A type style, by its name in `design/tokens.toml`. */
export type TypeStyle =
  | "clock_rest" | "clock" | "clock_place" | "clock_date" | "hero" | "hero_number" | "display" | "title" | "headline" | "heading" | "subheading"
  | "body_large" | "body" | "body_strong" | "button_large" | "button" | "button_small" | "label"
  | "label_number" | "small" | "caption" | "accent" | "mono" | "keycap" | "mono_large";

const PRECISION = 64;
/** HAZARD: Chromium clamps a font's size to 10000 px, on a canvas and in a page. A measure past it reads short. */
const LARGEST_FONT = 10000;

let canvas: CanvasRenderingContext2D | null = null;
const widths = new Map<string, number>();

/** A style's font as a measure takes it, and how many times its size that is. */
interface Meter {
  readonly font: string;
  readonly times: number;
}

const fonts = new Map<TypeStyle, Meter>();

/**
 * A style's font as the canvas takes it, at as near `PRECISION` times its size as a font may be
 * large: its tokens read on the root, which is at scale 1, so the size is in logical pixels.
 */
function font(style: TypeStyle): Meter {
  const known = fonts.get(style);
  if (known !== undefined) return known;
  const meter = document.createElement("div");
  meter.style.cssText = `position:absolute;visibility:hidden;width:var(--type-${style}-size)`;
  document.documentElement.append(meter);
  const size = meter.getBoundingClientRect().width;
  meter.remove();
  const read = (name: string) =>
    getComputedStyle(document.documentElement).getPropertyValue(`--type-${style}-${name}`).trim();
  const times = Math.min(PRECISION, Math.floor(LARGEST_FONT / size));
  const made = { font: `${read("weight")} ${size * times}px ${read("family")}`, times };
  fonts.set(style, made);
  return made;
}

/** Forgets every width and font measured: the tokens changed. */
export function forgetWidths(): void {
  widths.clear();
  masses.clear();
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
  const shape = shaped(words);
  const width = (tabular(style) ? laidOut(style, words, shape) : measured(style, words, shape)) / font(style).times;
  widths.set(key, width);
  return width;
}

/** Whether `style` draws its figures tabular (OpenType `tnum`). */
function tabular(style: TypeStyle): boolean {
  return getComputedStyle(document.documentElement).getPropertyValue(`--type-${style}-features`).trim() !== "normal";
}

/** `words`' advance at `font(style).times` its size, measured on the canvas. */
function measured(style: TypeStyle, words: string, shape: boolean): number {
  canvas ??= document.createElement("canvas").getContext("2d");
  if (!canvas) throw new Error("no 2D canvas to measure text with");
  canvas.font = font(style).font;
  canvas.fontKerning = shape ? "normal" : "none";
  canvas.textRendering = shape ? "optimizeLegibility" : "optimizeSpeed";
  return canvas.measureText(words).width;
}

/** `words`' advance at `font(style).times` its size, laid out with the style's features. */
function laidOut(style: TypeStyle, words: string, shape: boolean): number {
  const meter = document.createElement("span");
  meter.style.cssText = "position:absolute;visibility:hidden;white-space:pre";
  meter.style.font = font(style).font;
  meter.style.fontFeatureSettings = `var(--type-${style}-features)`;
  meter.style.fontKerning = shape ? "normal" : "none";
  meter.style.fontVariantLigatures = shape ? "normal" : "none";
  meter.textContent = words;
  document.documentElement.append(meter);
  const width = meter.getBoundingClientRect().width;
  meter.remove();
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

/** Where a line's ink sits: its centre's height above the baseline in ems, and how much ink. */
interface Mass {
  readonly height: number;
  readonly ink: number;
}

const masses = new Map<string, Mass>();
/** The size the ink is drawn at to be weighed, in pixels: enough rows to place its centre finely. */
const WEIGHED_AT = 64;
let scale: CanvasRenderingContext2D | null = null;

/**
 * Where `words`' ink sits in `style`: the centre of its mass above the baseline, weighed from the
 * glyphs Geist draws, descenders left out. The eye reads a line as centred when that mass is, so
 * "21:47" sits on its figures' middle and "Continue" nearer its lowercase's.
 */
export function inkMass(style: TypeStyle, words: string): Mass {
  const key = `${style}|${words}`;
  const known = masses.get(key);
  if (known !== undefined) return known;
  scale ??= document.createElement("canvas").getContext("2d", { willReadFrequently: true });
  if (!scale) throw new Error("no 2D canvas to weigh text with");
  scale.font = font(style).font.replace(/[\d.]+px/, `${WEIGHED_AT}px`);
  const width = Math.max(1, Math.ceil(scale.measureText(words).width));
  const height = WEIGHED_AT * 2;
  const baseline = Math.round(WEIGHED_AT * 1.5);
  scale.canvas.width = width;
  scale.canvas.height = height;
  scale.font = font(style).font.replace(/[\d.]+px/, `${WEIGHED_AT}px`);
  scale.fillStyle = "#000";
  scale.textBaseline = "alphabetic";
  scale.fillText(words, 0, baseline);
  const pixels = scale.getImageData(0, 0, width, baseline).data;
  let sum = 0;
  let moment = 0;
  for (let y = 0; y < baseline; y++) {
    let row = 0;
    for (let x = 0; x < width; x++) row += pixels[(y * width + x) * 4 + 3]!;
    sum += row;
    moment += row * (y + 0.5);
  }
  const mass = sum === 0 ? { height: 0, ink: 0 } : { height: (baseline - moment / sum) / WEIGHED_AT, ink: sum / 255 / (WEIGHED_AT * WEIGHED_AT) };
  masses.set(key, mass);
  return mass;
}

/** The size `style` is drawn at, in logical pixels. */
function sizeOf(style: TypeStyle): number {
  const meter = font(style);
  return Number.parseFloat(/([\d.]+)px/.exec(meter.font)![1]!) / meter.times;
}

/** Centres `element`'s line for the eye on `words` in `style`, its words now (`.k-text` in `lib.css`). */
export function centreFor(element: HTMLElement, style: TypeStyle, words: string): void {
  const mass = inkMass(style, words);
  element.style.setProperty("--mass", String(mass.height));
  element.dataset.ink = String(mass.ink * sizeOf(style) ** 2);
  element.dataset.size = String(sizeOf(style));
  element.dataset.mass = String(mass.height);
}

/**
 * Lines side by side in one row share one baseline: each is set on the row's mass, every line's
 * weighed by its ink, so a clock pill's date and time don't sit a pixel apart. A row inside a row
 * takes the outer's.
 */
export function shareRows(root: ParentNode): void {
  const rows = [...root.querySelectorAll<HTMLElement>(".k-row")];
  for (const row of rows.reverse()) {
    const lines: HTMLElement[] = [];
    const walk = (at: Element) => {
      for (const child of at.children) {
        if (child.matches(".k-text")) {
          if (!child.matches("[data-wrapped], [data-clamped='lines']") && (child as HTMLElement).dataset.mass !== undefined) lines.push(child as HTMLElement);
        } else if (child.matches(".k-row, .k-figure") || ![...child.classList].some((name) => name.startsWith("k-"))) {
          // Into the row's own wrappers only: another part (a chip beside a chip) keeps its own line.
          walk(child);
        }
      }
    };
    walk(row);
    if (lines.length < 2) continue;
    let ink = 0;
    let moment = 0;
    for (const line of lines) {
      const weight = Number(line.dataset.ink);
      ink += weight;
      moment += weight * Number(line.dataset.mass) * Number(line.dataset.size);
    }
    if (ink === 0) continue;
    const shared = moment / ink;
    for (const line of lines) line.style.setProperty("--mass", String(shared / Number(line.dataset.size)));
  }
}

/** `words` in `style`, in the colour role `color`. */
export function text(words: string, style: TypeStyle, color = "on_surface"): HTMLElement {
  const element = probe(document.createElement("span"), { role: "text" });
  element.className = "k-text";
  element.textContent = words;
  typeset(element, style);
  element.style.setProperty("--natural", String(advance(style, words)));
  centreFor(element, style, words);
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
  clock_rest: "21:47",
  clock: "21:47",
  clock_place: "東京・月曜日",
  clock_date: " · Monday, 2 November",
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
