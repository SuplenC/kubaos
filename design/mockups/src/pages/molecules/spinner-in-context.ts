// The Spinner in context page: `docs/design/components/spinner.md`. The Spinner atom where the
// builds place it: in a button's place, leading a row, under a field, and for a page loading,
// each at the size of the text beside it.

import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { shown, type Words } from "../../lib/content.ts";
import { field } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { type SpinnerSize, spinner } from "../../lib/spinner.ts";
import { clamped, text, type TypeStyle, wrapped } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";

const ROOM = "var(--size-column)";

const DOING: Words = { short: "Going down…", long: "Unmounting the disks and powering off; the machine goes dark in a moment, so pull the stick once it does…" };

/** How work that ends turns out, where its spinner was. */
type Outcome = "running" | "done" | "failed";

/** A spinner `size` beside `words` in `style`, `gap` apart: what is going on, said beside it. */
function beside(size: SpinnerSize, words: string, style: TypeStyle, gap: string, color = "primary"): HTMLElement {
  return row(gap, "centre", spinner(size, color), filling(wrapped(words, style, "on_surface_variant")));
}

/** The work's mark at `size`: the spinner while it runs, then how it went. */
function mark(outcome: Outcome, size: SpinnerSize): Element {
  if (outcome === "running") return spinner(size);
  return outcome === "done" ? icon("check_circle", `var(--icon-${size})`, "var(--color-primary)") : icon("error", `var(--icon-${size})`, "var(--color-error)");
}

/**
 * In a button's place: Kōbō's done screen, Reboot pressed, gives its place to the spinner and what
 * is going on; Stay in the live system keeps its own.
 */
function inPlace(pressed: boolean, words = "Going down…", danger = false): HTMLElement {
  const shownPart = pressed ? beside("s20", words, "body", "s12", danger ? "error" : "primary") : button({ label: "Reboot", size: "Lg", icon: "power_settings_new" });
  return within(ROOM, row("s12", "centre", shownPart, button({ label: "Stay in the live system", kind: "Text", size: "Lg" })));
}

/** A dialog's actions all waiting on the work: the power menu going down, in `error` when it loses work. */
function dialogActions(danger: boolean): HTMLElement {
  return within(ROOM, beside("s20", danger ? "Stopping the install and powering off…" : "Powering off…", "body", "s12", danger ? "error" : "primary"));
}

/** Kōbō's install steps: each row's mark, its title, and its time at its end. */
function installRow(title: string, outcome: Outcome | "waiting", time: string): HTMLElement {
  const lead = outcome === "waiting" ? icon("circle", "var(--icon-s20)", "var(--color-outline_variant)") : mark(outcome, "s20");
  const style: TypeStyle = outcome === "running" || outcome === "failed" ? "body_strong" : "body";
  const colour = outcome === "failed" ? "error" : outcome === "waiting" ? "on_surface_variant" : "on_surface";
  return filling(row("s12", "centre", lead, filling(clamped(title, style, colour)), text(time, "mono", "on_surface_variant")));
}

function installSteps(running: Outcome = "running"): HTMLElement {
  return within(ROOM, stack("s8", "start", installRow("Partitioning", "done", "0:04"), installRow("Copying the system", running, "1:12"), installRow("Installing the boot loader", "waiting", "")));
}

/** A job's row in the Command menu: the 16 spinner beside its `small` words, `space.s8` apart. */
function jobRow(outcome: Outcome = "running"): HTMLElement {
  return row("s8", "centre", mark(outcome, "s16"), clamped("Taking a snapshot · Copying /home", "small", outcome === "failed" ? "error" : "on_surface_variant"));
}

/** The connect form's field, its `small` status line `space.s8` under it saying what is going on. */
function underField(status: string): HTMLElement {
  const typed = field({ value: "correct horse battery", secret: true });
  return within(ROOM, stack("s8", "start", filling(typed), text(status, "small", "on_surface_variant")));
}

