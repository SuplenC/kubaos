// The Radio page: `docs/design/components/radio.md`, in the state model (`docs/DESIGN.md`,
// "States"): rest, lit, pressed and disabled, picked and not, by night and by day.

import { radio, radioGroup } from "../../lib/check.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const WORDS: Words = { short: "Dark", long: "Follow the time of day, dark after sunset" };

/** A radio, picked or not, in `state`. */
function sample(checked: boolean, state: Shown = "rest", words: string = shown(WORDS)): HTMLButtonElement {
  return radio({ label: words, checked, ...stateOptions(state) });
}

/** Unpicked and picked one under another in `state`: what each move plays on. */
function both(state: Shown = "rest"): () => HTMLElement {
  return () => stack("s12", "start", sample(false, state), sample(true, state));
}

const ON = "picked and not";

const VALUES = [
  { name: "Not picked", checked: false },
  { name: "Picked", checked: true },
] as const;

/** A live group of three, the middle one picked: picking another unpicks it. */
function group(): HTMLElement {
  const choices = ["Light", "Dark", "Follow the time of day"];
  return radioGroup("s24", ...choices.map((words, index) => radio({ label: shown(fixed(words)), checked: index === 1 })));
}

export default page({
  level: "atoms",
  slug: "radio",
  title: "Radio",
  summary: "A disc and its label, one press target: one of a group, exactly one picked. Its states follow the neon.",
  live: group,
  description: [
    "A radio picks one of a group, exactly one picked: light, dark or the time of day. Picking one unpicks the one that was. A group of two that means on and off is a `Switch`; picks that apply together are `Checkbox`es. A click on the disc or its label, Space or Enter picks it.",
    "Its disc is `size.check` across (`radius.full`), then its label in `body`, `on_surface`, `space.s12` from it, centred down the label's line, the whole row one press target. Not picked, the disc is the highest container (`surface_container_highest`) with a hairline of `outline` inside its edge. Picked, it is filled with the accent and holds a dot, `size.radio_dot` across, centred on a whole device pixel.",
    "Its states follow the neon (`docs/DESIGN.md`, \"States\"), with the line on the disc: rest, lit (hovered or focused, one state, over the disc or its label), pressed and disabled. At night it rests unlit: a picked disc is the unlit fill (`primary_fill`), as a Filled button's, with its dot near white (`on_primary_fill`). Lit, the line inside the disc's edge burns with the glow outside it: not picked, the tube over its outline (`primary` at `focus.tube`); picked, the core inside the fill's edge (`primary_fill_line`, `stroke.thin` wide), and the dot brightens to `on_primary_fill_lit`. The label doesn't light. Nothing but the glow reaches past the disc.",
    "Picking one fades over `short`: its fill and dot come in as its outline goes, while the one it unpicks fades back. Disabled, it can't be picked: grey, with no tone, no line and no glow, the disc at `opacity.disabled_container` of `on_surface` with no outline, its dot and its label at `opacity.disabled_content` of it.",
  ],
  variants: nonEmpty(
    VALUES.map(({ name, checked }) => ({
      name,
      for: checked ? "The one picked in its group: the disc filled with the accent, with its dot." : "Every other one of its group: the disc the highest container with its outline.",
      sample: () => sample(checked),
    })),
  ),
  sizes: doesntApply("a radio has one size, its disc `size.check` and its label `body`, in every row."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "picked and not are its value, and each is a row here, so every state shows in both.",
      loading: "picking takes effect at once, or when its form is applied, never while it waits.",
      empty: "a group always has one picked; a list with no choices is the list's empty state.",
      error: "it has no wrong value: one of its group is always picked.",
    },
    rows: nonEmpty(VALUES.map(({ name, checked }) => ({ name, cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(checked, state) }))),
    described: {
      rest: "Unlit at night. Not picked: the disc `surface_container_highest` with an `outline` hairline inside its edge. Picked: the disc the unlit fill, `primary_fill`, the dot `on_primary_fill`. The label `on_surface`. By day: not picked is the same roles in daylight colours; picked, the disc is the accent lifted to lightness 0.80, as a Filled button's day fill, with a dark dot.",
      lit: "Hovered or focused, over the disc or its label, here with the signs on. Not picked: the tube, `primary` at `focus.tube`, over the outline inside the disc's edge, with the glow outside it. Picked: the core inside the fill's edge, `primary_fill_line`, `stroke.thin` wide, with the glow outside it, and the dot brightens to `on_primary_fill_lit`. The label doesn't change. By day lit is its signs-off look in daylight colours: no glow, the tube as glass over the outline, a dark line inside a picked disc's edge.",
      pressed: "Overdriven, under the pointer or Space and Enter, so lit as well; the glow `focus.pressed_glow` times the lit one. Not picked: the tube runs white-hot, to the core; by day the tone tints the disc at `opacity.press_tint`. Picked: the fill lifts 0.035 (`primary_fill_pressed`), 0.04 darker by day. Letting go picks it.",
      disabled: "Grey, with no tone, no line and no glow: the disc `on_surface` at `opacity.disabled_container` with no outline, its dot and its label `on_surface` at `opacity.disabled_content`. No action, no focus.",
    },
  },
  focus: {
    tubes: [
      { name: "Not picked", tone: "accent", filled: false, content: false },
      { name: "Picked", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "At night at rest, the signs burn on the disc: not picked, the tube over its outline; picked, the core inside the fill's edge; the glow outside either, all that reaches past the disc. A picked dot brightens to `on_primary_fill_lit`; the label stays.",
      signsOff: "The same radio still hovered or focused with the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. Not picked, the tube is unlit glass, `primary_unlit` at `focus.glass` over the disc; picked, the line inside the fill's edge is `primary_fill_glass`. No glow, and the dot at its rest colour.",
    },
    signsOff: (index: number) => sample(VALUES[index]!.checked, "signs-off"),
  },
  content: {
    text: [{ name: "Its label", words: WORDS, sample: (words: string) => sample(true, "rest", words) }],
    missingPicture: doesntApply("its dot is drawn, never a picture."),
    counts: [
      { name: "A group", for: "Two or more, exactly one picked; picking another here unpicks the one that was. A group of one is a `Checkbox`, and a group of two meaning on and off is a `Switch`.", sample: group },
    ],
  },
  motion: {
    arrives: doesntApply("a radio comes with the surface or the row it is on."),
    leaves: doesntApply("a radio goes with the surface or the row it is on."),
    between: [
      {
        name: "Picked",
        what: "Picked: `--checked` from 0 to 1 over `short` on `standard`. The disc fades from the highest container to the accent's fill, its outline fades out and the dot fades in where it sits; nothing moves or grows. The one it unpicks plays the same backwards, at the same time.",
        tracks: [{ part: "picked", on: "the radio", target: ".k-check", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => sample(false),
      },
      {
        name: "Focus arrives",
        what: "Hovered or focused, it lights up: a fade over `short` on `standard`, never the flicker. The disc's line and glow fade in and a picked dot brightens as `--lit` goes from 0 to 1.",
        tracks: lights(".k-check", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: both(),
      },
      {
        name: "Focus leaves",
        what: "The pointer leaves or focus moves on: it fades back to rest over `short` on `standard`.",
        tracks: lights(".k-check", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: both("lit"),
      },
      {
        name: "Press",
        what: "A lit radio goes down over `micro`, and back as it comes up: not picked, its tube runs white-hot; picked, its fill lifts 0.035; either way its glow grows to `focus.pressed_glow` times the lit one.",
        tracks: press(".k-check", ON),
        reduced: "same",
        back: "plays",
        sample: both("lit"),
      },
      ...powerMoves(both("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Focus to the radio, or on to the next or previous control: each radio of a group is its own stop, in reading order." },
      { keys: ["Space", "Enter"], does: "Picks the focused radio, unpicking the one that was, once per press, never on key repeat." },
      { keys: ["Esc"], does: "Nothing of the radio's: it goes back one step, as the surface says." },
    ],
    order: "One stop per radio in the surface's reading order; a disabled one is skipped. Focus moving to it or away never moves anything: the line and the glow take no room.",
    lands: "It takes focus when the surface gives it: a form starts on its first control.",
  },
  measures: {
    samples: nonEmpty([{ name: "Picked", for: "The disc with its dot, and its label.", sample: () => sample(true) }]),
    measures: [
      { what: "The disc's side.", token: "size-check" },
      { what: "The disc.", token: "radius-full" },
      { what: "The dot.", token: "size-radio_dot" },
      { what: "The gap between the disc and its label.", token: "space-s12" },
      { what: "The label's line, which the disc is centred down.", token: "type-body-line" },
      { what: "The outline inside an unpicked disc's edge: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "The lit tube on an unpicked disc, rounded half up.", token: "focus-tube" },
      { what: "The lit line inside a picked disc's fill, rounded half up.", token: "stroke-thin" },
      { what: "The near glow's reach.", token: "focus-near" },
      { what: "The far glow's reach.", token: "focus-far" },
    ],
    reflow: doesntApply("a radio is as wide as its disc, gap and label at every window size; its group's layout lays the radios out."),
  },
});
