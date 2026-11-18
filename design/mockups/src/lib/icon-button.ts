// A circle with a symbol, as tall as the label button of its size: `kubaos-ui`'s
// `button::icon_button`, `KubaOS.Components`' `IconButton`. Its label is required: an icon alone
// says nothing out loud, so the label is its accessible name and shows in its hint while the
// pointer is over it or the keyboard is on it (`docs/design/components/icon-button.md`). It takes a
// button's kinds, colours and states (`lib/button.ts`), and the icon of the label button of its
// size.
//
// A held icon button (`hold`) acts only once held for `duration.hold`: the Spinner's ring drawn
// round its circle in place of its lit tube, an unlit ring with a lit arc running clockwise from the
// top as the hold fills, shown only while it is held. Its timing, its keys and what cancels it are
// the one hold every held part shares (`lib/hold.ts`). A settings row's reset is one.
//
// An outlined icon button (`outlined`) stands on a picture: its circle is the frame's glass, its
// symbol is in its tone, and its line shows at rest, as the tone's unlit glass; lit, it burns as
// the kind's tube and glow do. The Overview's delete zone, and its Undo with the held ring as its
// timer, are two.

import { KINDS, lightKind, METRICS, paintKind, TONES, type Kind, type Size } from "./button.ts";
import { toneColour } from "./focus.ts";
import { pin, type Pinned } from "./control.ts";
import { type Held, withHint } from "./hint.ts";
import { icon, type Morph, morphIcon } from "./icon.ts";
import { probe, variant } from "./probe.ts";
import { heldByKeys, holdable } from "./hold.ts";

const SVG = "http://www.w3.org/2000/svg";

/** The kinds an icon button comes in: every button kind, with its colours and states. */
export const ICON_KINDS = KINDS;
export type IconKind = Kind;

export interface IconButtonOptions {
  readonly icon: string;
  readonly label: string;
  readonly kind?: IconKind;
  readonly size?: Size;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
  /** Whether its hint is drawn: a state matrix leaves it out, since it floats over the row below. */
  readonly hint?: boolean;
  /** Its hint above it whatever the room: a button in a toolbar at a container's foot. */
  readonly hintAbove?: boolean;
  /**
   * Its symbol morphs between two states in place of `icon` (`morphIcon`, `setMorph`): play and
   * pause. `on` is the second state; the owner names the button for what a press does.
   */
  readonly morph?: { readonly pair: Morph; readonly on: boolean };
  /** Its hint held inside the window at its end: a button at the end of a page's title. */
  readonly hintAtEnd?: boolean;
  /** Held, not pressed: what it does once held for `duration.hold`. */
  readonly hold?: () => void;
  /** A held button drawn this far through its hold, 0 to 1: a sample's. */
  readonly held?: number;
  /** On a picture: the frame's glass for a circle, its symbol in its tone, its line shown at rest. */
  readonly outlined?: boolean;
}

export function iconButton(options: IconButtonOptions): HTMLButtonElement {
  const kind = options.kind ?? "Filled";
  const size = options.size ?? "Md";
  const style = METRICS[size].label;
  const element = document.createElement("button");
  element.className = "k-button k-kind k-control";
  element.type = "button";
  element.dataset.iconOnly = "";
  element.disabled = options.disabled ?? false;
  element.setAttribute("aria-label", options.label);
  pin(element, options.pinned);
  paintKind(element, kind);
  if (options.outlined) outline(element, kind);
  element.style.setProperty("--height", `var(--size-${METRICS[size].height})`);
  const slot = document.createElement("span");
  slot.className = "k-icon-slot";
  slot.append(options.morph ? morphIcon(options.morph.pair, `var(--type-${style}-icon)`, "var(--shown)", options.morph.on) : icon(options.icon, `var(--type-${style}-icon)`, "var(--shown)"));
  element.append(slot);
  lightKind(element, kind, "full", slot);
  if (options.hold || options.held !== undefined) {
    element.append(holdRing());
    element.dataset.hold = "";
    showHold(element, options.held ?? 0);
    if (options.hold) heldByKeys(element, holdable(element, options.hold, { empties: true }));
  }
  const held: Held | undefined = options.hintAtEnd ? "end" : undefined;
  if (options.hint ?? true) withHint(element, options.label, { ...(options.hintAbove ? { side: "above" as const } : {}), ...(held ? { held } : {}) });
  return probe(element, { role: "component", name: "IconButton", variant: variant(kind, size), fixed: { width: true, height: true } });
}

/** An outlined button's colours: the frame's glass in every state, and its symbol in its tone. */
function outline(element: HTMLElement, kind: Kind): void {
  element.dataset.outlined = "";
  const glass = "rgb(from var(--color-surface_container) r g b / var(--opacity-frame))";
  for (const name of ["--fill", "--fill-lit", "--fill-off", "--fill-pressed"]) element.style.setProperty(name, glass);
  if (kind === "Text" || kind === "Tonal") return;
  const tone = `var(--color-${toneColour(TONES[kind])})`;
  for (const name of ["--label", "--label-lit", "--label-off"]) element.style.setProperty(name, tone);
}

/** The ring a held button's hold fills: the Spinner's unlit ring and lit arc, round the circle. */
function holdRing(): SVGSVGElement {
  const drawing = document.createElementNS(SVG, "svg");
  drawing.setAttribute("class", "k-hold-ring");
  drawing.setAttribute("aria-hidden", "true");
  for (const part of ["k-hold-track", "k-hold-arc"]) {
    const circle = document.createElementNS(SVG, "circle");
    circle.setAttribute("class", part);
    // The arc's length in degrees, so its dash is its sweep.
    circle.setAttribute("pathLength", "360");
    drawing.append(circle);
  }
  return drawing;
}

/** Draws `element`'s hold `fraction` of the way round; the ring shows only while it is above 0. */
function showHold(element: HTMLElement, fraction: number): void {
  element.style.setProperty("--hold", String(fraction));
  element.toggleAttribute("data-holding", fraction > 0);
}
