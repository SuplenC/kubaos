// The Slider page: `docs/design/components/slider.md`, in the state model (`docs/DESIGN.md`,
// "States"). A value or one of a few stops as a pill whose fill is the value, regular and compact:
// dragged, stepped, snapping onto its stops, held at a sticky point, toggled by its icon (muted, on
// Auto), the fill brightening lit and more while held, and its icon's button lit and pressed, by
// night and by day. The knob variant, balance, moves a knob either side of a sticky centre.

import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { hintRoom } from "../../lib/hint.ts";
import { stack, within } from "../../lib/layout.ts";
import { type SliderOptions, type SliderSize, type SliderToggle, slider } from "../../lib/slider.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const PROFILES: Words = { short: "Saver", long: "Max Performance" };
const NAMES = ["Super Saver", "Power Saver", "Balanced", "Performance", "Max Performance"];
const AUTO: Words = { short: "Auto · 55%", long: "Automatic brightness · 55%" };
const BALANCE: Words = { short: "Centre", long: "Right 100%" };

// The locale's percentage, for the sticky volume's own reading of its value.
const PERCENT = new Intl.NumberFormat("en-GB", { style: "percent", maximumFractionDigits: 0 });

/** The room every sample's slider fills here; a slider fills whatever room it is given. */
const ROOM = "var(--size-menu)";

/** `options`' slider in the samples' room. */
function boxed(options: SliderOptions): HTMLElement {
  return within(ROOM, slider(options));
}

/** Volume's toggle: muted, its fill kept at the level it comes back to. */
const MUTE: SliderToggle = { icon: "volume_off", words: "Muted", label: "Mute", toggledLabel: "Unmute" };

/** Brightness's toggle, on Auto: the sensor's level, named. */
function autoToggle(words: string = shown(AUTO)): SliderToggle {
  return { icon: "motion_photos_auto", words, label: "Auto brightness", toggledLabel: "Manual brightness", on: true };
}

/** Volume at 62%, whose icon mutes it and unmutes it back to where it was. */
function volume(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "volume_up", size, value: 0.62, toggle: MUTE, ...extra };
}

/** Volume muted at 62%: the softer fill at the level it unmutes to. */
function muted(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return volume(size, { toggle: { ...MUTE, on: true }, ...extra });
}

/** The power profiles: five stops, resting on Balanced, the icon stepping from high to low. */
function profiles(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "balance", size, stops: 5, index: 2, names: NAMES, icons: ["bedtime", "eco", "balance", "bolt", "rocket_launch"], ...extra };
}

/** Volume boosted up to 150%, holding at 100% while a drag passes it. */
function boosted(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "volume_up", size, value: 2 / 3, sticky: 2 / 3, number: (at) => PERCENT.format(at * 1.5), widest: "150%", ...extra };
}

/** Balance as its words: the side and how far toward it, or Centre in the middle. */
function sideOf(at: number): string {
  const far = Math.round(Math.abs(at - 0.5) * 200);
  return far === 0 ? "Centre" : `${at < 0.5 ? "Left" : "Right"} ${PERCENT.format(far / 100)}`;
}

/** Balance: a knob either side of the centre, held there within `size.slider_hold`. */
function balance(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "balance", size, knob: true, value: 0.5, sticky: 0.5, number: sideOf, ...extra };
}

/** Balance 30% to the left. */
function leftward(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return balance(size, { value: 0.35, ...extra });
}

/** Brightness on Auto at 55%: the sensor's level in the softer accent, named; its icon goes manual. */
function auto(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "brightness_medium", size, value: 0.55, toggle: autoToggle(), ...extra };
}

type Options = (size: SliderSize, extra?: Partial<SliderOptions>) => SliderOptions;

/** The rows the matrix shows, each a slider in either size: the slider in each state, then its icon's button. */
const ROWS: readonly { readonly name: string; readonly options: Options; readonly icon?: boolean }[] = [
  { name: "Volume, 62%", options: volume },
  { name: "Volume toggled (Muted)", options: muted },
  { name: "Brightness toggled (Auto)", options: auto },
  { name: "Five stops", options: profiles },
  { name: "Sticky at 100% of 150%", options: boosted },
  { name: "Knob at the centre", options: balance },
  { name: "Knob, left 30%", options: leftward },
  { name: "Volume, its icon", options: volume, icon: true },
  { name: "Muted, its icon", options: muted, icon: true },
  { name: "Five stops, its icon", options: profiles, icon: true },
];

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

/** A row's slider in `state`, regular above compact. */
function cell(options: Options, state: Shown): HTMLElement {
  const held = state === "pressed";
  const pinned = held ? { pinned: "pressed" as const, held: true } : stateOptions(state);
  return within(ROOM, stack("s8", "start", slider(options("Regular", pinned)), slider(options("Compact", pinned))));
}

