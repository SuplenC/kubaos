// The system monitor's panel (`config/quickshell/modules/sysmon/SysmonPanel.qml`, `Readings.qml`,
// `Section.qml`, `Line.qml`): a sheet on the frame's right edge, from the bar to the frame's foot,
// holding a card per reading and the busiest processes. The processes have a page of their own.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { type SysmonOptions, sysmonOnScreen, TALL } from "./sysmon-parts.ts";

type Shown = "rest" | "lit" | "loading" | "error";

const SHORT = "calc(var(--size-app_min_height) + var(--space-s64) * 2)";

function shot(options: SysmonOptions = {}, height = TALL): HTMLElement {
  return sysmonOnScreen(options, height);
}

function live(): HTMLElement {
  const room = shot({ live: true });
  if (!ALL) requestAnimationFrame(() => room.querySelector<HTMLElement>(".k-scroll-view")?.focus({ preventScroll: true, focusVisible: false } as FocusOptions));
  return room;
}

const ON = "the system monitor";

const OPENING: Move = {
  name: "Opening",
  what: "Its pill or the Command menu: the sheet grows from the right edge on the last detailed sample.",
  tracks: [
    { part: "the sheet", on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the desktop's dim", on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the sheet", on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "the desktop's dim", on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => shot(),
};

const CLOSING: Move = {
  name: "Closing",
  what: "Esc, a press outside, its pill or btop: the sheet goes back, the sample held still.",
  tracks: [
    { part: "the sheet", on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the desktop's dim", on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the sheet", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: "the desktop's dim", on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => shot(),
};

const ARRIVING: Move = {
  name: "A detailed sample comes",
  what: "Each line only a detailed sample fills fades in over its placeholder; nothing moves.",
  tracks: [
    { part: "the placeholders", on: "the waiting lines", target: ".sm-wait[data-waiting] > .sm-bone", property: "opacity", from: 1, to: 0, duration: "slow", curve: "standard" },
    { part: "the words", on: "the waiting lines", target: ".sm-wait[data-waiting] > .sm-words", property: "opacity", from: 0, to: 1, duration: "slow", curve: "standard" },
  ],
  reduced: [
    { part: "the placeholders", on: "the waiting lines", target: ".sm-wait[data-waiting] > .sm-bone", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the words", on: "the waiting lines", target: ".sm-wait[data-waiting] > .sm-words", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "plays",
  sample: () => shot({ sample: "light" }),
};

const FADE: Move = {
  name: "The fade comes",
  what: "The cards leave the top: the header fade comes in over the gap under the header.",
  tracks: [{ part: "the header fade", on: ON, target: ".k-scroll", property: "--faded", from: 0, to: 1, duration: "micro", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => shot({ down: 0.3 }),
};

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return shot();
    case "lit":
      return shot({ tip: "btop" });
    case "loading":
      return shot({ sample: "light" });
    case "error":
      return shot({ unavailable: true });
  }
}

export default page<Shown>({
  level: "shell",
  slug: "sysmon",
  title: "System monitor",
  lede: "The right-edge sheet of readings: a card each, then the busiest processes, opened from its bar pill.",
  code: { qml: "SysmonPanel" },
  from: nonEmpty(["panel-page", "card", "meter", "sparkline", "table", "tabs", "slider", "skeleton", "tooltip"]),
  stage: { live, hint: "point at btop or ⓘ; scroll to the processes", daylight: true },
  variants: [
    { name: "Breaking a sweat", for: "the shots' sample: the backup disk close to full", sample: () => shot() },
    { name: "Cool as a cucumber", for: "every reading fine", sample: () => shot({ heat: 0, disk: 0 }) },
    { name: "This is fine", for: "the CPU past its limit, in `error`", sample: () => shot({ heat: 2, disk: 0 }) },
    { name: "A disk nearly full", for: "its line and meter in `error`", sample: () => shot({ heat: 0, disk: 2 }) },
    { name: "Btrfs quotas off", for: "size unknown; an info button says why", sample: () => shot({ quotasOff: true, tip: "quotas" }) },
    { name: "Uptime and kernel", for: "the corner's card, under the pointer only", sample: () => shot({ tip: "about" }) },
    { name: "Opened from rings", for: "the bar's compact form opens the same panel", sample: () => sysmonOnScreen({}, TALL, undefined, true) },
  ],
  sizes: [
    { name: "A 1000-high screen", for: "the shots' screen: the processes take the rest", sample: () => shot() },
    { name: "A short screen", for: "the whole body scrolls", sample: () => shot({}, SHORT) },
  ],
  states: {
    shown: ["rest", "lit", "loading", "error"],
    absent: {
      pressed: "its parts press on their own pages",
      disabled: "open or gone",
      selected: "a process picked is the processes' page",
      empty: "a reading the machine lacks leaves its card out",
    },
    rows: [{ name: "The panel", cell }],
    looks: {
      rest: "header, mood line, a card each",
      lit: "btop lit: its card below it",
      loading: "light sample: placeholders hold each line",
      error: "the engine down: an `error` line on top",
    },
  },
  focus: doesntApply("no line of its own; its buttons, tabs and rows light theirs."),
  content: {
    text: doesntApply("its words are readings; a long line ends in an ellipsis."),
    missingPicture: doesntApply("it draws no picture; every symbol is fixed."),
    counts: [
      { name: "No GPU or battery", for: "the GPU and Fans cards go; Power says mains", sample: () => shot({ bare: true }) },
      { name: "One GPU", for: "its reading is the card's own", sample: () => shot() },
      { name: "Scrolled", for: "the cards pass under the fade", sample: () => shot({ down: 0.6 }) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [ARRIVING, FADE, ...powerMoves(() => shot({ tip: "btop" }))],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "btop, the snapshots' info, the power stops, the processes" },
      { keys: ["↑", "↓", "PgUp", "PgDn"], does: "scroll the body" },
      { keys: ["Esc"], does: "closes the panel" },
    ],
    order: "btop, then the body; the tabs and the corner's info take the pointer only.",
    lands: "Opening, on the body, scrolled to its top; closing, back on the window that had it.",
  },
  measures: {
    samples: nonEmpty([{ name: "The panel", for: "width, padding, cards, the corner", sample: () => shot() }]),
    measures: [
      { part: "Panel's width", token: "size-panel" },
      { part: "Top, header to body, foot", token: "space-s16" },
      { part: "Between cards", token: "space-s8" },
      { part: "Card's padding", token: "space-s16" },
      { part: "Head to lines", token: "space-s8" },
      { part: "Between lines", token: "space-s4" },
      { part: "A core's bar", token: "space-s24" },
      { part: "Between cores", token: "space-s4" },
      { part: "Info button", token: "size-control_sm" },
    ],
    reflow: {
      note: "It keeps `size.panel` and runs from the bar to the frame's foot; the body scrolls.",
      sample: () => shot({}, SHORT),
    },
  },
});
