// The IconButton page: `docs/design/components/icon-button.md`, in the state model (`docs/DESIGN.md`,
// "States"): rest, lit, pressed and disabled, for six kinds at three sizes, by night and by day.

import { filled, labelGlow, METRICS, type Size, SIZES, TONES } from "../../lib/button.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { hintRoom } from "../../lib/hint.ts";
import { ICON_KINDS, type IconKind, iconButton } from "../../lib/icon-button.ts";
import { row } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The symbol and the label each kind shows here, the label as long as it gets. */
const SAMPLES: Record<IconKind, { icon: string; words: Words }> = {
  Filled: { icon: "add", words: { short: "New", long: "New snapshot of the home folder" } },
  Tonal: { icon: "settings", words: { short: "Settings", long: "Settings for notifications and sounds" } },
  Text: { icon: "close", words: { short: "Close", long: "Close the panel and go back" } },
  Danger: { icon: "power_settings_new", words: { short: "Shut down", long: "Shut down the computer now" } },
  Success: { icon: "check", words: { short: "Finish", long: "Finish and restart the computer" } },
  Warning: { icon: "restart_alt", words: { short: "Restart now", long: "Restart now and lose the open windows" } },
};

/** An icon button of `kind` and `size` in `state`; its hint drawn unless `hint` is false. */
function sample(kind: IconKind, size: Size, state: Shown = "rest", hint = true): HTMLButtonElement {
  const { icon, words } = SAMPLES[kind];
  return iconButton({ icon, label: shown(words), kind, size, hint, ...stateOptions(state) });
}

/** `control`, an icon button of `size` labelled `words`, with the room its hint floats in. */
function roomy(control: HTMLButtonElement, size: Size): HTMLElement {
  return hintRoom(control, control.getAttribute("aria-label") ?? "", `var(--size-${METRICS[size].height})`);
}

const FOR: Record<IconKind, string> = {
  Filled: "a toolbar's or card's main action: Add, New",
  Tonal: "a quieter action beside a main one: Settings",
  Text: "chrome and minor actions: Close, Back",
  Danger: "destroys something, after a confirmation: Shut down",
  Success: "completes something good: Finish",
  Warning: "has a cost to weigh: Restart now",
};

const SIZE_FOR: Record<Size, string> = {
  Sm: "chrome in a dense row, beside a field",
  Md: "a panel's or a card's actions",
  Lg: "beside a big page's lg button",
};

/** Every kind at md, at rest: what each move plays on. */
function kinds(): HTMLElement {
  return row("s16", "start", ...ICON_KINDS.map((kind) => sample(kind, "Md", "rest", false)));
}

/** Every kind at md, lit: what a press plays on, and the lights a sequence flickers and strikes. */
function litKinds(): HTMLElement {
  return row("s16", "start", ...ICON_KINDS.map((kind) => sample(kind, "Md", "lit", false)));
}

const ON = "every kind";

/** The state matrix's rows: every kind at every size. */
const ROWS = SIZES.flatMap((size) => ICON_KINDS.map((kind) => ({ kind, size })));

