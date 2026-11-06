// The Stack, Row, Fill and Gap page: `docs/design/components/layout.md`, "Stack", "Row", "Fill",
// "Gap" and "Divider". The one-axis layouts: children one under another or side by side, a spacing
// step apart; room that pushes to the far end; one larger step; and the hairline between two parts,
// shown here in a layout (the Divider's own page is the atom's).

import { button } from "../../lib/button.ts";
import { chip } from "../../lib/chip.ts";
import { shown, type Words } from "../../lib/content.ts";
import { icon } from "../../lib/icon.ts";
import { divider, fill, filling, gap, row, stack, within } from "../../lib/layout.ts";
import type { Align } from "../../lib/probe.ts";
import { switchControl } from "../../lib/switch.ts";
import { clamped, text, wrapped } from "../../lib/text.ts";
import { el } from "../../site/dom.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { settingsWindow } from "./example.ts";

const PANEL = "var(--size-panel)";

const LINE: Words = { short: "Wi-Fi", long: "Wi-Fi, and the networks it has joined before" };

/** A setting's line: its symbol, its name, then a switch pushed to the far end. */
function settingLine(words = "Wi-Fi", align: Align = "centre"): HTMLElement {
  return within(PANEL, row("s12", align, icon("wifi", "var(--type-body_strong-icon)", "var(--color-on_surface)"), filling(wrapped(words, "body_strong")), switchControl({ on: true })));
}

/** A row of `align`ed parts of different heights: a chip, a small and a regular button. */
function aligned(align: Align): HTMLElement {
  return row("s8", align, chip({ label: "Beta" }), button({ label: "Later", kind: "Text", size: "Sm" }), button({ label: "Update", kind: "Filled" }));
}

/** A stack of a title, its line and a button, `align`ed across. */
function stacked(align: Align = "start", width = PANEL): HTMLElement {
  return within(width, stack("s8", align, text("Updates", "heading"), text("KubaOS 0.4 is ready.", "body", "on_surface_variant"), button({ label: "Install", kind: "Filled", size: "Sm" })));
}

/** A row whose end is pushed to the far side by `Fill`. */
function pushed(): HTMLElement {
  return within(PANEL, row("s8", "centre", text("3 updates", "body"), fill(), button({ label: "Later", kind: "Text", size: "Sm" }), button({ label: "Update all", kind: "Tonal", size: "Sm" })));
}

/** A stack with one larger step: a title, `space.s16`, then its words, with no gap of the stack's own. */
function stepped(): HTMLElement {
  return within(PANEL, stack(0, "start", text("Battery", "heading"), gap("s16"), wrapped("82%, about 4 hours left. It charges to 80% to last longer.", "body", "on_surface_variant")));
}

/** A window's bar over its body: a row band, the hairline across, then the body. */
function divided(width = PANEL): HTMLElement {
  const bar = row("s8", "centre", text("Downloads", "body_strong"), fill(), button({ label: "Clear", kind: "Text", size: "Sm" }));
  const body = stack("s4", "start", clamped("kubaos-0.4.iso", "body"), text("2.1 GB · done", "small", "on_surface_variant"));
  return within(width, stack("s12", "start", filling(bar), divider(), body));
}

/** A row of `count` chips, as many as a row holds. */
function chips(count: number): HTMLElement {
  return within(PANEL, row("s8", "centre", ...["English", "Polish", "Japanese", "German"].slice(0, count).map((label) => chip({ label }))));
}

export default page({
  level: "layout",
  slug: "stack",
  title: "Stack, Row, Fill and Gap",
  lede: "Children one under another or side by side, a spacing step apart; room that pushes to the far end; one larger step.",
  code: { rust: "layout::stack, layout::row, layout::fill, layout::gap", qml: "Stack, Row, Fill, Gap" },
  stage: {
    live: () => el("div", "f-row", stack("s32", "start", pushed(), stepped()), stack("s32", "start", divided(), settingLine(shown(LINE)))),
    hint: "pick Text long: the words wrap, the switch stays",
    daylight: true,
  },
  variants: [
    { name: "Stack, at its start", for: "the usual: each child at its start", sample: () => stacked() },
    { name: "Stack, centred", for: "an empty state, a hero", sample: () => stacked("centre") },
    { name: "Row, along its top", for: "a symbol beside words that wrap", sample: () => within(PANEL, aligned("start")) },
    { name: "Row, centred", for: "the usual for a line of controls", sample: () => within(PANEL, aligned("centre")) },
    { name: "Row, along its bottom", for: "a figure beside its unit", sample: () => within(PANEL, aligned("end")) },
    { name: "Fill", for: "pushes what follows to the far end", sample: pushed },
    { name: "Gap", for: "one step larger than the stack's own", sample: stepped },
    { name: "Divider, in a stack", for: "a window's bar from its body, `space.s12` either side", sample: () => divided() },
  ],
  sizes: doesntApply("none has a size: its children and gaps, what is left, or the step given."),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "they take no pointer and no focus",
      pressed: "they take no press",
      disabled: "what they hold is disabled on its own",
      selected: "they hold no choice",
      loading: "a `Skeleton` in its own box, so nothing moves",
      error: "they have no value to be wrong",
    },
    rows: [
      { name: "Row", cell: (state: "rest" | "empty") => (state === "empty" ? chips(0) : chips(3)) },
      { name: "Stack", cell: (state: "rest" | "empty") => (state === "empty" ? within(PANEL, stack("s8", "start")) : stacked()) },
    ],
    looks: {
      rest: "they draw nothing but what they hold",
      empty: "no room and no gap",
    },
  },
  focus: doesntApply("none is a control: no line and no glow."),
  content: {
    text: [
      { name: "Words that fill a row", words: LINE, sample: (words: string) => settingLine(words) },
      { name: "Filling words, along its top", words: LINE, sample: (words: string) => settingLine(words, "start") },
    ],
    missingPicture: doesntApply("they draw no picture; a missing symbol keeps its box."),
    counts: [
      { name: "None", for: "no room and no gap", sample: () => chips(0) },
      { name: "One", for: "no gap", sample: () => chips(1) },
      { name: "Many", for: "past the row's room, it is a Flow", sample: () => chips(4) },
    ],
  },
  motion: {
    arrives: doesntApply("they draw nothing to arrive; what they hold has its own moves."),
    leaves: doesntApply("they draw nothing to leave."),
    between: doesntApply("layout never animates: a control's state takes no room."),
  },
  keyboard: {
    keys: [{ keys: ["Tab", "Shift+Tab"], does: "what they hold, in reading order" }],
    order: "The order on screen is the order of the keys; a fill, a gap and a divider are no stop.",
    lands: "Nowhere of their own: the surface lands on its first thing to do.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A stack", for: "`space.s8` between each child", sample: () => stacked() },
      { name: "A row with a fill", for: "`space.s8` between; the fill is what is left", sample: pushed },
      { name: "A gap in a stack", for: "one `space.s16` in a stack of none", sample: stepped },
      { name: "A divider in a stack", for: "the hairline, `space.s12` either side", sample: () => divided() },
    ]),
    measures: [
      { part: "The samples' gaps", token: "space-s8" },
      { part: "A setting's symbol to its name", token: "space-s12" },
      { part: "The gap's larger step", token: "space-s16" },
      { part: "The divider, one device pixel", token: "stroke-hairline_device_px" },
      { part: "The samples' room", token: "size-panel" },
    ],
    reflow: {
      note: "Settings' page is a stack of sections; a row's fill pushes its button to the far edge; a row never wraps.",
      sample: settingsWindow,
    },
  },
});