/**
 * A row's slider with its icon's button in `state`, regular above compact, each with the room its
 * hint floats in.
 */
function iconCell(options: Options, state: Shown): HTMLElement {
  const { pinned, disabled } = stateOptions(state);
  const each = (size: SliderSize) => {
    const made = slider(options(size, { ...(disabled ? { disabled } : {}), ...(pinned ? { pinnedIcon: pinned } : {}) }));
    const words = made.querySelector(".k-slider-icon-button")?.getAttribute("aria-label") ?? "";
    return hintRoom(made, words, ROOM, "start");
  };
  return within(ROOM, stack(0, "start", each("Regular"), each("Compact")));
}

/** The specimens' samples: a regular and a compact volume, the profiles and Auto. */
function specimen(): HTMLElement {
  return within(
    "var(--size-column)",
    stack("s16", "start", slider(volume()), slider(profiles()), slider(boosted()), slider(auto()), slider(balance()), slider(volume("Compact")), slider(profiles("Compact")), slider(balance("Compact", { value: 0.575 }))),
  );
}

/** A regular and a compact slider side by side in `state`, still: what a move plays on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => within(ROOM, stack("s8", "start", slider(volume("Regular", { ...stateOptions(state), still: true })), slider(volume("Compact", { ...stateOptions(state), still: true }))));
}

const ON = "a regular and a compact slider";

const SETTLES: Move = {
  name: "Settles on its stop",
  what: "Let go between stops, the fill eases on over `short` on `standard_decel`, then the anchor pops in on `pop`.",
  tracks: [
    { part: "fill", on: "five stops", target: ".k-slider", property: "--slider-shown", from: 0.4, to: 0.5, duration: "short", curve: "standard_decel" },
    { part: "anchor", on: "five stops", target: ".k-slider-anchor", property: "scale", from: 0, to: 1, duration: "short", cue: "short", curve: "pop" },
  ],
  reduced: [
    { part: "fill", on: "five stops", target: ".k-slider", property: "--slider-shown", from: 0.4, to: 0.5, duration: "instant", curve: "linear" },
    { part: "anchor", on: "five stops", target: ".k-slider-anchor", property: "scale", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  back: "cuts",
  sample: () => within(ROOM, slider(profiles("Regular", { still: true }))),
};

const EASES: Move = {
  name: "A value set from elsewhere",
  what: "From outside, the fill eases to the new value over `short` on `standard`; its words change at once.",
  tracks: [{ part: "fill", on: ON, target: ".k-slider", property: "--slider-shown", from: 0.3, to: 0.62, duration: "short", curve: "standard" }],
  reduced: [{ part: "fill", on: ON, target: ".k-slider", property: "--slider-shown", from: 0.3, to: 0.62, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: pair(),
};

const TOGGLES: Move = {
  name: "Toggled",
  what: "The icon pressed: the fill fades to `secondary` over `short`, staying at its level; the icon and name change at once.",
  tracks: [{ part: "fill", on: ON, target: ".k-slider", property: "--toggled", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: pair(),
};

const ICON: Move = {
  name: "Its icon lights",
  what: "The pointer on the icon lights its button, not the slider, over `short` on `standard`, with its hint.",
  tracks: lights(".k-slider-icon-button", "the icon's button", 0, 1),
  reduced: "same",
  back: "plays",
  sample: pair(),
};

/** A regular and a compact knob, still, at `value`. */
function knobs(value: number, state: Shown = "rest"): () => HTMLElement {
  return () => within(ROOM, stack("s8", "start", slider(balance("Regular", { value, ...stateOptions(state), still: true })), slider(balance("Compact", { value, ...stateOptions(state), still: true }))));
}

const KNOBS = "a regular and a compact knob";

