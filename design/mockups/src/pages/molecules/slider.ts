// The Slider page: `docs/design/components/slider.md`, in the state model (`docs/DESIGN.md`,
// "States"). A value or one of a few stops as a pill whose fill is the value, regular and compact:
// dragged, stepped, snapping onto its stops, held at a sticky point, toggled by its icon (muted, on
// Auto), the fill brightening lit and more while held, and its icon's button lit and pressed, by
// night and by day.

import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { hintRoom } from "../../lib/hint.ts";
import { stack, within } from "../../lib/layout.ts";
import { type SliderOptions, type SliderSize, type SliderToggle, slider } from "../../lib/slider.ts";
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
  return { icon: "balance", size, stops: 5, index: 2, names: NAMES, ...extra };
}

/** Volume boosted up to 150%, holding at 100% while a drag passes it. */
function boosted(size: SliderSize = "Regular", extra: Partial<SliderOptions> = {}): SliderOptions {
  return { icon: "volume_up", size, value: 2 / 3, sticky: 2 / 3, number: (at) => PERCENT.format(at * 1.5), widest: "150%", ...extra };
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
  return within("var(--size-column)", stack("s16", "start", slider(volume()), slider(profiles()), slider(boosted()), slider(auto()), slider(volume("Compact")), slider(profiles("Compact"))));
}

