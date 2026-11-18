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

import { KINDS, lightKind, METRICS, paintKind, type Kind, type Size } from "./button.ts";
import { pin, type Pinned } from "./control.ts";
import { type Held, withHint } from "./hint.ts";
import { icon } from "./icon.ts";
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
  /** Its hint held inside the window at its end: a button at the end of a page's title. */
  readonly hintAtEnd?: boolean;
  /** Held, not pressed: what it does once held for `duration.hold`. */
  readonly hold?: () => void;
  /** A held button drawn this far through its hold, 0 to 1: a sample's. */
  readonly held?: number;
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
  element.style.setProperty("--height", `var(--size-${METRICS[size].height})`);
  const slot = document.createElement("span");
  slot.className = "k-icon-slot";
  slot.append(icon(options.icon, `var(--type-${style}-icon)`, "var(--shown)"));
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
