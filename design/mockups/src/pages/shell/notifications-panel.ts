// The notifications panel: `config/quickshell/modules/notifications/NotificationPanel.qml` and
// `NotificationMenu.qml`. Every notification, New then Earlier, out of the frame's right edge.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { BUILD, KUBAOS, SIGNAL } from "../notifications.ts";
import { cornerOnScreen, noteMenu, panelOnScreen, type PanelOptions, type Section } from "./notification-parts.ts";

type Shown = "rest" | "lit" | "open" | "empty";

const READ: readonly Section[] = [
  { label: "New", groups: [] },
  { label: "Earlier", groups: [SIGNAL, KUBAOS] },
];
const NONE: readonly Section[] = [];

const ON = "the notifications panel";

const OPENING: Move = {
  name: "Opening",
  what: "The bell or its key: the sheet grows out of the right edge, the popups leave, the desktop dims.",
  tracks: [
    { part: "the sheet", on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the desktop's dim", on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the sheet", on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "the desktop's dim", on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => panelOnScreen(),
};

const CLOSING: Move = {
  name: "Closing",
  what: "Esc, a press outside or the bell: everything shown is marked read as the sheet goes back.",
  tracks: [
    { part: "the sheet", on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the desktop's dim", on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the sheet", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: "the desktop's dim", on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => panelOnScreen(),
};

const RECEDES: Move = {
  name: "The list sits back",
  what: "A group's preview spreads: the other rows fade by `opacity.recede`, those it covers go.",
  tracks: [{ part: "the rows behind", on: ON, target: ".k-notif-behind", property: "--recede", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [{ part: "the rows behind", on: ON, target: ".k-notif-behind", property: "--recede", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "plays",
  sample: () => panelOnScreen(),
};

const FADE: Move = {
  name: "The fade comes",
  what: "The list leaves its top: the header fade comes in over the gap under the header.",
  tracks: [{ part: "the header fade", on: ON, target: ".k-scroll", property: "--faded", from: 0, to: 1, duration: "micro", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => panelOnScreen(),
};

const IN_STATE = (state: Shown, extra: PanelOptions = {}) =>
  panelOnScreen({
    ...extra,
    ...(state === "open" ? { opened: [0] } : {}),
    ...(state === "lit" ? { selected: 0 } : {}),
    ...(state === "empty" ? { sections: NONE } : {}),
  });

function live(): HTMLElement {
  return panelOnScreen({ live: true, selected: 0 }, "var(--size-app_height)");
}

export default page<Shown>({
  level: "shell",
  slug: "notifications-panel",
  title: "Notifications panel",
  lede: "Every notification, New then Earlier, one app's in a group, out of the frame's right edge.",
  code: { qml: "NotificationPanel" },
  stage: {
    live,
    hint: "↑ ↓ move, → opens, Enter unfolds, Del, D",
    daylight: true,
  },
  variants: [
    { name: "New and Earlier", for: "unread under New until it closes", sample: () => panelOnScreen({ selected: 0 }) },
    { name: "All read", for: "Earlier alone; the line says All read", sample: () => panelOnScreen({ sections: READ }) },
    { name: "Do not disturb on", for: "its button filled; the line says why", sample: () => panelOnScreen({ dnd: true }) },
    { name: "A card unfolded", for: "its whole text; one at a time", sample: () => panelOnScreen({ unfolded: 1, selected: 1 }) },
    { name: "A group previewed", for: "the rest sit back", sample: () => panelOnScreen({ fanned: 0 }) },
    { name: "The keys", for: "the keys button's card, floating above it", sample: () => panelOnScreen({ keys: true }) },
    { name: "Do not disturb's card", for: "floats under its button: what turns it on", sample: () => panelOnScreen({ dndTip: true }) },
    { name: "A card's menu", for: "right-click: actions, Open, Dismiss, Clear all", sample: () => cornerOnScreen(noteMenu(BUILD, 0)) },
  ],
  sizes: doesntApply("`size.panel` wide, from the bar to the frame's foot."),
  states: {
    shown: ["rest", "lit", "open", "empty"],
    absent: {
      selected: "the keyboard on a row is lit",
      pressed: "a press inside does nothing; outside closes it",
      disabled: "open or gone",
      loading: "the history is there when it opens",
      error: "nothing it shows can fail",
    },
    rows: [{ name: "The panel", cell: (state) => IN_STATE(state) }],
    looks: {
      rest: "header, line, New then Earlier",
      open: "a group opened under its heading",
      lit: "the keyboard on a row: its tube",
      empty: "Nothing new., centred; Clear all goes",
    },
  },
  focus: doesntApply("no line of its own; its cards and buttons light theirs."),
  content: {
    text: doesntApply("its words are its cards'; the line under the title is fixed."),
    missingPicture: doesntApply("its pictures are its cards' tiles."),
    counts: [
      { name: "None", for: "Nothing new.; Silence is golden", sample: () => panelOnScreen({ sections: NONE }) },
      { name: "One", for: "one section, one card", sample: () => panelOnScreen({ sections: [{ label: "New", groups: [[BUILD]] }] }) },
      { name: "Many", for: "the list scrolls under the fade", sample: () => panelOnScreen({ opened: [0, 2, 3] }) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [RECEDES, FADE, ...powerMoves(() => IN_STATE("lit"))],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "j", "k"], does: "row to row; inside an opened group, card to card" },
      { keys: ["Ctrl+N", "Ctrl+P"], does: "the same, down and up" },
      { keys: ["→", "l"], does: "opens the group" },
      { keys: ["←", "h"], does: "closes the group" },
      { keys: ["Enter", "Space"], does: "opens a stack, or unfolds the card" },
      { keys: ["Shift+Enter"], does: "runs the card's main action" },
      { keys: ["Del"], does: "dismisses the card or the whole stack" },
      { keys: ["D"], does: "turns Do not disturb on or off" },
      { keys: ["Esc"], does: "folds, then closes the group, then the panel" },
    ],
    order: "The list is the one stop; its header buttons and the keys take the pointer.",
    lands: "Opening, on the first row, unlit; closing, back on the window that had it.",
  },
  measures: {
    samples: nonEmpty([{ name: "The panel", for: "header, labels, list, keys", sample: () => panelOnScreen({ selected: 0 }) }]),
    measures: [
      { part: "Panel's width", token: "size-panel" },
      { part: "Padding, header's top, list's top", token: "space-s16" },
      { part: "Title's line", token: "type-heading-line" },
      { part: "Status line", token: "type-small-line" },
      { part: "Header's buttons, keys", token: "size-control_sm" },
      { part: "Between rows, label to card", token: "space-s8" },
      { part: "Section label", token: "type-label-line" },
      { part: "List's reach for the glow", token: "focus-far" },
      { part: "Keys from the corner", token: "space-s16" },
    ],
    reflow: {
      note: "It keeps `size.panel` and runs from the bar to the frame's foot; the list scrolls.",
      sample: () => {
        const size = windowSize();
        return panelOnScreen({}, size.height);
      },
    },
  },
});
