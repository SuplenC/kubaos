// The Button page: `docs/design/components/button.md` is its full reference. Rest, lit, pressed
// and disabled, for six kinds in three sizes, by night and by day.

import { button, filled, KINDS, type Kind, labelGlow, type Pinned, type Size, SIZES, TONES } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The words and symbol each kind shows in the specimens, and how long they get. */
const SAMPLES: Record<Kind, { words: Words; icon: string; trailing: boolean }> = {
  Filled: { words: { short: "Continue", long: "Continue to the disks" }, icon: "arrow_forward", trailing: true },
  Tonal: { words: { short: "Save log", long: "Save the installation log" }, icon: "save", trailing: false },
  Text: { words: { short: "Back", long: "Back to the keyboard layout" }, icon: "arrow_back", trailing: false },
  Danger: { words: { short: "Erase disk", long: "Erase the disk and install" }, icon: "warning", trailing: false },
  Success: { words: { short: "Finish", long: "Finish and restart the computer" }, icon: "check", trailing: false },
  Warning: { words: { short: "Restart now", long: "Restart now and lose the open windows" }, icon: "warning", trailing: false },
};

function sample(kind: Kind, size: Size, state: Pinned | "rest" | "disabled" = "rest"): HTMLButtonElement {
  const { words, icon, trailing } = SAMPLES[kind];
  return button({
    label: shown(words),
    kind,
    size,
    icon,
    trailing,
    disabled: state === "disabled",
    ...(state === "rest" || state === "disabled" ? {} : { pinned: state }),
  });
}

const FOR: Record<Kind, string> = {
  Filled: "the one main action, the next step",
  Tonal: "medium emphasis",
  Text: "minor actions: Back, Skip, Cancel",
  Danger: "destroys something, after a confirmation",
  Success: "completes something good: Finish",
  Warning: "has a cost to weigh: Restart now",
};

const SIZE_FOR: Record<Size, string> = {
  Sm: "chrome round a screen, beside a label",
  Md: "a screen's own actions",
  Lg: "a big page's one main action",
};

/** Every kind side by side at md, at rest: what each move plays on, so each kind's tone shows. */
function kinds(): HTMLElement {
  return row("s16", "start", ...KINDS.map((kind) => sample(kind, "Md")));
}

/** Every kind at md, lit: what a press plays on, and the lights a sequence flickers and strikes. */
function litKinds(): HTMLElement {
  return row("s16", "start", ...KINDS.map((kind) => sample(kind, "Md", "lit")));
}

/** The state matrix's rows: every kind at every size. */
const ROWS = SIZES.flatMap((size) => KINDS.map((kind) => ({ kind, size })));

/** The kinds of a list, as the recipe says them. */
function on(list: readonly Kind[]): string {
  return list.length === KINDS.length ? "every kind" : list.join(", ");
}