const CAUGHT: Move = {
  name: "The knob caught at the centre",
  what: "Dragged within `size.slider_hold`, the knob jumps to the centre at once; the anchor pops in on `pop`.",
  tracks: [
    { part: "knob", on: KNOBS, target: ".k-slider", property: "--slider-shown", from: 0.47, to: 0.5, duration: "instant", curve: "linear" },
    { part: "anchor", on: KNOBS, target: ".k-slider-anchor", property: "scale", from: 0, to: 1, duration: "short", curve: "pop" },
  ],
  reduced: [
    { part: "knob", on: KNOBS, target: ".k-slider", property: "--slider-shown", from: 0.47, to: 0.5, duration: "instant", curve: "linear" },
    { part: "anchor", on: KNOBS, target: ".k-slider-anchor", property: "scale", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  back: "cuts",
  sample: knobs(0.5),
};

const KNOB_SET: Move = {
  name: "The knob set from elsewhere",
  what: "From outside, the knob eases to the new value over `short` on `standard`; the words change at once.",
  tracks: [{ part: "knob", on: KNOBS, target: ".k-slider", property: "--slider-shown", from: 0.35, to: 0.575, duration: "short", curve: "standard" }],
  reduced: [{ part: "knob", on: KNOBS, target: ".k-slider", property: "--slider-shown", from: 0.35, to: 0.575, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: knobs(0.575),
};

const NUMBER: Move = {
  name: "A compact slider's number shows",
  what: "A compact slider's percentage fades in over `micro` as it lights; a name always shows.",
  tracks: [{ part: "number", on: "a compact slider", target: ".k-slider-label", property: "opacity", from: 0, to: 1, duration: "micro", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => within(ROOM, slider(volume("Compact", { still: true }))),
};

export default page({
  level: "molecules",
  slug: "slider",
  title: "Slider",
  lede: "A value or one of a few stops as a pill whose fill is the value: drag it, step it, or press its icon.",
  code: { rust: "slider::slider", qml: "Slider" },
  stage: { live: specimen, hint: "drag, ← →, or press an icon", daylight: true },
  variants: [
    { name: "Continuous", for: "any value: volume, brightness; reads as a percentage", sample: () => boxed(volume()) },
    { name: "With stops", for: "one of a few named choices in order: power profiles", sample: () => boxed(profiles()) },
    { name: "A symbol per stop", for: "each stop's own in the circle: the power stops", sample: () => within(ROOM, stack("s16", "start", ...[0, 2, 4].map((index) => slider(profiles("Regular", { index }))))) },
    { name: "With a sticky point", for: "a volume boost to 150%, holding at 100%", sample: () => boxed(boosted()) },
    {
      name: "Toggled",
      for: "a second state its icon turns: Muted, Auto; softer fill",
      sample: () => within(ROOM, stack("s16", "start", slider(volume()), slider(auto()))),
    },
    {
      name: "Knob",
      for: "either side of a sticky centre: balance; no fill",
      sample: () => within(ROOM, stack("s16", "start", ...[0.5, 0.35, 0.575].map((value) => slider(balance("Regular", { value }))))),
    },
  ],
  sizes: [
    { name: "Regular", for: "`size.slider`: a panel's or a page's", sample: () => boxed(volume()) },
    { name: "Compact", for: "`size.slider_compact`: a dense panel; number only lit or held", sample: () => boxed(volume("Compact")) },
    { name: "Knob, regular and compact", for: "the knob `space.s4` inside either height", sample: () => within(ROOM, stack("s8", "start", slider(leftward()), slider(leftward("Compact")))) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "its value is the fill; Toggled has its own rows",
      loading: "it moves at once",
      empty: "at 0 the fill is the circle behind the icon",
      error: "a value that failed to apply eases back",
    },
    rows: nonEmpty(ROWS.map(({ name, options, icon }) => ({ name, cell: (state: MatrixState) => (icon ? iconCell(options, state) : cell(options, state)) }))),
    looks: {
      rest: "`primary` fill or knob; toggled, `secondary`; compact hides its number",
      lit: "fill or knob + `opacity.press`, tube, glow; icon rows: the button",
      pressed: "fill or knob + `opacity.held`, glow × `focus.pressed_glow`",
      disabled: "every part at `opacity.disabled_content`; no pointer, keys or Tab",
    },
  },
  focus: {
    tubes: [
      { name: "Every slider", tone: "accent", filled: false, content: false },
      { name: "The icon's button", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "the tube inside the pill's edge, glow outside; the fill brightens",
      signsOff: "the tube as `primary_glass`, no glow; the fill keeps its brightness",
    },
    signsOff: (index: number) => {
      const row = ROWS[index]!;
      return row.icon ? iconCell(row.options, "signs-off") : cell(row.options, "signs-off");
    },
  },
  content: {
    text: [
      { name: "A stop's name", words: PROFILES, sample: (words: string) => boxed({ icon: "balance", stops: 5, index: 4, names: NAMES.map((name, i) => (i === 4 ? words : name)) }) },
      { name: "A toggled value's name", words: AUTO, sample: (words: string) => boxed(auto("Regular", { toggle: autoToggle(words) })) },
      { name: "In a compact slider", words: AUTO, sample: (words: string) => boxed(auto("Compact", { toggle: autoToggle(words) })) },
      { name: "A knob's words", words: BALANCE, sample: (words: string) => boxed(balance("Regular", { value: 0.15, number: () => words })) },
    ],
    missingPicture: doesntApply("its icon is a symbol its owner names."),
    counts: [
      { name: "No stops", for: "no dots, the value anywhere", sample: () => boxed(volume()) },
      { name: "Two stops", for: "the fewest: each end", sample: () => boxed({ icon: "bedtime", stops: 2, index: 0, names: ["Off", "On"] }) },
      { name: "Five stops", for: "the power profiles", sample: () => boxed(profiles()) },
      { name: "Many stops", for: "dots too close to icon, label or fill aren't drawn", sample: () => boxed({ icon: "brightness_medium", stops: 11, index: 6 }) },
      { name: "Knob at either end", for: "it covers the icon, or the words", sample: () => within(ROOM, stack("s16", "start", slider(balance("Regular", { value: 0 })), slider(balance("Regular", { value: 1 })))) },
    ],
  },
  motion: {
    arrives: doesntApply("a slider comes with its surface."),
    leaves: doesntApply("a slider goes with its surface."),
    between: [
      { name: "Focus arrives", what: "Pointer or keyboard: over `short` on `standard`, the fill brightens, tube and glow fade in.", tracks: lights(".k-slider", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "Back to rest over `short` on `standard`.", tracks: lights(".k-slider", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Held", what: "Pressed or dragged: the fill to `opacity.held`, the glow to `focus.pressed_glow`, over `micro`.", tracks: press(".k-slider", ON), reduced: "same", back: "plays", sample: pair("lit") },
      TOGGLES,
      ICON,
      NUMBER,
      EASES,
      SETTLES,
      { name: "The knob lights", what: "Pointer or keyboard: over `short` on `standard`, the knob brightens, tube and glow fade in.", tracks: lights(".k-slider", KNOBS, 0, 1), reduced: "same", back: "plays", sample: knobs(0.35) },
      { name: "The knob held", what: "Pressed or dragged: the knob to `opacity.held`, the glow to `focus.pressed_glow`, over `micro`.", tracks: press(".k-slider", KNOBS), reduced: "same", back: "plays", sample: knobs(0.35, "lit") },
      CAUGHT,
      KNOB_SET,
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["←", "h"], does: "5% or one stop lower; leaves Toggled; a knob 10% left" },
      { keys: ["→", "l"], does: "5% or one stop higher; leaves Toggled; a knob 10% right" },
      { keys: ["Home"], does: "to 0, or the first stop; a knob fully left" },
      { keys: ["End"], does: "to 1, the last stop, a boost's 150%; a knob fully right" },
      {
        keys: ["Space"],
        does: "presses the icon, where it is a button, as the key lifts",
      },
      { keys: ["Tab", "Shift+Tab"], does: "to the slider or on; the icon's button is no stop" },
    ],
    order: "One stop in reading order; a horizontal wheel steps it, a vertical one scrolls past.",
    lands: "When the surface gives it; the control center never lands on a slider.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Regular, five stops", for: "height, icon's circle, dots, anchor, label inset", sample: () => boxed(profiles("Regular", { still: true })) },
      { name: "Compact, five stops", for: "compact height and anchor", sample: () => boxed(profiles("Compact", { still: true })) },
      { name: "Sticky", for: "sticky anchor, number's widest box", sample: () => boxed(boosted("Regular", { still: true })) },
      { name: "Knob, left 30%", for: "knob and its inset, centre dot, words", sample: () => boxed(leftward("Regular", { still: true })) },
      { name: "Knob at the centre", for: "the anchor on the knob", sample: () => boxed(balance("Compact", { still: true })) },
    ]),
    measures: [
      { part: "Height, icon's circle", token: "size-slider" },
      { part: "Compact height", token: "size-slider_compact" },
      { part: "Step's dot", token: "size-slider_stop" },
      { part: "Anchor", token: "size-slider_anchor" },
      { part: "Anchor, compact", token: "size-slider_anchor_compact" },
      { part: "Sticky hold", token: "size-slider_hold" },
      { part: "Knob inset, all round", token: "space-s4" },
      { part: "Icon", token: "type-label-icon" },
      { part: "Value's line", token: "type-label_number-line" },
      { part: "Value's inset", token: "space-s12" },
      { part: "Dot clear of icon, label", token: "space-s4" },
      { part: "Pill ends", token: "radius-full" },
      { part: "Lit tube", token: "focus-tube" },
    ],
    reflow: {
      note: "As wide as its room; fill, dots and anchor keep their fractions, the label `space.s12` from the end.",
      sample: () => within("100%", stack("s16", "start", slider(volume()), slider({ icon: "brightness_medium", stops: 11, index: 6 }))),
    },
  },
});
