// A few steps inside one page, on one line under its title: `Stepper`
// (`docs/design/components/stepper.md`). Each step is a bead with its number and a `label` beside
// it, `space.s8` apart, the steps `space.s24` apart. A step not reached is a ring in `outline` with
// its number and label in `on_surface_variant`; the current one's ring and number are `primary`,
// glowing near at night as the rail's lit symbols do, its label `primary_lit`; one done is a filled
// bead in `primary_fill` with its number in `on_primary_fill`, and takes a press to go back to it.
// Moving on, the bead left fills over `short` and the next one lights over `short` after it.
//
// It says where the reader is among a page's own steps (first boot's Make it yours); the pages of a
// flow are a StepRailPage's rail, never a Stepper.

import { probe, variant } from "./probe.ts";
import { text } from "./text.ts";

export interface StepperOptions {
  readonly steps: readonly string[];
  /** The step shown, by its index. */
  readonly current: number;
  /** What pressing a done step does; none where going back isn't possible (work running). */
  readonly onBack?: (index: number) => void;
}

type State = "done" | "current" | "todo";

const stateAt = (index: number, current: number): State => (index < current ? "done" : index === current ? "current" : "todo");

/** Gives `step` its state: its look, whether it takes a press, and what it says it is. */
function setState(step: HTMLButtonElement, state: State, onBack?: (index: number) => void): void {
  step.dataset.state = state;
  step.disabled = !(state === "done" && onBack !== undefined);
  step.toggleAttribute("aria-current", state === "current");
  if (state === "current") step.setAttribute("aria-current", "step");
}

/** One step: its bead with its number, then its label, a button only while it is done. */
function bead(label: string, index: number, state: State, onBack?: (index: number) => void): HTMLButtonElement {
  const step = document.createElement("button");
  step.type = "button";
  step.className = "k-stepper-step";
  // Back and Esc go back a step from the keyboard; the beads are for the pointer.
  step.tabIndex = -1;
  step.addEventListener("click", () => {
    if (step.dataset.state === "done") onBack?.(index);
  });
  const disc = document.createElement("span");
  disc.className = "k-stepper-bead";
  disc.append(text(String(index + 1), "label_number"));
  step.append(disc, text(label, "label"));
  setState(step, state, onBack);
  return step;
}

export function stepper(options: StepperOptions): HTMLElement {
  const line = document.createElement("span");
  line.className = "k-stepper";
  line.append(...options.steps.map((label, index) => bead(label, index, stateAt(index, options.current), options.onBack)));
  return probe(line, { role: "component", name: "Stepper", variant: variant(`${options.steps.length}-steps`), fixed: { width: false, height: true } });
}

/** Moves `line` to `current` in place: each bead and label fades to its new look over `short`. */
export function setStep(line: HTMLElement, current: number, onBack?: (index: number) => void): void {
  line.querySelectorAll<HTMLButtonElement>(":scope > .k-stepper-step").forEach((step, index) => setState(step, stateAt(index, current), onBack));
}
