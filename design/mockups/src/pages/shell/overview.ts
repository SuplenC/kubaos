// The Overview: every workspace at once, full screen on every screen, each screen showing the
// workspace groups it holds; windows and whole workspaces are carried by hand between workspaces
// and across screens, and either is dropped in the delete slot to go. `overview-parts.ts` draws it;
// a DRAFT, its parts local to the page until Kuba approves the look.

import type { Words } from "../../lib/content.ts";
import { doesntApply, type Move, nonEmpty, page, screen, type Track } from "../../site/page.ts";
import { DESKS, type OverviewOptions, overview, type Pin, sizeOf, VIEWS } from "./overview-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "empty";

/** The overview drawn still, from the three-screen desk's centre screen unless told. */
function still(options: Partial<OverviewOptions> & { pin?: Pin } = {}): HTMLElement {
  return overview({ desk: "three", view: "screen", ...options }).element;
}

const at = (options: Partial<OverviewOptions> = {}) => screen(() => still(options));

/**
 * A bottom corner of the centre screen cut out at its true size, `width` by `height` logical
 * pixels: the stage draws the whole screen at half its size, where the tip and the delete slot are
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

/** The terminal, carried toward the delete slot, or over it. */
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

const LIFT: Move = {
  name: "Lifted",
  what: "Pressed and moved `space.s8`: the window comes up to `scale.lift`, its shadow `shadow.lifted`.",
  tracks: [{ part: "the window", on: "the terminal", target: ".ov-win[data-held]", property: "--ov-held", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: [{ part: "the window", on: "no lift", target: ".ov-win[data-held]", property: "--ov-held", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: () => still({ pin: WINDOW_OVER }),
};

const ARRIVES: Move = {
  name: "A new workspace",
  what: "Dropped between two or on the slot: it grows in from `scale.behind` over `medium`.",
  tracks: [{ part: "the miniature", on: "the centre's 4", target: parts('.ov-ws[data-space="s9"]'), property: "--ov-arrive", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: "the miniature", on: "a fade", target: parts('.ov-ws[data-space="s9"]'), property: "--ov-arrive", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => still(),
};

/** Every part a swipe moves, each straight with the fingers: `long` stands in for the fingers' time. */
function swiped(opening: boolean): [Track<"linear">, ...Track<"linear">[]] {
  const [from, to] = opening ? [0, 1] : [1, 0];
  return [
    { part: "the frame and bar", on: "every screen", target: ".k-screen-glass, .k-frame, .k-bar", property: "--ov-aside", from, to, duration: "long", curve: "linear" },
    { part: "the wallpaper's blur and dim", on: "every screen", target: ".ov-screen > .k-screen-wallpaper", property: "--ov-veil", from, to, duration: "long", curve: "linear" },
    { part: "the shown workspace", on: "its screen", target: parts('.ov-ws[data-rank="home"]'), property: "--ov-fly", from: to, to: from, duration: "long", curve: "linear" },
    { part: "every other miniature", on: "slots and names too", target: `${parts('.ov-ws:not([data-rank="home"])')}, .ov-plus, .ov-label`, property: "--ov-arrive", from, to, duration: "long", curve: "linear" },
  ];
}

const SWIPE_OPENS: Move = {
  name: "Three fingers up",
  what: "Every part as far as the fingers have gone, unstaggered; lifted past half, it settles over `medium`.",
  tracks: swiped(true),
  reduced: [{ part: "every miniature", on: "a fade", target: `${parts(".ov-ws")}, .ov-plus, .ov-label`, property: "--ov-arrive", from: 0, to: 1, duration: "long", curve: "linear" }],
  back: "cuts",
  sample: () => still({ closed: true }),
};

const SWIPE_CLOSES: Move = {
  name: "Three fingers down",
  what: "The mirror: it closes as far as the fingers go; short of half when they lift, it falls back open.",
  tracks: swiped(false),
  reduced: [{ part: "every miniature", on: "a fade", target: `${parts('.ov-ws:not([data-rank="home"])')}, .ov-plus, .ov-label`, property: "--ov-arrive", from: 1, to: 0, duration: "long", curve: "linear" }],
  back: "cuts",
  sample: () => still(),
};

const GAP_CLOSES: Move = {
  name: "A gap deleted",
  what: "The empty 3 goes into the slot; the later ones close up over `medium` as their numbers roll down over `short`.",
  tracks: [
    { part: "the gap", on: "the empty 3", target: '.ov-ws[data-space="s8"]', property: "opacity", from: 1, to: 0, duration: "medium", curve: "standard_decel" },
    { part: "the later numbers", on: "4 to 3", target: '.ov-ws[data-space="s9"] .k-roll-slot', property: "--roll", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [{ part: "the gap", on: "a fade", target: '.ov-ws[data-space="s8"]', property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => still({ fill: "gap" }),
};

/** The game's one window, carried over its `side`. */
/**
 * The window manager's tiling, as the mockup stands in for it with dwindle (`config/hypr/looks.lua`):
 * the real Overview draws each window where the window manager has it, in whatever layout is set.
 */
const overGame = (side: "left" | "right" | "top" | "bottom"): Pin => ({ kind: "window", window: "w-term", to: { kind: "space", space: "s8", at: { window: "w-game", side } } });
const onGame = (side: "left" | "right" | "top" | "bottom"): Pin => ({ kind: "placed", window: "w-term", to: { kind: "space", space: "s8", at: { window: "w-game", side } } });
/** nvim, one of three on the centre's fourth, its left side. */
const NVIM_LEFT = { kind: "space", space: "s9", at: { window: "w-nvim", side: "left" } } as const;
/** On the portrait screen: lazygit carried over btop, alone on its tall workspace. */
const BTOP_TOP = { kind: "space", space: "s5", at: { window: "w-btop", side: "top" } } as const;
/** Telegram, floating over the centre's 2, carried to the game's workspace. */
const CHAT_TO_GAME = { kind: "space", space: "s8" } as const;

const CARRIED_TILED: Move = {
  name: "A tiled window lifted",
  what: "Out of its tiling it is split with nothing: over `short` it takes a lone window's shape, the grip kept under the pointer.",
  tracks: [{ part: "its width", on: "the terminal", target: '[data-window="w-term"]', property: "--ov-tw", from: { length: "size-app_min_width" }, to: { length: "size-app_width" }, duration: "short", curve: "standard" }],
  reduced: [{ part: "its width", on: "at once", target: '[data-window="w-term"]', property: "--ov-tw", from: { length: "size-app_width" }, to: { length: "size-app_width" }, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: () => still({ pin: { kind: "window", window: "w-term", to: { kind: "plus", group: "g-ext2" } } }),
};

const PLACE_FOLLOWS: Move = {
  name: "The place following the pointer",
  what: "A drop asks the window manager where it goes; here dwindle halves the window under the pointer, the dropped one taking that half.",
  tracks: [{ part: "the place it would take", on: "the game's workspace", target: ".ov-landing", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => still({ pin: overGame("left") }),
};

const DROP_SETTLES: Move = {
  name: "The drop settling",
  what: "Let go: it comes down from `scale.lift` over `short` and flies into its place over `medium`.",
  tracks: [{ part: "the lift", on: "the terminal", target: ".ov-win[data-held]", property: "--ov-held", from: 1, to: 0, duration: "short", curve: "standard" }],
  reduced: [{ part: "the lift", on: "none", target: ".ov-win[data-held]", property: "--ov-held", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: () => still({ pin: overGame("right") }),
};

const TIP_LEAVES: Move = {
  name: "The tip leaving",
  what: "Both drags made: as the second ends, it sinks `space.s16` and fades over `medium`, for good.",
  tracks: [
    { part: "its sink", on: "the tip", target: ".ov-tip", property: "--ov-sink", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
    { part: "its fade", on: "the tip", target: ".ov-tip", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
  ],
  reduced: [{ part: "its fade", on: "the tip", target: ".ov-tip", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => still({ learned: ["window"] }),
};

const SLOT_REDDENS: Move = {
  name: "The slot reddening",
  what: "Pulled in and held: the glass turns solid Danger over `short`; let go, it calms to red glass.",
  tracks: [{ part: "the solid red", on: "the slot", target: ".ov-bin", property: "--ov-solid", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => still({ pin: { kind: "binning", what: TERMINAL, over: false } }),
};

const SLOT_ARRIVES: Move = {
  name: "The delete slot arriving",
  what: "Lifted: it rises `space.s24` into the corner and fades in over `medium`; it leaves over `short`.",
  tracks: [{ part: "the slot", on: "a carry", target: ".ov-bin", property: "opacity", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: "the slot", on: "a fade", target: ".ov-bin", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "plays",
  sample: () => still({ pin: { kind: "binning", what: TERMINAL, over: false } }),
};

const PULLED: Move = {
  name: "Pulled into the slot",
  what: "In the reach it leaves the pointer and flies into the slot over `medium`; out of it, back to the pointer the same way.",
  tracks: [{ part: "the carried thing", on: "the terminal", target: '[data-window="w-term"]', property: "--ov-shrink", from: 1, to: 0.25, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: "the carried thing", on: "there at once", target: '[data-window="w-term"]', property: "--ov-shrink", from: 0.25, to: 0.25, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: () => still({ pin: { kind: "binning", what: TERMINAL, over: false } }),
};

const TIMER: Move = {
  name: "Waiting in the slot",
  what: "Let go in the reach, it stays as it sat; the bar runs down over `undo` on `linear`, reduced motion too.",
  tracks: [{ part: "the bar", on: "the slot's foot", target: ".ov-bin", property: "--hold", from: 1, to: 0, duration: "undo", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => still({ pin: { kind: "pending", what: TERMINAL, left: 1 } }),
};

const TIMES_OUT: Move = {
  name: "The time running out",
  what: "Deleted: it shrinks to nothing and fades where it is, over `medium`, as the symbol comes back.",
  tracks: [
    { part: "the thing", on: "the terminal", target: '[data-window="w-term"]', property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the symbol", on: "the slot", target: ".ov-slot-symbol", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [{ part: "the thing", on: "gone at once", target: '[data-window="w-term"]', property: "opacity", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: () => still({ pin: { kind: "pending", what: TERMINAL, left: 0 } }),
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
    { name: "Closed", for: "the desktop it opens from", sample: at({ closed: true }) },
    { name: "Over the game's left", for: "the terminal would take its left half", sample: at({ pin: overGame("left") }) },
    { name: "Over the game's right", for: "its right half", sample: at({ pin: overGame("right") }) },
    { name: "Over btop, on the portrait", for: "a tall window halves top and bottom", sample: at({ focus: "portrait", pin: { kind: "window", window: "w-git", to: BTOP_TOP } }) },
    { name: "Dropped on the left", for: "the terminal left, the game right", sample: at({ pin: onGame("left") }) },
    { name: "Dropped on the right", for: "the game left, the terminal right", sample: at({ pin: onGame("right") }) },
    { name: "Dropped over btop", for: "lazygit above btop, on the portrait", sample: at({ focus: "portrait", pin: { kind: "placed", window: "w-git", to: BTOP_TOP } }) },
    { name: "A tiled window, carried", for: "a lone window's shape, its tiling closed up", sample: at({ pin: { kind: "window", window: "w-term", to: { kind: "plus", group: "g-ext2" } } }) },
    { name: "A floating window", for: "Telegram over the centre's 2, above its tiles", sample: at({ pin: { kind: "lit", space: "s7" } }) },
    { name: "A floating window, carried", for: "its own size and shape, over the game", sample: at({ pin: { kind: "window", window: "w-chat", to: CHAT_TO_GAME } }) },
    { name: "A floating window, dropped", for: "floating on the game, at the same shares", sample: at({ pin: { kind: "placed", window: "w-chat", to: CHAT_TO_GAME } }) },
    { name: "Over one of three", for: "nvim's left: it gives up half, the rest stay", sample: at({ pin: { kind: "window", window: "w-term", to: NVIM_LEFT } }) },
    { name: "Dropped beside nvim", for: "the terminal where nvim's left half was", sample: at({ pin: { kind: "placed", window: "w-term", to: NVIM_LEFT } }) },
    { name: "Over an empty workspace", for: "no window there: it takes the whole", sample: at({ fill: "gap", pin: { kind: "window", window: "w-term", to: { kind: "space", space: "s8" } } }) },
    { name: "Dropped on the empty one", for: "the terminal fills the empty 3", sample: at({ fill: "gap", pin: { kind: "placed", window: "w-term", to: { kind: "space", space: "s8" } } }) },
    { name: "Three fingers up, half way", for: "arriving as far as the swipe has gone", sample: at({ closed: true, pin: { kind: "swiping", opening: true, progress: 0.5 } }) },
    { name: "Three fingers down, half way", for: "closing as far as the swipe has gone", sample: at({ pin: { kind: "swiping", opening: false, progress: 0.5 } }) },
    { name: "1, 2, empty 3, 4", for: "a gap: 3 holds its place, empty", sample: at({ fill: "gap" }) },
    { name: "The empty 3, deleted", for: "right-dragged into the slot", sample: at({ fill: "gap", pin: { kind: "binning", what: { space: "s8" }, over: true } }) },
    { name: "After it: 1, 2, 3", for: "4 rolled to 3; dragged back out, the gap returns", sample: at({ fill: "gap", pin: { kind: "pending", what: { space: "s8" }, left: 0.9 } }) },
    { name: "The delete slot, carrying", for: "on its way: the line full red, the glass warmed", sample: at({ pin: { kind: "binning", what: TERMINAL, over: false } }) },
    { name: "Over the delete slot", for: "pulled in and held: solid red; let go, it waits", sample: at({ pin: { kind: "binning", what: TERMINAL, over: true } }) },
    { name: "A workspace over the slot", for: "right-dragged: its miniature held in the slot", sample: at({ pin: { kind: "binning", what: GAME, over: true } }) },
    { name: "Waiting to close", for: "red glass, three fifths of its time left", sample: at({ pin: { kind: "pending", what: TERMINAL, left: 0.6 } }) },
    { name: "A second one over it", for: "held while one waits: solid red, both in the slot", sample: at({ pin: { kind: "pending", what: TERMINAL, left: 0.4, next: GAME } }) },
    { name: "The second dropped", for: "the first deleted, the second waiting afresh", sample: at({ pin: { kind: "pending", what: GAME, left: 1, confirmed: TERMINAL } }) },
    { name: "Taken back out", for: "dragged out of the slot: carried again, its timer gone", sample: at({ pin: { kind: "picked", what: TERMINAL } }) },
    { name: "The tip, shown", for: "nothing learned yet: the bottom left corner, 1:1", sample: corner({}, "left", 720, 160) },
    { name: "The tip, one drag learned", for: "a window dragged: it stays, all of it", sample: corner({ learned: ["window"] }, "left", 720, 160) },
    { name: "The tip, both learned", for: "both drags made: gone for good", sample: corner({ learned: ["window", "space"] }, "left", 720, 160) },
    { name: "The slot's reach", for: "dashed: let go anywhere inside, it waits in the slot", sample: at({ reach: true, pin: { kind: "binning", what: TERMINAL, over: false } }) },
    { name: "Just inside the reach", for: "a window, the pointer far off: already in the slot", sample: at({ reach: true, pin: { kind: "binning", what: TERMINAL, over: "edge" } }) },
    { name: "A workspace just inside", for: "right-dragged, the pointer far off: held in the slot", sample: at({ reach: true, pin: { kind: "binning", what: GAME, over: "edge" } }) },
    { name: "Carrying, at true size", for: "the empty slot, 1:1", sample: corner({ pin: { kind: "binning", what: TERMINAL, over: false } }, "right", 720, 400) },
    { name: "Over the slot, at true size", for: "pulled in and lit, 1:1", sample: corner({ pin: { kind: "binning", what: TERMINAL, over: true } }, "right", 720, 400) },
    { name: "Waiting, at true size", for: "the slot's timer, 1:1", sample: corner({ pin: { kind: "pending", what: TERMINAL, left: 0.6 } }, "right", 720, 400) },
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
      rest: "each window where the window manager has it, any layout",
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
      { name: "The delete slot, held in", tone: "burn_error", filled: true, content: false },
    ],
    looks: { lit: "the tube round the miniature, its glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: () => still({ pin: { kind: "lit", space: "s7" } }),
  },
  content: {
    text: [{ name: "A waiting group's name", words: GROUP_NAME, sample: (words: string) => still({ desk: "laptop", names: { "g-studio": words } }) }],
    missingPicture: doesntApply("a window is its live picture, never an icon in its place."),
    counts: [
      { name: "None", for: "one empty workspace, its slot", sample: at({ desk: "laptop", fill: "empty" }) },
      { name: "Nine on one", for: "the centre's 1, tiled small", sample: at({ fill: "crowded" }) },
      { name: "A full group", for: "ten in use: no slot", sample: at({ fill: "full" }) },
    ],
  },
  motion: {
    arrives: [OPENING, SWIPE_OPENS],
    leaves: [CLOSING, SWIPE_CLOSES],
    between: [LIFT, CARRIED_TILED, PLACE_FOLLOWS, DROP_SETTLES, ARRIVES, GAP_CLOSES, TIP_LEAVES, SLOT_ARRIVES, PULLED, SLOT_REDDENS, TIMER, TIMES_OUT],
  },
  keyboard: {
    keys: [
      { keys: ["Super+W"], does: "opens it; again, or Esc, closes" },
      { keys: ["Three fingers up"], does: "opens it as far as they go; past half it stays" },
      { keys: ["Three fingers down"], does: "closes it the same way: past half it goes" },
      { keys: ["Pointer over one"], does: "the open hand: a window or workspace can be carried" },
      { keys: ["Pointer, carrying"], does: "the closed hand, until it is let go" },
      { keys: ["Left button"], does: "on a window, dragged: carries the window" },
      { keys: ["Right button"], does: "on a workspace, dragged: carries the workspace" },
      { keys: ["Left button"], does: "pressed: goes there" },
      { keys: ["Either button"], does: "on the delete slot, dragged: takes it back out" },
      { keys: ["←", "→", "↑", "↓"], does: "the next workspace that way, across screens" },
      { keys: ["Shift+←", "Shift+→", "Shift+↑", "Shift+↓"], does: "carries its focused window that way" },
      { keys: ["Enter"], does: "goes there: it grows into the screen" },
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
      { part: "Number's row", token: "size-control_sm" },
      { part: "Number", token: "size-chip" },
      { part: "Name over a section", token: "size-chip" },
      { part: "Miniature corners", token: "radius-control" },
      { part: "Window corners, on the desktop", token: "radius-surface" },
      { part: "Window corners, in a miniature", token: "radius-control" },
      { part: "Window gaps", token: "space-s8" },
      { part: "A floating window's shadow", token: "shadow-panel-blur" },
      { part: "Wallpaper blur", token: "size-lock_blur" },
      { part: "Slit between two", token: "space-s4" },
      { part: "Held thing, its height", token: "size-drop_slot" },
      { part: "Gap round a held thing", token: "space-s16" },
      { part: "Slot in from the corner", token: "space-s48" },
      { part: "Slot's timer", token: "size-progress" },
      { part: "Slot warmed, carrying", token: "opacity-tint" },
      { part: "Slot's red glass, waiting", token: "opacity-error_fill" },
      { part: "Drop reach, of the shorter side", token: "scale-drop_reach" },
    ],
    reflow: { note: "Its own group up to a 3 × 3 grid's size; waiting groups at half; rows centred.", sample: at({ desk: "laptop" }) },
  },
});
