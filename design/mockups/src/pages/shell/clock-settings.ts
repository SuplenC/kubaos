// The clock popup's settings (`config/quickshell/modules/clock/ClockSettings.qml`): the gear swaps
// them in for the search, the month and the tools. Holidays, the first weekday, the time format,
// the date order, the analog face, the bar's countdown and the extra clocks.

import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { type ClockOptions, clockOnScreen } from "./clock-parts.ts";

type Shown = "rest" | "open" | "selected" | "error";

function shot(options: ClockOptions = {}): HTMLElement {
  return clockOnScreen({ page: "settings", ...options });
}

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return shot();
    case "open":
      return shot({ holidaysOpen: true });
    case "selected":
      return shot({ analog: true, pinned: "Release" });
    case "error":
      return shot({ unavailable: true });
  }
}

export default page<Shown>({
  level: "shell",
  slug: "clock-settings",
  title: "Clock settings",
  lede: "The clock popup's gear: how the calendar, the time and the bar's countdown read, and the extra clocks.",
  code: { qml: "ClockSettings" },
  from: nonEmpty(["select", "chip", "field", "icon-button"]),
  stage: { live: () => clockOnScreen({ page: "settings", live: true }), hint: "the × goes back; Esc too", daylight: true },
  variants: [
    { name: "The shots' settings", for: "Poland's holidays, Monday first, Vacation pinned", sample: () => shot() },
    { name: "Nothing pinned", for: "None picked: the bar shows no countdown", sample: () => shot({ pinned: null }) },
    { name: "Four clocks", for: "the field goes at four", sample: () => shot({ clocks: 4 }) },
  ],
  sizes: doesntApply("the popup's width; as tall as the settings."),
  states: {
    shown: ["rest", "open", "selected", "error"],
    absent: {
      lit: "each control lights on its own page",
      pressed: "each control presses on its own page",
      disabled: "every setting can change",
      loading: "the engine answers before it opens",
      empty: "No clocks: only the field",
    },
    rows: [{ name: "The settings", cell }],
    looks: {
      rest: "a title, a note, its choices",
      open: "the holidays' menu over the rest",
      selected: "the face on, Release pinned",
      error: "the engine's reply in `error` at the foot",
    },
  },
  focus: doesntApply("its select, chips and field light their own lines."),
  content: {
    text: doesntApply("its words are fixed; a clock's name runs to its IANA name."),
    missingPicture: doesntApply("it draws no picture."),
    counts: [
      { name: "No clocks", for: "the field alone", sample: () => shot({ clocks: 0 }) },
      { name: "One", for: "Tokyo, Remove at its end", sample: () => shot() },
      { name: "Four", for: "the most: no field", sample: () => shot({ clocks: 4 }) },
    ],
  },
  motion: {
    arrives: doesntApply("the gear swaps it in at once, with no move."),
    leaves: doesntApply("the × or Esc swaps it out at once."),
    between: doesntApply("a choice picked changes the chip's look, the chip's own move."),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "the ×, the holidays, the clocks' field; chips take the pointer" },
      { keys: ["Enter"], does: "in the field: adds the clock" },
      { keys: ["Esc"], does: "clears the field, or back to the popup" },
    ],
    order: "The ×, the holidays' select, each clock's Remove, the field.",
    lands: "Back in the popup's search when the settings close.",
  },
  measures: {
    samples: nonEmpty([{ name: "The settings", for: "sections, title to choices", sample: () => shot() }]),
    measures: [
      { part: "Between settings", token: "space-s16" },
      { part: "Title to note", token: "space-s4" },
      { part: "Note to choices", token: "space-s8" },
      { part: "Between chips", token: "space-s4" },
    ],
    reflow: doesntApply("the popup's width holds; only its height follows the settings."),
  },
});
