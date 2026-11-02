// The Sheet and PanelPage page: `docs/design/components/sheet.md` and `panel-page.md`. A shell panel
// growing out of the screen frame, its twins taking the surface over from one another, and the
// template inside it: the header band with its pun, the tabs, the body that alone scrolls under its
// fade, and the footer band.

import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { powerMoves } from "../../site/moves.ts";
import { windowSize } from "../../site/settings.ts";
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
  what: "A panel's button or its key: the sheet grows out of the frame's edge over `long` on `emphasized_decel`, its depth `--reveal` of its content's, the fillets and the far corners growing with it, so it comes out of a flat edge. The content rides on the sheet's far edge, sliding out from under the frame, and fades in a beat after the surface moves: its opacity is `--reveal` from a fifth of the way to three quarters. The desktop dims to `opacity.panel_scrim` over `medium` on `standard`. Nothing flickers: a panel isn't a light. With reduce motion the sheet is at its size at once and fades in over `short`, with the dim.",
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
  what: "Esc, a press outside it, or its button again: the sheet goes back into the edge over `medium` on `emphasized_accel`, its content fading out before it has gone, and the dim lifts over `medium` on `standard`. With reduce motion it fades out where it is over `short`.",
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
  what: "The notifications' button while the system monitor is open: the surface stays where it is and only the content changes. The system monitor's content slides `space.s24` away from the bell, to the left, over `micro` on `emphasized_accel`, fading over `micro` on `standard_accel`; the notifications' slides in from the bell's side, `space.s24` to the right, over `short` on `emphasized_decel`, starting `swap_lag` later, and fades in over `swap_fade` on `standard` once the other is gone, so the two are never drawn over each other: 250 ms in all. Where the twins differ in size the sheet settles into the new one over `medium` on `standard_decel`. With reduce motion the content swaps at once.",
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
  what: "The system monitor's button again before the notifications' content has begun to show: the system monitor's content turns back from wherever it is, here from as far as it goes, `space.s24` out and gone, to its place over `short` on `emphasized_decel`, fading back in over `short` on `standard`. The notifications' content, not yet shown, goes at once. A third twin asked for mid-swap takes over from the one arriving instead, and the one still leaving goes at once.",
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
  what: "The body's content leaves its top: the header fade over the gap under the header (under the tabs where there are some) fades in over `micro` on `standard`, so what passes under the header fades into the panel rather than being cut flat; back at the top it fades out the same way, so a focused first row keeps its whole glow.",
  tracks: [{ part: "the header fade", on: "the notifications", target: ".k-header-fade", property: "--faded", from: 0, to: 1, duration: "micro", curve: "standard" }],
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

