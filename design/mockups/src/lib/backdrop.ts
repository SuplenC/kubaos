// A full-screen moment: the Backdrop template (`docs/design/components/backdrop.md`), what
// polkit's prompt and Kōbō's welcome, done and step-rail screens are laid out on.
//
// A picture, a scrim over it, and one thing in its middle on whole device pixels. The scrim is
// one of three: behind words, a band of `surface` at `opacity.backdrop` across the screen, from
// `space.s64` above the thing to `space.s64` below it, fading to nothing over those `space.s64`, so
// the words keep 4.5:1 over any picture and the picture shows round them; behind a dialog or a
// panel, `scrim` at `opacity.scrim` over all of it; and for a flow of questions (Kōbō's), glass:
// the thing is a card of the shell's glass (`lib/glass.ts`) over the picture's blurred copy, filling
// the screen less its margin, no wider than a rail and two columns with `space.s40` beside each.
//
// The picture can come toward you, growing to `scale.push` round the middle of its bottom edge
// (`--backdrop-push`), and it can go dark toward `surface_container_lowest` (`--backdrop-dark`):
// Kōbō's room lighting up as it opens and as the install runs.
//
// It arrives over `scene` on `emphasized_decel`, the picture and the scrim fading in and the thing
// fading in as it rises `space.s24` into place, and lifts over `scene` on `emphasized_accel`, the
// whole of it fading off what is under it while the thing rises `space.s24` on. With reduce motion
// it fades, capped at `short`, with no rise. Where the room is shorter than the thing and
// `space.s32` above and below it, its parts marked as giving way (Kōbō's sign) go first.

import "./backdrop.css";
import { glass } from "./glass.ts";
import { probe } from "./probe.ts";

/** What lies over the picture: a band behind words, a dim behind a dialog or a panel, or glass. */
export type Scrim = "band" | "dim" | "glass";

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
  /** How far the picture has come toward you, 0 at 1:1 and 1 at `scale.push`. */
  readonly push?: number;
  /** How dark the picture has gone, 0 as it is and 1 `surface_container_lowest`. */
  readonly dark?: number;
}

export function backdrop(options: BackdropOptions): HTMLElement {
  const element = document.createElement("span");
  element.className = "k-backdrop";
  element.dataset.scrim = options.scrim;
  // A whole screen: a motion player draws it as it is laid out.
  element.dataset.whole = "";
  element.style.setProperty("--backdrop-width", options.width);
  element.style.setProperty("--backdrop-height", options.height);
  if (options.push !== undefined) element.style.setProperty("--backdrop-push", String(options.push));
  if (options.dark !== undefined) element.style.setProperty("--backdrop-dark", String(options.dark));
  const layers = document.createElement("span");
  layers.className = "k-backdrop-layers";
  if (options.picture) {
    const picture = document.createElement("img");
    picture.className = "k-backdrop-picture";
    picture.src = options.picture;
    picture.alt = "";
    const dark = document.createElement("span");
    dark.className = "k-backdrop-dark";
    layers.append(picture, dark);
  }
  const scrim = document.createElement("span");
  scrim.className = "k-backdrop-scrim";
  scrim.dataset.scrim = options.scrim;
  layers.append(scrim);
  const thing = options.scrim === "glass" ? glass(document.createElement("span")) : document.createElement("span");
  thing.classList.add("k-backdrop-thing");
  thing.append(options.content);
  element.append(...(options.under ? [options.under] : []), layers, thing);

  places.set(element, { thing });
  placer.observe(element);
  placer.observe(options.content);
  return probe(element, { role: "component", name: "Backdrop", variant: options.scrim, fixed: { width: false, height: false, holds: true } });
}

/** Each backdrop's thing, which placing centres. */
const places = new WeakMap<HTMLElement, { readonly thing: HTMLElement }>();

/** What placing a backdrop read: its size, its thing's, and what it can hold. */
interface Read {
  readonly element: HTMLElement;
  readonly thing: HTMLElement;
  readonly width: number;
  readonly height: number;
  readonly room: number;
  readonly wide: boolean;
  readonly thingWidth: number;
  readonly thingHeight: number;
}

function read(element: HTMLElement, thing: HTMLElement): Read {
  return {
    element,
    thing,
    width: element.clientWidth,
    height: element.clientHeight,
    room: element.clientHeight - 2 * lengthIn(element, "space-s32"),
    // The questions' card is `space.s32` in from the screen's edges where the screen holds two columns
    // side by side, and `space.s16` on a narrower one, as Kōbō's is.
    wide: element.clientWidth >= 2 * lengthIn(element, "size-column"),
    thingWidth: thing.offsetWidth,
    thingHeight: thing.offsetHeight,
  };
}

/**
 * The thing's place is its size's, which only layout knows: centred on whole device pixels, the
 * odd one after it, and what gives way gone while the room can't hold it. Only a backdrop whose
 * parts must give way or come back reads again after writing.
 */
function write(measured: Read): void {
  const { element, thing } = measured;
  let shown = measured;
  const parts = [...thing.querySelectorAll<HTMLElement>("[data-gives-way]")];
  const gone = parts.some((part) => part.dataset.gone !== undefined);
  if (parts.length > 0 && (measured.thingHeight > measured.room || gone)) {
    for (const part of parts) delete part.dataset.gone;
    for (const part of parts) {
      if (thing.offsetHeight <= measured.room) break;
      part.dataset.gone = "";
    }
    shown = read(element, thing);
  }
  element.style.setProperty("--panel-margin", shown.wide ? "var(--space-s32)" : "var(--space-s16)");
  element.style.setProperty("--thing-x", `${Math.floor((shown.width - shown.thingWidth) / 2)}px`);
  element.style.setProperty("--thing-y", `${Math.floor((shown.height - shown.thingHeight) / 2)}px`);
  element.style.setProperty("--thing-height", `${shown.thingHeight}px`);
}

/**
 * One observer for every backdrop: it reads every backdrop that changed before it writes any, so
 * a page of many lays them out once, not once each (49 forced layouts on a page of Kōbō's).
 */
const placer = new ResizeObserver((entries) => {
  const changed = new Set<HTMLElement>();
  for (const entry of entries) {
    const element = (entry.target as HTMLElement).closest<HTMLElement>(".k-backdrop");
    if (element && places.has(element)) changed.add(element);
  }
  const reads = [...changed].map((element) => read(element, places.get(element)!.thing));
  for (const measured of reads) write(measured);
});

/** The lengths measured so far, by the scale they were measured at and their token. */
const measured = new Map<string, number>();

/**
 * The length token `name` in the CSS pixels `element` is laid out in. Measuring adds a box and
 * lays it out, so each length is measured once a scale, for every backdrop at it.
 */
function lengthIn(element: HTMLElement, name: string): number {
  const key = `${getComputedStyle(element).getPropertyValue("--px")} ${name}`;
  const had = measured.get(key);
  if (had !== undefined) return had;
  const meter = document.createElement("span");
  meter.style.cssText = `position:absolute;visibility:hidden;width:var(--${name})`;
  element.append(meter);
  const width = meter.offsetWidth;
  meter.remove();
  measured.set(key, width);
  return width;
}

/** `part` marked as the first to go where the room is short: Kōbō's sign. */
export function givesWay<E extends HTMLElement>(part: E): E {
  part.dataset.givesWay = "";
  return part;
}
