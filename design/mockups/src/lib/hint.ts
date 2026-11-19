// A control's name in a small bubble under it, while the pointer is over it or the keyboard is on
// it: `kubaos-ui`'s `hint::hint`, `KubaOS.Components`' `Hint`. The inverse colours, `label` type,
// `space.s8` by `space.s4` inside, the row radius, `space.s8` below the control, centred under
// it or held to its start or the window's end, on whole device pixels (`docs/design/components/icon-button.md`). It floats: no part of the
// layout, and it never takes the pointer. It opens `space.s8` above its control instead where a part
// asks for that, or where its container has no room for it below: a toolbar at a well's foot. Where
// centring it would run it past a side of the box that cuts its control (a scrolling pane, the
// window), it moves along that side until it is whole inside, `space.s8` from the edge.

import { ALL } from "../site/settings.ts";
import { probe } from "./probe.ts";
import { advance, text } from "./text.ts";
import { length } from "./tokens.ts";

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
 * The box a bubble keeps inside across: the nearest ancestor of `element` that cuts what it holds
 * (the window), or none. A scroll's view cuts past the scroll's own box, reaching a glow's width into
 * the part beside it (`lib/scroll.ts`), which is drawn over that reach: there the scroll's box is the
 * edge a bubble keeps to.
 */
function sideways(element: HTMLElement): HTMLElement | null {
  for (let at = element.parentElement; at; at = at.parentElement) {
    if (getComputedStyle(at).overflowX === "visible") continue;
    return at.classList.contains("k-scroll-view") && at.parentElement?.classList.contains("k-scroll") ? at.parentElement : at;
  }
  return null;
}

/**
 * How far `bubble` moves across to be whole inside `box`, `space.s8` from its sides, in the
 * bubble's own pixels, whole device pixels: none where it fits, along the start where both sides
 * are too near.
 */
function shiftInto(bubble: HTMLElement, box: DOMRect): number {
  const at = bubble.getBoundingClientRect();
  if (at.width === 0) return 0;
  // The page may be zoomed: the rectangles are in the viewport's pixels, the shift in the bubble's.
  const ratio = bubble.offsetWidth / at.width;
  const margin = length("space-s8") / ratio * (Number.parseFloat(getComputedStyle(bubble).getPropertyValue("--scale")) || 1);
  const start = box.left + margin - at.left;
  const end = box.right - margin - at.right;
  const by = start > 0 ? start : end < 0 ? end : 0;
  return Math.round(by * ratio);
}

/**
 * Places `bubble` where its container lets it show whole: above its control when there is no room
 * below (unless `flips` is off), and moved across where centring would run it past a side. Now and
 * whenever the boxes resize; it runs as the bubble is made, laid out, so it never shows cut for a
 * frame.
 */
function placeWhole(control: HTMLElement, bubble: HTMLElement, flips: boolean): void {
  const place = () => {
    const box = container(control);
    if (flips && box) {
      delete bubble.dataset.side;
      // Laid out below while hidden too, so where it would end is where it ends now.
      if (bubble.getBoundingClientRect().bottom > box.getBoundingClientRect().bottom + 0.5) bubble.dataset.side = "above";
    }
    const across = sideways(control);
    bubble.style.setProperty("--shift", "0px");
    if (across) bubble.style.setProperty("--shift", `${shiftInto(bubble, across.getBoundingClientRect())}px`);
  };
  place();
  const watched = new ResizeObserver(place);
  for (const box of [container(control), sideways(control), control]) if (box) watched.observe(box);
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
  return probe(bubble, { role: "component", name: "Hint", variant: "", fixed: { width: false, height: true } }, { floats: true });
}

/** The hint each control drawn but not yet pointed at or reached by the keyboard will make. */
const pending = new WeakMap<Element, (arriving: boolean) => void>();

/** The controls already listening for the pointer and the keyboard: a control named again keeps them. */
const listening = new WeakSet<Element>();

/** Controls given a hint since the last frame, whose pin or key may have come after it. */
let unseen: Element[] = [];

/** Makes the hints of the controls given one since the last frame that show whatever the pointer does. */
function makePinned(): void {
  const these = unseen;
  unseen = [];
  for (const control of these) {
    if (control.hasAttribute("data-pinned") || control.hasAttribute("data-keyed")) pending.get(control)?.(false);
  }
}

/**
 * Gives `control` its hint, `words`: under it while the pointer is over it, or while it is lit
 * from the keyboard or pinned lit. Returns the control.
 *
 * The bubble is made the first time the pointer comes over the control or the keyboard reaches it,
 * not with the control: a page of hinted controls lays out none of them until one is wanted. One
 * made while its control is already pointed at is laid in hidden and its hidden look worked out
 * before it is let go, so it fades in as one made before would, on the same frame. A bubble shown
 * whatever the pointer does (pinned or keyed, by the next frame, or every sample drawn at once) is
 * made then, and a control named again while it is pointed at or holds the keyboard shows its new
 * one at once, as it showed the old.
 */
export function withHint<E extends HTMLElement>(control: E, words: string, options: HintOptions = {}): E {
  control.classList.add("k-hinted");
  pending.set(control, (arriving) => {
    pending.delete(control);
    const bubble = hint(words, options);
    if (arriving) bubble.dataset.arriving = "";
    control.append(bubble);
    if (arriving) {
      void getComputedStyle(bubble).opacity;
      delete bubble.dataset.arriving;
    }
    placeWhole(control, bubble, options.side !== "above");
  });
  if (options.shown || ALL) {
    requestAnimationFrame(() => pending.get(control)?.(false));
    return control;
  }
  if (!listening.has(control)) {
    listening.add(control);
    const wanted = () => pending.get(control)?.(true);
    control.addEventListener("pointerenter", wanted);
    control.addEventListener("focusin", wanted);
  }
  if (control.isConnected && control.matches(":hover, :focus-within")) {
    pending.get(control)?.(false);
    return control;
  }
  if (!unseen.length) requestAnimationFrame(makePinned);
  unseen.push(control);
  return control;
}

/** Makes every hint under `root` not made yet: a sample whose move plays its bubble. */
export function makeHints(root: ParentNode): void {
  for (const control of root.querySelectorAll(".k-hinted")) pending.get(control)?.(false);
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
