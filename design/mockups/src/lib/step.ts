// One step of a stepper or a checklist: `kubaos-ui`'s `step::step`, `KubaOS.Components`' `Step`
// (`docs/design/components/step.md`). An optional symbol (`icon.two_line`'s 24 beside a subtitle,
// `body_strong`'s 20 beside a title alone), a `body_strong` title on one line and a `small`
// subtitle that wraps to as many lines as it needs, then where it stands: a chip, or in a narrow
// rail a mark. `size.row` tall with one line; with a subtitle `space.s8` above and below, so
// `size.row_two_line` with its first line and a line taller for each line more; `space.s12` in at
// the sides, its corners its box's less the box's padding.
//
// To do is a Neutral chip, done an Accent chip with a check (a Neutral one on the selected row,
// where the accent's tint leaves its label under 4.5:1), an action a Filled chip with its symbol;
// the current step is the selected row, on `secondary_container`. In a rail the symbol is a bead on
// the rail's progress wire: lit in `primary` once its step is reached, the current one glowing at
// night and humming while it runs, and the current row is a quiet fill. A pressable step is a
// control: lit, the accent's tube inside its edge with the glow outside at night; pressed, the
// tube runs white-hot by night and the tone tints its fill by day.
//
// Where the row is too narrow for the chip beside the title's first word, the chip goes under the
// words, at their start, on a line of its own; in a list, every step's chip goes with it.

import "./feedback.css";
import { chip } from "./chip.ts";
import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { filling, row, stack } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { advance, clamped, wrapped } from "./text.ts";
import { length } from "./tokens.ts";
import { settings } from "../site/settings.ts";

export type StepState =
  | { readonly is: "todo" }
  | { readonly is: "current" }
  | { readonly is: "done" }
  /** Done as far as it can be, waiting on an action: its words (Reboot) and symbol. */
  | { readonly is: "action"; readonly label: string; readonly glyph: string };

/** The box a step sits in, which its corner follows: a list's well, or Kōbō's rail. */
export type StepBox = "well" | "rail";

export interface StepOptions {
  readonly title: string;
  readonly subtitle?: string;
  readonly icon?: string;
  /** The symbol in `error`: the step failed. */
  readonly failed?: boolean;
  readonly state?: StepState;
  /** Other words on the state's chip, in its look: Waiting, Not here. */
  readonly words?: { readonly label: string; readonly glyph?: string };
  /** The state as a mark at its end in place of a chip: a narrow rail's. */
  readonly mark?: boolean;
  /** The row picked; the current step is selected too. */
  readonly selected?: boolean;
  /** It takes a press. */
  readonly pressable?: boolean;
  /** The current step's work is running (the install): in a rail, its symbol hums. */
  readonly running?: boolean;
  readonly box?: StepBox;
  readonly pinned?: Pinned;
}

/** The chip at its end: to do, done or an action; the current step has none. */
function stateChip(state: StepState, selected: boolean, words: StepOptions["words"]): HTMLElement | null {
  const as = (() => {
    switch (state.is) {
      case "todo":
        return { kind: "Neutral", label: "To do" } as const;
      case "done":
        return { kind: selected ? "Neutral" : "Accent", label: "Done", glyph: "check" } as const;
      case "action":
        return { kind: "Filled", label: state.label, glyph: state.glyph } as const;
      case "current":
        return null;
    }
  })();
  if (!as) return null;
  const label = words?.label ?? as.label;
  const glyph = words ? words.glyph : "glyph" in as ? as.glyph : undefined;
  return chip({ label, kind: as.kind, ...(glyph ? { icon: glyph } : {}) });
}

/**
 * The mark at its end in a rail: a check when done, the action's symbol in the accent, other words'
 * symbol muted (Waiting, Not here), else its room.
 */
function stateMark(state: StepState, words?: StepOptions["words"]): Element {
  const size = "var(--type-body_strong-icon)";
  if (state.is === "done") return icon("check", size, "var(--step-ink)");
  if (state.is === "action") return icon(state.glyph, size, "var(--color-primary)");
  if (words?.glyph) return icon(words.glyph, size, "var(--step-muted)");
  const room = document.createElement("span");
  room.className = "k-step-mark-room";
  return room;
}

/** A rail step's symbol, as a bead on the rail's progress wire: what lights and hums. */
function bead(symbol: Element): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-step-bead";
  box.append(symbol);
  return box;
}

/**
 * Puts `end` under the words while the room beside it would leave the title less than its first
 * word. The room is the row's less the symbol, the chip and the gaps, read from layout, which the
 * chip's going under changes in height only, so the reading holds either way.
 */
