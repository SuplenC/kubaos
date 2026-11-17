// An icon tile: `kubaos-ui`'s `icon_tile`, `KubaOS.Components`' `IconTile`
// (`docs/design/components/icon-tile.md`). What an app or a sender looks like in a row: a symbol
// or a picture on a tinted square of one size at the control radius, so a list of apps reads as
// one set whatever each vendor drew. The symbol is centred on the nearest whole device pixel, the
// odd one before it, as `kubaos_ui::snap::centre` places a box centred both ways.
//
// An app's icon is measured once when it loads, and the result is kept per address: the box of
// what it draws, without the margin its theme left round it, and whether that box is a plate (a
// square it covers) or a shape. A plate fills the tile, cut by the tile's corners, so the two make
// one edge; a shape fits `space.s4` inside it. Either is kept whole, never cropped or stretched.
//
// An app with no symbol shows its initial in the symbol's place: the first character of its name,
// upper-cased, in the type style whose symbol box is the tile's, centred on its ink's box. Where an
// app stands as a 16 symbol with no tile (a menu item, a bar's circle), `smallInitial` draws it
// the same way in `label`.
//
// The symbol, or the initial, shows until the picture has loaded and been measured, and stays when
// it can't be read or draws nothing, so a row keeps the room a picture takes whether or not one
// arrives.

import { icon } from "./icon.ts";
import { probe, variant } from "./probe.ts";
import { centreOnInk, text, type TypeStyle } from "./text.ts";
import { number } from "./tokens.ts";

export const TILE_SIZES = ["Sm", "Md", "Lg"] as const;
export type TileSize = (typeof TILE_SIZES)[number];

/** Each size's side and the symbol inside it: the icon of the text it sits beside. */
export const TILE_MEASURES: Record<TileSize, { readonly side: string; readonly icon: string }> = {
  Sm: { side: "size-control_sm", icon: "type-body-icon" },
  Md: { side: "size-control_md", icon: "icon-two_line" },
  Lg: { side: "size-control_lg", icon: "type-headline-icon" },
};

export interface TileOptions {
  /** The symbol the app or the sender names; with none, the initial of `name` takes its place. */
  readonly glyph?: string;
  /** An app's name, whose first character, upper-cased, stands in for a symbol it doesn't name. */
  readonly name?: string;
  readonly size: TileSize;
  /** An app's icon: what it draws fills the tile as a plate, or sits inside it as a shape. */
  readonly picture?: string;
  /** A photo covering the whole tile. */
  readonly photo?: string;
  /** The symbol in the accent: a row in use, the network or the devices connected. */
  readonly accent?: boolean;
  /** On the error colour: a critical notification's. */
  readonly danger?: boolean;
  /** The picture held on its way, whatever its address does: the symbol shows meanwhile. */
  readonly pending?: boolean;
  /**
   * A count in the symbol's place, in `body_strong` (its line the body's symbol's box) in the
   * symbol's colour, 99+ past 99: a popup standing for several from one app.
   */
  readonly count?: number;
}

/** What an app's icon draws, as its measurement found it. */
export interface Drawing {
  /** The box of its pixels more opaque than `icon_tile.ink`, as shares of the picture's sides. */
  readonly box: { readonly x: number; readonly y: number; readonly width: number; readonly height: number };
  /** The picture's width over its height. */
  readonly aspect: number;
  /** The share of the box its pixels more opaque than `icon_tile.opaque` cover. */
  readonly coverage: number;
  /** Square and covered enough to fill the tile; a shape sits inside it. */
  readonly plate: boolean;
}

/**
 * The long side a picture is drawn at to be measured: fine enough that a pixel of the box is under
 * one of the largest tile's, and a quarter of a megapixel to read.
 */
const MEASURING = 256;

/** Each address's drawing, or `null` for a picture that can't be read or draws nothing. */
const drawings = new Map<string, Drawing | null>();