/** A regular and a compact slider side by side in `state`, still: what a move plays on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => within(ROOM, stack("s8", "start", slider(volume("Regular", { ...stateOptions(state), still: true })), slider(volume("Compact", { ...stateOptions(state), still: true }))));
}

const ON = "a regular and a compact slider";

const SETTLES: Move = {
  name: "Settles on its stop",
  what: "Let go between stops, the fill eases onto the nearest one over `short` on `standard_decel`, from where the drag left it: `--slider-shown`, the fraction the fill shows, from 0.4 to its stop's 0.5. Only once it has arrived, the anchor pops in on the stop, from nothing to its size over `short` on `pop`, a little past it and back. With reduced motion the fill is on its stop and the anchor shows at once.",
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
  what: "The value changes from outside (a volume key, another app): the fill eases from where it was to the new value over `short` on `standard`, here from 30% to 62%. The value's words show the new value at once. A drag never eases: the fill follows the pointer. With reduced motion it is there at once.",
  tracks: [{ part: "fill", on: ON, target: ".k-slider", property: "--slider-shown", from: 0.3, to: 0.62, duration: "short", curve: "standard" }],
  reduced: [{ part: "fill", on: ON, target: ".k-slider", property: "--slider-shown", from: 0.3, to: 0.62, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: pair(),
};

const TOGGLES: Move = {
  name: "Toggled",
  what: "The icon pressed on a slider with a toggle, here muting the volume: `--toggled` from 0 to 1 over `short` on `standard`. The fill's tone fades from `primary` to the softer `secondary` and its content from `on_primary` to `on_secondary`; the fill stays where it is, at the level (62%) it unmutes to. The icon (`volume_off`) and the value's name (Muted) change at once. Pressed again, or the value moved by a drag, a key or the wheel, it plays backwards. Reduced motion plays it the same: a fade of colour, with nothing moving.",
  tracks: [{ part: "fill", on: ON, target: ".k-slider", property: "--toggled", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: pair(),
};

const ICON: Move = {
  name: "Its icon lights",
  what: "The pointer over the icon's button lights the button and not the slider, as an icon button lights, over `short` on `standard`: its circle lifts by `opacity.press` toward the bright end, its tube and glow fade in, and its hint fades in under it. It goes back to rest the same way as the pointer leaves.",
  tracks: lights(".k-slider-icon-button", "the icon's button", 0, 1),
  reduced: "same",
  back: "plays",
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
  "A pill as wide as its room, `size.slider` (28) tall, or `size.slider_compact` (24) for a dense panel. The track is `surface_container_highest` at `opacity.track`. The fill is the value: from the start to its round end, whose circle's centre is the value, on a whole device pixel, so at 0 it is the circle behind the icon and at 1 it reaches the far end. It is `primary`, its content `on_primary`; toggled, the softer `secondary`, its content `on_secondary`. The icon, `label`'s (16), is centred in the circle at the fill's start, always on the fill. The value sits `space.s12` in from the far end: a percentage in `label_number`, through the locale's formatting, or a name in `label`, in `on_surface` over the track and the fill's content colour over the fill, split where the fill ends. A number's box is as wide as its widest (100%, or 150% for a boosted volume), so it ends in one place.";

const TOGGLED =
  "A continuous slider may have a toggle, which makes its icon a button and gives it a second state, Toggled: volume's icon mutes it, brightness's switches Auto on, where a light sensor and a service driving the backlight from it exist. Toggled, the fill fades to the softer accent over `short` on `standard`, the icon is the toggled one (`volume_off`, the Auto symbol) and the value reads as a name (Muted, Auto · 55%). The fill stays at a level: muted, the one unmuting comes back to; on Auto, the sensor's, as it changes. Pressing the icon again goes back. Moving the value, by a drag, a key or the wheel, leaves Toggled from where the value is: a drag on a muted volume unmutes it and the value follows the pointer, and on Auto it goes manual at the dragged value. The owner is told each time it turns. A slider with stops has no toggle: its icon steps one stop from high to low and back, turning at either end.";

const STEPS =
  "A slider with stops shows a dot, `size.slider_stop` (6), at each: in the fill's content colour at `opacity.subdued` where the fill has passed it, `on_surface_variant` ahead. The stop the value rests on is solid instead, `size.slider_anchor` (10, 8 compact) in the fill's content colour, popping in over `short` on `pop` once the fill has arrived; while a drag moves between stops there is none. No dot is drawn under the icon or the label, `space.s4` clear of each, nor astride the fill's end. A sticky point (a volume boost to 150% holding at 100%) holds a drag within `size.slider_hold` (12) of it and shows its anchor while the value rests there.";

export default page({
  level: "molecules",
  slug: "slider",
  title: "Slider",
  summary: "A value from 0 to 1, or one of a few stops, as a pill whose fill is the value: drag it, step it, or press its icon.",
  live: specimen,
  description: [
    "Rust `slider::slider`, QML `Slider`. Drag any of these, use ← and → on them, or tilt the wheel (or turn it with Shift) over them. The volume's icon mutes it and unmutes it back to where it was; the brightness's turns Auto off and on; the profiles' icon steps from high to low and back. With the keyboard on one, Space presses its icon.",
    DESCRIBED,
    TOGGLED,
    STEPS,
    "It takes effect as it moves, so it carries a live value. A drag follows the pointer, the pointer mapping to the fill's end circle's centre; on a slider with stops it moves freely and settles on the nearest stop when let go, over `short` on `standard_decel`. A value set from elsewhere eases there over `short` on `standard`. A compact slider shows its number only lit or held, fading over `micro`; a name always shows.",
    "Its states follow the neon (`docs/DESIGN.md`, \"States\"): rest, lit (under the pointer or with the keyboard on it, one state), held and disabled. Lit, the fill brightens by `opacity.press` toward the palette's bright end (`on_surface` at night, `surface` by day), and the tube burns inside its edge, `primary` at `focus.tube`, with the glow outside at night. Held, pressed or dragged, the fill brightens by `opacity.held`, and the glow by `focus.pressed_glow`.",
    "The icon, where it is a button, shows nothing of it at rest: it is the icon on the fill. Under the pointer it lights as a Filled icon button in the fill's tone: its circle lifts by `opacity.press` toward the bright end, as the fill does, the tube burns inside the circle's edge with the glow outside at night (the tube with no glow by day), and its hint names what a press does: Mute or Unmute, Auto brightness or Manual brightness, or on a slider with stops the stop the press goes to (Power Saver, from Balanced heading down; at the low end, where it turns, the stop above). A slider with stops and no names says the stop's value instead, as the label reads it (75%). The hint starts at the slider's start, so a long one never hangs past its edge. Pressed, the circle lifts by `opacity.held` and the glow by `focus.pressed_glow`. While the icon's button is lit the slider isn't. It is no stop for Tab: the keys move the value, and Space presses it, lit and pressed while the key is down, its hint showing, and acting as the key comes up.",
  ],
  variants: [
    { name: "Continuous", for: "Any value from 0 to 1: volume, brightness. Its value reads as a percentage.", sample: () => boxed(volume()) },
    { name: "With stops", for: "One of a few named choices in order: the power profiles. The stop's name shows; the icon steps from high to low and back.", sample: () => boxed(profiles()) },
    { name: "With a sticky point", for: "A value that goes past where it normally ends: a volume boost to 150%, holding at 100% as a drag passes.", sample: () => boxed(boosted()) },
    {
      name: "Toggled",
      for: "With a toggle: a second state the icon turns on and off, in the softer accent with the value named. Volume mutes, keeping its level; brightness goes on Auto, the fill the sensor's level. Moving the value leaves it.",
      sample: () => within(ROOM, stack("s16", "start", slider(volume()), slider(auto()))),
    },
  ],
  sizes: [
    { name: "Regular", for: "`size.slider`, 28: a panel's or a page's slider, as tall as a compact row.", sample: () => boxed(volume()) },
    { name: "Compact", for: "`size.slider_compact`, 24: a dense panel (the control center, the system monitor), as tall as a chip. Its number shows only lit or held.", sample: () => boxed(volume("Compact")) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "its value is the fill, and a stop it rests on is its anchor, both in every row here; Toggled has its own rows.",
      loading: "it moves at once; a value still on its way leaves the slider where it was.",
      empty: "it always has a value: at 0 the fill is the circle behind the icon. Muted isn't empty: it keeps the level it comes back to.",
      error: "it has no wrong value; a value that failed to apply eases back.",
    },
    rows: nonEmpty(ROWS.map(({ name, options, icon }) => ({ name, cell: (state: MatrixState) => (icon ? iconCell(options, state) : cell(options, state)) }))),
    described: {
      rest: "The fill `primary`, its content `on_primary`; toggled, `secondary` and `on_secondary`, at the level it keeps, the icon the toggled one and the value named. The track `surface_container_highest` at `opacity.track`, its words `on_surface`. A compact slider hides its number. The icon's button shows nothing: the icon on the fill. By day the same roles in daylight colours.",
      lit: "Under the pointer or with the keyboard on it, here with the signs on: the fill brightens by `opacity.press` toward `on_surface`, the tube burns inside the pill's edge, `primary` at `focus.tube`, with the glow outside, and a compact slider shows its number. In the icon's rows the icon's button is lit instead, and the slider isn't: its circle lifts by `opacity.press`, its tube burns inside the circle's edge with the glow outside, and its hint shows what a press does. By day the fill and the circle brighten toward `surface`, and the line is the tube by day with no glow.",
      pressed: "Held, pressed or dragged, so lit as well: the fill brightens by `opacity.held`, plainly, and the glow is `focus.pressed_glow` times the lit one. Letting go of a slider with stops settles it on the nearest. In the icon's rows the icon's button is pressed: its circle lifts by `opacity.held` and its glow is `focus.pressed_glow` times the lit one.",
      disabled: "Every part at `opacity.disabled_content`, where its value puts it, so its value still reads, a toggled one in its softer fill with its name: no line, no glow, no pointer, no keys, and no stop for Tab. Its icon is no button.",
    },
  },
  focus: {
    tubes: [
      { name: "Every slider", tone: "accent", filled: false, content: false },
      { name: "The icon's button", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the pill's edge, over the fill and the track alike, with the glow outside, all that reaches past the pill. The fill brightens by `opacity.press`.",
      signsOff: "The same slider still lit with the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. The tube is solid glass, `primary_glass`, with no glow; the fill keeps its lit brightness, since a fill isn't a light.",
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
      TOGGLES,
      ICON,
      NUMBER,
      EASES,
      SETTLES,
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["←", "h"], does: "A twentieth lower (5%), or one stop lower. Toggled, it leaves Toggled from the level it keeps." },
      { keys: ["→", "l"], does: "A twentieth higher, or one stop higher, leaving Toggled the same way." },
      { keys: ["Home"], does: "To 0, or the first stop." },
      { keys: ["End"], does: "To 1, or the last stop. A boosted volume's end is 150%." },
      {
        keys: ["Space"],
        does: "Presses the icon, where it is a button: it toggles (mutes and unmutes, Auto on and off), or on a slider with stops steps one stop as a press does. While the key is down the icon's button shows pressed and its hint shows, and the slider isn't lit; it acts as the key comes up, never on key repeat. On a slider whose icon is no button it does nothing, and the page has it.",
      },
      { keys: ["Tab", "Shift+Tab"], does: "To the slider, or on to the next or previous control. The icon's button is no stop of its own: the keys move the value." },
    ],
    order: "One stop in the surface's reading order; a disabled one is skipped. The horizontal wheel over it steps as the arrows do, a notch at a time, right raising it: a tilted wheel, or a turn of the wheel with Shift held. The vertical wheel scrolls the page past it, and doesn't move it.",
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
