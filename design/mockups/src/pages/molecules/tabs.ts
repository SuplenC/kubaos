// The Tabs page: `docs/design/components/tabs.md`. One of a few views or values, picked from a row
// of small buttons, the picked one Tonal and the rest Text; the same row is the segmented control.
// Its choices are Button atoms, so their states and moves are the Button page's.

import type { Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack } from "../../lib/layout.ts";
import { type Tab, tabs } from "../../lib/tabs.ts";
import { text, wrapped } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const SECTIONS: readonly Tab[] = [{ label: "Wi-Fi" }, { label: "Bluetooth" }, { label: "Output" }, { label: "Keyboard" }];
const TOOLS: readonly Tab[] = [
  { label: "Timer", icon: "timer" },
  { label: "Alarm", icon: "alarm" },
];
const MOTION: readonly Tab[] = [{ label: "Always" }, { label: "On AC" }, { label: "Never" }];

const LONG: Words = { short: "On AC", long: "Only while plugged in" };

/** A row of `choices`, `picked` lit as `state` would light it. */
function row(choices: readonly Tab[], picked: number, state: Shown = "rest"): HTMLElement {
  const { disabled, pinned } = stateOptions(state);
  return tabs({ choices, picked, ...(disabled ? { disabled } : {}), ...(pinned ? { pinned } : {}) });
}

/** The control center's sections with the view each shows under them, which follows the pick. */
function sections(): HTMLElement {
  const views = ["Networks near you, and the one you're on.", "Devices nearby, and the ones paired.", "Where sound goes, and how loud.", "Your layouts, and the keys they type."];
  const view = wrapped(views[0]!, "body", "on_surface_variant");
  const row = tabs({
    choices: SECTIONS,
    onPick: (index) => {
      view.textContent = views[index] ?? "";
    },
  });
  return stack("s16", "start", row, view);
}

