// The Select page: `docs/design/components/select.md` is its full reference. One of a list too
// long for radio rows or tabs: a field's look that grows into its list, or the inline look, its
// words and chevron in a line of text with its Menu built in.

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

/** The Audio panel's devices, each with the symbol of how it connects. */
const DEVICES = ["Built-in speakers", "WH-1000XM4", "Arctis Nova Pro Wireless", "Headphones", "LG ULTRAGEAR"];
const DEVICE_ICONS = ["device_laptop", "bluetooth", "usb", "headphones", "device_tv"];

/** The default device, each option with its own symbol. */
function devices(extra: Partial<SelectOptions> = {}): SelectOptions {
  return { options: DEVICES, icons: DEVICE_ICONS, picked: 1, ...extra };
}

/**
 * A Bluetooth headset's codecs, each with what it is good for under its name: one line, two, and one
 * too long for two, cut with an ellipsis.
 */
function codecs(extra: Partial<SelectOptions> = {}): SelectOptions {
  return {
    options: ["LDAC", "AAC", "SBC"],
    descriptions: ["The best sound; drops out sooner far from the PC", "Very good sound, steady", "Plain sound; every headset has it, so it is the one left when the headset and the computer share no better codec"],
    picked: 0,
    ...extra,
  };
}

/** The devices that play, a crossed-out mic at the end of each that has none. */
function playing(extra: Partial<SelectOptions> = {}): SelectOptions {
  return { ...devices(), trailing: ["mic_off", undefined, undefined, "mic_off", "mic_off"], ...extra };
}

/** An app's device, inline under its name: Default first, then every device. */
function inline(extra: Partial<SelectOptions> = {}): SelectOptions {
  return { look: "inline", options: ["Default", ...DEVICES], icons: ["bluetooth", ...DEVICE_ICONS], picked: 0, ...extra };
}

/** An inline select under the name it belongs to, as an app's row has it. */
function named(options: SelectOptions): HTMLElement {
  return within(ROOM, stack(0, "start", text("Spotify", "body_strong"), select(options)));
}

/** A select with no symbol: how often something runs. */
function often(extra: Partial<SelectOptions> = {}): SelectOptions {
  return { options: ["Every 5 minutes", "Hourly", "Daily"], picked: 0, placeholder: "How often", ...extra };
}