export default page({
  level: "templates",
  slug: "panel-page",
  title: "Sheet and PanelPage",
  summary: "A shell panel growing out of the screen frame, its twins taking the surface over, and the template inside it: a header with a pun, tabs, a body that alone scrolls, and a footer.",
  live: () => liveTwins("var(--size-app_min_width)", "var(--size-app_height)"),
  description: [
    "Rust `sheet::sheet`, QML `Sheet`; PanelPage is QML's alone, as only the shell has panels. Press the pills at the bar's right end: each opens its panel, the system monitor, the notifications and the control center, in that order along the bar. With one open, another takes the surface over; its own again, Esc or a press on the desktop closes it.",
    "A Sheet grows out of an inner edge of the screen frame, and the two are one surface: `surface` at the frame's glass, `opacity.frame` (0.94), in one layer, so where they meet nothing doubles up, and the layer casts `shadow.panel` (16 down, 48 blur) once over the desktop. Where the sheet meets the edge it curves into it through a concave fillet of the surface radius (`radius.surface`, 24), the frame's own inner corner, and its far corners take the surface radius. The right-edge panels hang from the bar: their top curves into it beyond their far side; a panel as tall as the frame's room meets its bottom edge the same way, and a shorter one ends in a fillet into the right edge under it. The clock hangs from the middle of the top edge and the wallpaper picker stands on the middle of the bottom one, a fillet into the edge at either end. While a panel is open the desktop under the frame dims to `scrim` at `opacity.panel_scrim`, lighter than a dialog's: a panel sits beside the windows, not over a question.",
    "Sheets in the same place are twins: the system monitor, the notifications and the control center. One asked for while another is open takes the surface over where it is, and only the content changes (Motion, A twin takes over).",
    "PanelPage is the inside of a Sheet, from the top: the header band, the tabs if it has them, the body, and the footer band if it has one. The panel is `space.s16` inside at its top and its foot, its parts are `space.s16` apart, and each takes the container's padding at its sides, 16 at the panel's 384 (`Container`). Only the body scrolls; the bands and the tabs never do.",
    "The header band is the title in `heading` over its subtitle line in `small`, `on_surface_variant`, one line each, eliding at the end where they run long; the panel's icon buttons sit at its end, Text and sm, `space.s8` apart and `space.s8` from the title's room, centred on the two lines. The subtitle line is always there and always holds a pun in the panel's own voice, never a count or a status, which stay in the body; each panel has lines of its own, which may rotate. So the header never shifts: not while a panel is open, not when twins swap, not when a count goes to zero.",
    "The tabs sit right under the header, across the container's width. The body is a `Scroll` reaching past its box into the room around it, so a resting control at its edge keeps its whole glow and the panel's own edge cuts what passes it; its thumb leaves `space.s16` at the content's end. A `HeaderFade` lies over the gap above it and shows only once content is behind it. The footer band holds the panel's footer action at its end, a Tonal sm button (Mark all read); a count that would sit beside it stays in the body.",
    "A panel is as tall as its content, up to the screen's room inside the frame: from the bar's foot to the frame's bottom edge. Past that only the body gives up height and scrolls, and the bands stay. It is `size.panel` (384) wide, or as wide as its content for the clock and the wallpaper picker, whose content has a width of its own.",
  ],
  variants: [
    { name: "No tabs, no footer", for: "A header over one body: the system monitor.", sample: () => onScreen(system()) },
    { name: "With a footer", for: "The panel's footer action at the end of a band under the body: the notifications' Mark all read.", sample: () => onScreen(notifications()) },
    { name: "With tabs", for: "Views of the panel picked under the header: the control center's Wi-Fi, Bluetooth, Output and Keyboard.", sample: () => onScreen(controlCenter()) },
    { name: "Sized to its content, on the top edge", for: "A body with a width of its own, hanging from the middle of the top edge: the clock.", sample: clockOnScreen },
  ],
  sizes: [
    { name: "As tall as its content", for: "Content shorter than the room: the sheet ends in a fillet into the right edge, the desktop under it.", sample: () => onScreen(system({ count: 2 })) },
    { name: "Up to the room, at its top", for: "Content taller than the room: the sheet meets the frame's bottom edge, the body scrolls and its thumb shows.", sample: () => onScreen(notifications({ count: 8 })) },
    { name: "Up to the room, scrolled", for: "The body scrolled: the header fade over the gap under the header, the footer band still in place.", sample: () => onScreen(notifications({ count: 8, down: 0.5, fadePinned: true })) },
  ],
  states: {
    shown: ["rest", "empty", "loading", "error"],
    absent: {
      lit: "the template isn't a control: its header's buttons, its tabs, its rows and its footer action light as their pages say.",
      pressed: "a press on the panel itself does nothing; a press outside it closes it.",
      disabled: "a panel is open or gone; what can't be used in it is disabled on its own.",
      selected: "the template holds no choice of its own; its tabs and rows show theirs.",
    },
    rows: nonEmpty(ROWS.map(({ name, make }) => ({ name, cell: (state: Shown) => make(state) }))),
    described: {
      rest: "Open, its content in its body. The surface is `surface` at the frame's glass in either mode, the title `on_surface` and the pun `on_surface_variant`. By day the same roles in daylight colours.",
      empty: "Nothing to show: an `EmptyState` in the body, its words saying so in the panel's voice. The header keeps its pun and its buttons, so nothing moves; the footer's action goes with what it acted on.",
      loading: "Still being found: the body's rows as skeletons where they will be, so nothing shifts as they arrive. The header is known before the panel opens.",
      error: "What broke and what to try, an `ErrorBlock` in the body, never in the header: the pun stays a pun.",
    },
  },
  focus: doesntApply("the template draws no line of its own; its header's buttons, its tabs, its rows and its footer action light as their pages say, and the panel's edge is the one that cuts their glow."),
  content: {
    text: [
      { name: "The title", words: TITLES.notifications, sample: (words: string) => onScreen(notifications({ title: words })) },
      { name: "The pun", words: PUNS.notifications, sample: (words: string) => onScreen(notifications({ subtitle: words })) },
      { name: "A pun, with no buttons", words: PUNS.system, sample: (words: string) => onScreen(system({ subtitle: words, bare: true })) },
    ],
    missingPicture: doesntApply("the template draws no picture; a row missing its icon keeps its tile, as the List page shows."),
    counts: [
      { name: "None", for: "Nothing in the body: its empty state, the header unmoved.", sample: () => onScreen(notifications({ state: "empty" })) },
      { name: "One", for: "One row: the sheet as short as it gets with a body.", sample: () => onScreen(notifications({ count: 1 })) },
      { name: "Many", for: "More than the room holds: the body scrolls, the bands stay.", sample: () => onScreen(notifications({ count: 8, down: 1, fadePinned: true })) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [SWAP, TURN_BACK, FADE, ...powerMoves(() => onScreen(notifications({ lit: true })))],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Through the panel in reading order: the header's buttons, the tabs (one stop, on the picked tab), the body's controls, the footer's action; round, never out of the panel while it is open." },
      { keys: ["←", "→", "h", "l"], does: "On the tabs, the next or the previous tab, round from either end." },
      { keys: ["1", "2", "3", "4"], does: "The control center: straight to that tab." },
      { keys: ["↑", "↓", "j", "k"], does: "The body's list, as the List page says; past its first or last row, nothing." },
      { keys: ["Enter", "Space"], does: "Presses what the keyboard is on." },
      { keys: ["Esc"], does: "Goes back a step in the body first (an unfolded card folds, a search clears); then closes the panel." },
    ],
    order: "The header's buttons, the tabs, the body, the footer, round. The bar's pills are never stops: a panel's keyboard never wanders into the bar. Nothing behind the panel takes the keyboard while it is open.",
    lands: "Opening: on the panel's first thing to do, the control center's first tab, the notifications' first row, the clock's search, else the body. A twin taking over lands on its own first thing. Closing: back on the window that had it.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The system monitor", for: "The panel's width, its padding at its top and foot and sides, the header band and the gap to the body.", sample: () => onScreen(system()) },
      { name: "The control center", for: "The tabs under the header and the body under them.", sample: () => onScreen(controlCenter()) },
      { name: "The notifications", for: "The footer band under the body.", sample: () => onScreen(notifications()) },
    ]),
    measures: [
      { what: "The panel's width.", token: "size-panel" },
      { what: "Inside the panel at its top and its foot, between its parts, and at the sides of each (the container's padding at 384).", token: "space-s16" },
      { what: "The title's line.", token: "type-heading-line" },
      { what: "The pun's line, always there.", token: "type-small-line" },
      { what: "The header's buttons, the tabs and the footer's action: a small control.", token: "size-control_sm" },
      { what: "Between the header's buttons, and between the title's room and them.", token: "space-s8" },
      { what: "The gap the header fade keeps clear, and its fade under it.", token: "space-s16" },
      { what: "How far the body's scroll reaches past its sides.", token: "focus-far" },
      { what: "The body's thumb.", token: "size-scrollbar" },
      { what: "The bar, the frame's top: where a right-edge panel hangs from.", token: "size-bar" },
      { what: "The frame's sides and its bottom.", token: "space-s8" },
      { what: "The fillets into the frame and the far corners.", token: "radius-surface" },
      { what: "The panel's shadow, cast once by the frame and every sheet: its drop.", token: "shadow-panel-y" },
      { what: "Its blur.", token: "shadow-panel-blur" },
      { what: "How far a twin's content slides as it swaps.", token: "space-s24" },
    ],
    reflow: {
      described:
        "The screen at the toolbar's window size, the twins live on it. A panel is `size.panel` wide at every size; only its height follows the screen: as tall as its content up to the room from the bar's foot to the frame's bottom edge, past which the body scrolls and the bands stay. At the minimum the notifications reach the bottom edge and scroll; on a full screen they end above it, in a fillet into the right edge.",
      sample: reflow,
    },
  },
});

