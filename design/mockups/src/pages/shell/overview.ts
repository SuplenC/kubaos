// The Overview: every workspace at once, full screen on every screen, each screen showing the
// workspace groups it holds; windows and whole workspaces are carried by hand between workspaces
// and across screens, and a workspace is deleted with a held button. `overview-parts.ts` draws it;
// a DRAFT, its parts local to the page until Kuba approves the look.

import type { Words } from "../../lib/content.ts";
import { doesntApply, type Move, nonEmpty, page, screen, type Track } from "../../site/page.ts";
import { DESKS, type OverviewOptions, overview, type Pin, sizeOf, VIEWS, zoneCard } from "./overview-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "empty";

/** The overview drawn still, from the three-screen desk's centre screen unless told. */
function still(options: Partial<OverviewOptions> & { pin?: Pin } = {}): HTMLElement {
  return overview({ desk: "three", view: "screen", ...options }).element;
}

const at = (options: Partial<OverviewOptions> = {}) => screen(() => still(options));

/**
 * A bottom corner of the centre screen cut out at its true size, `width` by `height` logical
 * pixels: the stage draws the whole screen at half its size, where the tip and the delete zone are
 * too small to read. The cut's size is the sample's frame, not a design value.
 */
function corner(options: Partial<OverviewOptions>, side: "left" | "right", width: number, height: number) {
  return screen(() => {
    const whole = sizeOf("three");
    const cut = document.createElement("span");
    cut.className = "ov-detail";
    cut.style.setProperty("--detail-w", `calc(${width} * var(--px))`);
    cut.style.setProperty("--detail-h", `calc(${height} * var(--px))`);
    cut.style.setProperty("--detail-x", side === "left" ? "0px" : `calc(${width - whole.width} * var(--px))`);
    cut.style.setProperty("--detail-y", `calc(${height - whole.height} * var(--px))`);
    cut.append(still(options));
    return cut;
  });
}

/**
 * The live stage, on the desk and in the view the stage's switches pick: which screens are plugged
 * in, and the focused screen alone or every screen as it stands. Both are the mockup's, not the
 * Overview's: it runs on every screen at once.
 */
function live(picked: readonly number[] = []): HTMLElement {
  return overview({ desk: DESKS[picked[0] ?? 0]!.desk, view: VIEWS[picked[1] ?? 0]!.view }).element;
}

/** The terminal, carried toward the delete zone, or over it. */
const TERMINAL = { window: "w-term" } as const;
/** The game's workspace, the centre's third. */
const GAME = { space: "s8" } as const;

/** The terminal on the centre's first workspace, carried over its third, the game. */
const WINDOW_OVER: Pin = { kind: "window", window: "w-term", to: { kind: "space", space: "s8" } };
/** The centre's fourth workspace carried between its first and second: they part for it. */
const SPACE_BETWEEN: Pin = { kind: "space", space: "s9", to: { kind: "between", group: "g-ext2", index: 1 } };

function stateOf(state: Shown): HTMLElement {
  if (state === "lit") return still({ pin: { kind: "lit", space: "s7" } });
  if (state === "pressed") return still({ pin: WINDOW_OVER });
  if (state === "selected") return still({ pin: SPACE_BETWEEN });
  if (state === "empty") return still({ desk: "laptop", fill: "empty" });
  return still();
}

const GROUP_NAME: Words = { short: "Studio", long: "Living room television" };

// ── Motion ────────────────────────────────────────────────────────────────────────────────────

/** The three parts of a workspace that move together: its box, its desktop, its number. */
const parts = (selector: string): string => `${selector}, ${selector} > .ov-ws-desk, ${selector} > .ov-ws-tag`;

