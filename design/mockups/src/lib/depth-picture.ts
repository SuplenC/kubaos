// A wallpaper with its depth (`docs/design/components/depth-picture.md`): the picture, its plate
// (the city painted out a little past its outline) and its city (the picture drawn through the
// cut-out's alpha), so the city can come off the sky. The Lock draws these three layers with its
// clock between the plate and the city; the wallpaper picker's wall draws them on every picture.
//
// Numbers move it, each a property a move animates: `--push`, from the picture at 1:1 (0) to
// pushed in round the middle of its bottom edge (1), the picture and plate to `scale.push` and the
// city to `scale.near`; `--depth`, how far the city stands off the sky (1) or lies flat on it (0),
// so turning depth on and off moves rather than snaps; and `--city-lead`, a length the city stands
// ahead of its sky as a picture slides past, which a browse eases back to nothing.

import "./depth-picture.css";
import type { Depth } from "./lock.ts";
import { probe, variant } from "./probe.ts";

/** The plate: only its painted band ever differs from the picture, so it comes in with the push. */
export function plateOf(source: string): HTMLImageElement {
  const plate = document.createElement("img");
  plate.className = "k-depth-plate";
  plate.src = source;
  plate.alt = "";
  return plate;
}

/**
 * The city: the picture itself drawn through the cut-out's alpha, so where the city is opaque it
 * is the picture's own pixels and, with nothing pushed, the three layers are the picture exactly.
 * The mask is on the picture and anything filtering the city filters the box round it, so the
 * city's edge blurs with it.
 */
export function cityOf(picture: string, alpha: string): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-depth-city";
  const city = document.createElement("img");
  city.className = "k-depth-city-picture";
  city.src = picture;
  city.alt = "";
  city.style.setProperty("mask-image", `url("${alpha}")`);
  box.append(city);
  return box;
}

export interface DepthPictureOptions {
  readonly picture: string;
  /** Its cut-out and plate; a picture without them is drawn flat, and nothing comes off it. */
  readonly depth?: Depth;
  /** Pushed in, the city off the sky: 1; at 1:1: 0. */
  readonly push?: number;
  /** Depth turned off for this picture: the city lies flat on the sky. */
  readonly flat?: boolean;
  /** Its box, a length's CSS each. */
  readonly width: string;
  readonly height: string;
}

/** The picture and its depth in a box of its own, `object-fit: cover` as a wallpaper is laid. */
export function depthPicture(options: DepthPictureOptions): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-depth-picture";
  box.style.width = options.width;
  box.style.height = options.height;
  box.style.setProperty("--push", String(options.push ?? 0));
  box.style.setProperty("--depth", options.depth && !options.flat ? "1" : "0");
  const picture = document.createElement("img");
  picture.className = "k-depth-photo";
  picture.src = options.picture;
  picture.alt = "";
  box.append(picture);
  if (options.depth) box.append(plateOf(options.depth.plate), cityOf(options.picture, options.depth.cutout));
  box.toggleAttribute("data-flat", !options.depth || (options.flat ?? false));
  return probe(box, { role: "component", name: "DepthPicture", variant: variant(options.depth ? "" : "flat", options.flat && "off"), fixed: { width: false, height: false } });
}
