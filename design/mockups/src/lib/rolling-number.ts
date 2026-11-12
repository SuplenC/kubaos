// A count that rolls as it changes, as an odometer does: `RollingNumber`, new, in neither library
// yet. Its figures are tabular, each in a slot one figure wide and the style's line tall that cuts
// what leaves it (a figure's advance rounded up to a whole device pixel, so each starts on one),
// and the box is as many slots wide as the widest value it is given, its figures held to its end, so nothing beside it moves as the count grows a figure.
//
// Only the figures that change roll. Going up, the new one comes in from below as the old one leaves
// upward; going down, from above as the old leaves downward: `--roll` 0 to 1 over `short` on
// `standard`, the pair moving together. A figure the count grows comes in from nothing, and one it
// sheds leaves into nothing. With reduce motion the figures change in place.

import "./rolling-number.css";
import { FIXED_HEIGHT, probe } from "./probe.ts";
import { advance, centreFor, text, type TypeStyle } from "./text.ts";
import { raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** Every figure, so each one is centred for the same mass and they share one baseline. */
const FIGURES = "0123456789";

export interface RollingOptions {
  readonly value: number;
  /** The widest value it shows, its figures all 0: "000" for a count up to 999. */
  readonly widest: string;
  readonly style?: TypeStyle;
  readonly color?: string;
}

export interface Rolling {
  readonly element: HTMLElement;
  /** Rolls to `value`: the figures that differ roll, up or down by which way it went. */
  readonly set: (value: number) => void;
  readonly value: () => number;
}

/** The locale's figures for `value`: a count is never padded by hand. */
const COUNT = new Intl.NumberFormat("en-GB", { useGrouping: false });

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** One figure in its slot's ink, centred for every figure's mass. */
function figure(char: string, style: TypeStyle, color: string): HTMLElement {
  const made = text(char, style, color);
  made.classList.add("k-roll-figure");
  centreFor(made, style, FIGURES);
  return made;
}

/** A slot one figure wide, holding `char`, or nothing for a figure the count hasn't got. */
function slot(char: string, style: TypeStyle, color: string): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-roll-slot";
  if (char) box.append(figure(char, style, color));
  return box;
}

export function rollingNumber(options: RollingOptions): Rolling {
  const style = options.style ?? "mono_large";
  const color = options.color ?? "on_surface";
  const element = document.createElement("span");
  element.className = "k-rolling";
  element.setAttribute("role", "status");
  element.style.setProperty("--slots", String(options.widest.length));
  element.style.setProperty("--figure", String(advance(style, "0")));
  element.style.setProperty("--t-line", `var(--type-${style}-line)`);
  let value = options.value;
  let shown = COUNT.format(value);
  element.append(...[...shown].map((char) => slot(char, style, color)));
  element.setAttribute("aria-label", shown);

  function set(next: number): void {
    if (next === value) return;
    const way = next > value ? 1 : -1;
    const words = COUNT.format(next);
    const width = Math.max(words.length, shown.length);
    const [was, now] = [shown.padStart(width, " "), words.padStart(width, " ")];
    while (element.children.length < width) element.prepend(slot("", style, color));
    const slots = [...element.children] as HTMLElement[];
    const reduced = settings().reduced;
    slots.forEach((box, index) => {
      if (was[index] === now[index]) return;
      const char = now[index]!.trim();
      for (const left of box.querySelectorAll("[data-out]")) left.remove();
      const old = box.querySelector<HTMLElement>(".k-roll-figure");
      const fresh = char ? figure(char, style, color) : null;
      if (reduced) {
        old?.remove();
        if (fresh) box.append(fresh);
        return;
      }
      box.style.setProperty("--way", String(way));
      if (old) old.dataset.out = "";
      if (fresh) {
        fresh.dataset.in = "";
        box.append(fresh);
      }
      const roll = box.animate([{ "--roll": 0 }, { "--roll": 1 }], { duration: ms("short"), easing: raw("easing-standard") });
      roll.finished.then(() => {
        old?.remove();
        if (fresh) delete fresh.dataset.in;
      }, () => {});
    });
    // A slot the count shed goes once its figure has rolled away; the box keeps its width.
    const shed = width - words.length;
    if (shed > 0) setTimeout(() => slots.slice(0, shed).forEach((box) => box.childElementCount === 0 && box.remove()), reduced ? 0 : ms("short"));
    if (reduced) slots.slice(0, shed).forEach((box) => box.remove());
    value = next;
    shown = words;
    element.setAttribute("aria-label", shown);
  }

  probe(element, { role: "component", name: "RollingNumber", variant: style, fixed: FIXED_HEIGHT });
  return { element, set, value: () => value };
}

/**
 * A rolling number caught as it starts to roll from `from` to `to`, its leaving and arriving figures
 * both in their slots at `--roll` 0: what a move's player rolls.
 */
export function rollingMidway(from: number, to: number, widest: string, style: TypeStyle = "mono_large"): HTMLElement {
  const made = rollingNumber({ value: from, widest, style });
  const way = to > from ? 1 : -1;
  const [a, b] = [COUNT.format(from), COUNT.format(to)];
  const width = Math.max(a.length, b.length);
  const [was, now] = [a.padStart(width, " "), b.padStart(width, " ")];
  while (made.element.children.length < width) made.element.prepend(slot("", style, "on_surface"));
  ([...made.element.children] as HTMLElement[]).forEach((box, index) => {
    if (was[index] === now[index]) return;
    box.style.setProperty("--way", String(way));
    box.querySelector<HTMLElement>(".k-roll-figure")?.setAttribute("data-out", "");
    const char = now[index]!.trim();
    if (char) {
      const fresh = figure(char, style, "on_surface");
      fresh.dataset.in = "";
      box.append(fresh);
    }
  });
  return made.element;
}
