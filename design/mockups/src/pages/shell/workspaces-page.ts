// The Workspaces page: `config/quickshell/modules/bar/Workspaces.qml`. This screen's workspaces in
// groups, flat on the bar's glass, each a mark in a fixed cell (the marks a setting), the shown
// one's lit; each group's `+` and a borrowed group's name while the pointer is on them or the
// Overview is up; a workspace carried to another place, and the right-click menu.

import type { Words } from "../../lib/content.ts";
import { rollingMidway } from "../../lib/rolling-number.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, type Track } from "../../site/page.ts";
import { onGlass } from "./bar-parts.ts";
import { type Group, type Marks, SHOTS_GROUPS, workspaces as part, type WorkspacesOptions } from "./workspaces.ts";

/** The workspaces on a piece of the bar's glass, as the bar holds them. */
const workspaces = (options: WorkspacesOptions = {}) => onGlass(part(options));

/** The marks' choices, as the setting names them, and what each draws. */
const FORMS: readonly { readonly name: string; readonly options: WorkspacesOptions }[] = [
  { name: "Dot and dash", options: { marks: "dotdash" } },
  { name: "Numbers", options: { marks: "numbers" } },
  { name: "Numbers, kanji", options: { marks: "numbers", kanji: true } },
  { name: "Only the current one", options: { marks: "numbers", kanji: true, onlyCurrent: true } },
  { name: "Dots", options: { marks: "dots" } },
  { name: "Dashes", options: { marks: "dashes" } },
];

type State = "rest" | "hovered";

const GROWN: readonly Group[] = [{ block: 0, spaces: [{ id: 1, occupied: true }, { id: 2, occupied: true }, { id: 3, occupied: true }, { id: 4, occupied: false }] }];
const TEN: readonly Group[] = [{ block: 0, spaces: Array.from({ length: 10 }, (_, index) => ({ id: index + 1, occupied: index < 9 })) }];
const UNNAMED: readonly Group[] = [SHOTS_GROUPS[0]!, { block: 1, borrowed: true, spaces: [{ id: 11, occupied: true }, { id: 12, occupied: false }] }];
const THREE: readonly Group[] = [
  SHOTS_GROUPS[0]!,
  { block: 1, borrowed: true, name: "Desk", spaces: [{ id: 11, occupied: true }, { id: 12, occupied: false }] },
  { block: 2, borrowed: true, name: "TV", spaces: [{ id: 21, occupied: true }, { id: 22, occupied: false }] },
];

/** The live sample's: four with windows, 2 shown, and Desk's two. */
const LIVE: readonly Group[] = [
  { block: 0, spaces: [1, 2, 3, 4].map((id) => ({ id, occupied: true })).concat([{ id: 5, occupied: false }]) },
  { block: 1, borrowed: true, name: "Desk", spaces: [{ id: 11, occupied: true }, { id: 12, occupied: true }, { id: 13, occupied: false }] },
];

/** The live sample's, Desk full: ten workspaces, so it has no room for another. */
const FULL: readonly Group[] = [LIVE[0]!, { block: 1, borrowed: true, name: "Desk", spaces: Array.from({ length: 10 }, (_, index) => ({ id: 11 + index, occupied: true })) }];

const DISC = ".shell-ws-disc";

/** A move's tracks: on a token's curve, one at least. */
type Tracks = NonEmpty<Track<Easing>>;

/** The same tracks at once: reduced motion moves nothing, it changes. */
function atOnce(tracks: Tracks): Tracks {
  return nonEmpty(tracks.map((track) => ({ ...track, duration: "instant" as const })));
}

/** The disc easing from where its glide started (its cell, or the hand) to where it is drawn, over `duration` on `curve`. */
function discMove(name: string, what: string, on: string, duration: "short" | "medium", curve: "standard" | "standard_decel" | "emphasized_decel", sample: () => HTMLElement): Move {
  const tracks: Tracks = [{ part: "the disc", on, target: DISC, property: "--ws-glide", from: 1, to: 0, duration, curve }];
  return { name, what, tracks, reduced: atOnce(tracks), back: "plays", sample };
}

const LIFTS = discMove("Lifted onto its disc", "Pressed and moved `space.s8`: the disc rises from its cell onto the hand over `short` on `standard`.", "workspace 4", "short", "standard", () => workspaces({ groups: LIVE, drag: { id: 4, loose: true } }));
const SNAPS = discMove("Snapped onto a place", "Its magnet reached: the disc eases from the hand onto the place over `medium` on `standard_decel`.", "workspace 4", "medium", "standard_decel", () => workspaces({ groups: LIVE, drag: { id: 4, to: { block: 0, index: 1 } } }));
const DROPS = discMove("Dropped", "Let go: the disc flies from the hand into its place over `medium` on `emphasized_decel`.", "workspace 4", "medium", "emphasized_decel", () => workspaces({ groups: LIVE, drag: { id: 4, to: { block: 0, index: 1 } } }));

