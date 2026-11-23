// What a control does, on a card over it while the pointer is over it or the keyboard is on it:
// `kubaos-ui`'s `tooltip::tooltip` and `tooltip::with_tooltip`, `KubaOS.Components`' `Tooltip`
// (`docs/design/components/tooltip.md`). A small card, sized for the small controls it explains:
// its name in `label`, a hint's type and no larger than a small button's label, the body in `small`
// and `on_surface_variant` `space.s4` under it, and what it holds `space.s8` below (its keys, as
// `KeyHints`). The name's own key stands on the name's line. The card is a popover as wide as its
// words, `size.tooltip` at most: the shell's glass at the control radius, `space.s8` inside above
// and below and `space.s12` at the sides, on the ladder's top step, a tooltip's
// (`lib/elevation.ts`).
//
// It isn't neon: no line, no glow and no states of its own. It floats `space.s8` above its control,
// starting where the control starts, or below where there is no room above, and fades in and out
// with its control's lit state. It never takes the pointer.
//
// A term in running text takes a card too (`termed`): the word stays in its sentence, dotted under,
// and its card hangs over it while the pointer or the keyboard is on it. The card sits beside the
// sentence, not in it, tied to the word by an anchor, so the sentence's words are only its own.
//
// A card can answer a press instead of a light (`floatingTooltip`'s `shows: "pressed"`): a press shows
// it for two holds, then it goes, and the next press starts it again.

