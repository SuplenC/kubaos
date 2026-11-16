// The Stepper page: `docs/design/components/stepper.md`. A page's own steps on one line under its
// title: a bead with its number and a label each; the current one lit, the done ones filled and
// pressable to go back.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { setStep, stepper } from "../../lib/stepper.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const ROOM = "var(--size-column)";
const STEPS = ["What for", "Your apps", "Go"];

const LABEL: Words = { short: "Go", long: "Check what goes on, then install it all" };

function still(current: number, steps: readonly string[] = STEPS): HTMLElement {
  return within(ROOM, stepper({ steps, current, onBack: () => undefined }));
}

/** The Stepper with Back and Next under it, moving it as a page's steps would. */
function live(): HTMLElement {
  let at = 0;
  const line = stepper({ steps: STEPS, current: at, onBack: (to) => go(to) });
  const go = (to: number) => {
    at = Math.max(0, Math.min(STEPS.length - 1, to));
    setStep(line, at, (index) => go(index));
  };
  const back = button({ label: "Back", kind: "Text", icon: "arrow_back" });
  back.addEventListener("click", () => go(at - 1));
  const next = button({ label: "Next", kind: "Filled", icon: "arrow_forward", trailing: true });
  next.addEventListener("click", () => go(at + 1));
  return within(ROOM, stack("s24", "start", line, row("s16", "centre", back, next)));
}

const ON = "a Stepper on its second step";

const MOVES_ON: Move = {
  name: "Moving on a step",
  what: "The step left fills over `short`; the next one's ring, number and label light over `short`.",
  tracks: [
    { part: "the step left", on: ON, target: ".k-stepper-step:nth-child(1) .k-stepper-bead", property: "opacity", from: { opacity: "subdued" }, to: 1, duration: "short", curve: "standard" },
    { part: "the step lit", on: ON, target: ".k-stepper-step:nth-child(2)", property: "opacity", from: { opacity: "subdued" }, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [
    { part: "the step left", on: ON, target: ".k-stepper-step:nth-child(1) .k-stepper-bead", property: "opacity", from: { opacity: "subdued" }, to: 1, duration: "instant", curve: "standard" },
    { part: "the step lit", on: ON, target: ".k-stepper-step:nth-child(2)", property: "opacity", from: { opacity: "subdued" }, to: 1, duration: "instant", curve: "standard" },
  ],
  back: "cuts",
  sample: () => still(1),
};

export default page<"rest" | "selected">({
  level: "molecules",
  slug: "stepper",
  title: "Stepper",
  lede: "A page's own few steps on one line under its title: where you are, and back a step.",
  code: { unbuilt: "neither library has it yet" },
  stage: { live, hint: "Next and Back move it; a done step takes a press", daylight: true },
  variants: [
    { name: "At the first", for: "nothing done", sample: () => still(0) },
    { name: "Along the way", for: "one done, one current", sample: () => still(1) },
    { name: "At the last", for: "all before it done", sample: () => still(2) },
  ],
  sizes: doesntApply("one size: its beads are `type.body`'s symbol, its labels `label`."),
  states: {
    shown: ["rest", "selected"],
    absent: {
      lit: "its beads take no keyboard: Back and Esc go back a step",
      pressed: "a done step goes back at once; nothing stays pressed",
      disabled: "while work runs, done steps take no press",
      loading: "the page shows its own work",
      empty: "a page with steps has two at least",
      error: "a failed step is its page's error block",
    },
    rows: [{ name: "Three steps", cell: (state) => still(state === "rest" ? 0 : 1) }],
    looks: {
      rest: "the first lit, the others rings",
      selected: "the first filled, the second lit",
    },
  },
  focus: doesntApply("its beads take no keyboard focus; the page's Back and Esc go back a step."),
  content: {
    text: [{ name: "A label", words: LABEL, sample: (words: string) => still(2, ["What for", "Your apps", words]) }],
    missingPicture: doesntApply("its beads are drawn, their numbers type."),
    counts: [
      { name: "Two", for: "the fewest", sample: () => still(1, ["Pick", "Go"]) },
      { name: "Three", for: "Make it yours", sample: () => still(1) },
      { name: "Five", for: "the most a line holds well", sample: () => still(2, ["One", "Two", "Three", "Four", "Five"]) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows with its page, already where the page is."),
    leaves: doesntApply("it goes with its page."),
    between: [MOVES_ON],
  },
  keyboard: doesntApply("its beads take no focus: Back and Esc go back a step, Next and Enter on."),
  measures: {
    samples: nonEmpty([{ name: "Along the way", for: "beads, labels and gaps", sample: () => still(1) }]),
    measures: [
      { part: "Bead", token: "type-body-icon" },
      { part: "Ring", token: "stroke-thin" },
      { part: "Bead → label", token: "space-s8" },
      { part: "Steps apart", token: "space-s24" },
      { part: "Glow", token: "focus-near" },
    ],
    reflow: { note: "Its steps wrap onto a second line where the room ends, `space.s8` under the first.", sample: () => within("100%", stepper({ steps: [shown(LABEL), "Your apps", "Go"], current: 1 })) },
  },
});
