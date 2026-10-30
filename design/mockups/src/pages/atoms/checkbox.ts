// The Checkbox page: `docs/design/components/checkbox.md`, in the state model (`docs/DESIGN.md`,
// "States"): rest, lit, pressed and disabled, empty, partial and full, by night and by day.

import { checkbox, type Tick, TICKS } from "../../lib/check.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const WORDS: Words = { short: "Wi-Fi", long: "Connect to known networks automatically" };

/** A checkbox showing `tick` in `state`. */
function sample(tick: Tick, state: Shown = "rest", words: string = shown(WORDS)): HTMLButtonElement {
  return checkbox({ label: words, tick, ...stateOptions(state) });
}

/** Every tick one under another in `state`: what each move plays on. */
function ticks(state: Shown = "rest"): () => HTMLElement {
  return () => stack("s12", "start", ...TICKS.map((tick) => sample(tick, state)));
}

const ON = "empty, partial and full";

const FOR: Record<Tick, string> = {
  Empty: "Not picked: the box the highest container with its outline.",
  Partial: "Some of a group picked: a checkbox standing for several others (Select all, with some selected) shows a dash, the box filled as a full one.",
  Full: "Picked: the box filled with the accent, with a tick.",
};

export default page({
  level: "atoms",
  slug: "checkbox",
  title: "Checkbox",
  summary: "A square and its label, one press target, for several independent picks that apply together: empty, partial or full. Its states follow the neon.",
  live: () => sample("Full"),
  description: [
    "A checkbox picks one of several independent things that apply together, usually with a button that applies them: the networks to forget, the files to copy. A setting that takes effect at once is a `Switch`; one of a group, exactly one picked, is a `Radio`. A click on the box or its label, Space or Enter toggles it.",
    "Its box is `size.check` square at the small radius (`radius.small`), then its label in `body`, `on_surface`, `space.s12` from it, centred down the label's line, the whole row one press target. Empty, the box is the highest container (`surface_container_highest`) with a hairline of `outline` inside its edge. Full, it is filled with the accent and holds a tick, `icon.s16`, centred on a whole device pixel. Partial, for a box standing for a group with some of it picked, it holds a dash instead: the box less `space.s4` at each side wide and the thin stroke tall, on whole device pixels.",
    "Its states follow the neon (`docs/DESIGN.md`, \"States\"), with the line on the box: rest, lit (hovered or focused, one state, over the box or its label), pressed and disabled. At night it rests unlit: a picked box is the unlit fill (`primary_fill`), as a Filled button's, with its mark near white (`on_primary_fill`). Lit, the line inside the box's edge burns with the glow outside it: empty, the tube over its outline (`primary` at `focus.tube`); picked, the core inside the fill's edge (`primary_fill_line`, `stroke.thin` wide), and the mark brightens to `on_primary_fill_lit`. The label doesn't light: it names the box, and stays `on_surface`. Nothing but the glow reaches past the box.",
    "Picking it or clearing it fades over `short`: the fill and the mark come in as the outline goes. Disabled, it can't be changed: grey, with no tone, no line and no glow, the box at `opacity.disabled_container` of `on_surface` with no outline, its mark and its label at `opacity.disabled_content` of it.",
  ],
  variants: nonEmpty(TICKS.map((tick) => ({ name: tick.toLowerCase(), for: FOR[tick], sample: () => sample(tick) }))),
  sizes: doesntApply("a checkbox has one size, its box `size.check` and its label `body`, in every row."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "empty, partial and full are its value, and each is a row here, so every state shows in each.",
      loading: "picking takes effect when its form is applied, never while it waits.",
      empty: "an empty checkbox is a value, its first row; a list with no checkboxes is the list's empty state.",
      error: "it has no wrong value of its own; a group that needs one picked says so below the group, as a `Note`.",
    },
    rows: nonEmpty(TICKS.map((tick) => ({ name: tick.toLowerCase(), cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(tick, state) }))),
    described: {
      rest: "Unlit at night. Empty: the box `surface_container_highest` with an `outline` hairline inside its edge. Partial and full: the box the unlit fill, `primary_fill`, the dash or tick `on_primary_fill`. The label `on_surface`. By day: empty is the same roles in daylight colours; picked, the box is the accent lifted to lightness 0.80, as a Filled button's day fill, with a dark mark (`on_primary_fill`).",
      lit: "Hovered or focused, over the box or its label, here with the signs on. Empty: the tube, `primary` at `focus.tube`, over the outline inside the box's edge, with the glow outside it. Picked: the core inside the fill's edge, `primary_fill_line`, `stroke.thin` wide, with the glow outside it, and the mark brightens to `on_primary_fill_lit`. The label doesn't change. By day lit is its signs-off look in daylight colours: no glow, the tube as glass over the outline, a dark line inside a picked box's edge.",
      pressed: "Overdriven, under the pointer or Space and Enter, so lit as well; the glow `focus.pressed_glow` times the lit one. Empty: the tube runs white-hot, to the core, as a Text button's; by day the tone tints the box at `opacity.press_tint`. Picked: the fill lifts 0.035 (`primary_fill_pressed`), 0.04 darker by day. Letting go toggles it.",
      disabled: "Grey, with no tone, no line and no glow: the box `on_surface` at `opacity.disabled_container` with no outline, its mark and its label `on_surface` at `opacity.disabled_content`. No action, no focus.",
    },
  },
  focus: {
    tubes: [
      { name: "empty", tone: "accent", filled: false, content: false },
      { name: "partial", tone: "accent", filled: true, content: false },
      { name: "full", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "At night at rest, the signs burn on the box: empty, the tube over its outline; picked, the core inside the fill's edge; the glow outside either, all that reaches past the box. A picked mark brightens to `on_primary_fill_lit`; the label stays.",
      signsOff: "The same checkbox still hovered or focused with the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. Empty, the tube is solid unlit glass, `primary_glass`, as a Tonal or Text button's; picked, the line inside the fill's edge is `primary_fill_glass`, the unlit fill lifted 0.12. No glow, and the mark at its rest colour.",
    },
    signsOff: (index: number) => sample(TICKS[index]!, "signs-off"),
  },
  content: {
    text: [{ name: "Its label", words: WORDS, sample: (words: string) => sample("Full", "rest", words) }],
    missingPicture: doesntApply("its tick is a symbol of KubaOS Symbols and its dash is drawn, so neither is ever missing."),
    counts: doesntApply("a checkbox holds one pick; checkboxes one under another are a group of the layout, each its own stop."),
  },
  motion: {
    arrives: doesntApply("a checkbox comes with the surface or the row it is on."),
    leaves: doesntApply("a checkbox goes with the surface or the row it is on."),
    between: [
      {
        name: "Picked",
        what: "Toggled from empty to full: `--checked` from 0 to 1 over `short` on `standard`. The box fades from the highest container to the accent's fill, its outline fades out and the tick fades in where it sits; nothing moves or grows. Lit, the line fades from the tube to the core with it. Cleared plays it backwards; a partial box fills and empties the same way, and partial to full swaps the dash for the tick at once.",
        tracks: [{ part: "picked", on: "the checkbox", target: ".k-check", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => {
          const control = sample("Full");
          control.style.setProperty("--checked", "0");
          return control;
        },
      },
      {
        name: "Focus arrives",
        what: "Hovered or focused, it lights up: a fade over `short` on `standard`, never the flicker. The box's line and glow fade in and a picked mark brightens as `--lit` goes from 0 to 1.",
        tracks: lights(".k-check", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: ticks(),
      },
      {
        name: "Focus leaves",
        what: "The pointer leaves or focus moves on: it fades back to rest over `short` on `standard`.",
        tracks: lights(".k-check", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: ticks("lit"),
      },
      {
        name: "Press",
        what: "A lit checkbox goes down over `micro`, and back as it comes up: empty, its tube runs white-hot; picked, its fill lifts 0.035; either way its glow grows to `focus.pressed_glow` times the lit one.",
        tracks: press(".k-check", ON),
        reduced: "same",
        back: "plays",
        sample: ticks("lit"),
      },
      ...powerMoves(ticks("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Focus to the checkbox, or on to the next or previous control: each checkbox of a group is its own stop, in reading order." },
      { keys: ["Space", "Enter"], does: "Toggles the focused checkbox, once per press, never on key repeat: empty or partial to full, full to empty." },
      { keys: ["Esc"], does: "Nothing of the checkbox's: it goes back one step, as the surface says." },
    ],
    order: "One stop in the surface's reading order; a disabled one is skipped. Focus moving to it or away never moves anything: the line and the glow take no room.",
    lands: "It takes focus when the surface gives it: a form starts on its first control.",
  },
  measures: {
    samples: nonEmpty([
      { name: "full", for: "The box with its tick, and its label.", sample: () => sample("Full") },
      { name: "partial", for: "The dash.", sample: () => sample("Partial") },
    ]),
    measures: [
      { what: "The box's side.", token: "size-check" },
      { what: "The box's corners.", token: "radius-small" },
      { what: "The tick.", token: "icon-s16" },
      { what: "The dash's inset at each side.", token: "space-s4" },
      { what: "The dash's thickness, rounded half up to device pixels.", token: "stroke-thin" },
      { what: "The gap between the box and its label.", token: "space-s12" },
      { what: "The label's line, which the box is centred down.", token: "type-body-line" },
      { what: "The outline inside an empty box's edge: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "The lit tube on an empty box, rounded half up.", token: "focus-tube" },
      { what: "The lit line inside a picked box's fill, rounded half up.", token: "stroke-thin" },
      { what: "The near glow's reach.", token: "focus-near" },
      { what: "The far glow's reach.", token: "focus-far" },
    ],
    reflow: doesntApply("a checkbox is as wide as its box, gap and label at every window size; a label longer than its row has room for is the row's to give room or elide."),
  },
});