export default page({
  level: "atoms",
  slug: "icon-button",
  title: "IconButton",
  lede: "A circle with a symbol and a required label, shown in its hint, in every button kind.",
  code: { rust: "button::icon_button(glyph, label)", qml: "IconButton" },
  stage: {
    live: () => row("s16", "start", ...ICON_KINDS.map((kind) => roomy(sample(kind, "Md"), "Md"))),
    hint: "hover it, or Tab to it",
    daylight: true,
  },
  variants: nonEmpty(ICON_KINDS.map((kind) => ({ name: kind, for: FOR[kind], sample: () => roomy(sample(kind, "Md"), "Md") }))),
  sizes: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => roomy(sample("Filled", size), size) }))),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "a choice that stays on is a `Switch`, `Chip` or `Tile`",
      loading: "what it starts shows its own progress",
      empty: "it always has its symbol and label",
      error: "what failed shows where its result would",
    },
    rows: nonEmpty(
      ROWS.map(({ kind, size }) => ({
        name: `${kind} ${size.toLowerCase()}`,
        cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(kind, size, state, false),
      })),
    ),
    looks: {
      rest: "unlit at night; the burning kinds burn",
      lit: "the line burns inside its edge, the glow outside; its hint shows",
      pressed: "overdriven: fill brighter, glow × `focus.pressed_glow`",
      disabled: "grey: no tone, line or glow; the hint still shows",
    },
  },
  focus: {
    tubes: nonEmpty(ICON_KINDS.map((kind) => ({ name: kind, tone: TONES[kind], filled: filled(kind), content: labelGlow(kind) === "content" }))),
    looks: {
      lit: "line and glow in its tone; the symbol lights like a label",
      signsOff: "the line as unlit glass, no glow; the hint shows either way",
    },
    signsOff: (index: number) => {
      const { kind, size } = ROWS[index]!;
      return sample(kind, size, "signs-off", false);
    },
  },
  content: {
    text: ICON_KINDS.map((kind) => ({
      name: `${kind}: its label, in its hint`,
      words: SAMPLES[kind].words,
      sample: (words: string) => roomy(iconButton({ icon: SAMPLES[kind].icon, label: words, kind, size: "Md", pinned: "lit" }), "Md"),
    })),
    missingPicture: [
      {
        name: "No picture to miss",
        for: "a named KubaOS symbol; the types require the label",
        sample: () => roomy(iconButton({ icon: "more_time", label: shown(fixed("Add five minutes")), kind: "Tonal", size: "Md", pinned: "lit" }), "Md"),
      },
    ],
    counts: doesntApply("one symbol each; several are a row of the layout."),
  },
  motion: {
    arrives: doesntApply("it comes with its surface or page."),
    leaves: doesntApply("it goes with its surface or page."),
    between: [
      {
        name: "Focus arrives",
        what: "A fade over `short` on `standard`, never the flicker: fill, symbol, line and glow move together, the hint with them.",
        tracks: lights(".k-button", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: kinds,
      },
      {
        name: "Focus leaves",
        what: "Back to rest over `short` on `standard`, the hint with it.",
        tracks: lights(".k-button", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: litKinds,
      },
      {
        name: "Press",
        what: "Down over `micro` and back as it comes up: the fill lifts or the tube runs white-hot, the glow grows.",
        tracks: press(".k-button", ON),
        reduced: "same",
        back: "plays",
        sample: litKinds,
      },
      ...powerMoves(litKinds),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "to the icon button, or on; its hint shows" },
      { keys: ["Enter", "Space"], does: "presses it, never on key repeat" },
      { keys: ["Esc"], does: "nothing of the icon button's; never presses a burning one" },
    ],
    order: "One stop in reading order; a disabled one is skipped.",
    lands: "Where its surface puts it, never on a Danger, Success or Warning one.",
  },
  measures: {
    samples: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => sample("Filled", size, "rest", false) }))),
    measures: [
      { part: "Side, sm", token: "size-control_sm" },
      { part: "Side, md", token: "size-control_md" },
      { part: "Side, lg", token: "size-control_lg" },
      { part: "Symbol, sm", token: "type-button_small-icon" },
      { part: "Symbol, md", token: "type-button-icon" },
      { part: "Symbol, lg", token: "type-button_large-icon" },
      { part: "Circle", token: "radius-full" },
      { part: "Tube, Tonal and Text", token: "focus-tube" },
      { part: "Line, filled kinds", token: "stroke-thin" },
      { part: "Near glow", token: "focus-near" },
      { part: "Far glow", token: "focus-far" },
      { part: "Text symbol's glow", token: "focus-content_near" },
      { part: "Filled symbol's glow", token: "focus-label_near" },
      { part: "Gap to its hint", token: "space-s8" },
    ],
    reflow: doesntApply("its size's side at every window size; it never stretches."),
  },
});
