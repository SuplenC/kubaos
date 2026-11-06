// The Card page: `docs/design/components/card.md`, the plain card. A region of the page, flat on
// it, holding a header, a body and a button bar; plain, it takes no
// input, and pressable, the whole card is one press target in the state model.

import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import { buttonBar } from "../../lib/button-bar.ts";
import { type CardOptions, card } from "../../lib/card.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { filling, flow, row, stack, within } from "../../lib/layout.ts";
import { media } from "../../lib/media.ts";
import { role } from "../../lib/radius.ts";
import { wrapped } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The stage's pane, less the room a lit control's glow throws past its edge, which it would otherwise scale down for. */
const ACROSS = "calc(100% - var(--space-s24))";

const TITLE: Words = { short: "Disk", long: "Samsung 990 Pro, the disk KubaOS installs on" };
const BODY: Words = {
  short: "Everything on it goes.",
  long: "Everything on it goes: three partitions, Windows and its recovery partition included, and the files in them.",
};

/** A card's room on a page: a column of text. */
const COLUMN = "var(--size-column)";
/** A card's room in a grid of two on a page: half the column, less half the gap between two. */
const HALF = "calc((var(--size-column) - var(--space-s16)) / 2)";

const DISK: CardOptions = {
  header: { glyph: "hard_drive", title: "Samsung 990 Pro", subtitle: "2 TB · NVMe · 3 partitions" },
  body: [wrapped("Everything on it goes: three partitions, Windows included.", "body", "on_surface")],
};

/** The disk's card with its button bar, its words at the toolbar's length. */
function withActions(): CardOptions {
  return {
    header: { glyph: "hard_drive", title: shown(TITLE), subtitle: "2 TB · NVMe · 3 partitions" },
    body: [wrapped(shown(BODY), "body", "on_surface")],
    actions: buttonBar({ actions: [{ label: "Pick another disk", kind: "Tonal" }] }).element,
  };
}

/** A pressable card: the install's log, opened by a press. */
function log(state: Shown = "rest"): HTMLElement {
  return card({ header: { glyph: "terminal", title: "Install log", subtitle: "312 lines · 2 warnings" }, onPress: () => undefined, ...stateOptions(state) });
}

/** A card holding a picture, `space.s16` in, at the row radius. */
function pictured(): HTMLElement {
  return card({
    header: { title: "Wallpaper", subtitle: "Shinjuku, night · 3840 × 2400" },
    body: [media({ source: photo, corner: role("row") })],
  });
}

/** The live sample: a card with actions and a pressable one, beside a card holding a picture. */
function live(): HTMLElement {
  return within(ACROSS, filling(flow("s16", within(COLUMN, stack("s16", "start", card(withActions()), log())), within(HALF, pictured()))));
}

/** Two cards side by side in a grid, ending together. */
function pair(): HTMLElement {
  const grid = row("s16", "start", within(HALF, card(DISK)), within(HALF, log()));
  grid.style.alignItems = "stretch";
  return within(COLUMN, grid);
}

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

const ROWS = [
  { name: "Pressable", options: { header: { glyph: "terminal", title: "Install log", subtitle: "312 lines · 2 warnings" } } },
  { name: "Pressable, a title alone", options: { header: { title: "Install log" } } },
] as const;

function cell(options: CardOptions, state: Shown): HTMLElement {
  return within("var(--size-thumbnail_width)", card({ ...options, onPress: () => undefined, ...stateOptions(state) }));
}