/** A page's one spinner: first boot's step running, at 48 beside its heading, in its card. */
function pageLoading(outcome: Outcome = "running"): HTMLElement {
  const words = stack("s8", "start", wrapped(outcome === "failed" ? "It didn't take." : "Setting up the fingerprint reader", "heading"), wrapped("Nothing for you to do. Watch the log if you like that sort of thing.", "small", "on_surface_variant"));
  return within(ROOM, card({ body: [row("s20", "centre", mark(outcome, "s48"), filling(words))] }));
}

/** A panel's one spinner, above its words: the progress panel while its job can't tell. */
function aboveWords(): HTMLElement {
  return stack("s16", "centre", spinner("s48"), text("Updating KubaOS", "headline"), text("Running · Resolving the flake", "body", "on_surface_variant"));
}

const PAIRS: readonly { readonly size: SpinnerSize; readonly style: TypeStyle; readonly gap: string; readonly words: string; readonly for: string }[] = [
  { size: "s16", style: "small", gap: "s8", words: "Checking again…", for: "Beside `small` or `label` text, `space.s8` apart: a running job in the Command menu, first boot checking again." },
  { size: "s20", style: "body", gap: "s12", words: "Poking at the hardware…", for: "Beside `body` text, `space.s12` apart: Kōbō looking at the machine, a step running, a button's place." },
  { size: "s24", style: "subheading", gap: "s12", words: "Looking for disks", for: "Beside a `subheading`, `space.s12` apart." },
  { size: "s48", style: "heading", gap: "s20", words: "Setting up the reader", for: "A page's one spinner beside its `heading`, `space.s20` apart, or above its words, `space.s16`." },
];

