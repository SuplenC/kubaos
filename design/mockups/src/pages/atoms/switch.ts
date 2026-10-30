// The Switch page: `docs/design/components/switch.md`, in the state model (`docs/DESIGN.md`,
// "States"): rest, lit, pressed and disabled, off and on, by night and by day.

import { type Shown, stateOptions } from "../../lib/control.ts";
import { row } from "../../lib/layout.ts";
import { switchControl } from "../../lib/switch.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** A switch, on or off, in `state`. */
function sample(on: boolean, state: Shown = "rest"): HTMLButtonElement {
  return switchControl({ on, ...stateOptions(state) });
}

/** Off and on side by side in `state`: what each move plays on. */
function both(state: Shown = "rest"): () => HTMLElement {
  return () => row("s16", "start", sample(false, state), sample(true, state));
}

const ON = "off and on";

const VALUES = [
  { name: "Off", on: false },
  { name: "On", on: true },
] as const;

export default page({
  level: "atoms",
  slug: "switch",
  title: "Switch",
  summary: "On or off, taking effect at once: a track with a knob that grows as it moves across. Its states follow the neon, off and on.",
  live: () => sample(true),
  description: [
    "A switch turns one setting on or off, and it takes effect at once: no Save follows it. A click, Space or Enter toggles it. It has no words of its own: the row it sits in names the setting (a settings row, a `ListRow`), and a press anywhere on that row is the row's to give it.",
    "Its track is `size.switch_width` by `size.switch_height`, a circle at each end (`radius.full`). Off, the track is the highest container (`surface_container_highest`) with a hairline of `outline` inside its edge, and the knob, `size.switch_knob_off` across, of `outline`, sits at the start. On, the track is filled with the accent and the knob, `size.switch_knob_on` across, sits at the end: it leaves 6 round itself off and 4 on. Toggled, the knob moves and grows over `short` on `standard`, and the track's colour, its outline and the knob's colour fade with it.",
    "Its states follow the neon (`docs/DESIGN.md`, \"States\"): rest, lit (hovered or focused, one state, with its two looks), pressed and disabled, each off and on. At night it rests unlit: on, the track is the unlit fill (`primary_fill`, the accent at 60% of its lightness and 1.2 times its chroma), as a Filled button's, with a near-white knob (`on_primary_fill`). Lit, the line inside its edge burns with the glow outside it: off, the tube over its outline (`primary` at `focus.tube`), and the knob brightens to `on_surface_variant`; on, the core inside the fill's edge (`primary_fill_line`, `stroke.thin` wide), and the knob brightens to `on_primary_fill_lit`. Nothing but the glow reaches past the track.",
    "Disabled, it can't be changed: grey, with no tone, no line and no glow, the track at `opacity.disabled_container` of `on_surface` with no outline and the knob at `opacity.disabled_content` of it, where its value puts it, so its value still reads.",
  ],
  variants: nonEmpty(
    VALUES.map(({ name, on }) => ({
      name,
      for: on ? "The setting is on: the track filled with the accent, the knob at the end." : "The setting is off: the track the highest container with its outline, the knob at the start.",
      sample: () => sample(on),
    })),
  ),
  sizes: doesntApply("a switch has one size, `size.switch_width` by `size.switch_height`, in every row; a row of small controls is the same switch."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "on and off are its value, and each is a row here, so every state shows in both.",
      loading: "it takes effect at once; a setting that takes a while shows its progress beside it, never in the switch.",
      empty: "it always has a value, on or off.",
      error: "it has no wrong value; a setting that failed to apply turns back and says so in the row.",
    },
    rows: nonEmpty(VALUES.map(({ name, on }) => ({ name, cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(on, state) }))),
    described: {
      rest: "Unlit at night. Off: the track `surface_container_highest` with an `outline` hairline inside its edge, the knob `outline`. On: the track the unlit fill, `primary_fill`, the knob `on_primary_fill`, near white tinted with the tone. By day: off is the same roles in daylight colours; on, the track is the accent lifted to lightness 0.80, as a Filled button's day fill, with a dark knob (`on_primary_fill`).",
      lit: "Hovered or focused, here with the signs on. Off: the tube, `primary` at `focus.tube`, over the outline inside the track's edge, with the glow outside it, and the knob brightens to `on_surface_variant`. On: the core inside the fill's edge, `primary_fill_line`, `stroke.thin` wide, with the glow outside it, and the knob brightens to `on_primary_fill_lit`; the fill stays unlit. By day lit is its signs-off look in daylight colours: no glow, the knob at its rest colour, off the tube as glass over the outline, on a dark line inside the fill's edge (`primary_fill_line` by day).",
      pressed: "Overdriven, under the pointer or Space and Enter, so lit as well; the glow `focus.pressed_glow` times the lit one. Off: the tube runs white-hot, to the core, as a Text button's; by day the tone tints the track at `opacity.press_tint`. On: the fill lifts 0.035 (`primary_fill_pressed`), 0.04 darker by day. Letting go toggles it.",
      disabled: "Grey, with no tone, no line and no glow: the track `on_surface` at `opacity.disabled_container` with no outline, the knob `on_surface` at `opacity.disabled_content`, where its value puts it. No action, no focus.",
    },
  },
  focus: {
    tubes: [
      { name: "Off", tone: "accent", filled: false, content: false },
      { name: "On", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "At night at rest, the signs burn: off, the tube over the track's outline; on, the core inside the fill's edge; the glow outside either, all that reaches past the track. The knob brightens: `on_surface_variant` off, `on_primary_fill_lit` on.",
      signsOff: "The same switch still hovered or focused with the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. Off, the tube is solid unlit glass, `primary_glass`, as a Tonal or Text button's; on, the line inside the fill's edge is `primary_fill_glass`, the unlit fill lifted 0.12. No glow, and the knob at its rest colour.",
    },
    signsOff: (index: number) => sample(VALUES[index]!.on, "signs-off"),
  },
  content: {
    text: doesntApply("a switch has no words: the row it sits in names the setting."),
    missingPicture: doesntApply("a switch draws no picture."),
    counts: doesntApply("a switch holds one value; switches one under another are rows of a list."),
  },
  motion: {
    arrives: doesntApply("a switch comes with the surface or the row it is on."),
    leaves: doesntApply("a switch goes with the surface or the row it is on."),
    between: [
      {
        name: "Turns on",
        what: "Toggled from off to on: `--checked` from 0 to 1 over `short` on `standard`. The knob moves from the start to the end and grows from `size.switch_knob_off` to `size.switch_knob_on`; the track fades from the highest container to the accent's fill, its outline fades out, and the knob's colour fades from `outline` to `on_primary_fill`. Lit, the line fades from the tube to the core with it. Turning off plays it backwards.",
        tracks: [{ part: "on", on: "the switch", target: ".k-switch", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => sample(false),
      },
      {
        name: "Focus arrives",
        what: "Hovered or focused, it lights up: a fade over `short` on `standard`, never the flicker. The line and the glow fade in and the knob brightens as `--lit` goes from 0 to 1.",
        tracks: lights(".k-switch", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: both(),
      },
      {
        name: "Focus leaves",
        what: "The pointer leaves or focus moves on: it fades back to rest over `short` on `standard`.",
        tracks: lights(".k-switch", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: both("lit"),
      },
      {
        name: "Press",
        what: "A lit switch goes down over `micro`, and back as it comes up: off, its tube runs white-hot; on, its fill lifts 0.035; either way its glow grows to `focus.pressed_glow` times the lit one.",
        tracks: press(".k-switch", ON),
        reduced: "same",
        back: "plays",
        sample: both("lit"),
      },
      ...powerMoves(both("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Focus to the switch, or on to the next or previous control, in reading order." },
      { keys: ["Space", "Enter"], does: "Toggles the focused switch, once per press, never on key repeat. It takes effect at once." },
      { keys: ["Esc"], does: "Nothing of the switch's: it goes back one step, as the surface says, and never undoes a toggle." },
    ],
    order: "One stop in the surface's reading order; a disabled one is skipped. Focus moving to it or away never moves anything: the line and the glow take no room.",
    lands: "It takes focus when the surface gives it; a settings page starts on its first control.",
  },
  measures: {
    samples: nonEmpty(VALUES.map(({ name, on }) => ({ name, for: on ? "The knob on, at the end." : "The knob off, at the start.", sample: () => sample(on) }))),
    measures: [
      { what: "The track's width.", token: "size-switch_width" },
      { what: "The track's height, and the circles at its ends.", token: "size-switch_height" },
      { what: "The knob off, leaving 6 round it.", token: "size-switch_knob_off" },
      { what: "The knob on, leaving 4 round it.", token: "size-switch_knob_on" },
      { what: "The track's ends.", token: "radius-full" },
      { what: "The outline inside the track's edge while off: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "The lit tube while off, rounded half up to device pixels.", token: "focus-tube" },
      { what: "The lit line inside the fill while on, rounded half up.", token: "stroke-thin" },
      { what: "The near glow's reach.", token: "focus-near" },
      { what: "The far glow's reach.", token: "focus-far" },
    ],
    reflow: doesntApply("a switch is its one size at every window size; its row gives it room at the row's end."),
  },
});
