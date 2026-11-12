// The InlineEdit page: new, in neither library yet. A row's text edited where it is, in a box of its
// own that moves nothing: a counter's name and its value, on its 56 two-line row.

import type { Words } from "../../lib/content.ts";
import { inlineEdit } from "../../lib/inline-edit.ts";
import { clamped } from "../../lib/text.ts";
import { within } from "../../lib/layout.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { COUNTERS, LONG_NAME, liveRow, rowAlone } from "../shell/counters-parts.ts";

type Shown = "rest" | "lit" | "error";

const COFFEE = COUNTERS[0]!;

/** The row, its name or its value editing. */
function editing(what?: "name" | "value", options: { current?: boolean } = {}): HTMLElement {
  return rowAlone(COFFEE, { widest: "000", ...(what ? { editing: what } : {}), ...(options.current ? { current: true } : {}) });
}

/** The name alone mid-shake, `at` of the way out to one side. */
function shaking(at: number): HTMLElement {
  const made = inlineEdit({ value: "Coffee", style: "body_strong", rest: (value) => clamped(value, "body_strong"), check: () => null, editing: true, shake: at });
  return within("var(--size-menu)", made.element);
}

const SHAKE: Move = {
  name: "Can't take it",
  what: "Enter on an empty name or a count that isn't one: the box shakes once, `space.s4` each way.",
  tracks: [
    { part: "the box, out", on: "the name's box", target: ".k-inline-edit", property: "--shake", from: 0, to: -1, duration: "micro", curve: "standard" },
    { part: "the box, across", on: "the name's box", target: ".k-inline-edit", property: "--shake", from: -1, to: 1, duration: "micro", cue: "micro", curve: "standard" },
    { part: "the box, back", on: "the name's box", target: ".k-inline-edit", property: "--shake", from: 1, to: 0, duration: "micro", cue: ["micro", "micro"], curve: "standard" },
  ],
  reduced: [{ part: "the box, still", on: "the name's box", target: ".k-inline-edit", property: "--shake", from: 0, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => shaking(0),
};

const NAME: Words = { short: "Coffee", long: LONG_NAME };

export default page<Shown>({
  level: "molecules",
  slug: "inline-edit",
  title: "InlineEdit",
  lede: "A row's text edited where it is, in a box of its own that moves nothing.",
  code: { unbuilt: "New: in neither library yet" },
  stage: { live: () => liveRow(), hint: "double-click the name or the value; F2, Shift+F2", daylight: true },
  variants: [
    { name: "A name", for: "from the start; never empty", sample: () => editing("name") },
    { name: "A count", for: "to the end, figures only, held to its stop", sample: () => editing("value") },
  ],
  sizes: doesntApply("its text's line tall: `body_strong`'s for a name, `mono_large`'s for a count."),
  states: {
    shown: ["rest", "lit", "error"],
    absent: {
      pressed: "a double click turns it editable",
      disabled: "a row that can't change has no InlineEdit",
      selected: "editing selects all of it: Lit",
      loading: "its value is there",
      empty: "a name can't be empty: Error",
    },
    rows: [
      { name: "A name", cell: (state: Shown) => (state === "rest" ? editing() : state === "lit" ? editing("name", { current: true }) : shaking(-1)) },
      { name: "A count", cell: (state: Shown) => (state === "rest" ? editing() : state === "lit" ? editing("value", { current: true }) : shaking(1)) },
    ],
    looks: {
      rest: "the row's own text, nothing round it",
      lit: "editing: `surface_container_highest`, `primary` underline, all selected",
      error: "Enter on what it can't take: one shake, the old value kept",
    },
  },
  focus: doesntApply("its row is the stop; editing shows the caret and the underline, no tube."),
  content: {
    text: [{ name: "A long name", words: NAME, sample: (words: string) => rowAlone({ ...COFFEE, name: words }, { widest: "000", editing: "name" }) }],
    missingPicture: doesntApply("it is words."),
    counts: doesntApply("one text each: a row holds one name and one value."),
  },
  motion: {
    arrives: doesntApply("editable at once: nothing moves."),
    leaves: doesntApply("back to rest at once; a saved count rolls (RollingNumber)."),
    between: [SHAKE, ...powerMoves(() => editing("name", { current: true }))],
  },
  keyboard: {
    keys: [
      { keys: ["F2"], does: "on the focused row: edits the name" },
      { keys: ["Shift+F2"], does: "on the focused row: edits the value" },
      { keys: ["Enter"], does: "saves; one it can't take shakes" },
      { keys: ["Esc"], does: "cancels, the old value back" },
      { keys: ["Tab", "Shift+Tab"], does: "cancels, as a click away does" },
    ],
    order: "Its row's stop: editing holds the keyboard until it ends.",
    lands: "Editing, in the box, all selected; ended, back on its row.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The name editing", for: "the box past the text, the underline", sample: () => editing("name") },
      { name: "The value editing", for: "to the end, the box's width", sample: () => editing("value") },
    ]),
    measures: [
      { part: "Box past the text", token: "space-s4" },
      { part: "Box fill", token: "color-surface_container_highest" },
      { part: "Underline", token: "stroke-thin" },
      { part: "Box corners", token: "radius-small" },
      { part: "Selection", token: "opacity-selection" },
      { part: "Row", token: "size-row_two_line" },
    ],
    reflow: doesntApply("the name takes its row's free room; the count its box."),
  },
});
