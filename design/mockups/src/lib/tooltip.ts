// What a control does, on a card over it while the pointer is over it or the keyboard is on it:
// `kubaos-ui`'s `tooltip::tooltip` and `tooltip::with_tooltip`, `KubaOS.Components`' `Tooltip`
// (`docs/design/components/tooltip.md`). A title in `body_strong`, the body in `small` and
// `on_surface_variant` `space.s4` under it, and what it holds `space.s8` below (its keys, as
// `KeyHints`), on a popover `size.menu` wide: `surface_container`, the control radius, `space.s12`
// inside, raised to the ladder's top step, a tooltip's (`lib/elevation.ts`).
//
// It isn't neon: no line, no glow and no states of its own. It floats `space.s8` above its control,
// starting where the control starts, or below where there is no room above, and fades in and out
// with its control's lit state. It never takes the pointer.

import { raise, type Step } from "./elevation.ts";
import { keycap, keys } from "./keycap.ts";
import { row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { text, wrapped } from "./text.ts";

/**
 * `content` on a popover `size.menu` wide, raised to `step`, at the control radius: `space.s4`
 * inside round a menu, `space.s12` round anything else. `kubaos-ui`'s `menu::popover`.
 */
export function popover(step: Step, ...content: Element[]): HTMLElement {
  const card = raise(document.createElement("span"), step);
  card.classList.add("k-popover");
  card.append(...content);
  return probe(card, { role: "component", name: "Popover", variant: "", fixed: { width: false, height: false, holds: true } });
}

/** The card: `title`, `body` under it when there is one, and `extra` below. */
export function tooltip(title: string, body: string, extra?: Element): HTMLElement {
  const words = stack("s4", "start", wrapped(title, "body_strong"), ...(body === "" ? [] : [wrapped(body, "small", "on_surface_variant")]));
  const card = document.createElement("span");
  card.className = "k-tooltip";
  card.append(popover(4, stack("s8", "start", words, ...(extra ? [extra] : []))));
  return probe(card, { role: "component", name: "Tooltip", variant: "", fixed: { width: true, height: false } });
}

/**
 * The card for a control its name says enough about, with its key: the name in `body_strong` and
 * the key's keycaps `space.s12` after it, on one line, `space.s4` apart, as wide as they are. Every
 * control that has a key and no words to add takes it, so a surface can drop its key hints.
 */
export function keyTooltip(title: string, combination: string): HTMLElement {
  const caps = row("s4", "centre", ...keys(combination).map(keycap));
  const card = document.createElement("span");
  card.className = "k-tooltip";
  card.dataset.line = "";
  card.append(popover(4, row("s12", "centre", text(title, "body_strong"), caps)));
  return probe(card, { role: "component", name: "Tooltip", variant: "key", fixed: { width: false, height: true } });
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

/** Which of the control's ends a floating card lines up with: its start, or its end near a window's edge. */
export type TipHeld = "start" | "end";

export interface FloatOptions {
  readonly side?: TipSide;
  readonly held?: TipHeld;
  /** Shown whatever the pointer and the keyboard do: a sample of the card over its control. */
  readonly shown?: boolean;
}

/**
 * `control` with its card, `tip`, floating `space.s8` below it (or above), lined up with its start
 * (or its end, for a control at a panel's end, so the card stays on the panel), taking no room in the
 * layout: the toolbar or header around the control never moves. It shows while the control is
 * hovered, focused from the keyboard or pinned lit, and fades over `short`. Returns the anchor that
 * holds both, which goes where the control would.
 */
export function floatingTooltip(control: HTMLElement, tip: HTMLElement, options: FloatOptions = {}): HTMLElement {
  const anchor = document.createElement("span");
  anchor.className = "k-tip-anchor";
  tip.dataset.side = options.side ?? "below";
  tip.dataset.held = options.held ?? "start";
  tip.toggleAttribute("data-shown", options.shown ?? false);
  anchor.append(control, tip);
  return anchor;
}
