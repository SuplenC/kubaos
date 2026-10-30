// The Chip page: `docs/design/components/chip.md`, in the state model (`docs/DESIGN.md`, "States"):
// a plain chip has no states; a pressable one rests, lights, is pressed and disabled, in five kinds,
// by night and by day, and so does its close button.

import { CHIP_KINDS, CHIP_TONES, type ChipKind, type ChipOptions, chip, chipFilled } from "../../lib/chip.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { hintRoom } from "../../lib/hint.ts";
import { row, stack } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The words and the symbol each kind shows here, as long as they get. */
const SAMPLES: Record<ChipKind, { words: Words; icon: string }> = {
  Neutral: { words: { short: "US", long: "English (US, international)" }, icon: "keyboard" },
  Selected: { words: { short: "Mac accents", long: "Mac accents on the Option key" }, icon: "keyboard_option_key" },
  Accent: { words: { short: "Default", long: "Recommended for this computer" }, icon: "star" },
  Cold: { words: { short: "Demo", long: "Preview of the next release" }, icon: "info" },
  Filled: { words: { short: "Reboot", long: "Reboot to finish the update" }, icon: "restart_alt" },
};

/** A chip of `kind`, with `options` beside its words. */
function sample(kind: ChipKind, options: Omit<ChipOptions, "label" | "kind"> = {}, words: string = shown(SAMPLES[kind].words)): HTMLElement {
  return chip({ label: words, kind, ...options });
}

/** A pressable chip of `kind` in `state`. */
function pressable(kind: ChipKind, state: Shown = "rest"): HTMLElement {
  return sample(kind, { press: "press", ...stateOptions(state) });
}

/** A pressable, removable chip of `kind`, its close button in `state`. */
function closing(kind: ChipKind, state: Shown): HTMLElement {
  const { pinned, disabled } = stateOptions(state);
  return sample(kind, { press: "press", removable: true, ...(disabled ? { disabled } : {}), ...(pinned ? { pinnedClose: pinned } : {}) });
}

/** Every kind, pressable, side by side in `state`: what each move plays on. */
function kinds(state: Shown = "rest"): () => HTMLElement {
  return () => row("s8", "start", ...CHIP_KINDS.map((kind) => pressable(kind, state)));
}

const ON = "every kind";

const FOR: Record<ChipKind, string> = {
  Neutral: "A fact: a picked keyboard layout, a file's type. `surface_container_high` with an `on_surface` label.",
  Selected: "One of several picks that is on: a setting a toolbar turns on (`Neutral` while off). The accent's quieter sibling, as a Tonal button: `secondary_container`, its label dimmed at night at rest (`on_secondary_container_rest`) and full lit.",
  Accent: "A tag that matters: Recommended, Default. The accent at `opacity.tint` under a `primary` label.",
  Cold: "Rarely: a demo, a preview. The cold accent at `opacity.tint` under a `cold` label; it lights in the cold accent.",
  Filled: "An action among states: Reboot beside Done and To do. Filled with the accent, so at night it rests on the unlit fill (`primary_fill`) with a near-white label (`on_primary_fill`), as a Filled button.",
};

/** The state matrix's rows: every kind pressable, then the close button on a Neutral and a Filled chip. */
const ROWS: readonly { name: string; cell: (state: Shown) => HTMLElement }[] = [
  ...CHIP_KINDS.map((kind) => ({ name: kind, cell: (state: Shown) => pressable(kind, state) })),
  { name: "Neutral, its close", cell: (state: Shown) => closing("Neutral", state) },
  { name: "Filled, its close", cell: (state: Shown) => closing("Filled", state) },
];