const ROWS = [
  { name: "Sections", choices: SECTIONS, picked: 0 },
  { name: "With symbols", choices: TOOLS, picked: 1 },
  { name: "A segmented control", choices: MOTION, picked: 1 },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

const ON = "the picked choice";

export default page({
  level: "molecules",
  slug: "tabs",
  title: "Tabs",
  summary: "One of a few views or values, picked from a row of small buttons: the picked one tonal, the rest text.",
  live: () => stack("s24", "start", sections(), row(TOOLS, 1), row(MOTION, 1)),
  description: [
    "Rust `tabs::tabs`, QML `Tabs`. Click a choice, or Tab to a row and use ← and →: the view under the sections follows the pick.",
    "A row of small (sm) buttons `space.s4` apart, as tall as `size.control_sm`: the picked one `Tonal`, the rest `Text`, each with an optional symbol before its label. It picks one of a few views (the control center's sections, the clock's tools, the wallpaper picker's folders, the system monitor's order) or one of a setting's few values side by side, as the segmented control (the wallpaper's motion: Always, On AC, Never).",
    "Each choice is a `Button`, with the Button page's states and moves: a choice lights under the pointer as any Text or Tonal button does. The row is one stop for Tab, on the picked choice, so the tube and its glow light the picked one while the row has the keyboard; ← and → pick the neighbour and the keyboard moves with the pick. A pick is at once: the picked choice turns `Tonal` and the one before `Text` on the same frame, the light moving with the keyboard as a fade, and the view it shows changes as its own surface says.",
    "Disabled, every choice is dimmed as a disabled button is, the picked one still tonal, so the setting still reads; the row takes no press and is no stop for Tab.",
  ],
  variants: [
    { name: "Views", for: "One of a few views of one thing: the control center's sections.", sample: () => row(SECTIONS, 0) },
    { name: "With symbols", for: "Choices a symbol helps tell apart: the clock's Timer and Alarm.", sample: () => row(TOOLS, 1) },
    { name: "Segmented control", for: "One of a setting's few values side by side, where radio rows would take a line each.", sample: () => row(MOTION, 1) },
  ],
  sizes: doesntApply("its choices are always small buttons (sm, `size.control_sm`): a row of tabs sits among small controls, and a view's tabs are no louder than the view."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "the picked choice is its Tonal button, in every row here.",
      loading: "a pick is at once; a view still loading shows its own skeleton under the row.",
      empty: "a row always has a pick: a row of one choice isn't tabs.",
      error: "it has no wrong pick.",
    },
    rows: nonEmpty(ROWS.map(({ name, choices, picked }) => ({ name, cell: (state: MatrixState) => row(choices, picked, state) }))),
    described: {
      rest: "The picked choice a resting `Tonal` button, its label dimmed on `secondary_container`; the rest resting `Text` buttons, their labels the accent unlit. By day the container and the accent in daylight colours.",
      lit: "The row with the keyboard on it, here with the signs on: the picked choice lit as a Tonal button is, its label to `on_secondary_container`, the tube inside its edge and the glow outside. A choice under the pointer lights the same way, as its own kind. By day the line with no glow.",
      pressed: "The picked choice pressed: its container lifts 0.035, the glow to `focus.pressed_glow` times the lit one. A Text choice pressed burns its tube white-hot; letting go picks it.",
      disabled: "Every choice dimmed as a disabled button is, the picked one still on its container: no line, no glow, no press, no stop for Tab.",
    },
  },
  focus: {
    tubes: [
      { name: "The picked choice", tone: "accent", filled: false, content: false },
      { name: "A choice under the pointer", tone: "accent", filled: false, content: true },
    ],
    looks: {
      lit: "At night at rest, the signs burn: the picked choice's tube inside its edge with the glow outside, its label brightening; a Text choice under the pointer burns its label as a tube as well.",
      signsOff: "Still lit, the neon unpowered: by day always, and at night while a power change has the lights out. The tube is solid glass, `primary_glass`, no glow, the label at rest.",
    },
    signsOff: (index: number) => row(ROWS[index]!.choices, ROWS[index]!.picked, "signs-off"),
  },
  content: {
    text: [{ name: "A choice's label", words: LONG, sample: (words: string) => row([{ label: "Always" }, { label: words }, { label: "Never" }], 1) }],
    missingPicture: doesntApply("a symbol is optional: a choice without one is its label alone, and a row's choices all have one or none."),
    counts: [
      { name: "Two", for: "The fewest: two views.", sample: () => row(TOOLS, 0) },
      { name: "Four", for: "The control center's sections.", sample: () => row(SECTIONS, 2) },
      {
        name: "Too many for the row",
        for: "Tabs don't wrap or scroll: a row that runs past its room has too many choices for tabs, and its choices are a Select instead.",
        sample: () => stack("s8", "start", row([...SECTIONS, { label: "Display" }, { label: "Power" }], 0), text("Six: past a panel's width, a Select's job.", "small", "on_surface_variant")),
      },
    ],
  },
  motion: {
    arrives: doesntApply("a row of tabs comes with the surface it is on."),
    leaves: doesntApply("it goes with the surface it is on."),
    between: [
      {
        name: "The keyboard comes to the row",
        what: "The row takes the keyboard: the picked choice lights up over `short` on `standard`, as any button does. A pick moving with ← or → is at once: the new choice turns Tonal and the old one Text on the same frame, and the keyboard moving with it lights the new one up over `short` as the old one goes back to rest, as focus moving between controls does.",
        tracks: lights('[aria-selected="true"]', ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: () => row(TOOLS, 1),
      },
      {
        name: "Pressed",
        what: "A lit choice pressed: over `micro`, and back as it comes up; letting go picks it.",
        tracks: press('[aria-selected="true"]', ON),
        reduced: "same",
        back: "plays",
        sample: () => row(TOOLS, 1, "lit"),
      },
      ...powerMoves(() => row(TOOLS, 1, "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["←", "h"], does: "Picks the choice before, round from the first to the last." },
      { keys: ["→", "l"], does: "Picks the choice after, round from the last to the first." },
      { keys: ["Tab", "Shift+Tab"], does: "Into the row, onto the picked choice, or on out of it: the row is one stop." },
      { keys: ["Space", "Enter"], does: "Nothing more: the choice the keyboard is on is already picked." },
    ],
    order: "One stop in the surface's reading order, on the picked choice; a disabled row is skipped.",
    lands: "When a surface opens on its tabs (the control center), the keyboard lands on the picked choice; a pick keeps the keyboard on the row, never moving it into the view.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With symbols", for: "Each button's height, padding, symbol and the gap between them.", sample: () => row(TOOLS, 1) },
      { name: "Sections", for: "The row as wide as its buttons.", sample: () => row(SECTIONS, 0) },
    ]),
    measures: [
      { what: "Each choice's height, a small button's.", token: "size-control_sm" },
      { what: "The gap between choices.", token: "space-s4" },
      { what: "Inside each choice's sides.", token: "space-s12" },
      { what: "The gap between a symbol and its label.", token: "space-s4" },
      { what: "Each choice's label, `button_small`'s line.", token: "type-button_small-line" },
      { what: "Each choice's corners.", token: "radius-control" },
    ],
    reflow: doesntApply("a row of tabs is as wide as its buttons at every window size: it never stretches, wraps or scrolls. A row that would run past its room is a Select (Counts, above)."),
  },
});
