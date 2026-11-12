// The RollingNumber page: new, in neither library yet. A count that rolls as it changes, as an
// odometer does, only the figures that change, in a box as wide as its widest value. Counters'
// values are its first use.

import { iconButton } from "../../lib/icon-button.ts";
import { row, stack } from "../../lib/layout.ts";
import { rollingMidway, rollingNumber } from "../../lib/rolling-number.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** The widest a counter's value is shown: three figures. */
const WIDEST = "000";

/** − and + round a rolling number from `from`, live: − stops at 0. */
function counting(from: number): HTMLElement {
  const shown = rollingNumber({ value: from, widest: WIDEST });
  const minus = iconButton({ icon: "remove", label: "−1", kind: "Tonal", size: "Sm", disabled: from === 0 });
  const plus = iconButton({ icon: "add", label: "+1", kind: "Filled", size: "Sm" });
  const to = (value: number) => {
    shown.set(Math.max(0, value));
    minus.disabled = shown.value() === 0;
  };
  minus.addEventListener("click", () => to(shown.value() - 1));
  plus.addEventListener("click", () => to(shown.value() + 1));
  return row("s8", "centre", minus, shown.element, plus);
}

/** A still rolling number at `value`. */
function still(value: number, widest = WIDEST): HTMLElement {
  return rollingNumber({ value, widest }).element;
}

/** The two figures of a roll, out and in, as tracks on the player's sample. */
function roll(name: string, from: number, to: number): Move {
  const way = to > from ? "up: the new from below" : "down: the new from above";
  return {
    name,
    what: `Only the figures that change roll, ${way}, over \`short\` on \`standard\`.`,
    tracks: [
      { part: "the figure leaving", on: "each figure that changes", target: ".k-roll-figure[data-out]", property: "--roll", from: 0, to: 1, duration: "short", curve: "standard" },
      { part: "the figure arriving", on: "each figure that changes", target: ".k-roll-figure[data-in]", property: "--roll", from: 0, to: 1, duration: "short", curve: "standard" },
    ],
    reduced: [
      { part: "the figure leaving", on: "each figure that changes", target: ".k-roll-figure[data-out]", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" },
      { part: "the figure arriving", on: "each figure that changes", target: ".k-roll-figure[data-in]", property: "--roll", from: 0, to: 1, duration: "instant", curve: "linear" },
    ],
    back: "cuts",
    sample: () => rollingMidway(from, to, WIDEST),
  };
}

export default page<"rest">({
  level: "atoms",
  slug: "rolling-number",
  title: "RollingNumber",
  lede: "A count that rolls as it changes, only its changing figures, in a box as wide as its widest value.",
  code: { unbuilt: "New: in neither library yet" },
  stage: {
    live: () => stack("s16", "end", counting(9), counting(99), counting(100)),
    hint: "press + and −: 9 → 10, 99 → 100, and back",
    daylight: true,
  },
  variants: [
    { name: "Going up", for: "the new figure in from below", sample: () => rollingMidway(99, 100, WIDEST) },
    { name: "Going down", for: "the new figure in from above", sample: () => rollingMidway(100, 99, WIDEST) },
  ],
  sizes: doesntApply("one size: `mono_large`, a counter's value beside its buttons."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "not a control: no pointer, no focus",
      pressed: "its buttons take the press",
      disabled: "a stopped count still shows its value",
      selected: "it holds no choice",
      loading: "a count is known at once",
      empty: "nothing counted shows 0",
      error: "a count can't be wrong",
    },
    rows: [
      { name: "0", cell: () => still(0) },
      { name: "42", cell: () => still(42) },
      { name: "999", cell: () => still(999) },
    ],
    looks: { rest: "`mono_large` figures in `on_surface`, at the box's end" },
  },
  focus: doesntApply("not a control: no line and no glow."),
  content: {
    text: doesntApply("figures, not words: the locale's, never padded."),
    missingPicture: doesntApply("it draws no picture."),
    counts: [
      { name: "0", for: "one figure at the box's end", sample: () => still(0) },
      { name: "999", for: "the box full: as wide as `000`", sample: () => still(999) },
      { name: "Past its widest", for: "1000 in a `000` box: it grows, once", sample: () => still(1000) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its row."),
    leaves: doesntApply("it goes with its row."),
    between: [roll("9 → 10", 9, 10), roll("10 → 9", 10, 9), roll("99 → 100", 99, 100), roll("100 → 99", 100, 99), ...powerMoves(() => still(42))],
  },
  keyboard: doesntApply("no focus and no key: its buttons take them."),
  measures: {
    samples: nonEmpty([
      { name: "Up to 999", for: "three slots a figure wide, at the end", sample: () => still(7) },
      { name: "Beside a word", for: "the word holds still as it rolls", sample: () => row("s8", "centre", still(128), text("glasses", "body", "on_surface_variant")) },
    ]),
    measures: [
      { part: "Line, slot height", token: "type-mono_large-line" },
      { part: "Figures", token: "type-mono_large-size" },
    ],
    reflow: doesntApply("as wide as its widest value at every window size."),
  },
});
