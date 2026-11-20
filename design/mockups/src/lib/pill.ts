// A group in the bar: `KubaOS.Components`' `Pill` (`docs/design/components/pill.md`), the shell's
// alone. A pill `size.control_sm` tall at the full radius on `surface_container`, its content a row
// `space.s8` apart centred on device pixels, `space.s12` at each end between the curve and the
// content (`space.s8` round a lone symbol). A pill of its own tone takes a fill and an ink together.
//
// An interactive pill takes the pointer and is a control in the state model (`.k-control` in
// `lib.css`): it is never a stop for Tab, so a panel's keyboard focus never wanders into the bar,
// and the pointer only ever gives it the quiet hover, no line and no glow. Hovered, its fill is a
// step lighter, pressed a step further: a neutral pill climbs the container ladder
// (`surface_container_high`, then `_highest`), and one of its own tone takes the state layer over
// its fill (`opacity.hover`, then `opacity.press` of `on_surface`). Its name shows in a hint under
// it while the pointer is on it. When its content changes,
// what stays (the same element, or an equal one) keeps its place, solid, and moves with the change;
// what leaves fades out as its room closes and what arrives fades in as its room opens, over
// `medium` on `standard_decel`, so the pill's width eases with them. Inside a symbol and its value,
// only the part that differs fades. Reduce motion changes it at once, with no fade.
//
// A symbol and its own value (a reading, a battery's share, a count down) are one thing in the row:
// `space.s4` apart, the value straight after its symbol (`paired`). Room a value is held to, so its
// neighbours stay put as it changes, falls after it, never between it and its symbol.

import { pin, type Pinned } from "./control.ts";
import { focusRing, type Tone } from "./focus.ts";
import { withHint } from "./hint.ts";
import { row } from "./layout.ts";
import { probe } from "./probe.ts";
import { cubic } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/**
 * A pill's tone: the bar's own, a screen recording's, the live preview's cold tint, or what is on
 * air, each on its meaning's container as a recording is, so it is found at a glance: the screen
 * shared on `sharing_container`, and the camera in use on `error_container`, red as a recording.
 */
export type PillTone = "neutral" | "recording" | "preview" | "sharing" | "camera";

/** A pill's colours: its fill at rest, hovered and pressed, its ink, and the tone its tube lights in. */
interface Fill {
  readonly fill: string;
  readonly hover: string;
  readonly pressed: string;
  readonly ink: string;
  readonly tone: Tone;
}

/** `fill` under the state layer, `on_surface` at the opacity token `opacity`: a tone's fill a step on. */
const layered = (fill: string, opacity: "hover" | "press") => `color-mix(in srgb, ${fill}, var(--color-on_surface) calc(var(--opacity-${opacity}) * 100%))`;

/** A tone's fill and ink, its steps the state layer over the fill. */
const toned = (fill: string, ink: string, tone: Tone): Fill => ({ fill, hover: layered(fill, "hover"), pressed: layered(fill, "press"), ink, tone });

const FILLS: Record<PillTone, Fill> = {
  neutral: {
    fill: "var(--color-surface_container)",
    hover: "var(--color-surface_container_high)",
    pressed: "var(--color-surface_container_highest)",
    ink: "var(--color-on_surface)",
    tone: "accent",
  },
  recording: toned("var(--color-error_container)", "var(--color-on_error_container)", "error"),
  preview: toned("rgb(from var(--color-cold) r g b / var(--opacity-tint))", "var(--color-cold)", "cold"),
  sharing: toned("var(--color-sharing_container)", "var(--color-on_sharing_container)", "sharing"),
  camera: toned("var(--color-error_container)", "var(--color-on_error_container)", "recording"),
};

export interface PillOptions {
  readonly content: readonly Element[];
  /** A lone symbol: `space.s8` at each end instead of `space.s12`. */
  readonly lone: boolean;
  /** The room at each end, as QML's `padding`, where a pill sets its own: the tray's `space.s4`. */
  readonly end?: string;
  /** The gap between its content, as QML's `spacing`, where a pill sets its own: `space.s4`, `space.s12`. */
  readonly gap?: string | 0;
  readonly tone?: PillTone;
  /** Takes the pointer: a quiet fill under it, a step more pressed, and its name in a hint. */
  readonly interactive?: boolean;
  /** Its name, in a hint under it while the pointer is on it. */
  readonly tooltip?: string;
  readonly pinned?: Pinned;
}

export interface Pill {
  readonly element: HTMLElement;
  /** Puts `content` in it: what leaves fades as its room closes, what arrives as its room opens. */
  readonly change: (content: readonly Element[]) => void;
  /** Catches it at the start of a change to `content`, without playing it: a move's sample. */
  readonly caught: (content: readonly Element[]) => void;
}

/** A symbol and its own value, `space.s4` apart: one thing in a pill's row. */
export function paired(symbol: Element, value: Element): HTMLElement {
  return row("s4", "centre", symbol, value);
}

/** A pill as an element, for a sample that never changes its content. */
export function pill(options: PillOptions): HTMLElement {
  return makePill(options).element;
}