const OPENING_TRACKS: [Track<"standard" | "emphasized_decel" | "emphasized_accel">, ...Track<"standard" | "emphasized_decel" | "emphasized_accel">[]] = [
  { part: "the frame and bar", on: "every screen", target: ".k-screen-glass, .k-frame, .k-bar", property: "--ov-aside", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
  { part: "the wallpaper's blur and dim", on: "every screen", target: ".ov-screen > .k-screen-wallpaper", property: "--ov-veil", from: 0, to: 1, duration: "long", curve: "standard" },
  { part: "the shown workspace", on: "its screen", target: parts('.ov-ws[data-rank="home"]'), property: "--ov-fly", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
  { part: "its neighbours", on: "one place away", target: parts('.ov-ws[data-rank="1"]'), property: "--ov-arrive", from: 0, to: 1, duration: "long", cue: "swap_lag", curve: "emphasized_decel" },
  { part: "two places away", on: "the next", target: parts('.ov-ws[data-rank="2"]'), property: "--ov-arrive", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
  { part: "the rest and the slots", on: "waiting groups too", target: `${parts('.ov-ws[data-rank="3"]')}, .ov-plus, .ov-label`, property: "--ov-arrive", from: 0, to: 1, duration: "long", cue: ["micro", "swap_lag"], curve: "emphasized_decel" },
];

const OPENING: Move = {
  name: "Opening",
  what: "The desktop shrinks into its miniature as the others arrive a `swap_lag` apart; the frame steps aside.",
  tracks: OPENING_TRACKS,
  reduced: [
    { part: "the frame and bar", on: "every screen", target: ".k-screen-glass, .k-frame, .k-bar", property: "--ov-aside", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the wallpaper's blur and dim", on: "every screen", target: ".ov-screen > .k-screen-wallpaper", property: "--ov-veil", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the shown workspace", on: "its place", target: parts('.ov-ws[data-rank="home"]'), property: "--ov-fly", from: 0, to: 0, duration: "instant", curve: "standard" },
    { part: "every miniature", on: "a fade", target: `${parts(".ov-ws")}, .ov-plus, .ov-label`, property: "--ov-arrive", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => still({ closed: true }),
};

const CLOSING: Move = {
  name: "Closing into a workspace",
  what: "The chosen miniature grows into the screen; the others fall back and fade; the frame comes back.",
  tracks: [
    { part: "the chosen workspace", on: "its screen", target: parts('.ov-ws[data-rank="home"]'), property: "--ov-fly", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the others", on: "every screen", target: `${parts('.ov-ws:not([data-rank="home"])')}, .ov-plus, .ov-label`, property: "--ov-arrive", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the wallpaper's blur and dim", on: "every screen", target: ".ov-screen > .k-screen-wallpaper", property: "--ov-veil", from: 1, to: 0, duration: "long", curve: "standard" },
    { part: "the frame and bar", on: "every screen", target: ".k-screen-glass, .k-frame, .k-bar", property: "--ov-aside", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "every miniature", on: "a fade", target: `${parts(".ov-ws:not([data-rank=\"home\"])")}, .ov-plus, .ov-label`, property: "--ov-arrive", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the wallpaper's blur and dim", on: "every screen", target: ".ov-screen > .k-screen-wallpaper", property: "--ov-veil", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the frame and bar", on: "every screen", target: ".k-screen-glass, .k-frame, .k-bar", property: "--ov-aside", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => still(),
};

const HOVER: Move = {
  name: "Pointed at",
  what: "A miniature under the pointer rises `space.s4` over `short`.",
  tracks: [{ part: "the miniature", on: "the centre's 2", target: '.ov-ws[data-space="s7"]', property: "--ov-hover", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: [{ part: "the miniature", on: "still", target: '.ov-ws[data-space="s7"]', property: "--ov-hover", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: () => still(),
};

const LIFT: Move = {
  name: "Lifted",
  what: "Pressed and moved `space.s8`: the window comes up to `scale.lift`, its shadow `shadow.lifted`.",
  tracks: [{ part: "the window", on: "the terminal", target: ".ov-win[data-held]", property: "--ov-held", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: [{ part: "the window", on: "no lift", target: ".ov-win[data-held]", property: "--ov-held", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: () => still({ pin: WINDOW_OVER }),
};

const DELETE: Move = {
  name: "Deleted",
  what: "Held to the end: its windows fade a `swap_lag` apart, then it folds back to `scale.behind`.",
  tracks: [
    { part: "its windows", on: "the centre's 2", target: '.ov-ws[data-space="s7"] .ov-win', property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the miniature", on: "folding", target: parts('.ov-ws[data-space="s7"]'), property: "--ov-arrive", from: 1, to: 0, duration: "medium", cue: "medium", curve: "emphasized_accel" },
  ],
  reduced: [{ part: "the miniature", on: "a fade", target: parts('.ov-ws[data-space="s7"]'), property: "--ov-arrive", from: 1, to: 0, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => still({ pin: { kind: "delete", space: "s7", held: 0 } }),
};

const ARRIVES: Move = {
  name: "A new workspace",
  what: "Dropped between two or on the slot: it grows in from `scale.behind` over `medium`.",
  tracks: [{ part: "the miniature", on: "the centre's 4", target: parts('.ov-ws[data-space="s9"]'), property: "--ov-arrive", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: "the miniature", on: "a fade", target: parts('.ov-ws[data-space="s9"]'), property: "--ov-arrive", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => still(),
};

const ZONE_ARRIVES: Move = {
  name: "The delete zone arriving",
  what: "Lifted: it rises `space.s24` into the corner and fades in over `medium`; it leaves over `short`.",
  tracks: [{ part: "the zone", on: "a carry", target: ".ov-bin", property: "opacity", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: "the zone", on: "a fade", target: ".ov-bin", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "plays",
  sample: () => still({ pin: { kind: "binning", what: TERMINAL, over: false } }),
};

const INTO_ZONE: Move = {
  name: "Into the delete zone",
  what: "Let go over it: the window shrinks into its symbol and fades over `medium`; the rest close up.",
  tracks: [{ part: "the window", on: "the terminal", target: '[data-window="w-term"]', property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: [{ part: "the window", on: "a fade", target: '[data-window="w-term"]', property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => still({ pin: { kind: "binning", what: TERMINAL, over: true } }),
};

const TIMER: Move = {
  name: "The timer running",
  what: "The Undo button's fill drains over `undo` on `linear`, as a held button's fills; empty, it closes.",
  tracks: [{ part: "the fill", on: "Undo", target: ".k-button", property: "--hold", from: 1, to: 0, duration: "undo", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => {
    const zone = document.createElement("span");
    zone.className = "ov-zone-alone";
    zone.append(zoneCard("window", { name: "just mockups", left: 1 }));
    return zone;
  },
};

export default page<Shown>({
  level: "shell",
  slug: "overview",
  title: "Overview",
  lede: "Every workspace at once, full screen on every screen: carry windows and workspaces between them and across screens.",
  code: { unbuilt: "a draft: not built" },
  from: ["icon-tile", "icon-button", "hint", "text", "workspaces"],
  stage: {
    live: screen(() => live()),
    hint: "left-drag a window, right-drag a workspace; arrows, Enter, Esc, W",
    daylight: true,
    switches: {
      each: [
        { label: "Desk", choices: nonEmpty(DESKS.map((each) => each.name)) },
        { label: "View", choices: nonEmpty(VIEWS.map((each) => each.name)) },
      ],
      live: screen(live),
    },
  },
  variants: [
    { name: "Three screens, the whole desk", for: "each screen its own group", sample: at({ view: "desk" }) },
    { name: "Laptop alone", for: "its group, three waiting under it", sample: at({ desk: "laptop" }) },
    { name: "Ultrawide alone", for: "its group, three waiting", sample: at({ desk: "ultrawide" }) },
    { name: "The portrait screen", for: "External 1's group, upright", sample: at({ focus: "portrait" }) },
    { name: "Crossing to another screen", for: "the terminal over the screens' edge", sample: at({ view: "desk", pin: { kind: "window", window: "w-term", to: { kind: "space", space: "s4" }, crossing: 0.55 } }) },
    { name: "On another screen's slot", for: "a window over the laptop's new slot", sample: at({ view: "desk", pin: { kind: "window", window: "w-term", to: { kind: "plus", group: "g-laptop" } } }) },
    { name: "Between two workspaces", for: "a window: a new workspace there", sample: at({ pin: { kind: "window", window: "w-term", to: { kind: "between", group: "g-ext2", index: 2 } } }) },
    { name: "A workspace to another screen", for: "the centre's 4 into the portrait's group", sample: at({ view: "desk", pin: { kind: "space", space: "s9", to: { kind: "between", group: "g-ext1", index: 1 } } }) },
    { name: "Refused by a full group", for: "ten in use: it dims, no room", sample: at({ view: "desk", fill: "full", pin: { kind: "space", space: "s1", to: { kind: "refused", group: "g-ext2" } } }) },
    { name: "The delete, lit", for: "under the keyboard's workspace", sample: at({ pin: { kind: "delete", space: "s7", held: 0 } }) },
    { name: "The delete, held halfway", for: "its ring half full", sample: at({ pin: { kind: "delete", space: "s7", held: 0.5 } }) },
    { name: "A workspace collapsing", for: "its windows gone, folding back", sample: at({ pin: { kind: "collapsing", space: "s7", at: 480 } }) },
    { name: "Closed", for: "the desktop it opens from", sample: at({ closed: true }) },
    { name: "The delete zone, carrying", for: "a window on its way to it", sample: at({ pin: { kind: "binning", what: TERMINAL, over: false } }) },
    { name: "Over the delete zone", for: "lit: let go, and it closes", sample: at({ pin: { kind: "binning", what: TERMINAL, over: true } }) },
    { name: "A workspace over the zone", for: "right-dragged: Remove", sample: at({ pin: { kind: "binning", what: GAME, over: true } }) },
    { name: "Waiting to close", for: "three fifths of its time left", sample: at({ pin: { kind: "pending", what: TERMINAL, left: 0.6 } }) },
    { name: "A second one over it", for: "let go, the first goes for good", sample: at({ pin: { kind: "pending", what: TERMINAL, left: 0.4, next: GAME } }) },
    { name: "The second dropped", for: "the first gone, the second waiting", sample: at({ pin: { kind: "pending", what: GAME, left: 1, confirmed: TERMINAL } }) },
    { name: "Brought back", for: "pressed: flying home", sample: at({ pin: { kind: "undoing", window: "w-term", share: 0.5 } }) },
    { name: "The tip, at true size", for: "the bottom left corner, 1:1", sample: corner({}, "left", 720, 160) },
    { name: "Over the zone, at true size", for: "the bottom right corner, 1:1", sample: corner({ pin: { kind: "binning", what: TERMINAL, over: true } }, "right", 720, 280) },
    { name: "Waiting, at true size", for: "the zone's timer and Undo, 1:1", sample: corner({ pin: { kind: "pending", what: TERMINAL, left: 0.6 } }, "right", 720, 280) },
  ],
  sizes: doesntApply("one size: the screen's; the miniatures take what the screen has."),
  states: {
    shown: ["rest", "lit", "pressed", "selected", "empty"],
    absent: {
      disabled: "always usable while open",
      loading: "drawn from what the compositor knows",
      error: "nothing it can fail at; a full group refuses",
    },
    rows: nonEmpty([{ name: "The centre screen", cell: screen((state: Shown) => stateOf(state)) }]),
    looks: {
      rest: "open: the groups this screen holds",
      lit: "the keyboard on a workspace: its tube",
      pressed: "a window carried over a workspace, its slot open",
      selected: "a workspace carried between two: they part",
      empty: "nothing open: one workspace, its slot",
    },
  },
  focus: {
    tubes: [
      { name: "A workspace, by the keyboard", tone: "accent", filled: false, content: false },
      { name: "A carry's target", tone: "accent", filled: false, content: false },
      { name: "The delete", tone: "burn_error", filled: true, content: false },
    ],
    looks: { lit: "the tube round the miniature, its glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: () => still({ pin: { kind: "lit", space: "s7" } }),
  },
  content: {
    text: [{ name: "A waiting group's name", words: GROUP_NAME, sample: (words: string) => still({ desk: "laptop", names: { "g-studio": words } }) }],
    missingPicture: doesntApply("a window is its live picture; an app with no icon has no badge."),
    counts: [
      { name: "None", for: "one empty workspace, its slot", sample: at({ desk: "laptop", fill: "empty" }) },
      { name: "Nine on one", for: "the centre's 1, tiled small", sample: at({ fill: "crowded" }) },
      { name: "A full group", for: "ten in use: no slot", sample: at({ fill: "full" }) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [HOVER, LIFT, DELETE, ARRIVES, ZONE_ARRIVES, INTO_ZONE, TIMER],
  },
  keyboard: {
    keys: [
      { keys: ["Super+W"], does: "opens it; again, or Esc, closes" },
      { keys: ["Left button"], does: "on a window, dragged: carries the window" },
      { keys: ["Right button"], does: "on a workspace, dragged: carries the workspace" },
      { keys: ["Left button"], does: "pressed: goes there; on the delete zone, brings it back" },
      { keys: ["←", "→", "↑", "↓"], does: "the next workspace that way, across screens" },
      { keys: ["Shift+←", "Shift+→", "Shift+↑", "Shift+↓"], does: "carries its focused window that way" },
      { keys: ["Enter"], does: "goes there: it grows into the screen" },
      { keys: ["Delete"], does: "held: deletes it, closing its windows" },
      { keys: ["Esc"], does: "puts a carried thing back, or closes" },
    ],
    order: "No stop for Tab: the arrows move the keyboard, across screens.",
    lands: "On the focused screen's shown workspace, its tube off until a key.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The centre screen", for: "miniatures, numbers, slot", sample: at() },
      { name: "Laptop alone", for: "the waiting sections", sample: at({ desk: "laptop" }) },
    ]),
    measures: [
      { part: "Screen margin", token: "space-s48" },
      { part: "Between miniatures", token: "space-s24" },
      { part: "Between sections", token: "space-s32" },
      { part: "Number under", token: "space-s8" },
      { part: "Number and delete row", token: "size-control_sm" },
      { part: "Number", token: "size-chip" },
      { part: "Name over a section", token: "size-chip" },
      { part: "Miniature corners", token: "radius-control" },
      { part: "Window corners", token: "radius-surface" },
      { part: "Window gaps", token: "space-s8" },
      { part: "Wallpaper blur", token: "size-lock_blur" },
      { part: "Slit between two", token: "space-s4" },
    ],
    reflow: { note: "Its own group up to a 3 × 3 grid's size; waiting groups at half; rows centred.", sample: at({ desk: "laptop" }) },
  },
});
