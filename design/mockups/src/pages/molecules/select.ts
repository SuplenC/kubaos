// The Select page: `docs/design/components/select.md`. One of a list too long for radio rows or
// tabs: a field's look that grows into its list, one raised surface with the value row where it
// was. Lit as a field is; open, its hairline and tube go and its chevron turns up.

import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack, within } from "../../lib/layout.ts";
import { type SelectOptions, select } from "../../lib/select.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const COUNTRIES = ["None", "Germany", "Italy", "Poland"];
const MANY = ["None", "Austria", "Belgium", "Czechia", "Denmark", "Finland", "France", "Germany", "Greece", "Italy", "Netherlands", "Poland", "Portugal", "Spain"];
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

/** The holidays of fourteen countries: eight in view, Poland picked far down. */
function many(extra: Partial<SelectOptions> = {}): SelectOptions {
  return { options: MANY, picked: 11, icon: "flag", placeholder: "Pick a country", ...extra };
}

/** A select with no symbol: how often something runs. */
function often(extra: Partial<SelectOptions> = {}): SelectOptions {
  return { options: ["Every 5 minutes", "Hourly", "Daily"], picked: 0, placeholder: "How often", ...extra };
}

const ROWS = [
  { name: "With a symbol", options: holidays },
  { name: "Without", options: often },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "open" | "disabled";

function cell(options: (extra?: Partial<SelectOptions>) => SelectOptions, state: MatrixState): HTMLElement {
  if (state === "open") return boxed(options({ open: true }));
  return boxed(options(stateOptions(state as Shown)));
}

/** A select shut and lit, and the same with no symbol: what the lighting moves play on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => stack("s16", "start", boxed(holidays(stateOptions(state))), boxed(often(stateOptions(state))));
}

/** The holidays drawn open, in a room as tall as the open surface: what the opening moves play on. */
function opened(extra: Partial<SelectOptions> = {}): () => HTMLElement {
  return () => boxed(holidays({ open: true, ...extra }));
}

const ON = "two selects";
const SURFACE = "the surface: its height, fill, shadow, rim, hairline and the chevron's turn";

const OPENS: Move = {
  name: "Opening",
  what: "A press, Enter, Space, ↓ or ↑: the box grows from its shut height to the value row and its list over `medium` on `emphasized_decel`. The step's shadow and rim come up with it, its fill lifts to the open role, its hairline fades and the chevron turns up over the same. Nothing flickers: a surface isn't a light. With reduced motion it is open at once, the chevron up.",
  tracks: [{ part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: opened(),
};

const CLOSES: Move = {
  name: "Closing",
  what: "Esc, Tab, a press outside or on the value row: it folds back to the shut box over `short` on `emphasized_accel`, the shadow, rim and fill going down with it and the chevron turning back. Once it is shut, the tube lights again over `short` if the keyboard is on it. With reduced motion it is shut at once.",
  tracks: [{ part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 1, to: 0, duration: "short", curve: "emphasized_accel" }],
  reduced: [{ part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: opened(),
};

const PICKS: Move = {
  name: "Picking",
  what: "Enter, Space or a press on an option, here Germany: the value row shows it at once, the highlighted row presses to `secondary_container_pressed` over `micro`, and only then does the surface fold, over `short` on `emphasized_accel`, so the pick is seen landing before the list goes. With reduced motion the press shows and the surface is shut at once.",
  tracks: [
    { part: "the picked row's press", on: "the holidays", target: ".k-menu-item[data-current]", property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 1, to: 0, duration: "short", cue: "micro", curve: "emphasized_accel" },
  ],
  reduced: [
    { part: "the picked row's press", on: "the holidays", target: ".k-menu-item[data-current]", property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 1, to: 0, duration: "instant", cue: "micro", curve: "linear" },
  ],
  back: "cuts",
  sample: opened({ picked: 1 }),
};

const DESCRIBED =
  "Shut, a field's look: `size.control_md` (40) tall at the control radius, filling its room, `surface_container_high` with a field's sunken edge, a hairline of `outline` at full inside its edge, `space.s12` inside. The picked option is `body_large` in `on_surface` on one line, ending in an ellipsis where it doesn't fit; an optional `body_large` symbol (20) leads it and a chevron (`expand_more`) ends it, both `on_surface_variant`, `space.s8` from the words. With nothing picked it shows its placeholder in `on_surface_variant`.";

export default page({
  level: "molecules",
  slug: "select",
  title: "Select",
  summary: "One of a list too long for radio rows or tabs: a field's look that grows into its list.",
  live: () => within("var(--size-column)", stack("s16", "start", select(holidays()), select(many()), select(often()))),
  description: [
    "Rust `select::select`, QML `Select`. Press one, or Tab to it and press Enter, Space, ↓ or ↑: ↑ ↓ move the highlight, a letter jumps to the next option starting with it, Enter, Space or a press picks, and Esc, Tab or a press anywhere else closes it. The fourteen countries scroll past eight.",
    DESCRIBED,
    "Open, the select's own box grows into its list: one surface, no gap and no second popover. The value row stays where it was at the top and the options are under it; the box rises to the ladder's step 2, a menu's (its contact and ambient layers of `shadow` and the rim), on `surface_container_high` by day and the highest container at night, one role lighter. Its hairline goes, its tube goes out and the chevron turns up. A rule in a divider's soft outline (`outline_variant` at `opacity.outline_soft`) separates the row from the list, `space.s12` in from the sides. A select too near the bottom of the room that clips it, or of the window, for its list grows upward instead: the value row stays put and the list opens above it, the rule between them.",
    "The options are menu items (the Menu page): `size.control_sm` tall, `space.s4` inside the surface, their words `body` lined up with the value row's start, `space.s12` in, at the row radius (`radius.row`, 8), the control radius less `space.s4`. The highlighted row is a lit menu item, `secondary_container` with `on_secondary_container`, fading over `short`; the pointer moving onto a row and the keyboard move one highlight. The picked option's words are `primary_lit`, a picked tab's, with the label glow round them at night while the signs are on; highlighted, the pick takes the highlight's colours. There is no check. Past eight options the list scrolls with no bar of its own; the surface is the value row and the list's visible part, never the whole list, and it opens scrolled so the pick has three rows above it where the list allows. The highlight keeps itself in view, `space.s4` clear of the list's edge, and the value row never scrolls.",
    "Its states follow the neon, as a field's do: lit under the pointer or with the keyboard on it, its fill rises to `surface_container_highest` and the tube burns over its hairline, `primary` at `focus.tube`, with the glow outside at night; pressed, the glow brightens to `focus.pressed_glow` times the lit one. Disabled, it is grey as a disabled field is. Whatever closes or hides what holds the select (its panel closed by any route, its page swapped out) closes it too.",
  ],
  variants: [
    { name: "With a symbol", for: "Options a symbol helps place: the clock's holidays, by country.", sample: () => boxed(holidays()) },
    { name: "Without", for: "Plain options: how often something runs.", sample: () => boxed(often()) },
    { name: "With a placeholder", for: "Nothing picked yet: what to pick, in `on_surface_variant`, never a pick that looks made.", sample: () => boxed(holidays({ picked: -1 })) },
    { name: "A long list", for: "Fourteen countries, open: eight in view, scrolled so the pick has three rows above it.", sample: () => boxed(many({ open: true })) },
    { name: "Growing upward", for: "Too near the bottom of its room for its list: the value row stays put and the list opens above it.", sample: () => boxed(holidays({ open: true, up: true })) },
  ],
  sizes: doesntApply("a select has one height, a field's (`size.control_md`), and fills the room it is given."),
  states: {
    shown: ["rest", "lit", "pressed", "open", "disabled"],
    absent: {
      selected: "the picked option is what it shows; open, its words are `primary_lit`.",
      loading: "its options are known when it shows; a list still on its way leaves it disabled.",
      empty: "nothing picked is the placeholder, a variant above.",
      error: "a pick can't be wrong; a select that needs one shows the placeholder until it has it.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    described: {
      rest: "`surface_container_high`, the hairline `outline_variant`, the pick `on_surface`, the symbol and the chevron `on_surface_variant`. By day the same roles in daylight colours.",
      lit: "Under the pointer or with the keyboard on it, here with the signs on: the fill rises to `surface_container_highest`, the tube burns over the hairline, `primary` at `focus.tube`, with the glow outside. By day the tube with no glow.",
      pressed: "Lit and pressed: the glow `focus.pressed_glow` times the lit one; letting go opens it.",
      open: "Grown into its list, the highlight on the pick: raised to step 2 with its shadow and rim, `surface_container_highest` at night and `surface_container_high` by day, no hairline and no tube, the chevron up, the rule under the value row. The pick's words would be `primary_lit` with the highlight elsewhere.",
      disabled: "Grey, as a disabled field: the fill `on_surface` at `opacity.disabled_container` with no hairline, the pick, the symbol and the chevron at `opacity.disabled_content`. No press, no stop for Tab.",
    },
  },
  focus: {
    tubes: [{ name: "Every select, shut", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the tube over the hairline inside its edge, the glow outside, all that reaches past it. Open, there is no tube: the highlight shows where the keyboard is.",
      signsOff: "Still lit with the neon unpowered: by day always, and at night while a power change has the lights out. The tube is solid glass, `primary_glass`, with no glow; the fill stays risen. An open select's pick loses its label glow and keeps `primary_lit`.",
    },
    signsOff: (index: number) => cell((extra) => ROWS[index]!.options({ ...extra, pinned: "signs-off" }), "rest"),
  },
  content: {
    text: [{ name: "The picked option", words: OPTION, sample: (words: string) => boxed({ options: [words], picked: 0, icon: "schedule" }) }],
    missingPicture: [{ name: "No symbol", for: "A select may have none: its words start `space.s12` in.", sample: () => boxed(often()) }],
    counts: [
      { name: "Nothing picked", for: "The placeholder.", sample: () => boxed(holidays({ picked: -1 })) },
      { name: "A few options", for: "Four, open with the highlight off the pick: the surface as tall as they are, the pick's words `primary_lit`.", sample: () => boxed(holidays({ open: true, highlight: 1 })) },
      { name: "Many", for: "Fourteen: eight in view, the rest scrolled under the rule and the surface's edge.", sample: () => boxed(many({ open: true })) },
    ],
  },
  motion: {
    arrives: doesntApply("a select comes with the surface it is on; its list arrives as it opens, below."),
    leaves: doesntApply("a select goes with the surface it is on, shut."),
    between: [
      OPENS,
      CLOSES,
      PICKS,
      { name: "Focus arrives", what: "Under the pointer or with the keyboard on it, it lights up over `short` on `standard`: the fill rises, the tube and the glow fade in.", tracks: lights(".k-select", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "It goes back to rest over `short` on `standard`.", tracks: lights(".k-select", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "A lit select pressed: the glow brightens over `micro`, and back as it comes up; letting go opens it.", tracks: press(".k-select", ON), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Enter", "Space", "↓", "↑"], does: "On the shut select, opens it, the highlight on the pick (on the first option with nothing picked)." },
      { keys: ["↑", "↓"], does: "Open, the highlight to the option before or after, stopping at either end." },
      { keys: ["Home", "End"], does: "Open, the highlight to the first or the last option." },
      { keys: ["A letter"], does: "Open, the highlight to the next option starting with it, round from the end: pressed again, the one after." },
      { keys: ["Enter", "Space"], does: "Open, picks the highlighted option and closes it." },
      { keys: ["Esc"], does: "Open, closes it without picking." },
      { keys: ["Tab", "Shift+Tab"], does: "To the select, or on to the next or previous control; open, it closes without picking as the keyboard moves on." },
    ],
    order: "One stop in the surface's reading order; a disabled one is skipped. The keyboard never leaves the select while it is open: its list is no stop of its own.",
    lands: "On the select, open or shut: picking, Esc and a press elsewhere leave it there.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a symbol", for: "Its height, padding, symbol, chevron and the gaps between them.", sample: () => boxed(holidays()) },
      { name: "Open", for: "The value row, the rule, the list's padding and its rows.", sample: () => boxed(holidays({ open: true })) },
    ]),
    measures: [
      { what: "Its height shut, a field's, and the value row's open.", token: "size-control_md" },
      { what: "Inside its sides, and the rule's inset.", token: "space-s12" },
      { what: "Between the symbol, the words and the chevron.", token: "space-s8" },
      { what: "The symbol and the chevron, `body_large`'s.", token: "type-body_large-icon" },
      { what: "The pick's line.", token: "type-body_large-line" },
      { what: "Its corners, shut and open.", token: "radius-control" },
      { what: "An option's corners: the open box's less its padding.", token: "radius-row" },
      { what: "The hairline inside its edge, the rim open, and the rule: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "The list's padding inside the surface.", token: "space-s4" },
      { what: "The options, eight in view before it scrolls.", token: "size-control_sm" },
      { what: "The open shadow's ambient layer: its drop.", token: "elevation-2-ambient-y" },
      { what: "The open shadow's ambient layer: its blur.", token: "elevation-2-ambient-blur" },
    ],
    reflow: {
      described: "A select fills the room it is given at every window size, its pick ending in an ellipsis where it doesn't fit; open, its list is as wide as it. Near the bottom of its room or the window it grows upward.",
      sample: () => within("100%", stack("s8", "start", select({ options: [shown(OPTION)], picked: 0, icon: "schedule" }), text("A select as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
