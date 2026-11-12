// The power menu, "Leaving the room" (`power-room.ts` draws it): the desktop steps back, a sheet of
// five tiles rises from the frame's bottom edge, and the action you're on plays out on the desktop
// before you commit. It opens from the control center's Power, the bar having none.

import type { Words } from "../../lib/content.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { controlCenter, onScreen } from "./control-center-parts.ts";
import {
  ACTIONS,
  blocking,
  CALM,
  closing,
  closingReduced,
  cycling,
  goingDown,
  goingDownReduced,
  goingDownUnflickered,
  holding,
  LOCK,
  locking,
  LOGOUT,
  loggingOut,
  lettingGo,
  opening,
  openingReduced,
  previewing,
  RESTART,
  room,
  SHUTDOWN,
  SUSPEND,
  suspending,
} from "./power-room.ts";

const UNSAVED = { unsaved: true, update: true };

const open = (at: number, more: Parameters<typeof room>[0] = {}) => () => room({ open: true, at, ...more });

/** A room wrapped, so a move's player reaches its root: the frame's and the lights' numbers are set there. */
const held = (made: () => HTMLElement) => () => {
  const box = document.createElement("span");
  box.className = "power-held";
  box.append(made());
  return box;
};

const OPENING: Move = {
  name: "Opening",
  what: "The frame closes in, the picture pushes in, the windows step back, the sheet rises: all over `long`.",
  tracks: opening(),
  reduced: openingReduced(),
  back: "cuts",
  sample: held(() => room()),
};

const CLOSING: Move = {
  name: "Closing",
  what: "Esc or a press outside: the sheet sinks and the frame opens out over `medium`; the desktop comes forward.",
  tracks: closing(),
  reduced: closingReduced(),
  back: "cuts",
  sample: held(open(LOCK, { played: false })),
};

/** An action's look, played from the desktop round the open sheet. */
function look(index: number, what: string, situation = CALM): Move {
  return {
    name: `${ACTIONS[index]!.name}, looked at${situation.unsaved ? ", unsaved" : ""}`,
    what,
    tracks: previewing(index, situation),
    reduced: previewing(index, situation, true),
    back: "cuts",
    sample: held(open(index, { played: false, situation })),
  };
}

const LOOKS: Move[] = [
  look(LOCK, "The lock as it is awake: windows gone, bar up, the clock lifted out of it, the picture blurred."),
  look(SUSPEND, "The lights go down to a dialog's `scrim` over `long`."),
  look(LOGOUT, "The windows leave a `micro` apart, dropping `space.s64` as they fade over `medium`."),
  look(LOGOUT, "The windows that won't quit stay, their `warning` tube lit and why on a chip.", UNSAVED),
  look(SHUTDOWN, "The windows and the bar go; the lights go down over `slow`, the sheet's last."),
];

