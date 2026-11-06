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
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

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
  { size: "s16", style: "small", gap: "s8", words: "Checking again…", for: "`small` or `label` words, `space.s8` apart" },
  { size: "s20", style: "body", gap: "s12", words: "Poking at the hardware…", for: "`body` words, `space.s12` apart" },
  { size: "s24", style: "subheading", gap: "s12", words: "Looking for disks", for: "a `subheading`, `space.s12` apart" },
  { size: "s48", style: "heading", gap: "s20", words: "Setting up the reader", for: "a page's one, `space.s20` beside or `space.s16` above" },
];

export default page<"loading" | "rest" | "error">({
  level: "molecules",
  slug: "spinner-in-context",
  title: "Spinner in context",
  lede: "The spinner where work can't tell how far it is: in a button's place, leading a row, under a field, for a page.",
  code: { rust: "progress::spinner", qml: "Spinner", unbuilt: "steady arc: not built yet" },
  stage: { live: () => stack("s32", "start", inPlace(true), installSteps(), pageLoading()), hint: "nothing to press: it turns while work runs", daylight: true },
  variants: [
    { name: "In a button's place", for: "a pressed button's place, with its words", sample: () => stack("s16", "start", inPlace(false), inPlace(true)) },
    { name: "In the actions' place", for: "all of a dialog's actions wait: the power menu", sample: () => dialogActions(false) },
    { name: "In the actions' place, danger", for: "work that loses something, in `error`", sample: () => dialogActions(true) },
    { name: "Leading a row", for: "a step among done and waiting ones", sample: () => installSteps() },
    { name: "Leading a job's row", for: "the Command menu's job, 16 beside `small`", sample: jobRow },
    { name: "Under a field", for: "the connect form's status line", sample: () => underField("Connecting…") },
    { name: "A page loading, beside its heading", for: "first boot's step: 48, in its card", sample: () => pageLoading() },
    { name: "A page loading, above its words", for: "a progress panel's job that can't tell", sample: aboveWords },
  ],
  sizes: nonEmpty(PAIRS.map((pair) => ({ name: pair.size.slice(1), for: pair.for, sample: () => within(ROOM, beside(pair.size, pair.words, pair.style, pair.gap)) }))),
  states: {
    shown: ["loading", "rest", "error"],
    absent: {
      lit: "it takes no pointer or focus",
      pressed: "the button gave its place to it",
      disabled: "work that can't run shows no spinner",
      selected: "it holds no choice",
      empty: "it always turns while it shows",
    },
    rows: [
      { name: "A row", cell: (state) => installSteps(state === "loading" ? "running" : state === "rest" ? "done" : "failed") },
      { name: "A job", cell: (state) => jobRow(state === "loading" ? "running" : state === "rest" ? "done" : "failed") },
      { name: "A page", cell: (state) => pageLoading(state === "loading" ? "running" : state === "rest" ? "done" : "failed") },
    ],
    looks: {
      loading: "the spinner at its text's size, words beside it",
      rest: "`check_circle` in `primary` in its place",
      error: "`error` in `error` in its place; a row's title too",
    },
  },
  focus: doesntApply("a spinner takes no focus; the parts round it keep their own."),
  content: {
    text: [{ name: "What is going on", words: DOING, sample: (words: string) => within(ROOM, beside("s20", words, "body", "s12")) }],
    missingPicture: doesntApply("a spinner is drawn."),
    counts: [
      { name: "One", for: "a page's one, or one line of work", sample: () => inPlace(true) },
      { name: "Many", for: "each line spins on one clock: running jobs", sample: () => stack("s8", "start", jobRow(), row("s8", "centre", spinner("s16"), text("Updating KubaOS · Resolving the flake", "small", "on_surface_variant"))) },
    ],
  },
  motion: {
    arrives: doesntApply("it takes its place at once, already turning."),
    leaves: doesntApply("the mark of how it went takes its place at once."),
    between: powerMoves(() => within(ROOM, beside("s20", "Poking at the hardware…", "body", "s12"))),
  },
  keyboard: doesntApply("a spinner takes no key; the keyboard goes on to the next stop."),
  measures: {
    samples: nonEmpty([
      { name: "Beside small", for: "16, `space.s8` from its words", sample: () => beside("s16", "Checking again…", "small", "s8") },
      { name: "Beside body", for: "20, `space.s12` from its words", sample: () => beside("s20", "Poking at the hardware…", "body", "s12") },
      { name: "A page loading", for: "48 beside its heading, in its card", sample: () => pageLoading() },
    ]),
    measures: [
      { part: "Beside `small`, `label`", token: "icon-s16" },
      { part: "Beside `body`", token: "icon-s20" },
      { part: "Beside a `subheading`", token: "icon-s24" },
      { part: "A page's one", token: "icon-s48" },
      { part: "From `small` words", token: "space-s8" },
      { part: "From `body` words", token: "space-s12" },
      { part: "A page's one from its heading", token: "space-s20" },
    ],
    reflow: {
      note: "It keeps its size; its words wrap beside it, the spinner centred down them.",
      sample: () => within("100%", beside("s20", shown(DOING), "body", "s12")),
    },
  },
});
