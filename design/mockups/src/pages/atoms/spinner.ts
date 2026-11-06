// The Spinner page: `docs/design/components/spinner.md`. Work that doesn't know how far it is: a
// lit stretch of neon turning round an unlit glass ring, an icon size across. It isn't a
// control and isn't neon: it turns while the work runs, and a power change only recolours it.

import { shown, type Words } from "../../lib/content.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { SPINNER_SIZES, type SpinnerSize, spinner } from "../../lib/spinner.ts";
import { text, wrapped } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const DOING: Words = {
  short: "Connecting…",
  long: "Poking at the hardware to find every disk KubaOS could be installed on, and what is on each one…",
};

/** What each size is for. */
const FOR: Record<SpinnerSize, string> = {
  s16: "beside `small` or `label`: a job's row",
  s20: "beside `body`: a line of what's going on",
  s24: "beside a `subheading`",
  s32: "a panel's one, above its words",
  s48: "a page's one, a progress panel's",
};

/** A spinner beside a line of body text saying what is going on, in a row `space.s12` apart. */
function doing(words = shown(DOING), color = "primary"): HTMLElement {
  return within("var(--size-column)", row("s12", "centre", spinner("s20", color), filling(wrapped(words, "body", "on_surface_variant"))));
}

/** The specimens' row: 16, 20 and 24, then what is going on. */
function specimen(): HTMLElement {
  return row("s24", "centre", spinner("s16"), spinner("s20"), spinner("s24"), text("Poking at the hardware…", "body", "on_surface_variant"));
}

export default page<"loading">({
  level: "atoms",
  slug: "spinner",
  title: "Spinner",
  lede: "Work that can't say how far it is: a stretch of neon turning round a glass ring.",
  code: { rust: "progress::spinner", qml: "Spinner" },
  stage: {
    live: () => row("s48", "centre", specimen(), spinner("s32"), spinner("s48"), spinner("s20", "error")),
    hint: "it keeps turning under reduced motion",
    daylight: true,
  },
  variants: [
    { name: "Beside text", for: "a line saying what is going on", sample: () => doing() },
    { name: "A page's one", for: "above its words, 32 or 48", sample: () => stack("s16", "centre", spinner("s48"), text("Looking at the hardware", "subheading")) },
    { name: "Danger", for: "work that destroys something, in `error`", sample: () => doing("Erasing the disk…", "error") },
  ],
  sizes: nonEmpty(SPINNER_SIZES.map((size) => ({ name: size.slice(1), for: FOR[size], sample: () => spinner(size) }))),
  states: {
    shown: ["loading"],
    absent: {
      rest: "it shows only while its work runs",
      lit: "no pointer and no focus",
      pressed: "it takes no press",
      disabled: "work that can't run shows none",
      selected: "it holds no choice",
      empty: "it always turns while it shows",
      error: "failed work swaps it for an `error` symbol",
    },
    rows: [
      { name: "16", cell: () => spinner("s16") },
      { name: "20", cell: () => spinner("s20") },
      { name: "24", cell: () => spinner("s24") },
      { name: "48", cell: () => spinner("s48") },
      { name: "Danger, 20", cell: () => spinner("s20", "error") },
    ],
    looks: {
      loading: "`spinner.arc` of the ring turns every `spinner.turn` on `linear`",
    },
  },
  focus: doesntApply("not a control: no focus; its arc is always lit."),
  content: {
    text: [{ name: "What is going on", words: DOING, sample: (words: string) => doing(words) }],
    missingPicture: doesntApply("it is drawn."),
    counts: [
      { name: "One", for: "a page's one, or one line of work", sample: () => doing() },
      {
        name: "Many",
        for: "each line of work its own, on one clock",
        sample: () => stack("s8", "start", row("s8", "centre", spinner("s16"), text("Updating KubaOS", "small", "on_surface_variant")), row("s8", "centre", spinner("s16"), text("Taking a snapshot", "small", "on_surface_variant"))),
      },
    ],
  },
  motion: {
    arrives: doesntApply("it shows at once with its work, already turning."),
    leaves: doesntApply("it goes at once when its work ends, replaced in its place."),
    between: powerMoves(specimen),
  },
  keyboard: doesntApply("no focus and no key; its line of words is read out."),
  measures: {
    samples: nonEmpty([
      { name: "20", for: "its box", sample: () => spinner("s20") },
      { name: "Beside text", for: "the gap to its words", sample: () => row("s12", "centre", spinner("s20"), text("Connecting…", "body", "on_surface_variant")) },
    ]),
    measures: [
      { part: "16", token: "icon-s16" },
      { part: "20", token: "icon-s20" },
      { part: "24", token: "icon-s24" },
      { part: "48", token: "icon-s48" },
      { part: "Ring and arc stroke", token: "stroke-thin" },
      { part: "Spinner → words", token: "space-s12" },
    ],
    reflow: doesntApply("its size at every window size; the words beside it wrap."),
  },
});