const ROWS = [
  { name: "With a symbol", options: holidays },
  { name: "Without", options: often },
  { name: "Inline", options: inline },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "open" | "disabled";

function cell(options: (extra?: Partial<SelectOptions>) => SelectOptions, state: MatrixState): HTMLElement {
  const draw = options === inline ? named : boxed;
  if (state === "open") return draw(options({ open: true }));
  return draw(options(stateOptions(state as Shown)));
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
  what: "The box grows into its list over `medium` on `emphasized_decel`, its shadow and rim with it.",
  tracks: [{ part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: opened(),
};

const CLOSES: Move = {
  name: "Closing",
  what: "It folds back to the shut box over `short` on `emphasized_accel`; the tube lights again after.",
  tracks: [{ part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 1, to: 0, duration: "short", curve: "emphasized_accel" }],
  reduced: [{ part: SURFACE, on: "the holidays", target: ".k-select-room", property: "--select-open", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: opened(),
};

const PICKS: Move = {
  name: "Picking",
  what: "The row presses over `micro`, then the surface folds, so the pick is seen landing.",
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

export default page({
  level: "molecules",
  slug: "select",
  title: "Select",
  lede: "One of a list too long for radio rows or tabs: a field's look that grows into its list.",
  code: { rust: "select::select", qml: "Select" },
  stage: {
    live: () =>
      within(
        "var(--size-column)",
        stack("s16", "start", select(holidays()), select(many()), select(playing()), select(codecs()), select(often()), stack(0, "start", text("Spotify", "body_strong"), select(inline()))),
      ),
    hint: "click it, or Tab and ↓",
    daylight: true,
  },
  variants: [
    { name: "With a symbol", for: "options a symbol helps place", sample: () => boxed(holidays()) },
    { name: "Without", for: "plain options", sample: () => boxed(often()) },
    { name: "A symbol each", for: "devices, by how they connect", sample: () => boxed(devices()) },
    { name: "A symbol each, open", for: "every row with its own", sample: () => boxed(devices({ open: true })) },
    { name: "Descriptions", for: "a short line under each option: codecs", sample: () => boxed(codecs()) },
    { name: "Descriptions, open", for: "at most two lines, then cut; shut, the name alone", sample: () => boxed(codecs({ open: true })) },
    { name: "A trailing symbol, open", for: "at a row's end, muted: no mic", sample: () => boxed(playing({ open: true })) },
    { name: "Inline", for: "a choice in a line of text: an app's device", sample: () => named(inline()) },
    { name: "Inline, open", for: "its Menu under the words", sample: () => named(inline({ open: true })) },
    { name: "With a placeholder", for: "nothing picked yet", sample: () => boxed(holidays({ picked: -1 })) },
    { name: "A long list", for: "eight in view, the pick three rows down", sample: () => boxed(many({ open: true })) },
    { name: "Growing upward", for: "too near its room's foot", sample: () => boxed(holidays({ open: true, up: true })) },
  ],
  sizes: doesntApply("one height, a field's, filling the room it is given."),
  states: {
    shown: ["rest", "lit", "pressed", "open", "disabled"],
    absent: {
      selected: "the pick is what it shows",
      loading: "a list on its way leaves it disabled",
      empty: "the placeholder, a variant",
      error: "a pick can't be wrong",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    looks: {
      rest: "`surface_container_high`, an `outline_variant` hairline",
      lit: "fill to `…_highest`, the `primary` tube, glow at night",
      pressed: "as lit, glow × `focus.pressed_glow`",
      open: "raised to step 2, no hairline or tube, chevron up",
      disabled: "grey, no hairline, no Tab stop",
    },
  },
  focus: {
    tubes: [{ name: "Every select, shut, but inline", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the tube over the hairline, glow outside; inline: words `primary_lit`",
      signsOff: "the tube as `primary_glass`, no glow",
    },
    signsOff: (index: number) => cell((extra) => ROWS[index]!.options({ ...extra, pinned: "signs-off" }), "rest"),
  },
  content: {
    text: [{ name: "The picked option", words: OPTION, sample: (words: string) => boxed({ options: [words], picked: 0, icon: "schedule" }) }],
    missingPicture: [{ name: "No symbol", for: "words start `space.s12` in", sample: () => boxed(often()) }],
    counts: [
      { name: "Nothing picked", for: "the placeholder", sample: () => boxed(holidays({ picked: -1 })) },
      { name: "Inline, a long pick", for: "it ends in an ellipsis", sample: () => named(inline({ options: [OPTION.long], icons: [], picked: 0 })) },
      { name: "A few options", for: "as tall as they are", sample: () => boxed(holidays({ open: true, highlight: 1 })) },
      { name: "Many", for: "eight in view, the rest scroll", sample: () => boxed(many({ open: true })) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface; its list arrives as it opens."),
    leaves: doesntApply("it goes with its surface, shut."),
    between: [
      OPENS,
      CLOSES,
      PICKS,
      { name: "Focus arrives", what: "It lights up over `short` on `standard`: the fill rises, the tube and glow fade in.", tracks: lights(".k-select", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "Back to rest over `short` on `standard`.", tracks: lights(".k-select", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "The glow brightens over `micro`, and back as it comes up.", tracks: press(".k-select", ON), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Enter", "Space", "↓", "↑"], does: "shut: opens it, highlight on the pick" },
      { keys: ["↑", "↓"], does: "open: the highlight up or down" },
      { keys: ["Home", "End"], does: "open: the first or last option" },
      { keys: ["A letter"], does: "open: the next option starting with it" },
      { keys: ["Enter", "Space"], does: "open: picks and closes" },
      { keys: ["Esc"], does: "open: closes without picking" },
      { keys: ["Tab", "Shift+Tab"], does: "on to the next control, closing it" },
    ],
    order: "One stop in reading order; a disabled one is skipped, and its list is never a stop.",
    lands: "On the select, open or shut.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a symbol", for: "height, inset, symbol, chevron", sample: () => boxed(holidays()) },
      { name: "Open", for: "value row, rule, list padding, rows", sample: () => boxed(holidays({ open: true })) },
      { name: "Inline, open", for: "words, chevron, the Menu's place", sample: () => named(inline({ open: true })) },
    ]),
    measures: [
      { part: "Height", token: "size-control_md" },
      { part: "Inset, rule inset", token: "space-s12" },
      { part: "Symbol → words → chevron", token: "space-s8" },
      { part: "Symbol, chevron", token: "type-body_large-icon" },
      { part: "Value line", token: "type-body_large-line" },
      { part: "Corner", token: "radius-control" },
      { part: "Option corner", token: "radius-row" },
      { part: "Hairline, rim, rule", token: "stroke-hairline_device_px" },
      { part: "List padding", token: "space-s4" },
      { part: "Option row", token: "size-control_sm" },
      { part: "Shadow drop", token: "elevation-2-ambient-y" },
      { part: "Shadow blur", token: "elevation-2-ambient-blur" },
      { part: "Inline: words", token: "type-small-line" },
      { part: "Inline: words → chevron", token: "space-s4" },
      { part: "Inline: chevron", token: "type-small-icon" },
      { part: "Inline: words → Menu", token: "space-s4" },
      { part: "Inline: Menu's width", token: "size-menu" },
    ],
    reflow: {
      note: "It fills the room it is given; the pick ends in an ellipsis where it doesn't fit.",
      sample: () => within("100%", stack("s8", "start", select({ options: [shown(OPTION)], picked: 0, icon: "schedule" }), text("A select as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
