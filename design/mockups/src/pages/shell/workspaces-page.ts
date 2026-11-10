// The Workspaces page: `config/quickshell/modules/bar/Workspaces.qml`. This screen's workspaces in
// groups, the shown one on a stretching indicator, each group's ghost after its last.

import type { Words } from "../../lib/content.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { type Group, SHOTS_GROUPS, workspaces } from "./workspaces.ts";

type State = "rest" | "lit";

const ONE: readonly Group[] = [{ block: 0, spaces: [{ id: 1, occupied: true }, { id: 2, occupied: true }, { id: 3, occupied: true }] }];
const GROWN: readonly Group[] = [{ block: 0, spaces: [{ id: 1, occupied: true }, { id: 2, occupied: true }, { id: 3, occupied: true }, { id: 4, occupied: false }] }];
const TEN: readonly Group[] = [{ block: 0, spaces: Array.from({ length: 10 }, (_, index) => ({ id: index + 1, occupied: index < 9 })) }];
const UNNAMED: readonly Group[] = [SHOTS_GROUPS[0]!, { block: 1, borrowed: true, spaces: [{ id: 11, occupied: true }, { id: 12, occupied: false }] }];
const THREE: readonly Group[] = [
  SHOTS_GROUPS[0]!,
  { block: 1, borrowed: true, name: "Desk", spaces: [{ id: 11, occupied: true }, { id: 12, occupied: false }] },
  { block: 2, borrowed: true, name: "TV", spaces: [{ id: 21, occupied: true }, { id: 22, occupied: false }] },
];

const INDICATOR = ".shell-ws-indicator";

function stretch(name: string, what: string, from: number, to: number): Move {
  const right = to > from;
  return {
    name,
    what,
    tracks: [
      { part: "the indicator's leading edge", on: "the shown workspace", target: INDICATOR, property: right ? "--ws-end" : "--ws-start", from: right ? from + 1 : from, to: right ? to + 1 : to, duration: "medium", curve: "emphasized_decel" },
      { part: "the indicator's trailing edge", on: "the shown workspace", target: INDICATOR, property: right ? "--ws-start" : "--ws-end", from: right ? from : from + 1, to: right ? to : to + 1, duration: "long", curve: "emphasized_decel" },
    ],
    reduced: [
      { part: "the indicator's leading edge", on: "the shown workspace", target: INDICATOR, property: right ? "--ws-end" : "--ws-start", from: right ? from + 1 : from, to: right ? to + 1 : to, duration: "instant", curve: "emphasized_decel" },
      { part: "the indicator's trailing edge", on: "the shown workspace", target: INDICATOR, property: right ? "--ws-start" : "--ws-end", from: right ? from : from + 1, to: right ? to : to + 1, duration: "instant", curve: "emphasized_decel" },
    ],
    back: "plays",
    sample: () => workspaces({ groups: ONE, active: from + 1 }),
  };
}

const GROWS: Move = {
  name: "A workspace grows in",
  what: "A new ghost after the last: its width over `long` on `emphasized_decel`, its opacity on `standard`.",
  tracks: [
    { part: "its width", on: "workspace 4", target: '.shell-ws-slot[data-key="4"]', property: "--present", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "its opacity", on: "workspace 4", target: '.shell-ws-slot[data-key="4"]', property: "--ws-shown", from: 0, to: 1, duration: "long", curve: "standard" },
  ],
  reduced: "same",
  back: "plays",
  sample: () => workspaces({ groups: GROWN, active: 3 }),
};

const GHOST: Move = {
  name: "A ghost turns into a workspace",
  what: "Shown or given a window: its number fades in and the `add` out, over `short` on `standard`.",
  tracks: [
    { part: "its number", on: "the ghost", target: ".shell-ws-cell[data-ghost] > .shell-ws-number", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "its add", on: "the ghost", target: ".shell-ws-cell[data-ghost] > .shell-ws-ghost", property: "opacity", from: { opacity: "placeholder" }, to: 0, duration: "short", curve: "standard" },
  ],
  reduced: "same",
  back: "plays",
  sample: () => workspaces(),
};

const NAME: Words = { short: "TV", long: "Living room" };

