// The Slider page: `docs/design/components/slider.md`, in the state model (`docs/DESIGN.md`,
// "States"). A value or one of a few stops as a pill whose fill is the value, regular and compact:
// dragged, stepped, snapping onto its stops, held at a sticky point, the fill brightening lit and
// more while held, by night and by day.

import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack, within } from "../../lib/layout.ts";
import { type SliderOptions, type SliderSize, slider } from "../../lib/slider.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const PROFILES: Words = { short: "Saver", long: "Max Performance" };
const NAMES = ["Super Saver", "Power Saver", "Balanced", "Performance", "Max Performance"];
const AUTO: Words = { short: "Auto · 55%", long: "Automatic brightness · 55%" };

// The locale's percentage, for the sticky volume's own reading of its value.
const PERCENT = new Intl.NumberFormat("en-GB", { style: "percent", maximumFractionDigits: 0 });

/** The room every sample's slider fills here; a slider fills whatever room it is given. */
const ROOM = "var(--size-menu)";

/** `options`' slider in the samples' room. */
function boxed(options: SliderOptions): HTMLElement {
  return within(ROOM, slider(options));
}

/** Volume at 62%, whose icon mutes it and unmutes it back to where it was. */
function volume(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return {
    icon: "volume_up",
    size,
    value: 0.62,
    onIcon: (control) => {
      const muted = control.element.dataset.muted !== undefined;
      control.element.toggleAttribute("data-muted", !muted);
      control.name(muted ? undefined : "Muted");
    },
    ...extra,
  };
}

/** The power profiles: five stops, resting on Balanced, the icon stepping from high to low. */
function profiles(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "balance", size, stops: 5, index: 2, names: NAMES, ...extra };
}

/** Volume boosted up to 150%, holding at 100% while a drag passes it. */
function boosted(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "volume_up", size, value: 2 / 3, sticky: 2 / 3, number: (at) => PERCENT.format(at * 1.5), widest: "150%", ...extra };
}

/** Brightness on Auto: the sensor's level in the softer accent, named. */
function auto(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "motion_photos_auto", size, value: 0.55, words: shown(AUTO), soft: true, ...extra };
}