export default page({
  level: "atoms",
  slug: "button",
  title: "Button",
  lede: "A labelled action: six kinds by what it is for, three sizes, an icon before or after or none.",
  code: { rust: "button::button(label)", qml: "Button" },
  stage: { live: kinds, hint: "hover it, or Tab to it", daylight: true },
  variants: nonEmpty([
    ...KINDS.map((kind) => ({ name: kind, for: FOR[kind], sample: () => sample(kind, "Md") })),
    { name: "Holding its alternatives' width", for: "Show and Hide one width: it never moves", sample: () => row("s16", "centre", button({ label: "Show", kind: "Text", size: "Sm", alternatives: ["Show", "Hide"], disclosure: false }), button({ label: "Hide", kind: "Text", size: "Sm", alternatives: ["Show", "Hide"], disclosure: true })) },
    { name: "Disclosing", for: "a chevron that turns with what it opens", sample: () => row("s16", "centre", button({ label: "More", kind: "Tonal", size: "Sm", disclosure: false }), button({ label: "More", kind: "Tonal", size: "Sm", disclosure: true })) },
  ]),
  sizes: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => sample("Filled", size) }))),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "it acts, holding no choice",
      loading: "what it starts shows its own progress",
      empty: "it always has its label",
      error: "it has no value to be wrong",
    },
    rows: nonEmpty(
      ROWS.map(({ kind, size }) => ({
        name: `${kind} ${size.toLowerCase()}`,
        cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(kind, size, state),
      })),
    ),
    looks: {
      rest: "unlit at night; the burning kinds burn",
      lit: "the line burns inside its edge, the glow outside",
      pressed: "overdriven: fill brighter, glow × `focus.pressed_glow`",
      disabled: "grey: no tone, line or glow",
    },
  },
  focus: {
    tubes: nonEmpty(KINDS.map((kind) => ({ name: kind, tone: TONES[kind], filled: filled(kind), content: labelGlow(kind) === "content" }))),
    looks: {
      lit: "line and glow in its tone; the label lights",
      signsOff: "the line as unlit glass, no glow",
    },
    signsOff: (index: number) => {
      const { kind, size } = ROWS[index]!;
      return sample(kind, size, "signs-off");
    },
  },
  content: {
    text: KINDS.map((kind) => ({
      name: kind,
      words: SAMPLES[kind].words,
      sample: (words: string) => button({ label: words, kind, size: "Md", icon: SAMPLES[kind].icon, trailing: SAMPLES[kind].trailing }),
    })),
    missingPicture: [{ name: "No icon", for: "the label alone; the same height", sample: () => button({ label: "Authenticate", kind: "Filled", size: "Md" }) }],
    counts: doesntApply("a button holds one label; a row of them is the layout's."),
  },
  motion: {
    arrives: doesntApply("it comes with its surface or page."),
    leaves: doesntApply("it goes with its surface or page."),
    between: [
      {
        name: "Focus arrives",
        what: "A fade over `short` on `standard`, never the flicker: fill, label, line and glow move together.",
        tracks: lights(".k-button", on(KINDS), 0, 1),
        reduced: "same",
        back: "cuts",
        sample: kinds,
      },
      {
        name: "Focus leaves",
        what: "Back to rest over `short` on `standard`.",
        tracks: lights(".k-button", on(KINDS), 1, 0),
        reduced: "same",
        back: "cuts",
        sample: litKinds,
      },
      {
        name: "Press",
        what: "Down over `micro` and back as it comes up: the fill lifts, the glow grows.",
        tracks: press(".k-button", on(KINDS)),
        reduced: "same",
        back: "plays",
        sample: litKinds,
      },
      ...powerMoves(litKinds),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "to the button, or on" },
      { keys: ["Enter"], does: "presses it, never on key repeat" },
      { keys: ["Space"], does: "presses it" },
      { keys: ["Esc"], does: "nothing of the button's" },
    ],
    order: "One stop in reading order; a disabled one is skipped.",
    lands: "Where its surface puts it: a screen's Filled, a Danger dialog's Cancel.",
  },
  measures: {
    samples: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => sample("Filled", size) }))),
    measures: [
      { part: "Height, sm", token: "size-control_sm" },
      { part: "Height, md", token: "size-control_md" },
      { part: "Height, lg", token: "size-control_lg" },
      { part: "Label line, sm", token: "type-button_small-line" },
      { part: "Label line, md", token: "type-button-line" },
      { part: "Label line, lg", token: "type-button_large-line" },
      { part: "Icon, sm", token: "type-button_small-icon" },
      { part: "Icon, md", token: "type-button-icon" },
      { part: "Icon, lg", token: "type-button_large-icon" },
      { part: "Icon → label, sm", token: "space-s4" },
      { part: "Icon → label, md and lg", token: "space-s8" },
      { part: "Sides, sm; Text sm and md", token: "space-s12" },
      { part: "Sides, Text lg", token: "space-s16" },
      { part: "Sides, md", token: "space-s24" },
      { part: "Sides, lg", token: "space-s32" },
      { part: "Corner", token: "radius-control" },
      { part: "Tube, Tonal and Text", token: "focus-tube" },
      { part: "Line, filled kinds", token: "stroke-thin" },
      { part: "Near glow", token: "focus-near" },
      { part: "Far glow", token: "focus-far" },
      { part: "Text label's glow", token: "focus-content_near" },
      { part: "Filled label's glow", token: "focus-label_near" },
    ],
    reflow: doesntApply("as wide as its label and paddings at every size; it never stretches."),
  },
});
