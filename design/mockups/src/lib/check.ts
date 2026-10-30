// A checkbox and a radio: `kubaos-ui`'s `check::checkbox` and `check::radio`, `KubaOS.Components`'
// `Checkbox` and `Radio`. A box `size.check` across (a square at the small radius, or a disc),
// then its label, `space.s12` from it, all one press target (`docs/design/components/
// checkbox.md`, `radio.md`).
//
// Their states are the neon's (`.k-control` in `lib.css`), with the line on the box. Unpicked,
// nothing is filled with the accent: the line is the tube over the box's outline. Picked, the box
// is filled with the accent, so at night it rests on the unlit fill, and its line is the core
// inside the fill's edge.

import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { row } from "./layout.ts";
import { probe } from "./probe.ts";
import { text } from "./text.ts";

/** How much of something a checkbox has picked. */
export const TICKS = ["Empty", "Partial", "Full"] as const;
export type Tick = (typeof TICKS)[number];

interface CheckOptions {
  readonly label: string;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

export interface CheckboxOptions extends CheckOptions {
  readonly tick?: Tick;
}

export interface RadioOptions extends CheckOptions {
  readonly checked?: boolean;
}

/** A picked box's mark, `--checked` showing it, in the colour the CSS works out. */
function marked(box: HTMLElement, mark: Element): void {
  const holder = document.createElement("span");
  holder.className = "k-check-mark";
  holder.append(mark);
  box.append(holder);
}

/**
 * The control: a box of `shape` with its marks, its ring at the radius token `corner`, then
 * `label`. Returns the control and the ring, which `turn` fills when it is picked.
 */
function control(shape: "box" | "disc", corner: string, options: CheckOptions, marks: Element[]): { element: HTMLButtonElement; ring: HTMLElement } {
  const element = document.createElement("button");
  element.className = "k-check k-control";
  element.type = "button";
  element.dataset.hot = "";
  element.disabled = options.disabled ?? false;
  pin(element, options.pinned);
  const box = document.createElement("span");
  box.className = "k-check-box";
  box.dataset.shape = shape;
  for (const mark of marks) marked(box, mark);
  const ring = focusRing(box, corner, "accent", false);
  ring.dataset.pressable = "";
  const words = text(options.label, "body");
  words.style.color = "var(--words)";
  element.append(row("s12", "centre", box, words));
  return { element, ring };
}

/** Sets whether `element` is picked, and so whether its line is the core inside the fill. */
function turn(element: HTMLElement, ring: HTMLElement, picked: boolean): void {
  element.style.setProperty("--checked", picked ? "1" : "0");
  ring.toggleAttribute("data-filled", picked);
}

/** Shows `tick` on a checkbox: none, its dash or its tick. */
function setTick(element: HTMLElement, ring: HTMLElement, tick: Tick): void {
  element.dataset.tick = tick;
  element.setAttribute("aria-checked", tick === "Full" ? "true" : tick === "Partial" ? "mixed" : "false");
  turn(element, ring, tick !== "Empty");
}

export function checkbox(options: CheckboxOptions): HTMLButtonElement {
  const dash = document.createElement("span");
  dash.className = "k-check-dash";
  const tick = icon("check", "var(--icon-s16)", "var(--mark)");
  tick.classList.add("k-check-tick");
  const { element, ring } = control("box", "small", options, [tick, dash]);
  element.setAttribute("role", "checkbox");
  setTick(element, ring, options.tick ?? "Empty");
  element.addEventListener("click", () => setTick(element, ring, element.dataset.tick === "Full" ? "Empty" : "Full"));
  return probe(element, { role: "component", name: "Checkbox", variant: "", fixed: { width: false, height: true } });
}

export function radio(options: RadioOptions): HTMLButtonElement {
  const dot = document.createElement("span");
  dot.className = "k-check-dot";
  const { element, ring } = control("disc", "full", options, [dot]);
  element.setAttribute("role", "radio");
  const pick = (picked: boolean) => {
    element.setAttribute("aria-checked", String(picked));
    turn(element, ring, picked);
  };
  pick(options.checked ?? false);
  element.addEventListener("click", () => {
    // Exactly one of a group is picked: picking one unpicks the others beside it.
    for (const other of element.closest("[data-radio-group]")?.querySelectorAll<HTMLButtonElement>(".k-check[role=radio]") ?? []) {
      if (other !== element) other.dispatchEvent(new CustomEvent("unpick"));
    }
    pick(true);
  });
  element.addEventListener("unpick", () => pick(false));
  return probe(element, { role: "component", name: "Radio", variant: "", fixed: { width: false, height: true } });
}

/** `radios` as one group, side by side `gap` apart: picking one unpicks the rest. */
export function radioGroup(gap: string, ...radios: HTMLButtonElement[]): HTMLElement {
  const group = row(gap, "start", ...radios);
  group.dataset.radioGroup = "";
  return group;
}
