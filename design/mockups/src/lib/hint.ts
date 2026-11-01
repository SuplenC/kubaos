// A control's name in a small bubble under it, while the pointer is over it or the keyboard is on
// it: `kubaos-ui`'s `hint::hint`, `KubaOS.Components`' `Hint`. The inverse colours, `label` type,
// `space.s8` by `space.s4` inside, the row radius, `space.s8` below the control, centred under
// it or held to its start or the window's end, on whole device pixels (`docs/design/components/icon-button.md`). It floats: no part of the
// layout, and it never takes the pointer.

import { probe } from "./probe.ts";
import { advance, text } from "./text.ts";

/**
 * Where a bubble sits across its control: centred, held to the control's start (its left edge on
 * the control's), or held inside the window at its end.
 */
export type Held = "centred" | "start" | "end";

export interface HintOptions {
  /** Shown whatever the pointer and the keyboard do: a sample of the bubble itself. */
  readonly shown?: boolean;
  /**
   * Held to its control's start, for a control at the start of something wider (a slider's icon);
   * or inside the window at its end, for a control at the window's edge.
   */
  readonly held?: Held;
}

/** The bubble saying `words`. */
export function hint(words: string, options: HintOptions = {}): HTMLElement {
  const bubble = document.createElement("span");
  bubble.className = "k-hint";
  bubble.style.setProperty("--natural", String(advance("label", words)));
  bubble.toggleAttribute("data-shown", options.shown ?? false);
  if (options.held !== undefined && options.held !== "centred") bubble.dataset.held = options.held;
  bubble.append(text(words, "label", "surface"));
  return probe(bubble, { role: "component", name: "Hint", variant: "", fixed: { width: false, height: true } });
}

/**
 * Gives `control` its hint, `words`: under it while the pointer is over it, or while it is lit
 * from the keyboard or pinned lit. Returns the control.
 */
export function withHint<E extends HTMLElement>(control: E, words: string, options: HintOptions = {}): E {
  control.classList.add("k-hinted");
  control.append(hint(words, options));
  return control;
}

/**
 * `control`, `side` wide (a length's custom property), in a box with the room its hint, `words`,
 * floats in: below it, and to each side where the bubble is wider than the control, so a sample's
 * bubble never covers its neighbours; held to its start, the room starts with the control, and held
 * at the window's end, the room's end is the window's.
 * The control keeps a whole device pixel.
 */
export function hintRoom(control: Element, words: string, side: string, held: Held = "centred"): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-hint-room";
  if (held !== "centred") room.dataset.held = held;
  room.style.setProperty("--natural", String(advance("label", words)));
  room.style.setProperty("--control-side", side);
  room.append(control);
  return room;
}