export function makePill(options: PillOptions): Pill {
  const tone = FILLS[options.tone ?? "neutral"];
  const element = document.createElement("span");
  element.className = "k-pill";
  element.toggleAttribute("data-lone", options.lone);
  element.style.setProperty("--fill", tone.fill);
  element.style.setProperty("--fill-hover", tone.hover);
  element.style.setProperty("--fill-pressed", tone.pressed);
  element.style.setProperty("--ink", tone.ink);
  if (options.end !== undefined) element.style.setProperty("--end", `var(--space-${options.end})`);
  const clip = document.createElement("span");
  clip.className = "k-pill-clip";
  const line = row(options.gap ?? "s8", "centre", ...options.content);
  clip.append(line);
  element.append(clip);
  if (options.interactive) {
    element.classList.add("k-control");
    element.dataset.hot = "";
    element.setAttribute("role", "button");
    pin(element, options.pinned);
    focusRing(element, "full", tone.tone, false).dataset.pressable = "";
    if (options.tooltip) withHint(element, options.tooltip);
  }

  function change(content: readonly Element[]): void {
    for (const slot of line.querySelectorAll<HTMLElement>(".k-pill-slot")) settle(slot);
    if (settings().reduced) {
      line.replaceChildren(...content);
      return;
    }
    line.replaceChildren(...swapped([...line.children], content));
    const easing = `cubic-bezier(${cubic("easing-standard_decel").join(", ")})`;
    for (const slot of line.querySelectorAll<HTMLElement>(".k-pill-slot")) {
      const leaving = slot.hasAttribute("data-out");
      const move = slot.animate([{ "--pill-open": leaving ? 1 : 0 }, { "--pill-open": leaving ? 0 : 1 }], { duration: motion("medium"), easing, fill: "forwards" });
      move.finished.then(() => settle(slot), () => {});
    }
  }

  const caught = (content: readonly Element[]) => line.replaceChildren(...swapped([...line.children], content));
  probe(element, { role: "component", name: "Pill", variant: "", fixed: { width: false, height: true } });
  return { element, change, caught };
}

/** Content in its own room while it leaves or arrives: the room closes or opens, the content fading with it. */
function slot(content: Element, way: "out" | "in"): HTMLElement {
  const made = document.createElement("span");
  made.className = "k-pill-slot";
  made.dataset[way] = "";
  made.append(content);
  return made;
}

/** A finished slot: what arrived takes its place in the row; what left goes. */
function settle(made: HTMLElement): void {
  if (made.hasAttribute("data-out")) made.remove();
  else if (made.firstElementChild) made.replaceWith(made.firstElementChild);
}

/**
 * The row while it changes from `old` to `next`. What is in both stays as it is: the same element,
 * or an equal one (the recording's dot, built again with its time), the old one kept. A row of the
 * library's layout in both, as alike as a symbol and its value (`paired`), stays too, and changes
 * inside the same way, so only the part that differs fades. What only `old` has leaves where it
 * was, and what only `next` has arrives where it will be. What stays keeps its order in both.
 */
function swapped(old: readonly Element[], next: readonly Element[]): Element[] {
  const kept = matched(old, next);
  const staying = new Set(kept);
  const merged: Element[] = [];
  let at = 0;
  for (const each of old) {
    if (!staying.has(each)) {
      merged.push(slot(each, "out"));
      continue;
    }
    const found = kept.indexOf(each, at);
    for (; at < found; at++) merged.push(slot(kept[at]!, "in"));
    merged.push(each);
    at = found + 1;
  }
  for (; at < kept.length; at++) merged.push(slot(kept[at]!, "in"));
  return merged;
}

/**
 * `next` with each element that has a match in `old` swapped for it, in order: the same element, an
 * equal one, or a layout row alike enough to change inside, whose children are swapped then.
 */
function matched(old: readonly Element[], next: readonly Element[]): Element[] {
  const identical = new Set(next);
  const free = old.filter((each) => !identical.has(each));
  let from = 0;
  return next.map((each) => {
    if (identical.has(each) && old.includes(each)) return each;
    const rest = free.slice(from);
    const equal = rest.find((candidate) => candidate.isEqualNode(each));
    const alike = equal ?? rest.find((candidate) => sameRow(candidate, each));
    if (!alike) return each;
    from = free.indexOf(alike) + 1;
    if (!equal) alike.replaceChildren(...swapped([...alike.children], [...each.children]));
    return alike;
  });
}

/** Two rows of the library's layout with the same look: one can change inside into the other. */
function sameRow(a: Element, b: Element): boolean {
  return a.hasAttribute("data-layout") && a.className === b.className && a.getAttribute("style") === b.getAttribute("style");
}

/**
 * A pill caught as its content changes from `from` to `to`, its leaving and arriving content in
 * their rooms at the change's start: what a move's player plays.
 */
export function pillChanging(options: Omit<PillOptions, "content">, from: readonly Element[], to: readonly Element[]): HTMLElement {
  const made = makePill({ ...options, content: from });
  made.caught(to);
  return made.element;
}

/** A duration token in milliseconds, as the toolbar's speed and reduce motion scale it. */
function motion(name: string): number {
  const probe = document.createElement("span");
  probe.style.transitionDuration = `var(--motion-${name})`;
  document.body.append(probe);
  const ms = Number.parseFloat(getComputedStyle(probe).transitionDuration) * 1000;
  probe.remove();
  return ms;
}
