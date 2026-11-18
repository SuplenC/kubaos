// Settings › Power: the battery, sleep and lock, and performance, from `power.toml` and
// `idle.toml`. It is the plain section the other ten follow, so its variants are the row grammar:
// each kind of row a setting takes, landed on and lit.

import { doesntApply, nonEmpty, page, screen } from "../../../site/page.ts";
import { deeperMoves } from "../../templates/sidebar-turns.ts";
import { POWER } from "./settings-data.ts";
import { litOn, liveWindow, MINIMUM, seeded, settingsWindow } from "./settings-window.ts";

type Shown = "rest" | "disabled" | "error";

const lit = (id: string, values = {}) => screen(() => settingsWindow(litOn(POWER, id, values)));

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: () => settingsWindow({ section: POWER }),
  disabled: () => settingsWindow(litOn(POWER, "battery-notes")),
  error: () => settingsWindow(litOn(POWER, "keep-for", { keepAwake: true, keepFor: "soon" })),
};

export default page<Shown>({
  level: "apps",
  slug: "settings-power",
  title: "Settings: Power",
  lede: "Battery, sleep and lock, and performance: the plain section whose rows the other ten follow.",
  code: { unbuilt: "kubaos-settings: not built yet" },
  from: ["sidebar-page", "list", "tabs", "slider", "select", "field", "switch", "banner", "keycap", "button", "icon-button"],
  stage: {
    live: () => liveWindow({ section: POWER }),
    hint: "hold a reset; ← → change a row",
    daylight: true,
  },
  variants: [
    { name: "A switch", for: "Keep awake: the row flips it", sample: lit("keep-awake") },
    { name: "A segmented choice", for: "Charge limit: three, side by side", sample: lit("charge-limit") },
    { name: "A slider and its value", for: "Power stop: five named stops", sample: lit("stop-battery") },
    { name: "A select", for: "Lock after: a list too long to show", sample: lit("lock") },
    { name: "A field, valid", for: "Keep awake for 2h", sample: lit("keep-for", { keepAwake: true, keepFor: "2h" }) },
    { name: "A field, invalid", for: "the line says what's wrong, in `error`", sample: screen(CELLS.error) },
    { name: "A row opening a page", for: "Tune each stop: a chevron", sample: lit("stops") },
    { name: "The page it opens", for: "Back, a stop picked, its settings", sample: screen(() => settingsWindow({ section: POWER, values: { opened: "stops" } })) },
    { name: "A dependent row", for: "Low means, under When it runs low", sample: lit("low-at") },
    { name: "Its row gone", for: "Do nothing: Low means leaves", sample: lit("low-stop", { lowStop: 2 }) },
    { name: "A disabled row", for: "Power off: dimmed, the reason its line", sample: screen(CELLS.disabled) },
    { name: "A risky value", for: "Doze deepest: the warning under it", sample: screen(() => settingsWindow({ section: POWER, values: { opened: "stops", tuned: 0 } })) },
    { name: "Plugged in", for: "a row's reset: this source; a heading's: both", sample: lit("source", { ...seeded(), source: 1 }) },
    { name: "A row at its default", for: "no reset; its place kept", sample: lit("charge-limit") },
    { name: "A row changed", for: "its reset just before its control", sample: lit("charge-limit", { chargeLimit: 0 }) },
    { name: "Its reset lit", for: "its hint: what it goes back to", sample: screen(() => settingsWindow({ ...litOn(POWER, "charge-limit", { chargeLimit: 0 }), resets: { at: "charge-limit", lit: true } })) },
    { name: "Its reset held halfway", for: "the ring half round; letting go empties it", sample: screen(() => settingsWindow({ ...litOn(POWER, "charge-limit", { chargeLimit: 0 }), resets: { at: "charge-limit", held: 0.5 } })) },
    { name: "A heading with its Reset", for: "Battery changed: Reset puts all of it back", sample: screen(() => settingsWindow({ section: POWER, values: seeded() })) },
    { name: "A heading's Reset lit", for: "its hint: Hold to reset Battery", sample: screen(() => settingsWindow({ section: POWER, values: seeded(), resets: { at: "sub:0", lit: true } })) },
    { name: "A heading's Reset held halfway", for: "its body half filled; the label reads either side", sample: screen(() => settingsWindow({ section: POWER, values: seeded(), resets: { at: "sub:0", held: 0.5 } })) },
    { name: "A heading without", for: "everything at its default", sample: screen(CELLS.rest) },
    { name: "The title's Reset lit", for: "anything changed: Hold to reset Power", sample: screen(() => settingsWindow({ section: POWER, values: seeded(), resets: { at: "section", lit: true } })) },
    { name: "The title's Reset held halfway", for: "filling from its start; let go, it empties", sample: screen(() => settingsWindow({ section: POWER, values: seeded(), resets: { at: "section", held: 0.5 } })) },
    { name: "The title without", for: "nothing changed: no Reset", sample: screen(CELLS.rest) },
  ],
  sizes: [
    { name: "Minimum", for: "720 × 480: words give way first", sample: screen(() => settingsWindow({ section: POWER, values: seeded() }, MINIMUM)) },
    { name: "Default", for: "1120 × 720", sample: screen(CELLS.rest) },
  ],
  states: {
    shown: ["rest", "disabled", "error"],
    absent: {
      lit: "a row lights as `ListRow` does",
      pressed: "a row presses as `ListRow` does",
      selected: "a choice's pick is its own state",
      loading: "every value is read as it opens",
      empty: "every sub-section has its settings",
    },
    rows: [{ name: "Power", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "Battery in view",
      disabled: "Power off dimmed above it, its reason its line",
      error: "“soon”: the field red, the line says why",
    },
  },
  focus: doesntApply("no line of its own: its rows and controls light their own."),
  content: {
    text: doesntApply("its words are fixed; names and lines clamp."),
    missingPicture: doesntApply("rows carry no picture."),
    counts: [
      { name: "Keep awake off", for: "one row", sample: lit("keep-awake") },
      { name: "Keep awake on", for: "For shows under it", sample: lit("keep-awake", { keepAwake: true }) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its section's rise, as every section does."),
    leaves: doesntApply("it goes with its section."),
    between: [
      {
        name: "A reset leaves",
        what: "Held full: the value moves as its control does, and the reset fades out over `short`.",
        tracks: [{ part: "the reset", on: "the row's reset", target: "[data-setting='charge-limit'] .st-reset", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "cuts",
        sample: () => settingsWindow(litOn(POWER, "charge-limit", { chargeLimit: 0 })),
      },
      ...deeperMoves(() => settingsWindow({ section: POWER, values: { opened: "stops" } })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓"], does: "the setting above or below" },
      { keys: ["←", "→"], does: "a choice's or a slider's value" },
      { keys: ["Enter", "Space"], does: "a switch flips; a select opens; a field takes the keys" },
      { keys: ["Enter", "Esc"], does: "in a field: back to its row" },
      { keys: ["Esc"], does: "a sub-page: back to Performance" },
      { keys: ["Tab"], does: "on a changed row: its reset, before the control" },
      { keys: ["Enter", "Space"], does: "held on a reset or Reset: puts it back once full" },
    ],
    order: "The title's reset, then each heading's and its wells; a changed row's reset.",
    lands: "On its first row from the sidebar; on the setting from search.",
  },
  measures: {
    samples: nonEmpty([{ name: "Battery", for: "every kind of row in one well", sample: CELLS.rest }]),
    measures: nonEmpty([
      { part: "A setting's row", token: "size-row_two_line" },
      { part: "The control column", token: "size-menu" },
      { part: "A select or field", token: "size-control_md" },
      { part: "A choice", token: "size-control_sm" },
      { part: "A slider", token: "size-slider_compact" },
      { part: "Well to its warning", token: "space-s16" },
      { part: "A row's reset", token: "size-control_sm" },
      { part: "A heading's Reset", token: "size-control_sm" },
      { part: "Reset to its control", token: "space-s12" },
      { part: "Heading to its reset", token: "space-s16" },
    ]),
    reflow: doesntApply("the window's own; Settings shows it."),
  },
});
