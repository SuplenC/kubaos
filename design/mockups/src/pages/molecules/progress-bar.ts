// The ProgressBar page: `docs/design/components/progress-bar.md`. Work that knows how far it is: a
// pill across its room, its fill following the work and glowing in its own colour at night; the
// danger kind on a danger banner's red; the timed kind a held button draws.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { figure, tick } from "../../lib/figure.ts";
import { icon } from "../../lib/icon.ts";
import { clamped, text } from "../../lib/text.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { spinner } from "../../lib/spinner.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const ROOM = "var(--size-column)";

const DOING: Words = { short: "Downloading", long: "Downloading the system image from the mirror closest to you" };

/** A bar with its label above it: what is going on at the start, how far at the end. */
function labelled(words: string, fraction: number, danger = false, room = ROOM): HTMLElement {
  const percent = figure(`${Math.round(fraction * 100).toLocaleString()}%`, "label_number", "on_surface_variant");
  const head = row("s12", "centre", filling(clamped(words, "label", "on_surface_variant")), percent);
  return within(room, stack("s8", "start", filling(head), progressBar({ fraction, danger })));
}

/** A bar over its count, `space.s4` under it in `small`: Kōbō's setup touching a key's slots. */
function counted(done: number, of: number): HTMLElement {
  return within(ROOM, stack("s4", "start", progressBar({ fraction: done / of }), text(`${done.toLocaleString()} of ${of.toLocaleString()}`, "small", "on_surface_variant")));
}

/** A job's row in the Command menu: its state and words, the bar in the last quarter. */
function jobRow(fraction: number): HTMLElement {
  const said = row("s8", "centre", icon("hourglass_top", "var(--icon-s16)", "var(--color-primary)"), clamped("Updating KubaOS · Fetching packages", "small", "on_surface_variant"));
  const bar = progressBar({ fraction });
  const room = within("calc(var(--size-column) / 4)", bar);
  return within(ROOM, row("s12", "centre", filling(said), room));
}

/** Work that can't tell how far it is yet: a spinner and its words in the bar's place. */
function cantTell(): HTMLElement {
  return within(ROOM, row("s12", "centre", spinner("s20"), text("Looking for mirrors…", "body", "on_surface_variant")));
}

const CHUNKS = 8;

/** A download in chunks: each press of the button fetches one, and the bar follows it. */
function live(): HTMLElement {
  let done = 3;
  const bar = progressBar({ fraction: done / CHUNKS });
  const percent = figure("38%", "label_number", "on_surface_variant");
  const words = (count: number) => `${Math.round((count / CHUNKS) * 100).toLocaleString()}%`;
  tick(percent, "label_number", words(done));
  const head = row("s12", "centre", filling(clamped("Downloading the system image", "label", "on_surface_variant")), percent);
  const next = button({ label: "Fetch a chunk", kind: "Tonal", size: "Sm", icon: "download" });
  next.addEventListener("click", () => {
    done = done >= CHUNKS ? 0 : done + 1;
    setProgress(bar, done / CHUNKS);
    tick(percent, "label_number", words(done));
  });
  return stack("s24", "start", within(ROOM, stack("s8", "start", filling(head), bar)), next);
}

const BAR = "a bar";

