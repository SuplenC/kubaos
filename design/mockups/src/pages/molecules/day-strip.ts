// The DayStrip page, a DRAFT: `docs/design/components/day-strip.md`. A row of days to pick one of,
// each its forecast at a glance: the clock popup's week in its weather view.

import type { Pinned } from "../../lib/control.ts";
import { dayStrip, type StripDay } from "../../lib/day-strip.ts";
import { within } from "../../lib/layout.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { CITIES, dayName, dayRange, degrees, HOME, symbolOf } from "../shell/weather-data.ts";

/** Warsaw's week, as the clock popup shows it. */
const WEEK: readonly StripDay[] = HOME.week.map((weather, d) => {
  const [high, low] = dayRange(HOME, d);
  return { name: dayName(d), symbol: symbolOf(weather), high: degrees(high), low: degrees(low) };
});

/** The strip in the popup's column, `count` days, `picked` picked, one day pinned. */
function strip(count = 7, picked = 0, pinned?: { readonly index: number; readonly state: Pinned }, days: readonly StripDay[] = WEEK): HTMLElement {
  return within("var(--size-column)", dayStrip({ days: days.slice(0, count), picked, ...(pinned ? { pinned } : {}) }));
}

type Shown = "rest" | "hovered" | "pressed" | "lit" | "selected";

/** A state on the day after the picked one, or on the picked one where only it can show it. */
function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return strip();
    case "hovered":
      return strip(7, 0, { index: 2, state: "hover" });
    case "pressed":
      return strip(7, 0, { index: 2, state: "pressed" });
    case "lit":
      return strip(7, 0, { index: 0, state: "lit" });
    case "selected":
      return strip(7, 3);
  }
}

/** New York's week: other weathers, for the symbols. */
const OTHER: readonly StripDay[] = CITIES[1]!.week.map((weather, d) => {
  const [high, low] = dayRange(CITIES[1]!, d);
  return { name: dayName(d), symbol: symbolOf(weather), high: degrees(high), low: degrees(low) };
});

export default page<Shown>({
  level: "molecules",
  slug: "day-strip",
  title: "DayStrip",
  lede: "A row of days to pick one of, each its name, symbol, high and low: the forecast's week.",
  code: { unbuilt: "Rust and QML: not built yet" },
  stage: { live: () => strip(), hint: "click a day; Tab, then ← →", daylight: true },
  variants: [
    { name: "A week", for: "seven days, today picked", sample: () => strip() },
    { name: "Another week", for: "other weathers' symbols", sample: () => strip(7, 0, undefined, OTHER) },
  ],
  sizes: doesntApply("its row's width, shared equally; its height its words'."),
  states: {
    shown: ["rest", "hovered", "pressed", "lit", "selected"],
    absent: {
      disabled: "every day can be picked",
      loading: "drawn with its forecast",
      empty: "a forecast always has days",
      error: "no forecast: no strip",
    },
    rows: nonEmpty([{ name: "The week", cell }]),
    looks: {
      rest: "no surface; today on the picked container",
      hovered: "the state layer, a faint shape",
      pressed: "the state layer a step further",
      lit: "the accent's neon on the keyboard's day",
      selected: "`secondary_container`, its words `on_secondary_container`",
    },
  },
  focus: {
    tubes: [{ name: "The keyboard's day", tone: "accent", filled: false, content: false }],
    looks: { lit: "the line inside the edge, the glow outside", signsOff: "the line as glass, no glow" },
    signsOff: () => strip(7, 0, { index: 0, state: "signs-off" }),
  },
  content: {
    text: doesntApply("short day names and temperatures; a day's name never wraps."),
    missingPicture: doesntApply("every weather has its symbol."),
    counts: [
      { name: "Three days", for: "fewer days, each wider", sample: () => strip(3) },
      { name: "Seven days", for: "the forecast's week", sample: () => strip(7) },
    ],
  },
  motion: {
    arrives: doesntApply("it is drawn with its view."),
    leaves: doesntApply("it goes with its view."),
    between: doesntApply("hover, press and picking fade as every control's do, over `short` and `micro`."),
  },
  keyboard: {
    keys: [
      { keys: ["←", "→", "h", "l"], does: "picks the day before or after" },
      { keys: ["Home", "End"], does: "picks the first or last day" },
    ],
    order: "The row is one stop for Tab, on the picked day.",
    lands: "Its owner says where the keyboard lands; the row never takes it unasked.",
  },
  measures: {
    samples: nonEmpty([{ name: "A week", for: "a day's padding and gaps", sample: () => strip() }]),
    measures: [
      { part: "Between days", token: "space-s4" },
      { part: "A day, above and below", token: "space-s8" },
      { part: "Between a day's words", token: "space-s4" },
      { part: "A day's corner", token: "radius-control" },
    ],
    reflow: doesntApply("its days share whatever width it is given."),
  },
});
