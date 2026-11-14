// The Checkbox page: `docs/design/components/checkbox.md`, in the state model (`docs/DESIGN.md`,
// "States"): rest, lit, pressed and disabled, empty, partial and full, by night and by day.

import { checkbox, type Tick, TICKS } from "../../lib/check.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { row, stack } from "../../lib/layout.ts";
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
  Empty: "not picked",
  Partial: "some of a group picked: Select all",
  Full: "picked",
};

/** A short list as a form has it: all of it picked, none and a part. */
function form(): HTMLElement {
  return row(
    "s32",
    "start",
    checkbox({ label: "Select all", tick: "Partial" }),
    checkbox({ label: "Wi-Fi", tick: "Full" }),
    checkbox({ label: "Bluetooth", tick: "Empty" }),
    checkbox({ label: "VPN", tick: "Full", disabled: true }),
  );
}

export default page({
  level: "atoms",
  slug: "checkbox",
  title: "Checkbox",
  lede: "A square and its label, one press target, for independent picks that apply together.",
  code: { rust: "check::checkbox", qml: "Checkbox" },
  stage: { live: form, hint: "click it, or Tab and Space", daylight: true },
  variants: nonEmpty(TICKS.map((tick) => ({ name: tick.toLowerCase(), for: FOR[tick], sample: () => sample(tick) }))),
  sizes: doesntApply("one size: its box `size.check`, its label `body`."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "empty, partial and full are its value, each a row",
      loading: "picks apply with the form, never while it waits",
      empty: "empty is a value, its first row",
      error: "a group that needs a pick says so in a `Note`",
    },
    rows: nonEmpty(TICKS.map((tick) => ({ name: tick.toLowerCase(), cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(tick, state) }))),
    looks: {
      rest: "empty on `surface_container_highest`; picked on `primary_fill`, unlit",
      lit: "the tube, or the core in a fill, with the glow outside",
      pressed: "glow × `focus.pressed_glow`; tube white-hot, or the fill lifts",
      disabled: "grey: no tone, line or glow, and no Tab stop",
    },
  },
  focus: {
    tubes: [
      { name: "empty", tone: "accent", filled: false, content: false },
      { name: "partial", tone: "accent", filled: true, content: false },
      { name: "full", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "on the box; the mark to `on_primary_fill_lit`, the label stays",
      signsOff: "`primary_glass` or `primary_fill_glass`, no glow, the mark at rest",
    },
    signsOff: (index: number) => sample(TICKS[index]!, "signs-off"),
  },
  content: {
    text: [{ name: "Its label", words: WORDS, sample: (words: string) => sample("Full", "rest", words) }],
    missingPicture: doesntApply("its tick is a KubaOS symbol and its dash is drawn."),
    counts: doesntApply("one pick each; several are a group of the layout."),
  },
  motion: {
    arrives: doesntApply("it comes with its surface or row."),
    leaves: doesntApply("it goes with its surface or row."),
    between: [
      {
        name: "Picked",
        what: "The fill and tick fade in as the outline goes, over `short` on `standard`; nothing moves or grows.",
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
        what: "A fade over `short` on `standard`, never the flicker: line, glow and a picked mark together.",
        tracks: lights(".k-check", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: ticks(),
      },
      {
        name: "Focus leaves",
        what: "Back to rest over `short` on `standard`.",
        tracks: lights(".k-check", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: ticks("lit"),
      },
      {
        name: "Press",
        what: "Down over `micro` and back as it comes up: the tube runs white-hot or the fill lifts, the glow grows.",
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
      { keys: ["Tab", "Shift+Tab"], does: "to the checkbox, or on; each is its own stop" },
      { keys: ["Space", "Enter"], does: "toggles it, never on key repeat" },
      { keys: ["Esc"], does: "nothing of the checkbox's" },
    ],
    order: "One stop in reading order; a disabled one is skipped.",
    lands: "Where its surface puts it: a form starts on its first control.",
  },
  measures: {
    samples: nonEmpty([
      { name: "full", for: "box, tick and label", sample: () => sample("Full") },
      { name: "partial", for: "the dash", sample: () => sample("Partial") },
    ]),
    measures: [
      { part: "Box", token: "size-check" },
      { part: "Corner", token: "radius-small" },
      { part: "Tick", token: "icon-s16" },
      { part: "Dash inset", token: "space-s4" },
      { part: "Dash; line in a picked fill", token: "stroke-thin" },
      { part: "Box → label", token: "space-s12" },
      { part: "Label line", token: "type-body-line" },
      { part: "Empty box's outline", token: "stroke-hairline_device_px" },
      { part: "Tube on an empty box", token: "focus-tube" },
      { part: "Near glow", token: "focus-near" },
      { part: "Far glow", token: "focus-far" },
    ],
    reflow: doesntApply("as wide as its box, gap and label; a long label is its row's to give room or elide."),
  },
});
