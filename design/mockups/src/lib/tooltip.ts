// What a control does, on a card over it while the pointer is over it or the keyboard is on it:
// `kubaos-ui`'s `tooltip::tooltip` and `tooltip::with_tooltip`, `KubaOS.Components`' `Tooltip`
// (`docs/design/components/tooltip.md`). A title in `body_strong`, the body in `small` and
// `on_surface_variant` `space.s4` under it, and what it holds `space.s8` below (its keys, as
// `KeyHints`), on a popover `size.menu` wide: `surface_container`, a soft hairline, the surface
// radius, `space.s12` inside and the panel's shadow.
//
// It isn't neon: no line, no glow and no states of its own. It floats `space.s8` above its control,
// starting where the control starts, or below where there is no room above, and fades in and out
// with its control's lit state. It never takes the pointer.

import { stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { wrapped } from "./text.ts";

/** `content` on a popover `size.menu` wide: `kubaos-ui`'s `menu::popover`. */
export function popover(...content: Element[]): HTMLElement {
  const card = document.createElement("span");
  card.className = "k-popover";
  card.append(...content);
  return probe(card, { role: "component", name: "Popover", variant: "", fixed: { width: false, height: false, holds: true } });
}

/** The card: `title`, `body` under it when there is one, and `extra` below. */
export function tooltip(title: string, body: string, extra?: Element): HTMLElement {
  const words = stack("s4", "start", wrapped(title, "body_strong"), ...(body === "" ? [] : [wrapped(body, "small", "on_surface_variant")]));
  const card = document.createElement("span");
  card.className = "k-tooltip";
  card.append(popover(stack("s8", "start", words, ...(extra ? [extra] : []))));
  return probe(card, { role: "component", name: "Tooltip", variant: "", fixed: { width: true, height: false } });
}

/** Where a card floats from its control: above it, or below where there is no room above. */
export type TipSide = "above" | "below";

export interface TipOptions {
  readonly side?: TipSide;
  /** Held inside the window at its end: the control is at the window's end, the card's end on it. */
  readonly held?: boolean;
  /** Shown whatever the pointer and the keyboard do: a sample of the card over its control. */
  readonly shown?: boolean;
}

/**
 * `control` with its card, `tip`, `space.s8` above it (or below), in a room for both: the card shows
 * while the control is hovered, focused from the keyboard or pinned lit, and fades over `short`.
 * The room keeps the card's place, so nothing moves as it comes and goes.
 */
export function withTooltip(control: HTMLElement, tip: HTMLElement, options: TipOptions = {}): HTMLElement {
  const side = options.side ?? "above";
  const room = stack("s8", options.held ? "end" : "start", ...(side === "above" ? [tip, control] : [control, tip]));
  room.classList.add("k-tip-room");
  tip.toggleAttribute("data-shown", options.shown ?? false);
  return room;
}
