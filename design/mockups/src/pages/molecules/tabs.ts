// The Tabs page: `docs/design/components/tabs.md`. One of a few views or values, picked along a
// rail of neon in one well; the same well is the segmented control. The pick's lit stretch travels
// to the new choice and flashes as it lands.

import type { Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack } from "../../lib/layout.ts";
import { type Tab, tabs } from "../../lib/tabs.ts";
import { wrapped } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, type Track } from "../../site/page.ts";

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

/** The well filling a row a menu wide, as a setting's row gives it. */
function filling(choices: readonly Tab[], picked: number): HTMLElement {
  const room = document.createElement("span");
  room.className = "f-tabs-room";
  room.append(tabs({ choices, picked, fill: true }));
  return room;
}

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

export default page({
  level: "molecules",
  slug: "tabs",
  title: "Tabs",
  lede: "One of a few views or values, picked along a rail of neon in one well.",
  code: { rust: "tabs::tabs", qml: "Tabs", unbuilt: "the neon rail: not built yet" },
  stage: { live: () => stack("s24", "start", sections(), row(TOOLS, 1), row(MOTION, 1)), hint: "click a choice, or Tab and ← →", daylight: true },
  variants: [
    { name: "Views", for: "a few views of one thing: the control center", sample: () => row(SECTIONS, 0) },
    { name: "With symbols", for: "a symbol helps tell them apart: Timer, Alarm", sample: () => row(TOOLS, 1) },
    { name: "Segmented control", for: "a setting's few values side by side", sample: () => row(MOTION, 1) },
    { name: "Filling its row", for: "a setting's row: the choices share the well, ending at its end", sample: () => filling(MOTION, 1) },
  ],
  sizes: doesntApply("always a small control tall: no louder than the view."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "the lit stretch and accent words, in every row",
      loading: "a view still loading shows its own skeleton",
      empty: "a row always has a pick",
      error: "it has no wrong pick",
    },
    rows: nonEmpty(ROWS.map(({ name, choices, picked }) => ({ name, cell: (state: MatrixState) => row(choices, picked, state) }))),
    looks: {
      rest: "unlit rail, the pick's stretch lit; words `primary_lit`",
      lit: "focus tube round the well; a pointed choice brightens its words",
      pressed: "the choice's words go to the picked colour",
      disabled: "every part at `opacity.disabled_content`, the pick still shown",
    },
  },
  focus: {
    tubes: [{ name: "The well, keyboard on the row", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "tube round the well, stretch and picked words glowing",
      signsOff: "tube and stretch `primary_glass`; words keep `primary_lit`, no glow",
    },
    signsOff: (index: number) => row(ROWS[index]!.choices, ROWS[index]!.picked, "signs-off"),
  },
  content: {
    text: [{ name: "A choice's words", words: LONG, sample: (words: string) => row([{ label: "Always" }, { label: words }, { label: "Never" }], 1) }],
    missingPicture: doesntApply("a symbol is optional: a row's choices all have one or none."),
    counts: [
      { name: "Two", for: "the fewest", sample: () => row(TOOLS, 0) },
      { name: "Four", for: "the control center's sections", sample: () => row(SECTIONS, 2) },
      {
        name: "Too many for the row",
        for: "past its room, choices become a `Select`",
        sample: () => row([...SECTIONS, { label: "Display" }, { label: "Power" }], 0),
      },
    ],
  },
  motion: {
    arrives: doesntApply("a row of tabs comes with the surface it is on."),
    leaves: doesntApply("it goes with the surface it is on."),
    between: [
      pickMove("Picking, to the right", "The stretch's leading edge gets there over `short`, the trail over `medium`; the core flashes, the words cross-fade: Wi-Fi to Output.", 0, 2),
      pickMove("Picking, to the left", "The same the longest way, Keyboard back to Wi-Fi: the left edge leads, on `emphasized_decel`.", 3, 0),
      {
        name: "The keyboard comes to the row",
        what: "The tube round the well lights over `short` on `standard`; a pick with ← → moves only the stretch.",
        tracks: lights(".k-tabs > .k-ring", "the well", 0, 1),
        reduced: "same",
        back: "cuts",
        sample: () => row(TOOLS, 1),
      },
      {
        name: "A choice under the pointer",
        what: "Its words brighten to `on_surface` over `short` on `standard`, and back as the pointer leaves.",
        tracks: lights('.k-tab[aria-selected="false"]', "the choice not picked", 0, 1),
        reduced: "same",
        back: "plays",
        sample: () => row(TOOLS, 1),
      },
      {
        name: "Pressed",
        what: "Its words go to the picked colour over `micro`; letting go picks it.",
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
      { keys: ["←", "h"], does: "picks the one before, wrapping round" },
      { keys: ["→", "l"], does: "picks the one after, wrapping round" },
      { keys: ["Tab", "Shift+Tab"], does: "into the row, on its pick, or out: one stop" },
      { keys: ["Space", "Enter"], does: "nothing more: the keyboard's choice is already picked" },
    ],
    order: "One stop in reading order, on the pick; a disabled row is skipped.",
    lands: "On the pick when a surface opens on its tabs; a pick keeps it on the row.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With symbols", for: "well, padding, choice, symbol and gap, rail", sample: () => row(TOOLS, 1) },
      { name: "Views", for: "the well as wide as its choices; the lit stretch", sample: () => row(SECTIONS, 0) },
    ]),
    measures: [
      { part: "Well height", token: "size-control_sm" },
      { part: "Well padding", token: "space-s4" },
      { part: "Choice sides, stretch inset", token: "space-s12" },
      { part: "Symbol → words", token: "space-s4" },
      { part: "Words' line", token: "type-button_small-line" },
      { part: "Well corners", token: "radius-full" },
      { part: "Rail at night", token: "tabs-rail" },
      { part: "Rail by day", token: "tabs-rail_day" },
    ],
    reflow: doesntApply("as wide as its choices: it never stretches, wraps or scrolls."),
  },
});
