// The Tabs page: `docs/design/components/tabs.md`. One of a few views or values, picked along a
// rail of neon in one well; the same well is the segmented control. The pick's lit stretch travels
// to the new choice and flashes as it lands.

import type { Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack } from "../../lib/layout.ts";
import { type Tab, tabs } from "../../lib/tabs.ts";
import { text, wrapped } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, type Track } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";

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
  { name: "Views", choices: SECTIONS, picked: 0 },
  { name: "With symbols", choices: TOOLS, picked: 1 },
  { name: "Segmented control", choices: MOTION, picked: 1 },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

const STRETCH = "the lit stretch";
const NEW = "the choice picked";
const OLD = "the choice it leaves";

/** The pick's tracks: the stretch's two edges, its flash, and the words of the two choices. */
function picking(reduced: boolean): NonEmpty<Track<Easing>> {
  const edge = (part: string, property: string, travel: "short" | "medium", cue?: "micro"): Track<Easing> =>
    reduced
      ? { part, on: STRETCH, target: ".k-tabs-lit", property, from: 0, to: 1, duration: "instant", curve: "linear" }
      : { part, on: STRETCH, target: ".k-tabs-lit", property, from: 0, to: 1, duration: travel, ...(cue ? { cue } : {}), curve: "emphasized_decel" };
  return [
    edge("leading edge", "--lead", "short"),
    edge("trailing edge", "--trail", "medium", "micro"),
    { part: "flash", on: STRETCH, target: ".k-tabs-lit", property: "--flash", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: "flash", on: STRETCH, target: ".k-tabs-lit", property: "--flash", from: 1, to: 0, duration: reduced ? "short" : "long", cue: "micro", curve: "standard" },
    { part: "words", on: NEW, target: '.k-tab[aria-selected="true"]', property: "--picked", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "words", on: OLD, target: ".k-tab[data-was]", property: "--picked", from: 1, to: 0, duration: "short", curve: "standard" },
  ];
}

/** A pick from choice `from` to `to` of the sections, as a move. */
function pickMove(name: string, what: string, from: number, to: number): Move {
  return {
    name,
    what,
    tracks: picking(false),
    reduced: picking(true),
    back: "cuts",
    // The player moves parts inside its sample, so the well sits in a box of its own.
    sample: () => stack(0, "start", tabs({ choices: SECTIONS, picked: to, from })),
  };
}

const PICK_WHAT =
  "The lit stretch travels from the old choice to the new one and stretches as it goes: its leading edge, the one toward the new choice, gets there over `short` on `emphasized_decel` (`--lead`, 0 to 1), and its trailing edge follows a `micro` later over `medium` on the same curve (`--trail`), catching up. As the pick switches its core flashes: up to `primary_flash` over `micro` on `standard` (`--flash`, 0 to 1), at night most of the way to white with its glow swelling to `focus.pressed_glow` times its own, by day a brighter tint of the tone; then back to its colour over `long`. The words cross-fade over `short` on `standard` (`--picked`), the new choice's to the picked colour as the old one's go back. A pick made while the stretch is still travelling starts from where it is. With reduced motion the stretch is on the new choice at once, and the flash and the words still fade, the flash settling over `short`.";

