// The Switch page: `docs/design/components/switch.md`, in the state model (`docs/DESIGN.md`,
// "States"): rest, lit, pressed and disabled, off and on, by night and by day.

import { type Shown, stateOptions } from "../../lib/control.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { type SwitchOptions, switchControl } from "../../lib/switch.ts";
import { text } from "../../lib/text.ts";
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

/** A setting's row: its name, then its switch at the row's end, where every row's lines up. */
function setting(name: string, options: SwitchOptions): HTMLElement {
  return filling(row("s12", "centre", filling(text(name, "body")), switchControl(options)));
}

/** Settings rows one under another: each takes effect at once. */
function settingRows(): HTMLElement {
  return within(
    "var(--size-column)",
    stack("s16", "start", setting("Wi-Fi", { on: true }), setting("Bluetooth", { on: false }), setting("Flight mode", { on: true, disabled: true })),
  );
}

export default page({
  level: "atoms",
  slug: "switch",
  title: "Switch",
  lede: "On or off, taking effect at once: a track with a knob that grows as it crosses.",
  code: { rust: "switch::switch", qml: "Switch" },
  stage: { live: settingRows, hint: "click it, or Tab and Space", daylight: true },
  variants: nonEmpty(
    VALUES.map(({ name, on }) => ({
      name,
      for: on ? "the track filled, the knob at the end" : "the track empty, the knob at the start",
      sample: () => sample(on),
    })),
  ),
  sizes: doesntApply("one size: `size.switch_width` by `size.switch_height`."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "on and off are its value, each a row",
      loading: "what takes a while shows progress beside it",
      empty: "it always has a value",
      error: "a failed setting turns back and says so in its row",
    },
    rows: nonEmpty(VALUES.map(({ name, on }) => ({ name, cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(on, state) }))),
    looks: {
      rest: "off on `surface_container_highest`; on `primary_fill`, unlit",
      lit: "the tube or the core, with its glow; the knob brightens",
      pressed: "glow × `focus.pressed_glow`; tube white-hot, or the fill lifts",
      disabled: "grey, its knob where its value puts it; no Tab stop",
    },
  },
  focus: {
    tubes: [
      { name: "Off", tone: "accent", filled: false, content: false },
      { name: "On", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "on the track; the knob to `on_surface_variant` or `on_primary_fill_lit`",
      signsOff: "`primary_glass` or `primary_fill_glass`, no glow, the knob at rest",
    },
    signsOff: (index: number) => sample(VALUES[index]!.on, "signs-off"),
  },
  content: {
    text: doesntApply("no words: its row names the setting."),
    missingPicture: doesntApply("it draws no picture."),
    counts: doesntApply("one value each; several are rows of a list."),
  },
  motion: {
    arrives: doesntApply("it comes with its surface or row."),
    leaves: doesntApply("it goes with its surface or row."),
    between: [
      {
        name: "Turns on",
        what: "The knob crosses and grows while the track fades to the fill, over `short` on `standard`; off plays backwards.",
        tracks: [{ part: "on", on: "the switch", target: ".k-switch", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => sample(false),
      },
      {
        name: "Focus arrives",
        what: "A fade over `short` on `standard`, never the flicker: line, glow and knob together.",
        tracks: lights(".k-switch", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: both(),
      },
      {
        name: "Focus leaves",
        what: "Back to rest over `short` on `standard`.",
        tracks: lights(".k-switch", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: both("lit"),
      },
      {
        name: "Press",
        what: "Down over `micro` and back as it comes up: the tube runs white-hot or the fill lifts, the glow grows.",
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
      { keys: ["Tab", "Shift+Tab"], does: "to the switch, or on" },
      { keys: ["Space", "Enter"], does: "toggles it at once, never on key repeat" },
      { keys: ["Esc"], does: "nothing of the switch's; it never undoes a toggle" },
    ],
    order: "One stop in reading order; a disabled one is skipped.",
    lands: "Where its surface puts it: a settings page starts on its first control.",
  },
  measures: {
    samples: nonEmpty(VALUES.map(({ name, on }) => ({ name, for: on ? "the knob on, at the end" : "the knob off, at the start", sample: () => sample(on) }))),
    measures: [
      { part: "Track width", token: "size-switch_width" },
      { part: "Track height", token: "size-switch_height" },
      { part: "Knob, off", token: "size-switch_knob_off" },
      { part: "Knob, on", token: "size-switch_knob_on" },
      { part: "Track ends", token: "radius-full" },
      { part: "Outline, off", token: "stroke-hairline_device_px" },
      { part: "Tube, off", token: "focus-tube" },
      { part: "Line, on", token: "stroke-thin" },
      { part: "Near glow", token: "focus-near" },
      { part: "Far glow", token: "focus-far" },
    ],
    reflow: doesntApply("one size at every window size; its row gives it room at the end."),
  },
});
