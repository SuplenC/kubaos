// The clock popup (`config/quickshell/modules/clock/ClockPopup.qml`, `AnalogFace.qml`, `Tools.qml`):
// a sheet out of the top edge's middle, under the bar's clock. The time to the second, the date, the
// extra clocks, a date search, the month and the picked day, countdowns, and the timer, stopwatch
// and alarms. Its settings page has a page of its own.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { logicalWidth } from "./bar-parts.ts";
import { type ClockOptions, clockOnScreen } from "./clock-parts.ts";
import { windowSize } from "../../site/settings.ts";

type Shown = "rest" | "lit" | "selected" | "empty" | "error";

/** A bar narrow enough to shed its date and its countdown: the popup still holds both. */
const NARROW = 760;

function shot(options: ClockOptions = {}, width = 1440): HTMLElement {
  return clockOnScreen(options, width);
}

const ON = "the clock popup";

const OPENING: Move = {
  name: "Opening",
  what: "The bar's clock, Super+C or a Command menu Item: the sheet grows down from the top edge.",
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
  what: "Esc with nothing to clear, a press outside or the clock: the sheet goes back up.",
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

const ANSWER: Move = {
  name: "The answer opens",
  what: "A question typed: its card opens under the search, as a log does, pushing the month down.",
  tracks: [{ part: "the answer", on: "the search", target: ".ck-answer", property: "--ck-open", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the answer", on: "the search", target: ".ck-answer", property: "--ck-open", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "plays",
  sample: () => shot({ query: "days until vacation" }),
};

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return shot();
    case "lit":
      return shot({ query: "days until vacation" });
    case "selected":
      return shot({ picked: 1 });
    case "empty":
      return shot({ clocks: 0, countdowns: 0, tool: "alarm", alarms: 0, timer: "none" });
    case "error":
      return shot({ countdownError: "Name it, then the date: Vacation: 19 dec", unavailable: true });
  }
}

export default page<Shown>({
  level: "shell",
  slug: "clock",
  title: "Clock popup",
  lede: "Under the bar's clock: the time to the second, the date, extra clocks, a date search, the month, countdowns and timers.",
  code: { qml: "ClockPopup" },
  from: nonEmpty(["panel-page", "field", "card", "calendar", "tabs", "chip", "icon-button", "figure", "progress-bar", "keycap"]),
  stage: { live: () => shot({ live: true }), hint: "type in the search; ← → months; the gear", daylight: true },
  variants: [
    { name: "An answer", for: "a typed question, Enter copies it", sample: () => shot({ query: "days until vacation" }) },
    { name: "Copied", for: "the line in place of the hints", sample: () => shot({ query: "days until vacation", copied: true }) },
    { name: "Analog face", for: "beside the time, from the settings", sample: () => shot({ analog: true }) },
    { name: "The year", for: "the month's title opens its twelve months", sample: () => shot({ year: true }) },
    { name: "A holiday picked", for: "its name in `tertiary` under the day", sample: () => shot({ picked: 1 }) },
    { name: "From a narrow bar", for: "the bar shed its date and countdown; here both stay", sample: () => shot({}, NARROW) },
    { name: "Timer paused", for: "Resume; the time in `on_surface_variant`", sample: () => shot({ timer: "paused" }) },
    { name: "Timer not set", for: "a duration's field and four presets", sample: () => shot({ timer: "none" }) },
    { name: "Stopwatch", for: "start, lap, reset; the last two laps", sample: () => shot({ tool: "stopwatch" }) },
    { name: "Alarms", for: "a field over up to three alarms", sample: () => shot({ tool: "alarm", alarms: 3 }) },
  ],
  sizes: doesntApply("as wide as the month, the rail and the gap; as tall as it holds."),
  states: {
    shown: ["rest", "lit", "selected", "empty", "error"],
    absent: {
      pressed: "its parts press on their own pages",
      disabled: "open or gone",
      loading: "the engine answers before it opens",
    },
    rows: [{ name: "The popup", cell }],
    looks: {
      rest: "today picked, the timer running",
      lit: "the search lit, its answer under it",
      selected: "another day picked: its details",
      empty: "no clocks, countdowns or alarms",
      error: "a countdown refused; the engine down",
    },
  },
  focus: doesntApply("no line of its own; the search, the month and the tools light theirs."),
  content: {
    text: [
      {
        name: "The answer",
        words: { short: "53 days", long: "In 53 days, 7 weeks and 4 days, on Saturday 19 December 2026" },
        sample: (words: string) => shot({ query: "days until vacation", answer: { title: words, detail: "From today to Saturday 19 December 2026" } }),
      },
    ],
    missingPicture: doesntApply("it draws no picture; every symbol is fixed."),
    counts: [
      { name: "No extra clocks", for: "the row goes", sample: () => shot({ clocks: 0 }) },
      { name: "One clock, two countdowns", for: "the shots' day", sample: () => shot() },
      { name: "Four clocks, three countdowns", for: "the clocks wrap; one passed", sample: () => shot({ clocks: 4, countdowns: 3 }) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [ANSWER, ...powerMoves(() => shot({ query: "days until vacation" }))],
  },
  keyboard: {
    keys: [
      { keys: ["a–z"], does: "types into the search" },
      { keys: ["←", "→"], does: "the search empty: a month back or on; a year" },
      { keys: ["T"], does: "the search empty: back to today" },
      { keys: ["Enter"], does: "copies the answer, or the picked day" },
      { keys: ["Ctrl+Enter"], does: "copies its ISO date" },
      { keys: ["Tab", "Shift+Tab"], does: "search, gear, month, countdowns, the tools" },
      { keys: ["Esc"], does: "clears the search, leaves the year, then closes" },
    ],
    order: "The search, then the gear, the month, the countdowns' field, the tools' tabs and fields.",
    lands: "Opening, in the search, today picked; the settings left, back in the search.",
  },
  measures: {
    samples: nonEmpty([{ name: "The popup", for: "padding, columns, gaps", sample: () => shot({ query: "days until vacation" }) }]),
    measures: [
      { part: "Above and below the body", token: "space-s20" },
      { part: "Between sections", token: "space-s16" },
      { part: "Time to date to day", token: "space-s4" },
      { part: "Time to seconds", token: "space-s12" },
      { part: "Face to the time", token: "space-s20" },
      { part: "Extra clock", token: "size-control_md" },
      { part: "Between extra clocks", token: "space-s8" },
      { part: "Month to the rail", token: "space-s20" },
      { part: "The rail", token: "size-rail" },
      { part: "Tabs to a tool", token: "space-s12" },
    ],
    reflow: {
      note: "Its width is its content's; the bar narrows round it, shedding the date and countdown.",
      sample: () => shot({}, Math.max(NARROW, logicalWidth(windowSize().width))),
    },
  },
});

