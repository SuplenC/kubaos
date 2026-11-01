// An icon tile: `kubaos-ui`'s `icon_tile`, `KubaOS.Components`' `IconTile`
// (`docs/design/components/icon-tile.md`). What an app or a sender looks like in a row: a symbol
// or a picture on a tinted square of one size at the control radius, so a list of apps reads as
// one set whatever each vendor drew. The symbol is centred on a whole device pixel, the odd one
// after it, as `kubaos_ui::snap::centre` places it.
//
// The symbol shows until the picture has loaded and stays when it can't be read, so a row keeps
// the room a picture takes whether or not one arrives.

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
  /** The picture held on its way, whatever its address does: the symbol shows meanwhile. */
  readonly pending?: boolean;
}

/** The picture at `source`, shown once it has loaded; one that can't be read is taken away. */
function image(source: string, className: string, pending: boolean): HTMLImageElement {
  const picture = document.createElement("img");
  picture.className = className;
  picture.alt = "";
  if (!pending) {
    picture.addEventListener("load", () => picture.toggleAttribute("data-loaded", true));
    picture.addEventListener("error", () => picture.remove());
    picture.src = source;
  }
  return picture;
}

export function iconTile(options: TileOptions): HTMLElement {
  const { side, icon: size } = TILE_MEASURES[options.size];
  const tile = document.createElement("span");
  tile.className = "k-icon-tile";
  tile.style.setProperty("--side", `var(--${side})`);
  tile.style.setProperty("--icon", `var(--${size})`);
  const tint = options.danger ? "error" : "surface_container_highest";
  const ink = options.danger ? "on_error" : options.accent ? "primary" : "on_surface_variant";
  tile.style.setProperty("--tint", `var(--color-${tint})`);
  tile.append(icon(options.glyph, "var(--icon)", `var(--color-${ink})`));
  const pending = options.pending ?? false;
  if (options.photo) tile.append(image(options.photo, "k-icon-tile-photo", pending));
  else if (options.picture) tile.append(image(options.picture, "k-icon-tile-picture", pending));
  return probe(tile, { role: "component", name: "IconTile", variant: variant(options.size), fixed: { width: true, height: true } });
}
