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
  lede: "A disc and its label, one press target: one of a group, exactly one picked.",
  code: { rust: "check::radio", qml: "Radio" },
  stage: { live: group, hint: "click one, or Tab and Space", daylight: true },
  variants: nonEmpty(
    VALUES.map(({ name, checked }) => ({
      name,
      for: checked ? "the one picked in its group" : "every other one of its group",
      sample: () => sample(checked),
    })),
  ),
  sizes: doesntApply("one size: its disc `size.check`, its label `body`."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "picked and not are its value, each a row",
      loading: "picking takes effect at once or with its form",
      empty: "a group always has one picked",
      error: "one of its group is always picked",
    },
    rows: nonEmpty(VALUES.map(({ name, checked }) => ({ name, cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(checked, state) }))),
    looks: {
      rest: "not picked on `surface_container_highest`; picked on `primary_fill`, unlit",
      lit: "the tube, or the core in a fill, with the glow outside",
      pressed: "glow × `focus.pressed_glow`; tube white-hot, or the fill lifts",
      disabled: "grey: no tone, line or glow, and no Tab stop",
    },
  },
  focus: {
    tubes: [
      { name: "Not picked", tone: "accent", filled: false, content: false },
      { name: "Picked", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "on the disc; the dot to `on_primary_fill_lit`, the label stays",
      signsOff: "`primary_glass` or `primary_fill_glass`, no glow, the dot at rest",
    },
    signsOff: (index: number) => sample(VALUES[index]!.checked, "signs-off"),
  },
  content: {
    text: [{ name: "Its label", words: WORDS, sample: (words: string) => sample(true, "rest", words) }],
    missingPicture: doesntApply("its dot is drawn."),
    counts: [{ name: "A group", for: "two or more; on and off is a `Switch`", sample: group }],
  },
  motion: {
    arrives: doesntApply("it comes with its surface or row."),
    leaves: doesntApply("it goes with its surface or row."),
    between: [
      {
        name: "Picked",
        what: "The fill and dot fade in as the outline goes, over `short` on `standard`; the one it unpicks plays backwards.",
        tracks: [{ part: "picked", on: "the radio", target: ".k-check", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => sample(false),
      },
      {
        name: "Focus arrives",
        what: "A fade over `short` on `standard`, never the flicker: line, glow and a picked dot together.",
        tracks: lights(".k-check", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: both(),
      },
      {
        name: "Focus leaves",
        what: "Back to rest over `short` on `standard`.",
        tracks: lights(".k-check", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: both("lit"),
      },
      {
        name: "Press",
        what: "Down over `micro` and back as it comes up: the tube runs white-hot or the fill lifts, the glow grows.",
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
      { keys: ["Tab", "Shift+Tab"], does: "to the radio, or on; each is its own stop" },
      { keys: ["Space", "Enter"], does: "picks it, unpicking the other, never on key repeat" },
      { keys: ["Esc"], does: "nothing of the radio's" },
    ],
    order: "One stop per radio in reading order; a disabled one is skipped.",
    lands: "Where its surface puts it: a form starts on its first control.",
  },
  measures: {
    samples: nonEmpty([{ name: "Picked", for: "disc, dot and label", sample: () => sample(true) }]),
    measures: [
      { part: "Disc", token: "size-check" },
      { part: "Corner", token: "radius-full" },
      { part: "Dot", token: "size-radio_dot" },
      { part: "Disc → label", token: "space-s12" },
      { part: "Label line", token: "type-body-line" },
      { part: "Unpicked outline", token: "stroke-hairline_device_px" },
      { part: "Tube, not picked", token: "focus-tube" },
      { part: "Line, picked", token: "stroke-thin" },
      { part: "Near glow", token: "focus-near" },
      { part: "Far glow", token: "focus-far" },
    ],
    reflow: doesntApply("as wide as its disc, gap and label; its group's layout lays them out."),
  },
});