const SLIDE: Tracks = [{ part: "its content", on: "the windows of 3, now 2", target: '.shell-ws-slot[data-key="w3@0"]', property: "--ws-slide", from: 1, to: 0, duration: "medium", curve: "standard_decel" }];
const SLIDES: Move = {
  name: "The cells between sliding over",
  what: "Its place moved: each cell between slides a place over `medium` on `standard_decel`; its tint stays.",
  tracks: SLIDE,
  reduced: atOnce(SLIDE),
  back: "plays",
  sample: () => workspaces({ groups: LIVE, drag: { id: 2, to: { block: 0, index: 3 } } }),
};

const CROSSING: Tracks = [{ part: "its place", on: "Desk's new place", target: '.shell-ws-slot[data-kind="hole"]', property: "--present", from: 0, to: 1, duration: "medium", curve: "standard_decel" }];
const CROSSES: Move = {
  name: "Its place changing group",
  what: "One group gives its place up as the other grows one, together over `medium` on `standard_decel`.",
  tracks: CROSSING,
  reduced: atOnce(CROSSING),
  back: "plays",
  sample: () => workspaces({ groups: LIVE, drag: { id: 4, to: { block: 1, index: 1 } } }),
};

const ROLL: Tracks = [
  { part: "the figure leaving", on: "a number that changes", target: ".k-roll-figure[data-out]", property: "--roll", from: 0, to: 1, duration: "short", curve: "standard" },
  { part: "the figure arriving", on: "a number that changes", target: ".k-roll-figure[data-in]", property: "--roll", from: 0, to: 1, duration: "short", curve: "standard" },
];
const ROLLS: Move = {
  name: "A number rolling",
  what: "Windows at another number roll to it over `short` on `standard`; a change of figure count eases the box.",
  tracks: ROLL,
  reduced: atOnce(ROLL),
  back: "cuts",
  sample: () => rollingMidway(3, 4, "0", "button"),
};

