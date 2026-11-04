// The lock's colours and the shade behind its words (`docs/design/components/lock.md`): the night
// scheme matugen derives from the wallpaper, painted on the lock whatever the mode, and the least
// `surface` that keeps the words it backs at 4.5:1 over the picture under them.
//
// The shade is worked out from the picture, as the shell works it out from the wallpaper it loads:
// the brightest of the picture under the words (its 98th percentile by luminance, so one lit pixel
// doesn't decide it) with `surface` laid over it at the shade's opacity must hold each of the
// words' colours at 4.5:1. A dark sky needs none; a busy street of signs needs a lot.

import { ratio } from "../site/contrast.ts";
import { paint, type Palette, paletteOf } from "../site/recolour.ts";

/** The roles matugen derives from a wallpaper in its night scheme, by name. */
export type NightScheme = Palette;

/** A box on the screen, in its CSS pixels. */
export interface Box {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** How the screen crops the picture: its scale, and where its top left corner lands. */
export interface Cover {
  readonly scale: number;
  readonly left: number;
  readonly top: number;
}

/** The words of the clock, by colour role: the digits, the kicker's place and its date. */
export const CLOCK_WORDS = ["on_surface", "primary", "on_surface_variant"] as const;

/** The kicker's words alone: its place and its date. */
export const KICKER_WORDS = ["primary", "on_surface_variant"] as const;

/** A colour role the clock's words are drawn in. */
export type Word = (typeof CLOCK_WORDS)[number];

/** The least ratio the clock's words keep over the picture. */
const LEAST = 4.5;

/** The share of the picture under the words that may be brighter than what the shade is set by. */
const BRIGHTEST = 0.98;

/** The lock's palette: `night` over the tokens' dark fallback, its derived roles worked out. */
export function nightPalette(night: NightScheme): Palette {
  return paletteOf(night, "dark");
}

/** Paints `element` in `night`, in dark mode, whatever the page's mode. */
export function paintNight(element: HTMLElement, night: NightScheme): Palette {
  const colours = nightPalette(night);
  element.dataset.mode = "dark";
  paint(element, colours);
  return colours;
}

const pixels = new Map<string, ImageData>();

/** `picture`'s pixels at its own size, blurred by `blur` of its pixels, read once per both. */
function pixelsOf(picture: HTMLImageElement, blur: number): ImageData | null {
  const key = `${picture.src}|${blur}`;
  const known = pixels.get(key);
  if (known) return known;
  if (!picture.complete || picture.naturalWidth === 0) return null;
  const canvas = document.createElement("canvas");
  canvas.width = picture.naturalWidth;
  canvas.height = picture.naturalHeight;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) return null;
  if (blur > 0) context.filter = `blur(${blur}px)`;
  context.drawImage(picture, 0, 0);
  const read = context.getImageData(0, 0, canvas.width, canvas.height);
  pixels.set(key, read);
  return read;
}

const hex = (r: number, g: number, b: number) => `#${[r, g, b].map((v) => Math.round(v).toString(16).padStart(2, "0")).join("")}`;

/** `over` laid at `alpha` on `under`, blended per channel as the browser draws it. */
function laid(over: string, under: readonly [number, number, number], alpha: number): string {
  const channel = (at: number, i: number) => alpha * Number.parseInt(over.slice(at, at + 2), 16) + (1 - alpha) * under[i]!;
  return hex(channel(1, 0), channel(3, 1), channel(5, 2));
}

/** How the picture is seen under the words: blurred by `blur` screen pixels, dimmed by `dim`. */
export interface Seen {
  readonly blur: number;
  /** The opacity of `scrim` over it. */
  readonly dim: number;
}

/**
 * The shade's opacity behind `words` in `box` on the screen, in `colours`, over `picture` cropped
 * as `cover` says and seen as `seen` says: the least, in hundredths, that keeps every word at 4.5:1
 * over the picture's brightest under them.
 */
export function shadeFor(picture: HTMLImageElement, colours: Palette, words: readonly Word[], box: Box, cover: Cover, seen: Seen = { blur: 0, dim: 0 }): number {
  const data = pixelsOf(picture, Math.round(seen.blur / cover.scale));
  const surface = colours.surface;
  if (!data || !surface) return 0;
  const step = Math.max(1, Math.round(2 / cover.scale));
  const toPicture = (v: number, origin: number) => Math.round((v - origin) / cover.scale);
  const [x0, x1] = [toPicture(box.x, cover.left), toPicture(box.x + box.width, cover.left)].map((v) => Math.min(data.width - 1, Math.max(0, v))) as [number, number];
  const [y0, y1] = [toPicture(box.y, cover.top), toPicture(box.y + box.height, cover.top)].map((v) => Math.min(data.height - 1, Math.max(0, v))) as [number, number];
  const found: { lum: number; rgb: [number, number, number] }[] = [];
  for (let y = y0; y <= y1; y += step) {
    for (let x = x0; x <= x1; x += step) {
      const at = (y * data.width + x) * 4;
      const rgb: [number, number, number] = [data.data[at]!, data.data[at + 1]!, data.data[at + 2]!];
      found.push({ lum: 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2], rgb });
    }
  }
  if (found.length === 0) return 0;
  found.sort((a, b) => a.lum - b.lum);
  const brightest = found[Math.min(found.length - 1, Math.floor(found.length * BRIGHTEST))]!.rgb;
  const scrim = colours.scrim ?? "#000000";
  const dimmed = laid(scrim, brightest, seen.dim);
  const bright: [number, number, number] = [1, 3, 5].map((at) => Number.parseInt(dimmed.slice(at, at + 2), 16)) as [number, number, number];
  const holds = (alpha: number) => words.every((role) => ratio(colours[role] ?? "#ffffff", laid(surface, bright, alpha)) >= LEAST);
  for (let hundredths = 0; hundredths <= 100; hundredths++) {
    if (holds(hundredths / 100)) return hundredths / 100;
  }
  return 1;
}

/** The normal distribution's cumulative share under `z` (Abramowitz and Stegun's 7.1.26). */
function normal(z: number): number {
  const x = Math.abs(z) / Math.SQRT2;
  const t = 1 / (1 + 0.3275911 * x);
  const erf = 1 - (((((1.061405429 * t - 1.453152027) * t + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t) * Math.exp(-x * x);
  return z >= 0 ? (1 + erf) / 2 : (1 - erf) / 2;
}

/**
 * How much of a box blurred by `soft` (a standard deviation) is left at the corners of the words
 * it backs, when it reaches `spread` past them across and down: the share its opacity is raised by
 * so the words keep their contrast where it is weakest.
 */
export function backingCover(spread: { readonly x: number; readonly y: number }, soft: number): number {
  if (soft <= 0) return 1;
  return normal(spread.x / soft) * normal(spread.y / soft);
}