/** The loaded `picture`'s drawing, measured the first time its address is seen. */
function drawingOf(picture: HTMLImageElement): Drawing | null {
  const known = drawings.get(picture.src);
  if (known !== undefined) return known;
  const drawing = read(picture);
  drawings.set(picture.src, drawing);
  return drawing;
}

/** The drawing at `source`, loading it to measure it unless its address was measured already. */
export function measure(source: string): Promise<Drawing | null> {
  const picture = new Image();
  return new Promise((resolve) => {
    picture.addEventListener("load", () => resolve(drawingOf(picture)));
    picture.addEventListener("error", () => resolve(null));
    picture.src = source;
  });
}

/** `picture` drawn at `MEASURING` and its alpha read: the box it draws in and what it covers. */
function read(picture: HTMLImageElement): Drawing | null {
  const { naturalWidth, naturalHeight } = picture;
  if (naturalWidth === 0 || naturalHeight === 0) return null;
  const k = MEASURING / Math.max(naturalWidth, naturalHeight);
  const width = Math.max(1, Math.round(naturalWidth * k));
  const height = Math.max(1, Math.round(naturalHeight * k));
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) return null;
  context.drawImage(picture, 0, 0, width, height);
  let alpha: Uint8ClampedArray;
  try {
    alpha = context.getImageData(0, 0, width, height).data;
  } catch {
    return null;
  }
  const ink = number("icon_tile-ink") * 255;
  let left = width;
  let top = height;
  let right = -1;
  let bottom = -1;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (alpha[(y * width + x) * 4 + 3]! <= ink) continue;
      left = Math.min(left, x);
      right = Math.max(right, x);
      top = Math.min(top, y);
      bottom = Math.max(bottom, y);
    }
  }
  if (right < 0) return null;
  const opaque = number("icon_tile-opaque") * 255;
  let covered = 0;
  for (let y = top; y <= bottom; y++) {
    for (let x = left; x <= right; x++) {
      if (alpha[(y * width + x) * 4 + 3]! > opaque) covered++;
    }
  }
  const boxWidth = right - left + 1;
  const boxHeight = bottom - top + 1;
  const coverage = covered / (boxWidth * boxHeight);
  const square = Math.min(boxWidth, boxHeight) / Math.max(boxWidth, boxHeight) >= number("icon_tile-square");
  return {
    box: { x: left / width, y: top / height, width: boxWidth / width, height: boxHeight / height },
    aspect: naturalWidth / naturalHeight,
    coverage,
    plate: square && coverage >= number("icon_tile-plate"),
  };
}

/**
 * Places `picture` by its drawing, as shares of the box's long side the CSS scales to the room it
 * fits (`lib.css`, `.k-icon-tile-picture`): so the box lands where the tile's size and scale say,
 * at every scale, from one measurement.
 */
function place(picture: HTMLImageElement, drawing: Drawing): void {
  const { box, aspect } = drawing;
  const pictureWidth = Math.min(1, aspect);
  const pictureHeight = Math.min(1, 1 / aspect);
  const long = Math.max(box.width * pictureWidth, box.height * pictureHeight);
  const share = (name: string, value: number) => picture.style.setProperty(`--${name}`, String(value / long));
  share("box-x", box.x * pictureWidth);
  share("box-y", box.y * pictureHeight);
  share("box-width", box.width * pictureWidth);
  share("box-height", box.height * pictureHeight);
  share("picture-width", pictureWidth);
  share("picture-height", pictureHeight);
  picture.toggleAttribute("data-plate", drawing.plate);
}

/** A photo at `source`, shown once it has loaded; one that can't be read is taken away. */
function photo(source: string, pending: boolean): HTMLImageElement {
  const picture = document.createElement("img");
  picture.className = "k-icon-tile-photo";
  picture.alt = "";
  if (!pending) {
    picture.addEventListener("load", () => picture.toggleAttribute("data-loaded", true));
    picture.addEventListener("error", () => picture.remove());
    picture.src = source;
  }
  return picture;
}

