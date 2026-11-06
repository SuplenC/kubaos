// The ProgressPanel page: `docs/design/components/progress-panel.md`. A job followed to its end in
// the Command menu's progress View: its state as the room's one big symbol, its title, a bar while
// it knows how far it is, and what it last said.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { type Job, progressPanel } from "../../lib/progress-panel.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** The Command menu's room for a View: its panel's width less its padding, a few rows tall. */
const ROOM = "var(--size-column)";
const TALL = "calc(4 * var(--space-s64))";

const TITLE: Words = { short: "Updating", long: "Updating KubaOS and every package pinned in your user layer" };
const SAID: Words = { short: "Fetching", long: "Fetching 214 packages from the cache, 1.2 GB in all" };

const STARTING: Job = { title: "Updating KubaOS", state: "running", progress: -1, message: "Resolving the flake" };
const FETCHING: Job = { title: "Updating KubaOS", state: "running", progress: 0.42, message: "Fetching packages" };
const DONE: Job = { title: "Updating KubaOS", state: "succeeded", progress: 1, message: "Reboot to use it" };
const FAILED: Job = { title: "Updating KubaOS", state: "failed", progress: 0.42, message: "The cache didn't answer" };

/** `job`'s panel in a View's room as wide as a column, as tall as it is. */
function inRoom(job: Job): HTMLElement {
  return within(ROOM, progressPanel(job));
}

/**
 * The job's life in a View's room a few rows tall, the panel centred in it: Next moves it on, from
 * can't tell, through how far, to done or failed.
 */
function live(): HTMLElement {
  const lives: readonly Job[] = [STARTING, { ...FETCHING, progress: 0.12 }, FETCHING, { ...FETCHING, progress: 0.8, message: "Building the system" }, DONE, FAILED];
  let at = 1;
  const room = within(ROOM, progressPanel(lives[at]!));
  const draw = () => {
    const panel = progressPanel(lives[at]!);
    panel.style.setProperty("--room-height", TALL);
    room.replaceChildren(panel);
  };
  draw();
  const next = button({ label: "Next", kind: "Tonal", size: "Sm", icon: "arrow_forward" });
  next.addEventListener("click", () => {
    at = (at + 1) % lives.length;
    draw();
  });
  return stack("s16", "start", room, next);
}

const BAR = "a running job";

const FOLLOWS: Move = {
  name: "The bar follows the job",
  what: "The bar's fill eases on over `short` on `linear`; the line says the new percent at once.",
  tracks: [{ part: "the bar's fill", on: BAR, target: ".k-progress", property: "--progress", from: 0.12, to: 0.42, duration: "short", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => inRoom({ ...FETCHING, progress: 0.12 }),
};

export default page<"loading" | "error">({
  level: "molecules",
  slug: "progress-panel",
  title: "ProgressPanel",
  lede: "A job followed to its end: its state as one big symbol, its title, a bar, and its last words.",
  code: { qml: "ProgressPanel", unbuilt: "QML only; Rust not built yet" },
  stage: { live, hint: "press Next to move the job on", daylight: true },
  variants: [
    { name: "Can't tell yet", for: "no progress yet: the spinner and its line", sample: () => inRoom(STARTING) },
    { name: "Running", for: "the hourglass over its bar", sample: () => inRoom(FETCHING) },
    { name: "Done", for: "the check, the bar full, what to do next", sample: () => inRoom(DONE) },
    { name: "Failed", for: "the error symbol and line in `error`, no bar", sample: () => inRoom(FAILED) },
  ],
  sizes: doesntApply("its column is `size.column` wide at most, its room's width below that."),
  states: {
    shown: ["loading", "error"],
    absent: {
      rest: "done is a variant of its life",
      lit: "it takes no pointer or focus",
      pressed: "it takes no press",
      disabled: "a job is never disabled",
      selected: "it holds no choice",
      empty: "with no job the View isn't open",
    },
    rows: [
      { name: "Can't tell", cell: (state) => inRoom(state === "error" ? { ...FAILED, progress: -1 } : STARTING) },
      { name: "Knows how far", cell: (state) => inRoom(state === "error" ? FAILED : FETCHING) },
    ],
    looks: {
      loading: "spinner, else the hourglass over its bar, then its line",
      error: "`error` symbol and line, no bar",
    },
  },
  focus: doesntApply("the panel takes no focus: the View's Esc goes back."),
  content: {
    text: [
      { name: "Its title", words: TITLE, sample: (words: string) => inRoom({ ...FETCHING, title: words }) },
      { name: "What it last said", words: SAID, sample: (words: string) => inRoom({ ...FETCHING, message: words }) },
    ],
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, chosen by its state."),
    counts: [
      { name: "Nothing said", for: "its line is its state alone", sample: () => inRoom({ ...FETCHING, message: "" }) },
      { name: "One job", for: "others run on as rows in the menu's list", sample: () => inRoom(FETCHING) },
    ],
  },
  motion: {
    arrives: doesntApply("it arrives with its View."),
    leaves: doesntApply("it leaves with its View; the job runs on."),
    between: [FOLLOWS, ...powerMoves(() => inRoom(FETCHING))],
  },
  keyboard: {
    keys: [{ keys: ["Esc"], does: "back to the list; the job runs on" }],
    order: "No stop: nothing in the panel takes the keyboard.",
    lands: "On the View; going back, on the job's row.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Running", for: "column, gaps, symbol, bar", sample: () => inRoom(FETCHING) },
      { name: "Can't tell yet", for: "the spinner in the symbol's place", sample: () => inRoom(STARTING) },
    ]),
    measures: [
      { part: "Column at most", token: "size-column" },
      { part: "Between parts", token: "space-s16" },
      { part: "Symbol or spinner", token: "icon-s48" },
      { part: "Title line", token: "type-headline-line" },
      { part: "Bar", token: "size-progress" },
      { part: "Last line", token: "type-body-line" },
    ],
    reflow: {
      note: "`size.column` wide at most, centred; in a narrower room its title and line wrap.",
      sample: () => within("100%", progressPanel({ ...FETCHING, title: shown(TITLE), message: shown(SAID) })),
    },
  },
});
