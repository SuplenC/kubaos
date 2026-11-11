// The system monitor's processes (`config/quickshell/modules/sysmon/Processes.qml`,
// `ProcessRow.qml`, `ProcessDetails.qml`): the busiest, by CPU or memory, as many rows as fit; a
// row's details open under it, with End and Kill, each asking first.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { type SysmonOptions, sysmonOnScreen, TALL } from "./sysmon-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "loading" | "empty" | "error";

const SHORT = "calc(var(--size-app_min_height) + var(--space-s64) * 2)";

/** The panel scrolled to its foot, the processes in view, as the shots' "scrolled". */
function shot(options: SysmonOptions = {}, height = TALL): HTMLElement {
  return sysmonOnScreen({ down: 1, ...options }, height);
}

const DETAILS: Move = {
  name: "Details open",
  what: "A click on a row: its details open under it, the rows below moving down.",
  tracks: [{ part: "the details", on: "the row picked", target: ".sm-details", property: "--sm-open", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [{ part: "the details", on: "the row picked", target: ".sm-details", property: "--sm-open", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "plays",
  sample: () => shot({ picked: 0 }),
};

const COUNTED: Move = {
  name: "The rows come",
  what: "The first detailed sample: each process fades in over its row's placeholder.",
  tracks: [
    { part: "the placeholders", on: "the rows", target: ".sm-placeholder .sm-bone", property: "opacity", from: 1, to: 0, duration: "slow", curve: "standard" },
  ],
  reduced: [{ part: "the placeholders", on: "the rows", target: ".sm-placeholder .sm-bone", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
  back: "plays",
  sample: () => shot({ sample: "light" }),
};

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return shot();
    case "lit":
      return shot({ tip: "about" });
    case "pressed":
      return shot({ picked: 0, confirming: "term" });
    case "selected":
      return shot({ picked: 0 });
    case "loading":
      return shot({ sample: "light" });
    case "empty":
      return shot({ noProcesses: true });
    case "error":
      return shot({ picked: 0, confirming: "kill" });
  }
}

export default page<Shown>({
  level: "shell",
  slug: "sysmon-processes",
  title: "System monitor processes",
  lede: "The busiest processes under the readings: by CPU or memory, a row's details and its End and Kill.",
  code: { qml: "Processes" },
  from: nonEmpty(["table", "tabs", "button", "skeleton", "hint"]),
  stage: { live: () => shot({ live: true }), hint: "click a row; End or Kill asks first", daylight: true },
  variants: [
    { name: "By CPU", for: "the busiest first", sample: () => shot() },
    { name: "By memory", for: "the heaviest first", sample: () => shot({ order: "memory" }) },
    { name: "A row's details", for: "its command, whose, End and Kill", sample: () => shot({ picked: 0 }) },
    { name: "End asking", for: "End filled; the question and Cancel after", sample: () => shot({ picked: 0, confirming: "term" }) },
    { name: "Kill asking", for: "Kill `Danger`; the question in `error`", sample: () => shot({ picked: 0, confirming: "kill" }) },
  ],
  sizes: [
    { name: "A 1000-high screen", for: "rows fill the room left", sample: () => shot() },
    { name: "A short screen", for: "never fewer than five rows", sample: () => shot({}, SHORT) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "selected", "loading", "empty", "error"],
    absent: { disabled: "the button not asked about rests disabled while one asks" },
    rows: [{ name: "The processes", cell }],
    looks: {
      rest: "heading, the order, the heads, the rows",
      lit: "the corner's card over the rows",
      pressed: "End pressed: it asks, filled",
      selected: "a row picked: its details under it",
      loading: "a bar in each column a row",
      empty: "No processes to show",
      error: "Kill pressed: `Danger`, the question red",
    },
  },
  focus: doesntApply("the table and its buttons light their own lines."),
  content: {
    text: [{ name: "A long command", words: { short: "Hyprland", long: "/usr/lib/firefox/firefox -contentproc -childID 12 -isForBrowser -prefsLen 31000 -prefMapSize 244787" }, sample: (words: string) => shot({ picked: 0, command: words }) }],
    missingPicture: doesntApply("a row has no picture."),
    counts: [
      { name: "None", for: "the heads, then one quiet line", sample: () => shot({ noProcesses: true }) },
      { name: "Five", for: "the fewest rows it keeps", sample: () => shot({}, SHORT) },
      { name: "As many as fit", for: "up to forty, by the room left", sample: () => shot() },
    ],
  },
  motion: {
    arrives: [COUNTED],
    leaves: doesntApply("the rows go with the panel."),
    between: [DETAILS, ...powerMoves(() => shot({ tip: "about" }))],
  },
  keyboard: {
    keys: [
      { keys: ["Tab"], does: "onto the rows; the order and End/Kill take the pointer" },
      { keys: ["↑", "↓", "j", "k"], does: "row to row" },
      { keys: ["Enter"], does: "opens or shuts the row's details" },
    ],
    order: "The rows are one stop, after the readings' controls.",
    lands: "Opened from a Command menu Item, on that process's details.",
  },
  measures: {
    samples: nonEmpty([{ name: "Details open", for: "rows, details, buttons", sample: () => shot({ picked: 0 }) }]),
    measures: [
      { part: "Row", token: "size-row_compact" },
      { part: "Row's sides", token: "space-s12" },
      { part: "Heading to the heads", token: "space-s8" },
      { part: "Details' top", token: "space-s8" },
      { part: "Details' sides and foot", token: "space-s12" },
      { part: "Between details' lines", token: "space-s4" },
      { part: "Between buttons", token: "space-s8" },
    ],
    reflow: { note: "The rows take the room left above the foot, never fewer than five.", sample: () => shot({}, SHORT) },
  },
});
