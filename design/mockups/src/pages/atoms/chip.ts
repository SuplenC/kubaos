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
  Neutral: "a fact: a layout, a file's type",
  Selected: "a pick that is on; `Neutral` while off",
  Accent: "a tag that matters: Recommended, Default",
  Cold: "rarely: a demo, a preview",
  Filled: "an action among states: Reboot beside Done",
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
  lede: "A short fact or pick in a pill: five kinds, an optional icon and close button.",
  code: { rust: "chip::chip", qml: "Chip" },
  stage: {
    live: () => row("s8", "start", sample("Neutral", { press: "press", icon: "keyboard", removable: true }), sample("Selected", { press: "toggle", on: true }), sample("Accent", { press: "press" }), sample("Cold", { press: "press" }), sample("Filled", { press: "press" })),
    hint: "hover it, or Tab; the second toggles",
    daylight: true,
  },
  variants: nonEmpty([
    ...CHIP_KINDS.map((kind) => ({ name: kind, for: FOR[kind], sample: () => sample(kind) })),
    { name: "With an icon", for: "the left side pads `space.s8`, not `space.s12`", sample: () => sample("Neutral", { icon: SAMPLES.Neutral.icon }) },
    { name: "Removable", for: "any kind; the close's curve is the chip's own", sample: () => sample("Neutral", { removable: true }) },
    {
      name: "Pressable, with a tooltip",
      for: "its name in a hint while it is lit",
      sample: () => hintRoom(sample("Selected", { press: "press", tooltip: "Five minutes" }, shown(fixed("5 min"))), "Five minutes", "var(--size-chip)"),
    },
  ]),
  sizes: doesntApply("one size: `size.chip` tall, `label` type."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "`Selected` is a kind, a row here",
      loading: "what is on its way is a `Skeleton`",
      empty: "it always has its label",
      error: "an error is a `Badge` or a `Note`",
    },
    rows: nonEmpty(ROWS.map(({ name, cell }) => ({ name, cell: (state: "rest" | "lit" | "pressed" | "disabled") => cell(state) }))),
    looks: {
      rest: "its kind's colours; a plain chip is always this",
      lit: "the tube, or Filled's core, with the glow outside",
      pressed: "glow × `focus.pressed_glow`; tube white-hot, or the fill lifts",
      disabled: "grey: no tone, line or glow, and no press",
    },
  },
  focus: {
    tubes: nonEmpty(CHIP_KINDS.map((kind) => ({ name: kind, tone: CHIP_TONES[kind], filled: chipFilled(kind), content: false }))),
    looks: {
      lit: "inside the pill's edge; Filled's label lights; a lit close lights alone",
      signsOff: "the tone's `_glass`, or `primary_fill_glass`; no glow, label at rest",
    },
    signsOff: (index: number) => ROWS[index]!.cell("signs-off"),
  },
  content: {
    text: CHIP_KINDS.map((kind) => ({ name: kind, words: SAMPLES[kind].words, sample: (words: string) => sample(kind, { icon: SAMPLES[kind].icon }, words) })),
    missingPicture: [{ name: "No icon", for: "the label starts `space.s12` in; the same height", sample: () => sample("Accent") }],
    counts: [
      { name: "One", for: "a tag beside a title", sample: () => sample("Accent") },
      {
        name: "Many",
        for: "a row of the layout; a removed one closes the gap",
        sample: () => row("s8", "start", ...["US", "Polish", "German (no dead keys)"].map((words) => sample("Neutral", { removable: true }, shown(fixed(words))))),
      },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface or row."),
    leaves: doesntApply("it goes with its surface or row; a removed one with its row's layout."),
    between: [
      {
        name: "Turns on",
        what: "A toggling chip fades from Neutral's colours to Selected's over `short` on `standard`; nothing moves.",
        tracks: [{ part: "on", on: "a toggling chip", target: ".k-chip", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => sample("Selected", { press: "toggle", on: false }),
      },
      {
        name: "Focus arrives",
        what: "A fade over `short` on `standard`, never the flicker: line, glow and Filled's label together.",
        tracks: lights(".k-chip", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: kinds(),
      },
      {
        name: "Focus leaves",
        what: "Back to rest over `short` on `standard`.",
        tracks: lights(".k-chip", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: kinds("lit"),
      },
      {
        name: "Press",
        what: "Down over `micro` and back as it comes up: the tube runs white-hot or the fill lifts, the glow grows.",
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
      { keys: ["Tab", "Shift+Tab"], does: "to a pressable chip, then its close, or on" },
      { keys: ["Enter", "Space"], does: "presses or toggles it; on its close, removes it" },
      { keys: ["Esc"], does: "nothing of the chip's" },
    ],
    order: "A pressable chip and its close are two stops; a plain, disabled or `tabStop: false` chip is none.",
    lands: "Where its surface puts it; a row of chips is never where a surface starts.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Plain", for: "label and paddings", sample: () => sample("Neutral") },
      { name: "With an icon, removable", for: "icon padding, gaps, close", sample: () => sample("Neutral", { icon: SAMPLES.Neutral.icon, removable: true }) },
    ]),
    measures: [
      { part: "Height, close's side", token: "size-chip" },
      { part: "Corner", token: "radius-control" },
      { part: "Label line", token: "type-label-line" },
      { part: "Icon, close symbol", token: "type-label-icon" },
      { part: "Sides, no icon", token: "space-s12" },
      { part: "Before an icon", token: "space-s8" },
      { part: "Gaps, close's inset", token: "space-s4" },
      { part: "Tube", token: "focus-tube" },
      { part: "Line, Filled", token: "stroke-thin" },
      { part: "Near glow", token: "focus-near" },
      { part: "Far glow", token: "focus-far" },
    ],
    reflow: doesntApply("as wide as its content: it never stretches or wraps."),
  },
});