/** An app's icon at `source`, shown once it has loaded and been measured, placed by its drawing. */
function appIcon(source: string, pending: boolean): HTMLImageElement {
  const picture = document.createElement("img");
  picture.className = "k-icon-tile-picture";
  picture.alt = "";
  if (!pending) {
    picture.addEventListener("load", () => {
      const drawing = drawingOf(picture);
      if (!drawing) {
        picture.remove();
        return;
      }
      place(picture, drawing);
      picture.toggleAttribute("data-loaded", true);
    });
    picture.addEventListener("error", () => picture.remove());
    picture.src = source;
  }
  return picture;
}

const graphemes = new Intl.Segmenter(undefined, { granularity: "grapheme" });

/** The first character of `name`, upper-cased: one letter, in whatever script the name is in. */
export function initial(name: string): string {
  const [first] = graphemes.segment(name.trim());
  return first ? first.segment.toUpperCase() : "";
}

/** The letter in each size's type style: the style whose symbol box is the tile's symbol's. */
const INITIAL_STYLE = { Sm: "body_strong", Md: "subheading", Lg: "headline" } as const satisfies Record<TileSize, TypeStyle>;

/**
 * `name`'s initial in `style`, in the colour role `color`, its line centred on its ink's box: the
 * letter that stands in for an app's picture and symbol, on a tile or in a 16 symbol's box.
 */
export function initialLetter(name: string, style: TypeStyle, color: string): HTMLElement {
  const letter = text(initial(name), style, color);
  // The letter's line is the symbol's box tall, so it sits where the symbol would, centred on
  // its ink's box: Geist's caps sit high in their line, and a capital's weight (an L's foot)
  // would set it off its middle.
  centreOnInk(letter, style, letter.textContent ?? "");
  return letter;
}

/**
 * `name`'s initial in a 16 symbol's box, in `label`, whose symbol is 16, in `color`, a CSS colour as
 * a symbol beside it takes: an app with no picture where a 16 symbol or picture stands for it, a
 * menu item's or a bar's circle.
 */
export function smallInitial(name: string, color: string): HTMLElement {
  const letter = initialLetter(name, "label", "on_surface");
  letter.style.color = color;
  // The letter stands where a symbol would: the geometry check reads its box as the symbol's, as
  // it reads a drawn one, and the letter as paint inside it.
  delete letter.dataset.probe;
  const box = document.createElement("span");
  box.className = "k-small-initial";
  box.append(letter);
  return probe(box, { role: "icon", name: "initial" });
}

export function iconTile(options: TileOptions): HTMLElement {
  const { side, icon: size } = TILE_MEASURES[options.size];
  const tile = document.createElement("span");
  tile.className = "k-icon-tile";
  tile.style.setProperty("--side", `var(--${side})`);
  tile.style.setProperty("--icon", `var(--${size})`);
  const ink = options.danger ? "on_error" : options.accent ? "primary" : "on_surface_variant";
  // The tint is the stylesheet's, which knows the surface the tile is on; a critical one's is `error`.
  if (options.danger) tile.style.setProperty("--tint", "var(--color-error)");
  if (options.count !== undefined) {
    const words = options.count > 99 ? "99+" : String(options.count);
    const shown = text(words, "body_strong", ink);
    shown.classList.add("k-icon-tile-count");
    tile.append(shown);
  } else if (options.glyph === undefined) {
    const letter = initialLetter(options.name ?? "", INITIAL_STYLE[options.size], ink);
    letter.classList.add("k-icon-tile-initial");
    tile.append(letter);
  } else tile.append(icon(options.glyph, "var(--icon)", `var(--color-${ink})`));
  const pending = options.pending ?? false;
  if (options.photo) tile.append(photo(options.photo, pending));
  else if (options.picture) tile.append(appIcon(options.picture, pending));
  return probe(tile, { role: "component", name: "IconTile", variant: variant(options.size, options.count !== undefined ? "count" : options.glyph === undefined && "initials"), fixed: { width: true, height: true } });
}