const CYCLE: Move = {
  name: "Restart, looked at",
  what: "The lights cycle: down over `medium`, a `long` held, back up; again every two `scene`s and a `long`.",
  tracks: cycling(),
  reduced: [{ part: "nothing", on: "the room", target: ".power-room", property: "--power-dark", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: held(open(RESTART)),
};

const HOLDS: Move = {
  name: "Holding Shut down",
  what: "The tile runs white-hot, its tint fills it over `hold`, and the room darkens with it.",
  tracks: holding(SHUTDOWN),
  reduced: "same",
  back: "cuts",
  sample: held(open(SHUTDOWN)),
};

const LETS_GO: Move = {
  name: "Letting go early",
  what: "The fill drains over `short`; the room comes back to the look.",
  tracks: lettingGo(SHUTDOWN),
  reduced: "same",
  back: "cuts",
  sample: held(open(SHUTDOWN, { held: 0.5 })),
};

const BLOCKS: Move = {
  name: "Apps that won't quit",
  what: "Held with unsaved work: the blocker rises `space.s24` into the sheet over `long`; the sheet grows over `medium`.",
  tracks: blocking(),
  reduced: blocking(true),
  back: "cuts",
  sample: held(open(SHUTDOWN, { blocked: true, situation: UNSAVED })),
};

const GOES_DOWN: Move = {
  name: "Shut down or Restart, held",
  what: "Dark over `long` with the sign lit over it; a `slow` held; a group out every `short`, ク last.",
  tracks: goingDown(SHUTDOWN),
  reduced: goingDownReduced(SHUTDOWN),
  unflickered: goingDownUnflickered(SHUTDOWN),
  back: "cuts",
  sample: held(open(SHUTDOWN, { held: 1 })),
};

const LOGS_OUT: Move = {
  name: "Log out, held",
  what: "The windows, the sheet, the frame and the bar go as the room goes dark, over `medium`.",
  tracks: loggingOut(),
  reduced: loggingOut(true),
  back: "cuts",
  sample: held(open(LOGOUT, { held: 1 })),
};

const SLEEPS: Move = {
  name: "Suspend",
  what: "The sheet fades and the lights go down to black over `slow`.",
  tracks: suspending(),
  reduced: suspending(true),
  back: "cuts",
  sample: held(open(SUSPEND)),
};

const LOCKS: Move = {
  name: "Lock",
  what: "From its look: the sheet sinks, the frame slides out, the clock goes home behind the city.",
  tracks: locking(),
  reduced: locking(true),
  back: "cuts",
  sample: held(open(LOCK)),
};

type Shown = "rest" | "lit" | "pressed";

const ROWS = [
  { name: "Lock", at: LOCK },
  { name: "Log out, held", at: LOGOUT },
  { name: "Shut down, held", at: SHUTDOWN },
] as const;

function cell(at: number, state: Shown): HTMLElement {
  if (state === "rest") return room({ open: true, at: at === LOCK ? SUSPEND : LOCK, played: false });
  if (state === "lit") return room({ open: true, at });
  return ACTIONS[at]!.held ? room({ open: true, at, held: 0.5 }) : room({ open: true, at, pinned: "pressed" });
}

const LEAD: Words = { short: "Closes everything and turns the lights off.", long: "Off, update, back on. Takes about 40 s." };
const NAME: Words = { short: "Shut down", long: "Shut down now" };

export default page<Shown>({
  level: "shell",
  slug: "power-menu",
  title: "Power menu",
  lede: "Leaving the room: five tiles on a sheet at the bottom edge, each action played on the desktop first.",
  code: { unbuilt: "QML PowerMenu: old design" },
  from: ["lock", "panel-page", "tooltip", "chip", "banner"],
  stage: { live: () => room({ live: true }), hint: "click to open; arrows move; hold Enter", daylight: true },
  variants: [
    { name: "Opened", for: "the keyboard on Lock; nothing played yet", sample: open(LOCK, { played: false }) },
    { name: "Lock, looked at", for: "the lock awake round the sheet", sample: open(LOCK) },
    { name: "Suspend, looked at", for: "the lights down to `scrim`", sample: open(SUSPEND) },
    { name: "Log out, looked at", for: "the windows gone", sample: open(LOGOUT) },
    { name: "Restart, looked at", for: "the lights mid-cycle, an update waiting", sample: open(RESTART) },
    { name: "Restart, nothing waiting", for: "the line and facts with no update", sample: open(RESTART, { situation: { unsaved: false, update: false } }) },
    { name: "Shut down, looked at", for: "windows and bar gone, lights down", sample: open(SHUTDOWN) },
    { name: "A tile's tooltip", for: "its name, held or not, and its letter", sample: open(SHUTDOWN, { tip: true }) },
    { name: "Unsaved work, live", for: "try holding Log out, Restart or Shut down", sample: () => room({ live: true, situation: UNSAVED }) },
    { name: "Unsaved work, looked at", for: "the windows that won't quit stay lit", sample: open(LOGOUT, { situation: UNSAVED }) },
    { name: "Apps that won't quit", for: "held to the end with unsaved work", sample: open(SHUTDOWN, { blocked: true, situation: UNSAVED }) },
    { name: "Gone dark, the sign lit", for: "the room black, the splash's sign burning", sample: () => room({ dark: { sign: 0 } }) },
    { name: "Going out", for: "OS and Kuba out", sample: () => room({ dark: { sign: 2 } }) },
    { name: "ク last", for: "only ク still lit", sample: () => room({ dark: { sign: 4 } }) },
    { name: "Black", for: "after the glass, and after Log out or Suspend", sample: () => room({ dark: "black" }) },
    { name: "Locked", for: "Lock taken: the lock at rest", sample: () => room({ locked: true }) },
    { name: "Its way in", for: "the control center's Power, lit", sample: () => onScreen(controlCenter({ powerLit: "lit" })) },
  ],
  sizes: doesntApply("one size: five `size.power_tile` tiles, the sheet as wide as they are."),
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "every action is always there",
      selected: "the keyboard's tile is lit, not picked",
      loading: "a held tile's fill is its progress",
      empty: "always five tiles",
      error: "apps that won't quit: see Variants",
    },
    rows: nonEmpty(ROWS.map(({ name, at }) => ({ name, cell: (state: Shown) => cell(at, state) }))),
    looks: {
      rest: "another tile current; its symbol muted",
      lit: "its tube and glow; the desktop plays it",
      pressed: "white-hot; held ones fill with the tint",
    },
  },
  focus: {
    tubes: [
      { name: "A tile", tone: "accent", filled: false, content: false },
      { name: "A window that won't quit", tone: "warning", filled: false, content: false },
      { name: "Quit anyway", tone: "burn_error", filled: true, content: false },
    ],
    looks: { lit: "the tube inside, glow outside; the symbol in the accent", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => room({ open: true, at: ROWS[index]!.at, pinned: "signs-off" }),
  },
  content: {
    text: [
      { name: "The line over the tiles", words: LEAD, sample: (words: string) => room({ open: true, at: SHUTDOWN, words }) },
      { name: "A tile's name", words: NAME, sample: (words: string) => room({ open: true, at: SHUTDOWN, names: ACTIONS.map((each, i) => (i === SHUTDOWN ? words : each.name)) }) },
    ],
    missingPicture: doesntApply("every symbol is the font's; the sign's layers ship with the splash."),
    counts: doesntApply("always five tiles; each action has its own facts."),
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING, LOCKS, SLEEPS, LOGS_OUT, GOES_DOWN],
    between: [
      { name: "Lit", what: "The keyboard or pointer onto a tile: it lights over `short`, and its action plays.", tracks: lights(".power-tile[data-action=\"suspend\"]", "a tile", 0, 1), reduced: "same", back: "cuts", sample: held(open(LOCK, { played: false })) },
      ...LOOKS,
      CYCLE,
      HOLDS,
      LETS_GO,
      BLOCKS,
      ...powerMoves(held(open(SHUTDOWN))),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Super+Escape", "XF86PowerOff"], does: "opens it; Esc closes it" },
      { keys: ["←", "→"], does: "the last or next tile, round; it plays" },
      { keys: ["Tab", "Shift+Tab"], does: "the same, the next or the last" },
      { keys: ["L", "S"], does: "Lock, Suspend: at once" },
      { keys: ["O", "R", "D"], does: "Log out, Restart, Shut down: held" },
      { keys: ["Enter", "Space"], does: "the tile's action; held for the last three" },
      { keys: ["Esc"], does: "closes it; in the blocker, back to the tiles" },
    ],
    order: "One stop: every key moves along the five tiles, round; the blocker's buttons are a round of their own.",
    lands: "Opening, on Lock once the sheet is up; the blocker, on its first Show; closing, back where it was.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Shut down, looked at", for: "the frame, the sheet, the tiles", sample: open(SHUTDOWN) },
      { name: "Apps that won't quit", for: "the blocker in the sheet", sample: open(SHUTDOWN, { blocked: true, situation: UNSAVED }) },
      { name: "Gone dark, the sign lit", for: "the sign's place", sample: () => room({ dark: { sign: 0 } }) },
    ]),
    measures: [
      { part: "Frame, closed in", token: "space-s32" },
      { part: "Sheet's padding", token: "space-s16" },
      { part: "Blocker, line, tiles apart", token: "space-s16" },
      { part: "Line to its facts", token: "space-s8" },
      { part: "Line's inset", token: "space-s4" },
      { part: "Facts apart", token: "space-s8" },
      { part: "Fact", token: "size-chip" },
      { part: "Tile", token: "size-power_tile" },
      { part: "Tile's padding", token: "space-s16" },
      { part: "Symbol to name", token: "space-s4" },
      { part: "Symbol", token: "icon-s24" },
      { part: "Tiles apart", token: "space-s8" },
      { part: "Tile's corner", token: "radius-control" },
      { part: "Sheet's corners, fillets", token: "radius-surface" },
      { part: "A leaving window's drop", token: "space-s64" },
      { part: "The blocker's rise", token: "space-s24" },
      { part: "Room under the sign", token: "size-control_md" },
    ],
    reflow: doesntApply("the sheet keeps its size, centred on the bottom edge, on any screen."),
  },
});
