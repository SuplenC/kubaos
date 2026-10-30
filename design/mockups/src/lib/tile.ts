// An icon tile: `kubaos-ui`'s `icon_tile`, `KubaOS.Components`' `IconTile`
// (`docs/design/components/icon-tile.md`). What an app or a sender looks like in a row: a symbol
// or a picture on a tinted square of one size at the control radius, so a list of apps reads as
// one set whatever each vendor drew. The symbol is centred on a whole device pixel, the odd one
// after it, as `kubaos_ui::snap::centre` places it.

import { icon } from "./icon.ts";
import { probe, variant } from "./probe.ts";

export const TILE_SIZES = ["Sm", "Md", "Lg"] as const;
export type TileSize = (typeof TILE_SIZES)[number];

/** Each size's side and the symbol inside it: the icon of the text it sits beside. */
export const TILE_MEASURES: Record<TileSize, { readonly side: string; readonly icon: string }> = {
  Sm: { side: "size-control_sm", icon: "type-body-icon" },
  Md: { side: "size-control_md", icon: "icon-two_line" },
  Lg: { side: "size-control_lg", icon: "type-headline-icon" },
};

export interface TileOptions {
  readonly glyph: string;
  readonly size: TileSize;
  /** An app's icon, inset to the icon size, keeping its own margins. */
  readonly picture?: string;
  /** A photo covering the whole tile. */
  readonly photo?: string;
  /** The symbol in the accent: the one tile of a list that comes first. */
  readonly accent?: boolean;
  /** On the error colour: a critical notification's. */
  readonly danger?: boolean;
}

function image(source: string, className: string): HTMLImageElement {
  const picture = document.createElement("img");
  picture.className = className;
  picture.src = source;
  picture.alt = "";
  return picture;
}

export function iconTile(options: TileOptions): HTMLElement {
  const { side, icon: size } = TILE_MEASURES[options.size];
  const tile = document.createElement("span");
  tile.className = "k-tile";
  tile.style.setProperty("--side", `var(--${side})`);
  tile.style.setProperty("--icon", `var(--${size})`);
  const tint = options.danger ? "error" : "surface_container_highest";
  const ink = options.danger ? "on_error" : options.accent ? "primary" : "on_surface_variant";
  tile.style.setProperty("--tint", `var(--color-${tint})`);
  if (options.photo) tile.append(image(options.photo, "k-tile-photo"));
  else if (options.picture) tile.append(image(options.picture, "k-tile-picture"));
  else tile.append(icon(options.glyph, "var(--icon)", `var(--color-${ink})`));
  return probe(tile, { role: "component", name: "IconTile", variant: variant(options.size), fixed: { width: true, height: true } });
}
