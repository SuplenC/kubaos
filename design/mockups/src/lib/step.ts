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
// the current step is the selected row, on `secondary_container`. A pressable step is a control:
// lit, the accent's tube inside its edge with the glow outside at night; pressed, the tube runs
// white-hot by night and the tone tints its fill by day.

import "./feedback.css";
import { chip } from "./chip.ts";
import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { filling, row, stack } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { clamped, wrapped } from "./text.ts";

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

/** The mark at its end in a rail: a check when done, the action's symbol in the accent, else its room. */
function stateMark(state: StepState): Element {
  const size = "var(--type-body_strong-icon)";
  if (state.is === "done") return icon("check", size, "var(--step-ink)");
  if (state.is === "action") return icon(state.glyph, size, "var(--color-primary)");
  const room = document.createElement("span");
  room.className = "k-step-mark-room";
  return room;
}

export function step(options: StepOptions): HTMLElement {
  const state = options.state ?? { is: "todo" };
  const selected = (options.selected ?? false) || state.is === "current";
  const two = options.subtitle !== undefined;
  const element = document.createElement("span");
  element.className = "k-step";
  element.dataset.box = options.box ?? "well";
  element.dataset.lines = two ? "two" : "one";
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
    parts.push(icon(options.failed ? "error" : options.icon!, size, options.failed ? "var(--color-error)" : "var(--step-ink)"));
  }
  parts.push(filling(stack("s4", "start", ...lines)));
  const end = options.mark ? stateMark(state) : stateChip(state, selected, options.words);
  if (end) parts.push(end);
  element.append(filling(row("s12", "centre", ...parts)));
  if (options.pressable) {
    element.classList.add("k-control");
    element.dataset.hot = "";
    element.setAttribute("role", "button");
    element.tabIndex = 0;
    pin(element, options.pinned);
    const ring = focusRing(element, "row", "accent", false);
    ring.dataset.pressable = "";
    if (options.box === "rail") ring.style.setProperty("--corner", "var(--step-corner)");
  }
  const named = variant(state.is, options.icon ? "icon" : "", options.mark ? "mark" : "", options.failed ? "failed" : "");
  return probe(element, { role: "component", name: "Step", variant: named, fixed: { width: false, height: false } });
}
