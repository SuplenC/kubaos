// The Card page: `docs/design/components/card.md`, the plain card. A surface inside the page,
// raised one step on the ladder, holding a header, a body and a button bar; plain, it takes no
// input, and pressable, the whole card is one press target in the state model.

import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import { buttonBar } from "../../lib/button-bar.ts";
import { type CardOptions, card } from "../../lib/card.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { media } from "../../lib/media.ts";
import { role } from "../../lib/radius.ts";
import { text, wrapped } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

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
function log(state: Shown = "rest", onPress: () => void = () => undefined): HTMLElement {
  return card({ header: { glyph: "terminal", title: "Install log", subtitle: "312 lines · 2 warnings" }, onPress, ...stateOptions(state) });
}

/** A card holding a picture, `space.s16` in, at the row radius. */
function pictured(): HTMLElement {
  return card({
    header: { title: "Wallpaper", subtitle: "Shinjuku, night · 3840 × 2400" },
    body: [media({ source: photo, corner: role("row") })],
  });
}

/** The live sample: a card with actions, and a pressable one that says it was pressed. */
function live(): HTMLElement {
  const said = text("Press the log's card, or Tab to it and press Enter.", "small", "on_surface_variant");
  const pressed = log("rest", () => {
    said.textContent = "The log opens in its own view.";
  });
  return within(COLUMN, stack("s16", "start", card(withActions()), pressed, said));
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

const DESCRIBED =
  "A surface raised one step on the ladder (`[elevation]`, `docs/DESIGN.md`, \"Elevation\"): `surface_container_high` by day, `surface_container_highest` at night, step 1's contact and ambient shadow, and the rim inside its edge (`elevation.rim`, `light_rim` by day). Its corners are the content radius, `radius.content` (16): a card is a box of content inside a window or a panel, as a list well and a dialog are, a step under the window's or the panel's own 24. `space.s16` inside, as wide as its room; as tall as what it holds, or, in a grid, as its row, so cards side by side end together.";

export default page({
  level: "molecules",
  slug: "card",
  title: "Card",
  summary: "A surface inside the page that holds a group of things belonging together: a header, a body and its buttons.",
  live,
  description: [
    "Rust `card::card`, QML `Card`. A group of controls or facts that belong together but aren't a list (a disk and what installing on it does, a proxy's details) sits in a card; a list of like items sits in a well (`List`), and the panel's own controls sit on the panel with nothing round them.",
    DESCRIBED,
    "What it holds runs top to bottom, `space.s16` apart: a header, then the body, then a button bar at its foot (`ButtonBar`), each as wide as the card's padding leaves. The header is its title in `subheading` (`on_surface`), breaking between words, after a symbol at the title's icon size (`subheading`'s 24) `space.s8` before it; and a line under it in `small`, `on_surface_variant`, `space.s4` below, from the card's padding. The button bar is the surface's: the card's action Tonal, since a page's one Filled action is the page's, not a card's.",
    "A plain card takes no input and has no states: its controls are each a stop of their own. A pressable card is one press target as a whole (a log, a device to open): it lights as any control does, its tube (`primary` at `focus.tube`) inside its edge at its own corner with the glow outside at night; pressed, its tube runs white-hot and the glow brightens, and by day the tone tints it (`opacity.press_tint`). It holds no controls of its own, since a press anywhere on it is its press. Disabled, it keeps its resting surface and what it holds dims to `opacity.disabled_content`, with no line, no glow and no stop for Tab.",
    "Nesting: a card or a well is never inside another; the container's padding is the only padding round it (`docs/DESIGN.md`, \"The container\"). A control in a card keeps its own role's corner (a button's 12). A picture filling the card less its `space.s16` takes the row radius, `radius.row` (8): the card's 16 less its 16 would leave no corner.",
  ],
  variants: [
    { name: "A header and a body", for: "Facts that belong together: the disk, what installing on it does.", sample: () => within(COLUMN, card(DISK)) },
    { name: "With a button bar", for: "Something to do about what it holds, at its foot.", sample: () => within(COLUMN, card(withActions())) },
    { name: "Holding a picture", for: "A picture `space.s16` in, at the row radius.", sample: () => within(HALF, pictured()) },
    { name: "Pressable", for: "One press target as a whole: a log, a device to open.", sample: () => within(COLUMN, log()) },
  ],
  sizes: doesntApply("a card has one padding and one corner, and is as wide as the room it is given: a column, a cell of a grid, a panel's body."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "a card picked from a set is a `ChoiceCard`, on its own page.",
      loading: "what a card holds loads in its own parts, a `Skeleton` in their place; the card is there from the start.",
      empty: "a card always holds something; an empty group is no card, and an empty list is the well's `EmptyList`.",
      error: "an error is said by what it holds (an `ErrorBlock`, a field's Invalid state), never by the card.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    described: {
      rest: "Raised one step: `surface_container_highest` at night, `surface_container_high` by day, the rim inside its edge and step 1's shadow. A plain card is always this.",
      lit: "Under the pointer or with the keyboard on it, here with the signs on: the tube, `primary` at `focus.tube`, inside its edge at its own corner, the glow outside. The fill and what it holds don't change. By day the tube with no glow.",
      pressed: "Lit and pressed: the tube runs white-hot, to the core (`primary_core`), and the glow is `focus.pressed_glow` times the lit one; by day the tone tints it at `opacity.press_tint`. Letting go does what it does.",
      disabled: "Its resting surface, the rim and shadow kept; what it holds at `opacity.disabled_content`. No line, no glow, no press, no stop for Tab.",
    },
  },
  focus: {
    tubes: [{ name: "Pressable card", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest the sign burns: the tube inside the card's edge at `radius.content`, the glow outside it, every ring concentric with the corner. What it holds doesn't light.",
      signsOff: "Still lit with the neon unpowered, by day always and at night while a power change has the lights out: the tube is `primary_glass`, with no glow.",
    },
    signsOff: (index: number) => cell(ROWS[index]!.options, "signs-off"),
  },
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => within(HALF, card({ header: { glyph: "hard_drive", title: words, subtitle: "2 TB · NVMe" } })) },
      { name: "The body", words: BODY, sample: (words: string) => within(HALF, card({ header: { title: "Disk" }, body: [wrapped(words, "body", "on_surface")] })) },
    ],
    missingPicture: doesntApply("a card draws no picture of its own: a picture in it is a `Media`, whose page shows one that is missing; its symbol is one of KubaOS Symbols, always there."),
    counts: [
      { name: "A title alone", for: "The least a card holds: its header's one line.", sample: () => within(HALF, card({ header: { title: "Install log" } })) },
      { name: "A header and a body", for: "The usual card.", sample: () => within(HALF, card(DISK)) },
      { name: "Two side by side", for: "Cards in a grid share their row's height, so they end together.", sample: pair },
    ],
  },
  motion: {
    arrives: doesntApply("a card comes with the page or the surface it is on."),
    leaves: doesntApply("it goes with the page or the surface it is on."),
    between: [
      { name: "Focus arrives", what: "A pressable card under the pointer or with the keyboard on it lights up over `short` on `standard`: its tube and glow fade in. Never the flicker.", tracks: lights(".k-card", "a pressable card", 0, 1), reduced: "same", back: "cuts", sample: () => cell(ROWS[0].options, "rest") },
      { name: "Focus leaves", what: "It goes back to rest over `short` on `standard`.", tracks: lights(".k-card", "a pressable card", 1, 0), reduced: "same", back: "cuts", sample: () => cell(ROWS[0].options, "lit") },
      { name: "Pressed", what: "A lit card pressed: over `micro` its tube runs white-hot and the glow brightens, and by day the tone tints it.", tracks: press(".k-card", "a pressable card"), reduced: "same", back: "plays", sample: () => cell(ROWS[0].options, "lit") },
      ...powerMoves(() => row("s16", "start", cell(ROWS[0].options, "lit"), within("var(--size-thumbnail_width)", card(DISK)))),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "A pressable card is one stop in the surface's reading order. A plain card is none: its controls are each a stop, in reading order." },
      { keys: ["Enter", "Space"], does: "Presses the pressable card the keyboard is on, once per press, never on key repeat." },
      { keys: ["Esc"], does: "Nothing of the card's: it goes back one step, as the surface says." },
    ],
    order: "A disabled pressable card is skipped. Focus moving to it or away never moves anything: the line and the glow take no room.",
    lands: "A card takes the keyboard only when the surface gives it: a surface starts on its first thing to do.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a button bar", for: "Its padding, the header's symbol, title and line, the gaps between its parts, and the bar.", sample: () => within(COLUMN, card(withActions())) },
      { name: "Holding a picture", for: "The picture's corner, the row radius, `space.s16` in.", sample: () => within(HALF, pictured()) },
    ]),
    measures: [
      { what: "Inside its edge, and between its header, body and button bar.", token: "space-s16" },
      { what: "Its corners.", token: "radius-content" },
      { what: "The header's symbol, `subheading`'s.", token: "type-subheading-icon" },
      { what: "The title's line.", token: "type-subheading-line" },
      { what: "Between the symbol and the title.", token: "space-s8" },
      { what: "Between the title and the line under it.", token: "space-s4" },
      { what: "The line under the title.", token: "type-small-line" },
      { what: "The button bar's buttons, regular.", token: "size-control_md" },
      { what: "Between two cards side by side.", token: "space-s16" },
    ],
    reflow: {
      described: "A card is as wide as its room, its words breaking between them and its button bar stacking where its buttons don't fit (`ButtonBar`). In a grid it takes its cell and its row's height. Nothing in it is a fixed width.",
      sample: () => within("100%", card(withActions())),
    },
  },
});