export default page({
  level: "molecules",
  slug: "card",
  title: "Card",
  lede: "A surface inside the page holding things that belong together: a header, a body and its buttons.",
  code: { rust: "card::card", qml: "Card" },
  stage: {
    live,
    hint: "press the log's card, or Tab to it and Enter",
    daylight: true,
  },
  variants: [
    { name: "A header and a body", for: "facts that belong together", sample: () => within(COLUMN, card(DISK)) },
    { name: "With a button bar", for: "something to do, at its foot, Tonal", sample: () => within(COLUMN, card(withActions())) },
    { name: "Holding a picture", for: "`space.s16` in, at `radius.row`", sample: () => within(HALF, pictured()) },
    { name: "Pressable", for: "one press target as a whole", sample: () => within(COLUMN, log()) },
  ],
  sizes: doesntApply("one padding and one corner; as wide as its room."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "a card picked from a set is a `ChoiceCard`",
      loading: "its parts load, a `Skeleton` in their place",
      empty: "an empty list is the well's `EmptyList`",
      error: "said by what it holds: an `ErrorBlock`",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    looks: {
      rest: "flat, `surface_container`, soft hairline, no shadow",
      lit: "the `primary` tube inside its edge, glow at night",
      pressed: "white-hot, glow × `focus.pressed_glow`; by day a tint",
      disabled: "its fill kept, content at `opacity.disabled_content`, no Tab stop",
    },
  },
  focus: {
    tubes: [{ name: "Pressable card", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the tube inside its edge at `radius.content`, the glow outside",
      signsOff: "the tube as `primary_glass`, no glow",
    },
    signsOff: (index: number) => cell(ROWS[index]!.options, "signs-off"),
  },
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => within(HALF, card({ header: { glyph: "hard_drive", title: words, subtitle: "2 TB · NVMe" } })) },
      { name: "The body", words: BODY, sample: (words: string) => within(HALF, card({ header: { title: "Disk" }, body: [wrapped(words, "body", "on_surface")] })) },
    ],
    missingPicture: doesntApply("a picture in it is a `Media`; its symbol is always there."),
    counts: [
      { name: "A title alone", for: "the least it holds", sample: () => within(HALF, card({ header: { title: "Install log" } })) },
      { name: "A header and a body", for: "the usual card", sample: () => within(HALF, card(DISK)) },
      { name: "Two side by side", for: "one row's height, ending together", sample: pair },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its page or surface."),
    leaves: doesntApply("it goes with its page or surface."),
    between: [
      { name: "Focus arrives", what: "The tube and glow fade in over `short` on `standard`.", tracks: lights(".k-card", "a pressable card", 0, 1), reduced: "same", back: "cuts", sample: () => cell(ROWS[0].options, "rest") },
      { name: "Focus leaves", what: "Back to rest over `short` on `standard`.", tracks: lights(".k-card", "a pressable card", 1, 0), reduced: "same", back: "cuts", sample: () => cell(ROWS[0].options, "lit") },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot and the glow brightens; by day the tone tints it.", tracks: press(".k-card", "a pressable card"), reduced: "same", back: "plays", sample: () => cell(ROWS[0].options, "lit") },
      ...powerMoves(() => row("s16", "start", cell(ROWS[0].options, "lit"), within("var(--size-thumbnail_width)", card(DISK)))),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "pressable: one stop; plain: none, its controls are" },
      { keys: ["Enter", "Space"], does: "presses a pressable card, once per press" },
      { keys: ["Esc"], does: "nothing of its own: the surface's" },
    ],
    order: "In the surface's reading order; a disabled pressable card is skipped.",
    lands: "Only where the surface gives it the keyboard.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a button bar", for: "padding, header, gaps, the bar", sample: () => within(COLUMN, card(withActions())) },
      { name: "Holding a picture", for: "the picture's corner and inset", sample: () => within(HALF, pictured()) },
    ]),
    measures: [
      { part: "Inset, gaps between parts", token: "space-s16" },
      { part: "Corner", token: "radius-content" },
      { part: "Edge", token: "stroke-hairline_device_px" },
      { part: "Header symbol", token: "type-subheading-icon" },
      { part: "Title line", token: "type-subheading-line" },
      { part: "Symbol → title", token: "space-s8" },
      { part: "Title → line", token: "space-s4" },
      { part: "Line under the title", token: "type-small-line" },
      { part: "Button bar's buttons", token: "size-control_md" },
      { part: "Between two cards", token: "space-s16" },
    ],
    reflow: {
      note: "As wide as its room: its words break and its button bar stacks; in a grid it takes its row's height.",
      sample: () => within("100%", card(withActions())),
    },
  },
});