function chipGivesWay(element: HTMLElement, line: HTMLElement, end: HTMLElement, title: string): void {
  const first = title.split(/\s+/)[0] ?? title;
  new ResizeObserver(() => {
    const scale = Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale")) || settings().scale;
    const between = length("space-s12") * scale;
    // The symbol may be an SVG, which has no offset width.
    const lead = line.firstElementChild!;
    const symbol = lead !== end && !lead.matches(".k-stack") ? Number.parseFloat(getComputedStyle(lead).width) + between : 0;
    const room = line.clientWidth - symbol - between - end.offsetWidth;
    element.toggleAttribute("data-chip-under", room < Math.ceil(advance("body_strong", first) * scale));
    // Steps side by side in a list keep their chips in one column: one under puts them all under.
    const list = element.parentElement;
    list?.toggleAttribute("data-chips-under", [...list.children].some((each) => each.hasAttribute("data-chip-under")));
  }).observe(line);
}

export function step(options: StepOptions): HTMLElement {
  const state = options.state ?? { is: "todo" };
  const selected = (options.selected ?? false) || state.is === "current";
  const two = options.subtitle !== undefined;
  const element = document.createElement("span");
  element.className = "k-step";
  element.dataset.box = options.box ?? "well";
  element.dataset.lines = two ? "two" : "one";
  element.dataset.state = state.is;
  element.toggleAttribute("data-failed", options.failed ?? false);
  element.toggleAttribute("data-running", options.running ?? false);
  element.toggleAttribute("data-selected", selected);
  const title = clamped(options.title, "body_strong");
  title.style.color = "var(--step-ink)";
  const lines: Element[] = [title];
  if (two) {
    const subtitle = wrapped(options.subtitle!, "small");
    subtitle.style.color = "var(--step-muted)";
    lines.push(subtitle);
  }
  const parts: Element[] = [];
  if (options.icon || options.failed) {
    const size = two ? "var(--icon-two_line)" : "var(--type-body_strong-icon)";
    const symbol = icon(options.failed ? "error" : options.icon!, size, options.failed ? "var(--color-error)" : "var(--bead-ink, var(--step-ink))");
    parts.push(options.box === "rail" ? bead(symbol) : symbol);
  }
  parts.push(filling(stack("s4", "start", ...lines)));
  const end = options.mark ? stateMark(state, options.words) : stateChip(state, selected, options.words);
  if (end) parts.push(end);
  const line = filling(row("s12", "centre", ...parts));
  element.append(line);
  if (end && !options.mark) chipGivesWay(element, line, end as HTMLElement, options.title);
  if (options.pressable) {
    pressable(element, true);
    pin(element, options.pinned);
  }
  const named = variant(state.is, options.icon ? "icon" : "", options.mark ? "mark" : "", options.failed ? "failed" : "", options.running ? "running" : "");
  return probe(element, { role: "component", name: "Step", variant: named, fixed: { width: false, height: false } });
}

/**
 * Moves a rail step drawn by `step` to `state`, in place: its look follows from its attributes and
 * its mark, so a rail changing page changes no more than that. Whether it takes a press, and a
 * failed symbol, are drawn by `step`.
 */
export function setStepState(element: HTMLElement, state: StepState, running: boolean): void {
  if (element.dataset.state === state.is && element.hasAttribute("data-running") === running) return;
  element.dataset.state = state.is;
  element.toggleAttribute("data-selected", state.is === "current");
  element.toggleAttribute("data-running", running);
  element.firstElementChild?.lastElementChild?.replaceWith(stateMark(state));
  const probed = element.dataset.probe ? (JSON.parse(element.dataset.probe) as { variant?: string }) : null;
  if (probed?.variant !== undefined) {
    const words = probed.variant.split(" ").filter((word) => !["todo", "done", "current", "action", "running"].includes(word));
    probed.variant = variant(state.is, ...words, running ? "running" : "");
    element.dataset.probe = JSON.stringify(probed);
  }
}

/**
 * Makes a step take a press, or stop taking one, in place: a control in the state model with its
 * focus ring (made the first time, kept after), or a plain row the keyboard passes by.
 */
export function pressable(element: HTMLElement, takes: boolean): void {
  element.classList.toggle("k-control", takes);
  element.toggleAttribute("data-hot", takes);
  if (takes) element.setAttribute("role", "button");
  else element.removeAttribute("role");
  element.tabIndex = takes ? 0 : -1;
  if (takes && !element.querySelector(":scope > .k-ring")) {
    const ring = focusRing(element, "row", "accent", false);
    ring.dataset.pressable = "";
    if (element.dataset.box === "rail") ring.style.setProperty("--corner", "var(--step-corner)");
  }
}