export default page<State>({
  level: "shell",
  slug: "workspaces",
  title: "Workspaces",
  lede: "This screen's workspaces in groups: the shown one on the indicator, a ghost after each group's last.",
  code: { qml: "Workspaces" },
  stage: { live: () => workspaces({ live: true }), hint: "click a number or a ghost; scroll over it", daylight: true },
  variants: [
    { name: "Numbers", for: "1–10, as SUPER + 1–0 reach them", sample: () => workspaces() },
    { name: "Kanji", for: "the setting: 一 to 十 in `accent`", sample: () => workspaces({ kanji: true }) },
    { name: "One group", for: "the screen's own, its ghost last", sample: () => workspaces({ groups: SHOTS_GROUPS.slice(0, 1) }) },
    { name: "Borrowed, unnamed", for: "the tint alone", sample: () => workspaces({ groups: UNNAMED }) },
    { name: "Borrowed, named", for: "its name first, `secondary`", sample: () => workspaces() },
  ],
  sizes: doesntApply("one height, `size.control_sm`; as wide as its entries."),
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "a click shows the workspace; nothing stays pressed",
      disabled: "always there",
      selected: "the shown workspace: the indicator, in every row",
      loading: "drawn once Hyprland answers",
      empty: "a screen always has its first workspace",
      error: "nothing it can fail at",
    },
    rows: nonEmpty([
      { name: "Numbers", cell: (state: State) => workspaces(state === "lit" ? { pinned: "lit" } : {}) },
      { name: "Kanji", cell: (state: State) => workspaces({ kanji: true, ...(state === "lit" ? { pinned: "lit" as const } : {}) }) },
    ]),
    looks: {
      rest: "`surface_container`; shown `on_primary` on `primary`; with windows `on_surface`",
      lit: "pointer: the pill's tube and glow",
    },
  },
  focus: {
    tubes: [{ name: "The pill", tone: "accent", filled: false, content: false }],
    looks: { lit: "the tube inside the pill's edge, the glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => workspaces({ kanji: index === 1, pinned: "signs-off" }),
  },
  content: {
    text: [{ name: "A group's name", words: NAME, sample: (words: string) => workspaces({ groups: [SHOTS_GROUPS[0]!, { ...SHOTS_GROUPS[1]!, name: words }] }) }],
    missingPicture: doesntApply("numbers and one symbol, `add`, always there."),
    counts: [
      { name: "One", for: "the screen's first workspace, shown", sample: () => workspaces({ groups: [{ block: 0, spaces: [{ id: 1, occupied: false }] }], active: 1 }) },
      { name: "Ten", for: "a full group: no ghost", sample: () => workspaces({ groups: TEN, active: 4 }) },
      { name: "Three groups", for: "a rule in each gap", sample: () => workspaces({ groups: THREE }) },
    ],
  },
  motion: {
    arrives: [GROWS],
    leaves: doesntApply("an empty workspace left shrinks out: Arriving's move, played back."),
    between: [
      stretch("To the right", "The leading edge over `medium`, the trailing one over `long`, both `emphasized_decel`.", 0, 2),
      stretch("To the left", "The same stretch, mirrored: the left edge leads.", 2, 0),
      GHOST,
      { name: "Lit", what: "The pointer onto it: it lights over `short` on `standard`.", tracks: lights(".shell-ws", "the pill", 0, 1), reduced: "same", back: "plays", sample: () => workspaces() },
      ...powerMoves(() => workspaces({ pinned: "lit" })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["SUPER+1", "SUPER+0"], does: "workspace 1 to 10 of this screen's group" },
      { keys: ["Wheel"], does: "the next or previous, round, in block order" },
    ],
    order: "Never a stop for Tab.",
    lands: "The keyboard stays with the window shown.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Two groups", for: "cells, the rule, the tint, the name", sample: () => workspaces() },
      { name: "Kanji", for: "the same cells", sample: () => workspaces({ kanji: true }) },
    ]),
    measures: [
      { part: "Height", token: "size-control_sm" },
      { part: "Inset round the cells", token: "space-s4" },
      { part: "Cell, indicator", token: "size-chip" },
      { part: "Gap between groups", token: "space-s12" },
      { part: "Rule", token: "stroke-hairline_device_px" },
      { part: "A name, at most", token: "space-s48" },
      { part: "Ghost", token: "icon-s16" },
      { part: "Number's line", token: "type-button_small-line" },
    ],
    reflow: doesntApply("as wide as its entries; the title beside it gives way."),
  },
});