import type { Step } from "./elevation.ts";
import { glass } from "./glass.ts";
import { keycap, keys } from "./keycap.ts";
import { row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { text, wrapped } from "./text.ts";
import { milliseconds } from "../site/recipe.ts";
import { length } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** Why a menu's popover is drawn ahead of `kubaos-ui`'s, which pads it 12 and draws its items' symbols 20, 12 from their words. */
const MENU_AHEAD = "menu.md: a menu's popover is 4 inside, its items' symbols 16, 8 from their words";

/**
 * `content` on a popover `size.menu` wide, on the shell's glass at the control radius: `space.s4`
 * inside round a menu, `space.s12` round anything else. `step` is where it stands on the ladder,
 * which orders what floats over what; the glass draws every step alike. `kubaos-ui`'s
 * `menu::popover`.
 */
export function popover(step: Step, ...content: Element[]): HTMLElement {
  const card = glass(document.createElement("span"));
  card.classList.add("k-popover");
  card.dataset.step = String(step);
  card.append(...content);
  // A menu's popover and its items are drawn as `docs/design/components/menu.md` has them, which
  // `kubaos-ui`'s haven't caught up with yet.
  const ahead = content.some((each) => each.matches(".k-menu")) ? { ahead: MENU_AHEAD } : {};
  return probe(card, { role: "component", name: "Popover", variant: "", fixed: { width: false, height: false, holds: true } }, ahead);
}

/** Where a popover stands by its anchor, and how far inside its room it is held. */
export interface Anchoring {
  /** Above its anchor, or below it. */
  readonly side?: "above" | "below";
  /** How far in from the room's sides it is held, in the room's CSS pixels: its padding. */
  readonly inset?: number;
}

/**
 * Stands `card`, laid out absolutely in `room`, by its `anchor`: `space.s8` off the anchor's edge,
 * as a hint or a tooltip floats off its control, centred across it, and held inside `room` by
 * `inset` at either end, on whole pixels. Laid out again whenever its owner asks, as the anchor
 * moves with the room.
 */
export function anchorPopover(card: HTMLElement, anchor: HTMLElement, room: HTMLElement, anchoring: Anchoring = {}): void {
  // In the room's own pixels: its box is scaled by the page's zoom like everything in it.
  const box = room.getBoundingClientRect();
  const ratio = room.offsetWidth / Math.max(1, box.width);
  const at = anchor.getBoundingClientRect();
  const gap = length("space-s8") * settings().scale;
  const inset = anchoring.inset ?? 0;
  const centre = (at.left + at.width / 2 - box.left) * ratio;
  const left = Math.max(inset, Math.min(room.offsetWidth - inset - card.offsetWidth, centre - card.offsetWidth / 2));
  card.style.left = `${Math.round(left)}px`;
  if ((anchoring.side ?? "above") === "above") {
    card.style.top = "auto";
    card.style.bottom = `${Math.round((box.bottom - at.top) * ratio + gap)}px`;
  } else {
    card.style.bottom = "auto";
    card.style.top = `${Math.round((at.bottom - box.top) * ratio + gap)}px`;
  }
}

/** The body's line: quieter than the name, and a short sentence of it on one line. */
const quiet = (body: string): HTMLElement[] => (body === "" ? [] : [wrapped(body, "small", "on_surface_variant")]);

/** The card round `words`, raised and floating as every tooltip's. */
function card(words: Element, variant: string): HTMLElement {
  const made = document.createElement("span");
  made.className = "k-tooltip";
  made.append(popover(4, words));
  return probe(made, { role: "component", name: "Tooltip", variant, fixed: { width: false, height: false } }, { floats: true });
}

/** The card: `title`, `body` under it when there is one, and `extra` below. */
export function tooltip(title: string, body: string, extra?: Element): HTMLElement {
  const words = stack("s4", "start", wrapped(title, "label"), ...quiet(body));
  return card(stack("s8", "start", words, ...(extra ? [extra] : [])), "");
}

/**
 * The card for a control with a key of its own: the name in `label` and the key's keycaps
 * `space.s12` after it, on one line, `space.s4` apart, and the body, when there is one, a quiet
 * line `space.s4` under them. Every control that has a key takes it, so a surface can drop its key
 * hints; the keys stay on the name's line, never on a line of their own under the body.
 */
export function keyTooltip(title: string, combination: string, body = ""): HTMLElement {
  const caps = row("s4", "centre", ...keys(combination).map(keycap));
  caps.classList.add("k-tooltip-keys");
  const name = row("s12", "centre", text(title, "label"), caps);
  return card(body === "" ? name : stack("s4", "start", name, ...quiet(body)), "key");
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

/** When a floating card shows: while its control is lit, or for a while after it is pressed. */
export type TipShows = "lit" | "pressed";

export interface FloatOptions {
  readonly side?: TipSide;
  readonly held?: TipHeld;
  /** Shown whatever the pointer and the keyboard do: a sample of the card over its control. */
  readonly shown?: boolean;
  readonly shows?: TipShows;
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
  if (options.shows === "pressed") {
    anchor.dataset.shows = "pressed";
    let gone = 0;
    control.addEventListener("click", () => {
      window.clearTimeout(gone);
      tip.removeAttribute("data-shown");
      requestAnimationFrame(() => tip.setAttribute("data-shown", ""));
      gone = window.setTimeout(() => tip.removeAttribute("data-shown"), (milliseconds("hold") * 2) / settings().speed);
    });
  }
  return anchor;
}

/** How many terms are on the page: each one's anchor is named for its place. */
let terms = 0;

/**
 * `sentence`, a text, with `term` (one of its words, as `term()` makes it) explained on `tip`: the
 * card floats `space.s8` above the word, from its start, over whatever is there, and shows while the
 * word is hovered or focused from the keyboard. Returns the room that holds both, which goes where
 * the sentence would.
 */
export function termed(sentence: HTMLElement, term: HTMLElement, tip: HTMLElement, options: { readonly shown?: boolean } = {}): HTMLElement {
  const name = `--k-term-${++terms}`;
  term.style.setProperty("anchor-name", name);
  tip.style.setProperty("position-anchor", name);
  tip.toggleAttribute("data-shown", options.shown ?? false);
  const room = document.createElement("span");
  room.className = "k-term-room";
  room.append(sentence, tip);
  return room;
}

/** `words` as a term in a sentence: dotted under, and reached by Tab, so its card can be read from the keyboard. */
export function term(words: string): HTMLElement {
  const run = document.createElement("span");
  run.className = "k-term";
  run.tabIndex = 0;
  run.textContent = words;
  return run;
}
