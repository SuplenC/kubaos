// The bell's peek: `config/quickshell/modules/notifications/Peek.qml`. The most recent unread
// notifications in the popups' place while the bar asks for it, with one-click actions on them.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { BUILD, cornerOnScreen, group, type Note, peekFooter, SIGNAL, SLACK, stackOf } from "./notification-parts.ts";

type Shown = "rest" | "lit" | "open";

const GROUPS: readonly (readonly Note[])[] = [SLACK, [BUILD], SIGNAL.map((each) => ({ ...each, unread: true }))];

/** The peek on its corner: its groups, then its footer. */
function peek(options: { groups?: readonly (readonly Note[])[]; more?: number; fan?: boolean; open?: boolean; live?: boolean } = {}): HTMLElement {
  const groups = options.groups ?? GROUPS;
  const made = groups.map((notes, index) => group(notes, { live: options.live ?? false, fan: index === 0 && options.fan ? 1 : 0, open: index === 0 && (options.open ?? false) }));
  return cornerOnScreen(stackOf(...made, peekFooter(options.more ?? 1)));
}

const ON = "the peek";

const OPENS: Move = {
  name: "Opens",
  what: "The bar asks for it: it grows down from the corner as the popups' sheet does.",
  tracks: [{ part: "the sheet", on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the sheet", on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => peek(),
};

const CLOSES: Move = {
  name: "Closes",
  what: "The pointer leaves it and the bell: it goes with its last cards on it.",
  tracks: [{ part: "the sheet", on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: [{ part: "the sheet", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => peek(),
};

export default page<Shown>({
  level: "shell",
  slug: "notification-peek",
  title: "Notification peek",
  lede: "The bell's peek: the latest unread, in the popups' place, with Mark all read and Clear.",
  code: { qml: "Peek" },
  stage: {
    live: () => peek({ live: true }),
    hint: "rest on a stack; click opens or unfolds",
    daylight: true,
  },
  variants: [
    { name: "Three apps and more", for: "+N more opens the panel", sample: () => peek() },
    { name: "Nothing past it", for: "no +N more", sample: () => peek({ groups: [[BUILD]], more: 0 }) },
  ],
  sizes: doesntApply("`size.panel` wide; at most three groups tall."),
  states: {
    shown: ["rest", "lit", "open"],
    absent: {
      pressed: "the press is its cards' and buttons'",
      disabled: "shown or gone",
      selected: "it never takes the keyboard",
      loading: "the history is there",
      empty: "nothing unread: the bell shows no peek",
      error: "nothing it shows can fail",
    },
    rows: [{ name: "The peek", cell: (state) => peek({ fan: state === "lit", open: state === "open" }) }],
    looks: {
      rest: "up to three groups, the footer under them",
      lit: "a group previewed; the rest sit back",
      open: "a group opened in place",
    },
  },
  focus: doesntApply("it never takes the keyboard."),
  content: {
    text: doesntApply("its words are its cards' and its footer's fixed three."),
    missingPicture: doesntApply("its pictures are its cards' tiles."),
    counts: [
      { name: "One", for: "one group, Mark all read, Clear", sample: () => peek({ groups: [[BUILD]], more: 0 }) },
      { name: "Many", for: "three groups and +N more", sample: () => peek({ more: 4 }) },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: powerMoves(() => peek()),
  },
  keyboard: doesntApply("pointer only: looking marks nothing read; its buttons do."),
  measures: {
    samples: nonEmpty([{ name: "The peek", for: "padding, gaps, the footer", sample: () => peek() }]),
    measures: [
      { part: "Width", token: "size-panel" },
      { part: "Inside, between groups", token: "space-s8" },
      { part: "Between footer buttons", token: "space-s4" },
      { part: "Footer buttons", token: "size-control_sm" },
    ],
    reflow: doesntApply("it keeps its corner and `size.panel` on every screen."),
  },
});
