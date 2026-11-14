// A picture that never moves what is around it: `kubaos-ui`'s `media::media`, `KubaOS.Components`'
// `Media` (`docs/design/components/media.md`). A box as wide as its room and as tall as an
// `[aspect]` token makes it, down to the 4 px grid, so boxes side by side whose widths differ by a
// device pixel come out one height. The picture covers it, cut at the corner its place gives it.
//
// Until the picture has loaded the box holds a block skeleton of its own shape, breathing; a
// picture that can't be read leaves the skeleton held still, so the box is the same before, after
// and without the picture. Dimmed (in a disabled `PictureChoice`), the picture is at
// `opacity.disabled_content` and the placeholder is the disabled container.

import { probe } from "./probe.ts";
import type { Corner } from "./radius.ts";
import { wrapped } from "./text.ts";

/** Where the picture is: on its way, there, or unreadable. */
export type Arrival = "loading" | "loaded" | "broken";

export interface MediaOptions {
  /** The picture's address; none holds the placeholder, loading. */
  readonly source?: string;
  /** Where the picture is, whatever its address does: a sample held loading or broken. */
  readonly arrival?: Arrival;
  /** The corner its place gives it: the control radius in a picker, the row radius in a card. */
  readonly corner: Corner;
  /** The `[aspect]` token its box keeps. */
  readonly aspect?: "wallpaper";
  /** A line under it, `small` in `on_surface_variant`, `space.s8` below, wrapping in its width. */
  readonly caption?: string;
  readonly dimmed?: boolean;
}

/** The placeholder: a block skeleton in the box's own shape, breathing, or held still. */
function placeholder(still: boolean): HTMLElement {
  const shape = document.createElement("span");
  shape.className = "k-skeleton k-media-placeholder";
  shape.dataset.bone = "block";
  shape.toggleAttribute("data-still", still);
  return shape;
}

/** Shows the picture in `box` where `arrival` says it is. */
function arrive(box: HTMLElement, picture: HTMLImageElement | null, arrival: Arrival): void {
  box.dataset.arrival = arrival;
  box.querySelector(".k-media-placeholder")?.toggleAttribute("data-still", arrival === "broken");
  if (picture) picture.hidden = arrival !== "loaded";
}

/** Shows `room`, a media's, where `arrival` says its picture is: a sample that loads again. */
export function settle(room: HTMLElement, arrival: Arrival): void {
  const box = room.querySelector<HTMLElement>(".k-media-box");
  if (box) arrive(box, box.querySelector("img"), arrival);
}

export function media(options: MediaOptions): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-media";
  room.style.setProperty("--aspect", `var(--aspect-${options.aspect ?? "wallpaper"})`);
  const box = document.createElement("span");
  box.className = "k-media-box";
  box.style.setProperty("--corner", options.corner.css);
  room.toggleAttribute("data-dimmed", options.dimmed ?? false);
  box.append(placeholder(false));
  let picture: HTMLImageElement | null = null;
  if (options.source) {
    picture = document.createElement("img");
    picture.className = "k-media-picture";
    picture.alt = "";
    picture.decoding = "async";
    picture.src = options.source;
    box.append(picture);
    const shown = picture;
    if (options.arrival === undefined) shown.addEventListener("error", () => arrive(box, shown, "broken"));
  }
  arrive(box, picture, options.arrival ?? (options.source ? "loaded" : "loading"));
  room.append(probe(box, { role: "component", name: "Media", variant: "", fixed: { width: false, height: false } }));
  if (options.caption !== undefined) {
    const caption = wrapped(options.caption, "small", "on_surface_variant");
    caption.style.color = "var(--caption-colour, var(--color-on_surface_variant))";
    room.append(caption);
  }
  return room;
}
