// The lock's colours, drawn from its wallpaper (`docs/design/components/lock.md`): the night scheme
// matugen derives from it, painted on the lock whatever the mode, and the rest clock's tint, a light
// colour of the picture behind its digits, which the kicker above them takes too.
//
// The tint is worked out from the picture, as the shell works it out from the wallpaper it loads:
// it takes the colour the picture shows most strongly under the digits and lifts it, so the clock
// reads as part of the picture rather than as words laid on it.

import { toHex, toLch } from "../site/derive.ts";
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

const pixels = new Map<string, ImageData>();

/** `picture`'s pixels at its own size, read once. */
function pixelsOf(picture: HTMLImageElement): ImageData | null {
  const known = pixels.get(picture.src);
  if (known) return known;
  if (!picture.complete || picture.naturalWidth === 0) return null;
  const canvas = document.createElement("canvas");
  canvas.width = picture.naturalWidth;
  canvas.height = picture.naturalHeight;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) return null;
  context.drawImage(picture, 0, 0);
  const read = context.getImageData(0, 0, canvas.width, canvas.height);
  pixels.set(picture.src, read);
  return read;
}

const hex = (r: number, g: number, b: number) => `#${[r, g, b].map((v) => Math.round(v).toString(16).padStart(2, "0")).join("")}`;

/**
 * The picture's pixels under `box` on the screen, as `#rrggbb`: every other one of the picture's
 * own, so what is read is the same at every display scale.
 */
function under(data: ImageData, box: Box, cover: Cover): string[] {
  const step = 2;
  const toPicture = (v: number, origin: number) => Math.round((v - origin) / cover.scale);
  const [x0, x1] = [toPicture(box.x, cover.left), toPicture(box.x + box.width, cover.left)].map((v) => Math.min(data.width - 1, Math.max(0, v))) as [number, number];
  const [y0, y1] = [toPicture(box.y, cover.top), toPicture(box.y + box.height, cover.top)].map((v) => Math.min(data.height - 1, Math.max(0, v))) as [number, number];
  const found: string[] = [];
  for (let y = y0; y <= y1; y += step) {
    for (let x = x0; x <= x1; x += step) {
      const at = (y * data.width + x) * 4;
      found.push(hex(data.data[at]!, data.data[at + 1]!, data.data[at + 2]!));
    }
  }
  return found;
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
  const data = pixelsOf(picture);
  if (!data) return null;
  // The same picture under the same box is the same tint: read once, as a lock is placed again on
  // every resize and many locks show one picture.
  const key = [picture.src, ...[box.x, box.y, box.width, box.height, cover.left, cover.top].map((v) => Math.round(v * 2)), cover.scale.toFixed(5)].join("|");
  const known = tints.get(key);
  if (known !== undefined) return known;
  const made = tintOf(data, box, cover);
  tints.set(key, made);
  return made;
}

const tints = new Map<string, string>();

/** The tint, worked out from the pixels under `box`. */
function tintOf(data: ImageData, box: Box, cover: Cover): string {
  let [a, b, weight, chroma] = [0, 0, 0, 0];
  for (const colour of under(data, box, cover)) {
    const { c, h } = toLch(colour);
    const squared = c * c;
    a += squared * Math.cos((h * Math.PI) / 180);
    b += squared * Math.sin((h * Math.PI) / 180);
    weight += squared;
    chroma += squared * c;
  }
  if (weight === 0) return toHex({ l: TINT_LIGHTNESS, c: 0, h: 0 });
  const hue = ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360;
  return toHex({ l: TINT_LIGHTNESS, c: (chroma / weight) * TINT_CHROMA, h: hue });
}