export default page({
  level: "atoms",
  slug: "chip",
  title: "Chip",
  summary: "A short fact or pick in a pill: five kinds, an optional icon and close button. A plain chip has no states; a pressable one follows the neon.",
  live: () => row("s8", "start", sample("Neutral", { icon: "keyboard", removable: true }), sample("Selected", { press: "toggle", on: true }), sample("Accent"), sample("Filled", { press: "press" })),
  description: [
    "A chip is a short fact or pick: a keyboard layout, a tag, a state. It is `size.chip` tall at the control radius, which on a chip is a pill, its label `label` type. An optional icon, `label`'s icon, sits `space.s4` before the label; an optional close button ends it, a circle as tall as the chip with its `close` symbol `space.s4` in, whose curve is the chip's own. Its left side pads `space.s12`, or `space.s8` with an icon; its right `space.s12`, or nothing when the close button ends it. Its content is centred down it.",
    "A plain chip is a fact and has no states: it is what its kind's colours are at rest. A pressable one takes a press of its own (a pick made the default, a setting turned on), and in the shell the keyboard too; one that toggles a setting is `Selected` while on and `Neutral` while off, fading between the two over `short`. In QML a pressable chip may be no stop for Tab (`tabStop: false`) where the field or the row beside the chips is the stop and a click picks one: the clock's timer presets. A pressable chip's `tooltip` names it in a `Hint` under it while it is lit.",
    "A pressable chip's states follow the neon (`docs/DESIGN.md`, \"States\"): rest, lit (hovered or focused, one state, with its two looks), pressed and disabled. Lit, the line inside its edge burns with the glow outside it, in its kind's tone: the tube, `focus.tube` wide, on Neutral, Selected and Accent in the accent and on Cold in the cold accent; the core inside the fill's edge, `stroke.thin` wide, on Filled. Selected's label goes from dimmed to full and Filled's brightens, as a Tonal and a Filled button's do; the others' labels don't change. Nothing but the glow reaches past the pill. Lighting up, going back to rest and pressing are fades; the flicker is for power changes only.",
    "Its close button is a control of its own: it lights as a Text icon button does, the tube inside its circle in the chip's tone (the core on a Filled chip) with the glow outside it, and with the pointer over it the close button lights alone, so it is plain which press is under the pointer. Disabled, neither the chip's press nor its close button takes the pointer or the keyboard: the chip goes grey, `on_surface` at `opacity.disabled_container` under its content at `opacity.disabled_content`, with no tone, no line and no glow.",
  ],
  variants: nonEmpty([
    ...CHIP_KINDS.map((kind) => ({ name: kind, for: FOR[kind], sample: () => sample(kind) })),
    { name: "With an icon", for: "A symbol before the label, `space.s4` from it; the left side pads `space.s8` instead of `space.s12`, as the icon is rounder than a letter.", sample: () => sample("Neutral", { icon: SAMPLES.Neutral.icon }) },
    { name: "Removable", for: "A close button ends it, and its right side has no padding: the circle's curve is the chip's own. Any kind can be removable; the close sends `removed`.", sample: () => sample("Neutral", { removable: true }) },
    {
      name: "Pressable, with a tooltip",
      for: "A press of its own, and its name in a hint under it while it is lit.",
      sample: () => hintRoom(sample("Selected", { press: "press", tooltip: "Five minutes" }, shown(fixed("5 min"))), "Five minutes", "var(--size-chip)"),
    },
  ]),
  sizes: doesntApply("a chip has one size, `size.chip` tall with `label` type, in every row."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "being picked is a kind of its own, `Selected`, and a row here; a chip that toggles fades between `Neutral` and `Selected` (Motion).",
      loading: "a chip states a fact that is there; what is still on its way shows as a `Skeleton` in its place.",
      empty: "a chip always has its label; a list of chips with none is the list's empty state.",
      error: "a chip has no value to be wrong; a fact that is an error is a `Badge` in the error tone, or a `Note`.",
    },
    rows: nonEmpty(ROWS.map(({ name, cell }) => ({ name, cell: (state: "rest" | "lit" | "pressed" | "disabled") => cell(state) }))),
    described: {
      rest: "Its kind's colours. Neutral `surface_container_high` under `on_surface`; Selected `secondary_container` under its dimmed label at night (`on_secondary_container_rest`); Accent the accent at `opacity.tint` under `primary`; Cold the cold accent at `opacity.tint` under `cold`; Filled at night the unlit fill, `primary_fill`, under `on_primary_fill`. By day: the same roles in daylight colours, Filled the accent lifted to lightness 0.80 with a dark label. A plain chip is always this.",
      lit: "Hovered or focused, here with the signs on: the line inside the pill's edge burns with the glow outside it, the tube in the accent on Neutral, Selected and Accent and in the cold accent on Cold, the core (`primary_fill_line`, `stroke.thin` wide) on Filled. Selected's label goes full (`on_secondary_container`), Filled's brightens (`on_primary_fill_lit`). On the close rows the close button is lit: the tube inside its circle, which shares the pill's curve, or the core on Filled. By day lit is its signs-off look in daylight colours: no glow, the line as glass.",
      pressed: "Overdriven, under the pointer or Space and Enter, so lit as well; the glow `focus.pressed_glow` times the lit one. Neutral, Accent and Cold: the tube runs white-hot, to the core; by day their tone tints the pill at `opacity.press_tint`. Selected: the container lifts 0.035 (`secondary_container_pressed`), 0.04 darker by day. Filled: the fill lifts 0.035 (`primary_fill_pressed`), 0.04 darker by day. The close button's tube runs white-hot, its tone a tint under it by day.",
      disabled: "Grey, with no tone, no line and no glow: the pill `on_surface` at `opacity.disabled_container`, its label, icon and close at `opacity.disabled_content`. Neither its press nor its close takes the pointer or the keyboard.",
    },
  },
  focus: {
    tubes: nonEmpty(CHIP_KINDS.map((kind) => ({ name: kind, tone: CHIP_TONES[kind], filled: chipFilled(kind), content: false }))),
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the pill's edge, or the core inside a Filled chip's fill, with the glow outside it, all that reaches past the pill; Selected's and Filled's labels light. A lit close button draws its own line inside its circle, the pill's curve at its end.",
      signsOff: "The same chip still hovered or focused with the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. The tube is unlit glass, its tone's `_unlit` at `focus.glass` over the pill, as a field's; a Filled chip's line is `primary_fill_glass`, the unlit fill lifted 0.12. No glow, and the label at its rest colour.",
    },
    signsOff: (index: number) => ROWS[index]!.cell("signs-off"),
  },
  content: {
    text: CHIP_KINDS.map((kind) => ({ name: kind, words: SAMPLES[kind].words, sample: (words: string) => sample(kind, { icon: SAMPLES[kind].icon }, words) })),
    missingPicture: [
      { name: "No icon", for: "The icon is optional: without one the label starts `space.s12` in, where the curve ends, and the chip keeps its height. A named symbol is never missing.", sample: () => sample("Accent") },
    ],
    counts: [
      { name: "One", for: "A chip on its own: a tag beside a title.", sample: () => sample("Accent") },
      {
        name: "Many",
        for: "Chips side by side in a row of the layout; each is its own stop for Tab when pressable, and a removed one closes the gap.",
        sample: () => row("s8", "start", ...["US", "Polish", "German (no dead keys)"].map((words) => sample("Neutral", { removable: true }, shown(fixed(words))))),
      },
    ],
  },
  motion: {
    arrives: doesntApply("a chip comes with the surface or the row it is on."),
    leaves: doesntApply("a chip goes with the surface or the row it is on; a removed one leaves with its row's layout."),
    between: [
      {
        name: "Turns on",
        what: "A chip that toggles a setting, pressed from off to on: `--checked` from 0 to 1 over `short` on `standard`, its fill and label fading from Neutral's to Selected's. Nothing moves or grows. Turning off plays it backwards.",
        tracks: [{ part: "on", on: "a toggling chip", target: ".k-chip", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => sample("Selected", { press: "toggle", on: false }),
      },
      {
        name: "Focus arrives",
        what: "Hovered or focused, a pressable chip lights up: a fade over `short` on `standard`, never the flicker. Its line and glow fade in and Selected's and Filled's labels light as `--lit` goes from 0 to 1.",
        tracks: lights(".k-chip", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: kinds(),
      },
      {
        name: "Focus leaves",
        what: "The pointer leaves or focus moves on: it fades back to rest over `short` on `standard`.",
        tracks: lights(".k-chip", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: kinds("lit"),
      },
      {
        name: "Press",
        what: "A lit chip goes down over `micro`, and back as it comes up: Neutral's, Accent's and Cold's tube runs white-hot, Selected's container and Filled's fill lift 0.035, and every glow grows to `focus.pressed_glow` times the lit one.",
        tracks: press(".k-chip", ON),
        reduced: "same",
        back: "plays",
        sample: kinds("lit"),
      },
      ...powerMoves(() => stack("s12", "start", kinds("lit")(), row("s8", "start", sample("Accent"), sample("Cold")))),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Focus to a pressable chip, then to its close button, or on to the next or previous control, in reading order. A plain chip, and a chip with `tabStop` off, is no stop." },
      { keys: ["Enter", "Space"], does: "Presses the focused chip (or turns it on or off), or its focused close button, which removes it." },
      { keys: ["Esc"], does: "Nothing of the chip's: it goes back one step, as the surface says." },
    ],
    order: "A pressable chip is one stop and its close button another, after it; a disabled chip is neither. Where focus goes after a chip is removed is its surface's to say. Focus moving never moves anything: the line and the glow take no room.",
    lands: "A chip takes focus when the surface gives it; a row of chips is never where a surface starts.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Plain", for: "Its label between the paddings.", sample: () => sample("Neutral") },
      { name: "With an icon, removable", for: "The icon's padding, the gaps and the close button.", sample: () => sample("Neutral", { icon: SAMPLES.Neutral.icon, removable: true }) },
    ]),
    measures: [
      { what: "Its height, and the close button's side.", token: "size-chip" },
      { what: "The pill: the control radius, half its height.", token: "radius-control" },
      { what: "Its label's line, centred down it.", token: "type-label-line" },
      { what: "The icon and the close symbol.", token: "type-label-icon" },
      { what: "The padding at its sides without an icon.", token: "space-s12" },
      { what: "The padding before an icon.", token: "space-s8" },
      { what: "The gaps between icon, label and close, and the close symbol's inset.", token: "space-s4" },
      { what: "The lit tube, rounded half up to device pixels.", token: "focus-tube" },
      { what: "The lit line inside a Filled chip's fill, rounded half up.", token: "stroke-thin" },
      { what: "The near glow's reach.", token: "focus-near" },
      { what: "The far glow's reach.", token: "focus-far" },
    ],
    reflow: doesntApply("a chip is as wide as its content and paddings at every window size: it never stretches or wraps, and the layout it is in gives it room."),
  },
});
