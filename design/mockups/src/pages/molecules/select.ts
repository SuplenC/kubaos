// The Select page: `docs/design/components/select.md`. One of a list too long for radio rows or
// tabs: a field's look that opens a menu of its options on a popover under it, the picked one
// checked. Lit as a field is; open, its hairline takes the accent and its chevron turns up.

import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack, within } from "../../lib/layout.ts";
import { type SelectOptions, select } from "../../lib/select.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const COUNTRIES = ["None", "Germany", "Italy", "Poland"];
const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const OPTION: Words = { short: "Daily", long: "Every 5 minutes while on AC power, hourly on battery" };

/** The room every sample's select fills here; a select fills whatever room it is given. */
const ROOM = "var(--size-menu)";

function boxed(options: SelectOptions): HTMLElement {
  return within(ROOM, select(options));
}

/** The clock's holidays: a flag, Poland picked. */
function holidays(extra: Partial<SelectOptions> = {}): SelectOptions {
  return { options: COUNTRIES, picked: 3, icon: "flag", placeholder: "Pick a country", ...extra };
}

/** A select with no symbol: how often something runs. */
function often(extra: Partial<SelectOptions> = {}): SelectOptions {
  return { options: ["Every 5 minutes", "Hourly", "Daily"], picked: 0, placeholder: "How often", ...extra };
}

