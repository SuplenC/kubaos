// The lock's colours, drawn from its wallpaper (`docs/design/components/lock.md`): the night scheme
// matugen derives from it, painted on the lock whatever the mode, and the rest clock's tint, a light
// colour of the picture behind its digits, which the kicker above them takes too.
//
// The tint is worked out from the picture, as the shell works it out from the wallpaper it loads:
// it takes the colour the picture shows most strongly under the digits and lifts it, so the clock
// reads as part of the picture rather than as words laid on it.

import { bytesToLch, toHex } from "../site/derive.ts";
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

/** The OKLCH lightness the rest digits' tint is set to: light, as a sky's highlight is. */
export const TINT_LIGHTNESS = 0.84;

/** How much the tint raises the picture's chroma, so a hazy sky still gives a colour. */
const TINT_CHROMA = 1.2;

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

const drawn = new Map<string, CanvasRenderingContext2D>();
/**
 * `picture` drawn at its own size on a canvas whose pixels are read, once: each tint reads only the
 * pixels under its digits, a few thousand, not the whole picture's millions.
 */
function canvasOf(picture: HTMLImageElement): CanvasRenderingContext2D | null {
  const known = drawn.get(picture.src);
  if (known) return known;
  if (!picture.complete || picture.naturalWidth === 0) return null;
  const canvas = document.createElement("canvas");
  canvas.width = picture.naturalWidth;
  canvas.height = picture.naturalHeight;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) return null;
  context.drawImage(picture, 0, 0);
  drawn.set(picture.src, context);
  return context;
}

/** The picture's pixels under a box on the screen: the first and last column and row, inclusive. */
interface Region {
  readonly x0: number;
  readonly x1: number;
  readonly y0: number;
  readonly y1: number;
}

/** The picture's pixels under `box` on a screen that crops it as `cover` says. */
function regionUnder(context: CanvasRenderingContext2D, box: Box, cover: Cover): Region {
  const { width, height } = context.canvas;
  const toPicture = (v: number, origin: number) => Math.round((v - origin) / cover.scale);
  const [x0, x1] = [toPicture(box.x, cover.left), toPicture(box.x + box.width, cover.left)].map((v) => Math.min(width - 1, Math.max(0, v))) as [number, number];
  const [y0, y1] = [toPicture(box.y, cover.top), toPicture(box.y + box.height, cover.top)].map((v) => Math.min(height - 1, Math.max(0, v))) as [number, number];
  return { x0, x1, y0, y1 };
}

/**
 * Hands `each` the picture's pixels in `region`, as their 8 bit channels: every other one of the
 * picture's own, so what is read is the same at every display scale.
 */
function under(context: CanvasRenderingContext2D, { x0, x1, y0, y1 }: Region, each: (r: number, g: number, b: number) => void): void {
  const step = 2;
  const read = context.getImageData(x0, y0, x1 - x0 + 1, y1 - y0 + 1);
  for (let y = y0; y <= y1; y += step) {
    for (let x = x0; x <= x1; x += step) {
      const at = ((y - y0) * read.width + (x - x0)) * 4;
      each(read.data[at]!, read.data[at + 1]!, read.data[at + 2]!);
    }
  }
}

/**
 * The rest digits' colour over `picture` cropped as `cover` says, the digits in `box` on the
 * screen: the colour the picture shows most strongly under them, lifted. Each pixel's OKLCH hue
 * counts by its chroma squared, so the signs and the glow of a sky decide it and grey haze and
 * dark walls barely do; the chroma is the pixels' own, weighted the same way. That hue and chroma,
 * the chroma raised by `TINT_CHROMA`, are set to `TINT_LIGHTNESS` and fitted into sRGB by lowering
 * the chroma alone. Null until the picture has arrived.
 */
export function tintFor(picture: HTMLImageElement, box: Box, cover: Cover): string | null {
  const context = canvasOf(picture);
  if (!context) return null;
  // The same picture under the same box is the same tint: read once, as a lock is placed again on
  // every resize and many locks show one picture.
  const key = [picture.src, ...[box.x, box.y, box.width, box.height, cover.left, cover.top].map((v) => Math.round(v * 2)), cover.scale.toFixed(5)].join("|");
  const known = tints.get(key);
  if (known !== undefined) return known;
  // Locks of one picture at other sizes read the same pixels under their digits, the clock standing
  // at the same share of the screen: those pixels' tint is worked out once.
  const region = regionUnder(context, box, cover);
  const at = [picture.src, region.x0, region.x1, region.y0, region.y1].join("|");
  const made = regionTints.get(at) ?? tintOf(context, region);
  regionTints.set(at, made);
  tints.set(key, made);
  return made;
}

const tints = new Map<string, string>();
const regionTints = new Map<string, string>();

/** The tint, worked out from the pixels in `region`. */
function tintOf(context: CanvasRenderingContext2D, region: Region): string {
  let [a, b, weight, chroma] = [0, 0, 0, 0];
  under(context, region, (red, green, blue) => {
    const { c, h } = bytesToLch(red, green, blue);
    const squared = c * c;
    a += squared * Math.cos((h * Math.PI) / 180);
    b += squared * Math.sin((h * Math.PI) / 180);
    weight += squared;
    chroma += squared * c;
  });
  if (weight === 0) return toHex({ l: TINT_LIGHTNESS, c: 0, h: 0 });
  const hue = ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360;
  return toHex({ l: TINT_LIGHTNESS, c: (chroma / weight) * TINT_CHROMA, h: hue });
}
