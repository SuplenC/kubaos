// The clock popup, a DRAFT (`config/quickshell/modules/clock/ClockPopup.qml` is the build to come):
// a sheet out of the top edge's middle, under the bar's clock pill. The clock view (the time, the
// date, home's weather, the other cities, the month and the picked day) and the weather view a
// weather card turns it into. The clock pill's states are here too, at every step a narrow bar
// takes.
//
// Its settings, and the bar clock's, are the Settings app's Date & time section
// (`apps/settings/settings-date-time.ts`); the popup holds none.

import { barStrip, EVERYDAY, EVERYTHING } from "./bar-parts.ts";
import { type ClockOptions, clockOnScreen } from "./clock-parts.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

type Shown = "rest" | "empty" | "error";

function shot(options: ClockOptions = {}, width = 1440): HTMLElement {
  return clockOnScreen(options, width);
}

/** The bar alone at `width`, with every pill or an ordinary day's. */
const strip = (width: number, all = false) => () => barStrip({ width, content: all ? EVERYTHING : EVERYDAY });

const ON = "the clock popup";

const OPENING: Move = {
  name: "Opening",
  what: "The bar's time or temperature, Super+C or a Command menu Item: the sheet grows down from the top edge.",
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
  what: "Esc, a press outside or the clock: the sheet goes back up.",
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

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return shot();
    case "empty":
      return shot({ home: "no city", cities: 0 });
    case "error":
      return shot({ home: "no connection" });
  }
}

const counts = (n: number) => () => shot({ cities: n });

export default page<Shown>({
  level: "shell",
  slug: "clock",
  title: "Clock popup",
  lede: "Under the bar's clock pill: the time, home's weather, the other cities, the month, and each city's weather.",
  code: { qml: "ClockPopup" },
  from: nonEmpty(["weather-sky", "day-strip", "panel-page", "calendar", "description-list", "banner", "card", "button", "pill"]),
  stage: { live: () => shot({ live: true }), hint: "click a weather card, then Clock; the bar's time and temperature", daylight: true },
  variants: [
    { name: "Home's weather", for: "the weather view, home's warning", sample: () => shot({ view: "weather", whose: "home" }) },
    { name: "A city's weather", for: "Cairo: no warnings source, one quiet line", sample: () => shot({ view: "weather", whose: 7 }) },
    { name: "Another day", for: "Thursday picked: its banner, hours and tiles", sample: () => shot({ view: "weather", whose: "home", day: 2 }) },
    { name: "A holiday picked", for: "1 November, All Saints' Day", sample: () => shot({ picked: { year: 2026, month: 11, day: 1 } }) },
    { name: "Long city names", for: "cut with an ellipsis, one line", sample: () => shot({ order: [5, 4, 0, 1] }) },
  ],
  sizes: doesntApply("the column's width; as tall as the view it holds."),
  groups: [
    {
      title: "The pill as the bar narrows",
      rows: nonEmpty([
        { name: "Full width", for: "seal, date, capsule, weather", sample: strip(1920) },
        { name: "1280 with every pill", for: "the weather goes first", sample: strip(1280, true) },
        { name: "1152 with every pill", for: "the date gone, the seal kept", sample: strip(1152, true) },
        { name: "1024 with every pill", for: "recording joins: its colours, its dot", sample: strip(1024, true) },
        { name: "540, an ordinary day", for: "the seal gone last; the time stays", sample: strip(540) },
      ]),
    },
    {
      title: "Cities",
      rows: nonEmpty([
        { name: "None", for: "the row goes", sample: counts(0) },
        { name: "One", for: "the row to itself", sample: counts(1) },
        { name: "Two", for: "sharing the row", sample: counts(2) },
        { name: "Three", for: "sharing the row", sample: counts(3) },
        { name: "Four", for: "one row", sample: counts(4) },
        { name: "Six", for: "a second row, stretched", sample: counts(6) },
        { name: "Eight, the most", for: "two rows of four", sample: counts(8) },
      ]),
    },
    {
      title: "Home's weather",
      rows: nonEmpty([
        { name: "Offline, 2 h", for: "the last reading dimmed, its age", sample: () => shot({ home: "aged" }) },
        { name: "Offline over a day", for: "No connection in its place", sample: () => shot({ home: "no connection" }) },
        { name: "No home city", for: "Set your city; no weather in the pill", sample: () => shot({ home: "no city" }) },
      ]),
    },
  ],
  states: {
    shown: ["rest", "empty", "error"],
    absent: {
      lit: "keyboard focus lights its parts, each on its own page",
      pressed: "its parts press on their own pages",
      disabled: "open or gone",
      selected: "the month and the week pick on their pages",
      loading: "the last reading shows until a new one comes",
    },
    rows: [{ name: "The popup", cell }],
    looks: {
      rest: "today picked, four cities, Warsaw's weather",
      empty: "no home city, no cities",
      error: "offline past a day: No connection",
    },
  },
  focus: doesntApply("no line of its own; the month, the tabs and Back light theirs."),
  content: {
    text: doesntApply("a city's long name is cut with an ellipsis: see Long city names."),
    missingPicture: doesntApply("its skies are drawn; every symbol is fixed."),
    counts: doesntApply("none to eight cities: see Cities."),
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [...powerMoves(() => shot())],
  },
  keyboard: doesntApply("a draft: the month and the tabs keep their own keys; the popup's own come with its hardening."),
  measures: {
    samples: nonEmpty([{ name: "The popup", for: "padding, rows, gaps", sample: () => shot() }]),
    measures: [
      { part: "Above and below the body", token: "space-s20" },
      { part: "Its width", token: "size-column" },
      { part: "Between rows", token: "space-s16" },
      { part: "Time to home's card", token: "space-s20" },
      { part: "Between cities", token: "space-s8" },
      { part: "Month to the picked day", token: "space-s20" },
      { part: "The weather view's rise", token: "space-s16" },
    ],
    reflow: doesntApply("its width is the column's at every window size."),
  },
});