export default page<"loading" | "rest" | "error">({
  level: "molecules",
  slug: "spinner-in-context",
  title: "Spinner in context",
  summary: "The spinner where work that can't tell how far it is shows: in a button's place, leading a row, under a field, and for a page loading.",
  live: () => stack("s32", "start", inPlace(true), installSteps(), pageLoading()),
  description: [
    "The Spinner atom (`docs/design/components/spinner.md`) placed in the parts round it. It is for work that can't say how far it has got; work that can shows a ProgressBar. Its size is the icon size of the text beside it, so it sits on that text's line like any symbol, and its gap is the symbol's: `space.s8` beside `small`, `space.s12` beside `body`.",
    "In a button's place: a button that starts work it can't measure gives its place to a 20 spinner and a `body` line saying what is going on, `on_surface_variant`, `space.s12` apart. On Kōbō's done screen Reboot gives its place and Stay in the live system keeps its own; in the power menu every action waits on the work, so the spinner and its words take all their places. Work that destroys something spins in `error`. A spinner never sits inside a button: a button is something to press, and work going on isn't.",
    "Leading a row: a row of work (Kōbō's install steps, the Command menu's running jobs) leads with its mark, the spinner while it runs; once it ends, the mark says how it went, `check_circle` in `primary` or `error` in `error`, in the spinner's place, so nothing moves. Under a field: the form's `small` status line says what is going on in words (Connecting…), its room kept while it is empty.",
    "For a page loading: the page's one spinner, 48, beside its `heading` in a card (first boot's step running) or above its words (a progress panel whose job can't tell). Only work that can't tell spins: a running animation makes every frame wait on the screen's refresh, so the moment it can tell, a bar takes its place.",
  ],
  variants: [
    { name: "In a button's place", for: "A pressed button whose work can't be measured: the spinner and its words where the buttons were.", sample: () => stack("s16", "start", inPlace(false), inPlace(true)) },
    { name: "In the actions' place", for: "A dialog whose actions all wait on the work: the power menu going down.", sample: () => dialogActions(false) },
    { name: "In the actions' place, danger", for: "Work that loses something, in `error`: powering off mid-install.", sample: () => dialogActions(true) },
    { name: "Leading a row", for: "A step running among done and waiting ones: Kōbō's install.", sample: () => installSteps() },
    { name: "Leading a job's row", for: "The Command menu's running job, 16 beside `small`.", sample: jobRow },
    { name: "Under a field", for: "The connect form's status line: what is going on, in words under the field.", sample: () => underField("Connecting…") },
    { name: "A page loading, beside its heading", for: "First boot's step running: 48 beside its `heading`, in its card.", sample: () => pageLoading() },
    { name: "A page loading, above its words", for: "A progress panel's job that can't tell: 48 above its title.", sample: aboveWords },
  ],
  sizes: nonEmpty(PAIRS.map((pair) => ({ name: pair.size.slice(1), for: pair.for, sample: () => within(ROOM, beside(pair.size, pair.words, pair.style, pair.gap)) }))),
  states: {
    shown: ["loading", "rest", "error"],
    absent: {
      lit: "a spinner takes no pointer and no focus; what is round it keeps its own states.",
      pressed: "it takes no press: the button that started the work has given its place to it.",
      disabled: "work that can't run shows no spinner.",
      selected: "it holds no choice.",
      empty: "a spinner always turns while it shows.",
    },
    rows: [
      { name: "A row", cell: (state) => installSteps(state === "loading" ? "running" : state === "rest" ? "done" : "failed") },
      { name: "A job", cell: (state) => jobRow(state === "loading" ? "running" : state === "rest" ? "done" : "failed") },
      { name: "A page", cell: (state) => pageLoading(state === "loading" ? "running" : state === "rest" ? "done" : "failed") },
    ],
    described: {
      loading: "Running: the spinner turning at the size of the text beside it, its words saying what is going on.",
      rest: "Done: `check_circle` in `primary` in the spinner's place, at its size, so nothing moves.",
      error: "Failed: `error` in `error` in the spinner's place; a row's title turns `error` and `body_strong`.",
    },
  },
  focus: doesntApply("a spinner takes no focus; the parts round it keep their own."),
  content: {
    text: [{ name: "What is going on", words: DOING, sample: (words: string) => within(ROOM, beside("s20", words, "body", "s12")) }],
    missingPicture: doesntApply("a spinner is drawn."),
    counts: [
      { name: "One", for: "A page's one spinner, or one line of work.", sample: () => inPlace(true) },
      { name: "Many", for: "Each line of work that can't tell spins on its own, on one clock: the Command menu's running jobs.", sample: () => stack("s8", "start", jobRow(), row("s8", "centre", spinner("s16"), text("Updating KubaOS · Resolving the flake", "small", "on_surface_variant"))) },
    ],
  },
  motion: {
    arrives: doesntApply("the spinner takes the place of what started the work at once, already turning; its turn is the Spinner page's."),
    leaves: doesntApply("the mark of how the work went takes its place at once, at its size, so nothing moves."),
    between: powerMoves(() => within(ROOM, beside("s20", "Poking at the hardware…", "body", "s12"))),
  },
  keyboard: doesntApply("a spinner takes no key; where it takes a button's place, the keyboard stays on the button's place and goes on to the next stop."),
  measures: {
    samples: nonEmpty([
      { name: "Beside small", for: "16, `space.s8` from its words.", sample: () => beside("s16", "Checking again…", "small", "s8") },
      { name: "Beside body", for: "20, `space.s12` from its words.", sample: () => beside("s20", "Poking at the hardware…", "body", "s12") },
      { name: "A page loading", for: "48 beside its heading, in its card.", sample: () => pageLoading() },
    ]),
    measures: [
      { what: "Beside `small` and `label`.", token: "icon-s16" },
      { what: "Beside `body`.", token: "icon-s20" },
      { what: "Beside a `subheading`.", token: "icon-s24" },
      { what: "A page's one.", token: "icon-s48" },
      { what: "From `small` words.", token: "space-s8" },
      { what: "From `body` words.", token: "space-s12" },
      { what: "A page's one from its heading.", token: "space-s20" },
    ],
    reflow: {
      described: "The spinner keeps its size at every window size; its words wrap beside it, the spinner centred down them.",
      sample: () => within("100%", stack("s8", "start", beside("s20", shown(DOING), "body", "s12"), text("A spinner and its words as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
