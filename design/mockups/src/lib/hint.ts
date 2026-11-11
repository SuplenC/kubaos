// A control's name in a small bubble under it, while the pointer is over it or the keyboard is on
// it: `kubaos-ui`'s `hint::hint`, `KubaOS.Components`' `Hint`. The inverse colours, `label` type,
// `space.s8` by `space.s4` inside, the row radius, `space.s8` below the control, centred under
// it or held to its start or the window's end, on whole device pixels (`docs/design/components/icon-button.md`). It floats: no part of the
// layout, and it never takes the pointer. It opens `space.s8` above its control instead where a part
// asks for that, or where its container has no room for it below: a toolbar at a well's foot.

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
  /**
   * Which side of its control it opens on: below, unless there is no room for it below in the box
   * that cuts or scrolls its control (`auto`, the rule); or above whatever the room (`above`), for a
   * part that always sits at its container's foot.
   */
  readonly side?: "auto" | "above";
}

/** The nearest ancestor of `element` that cuts or scrolls what it holds, or none. */
function container(element: HTMLElement): HTMLElement | null {
  for (let at = element.parentElement; at; at = at.parentElement) {
    const { overflowY } = getComputedStyle(at);
    if (overflowY !== "visible") return at;
  }
  return null;
}

/**
 * Opens `bubble` above its control when its container has no room for it below. The bubble has
 * its size once it is laid out, so this runs then, and again whenever the container resizes.
 */
function flipWhenCut(control: HTMLElement, bubble: HTMLElement): void {
  const place = () => {
    const box = container(control);
    if (!box) return;
    delete bubble.dataset.side;
    // Laid out below while hidden too, so where it would end is where it ends now.
    if (bubble.getBoundingClientRect().bottom > box.getBoundingClientRect().bottom + 0.5) bubble.dataset.side = "above";
  };
  requestAnimationFrame(() => {
    place();
    const box = container(control);
    if (box) new ResizeObserver(place).observe(box);
  });
}

/** The bubble saying `words`. */
export function hint(words: string, options: HintOptions = {}): HTMLElement {
  const bubble = document.createElement("span");
  bubble.className = "k-hint";
  bubble.style.setProperty("--natural", String(advance("label", words)));
  bubble.toggleAttribute("data-shown", options.shown ?? false);
  if (options.held !== undefined && options.held !== "centred") bubble.dataset.held = options.held;
  if (options.side === "above") bubble.dataset.side = "above";
  bubble.append(text(words, "label", "surface"));
  return probe(bubble, { role: "component", name: "Hint", variant: "", fixed: { width: false, height: true } });
}

/**
 * Gives `control` its hint, `words`: under it while the pointer is over it, or while it is lit
 * from the keyboard or pinned lit. Returns the control.
 */
export function withHint<E extends HTMLElement>(control: E, words: string, options: HintOptions = {}): E {
  control.classList.add("k-hinted");
  const bubble = hint(words, options);
  control.append(bubble);
  if (options.side !== "above") flipWhenCut(control, bubble);
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