export default page({
  level: "molecules",
  slug: "tabs",
  title: "Tabs",
  summary: "One of a few views or values, picked along a rail of neon in one well: the pick's stretch of the rail lit, its words the accent.",
  live: () => stack("s24", "start", sections(), row(TOOLS, 1), row(MOTION, 1)),
  description: [
    "Rust `tabs::tabs`, QML `Tabs`. Click a choice, or Tab to a row and use ← and →: the view under the sections follows the pick.",
    "The choices sit side by side in one well: a pill (`radius.full`) in `surface_container_highest` at `opacity.track`, as a slider's track is, `size.control_sm` tall with `space.s4` round the choices. Each choice is as tall as the well less that padding, `space.s12` inside its sides, its words `button_small` with an optional symbol `space.s4` before them in the words' colour. There is no fill behind the picked one. It picks one of a few views (the control center's sections, the clock's tools, the wallpaper picker's folders, the system monitor's order) or one of a setting's few values side by side, as the segmented control (the wallpaper's motion: Always, On AC, Never).",
    "An unlit rail of neon runs straight along the well's bottom edge, on the edge with nothing below it, across the flat part between the round ends: the tone at the spinner's ring's `opacity.spinner_track`. The picked choice's stretch of it is lit, under its words (inset `space.s12` from its sides), square at both ends: at night the core with the content glow round it, as the spinner's arc; by day the tone, with no glow. It is `tabs.rail` thick at night and `tabs.rail_day` by day, whole device pixels. The picked words are `primary_lit`, the core at night with the label glow round them and the tone by day, with no change of weight, so nothing shifts; the rest are `on_surface_variant`.",
    "A choice under the pointer brightens its words toward `on_surface`; pressed, its words go to the picked colour, and letting go picks it. The row is one stop for Tab, on the picked choice, and while it has the keyboard the focus tube lights the whole well's edge with its glow; the lit stretch already says which choice is picked. ← and → pick the neighbour, and the keyboard moves with the pick.",
    "Disabled, every part is at `opacity.disabled_content` with no glow, the pick still shown, so the setting still reads; the row takes no press and is no stop for Tab.",
  ],
  variants: [
    { name: "Views", for: "One of a few views of one thing: the control center's sections.", sample: () => row(SECTIONS, 0) },
    { name: "With symbols", for: "Choices a symbol helps tell apart: the clock's Timer and Alarm.", sample: () => row(TOOLS, 1) },
    { name: "Segmented control", for: "One of a setting's few values side by side, where radio rows would take a line each.", sample: () => row(MOTION, 1) },
  ],
  sizes: doesntApply("the well is always a small control's height (`size.control_sm`): a row of tabs sits among small controls, and a view's tabs are no louder than the view."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "the pick is the lit stretch and the accent words, in every row here.",
      loading: "a pick is at once; a view still loading shows its own skeleton under the row.",
      empty: "a row always has a pick: a row of one choice isn't tabs.",
      error: "it has no wrong pick.",
    },
    rows: nonEmpty(ROWS.map(({ name, choices, picked }) => ({ name, cell: (state: MatrixState) => row(choices, picked, state) }))),
    described: {
      rest: "The well on its surface, the unlit rail along its bottom edge and the pick's stretch of it lit: at night the core with its glow, the picked words the core with the label glow, the rest `on_surface_variant`. By day the stretch and the picked words are the tone, with no glow, and the stretch a pixel thicker.",
      lit: "The row with the keyboard on it, here with the signs on: the focus tube burns round the whole well's edge with the glow outside, under the rail, and the pick is as at rest. A choice under the pointer lights only its words, from `on_surface_variant` to `on_surface`; the rail stays as it is. By day the tube with no glow.",
      pressed: "The choice after the pick held under the pointer: its words go to the picked colour over `micro`, before letting go picks it and the stretch travels there. The well doesn't light: a press is the pointer's.",
      disabled: "Every part at `opacity.disabled_content` (the well, the rail, the lit stretch and the words), with no glow, the pick still shown: no press, no stop for Tab.",
    },
  },
  focus: {
    tubes: [{ name: "The well, with the keyboard on the row", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the focus tube round the well's edge with the glow outside it, the lit stretch the core with its glow and the picked words with theirs.",
      signsOff: "Still lit, the neon unpowered: by day always, and at night while a power change has the lights out. The tube round the well and the lit stretch are solid glass, `primary_glass`, with no glow; the picked words keep `primary_lit`, so the pick still reads, with no glow round them.",
    },
    signsOff: (index: number) => row(ROWS[index]!.choices, ROWS[index]!.picked, "signs-off"),
  },
  content: {
    text: [{ name: "A choice's words", words: LONG, sample: (words: string) => row([{ label: "Always" }, { label: words }, { label: "Never" }], 1) }],
    missingPicture: doesntApply("a symbol is optional: a choice without one is its words alone, and a row's choices all have one or none."),
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
      pickMove("Picking, to the right", `${PICK_WHAT} Here from Wi-Fi to Output: the right edge leads.`, 0, 2),
      pickMove("Picking, to the left", `${PICK_WHAT} Here from Keyboard back to Wi-Fi, the longest way: the left edge leads.`, 3, 0),
      {
        name: "The keyboard comes to the row",
        what: "The row takes the keyboard: the focus tube round the well lights up over `short` on `standard`, as any control does. A pick with ← or → moves only the lit stretch and the words, as above; the tube stays on the well.",
        tracks: lights(".k-tabs > .k-ring", "the well", 0, 1),
        reduced: "same",
        back: "cuts",
        sample: () => row(TOOLS, 1),
      },
      {
        name: "A choice under the pointer",
        what: "The pointer comes to a choice that isn't picked: its words brighten from `on_surface_variant` to `on_surface` over `short` on `standard`, and back as it leaves. Nothing else lights.",
        tracks: lights('.k-tab[aria-selected="false"]', "the choice not picked", 0, 1),
        reduced: "same",
        back: "plays",
        sample: () => row(TOOLS, 1),
      },
      {
        name: "Pressed",
        what: "A choice that isn't picked, pressed: its words go to the picked colour over `micro`, and back if the pointer leaves before letting go; letting go picks it.",
        tracks: press('.k-tab[aria-selected="false"]', "the choice not picked"),
        reduced: "same",
        back: "plays",
        sample: () => row(TOOLS, 1),
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
      { name: "With symbols", for: "The well, its padding, each choice's height and padding, the symbol and its gap, and the rail.", sample: () => row(TOOLS, 1) },
      { name: "Views", for: "The well as wide as its choices, and the lit stretch under the pick's words.", sample: () => row(SECTIONS, 0) },
    ]),
    measures: [
      { what: "The well's height, a small control's.", token: "size-control_sm" },
      { what: "The well's padding round its choices.", token: "space-s4" },
      { what: "Inside each choice's sides, and the lit stretch's inset from them.", token: "space-s12" },
      { what: "The gap between a symbol and its words.", token: "space-s4" },
      { what: "Each choice's words, `button_small`'s line.", token: "type-button_small-line" },
      { what: "The well's corners.", token: "radius-full" },
      { what: "The rail's and the lit stretch's thickness at night.", token: "tabs-rail" },
      { what: "The rail's and the lit stretch's thickness by day.", token: "tabs-rail_day" },
    ],
    reflow: doesntApply("a row of tabs is as wide as its choices at every window size: it never stretches, wraps or scrolls. A row that would run past its room is a Select (Counts, above)."),
  },
});
