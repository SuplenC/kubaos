// The Sheet and PanelPage page: `docs/design/components/sheet.md` and `panel-page.md`. A shell panel
// growing out of the screen frame, its twins taking the surface over from one another, and the
// template inside it: the header band with its pun, the tabs, the body that alone scrolls under its
// fade, and the footer band.

import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { powerMoves } from "../../site/moves.ts";
import { ALL, windowSize } from "../../site/settings.ts";
import {
  clockOnScreen,
  controlCenter,
  liveTwins,
  notifications,
  onScreen,
  PUNS,
  swapSample,
  system,
  TITLES,
} from "./panels.ts";

type Shown = "rest" | "empty" | "loading" | "error";

const SHEET = "the sheet";
const SCRIM = "the desktop's dim";
const LEAVING = ".k-sheet-rider > [data-leaving]";
const ARRIVING = ".k-sheet-rider > :not([data-leaving])";

const OPENING: Move = {
  name: "Opening",
  what: "A pill or its key: the sheet grows out of the frame's edge, its content riding on it, and the desktop dims.",
  tracks: [
    { part: SHEET, on: "the system monitor", target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: SCRIM, on: "the system monitor", target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: SHEET, on: "the system monitor", target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: SCRIM, on: "the system monitor", target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => onScreen(system()),
};

const CLOSING: Move = {
  name: "Closing",
  what: "Esc, a press outside, or its pill: the sheet goes back into the edge and the dim lifts.",
  tracks: [
    { part: SHEET, on: "the system monitor", target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: SCRIM, on: "the system monitor", target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: SHEET, on: "the system monitor", target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: SCRIM, on: "the system monitor", target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => onScreen(system()),
};

const SWAP: Move = {
  name: "A twin takes over",
  what: "Another pill while one is open: the surface stays, and only the content slides over, the two never drawn together.",
  tracks: [
    { part: "the leaving content's slide", on: "the system monitor", target: LEAVING, property: "--swap-shift", from: 0, to: -1, duration: "micro", curve: "emphasized_accel" },
    { part: "the leaving content's fade", on: "the system monitor", target: LEAVING, property: "--swap-presence", from: 1, to: 0, duration: "micro", curve: "standard_accel" },
    { part: "the arriving content's slide", on: "the notifications", target: ARRIVING, property: "--swap-shift", from: 1, to: 0, duration: "short", cue: "swap_lag", curve: "emphasized_decel" },
    { part: "the arriving content's fade", on: "the notifications", target: ARRIVING, property: "--swap-presence", from: 0, to: 1, duration: "swap_fade", cue: "micro", curve: "standard" },
  ],
  reduced: [
    { part: "the leaving content's fade", on: "the system monitor", target: LEAVING, property: "--swap-presence", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "the arriving content's fade", on: "the notifications", target: ARRIVING, property: "--swap-presence", from: 0, to: 1, duration: "instant", curve: "standard" },
  ],
  back: "cuts",
  sample: swapSample,
};

const TURN_BACK: Move = {
  name: "Turning back",
  what: "Its pill again before the other shows: the content turns back from where it is.",
  tracks: [
    { part: "the content's slide", on: "the system monitor", target: ".k-sheet-rider > *", property: "--swap-shift", from: -1, to: 0, duration: "short", curve: "emphasized_decel" },
    { part: "the content's fade", on: "the system monitor", target: ".k-sheet-rider > *", property: "--swap-presence", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [{ part: "the content's fade", on: "the system monitor", target: ".k-sheet-rider > *", property: "--swap-presence", from: 0, to: 1, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: () => onScreen(system()),
};

const FADE: Move = {
  name: "The fade comes",
  what: "The body leaves its top: the header fade comes in, so nothing is cut flat.",
  tracks: [{ part: "the header fade", on: "the notifications", target: ".k-scroll", property: "--faded", from: 0, to: 1, duration: "micro", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => onScreen(notifications({ down: 0.3 })),
};

const ROWS = [
  { name: "System monitor", make: (state: Shown) => onScreen(system({ state })) },
  { name: "Notifications", make: (state: Shown) => onScreen(notifications({ state })) },
  { name: "Control center", make: (state: Shown) => onScreen(controlCenter({ state })) },
] as const;

/** The twins at the toolbar's window size: the whole screen, a panel open at its right edge. */
function reflow(): HTMLElement {
  const size = windowSize();
  return liveTwins(size.width, size.height);
}

/**
 * The stage's screen, its first twin opening as it comes, so the stage shows a panel and not only a
 * desktop. The shots and the frames draw it shut: `?all` holds what they measure still.
 */
function live(): HTMLElement {
  const screen = liveTwins("var(--size-app_min_width)", "var(--size-app_height)");
  if (!ALL) requestAnimationFrame(() => screen.querySelector<HTMLElement>(".k-pill")?.click());
  return screen;
}

export default page({
  level: "templates",
  slug: "panel-page",
  title: "Sheet and PanelPage",
  lede: "A shell panel growing out of the screen frame, and the template inside it: header, tabs, body, footer.",
  code: { rust: "sheet::sheet", qml: "Sheet", unbuilt: "QML PanelPage: not built yet" },
  stage: {
    live,
    hint: "press another pill to swap; Esc closes",
    daylight: true,
  },
  variants: [
    { name: "No tabs, no footer", for: "a header over one body: the system monitor", sample: () => onScreen(system()) },
    { name: "With a footer", for: "an action under the body", sample: () => onScreen(system({ footer: true })) },
    { name: "A status for a pun", for: "the notifications: their count, never shifting", sample: () => onScreen(notifications()) },
    { name: "With tabs", for: "views under the header: the control center", sample: () => onScreen(controlCenter()) },
    { name: "Sized to its content", for: "hangs from the top edge's middle: the clock", sample: clockOnScreen },
  ],
  sizes: [
    { name: "As tall as its content", for: "ends in a fillet into the right edge", sample: () => onScreen(system({ count: 2 })) },
    { name: "Up to the room", for: "meets the frame's bottom edge; the body scrolls", sample: () => onScreen(notifications({ count: 8 })) },
    { name: "Up to the room, scrolled", for: "the header fade on; the bands stay", sample: () => onScreen(notifications({ count: 8, down: 0.5, fadePinned: true })) },
  ],
  states: {
    shown: ["rest", "empty", "loading", "error"],
    absent: {
      lit: "its buttons, tabs and rows light on their pages",
      pressed: "a press inside does nothing; outside closes it",
      disabled: "open or gone; its controls disable themselves",
      selected: "its tabs and rows show theirs",
    },
    rows: nonEmpty(ROWS.map(({ name, make }) => ({ name, cell: (state: Shown) => make(state) }))),
    looks: {
      rest: "`surface` at the frame's glass, the pun `on_surface_variant`",
      empty: "an `EmptyState`; the header keeps its pun",
      loading: "the rows as skeletons; nothing shifts",
      error: "an `ErrorBlock` in the body; the pun stays",
    },
  },
  focus: doesntApply("no line of its own; the panel's edge cuts its controls' glow."),
  content: {
    text: [
      { name: "The title", words: TITLES.notifications, sample: (words: string) => onScreen(notifications({ title: words })) },
      { name: "The pun", words: PUNS.notifications, sample: (words: string) => onScreen(notifications({ subtitle: words })) },
      { name: "A pun, with no buttons", words: PUNS.system, sample: (words: string) => onScreen(system({ subtitle: words, bare: true })) },
    ],
    missingPicture: doesntApply("it draws no picture; a row missing its icon keeps its tile."),
    counts: [
      { name: "None", for: "its empty state; the header doesn't move", sample: () => onScreen(notifications({ state: "empty" })) },
      { name: "One", for: "as short as a sheet with a body gets", sample: () => onScreen(system({ count: 1 })) },
      { name: "Many", for: "the body scrolls, the bands stay", sample: () => onScreen(notifications({ count: 8, down: 1, fadePinned: true })) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [SWAP, TURN_BACK, FADE, ...powerMoves(() => onScreen(notifications({ lit: true })))],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "round the panel in reading order, never out of it" },
      { keys: ["←", "→", "h", "l"], does: "the next or previous tab, round" },
      { keys: ["1", "2", "3", "4"], does: "the control center: straight to that tab" },
      { keys: ["↑", "↓", "j", "k"], does: "the body's list; past its ends, nothing" },
      { keys: ["Enter", "Space"], does: "presses what the keyboard is on" },
      { keys: ["Esc"], does: "back a step in the body, then closes" },
    ],
    order: "The header's buttons, tabs, body, footer, round; the bar's pills are never stops.",
    lands: "Opening, on the panel's first thing to do; closing, back on the window that had it.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The system monitor", for: "width, padding, the header band", sample: () => onScreen(system()) },
      { name: "The control center", for: "the tabs and the body under them", sample: () => onScreen(controlCenter()) },
      { name: "The system monitor, footer", for: "the footer band", sample: () => onScreen(system({ footer: true })) },
    ]),
    measures: [
      { part: "Panel's width", token: "size-panel" },
      { part: "Padding, gaps, the header fade", token: "space-s16" },
      { part: "Title's line", token: "type-heading-line" },
      { part: "Pun's line", token: "type-small-line" },
      { part: "Header's buttons, tabs, footer action", token: "size-control_sm" },
      { part: "Between buttons; frame's sides, foot", token: "space-s8" },
      { part: "Scroll's reach past its sides", token: "focus-far" },
      { part: "Body's thumb", token: "size-scrollbar" },
      { part: "Bar, the frame's top", token: "size-bar" },
      { part: "Fillets, far corners", token: "radius-surface" },
      { part: "Shadow's drop", token: "shadow-panel-y" },
      { part: "Shadow's blur", token: "shadow-panel-blur" },
      { part: "A twin's slide", token: "space-s24" },
    ],
    reflow: {
      note: "The panel keeps `size.panel`; only its height follows the screen, then the body scrolls.",
      sample: reflow,
    },
  },
});
