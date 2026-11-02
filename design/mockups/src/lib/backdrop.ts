// A full-screen moment: the Backdrop template (`docs/design/components/backdrop.md`), what the
// lock screen, polkit's prompt and Kōbō's welcome, done and step-rail screens are laid out on.
//
// A picture, a scrim over it, and one thing in its middle on whole device pixels. The scrim is
// one of two: behind words, a band of `surface` at `opacity.backdrop` across the screen, from
// `space.s64` above the thing to `space.s64` below it, fading to nothing over those `space.s64`, so
// the words keep 4.5:1 over any picture and the picture shows round them; behind a dialog or a
// panel, `scrim` at `opacity.scrim` over all of it. The thing may stand `--backdrop-lift` above the
// middle, the lock's optical centre.
//
// It arrives over `scene` on `emphasized_decel`, the picture and the scrim fading in and the thing
// fading in as it rises `space.s24` into place, and lifts over `scene` on `emphasized_accel`, the
// whole of it fading off what is under it while the thing rises `space.s24` on. With reduce motion
// it fades, capped at `short`, with no rise. Where the room is shorter than the thing and
// `space.s32` above and below it, its parts marked as giving way (Kōbō's sign) go first.

import "./backdrop.css";
import { probe } from "./probe.ts";

/** What lies over the picture: a band behind words, or a dim behind a dialog or a panel. */
export type Scrim = "band" | "dim";

export interface BackdropOptions {
  readonly width: string;
  readonly height: string;
  /** The picture: a wallpaper's address. None over `under`, which shows through the dim. */
  readonly picture?: string;
  readonly scrim: Scrim;
  /** The one thing in the middle. */
  readonly content: HTMLElement;
  /** What it lies over and lifts off: the desktop. */
  readonly under?: HTMLElement;
  /** How far above the middle the thing stands, a spacing token's name: the lock's `s64`. */
  readonly lift?: string;
}

export function backdrop(options: BackdropOptions): HTMLElement {
  const element = document.createElement("span");
  element.className = "k-backdrop";
  // A whole screen: a motion player draws it as it is laid out.
  element.dataset.whole = "";
  element.style.setProperty("--backdrop-width", options.width);
  element.style.setProperty("--backdrop-height", options.height);
  element.style.setProperty("--backdrop-lift", options.lift ? `var(--space-${options.lift})` : "0px");
  const layers = document.createElement("span");
  layers.className = "k-backdrop-layers";
  if (options.picture) {
    const picture = document.createElement("img");
    picture.className = "k-backdrop-picture";
    picture.src = options.picture;
    picture.alt = "";
    layers.append(picture);
  }
  const scrim = document.createElement("span");
  scrim.className = "k-backdrop-scrim";
  scrim.dataset.scrim = options.scrim;
  layers.append(scrim);
  const thing = document.createElement("span");
  thing.className = "k-backdrop-thing";
  thing.append(options.content);
  element.append(...(options.under ? [options.under] : []), layers, thing);

  // The thing's place is its size's, which only layout knows: centred on whole device pixels, the
  // odd one after it, and what gives way gone while the room can't hold it.
  const place = () => {
    const room = element.clientHeight - 2 * lengthIn(element, "space-s32");
    // A glass panel is `space.s32` in from the screen's edges where the screen holds two columns
    // side by side, and `space.s16` on a narrower one, as Kōbō's is.
    const wide = element.clientWidth >= 2 * lengthIn(element, "size-column");
    element.style.setProperty("--panel-margin", wide ? "var(--space-s32)" : "var(--space-s16)");
    const parts = [...thing.querySelectorAll<HTMLElement>("[data-gives-way]")];
    for (const part of parts) delete part.dataset.gone;
    for (const part of parts) {
      if (thing.offsetHeight <= room) break;
      part.dataset.gone = "";
    }
    element.style.setProperty("--thing-x", `${Math.floor((element.clientWidth - thing.offsetWidth) / 2)}px`);
    element.style.setProperty("--thing-y", `${Math.floor((element.clientHeight - thing.offsetHeight) / 2)}px`);
    element.style.setProperty("--thing-height", `${thing.offsetHeight}px`);
    // A glass panel's blurred picture lies where the screen's does, so it is moved back by the
    // panel's own place.
    element.style.setProperty("--panel-left", `${thing.offsetLeft}px`);
  };
  const watch = new ResizeObserver(place);
  watch.observe(element);
  watch.observe(options.content);
  return probe(element, { role: "component", name: "Backdrop", variant: options.scrim, fixed: { width: false, height: false, holds: true } });
}

/** The length token `name` in the CSS pixels `element` is laid out in. */
function lengthIn(element: HTMLElement, name: string): number {
  const meter = document.createElement("span");
  meter.style.cssText = `position:absolute;visibility:hidden;width:var(--${name})`;
  element.append(meter);
  const width = meter.offsetWidth;
  meter.remove();
  return width;
}

/** `part` marked as the first to go where the room is short: Kōbō's sign. */
export function givesWay<E extends HTMLElement>(part: E): E {
  part.dataset.givesWay = "";
  return part;
}

/**
 * A glass panel over the backdrop, as Kōbō's step-rail screens hold theirs: its own blurred copy
 * of the picture under `surface` at `opacity.glass`, at the surface radius, a soft hairline inside
 * its edge, casting `shadow.panel`; `space.s32` in from the screen's edges, `space.s16` on a narrow
 * one, and no wider than a rail and two columns with `space.s40` beside each.
 */
export function glassPanel(blurred: string, ...content: Element[]): HTMLElement {
  const panel = document.createElement("span");
  panel.className = "k-glass-panel";
  const picture = document.createElement("img");
  picture.className = "k-glass-picture";
  picture.src = blurred;
  picture.alt = "";
  const glass = document.createElement("span");
  glass.className = "k-glass-body k-sunken";
  glass.append(...content);
  panel.append(picture, glass);
  return panel;
}