const FOLLOWS: Move = {
  name: "Following the work",
  what: "The work moves on: the fill eases from where it was to the new fraction over `short` on `linear`, so it moves with the work instead of jumping, its glow with it. Work that reports often keeps it gliding; a report while it moves starts from where it is. Reduce motion caps every move at `short`, so it plays the same.",
  tracks: [{ part: "the fill", on: BAR, target: ".k-progress", property: "--progress", from: 0.3, to: 0.7, duration: "short", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => within(ROOM, progressBar({ fraction: 0.3 })),
};

const DANGER_FOLLOWS: Move = {
  name: "Following the work, danger",
  what: "The same on a danger banner's red: the `error` fill eases to the new fraction over `short` on `linear`.",
  tracks: [{ part: "the fill", on: "a danger bar", target: ".k-progress", property: "--progress", from: 0.3, to: 0.7, duration: "short", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => within(ROOM, progressBar({ fraction: 0.3, danger: true })),
};

const DESCRIBED =
  "A pill `size.progress` (8) tall across its room, at `radius.full`. The track is `surface_container_highest` at `opacity.track`; the fill runs from the left edge to the work's fraction of the width, rounded up to a device pixel, in `primary`, with `shadow.glow` under it in its own colour (12 blur at 0.55): a light, so it glows at night while the signs are on, flickers with them in a power change, and is out by day. A fill of no width draws nothing, its glow included.";

export default page<"empty" | "loading">({
  level: "molecules",
  slug: "progress-bar",
  title: "ProgressBar",
  summary: "Work that knows how far it is: a pill across its room, its fill following the work and glowing in its own colour at night.",
  live,
  description: [
    "Rust `progress::bar(look, fraction)`, `danger_bar`, `timed_bar` and `danger_timed_bar`; QML `ProgressBar` (`value`, `danger`). Press Fetch a chunk: the fill follows each chunk, and the percent beside it ticks in place.",
    DESCRIBED,
    "The fill follows the work over `short` on `linear`, the curve of a real quantity; a bar that shows up already part full starts there, never growing from nothing. A timed bar is drawn where it is given: the app already times the fraction against the clock (how long a held button has been held, `progress::timed_bar`), so no easing trails behind it. Kōbō's install bar is timed too: the app eases its percent on every tick.",
    "On a danger banner's red the fill is `error` on a track of `on_error_container` at `opacity.tint`, so the hold that erases a disk on Kōbō's Last chance reads as red all the way. The bar has no words of its own: what is going on and how far sit beside it as the page's text, a `label` line above it with the percent at its end as a figure that holds its width, or a `small` count `space.s4` under it.",
    "It is for work that can say how far it has got. Work that can't, yet, shows a spinner and its words in the bar's place, since a running animation makes every frame wait on the screen's refresh; once the work can tell, the bar takes its place.",
  ],
  variants: [
    { name: "Following", for: "Work that reports how far it is: a download, an update, a copy.", sample: () => labelled("Downloading the system image", 0.42) },
    { name: "Timed", for: "A fraction the app times itself: how long a held button has been held. Drawn where it is given.", sample: () => within(ROOM, progressBar({ fraction: 0.6, timed: true })) },
    { name: "Danger", for: "On a danger banner's red: the hold that erases a disk.", sample: () => labelled("Erasing the disk", 0.42, true) },
    { name: "With a count", for: "Steps of a whole: the slots of a key touched, `space.s4` over its `small` count.", sample: () => counted(3, 5) },
    { name: "In a row", for: "A running job in the Command menu: its state and words, the bar in the last quarter of the row.", sample: () => jobRow(0.62) },
    { name: "Can't tell yet", for: "Work that can't say how far it is shows a spinner and its words where the bar will be.", sample: cantTell },
  ],
  sizes: doesntApply("a bar has one height, `size.progress`; its width is its room's."),
  states: {
    shown: ["empty", "loading"],
    absent: {
      rest: "a bar shows only while its work runs; done, it is full for as long as its place keeps it (a progress panel's job that succeeded).",
      lit: "it takes neither the pointer nor the keyboard.",
      pressed: "it takes no press.",
      disabled: "work that isn't running shows no bar.",
      selected: "it holds no choice.",
      error: "work that fails takes its bar away: an ErrorBlock, or a progress panel's error line, says what broke.",
    },
    rows: [
      { name: "Following", cell: (state) => within(ROOM, progressBar({ fraction: state === "empty" ? 0 : 0.42 })) },
      { name: "Danger", cell: (state) => within(ROOM, progressBar({ fraction: state === "empty" ? 0 : 0.42, danger: true })) },
    ],
    described: {
      empty: "Nothing done yet: the track alone, the fill and its glow not drawn. By day the same roles in daylight colours.",
      loading: `Part way: ${DESCRIBED} By day the fill is the day's \`primary\` (or \`error\`) with no glow.`,
    },
  },
  focus: doesntApply("a bar isn't a control: it takes no focus. Its glow is a light, lit at night while the signs are on."),
  content: {
    text: [{ name: "Its label", words: DOING, sample: (words: string) => labelled(words, 0.42) }],
    missingPicture: doesntApply("a bar is drawn."),
    counts: [
      { name: "Nothing", for: "0: the track alone.", sample: () => within(ROOM, progressBar({ fraction: 0 })) },
      { name: "A sliver", for: "Just begun: the fill at least a device pixel wide, its rounded ends squeezed into it.", sample: () => within(ROOM, progressBar({ fraction: 0.005 })) },
      { name: "Full", for: "Done: the fill across the whole track.", sample: () => within(ROOM, progressBar({ fraction: 1 })) },
    ],
  },
  motion: {
    arrives: doesntApply("a bar shows with its work, already where the work is."),
    leaves: doesntApply("it goes with its work's place: a panel or a screen moving on."),
    between: [FOLLOWS, DANGER_FOLLOWS, ...powerMoves(() => within(ROOM, progressBar({ fraction: 0.6 })))],
  },
  keyboard: doesntApply("a bar takes no focus and no key; its label is read out."),
  measures: {
    samples: nonEmpty([
      { name: "A bar", for: "Its height and its fill.", sample: () => within(ROOM, progressBar({ fraction: 0.42 })) },
      { name: "Labelled", for: "The label above it and its gap.", sample: () => labelled("Downloading the system image", 0.42) },
      { name: "With a count", for: "The count under it and its gap.", sample: () => counted(3, 5) },
    ]),
    measures: [
      { what: "Its height.", token: "size-progress" },
      { what: "Its glow's blur, under the fill.", token: "shadow-glow-blur" },
      { what: "Between the label and the bar.", token: "space-s8" },
      { what: "Between the bar and its count.", token: "space-s4" },
      { what: "Between the label and its percent.", token: "space-s12" },
      { what: "The label's line, `label`'s.", token: "type-label-line" },
    ],
    reflow: {
      described: "It fills its room at every window size, its fill the same fraction of it; its label ends in an ellipsis before the percent, which keeps its place.",
      sample: () => within("100%", stack("s8", "start", labelled(shown(DOING), 0.42, false, "100%"), text("A bar as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