const ROWS = [
  { name: "With a symbol", options: holidays },
  { name: "Without", options: often },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "disabled" | "empty";

function cell(options: (extra?: Partial<SelectOptions>) => SelectOptions, state: MatrixState): HTMLElement {
  if (state === "empty") return boxed(options({ picked: -1 }));
  return boxed(options(stateOptions(state as Shown)));
}

/** A select shut and lit, and the same open: what the moves play on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => stack("s16", "start", boxed(holidays(stateOptions(state))), boxed(often(stateOptions(state))));
}

const ON = "two selects";

const DESCRIBED =
  "A field's look: `size.control_md` (40) tall at the control radius, filling its room, `surface_container_high` with a hairline of `outline_variant` inside its edge, `space.s12` inside. The picked option is `body_large` in `on_surface` on one line, ending in an ellipsis where it doesn't fit; an optional `body_large` symbol (20) leads it and a chevron (`expand_more`) ends it, both `on_surface_variant`, `space.s8` from the words. With nothing picked it shows its placeholder in `on_surface_variant`.";

export default page({
  level: "molecules",
  slug: "select",
  title: "Select",
  summary: "One of a list too long for radio rows or tabs: a field's look that opens a menu of its options under it.",
  live: () => within("var(--size-column)", stack("s16", "start", select(holidays()), select({ options: MONTHS, picked: 9, icon: "event" }))),
  description: [
    "Rust `select::select`, QML `Select`. Press one, or Tab to it and press Enter or Space: ↑ ↓ and Enter pick, Esc or a press anywhere else closes it. The months scroll past eight.",
    DESCRIBED,
    "Enter, Space or a press opens its menu (the Menu page) on a popover (the Popover page) `space.s4` under it, as wide as it, the picked option checked and the keyboard on it, raised to step 2. Open, its fill is `surface_container_highest`, its hairline `primary` and its chevron turns up (`expand_less`); it shows no tube, since the keyboard is in its menu. A menu of more than eight options scrolls, the keyboard's option in view. Picking one, Esc or a press anywhere else closes it and gives the keyboard back to the select; whatever closes or hides what holds the select (its panel closed by any route, its page swapped out) closes its menu too.",
    "Its states follow the neon, as a field's do: lit under the pointer or with the keyboard on it, its fill rises to `surface_container_highest` and the tube burns over its hairline, `primary` at `focus.tube`, with the glow outside at night; pressed, the glow brightens to `focus.pressed_glow` times the lit one. Disabled, it is grey as a disabled field is.",
  ],
  variants: [
    { name: "With a symbol", for: "Options a symbol helps place: the clock's holidays, by country.", sample: () => boxed(holidays()) },
    { name: "Without", for: "Plain options: how often something runs.", sample: () => boxed(often()) },
    { name: "Open", for: "Its menu under it, the picked option checked and the keyboard on it.", sample: () => within(ROOM, select(holidays({ open: true }))) },
  ],
  sizes: doesntApply("a select has one height, a field's (`size.control_md`), and fills the room it is given."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled", "empty"],
    absent: {
      selected: "the picked option is what it shows; open is a variant above, its menu's checked option the pick.",
      loading: "its options are known when it shows; a list still on its way leaves it disabled.",
      error: "a pick can't be wrong; a select that needs one shows the placeholder until it has it.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    described: {
      rest: "`surface_container_high`, the hairline `outline_variant`, the pick `on_surface`, the symbol and the chevron `on_surface_variant`. By day the same roles in daylight colours.",
      lit: "Under the pointer or with the keyboard on it, here with the signs on: the fill rises to `surface_container_highest`, the tube burns over the hairline, `primary` at `focus.tube`, with the glow outside. By day the tube with no glow.",
      pressed: "Lit and pressed: the glow `focus.pressed_glow` times the lit one; letting go opens the menu.",
      disabled: "Grey, as a disabled field: the fill `on_surface` at `opacity.disabled_container` with no hairline, the pick, the symbol and the chevron at `opacity.disabled_content`. No press, no stop for Tab.",
      empty: "Nothing picked: the placeholder in `on_surface_variant`, never a pick that looks made.",
    },
  },
  focus: {
    tubes: [{ name: "Every select", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the tube over the hairline inside its edge, the glow outside, all that reaches past it.",
      signsOff: "Still lit with the neon unpowered: by day always, and at night while a power change has the lights out. The tube is solid glass, `primary_glass`, with no glow; the fill stays risen.",
    },
    signsOff: (index: number) => cell((extra) => ROWS[index]!.options({ ...extra, pinned: "signs-off" }), "rest"),
  },
  content: {
    text: [{ name: "The picked option", words: OPTION, sample: (words: string) => boxed({ options: [words], picked: 0, icon: "schedule" }) }],
    missingPicture: [{ name: "No symbol", for: "A select may have none: its words start `space.s12` in.", sample: () => boxed(often()) }],
    counts: [
      { name: "Nothing picked", for: "The placeholder.", sample: () => boxed(holidays({ picked: -1 })) },
      { name: "A few options", for: "Four: the menu as tall as they are.", sample: () => within(ROOM, select(holidays({ open: true }))) },
      { name: "Many", for: "Twelve: the menu shows eight and scrolls.", sample: () => within(ROOM, select({ options: MONTHS, picked: 9, icon: "event", open: true })) },
    ],
  },
  motion: {
    arrives: doesntApply("a select comes with the surface it is on; its menu's popover arrives as the Popover page shows."),
    leaves: doesntApply("a select goes with the surface it is on, and takes its menu with it."),
    between: [
      { name: "Focus arrives", what: "Under the pointer or with the keyboard on it, it lights up over `short` on `standard`: the fill rises, the tube and the glow fade in.", tracks: lights(".k-select", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "It goes back to rest over `short` on `standard`.", tracks: lights(".k-select", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "A lit select pressed: the glow brightens over `micro`, and back as it comes up; letting go opens its menu.", tracks: press(".k-select", ON), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Enter", "Space"], does: "Opens its menu, the keyboard on the picked option." },
      { keys: ["↑", "↓"], does: "In the open menu, the option before or after." },
      { keys: ["Enter"], does: "In the open menu, picks the option and closes it; the keyboard comes back to the select." },
      { keys: ["Esc"], does: "Closes the menu without picking; the keyboard comes back to the select." },
      { keys: ["Tab", "Shift+Tab"], does: "To the select, or on to the next or previous control. Open, the keyboard is in the menu, a stop of its own." },
    ],
    order: "One stop in the surface's reading order; a disabled one is skipped.",
    lands: "Opening, on the picked option in its menu; closing, back on the select.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a symbol", for: "Its height, padding, symbol, chevron and the gaps between them.", sample: () => boxed(holidays()) },
      { name: "Open", for: "The menu's gap under it and its width.", sample: () => within(ROOM, select(holidays({ open: true }))) },
    ]),
    measures: [
      { what: "Its height, a field's.", token: "size-control_md" },
      { what: "Inside its sides.", token: "space-s12" },
      { what: "Between the symbol, the words and the chevron.", token: "space-s8" },
      { what: "The symbol and the chevron, `body_large`'s.", token: "type-body_large-icon" },
      { what: "The pick's line.", token: "type-body_large-line" },
      { what: "Its corners.", token: "radius-control" },
      { what: "The hairline inside its edge: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "The gap between it and its menu.", token: "space-s4" },
      { what: "The menu's items, eight before it scrolls.", token: "size-control_sm" },
    ],
    reflow: {
      described: "A select fills the room it is given at every window size, its pick ending in an ellipsis where it doesn't fit; its menu is as wide as it.",
      sample: () => within("100%", stack("s8", "start", select({ options: [shown(OPTION)], picked: 0, icon: "schedule" }), text("A select as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
