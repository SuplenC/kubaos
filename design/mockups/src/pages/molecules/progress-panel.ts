// The ProgressPanel page: `docs/design/components/progress-panel.md`. A job followed to its end in
// the Command menu's progress View: its state as the room's one big symbol, its title, a bar while
// it knows how far it is, and what it last said.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { type Job, progressPanel } from "../../lib/progress-panel.ts";
import { text } from "../../lib/text.ts";
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
  what: "The job says it is further on: its bar's fill eases there over `short` on `linear`, as every progress bar follows its work, and the line under it says the new percent at once. Nothing else moves.",
  tracks: [{ part: "the bar's fill", on: BAR, target: ".k-progress", property: "--progress", from: 0.12, to: 0.42, duration: "short", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => inRoom({ ...FETCHING, progress: 0.12 }),
};

const DESCRIBED =
  "A column `size.column` wide at most, centred in its room on whole device pixels, its parts `space.s16` apart and centred: the job's state as the room's one big symbol (`icon.s48`), its title in `headline`, a ProgressBar across the column while the job knows how far it is, and a `body` line in `on_surface_variant` with its state and what it last said (\"42% · Fetching packages\"). Its words wrap, centred.";

export default page<"loading" | "error">({
  level: "molecules",
  slug: "progress-panel",
  title: "ProgressPanel",
  summary: "A job followed to its end: its state as one big symbol, its title, a bar while it knows how far it is, and what it last said.",
  live,
  description: [
    "QML `ProgressPanel` (`job`, `moves`), the Command menu's progress View: an Item that runs a job opens it, and the job's own row in the menu's list shows the same state at 16. Its `kubaos-ui` twin comes with the list and progress molecules. Press Next to move the job through its life.",
    DESCRIBED,
    "`job` is `{title, state, progress, message}`: `state` running, succeeded or failed, `progress` from 0 to 1, or below 0 while it can't tell. While it can't tell, the symbol is a 48 Spinner and there is no bar; once it can, an hourglass in `primary` over the bar. It succeeds with `check_circle` in `primary` and the bar full; it fails with `error` in `error`, the line in `error` and no bar.",
    "Only a job that can't tell how far it is spins, since a running animation makes every frame wait on the screen's refresh; with the menu's motion set to fades (`moves` off) the spinner stands still. The View takes no keys of its own: Esc goes back to the list as every View does, and the job runs on, its row in the list following it.",
  ],
  variants: [
    { name: "Can't tell yet", for: "A job that hasn't said how far it is: the spinner, and its line.", sample: () => inRoom(STARTING) },
    { name: "Running", for: "A job that knows how far it is: the hourglass over its bar.", sample: () => inRoom(FETCHING) },
    { name: "Done", for: "It succeeded: the check, the bar full, what to do next.", sample: () => inRoom(DONE) },
    { name: "Failed", for: "It failed: the error symbol and its line in `error`, no bar.", sample: () => inRoom(FAILED) },
  ],
  sizes: doesntApply("its column is `size.column` wide at most, its room's width below that; its parts keep their own sizes."),
  states: {
    shown: ["loading", "error"],
    absent: {
      rest: "a job that is done is a variant of its life, not a resting state: Done above.",
      lit: "the panel takes no pointer and no focus; Esc is the View's.",
      pressed: "it takes no press.",
      disabled: "a job is running, done or failed; never disabled.",
      selected: "it holds no choice.",
      empty: "a panel always follows a job; with none, the View isn't open.",
    },
    rows: [
      { name: "Can't tell", cell: (state) => inRoom(state === "error" ? { ...FAILED, progress: -1 } : STARTING) },
      { name: "Knows how far", cell: (state) => inRoom(state === "error" ? FAILED : FETCHING) },
    ],
    described: {
      loading: "Running: the spinner while it can't tell, else the hourglass in `primary` over its bar; its line says how far and what it last said, in `on_surface_variant`. By day the same roles in daylight colours.",
      error: "Failed: the `error` symbol in `error`, its line in `error`, and no bar, whether it knew how far it was or not.",
    },
  },
  focus: doesntApply("the panel takes no focus: it shows a job, and the View's Esc goes back."),
  content: {
    text: [
      { name: "Its title", words: TITLE, sample: (words: string) => inRoom({ ...FETCHING, title: words }) },
      { name: "What it last said", words: SAID, sample: (words: string) => inRoom({ ...FETCHING, message: words }) },
    ],
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, chosen by its state."),
    counts: [
      { name: "Nothing said", for: "A job with no message: its line is its state alone.", sample: () => inRoom({ ...FETCHING, message: "" }) },
      { name: "One job", for: "The View follows the one job it was opened for; others run on as rows in the menu's list.", sample: () => inRoom(FETCHING) },
    ],
  },
  motion: {
    arrives: doesntApply("it arrives with its View, which slides in over `medium` as every Command menu View does."),
    leaves: doesntApply("it leaves with its View; the job runs on."),
    between: [FOLLOWS, ...powerMoves(() => inRoom(FETCHING))],
  },
  keyboard: {
    keys: [{ keys: ["Esc"], does: "Back to the Command menu's list, as every View: the job runs on, its row following it." }],
    order: "No stop: nothing in the panel takes the keyboard.",
    lands: "On the View, which takes Esc; going back, on the job's row in the list.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Running", for: "Its column, its gaps, the symbol and the bar.", sample: () => inRoom(FETCHING) },
      { name: "Can't tell yet", for: "The spinner in the symbol's place.", sample: () => inRoom(STARTING) },
    ]),
    measures: [
      { what: "Its column at most.", token: "size-column" },
      { what: "Between its parts.", token: "space-s16" },
      { what: "The symbol, or the spinner.", token: "icon-s48" },
      { what: "The title's line, `headline`'s.", token: "type-headline-line" },
      { what: "The bar.", token: "size-progress" },
      { what: "The line's, `body`'s.", token: "type-body-line" },
    ],
    reflow: {
      described: "Its column is `size.column` wide at most and centred in its room; in a narrower room it takes the room's width, its title and line wrapping.",
      sample: () => within("100%", stack("s8", "start", progressPanel({ ...FETCHING, title: shown(TITLE), message: shown(SAID) }), text("A progress panel in a room as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
