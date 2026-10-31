// The Spinner page: `docs/design/components/spinner.md`. Work that doesn't know how far it is: a
// thin arc that turns and breathes over a faint full ring, an icon size across. It isn't a
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
  s16: "Beside `small` or `label` text: a running job's row in the Command menu.",
  s20: "Beside `body` text: Kōbō's step that is running, a line saying what is going on.",
  s24: "Beside a `subheading`.",
  s32: "A page's or a panel's one spinner, above its words.",
  s48: "A page's one spinner: a progress panel's, Kōbō's while it looks at the hardware.",
};

/** A spinner beside a line of body text saying what is going on, in a row `space.s12` apart. */
function doing(words = shown(DOING), color = "primary"): HTMLElement {
  return within("var(--size-column)", row("s12", "centre", spinner("s20", color), filling(wrapped(words, "body", "on_surface_variant"))));
}

/** The specimens' row: 16, 20 and 24, then what is going on. */
function specimen(): HTMLElement {
  return row("s24", "centre", spinner("s16"), spinner("s20"), spinner("s24"), text("Poking at the hardware…", "body", "on_surface_variant"));
}

const DESCRIBED =
  "A faint full ring, its colour at `opacity.spinner_track`, and over it an arc in the colour with round ends, both the thin stroke (`stroke.thin`, rounded half up to whole device pixels: 2, 3, 3, 4 at 1, 1.25, 1.5 and 2), the ring's middle a stroke in from the box. The arc turns 0.8 of a turn a second, on `linear`, and breathes between 0.45 and 1.05 of a half turn, 0.6 breaths a second, so its shortest arc still reads as a spinner and its loop comes round every 5 s without a jump.";

export default page<"loading">({
  level: "atoms",
  slug: "spinner",
  title: "Spinner",
  summary: "Work that doesn't know how far it is: a thin arc that turns and breathes over a faint full ring, an icon size across.",
  live: () => stack("s32", "start", specimen(), row("s32", "centre", spinner("s32"), spinner("s48"), spinner("s20", "error"))),
  description: [
    "Rust `progress::spinner(look, size, seconds, color)`, QML `Spinner` (`size`, `color`, `running`). It is for work that can't say how far it has got; work that can is a progress bar. Only such work spins: a running animation makes every frame wait on the screen's refresh, typing's included, so a job that can say how far it is shows that instead.",
    DESCRIBED,
    "The caller passes the time: Rust takes the seconds since the work began, QML runs its own clock while `running` and visible, so a spinner nobody sees costs no frames. It is `primary`, or `error` for work that destroys something (a danger action going down).",
  ],
  variants: [
    { name: "Beside text", for: "A line saying what is going on: 16, 20 or 24, the size of the text's icon.", sample: () => doing() },
    { name: "A page's one", for: "Above its words, 32 or 48: a progress panel's.", sample: () => stack("s16", "centre", spinner("s48"), text("Looking at the hardware", "subheading")) },
    { name: "Danger", for: "Work that destroys something, in `error`: a danger action going down.", sample: () => doing("Erasing the disk…", "error") },
  ],
  sizes: nonEmpty(SPINNER_SIZES.map((size) => ({ name: size.slice(1), for: FOR[size], sample: () => spinner(size) }))),
  states: {
    shown: ["loading"],
    absent: {
      rest: "a spinner shows only while its work runs; when the work ends it is replaced, by a mark of how it went or by what arrived.",
      lit: "it takes no pointer and no focus, and isn't neon.",
      pressed: "it takes no press.",
      disabled: "work that can't run shows no spinner.",
      selected: "it holds no choice.",
      empty: "a spinner is always turning while it shows.",
      error: "work that failed replaces its spinner with an `error` symbol; a danger spinner is a tone, not a state.",
    },
    rows: [
      { name: "16", cell: () => spinner("s16") },
      { name: "20", cell: () => spinner("s20") },
      { name: "24", cell: () => spinner("s24") },
      { name: "48", cell: () => spinner("s48") },
      { name: "Danger, 20", cell: () => spinner("s20", "error") },
    ],
    described: {
      loading: `Turning. ${DESCRIBED} By day the same, in the day's accent and red.`,
    },
  },
  focus: doesntApply("a spinner isn't a control and isn't neon: it has no line and no glow, by night or by day."),
  content: {
    text: [{ name: "What is going on", words: DOING, sample: (words: string) => doing(words) }],
    missingPicture: doesntApply("a spinner is drawn."),
    counts: [
      { name: "One", for: "A page's one spinner, or one line of work.", sample: () => doing() },
      {
        name: "Many",
        for: "Each line of work that can't tell how far it is spins on its own: the Command menu's running jobs, each at 16 on the same clock.",
        sample: () => stack("s8", "start", row("s8", "centre", spinner("s16"), text("Updating KubaOS", "small", "on_surface_variant")), row("s8", "centre", spinner("s16"), text("Taking a snapshot", "small", "on_surface_variant"))),
      },
    ],
  },
  motion: {
    arrives: doesntApply("a spinner shows at once with its work, already turning."),
    leaves: doesntApply("it goes at once when its work ends, replaced in its place."),
    between: powerMoves(specimen),
  },
  keyboard: doesntApply("a spinner takes no focus and no key; its line of words is read out."),
  measures: {
    samples: nonEmpty([
      { name: "20", for: "Its box, an icon size.", sample: () => spinner("s20") },
      { name: "Beside text", for: "The gap to its words.", sample: () => row("s12", "centre", spinner("s20"), text("Connecting…", "body", "on_surface_variant")) },
    ]),
    measures: [
      { what: "The smallest spinner, beside small text.", token: "icon-s16" },
      { what: "Beside body text.", token: "icon-s20" },
      { what: "Beside a subheading.", token: "icon-s24" },
      { what: "A page's one spinner.", token: "icon-s48" },
      { what: "The ring's and the arc's stroke, before it is rounded to whole device pixels.", token: "stroke-thin" },
      { what: "The gap between a spinner and its words.", token: "space-s12" },
    ],
    reflow: doesntApply("a spinner is its size at every window size; the words beside it wrap."),
  },
});
