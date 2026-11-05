// The Stack, Row, Fill and Gap page: `docs/design/components/layout.md`, "Stack", "Row", "Fill",
// "Gap" and "Divider". The one-axis layouts: children one under another or side by side, a spacing
// step apart; room that pushes to the far end; one larger step; and the hairline between two parts,
// shown here in a layout (the Divider's own page is the atom's).

import { button } from "../../lib/button.ts";
import { chip } from "../../lib/chip.ts";
import type { Words } from "../../lib/content.ts";
import { icon } from "../../lib/icon.ts";
import { divider, fill, filling, gap, row, stack, within } from "../../lib/layout.ts";
import type { Align } from "../../lib/probe.ts";
import { switchControl } from "../../lib/switch.ts";
import { clamped, text, wrapped } from "../../lib/text.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
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
  summary: "Children one under another or side by side, a spacing step apart; room that pushes what follows to the far end; and one larger step in a stack.",
  live: () => stack("s32", "start", settingLine(), pushed(), divided()),
  description: [
    "Rust `layout::stack`, `layout::row`, `layout::fill` and `layout::gap`; QML `Stack`, `Row`, `Fill` and `Gap`. Every box in a surface is placed by one of these or by a grid: nothing is placed by a margin, a number or an absolute position. Each child is rounded up to whole device pixels, so every child starts on one.",
    "Stack puts its children one under another, one spacing step apart (`space.*`, or none). It lines them up at its start, or centres each across its whole width on a whole device pixel, an odd spare pixel after the child, never before it. A child that fills (`filling`) takes the whole width; a child that fills the height (`fillingHeight`) takes what the stack has left of it.",
    "Row puts its children side by side, one spacing step apart, along its top, its middle or its bottom. A child that fills takes the room the others leave, and the row is as tall as its tallest child.",
    "Fill is room that grows across a row and pushes what follows to its far end: a count at the start, its buttons at the end. It never grows down, so the row stays as tall as its content.",
    "Gap is one spacing step of empty room both ways, where one step of a stack wants to be larger than its gap: a title with `space.s16` under it in a stack of `space.s4`. Where every step is the same, the stack's gap is the answer; where a whole box wants room round it, that box's padding is (a band's `space.s16`, a card's), never a gap before and after it.",
    "Divider is the hairline across, in the soft outline, between two parts that would otherwise run together where space alone can't part them: a window's bar from its body. It takes no room but its pixel; the stack's gap keeps it clear either side. Where a gap is enough there is no divider (its own page is the atom's).",
    "When to use which: one line of things that never wraps is a Row; things that wrap onto new lines (chips, tags) are a Flow; columns whose edges line up from row to row are a Grid or a Split. A label beside its value where every label lines up down the page is a Split, never a Row with a fixed label width.",
  ],
  variants: [
    { name: "Stack, at its start", for: "The usual: each child at the stack's start.", sample: () => stacked() },
    { name: "Stack, centred", for: "Each child centred across: an empty state, a hero.", sample: () => stacked("centre") },
    { name: "Row, along its top", for: "Parts of different heights from the top: a symbol beside words that wrap.", sample: () => within(PANEL, aligned("start")) },
    { name: "Row, centred", for: "The usual for a line of controls: each on the row's middle.", sample: () => within(PANEL, aligned("centre")) },
    { name: "Row, along its bottom", for: "Parts sharing a baseline's foot: a figure beside its unit.", sample: () => within(PANEL, aligned("end")) },
    { name: "Fill", for: "Pushes what follows to the row's far end.", sample: pushed },
    { name: "Gap", for: "One larger step in a stack with none of its own.", sample: stepped },
    { name: "Divider, in a stack", for: "A window's bar parted from its body, the stack's `space.s12` either side.", sample: () => divided() },
  ],
  sizes: doesntApply("none has a size: a stack and a row are their children and their gaps, a fill is what the row leaves, a gap is the step it is given."),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "they take no pointer and no focus.",
      pressed: "they take no press.",
      disabled: "they hold controls, disabled each on their own.",
      selected: "they hold no choice.",
      loading: "what loads in them holds its own `Skeleton`, in its own box, so the layout doesn't move when it arrives.",
      error: "they have no value to be wrong.",
    },
    rows: [
      { name: "Row", cell: (state: "rest" | "empty") => (state === "empty" ? chips(0) : chips(3)) },
      { name: "Stack", cell: (state: "rest" | "empty") => (state === "empty" ? within(PANEL, stack("s8", "start")) : stacked()) },
    ],
    described: {
      rest: "They draw nothing: what shows is what they hold, where they put it.",
      empty: "With nothing in it a stack or a row takes no room and adds no gap; a surface with nothing to show holds an `EmptyState` instead.",
    },
  },
  focus: doesntApply("none is a control; they draw no line and no glow."),
  content: {
    text: [
      { name: "Words that fill a row", words: LINE, sample: (words: string) => settingLine(words) },
      { name: "Words that fill a row, along its top", words: LINE, sample: (words: string) => settingLine(words, "start") },
    ],
    missingPicture: doesntApply("they draw no picture; a missing symbol is its own `Icon`'s, and keeps its box, so the row doesn't move."),
    counts: [
      { name: "None", for: "No room and no gap.", sample: () => chips(0) },
      { name: "One", for: "One child, no gap.", sample: () => chips(1) },
      { name: "Many", for: "Each a step apart; past the room a row has, it is a Flow.", sample: () => chips(4) },
    ],
  },
  motion: {
    arrives: doesntApply("they draw nothing to arrive; what they hold arrives by its own moves, and a list's rows open their room by `LiveList`'s."),
    leaves: doesntApply("they draw nothing to leave."),
    between: doesntApply("layout never animates: a control's state takes no room, and a window's size never eases (`docs/DESIGN.md`, \"Motion\"). A divider's colours change with the theme on its own page."),
  },
  keyboard: {
    keys: [{ keys: ["Tab", "Shift+Tab"], does: "Through what they hold in reading order: a stack top to bottom, a row from its start to its end." }],
    order: "The order on screen is the order of the keys: a layout never puts a child where Tab wouldn't reach it next. A fill, a gap and a divider are no stop.",
    lands: "Nowhere of their own: the surface lands on its first thing to do.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A stack", for: "Its `space.s8` between each child.", sample: () => stacked() },
      { name: "A row with a fill", for: "Its `space.s8` between each child; the fill is what the others leave.", sample: pushed },
      { name: "A gap in a stack", for: "One `space.s16` in a stack of none.", sample: stepped },
      { name: "A divider in a stack", for: "The hairline, the stack's `space.s12` either side.", sample: () => divided() },
    ]),
    measures: [
      { what: "The gaps the samples' stacks and rows take, from the spacing steps.", token: "space-s8" },
      { what: "Between a setting's symbol and its name.", token: "space-s12" },
      { what: "The gap's one larger step.", token: "space-s16" },
      { what: "The divider: one device pixel at every scale.", token: "stroke-hairline_device_px" },
      { what: "The room the samples are given, a panel's width.", token: "size-panel" },
    ],
    reflow: {
      described:
        "Settings' page at the toolbar's window size is a stack of its header and sections, `space.s32` apart; each section a stack; the wallpaper's line a row whose fill pushes its button to the page's far edge at every width. A row never wraps: what doesn't fit is a Flow's or ends in an ellipsis.",
      sample: settingsWindow,
    },
  },
});
