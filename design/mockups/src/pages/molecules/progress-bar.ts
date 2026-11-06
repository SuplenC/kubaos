// The ProgressBar page: `docs/design/components/progress-bar.md`. Work that knows how far it is: a
// pill across its room, its fill following the work and glowing in its own colour at night; the
// danger kind in a Danger banner; the timed kind a held button draws.

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
  what: "The fill eases to the new fraction over `short` on `linear`, its glow with it.",
  tracks: [{ part: "the fill", on: BAR, target: ".k-progress", property: "--progress", from: 0.3, to: 0.7, duration: "short", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => within(ROOM, progressBar({ fraction: 0.3 })),
};

const DANGER_FOLLOWS: Move = {
  name: "Following the work, danger",
  what: "The same in a Danger banner, in `error`.",
  tracks: [{ part: "the fill", on: "a danger bar", target: ".k-progress", property: "--progress", from: 0.3, to: 0.7, duration: "short", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => within(ROOM, progressBar({ fraction: 0.3, danger: true })),
};

export default page<"empty" | "loading">({
  level: "molecules",
  slug: "progress-bar",
  title: "ProgressBar",
  lede: "Work that knows how far it is: a pill across its room, its fill following the work.",
  code: { rust: "progress::bar", qml: "ProgressBar", unbuilt: "day without glow: not built yet" },
  stage: { live, hint: "press Fetch a chunk", daylight: true },
  variants: [
    { name: "Following", for: "a download, an update, a copy", sample: () => labelled("Downloading the system image", 0.42) },
    { name: "Timed", for: "a fraction the app times itself: a held button", sample: () => within(ROOM, progressBar({ fraction: 0.6, timed: true })) },
    { name: "Danger", for: "a Danger banner: the hold that erases a disk", sample: () => labelled("Erasing the disk", 0.42, true) },
    { name: "With a count", for: "steps of a whole, `space.s4` over its `small` count", sample: () => counted(3, 5) },
    { name: "In a row", for: "a job in the Command menu: the row's last quarter", sample: () => jobRow(0.62) },
    { name: "Can't tell yet", for: "a spinner and its words where the bar will be", sample: cantTell },
  ],
  sizes: doesntApply("one height, `size.progress`; its width is its room's."),
  states: {
    shown: ["empty", "loading"],
    absent: {
      rest: "it shows only while its work runs",
      lit: "it takes neither pointer nor keyboard",
      pressed: "it takes no press",
      disabled: "work not running shows no bar",
      selected: "it holds no choice",
      error: "failed work takes its bar away: an `ErrorBlock` says why",
    },
    rows: [
      { name: "Following", cell: (state) => within(ROOM, progressBar({ fraction: state === "empty" ? 0 : 0.42 })) },
      { name: "Danger", cell: (state) => within(ROOM, progressBar({ fraction: state === "empty" ? 0 : 0.42, danger: true })) },
    ],
    looks: {
      empty: "the track alone, no fill or glow",
      loading: "`primary` fill to the fraction, glow at night, none by day",
    },
  },
  focus: doesntApply("a bar isn't a control: it takes no focus; its glow is a light."),
  content: {
    text: [{ name: "Its label", words: DOING, sample: (words: string) => labelled(words, 0.42) }],
    missingPicture: doesntApply("a bar is drawn."),
    counts: [
      { name: "Nothing", for: "the track alone", sample: () => within(ROOM, progressBar({ fraction: 0 })) },
      { name: "A sliver", for: "at least a device pixel wide", sample: () => within(ROOM, progressBar({ fraction: 0.005 })) },
      { name: "Full", for: "the fill across the whole track", sample: () => within(ROOM, progressBar({ fraction: 1 })) },
    ],
  },
  motion: {
    arrives: doesntApply("a bar shows with its work, already where the work is."),
    leaves: doesntApply("it goes with its work's place."),
    between: [FOLLOWS, DANGER_FOLLOWS, ...powerMoves(() => within(ROOM, progressBar({ fraction: 0.6 })))],
  },
  keyboard: doesntApply("a bar takes no focus or key; its label is read out."),
  measures: {
    samples: nonEmpty([
      { name: "A bar", for: "height and fill", sample: () => within(ROOM, progressBar({ fraction: 0.42 })) },
      { name: "Labelled", for: "the label above it and its gap", sample: () => labelled("Downloading the system image", 0.42) },
      { name: "With a count", for: "the count under it and its gap", sample: () => counted(3, 5) },
    ]),
    measures: [
      { part: "Height", token: "size-progress" },
      { part: "Glow blur", token: "shadow-glow-blur" },
      { part: "Label → bar", token: "space-s8" },
      { part: "Bar → count", token: "space-s4" },
      { part: "Label → percent", token: "space-s12" },
      { part: "Label line", token: "type-label-line" },
    ],
    reflow: {
      note: "It fills its room, the fill the same fraction; its label ends in an ellipsis before the percent.",
      sample: () => labelled(shown(DOING), 0.42, false, "100%"),
    },
  },
});