/** The rows the matrix shows, each a slider in either size. */
const ROWS = [
  { name: "Volume, 62%", options: volume },
  { name: "Muted", options: (size: SliderSize, extra: Partial<SliderOptions> = {}) => ({ icon: "volume_off", size, value: 0, words: "Muted", ...extra }) },
  { name: "Auto brightness", options: auto },
  { name: "Five stops", options: profiles },
  { name: "Sticky at 100% of 150%", options: boosted },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

/** A row's slider in `state`, regular above compact. */
function cell(options: (size: SliderSize, extra?: Partial<SliderOptions>) => SliderOptions, state: Shown): HTMLElement {
  const held = state === "pressed";
  const pinned = held ? { pinned: "pressed" as const, held: true } : stateOptions(state);
  return within(ROOM, stack("s8", "start", slider(options("Regular", pinned)), slider(options("Compact", pinned))));
}

/** The specimens' samples: a regular and a compact volume, the profiles and Auto. */
function specimen(): HTMLElement {
  return within("var(--size-column)", stack("s16", "start", slider(volume()), slider(profiles()), slider(boosted()), slider(auto()), slider(volume("Compact")), slider(profiles("Compact"))));
}

/** A regular and a compact slider side by side in `state`, still: what a move plays on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => within(ROOM, stack("s8", "start", slider(volume("Regular", { ...stateOptions(state), still: true })), slider(volume("Compact", { ...stateOptions(state), still: true }))));
}

const ON = "a regular and a compact slider";

const SETTLES: Move = {
  name: "Settles on its stop",
  what: "Let go between stops, the fill eases onto the nearest one over `short` on `standard_decel`, from where the drag left it: `--shown`, the fraction the fill shows, from 0.4 to its stop's 0.5. Only once it has arrived, the anchor pops in on the stop, from nothing to its size over `short` on `pop`, a little past it and back. With reduced motion the fill is on its stop and the anchor shows at once.",
  tracks: [
    { part: "fill", on: "five stops", target: ".k-slider", property: "--shown", from: 0.4, to: 0.5, duration: "short", curve: "standard_decel" },
    { part: "anchor", on: "five stops", target: ".k-slider-anchor", property: "scale", from: 0, to: 1, duration: "short", cue: "short", curve: "pop" },
  ],
  reduced: [
    { part: "fill", on: "five stops", target: ".k-slider", property: "--shown", from: 0.4, to: 0.5, duration: "instant", curve: "linear" },
    { part: "anchor", on: "five stops", target: ".k-slider-anchor", property: "scale", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  back: "cuts",
  sample: () => within(ROOM, slider(profiles("Regular", { still: true }))),
};

const EASES: Move = {
  name: "A value set from elsewhere",
  what: "The value changes from outside (a volume key, another app): the fill eases from where it was to the new value over `short` on `standard`, here from 30% to 62%. The value's words show the new value at once. A drag never eases: the fill follows the pointer. With reduced motion it is there at once.",
  tracks: [{ part: "fill", on: ON, target: ".k-slider", property: "--shown", from: 0.3, to: 0.62, duration: "short", curve: "standard" }],
  reduced: [{ part: "fill", on: ON, target: ".k-slider", property: "--shown", from: 0.3, to: 0.62, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: pair(),
};

const NUMBER: Move = {
  name: "A compact slider's number shows",
  what: "A compact slider's percentage shows only lit or held: it fades in over `micro` on `standard` as the slider lights up, and out the same as it goes back to rest. A name (Muted, Auto · 55%, a stop's) always shows.",
  tracks: [{ part: "number", on: "a compact slider", target: ".k-slider-label", property: "opacity", from: 0, to: 1, duration: "micro", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => within(ROOM, slider(volume("Compact", { still: true }))),
};

const DESCRIBED =
  "A pill as wide as its room, `size.slider` (36) tall, or `size.slider_compact` (28) for a dense panel. The track is `surface_container_highest` at `opacity.track`. The fill is the value: from the start to its round end, whose circle's centre is the value, on a whole device pixel, so at 0 it is the circle behind the icon and at 1 it reaches the far end. It is `primary`, or `secondary` for a level something else picked (Auto brightness), and its content is `on_primary` (`on_secondary`). The icon, `label`'s (16), is centred in the circle at the fill's start, always on the fill. The value sits `space.s12` in from the far end: a percentage in `label_number`, through the locale's formatting, or a name in `label`, in `on_surface` over the track and the fill's content colour over the fill, split where the fill ends. A number's box is as wide as its widest (100%, or 150% for a boosted volume), so it ends in one place.";

const STEPS =
  "A slider with stops shows a dot, `size.slider_stop` (6), at each: in the fill's content colour at `opacity.subdued` where the fill has passed it, `on_surface_variant` ahead. The stop the value rests on is solid instead, `size.slider_anchor` (10, 8 compact) in the fill's content colour, popping in over `short` on `pop` once the fill has arrived; while a drag moves between stops there is none. No dot is drawn under the icon or the label, `space.s4` clear of each, nor astride the fill's end. A sticky point (a volume boost to 150% holding at 100%) holds a drag within `size.slider_hold` (12) of it and shows its anchor while the value rests there.";

export default page({
  level: "molecules",
  slug: "slider",
  title: "Slider",
  summary: "A value from 0 to 1, or one of a few stops, as a pill whose fill is the value: drag it, step it, or press its icon.",
  live: specimen,
  description: [
    "Rust `slider::slider`, QML `Slider`. Drag any of these, use ← and → on them, or turn the wheel over them. The volume's icon mutes it and unmutes it back to where it was; the profiles' icon steps from high to low and back.",
    DESCRIBED,
    STEPS,
    "It takes effect as it moves, so it carries a live value. A drag follows the pointer, the pointer mapping to the fill's end circle's centre; on a slider with stops it moves freely and settles on the nearest stop when let go, over `short` on `standard_decel`. A value set from elsewhere eases there over `short` on `standard`. A compact slider shows its number only lit or held, fading over `micro`; a name always shows.",
    "Its states follow the neon (`docs/DESIGN.md`, \"States\"): rest, lit (under the pointer or with the keyboard on it, one state), held and disabled. Lit, the fill brightens by `opacity.press` toward the palette's bright end (`on_surface` at night, `surface` by day), and the tube burns inside its edge, `primary` at `focus.tube`, with the glow outside at night. Held, pressed or dragged, the fill brightens by `opacity.held`, and the glow by `focus.pressed_glow`. The icon, where it is a button, takes the fill's content colour as a state layer under the pointer and pressed, and the slider isn't lit while the pointer is on it.",
  ],
  variants: [
    { name: "Continuous", for: "Any value from 0 to 1: volume, brightness. Its value reads as a percentage.", sample: () => boxed(volume()) },
    { name: "With stops", for: "One of a few named choices in order: the power profiles. The stop's name shows; the icon steps from high to low and back.", sample: () => boxed(profiles()) },
    { name: "With a sticky point", for: "A value that goes past where it normally ends: a volume boost to 150%, holding at 100% as a drag passes.", sample: () => boxed(boosted()) },
    { name: "Named", for: "A value that is better said: Muted, Auto · 55%. The name always shows.", sample: () => boxed(volume("Regular", { value: 0, icon: "volume_off", words: "Muted" })) },
    { name: "Soft", for: "A level something else picked, in the softer, desaturated accent: brightness on Auto, the sensor's level.", sample: () => boxed(auto()) },
  ],
  sizes: [
    { name: "Regular", for: "`size.slider`, 36: a panel's or a page's slider.", sample: () => boxed(volume()) },
    { name: "Compact", for: "`size.slider_compact`, 28: a dense panel (the control center, the system monitor), as tall as a compact row. Its number shows only lit or held.", sample: () => boxed(volume("Compact")) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "its value is the fill, and a stop it rests on is its anchor, both in every row here.",
      loading: "it moves at once; a value still on its way leaves the slider where it was.",
      empty: "it always has a value: at 0 the fill is the circle behind the icon (Muted, here).",
      error: "it has no wrong value; a value that failed to apply eases back.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    described: {
      rest: "The fill `primary` (`secondary` soft), its content `on_primary`; the track `surface_container_highest` at `opacity.track`, its words `on_surface`. A compact slider hides its number. By day the same roles in daylight colours.",
      lit: "Under the pointer or with the keyboard on it, here with the signs on: the fill brightens by `opacity.press` toward `on_surface`, the tube burns inside the pill's edge, `primary` at `focus.tube`, with the glow outside, and a compact slider shows its number. By day the fill brightens toward `surface`, and the line is the tube by day with no glow.",
      pressed: "Held, pressed or dragged, so lit as well: the fill brightens by `opacity.held`, plainly, and the glow is `focus.pressed_glow` times the lit one. Letting go of a slider with stops settles it on the nearest.",
      disabled: "Every part at `opacity.disabled_content`, where its value puts it, so its value still reads: no line, no glow, no pointer, no keys, and no stop for Tab.",
    },
  },
  focus: {
    tubes: [{ name: "Every slider", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the pill's edge, over the fill and the track alike, with the glow outside, all that reaches past the pill. The fill brightens by `opacity.press`.",
      signsOff: "The same slider still lit with the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. The tube is solid glass, `primary_glass`, with no glow; the fill keeps its lit brightness, since a fill isn't a light.",
    },
    signsOff: (index: number) => cell(ROWS[index]!.options, "signs-off"),
  },
  content: {
    text: [
      { name: "A stop's name", words: PROFILES, sample: (words: string) => boxed({ icon: "balance", stops: 5, index: 4, names: NAMES.map((name, i) => (i === 4 ? words : name)) }) },
      { name: "A name for the value", words: AUTO, sample: (words: string) => boxed(auto("Regular", { words })) },
      { name: "In a compact slider", words: AUTO, sample: (words: string) => boxed(auto("Compact", { words })) },
    ],
    missingPicture: doesntApply("its icon is a symbol its owner names; a name with no drawing is the Icon page's."),
    counts: [
      { name: "No stops", for: "A continuous value: no dots, the value anywhere.", sample: () => boxed(volume()) },
      { name: "Two stops", for: "The fewest a slider with stops has: each end.", sample: () => boxed({ icon: "bedtime", stops: 2, index: 0, names: ["Off", "On"] }) },
      { name: "Five stops", for: "The power profiles.", sample: () => boxed(profiles()) },
      { name: "Many stops", for: "Eleven: dots too close to the icon, the label or the fill's end aren't drawn.", sample: () => boxed({ icon: "brightness_medium", stops: 11, index: 6 }) },
    ],
  },
  motion: {
    arrives: doesntApply("a slider comes with the surface it is on."),
    leaves: doesntApply("a slider goes with the surface it is on."),
    between: [
      { name: "Focus arrives", what: "Under the pointer or with the keyboard on it, it lights up over `short` on `standard`, never the flicker: the fill brightens by `opacity.press`, the tube and the glow fade in.", tracks: lights(".k-slider", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "The pointer leaves or focus moves on: it fades back to rest over `short` on `standard`.", tracks: lights(".k-slider", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Held", what: "A lit slider pressed or dragged: the fill brightens on to `opacity.held` and the glow to `focus.pressed_glow` times the lit one, over `micro`, and back as it is let go.", tracks: press(".k-slider", ON), reduced: "same", back: "plays", sample: pair("lit") },
      NUMBER,
      EASES,
      SETTLES,
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["←", "h"], does: "A twentieth lower (5%), or one stop lower." },
      { keys: ["→", "l"], does: "A twentieth higher, or one stop higher." },
      { keys: ["Home"], does: "To 0, or the first stop." },
      { keys: ["End"], does: "To 1, or the last stop. A boosted volume's end is 150%." },
      { keys: ["Tab", "Shift+Tab"], does: "To the slider, or on to the next or previous control. The icon's button is no stop of its own: the keys move the value." },
    ],
    order: "One stop in the surface's reading order; a disabled one is skipped. The wheel over it steps as the arrows do, a notch at a time.",
    lands: "It takes focus when the surface gives it: the control center lands on its first control, never on a slider.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Regular, five stops", for: "Its height, the icon's circle, the dots, the anchor and the label's inset.", sample: () => boxed(profiles("Regular", { still: true })) },
      { name: "Compact, five stops", for: "The compact height and anchor.", sample: () => boxed(profiles("Compact", { still: true })) },
      { name: "Sticky", for: "The sticky point's anchor and the number's widest box.", sample: () => boxed(boosted("Regular", { still: true })) },
    ]),
    measures: [
      { what: "A regular slider's height, and the circle the icon sits in.", token: "size-slider" },
      { what: "A compact slider's height.", token: "size-slider_compact" },
      { what: "A step's dot.", token: "size-slider_stop" },
      { what: "The anchor, regular.", token: "size-slider_anchor" },
      { what: "The anchor, compact.", token: "size-slider_anchor_compact" },
      { what: "How near a drag holds on a sticky point.", token: "size-slider_hold" },
      { what: "The icon, `label`'s.", token: "type-label-icon" },
      { what: "The value's line, `label_number` or `label`.", token: "type-label_number-line" },
      { what: "The value's inset from the far end.", token: "space-s12" },
      { what: "How clear of the icon and the label a dot stays.", token: "space-s4" },
      { what: "The pill's ends.", token: "radius-full" },
      { what: "The lit tube, rounded half up to device pixels.", token: "focus-tube" },
    ],
    reflow: {
      described: "A slider is as wide as its room at every window size: the fill, the dots and the anchor keep their places as fractions of the track, and the label stays `space.s12` from the end. Dots that come too close to the icon, the label or the fill's end aren't drawn.",
      sample: () => within("100%", stack("s16", "start", slider(volume()), slider({ icon: "brightness_medium", stops: 11, index: 6 }), text("A slider in a panel as wide as the window: drag the window's size in the toolbar.", "small", "on_surface_variant"))),
    },
  },
});