/** Showing another workspace: the leaving mark turns off as the arriving one turns on, a `swap_lag` later. */
function switching(marks: Marks): Move {
  const leaving = '.shell-ws-cell[data-space="w2"]';
  const arriving = '.shell-ws-cell[data-space="w1"]';
  const tracks: Track<Easing>[] = [
    { part: "the leaving mark's light", on: "workspace 2", target: leaving, property: "--ws-lit", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the arriving mark's light", on: "workspace 1", target: arriving, property: "--ws-lit", from: 0, to: 1, duration: "short", cue: "swap_lag", curve: "standard" },
  ];
  if (marks !== "dots" && marks !== "numbers") {
    tracks.push(
      { part: "the leaving dash narrowing", on: "workspace 2", target: leaving, property: "--ws-shape", from: 1, to: 0, duration: "short", curve: "standard" },
      { part: "the arriving dash widening", on: "workspace 1", target: arriving, property: "--ws-shape", from: 0, to: 1, duration: "medium", cue: "swap_lag", curve: "emphasized_decel" },
    );
  }
  return {
    name: `Switching, ${FORMS.find((form) => form.options.marks === marks)!.name}`,
    what: "The leaving mark turns off over `short`; the arriving one turns on a `swap_lag` later, a dash widening over `medium`.",
    tracks: nonEmpty(tracks),
    reduced: atOnce(nonEmpty(tracks)),
    back: "plays",
    sample: () => workspaces({ marks, active: 1 }),
  };
}

const EXTRA: Tracks = [
  { part: "their room", on: "each `+`, Desk's name", target: ".shell-ws", property: "--ws-extras", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
  { part: "their opacity", on: "each `+`, Desk's name", target: ".shell-ws", property: "--ws-extras-shown", from: 0, to: 1, duration: "long", curve: "standard" },
];
const EXTRAS: Move = {
  name: "The pointer onto them",
  what: "Each group's `+` and Desk's name open their room over `long` on `emphasized_decel`, fading in on `standard`.",
  tracks: EXTRA,
  reduced: atOnce(EXTRA),
  back: "plays",
  sample: () => workspaces({ groups: LIVE }),
};

const MARK: Tracks = [{ part: "its circle", on: "workspace 3", target: ".shell-ws-slot[data-menu] > .shell-ws-cell", property: "--ws-mark", from: 0, to: 1, duration: "short", curve: "standard" }];
const MARKS: Move = {
  name: "The menu's mark",
  what: "Its menu opened: a circle of `primary` at `opacity.tint` fades in behind it over `short` on `standard`.",
  tracks: MARK,
  reduced: atOnce(MARK),
  back: "plays",
  sample: () => workspaces({ groups: LIVE, menu: { workspace: 3 } }),
};

const GONE: Tracks = [
  { part: "its width", on: "Desk, all of it", target: '.shell-ws-slot[data-block="1"]', property: "--present", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
  { part: "its opacity", on: "Desk, all of it", target: '.shell-ws-slot[data-block="1"]', property: "--ws-shown", from: 1, to: 0, duration: "long", curve: "standard" },
];
const GROUP_GOES: Move = {
  name: "A borrowed group deleted",
  what: "Its hairlines, name and workspaces shrink out over `long` on `emphasized_decel`, fading with them.",
  tracks: GONE,
  reduced: atOnce(GONE),
  back: "plays",
  sample: () => workspaces(),
};

const GROWS: Move = {
  name: "A workspace grows in",
  what: "Added by `+`, before it: its width over `long` on `emphasized_decel`, its opacity on `standard`.",
  tracks: [
    { part: "its width", on: "workspace 4", target: '.shell-ws-slot[data-key="w4@0"]', property: "--present", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "its opacity", on: "workspace 4", target: '.shell-ws-slot[data-key="w4@0"]', property: "--ws-shown", from: 0, to: 1, duration: "long", curve: "standard" },
  ],
  reduced: "same",
  back: "plays",
  sample: () => workspaces({ groups: GROWN, active: 4 }),
};


const NAME: Words = { short: "TV", long: "Living room" };

export default page<State>({
  level: "shell",
  slug: "workspaces",
  title: "Workspaces",
  lede: "This screen's workspaces in groups, a mark each, the shown one lit; `+` and names under the pointer.",
  code: { qml: "Workspaces" },
  stage: {
    live: () => workspaces({ groups: LIVE, live: true }),
    switches: {
      each: [
        { label: "Marks", choices: ["Dot and dash", "Numbers", "Kanji", "Only current", "Dots", "Dashes"] },
        { label: "Overview", choices: ["Down", "Up"] },
      ],
      live: ([form, overview]) => workspaces({ groups: LIVE, live: true, urgent: 3, ...FORMS[form ?? 0]!.options, overview: overview === 1 }),
    },
    hint: "click or scroll; drag one; right-click one",
    daylight: true,
  },
  variants: [
    ...FORMS.map(({ name, options }) => ({ name, for: name === "Dot and dash" ? "the default: the shown one a dash" : "a setting", sample: () => workspaces({ groups: LIVE, ...options }) })),
    { name: "Urgent, Dot and dash", for: "a lit dot in `error`", sample: () => workspaces({ groups: LIVE, urgent: 3 }) },
    { name: "Urgent, Dashes", for: "a lit dash in `error`", sample: () => workspaces({ groups: LIVE, marks: "dashes", urgent: 3 }) },
    { name: "Urgent, Numbers", for: "its number lit in `error`", sample: () => workspaces({ groups: LIVE, marks: "numbers", urgent: 3 }) },
    { name: "One group", for: "the screen's own", sample: () => workspaces({ groups: SHOTS_GROUPS.slice(0, 1) }) },
    { name: "Borrowed, at rest", for: "a stronger hairline before and after it", sample: () => workspaces() },
    { name: "Borrowed, unnamed", for: "the hairlines alone, even under the pointer", sample: () => workspaces({ groups: UNNAMED, pinned: "hover" }) },
    { name: "The Overview up", for: "every `+` and the name, as under the pointer", sample: () => workspaces({ groups: LIVE, overview: true }) },
  ],
  sizes: doesntApply("one row, `size.bar_row`; fixed cells, `size.ws_cell`."),
  groups: [
    {
      title: "Carried",
      rows: [
        { name: "Lifted", for: "4 on its disc, its place left empty", sample: () => workspaces({ groups: LIVE, drag: { id: 4, loose: true } }) },
        { name: "Its place moved", for: "snapped onto 2, reading 2: 2 and 3 slid over", sample: () => workspaces({ groups: LIVE, drag: { id: 4, to: { block: 0, index: 1 } } }) },
        { name: "A group taking it", for: "Desk outlined, an empty place in it", sample: () => workspaces({ groups: LIVE, drag: { id: 4, to: { block: 1, index: 1 } } }) },
        { name: "Onto a group's ghost", for: "last in Desk", sample: () => workspaces({ groups: LIVE, drag: { id: 4, to: { block: 1, index: 2 } } }) },
        { name: "Refused by a full group", for: "over Desk's ten: it dims, its place home", sample: () => workspaces({ groups: FULL, drag: { id: 3, over: 1 } }) },
      ],
    },
    {
      title: "Its menu",
      rows: [
        { name: "On a workspace", for: "its actions, then its group's; it takes a light primary circle", sample: () => workspaces({ groups: LIVE, menu: { workspace: 3 } }) },
        { name: "On a group", for: "its name or ghost: the group's alone", sample: () => workspaces({ groups: LIVE, menu: { group: 1 } }) },
        { name: "Delete held", for: "`error` sweeps across; acts when full", sample: () => workspaces({ groups: LIVE, menu: { workspace: 3, holding: 0.6 } }) },
        { name: "Delete group, borrowed", for: "Desk's windows close and the whole group goes", sample: () => workspaces({ groups: LIVE, menu: { group: 1, holding: 0.6 } }) },
        { name: "Delete group, the screen's own", for: "its windows close; one empty workspace stays", sample: () => workspaces({ groups: LIVE, menu: { group: 0, holding: 0.6 } }) },
      ],
    },
  ],
  states: {
    shown: ["rest", "hovered"],
    absent: {
      lit: "no keyboard focus: the bar is never a Tab stop",
      pressed: "a click shows the workspace; nothing stays pressed",
      disabled: "always there",
      selected: "the shown workspace: its mark lit, in every row",
      loading: "drawn once Hyprland answers",
      empty: "a screen always has its first workspace",
      error: "nothing it can fail at",
    },
    rows: nonEmpty(FORMS.map(({ name, options }) => ({ name, cell: (state: State) => workspaces({ ...options, ...(state === "hovered" ? { pinned: "hover" as const } : {}) }) }))),
    looks: {
      rest: "flat; shown lit in `primary`; with windows `on_surface_variant`; empty `outline`",
      hovered: "`+` and names in; a `surface_container_high` circle under the cell",
    },
  },
  focus: doesntApply("the bar takes no keyboard focus; no edge light under the workspaces."),
  content: {
    text: [{ name: "A group's name", words: NAME, sample: (words: string) => workspaces({ groups: [SHOTS_GROUPS[0]!, { ...SHOTS_GROUPS[1]!, name: words }] }) }],
    missingPicture: doesntApply("marks and one symbol, `add`, always there."),
    counts: [
      { name: "One", for: "the screen's first workspace, shown", sample: () => workspaces({ groups: [{ block: 0, spaces: [{ id: 1, occupied: false }] }], active: 1 }) },
      { name: "Ten", for: "a full group: no `+`", sample: () => workspaces({ groups: TEN, active: 4 }) },
      { name: "Three groups", for: "a hairline in each gap", sample: () => workspaces({ groups: THREE }) },
    ],
  },
  motion: {
    arrives: [GROWS],
    leaves: doesntApply("a deleted workspace, or a deleted borrowed group with its rule and name, shrinks out: Arriving's move, played back."),
    between: [
      switching("dotdash"),
      switching("dashes"),
      switching("dots"),
      switching("numbers"),
      EXTRAS,
      { name: "Hovered", what: "The pointer onto a cell: its circle fades in over `short`.", tracks: [{ part: "the circle", on: "the cell under the pointer", target: ".shell-ws-cell[data-pointer]", property: "--cell-hover", from: 0, to: 1, duration: "short", curve: "standard" }], reduced: "same", back: "plays", sample: () => workspaces() },
      LIFTS,
      SNAPS,
      SLIDES,
      CROSSES,
      DROPS,
      ROLLS,
      MARKS,
      GROUP_GOES,
    ],
  },
  keyboard: {
    keys: [
      { keys: ["SUPER+1", "SUPER+0"], does: "workspace 1 to 10 of this screen's group" },
      { keys: ["Wheel"], does: "the next or previous, round, in block order" },
      { keys: ["Drag"], does: "a workspace to another place or group; a full group refuses" },
      { keys: ["Right click"], does: "its menu; Esc backs out of a list, then closes" },
    ],
    order: "Never a stop for Tab.",
    lands: "The keyboard stays with the window shown.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Two groups", for: "cells, the hairlines, the name", sample: () => workspaces({ pinned: "hover" }) },
      { name: "Kanji", for: "the same cells", sample: () => workspaces({ marks: "numbers", kanji: true }) },
      { name: "Dashes", for: "a dash and the shown one's", sample: () => workspaces({ marks: "dashes" }) },
    ]),
    measures: [
      { part: "Row", token: "size-bar_row" },
      { part: "Cell", token: "size-ws_cell" },
      { part: "A dot", token: "size-ws_dot" },
      { part: "A dash", token: "size-ws_dash" },
      { part: "The shown dash", token: "size-ws_dash_lit" },
      { part: "A dash's thickness", token: "size-ws_dash_thick" },
      { part: "Dot and dash's thickness", token: "size-ws_dotdash_thick" },
      { part: "An empty ring", token: "stroke-thin" },
      { part: "Gap between groups", token: "space-s20" },
      { part: "Hairline's height", token: "size-bar_divider" },
      { part: "A name, at most", token: "space-s48" },
      { part: "Ghost", token: "icon-s16" },
      { part: "Number's line", token: "type-button-line" },
    ],
    reflow: doesntApply("as wide as its entries; the title beside it gives way."),
  },
});
