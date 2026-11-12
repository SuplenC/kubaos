// One value asked for in a popover: `Prompt`, new, in neither library yet. Its label in `label` type
// with an Enter keycap at the row's end, the Field under it with all of its value selected, and the
// Field's Note under that when there is something to say: the library's `Labelled`, `Field`, `Keycap`
// and `Note`, nothing of its own.
//
// It floats on its own Popover, `space.s8` under the text it changes, its label and Field lined up
// with that text's start (or its end, for text at a panel's end); or it sits inside a
// menu's popover between its separators, inset so its edges land where the floating one's do. Enter
// takes the value; Esc or a click outside it cancels. A value it can't take turns the Field invalid,
// the reason in its Note, and the prompt stays open until the value is one it takes.

import "./prompt.css";
import type { Pinned } from "./control.ts";
import { type Caret, field, labelled, note, setInvalid } from "./field.ts";
import { keycap } from "./keycap.ts";
import { probe } from "./probe.ts";
import { popover } from "./tooltip.ts";

/** Where it is: on its own popover under its anchor, or inside a menu's popover. */
export type PromptPlace = "floating" | "menu";

export interface PromptOptions {
  /** What it asks for: Rename, Set value. */
  readonly label: string;
  readonly value: string;
  /** Why `typed` can't be taken, in a few words, or null when it can. */
  readonly check: (typed: string) => string | null;
  /** What the Note says while the value is one it takes, if anything: what a rule reads as. */
  readonly note?: (typed: string) => string | undefined;
  readonly place?: PromptPlace;
  /** The Field pinned lit, as with the keyboard in it: a still sample. */
  readonly pinned?: Pinned;
  /** The caret; all of the value selected unless given. */
  readonly caret?: Caret;
  /** Drawn after Enter on a value it can't take: the Field invalid, the reason in its Note. */
  readonly invalid?: boolean;
  readonly onInput?: (typed: string) => void;
  readonly onSave?: (typed: string) => void;
  readonly onCancel?: () => void;
}

export interface Prompt {
  readonly element: HTMLElement;
  /** Puts the keyboard in its Field, the caret striking on. */
  readonly focus: () => void;
  readonly value: () => string;
}

export function prompt(options: PromptOptions): Prompt {
  const place = options.place ?? "floating";
  let typed = options.value;
  let invalid = options.invalid ?? false;
  const count = [...typed].length;
  const box = field({
    value: typed,
    caret: options.caret ?? { pos: count, anchor: 0 },
    ...(invalid ? { state: "Invalid" as const } : {}),
    ...(options.pinned ? { pinned: options.pinned } : {}),
    onInput: (value) => {
      typed = value;
      // Once it is a value it takes, the Field is no longer invalid; until then it stays so.
      if (invalid && options.check(value) === null) {
        invalid = false;
        setInvalid(box, false);
      }
      say();
      options.onInput?.(value);
    },
  });
  const column = labelled(options.label, box, { action: keycap("Enter") });
  let line: HTMLElement | null = null;

  /** The Note: the reason while invalid, else what `note` says, else none. */
  function say(): void {
    const words = invalid ? options.check(typed) : options.note?.(typed);
    const fresh = words ? note(words, invalid) : null;
    line?.remove();
    line = fresh;
    if (fresh) column.append(fresh);
  }
  say();

  box.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      if (options.check(typed) === null) options.onSave?.(typed);
      else {
        invalid = true;
        setInvalid(box, true);
        say();
      }
      event.preventDefault();
    } else if (event.key === "Escape") {
      options.onCancel?.();
      event.preventDefault();
    } else if (event.key === "Tab") {
      // Floating, its Field is its one stop; in a menu, Tab goes on through the menu.
      if (place === "floating") event.preventDefault();
      else return;
    }
    // Its keys are its own while it is open: none reaches the list or the sheet under it.
    event.stopPropagation();
  });

  let element: HTMLElement;
  if (place === "floating") {
    element = popover(2, column);
    element.classList.add("k-prompt");
  } else {
    element = document.createElement("span");
    element.className = "k-prompt-in";
    element.append(column);
  }
  probe(element, { role: "component", name: "Prompt", variant: place === "menu" ? "menu" : "", fixed: { width: true, height: false } });
  return { element, focus: () => box.focus({ preventScroll: true }), value: () => typed };
}

/** Which end of its anchor a floating prompt lines up with. */
export type PromptHeld = "start" | "end";

export interface FloatOptions {
  /** Its end on the anchor's end: text at a panel's end, so it stays on the panel. */
  readonly held?: PromptHeld;
  /** The layer grows to hold it: a sample, where nothing else gives it room. */
  readonly grow?: boolean;
}

/**
 * Floats `card` in `layer` (its containing block) `space.s8` under `anchor`, its content lined up
 * with the anchor's start or its end, and keeps it there as the layer is laid out again: a new
 * scale, a sample drawn.
 */
export function floatUnder(card: HTMLElement, anchor: HTMLElement, layer: HTMLElement, options: FloatOptions = {}): void {
  card.classList.add("k-prompt-float");
  if (card.parentElement !== layer) layer.append(card);
  const place = () => {
    const room = layer.getBoundingClientRect();
    const at = anchor.getBoundingClientRect();
    if (room.width === 0 || at.width === 0) return;
    // The layer may be drawn scaled, to fit a pane: its laid-out pixels are its drawn ones over this.
    const ratio = room.width / layer.offsetWidth || 1;
    // Inside the page a CSS pixel is a device pixel, so whole ones keep it on the device grid.
    const top = Math.round((at.bottom - room.top) / ratio);
    card.style.top = `calc(${top}px + var(--space-s8))`;
    // Its label and Field line up with the text, the popover's padding reaching past it.
    if (options.held === "end") {
      card.style.left = "auto";
      card.style.right = `calc(${Math.round((room.right - at.right) / ratio)}px - var(--space-s12))`;
    } else {
      card.style.right = "auto";
      card.style.left = `calc(${Math.round((at.left - room.left) / ratio)}px - var(--space-s12))`;
    }
    if (options.grow) layer.style.minHeight = `calc(${top + card.offsetHeight}px + var(--space-s8))`;
  };
  // The layer, and the anchor too: text that settles its width after the layer moves the card.
  const sized = new ResizeObserver(place);
  sized.observe(layer);
  sized.observe(anchor);
  requestAnimationFrame(place);
}
