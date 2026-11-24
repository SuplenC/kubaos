// Settings: the window. Every KubaOS setting has a control here, in fourteen sections on SidebarPage:
// it opens on Home, which is search; a section's sub-sections open under it in the sidebar, which
// follows the scroll; search lands on the exact row and lights it; the Command menu opens the window
// on one row. Every section has a page of its own beside this one, the window's.

import { doesntApply, nonEmpty, page, screen } from "../../../site/page.ts";
import { turnMoves } from "../../templates/sidebar-turns.ts";
import { APPEARANCE, APPS, POWER } from "./settings-data.ts";
import { bareWindow, FULL_HD, landedOn, LAPTOP, liveWindow, MINIMUM, settingsWindow } from "./settings-window.ts";

type Shown = "rest" | "selected" | "empty";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: () => settingsWindow(),
  selected: () => settingsWindow({ section: POWER, sub: 1 }),
  empty: () => settingsWindow({ query: "teleport", keyboard: "home" }),
};

export default page<Shown>({
  level: "apps",
  slug: "settings",
  title: "Settings",
  lede: "The system's settings, never an app's own, in fourteen sections; search and the Command menu land on the row.",
  code: { unbuilt: "kubaos-settings: not built yet" },
  from: ["sidebar-page", "list", "tabs", "slider", "select", "field", "switch", "banner", "keycap"],
  stage: {
    live: (mode) => liveWindow({ values: { drawn: mode } }),
    hint: "type “lock”, Enter; hold a reset",
    daylight: true,
  },
  variants: [
    { name: "Home", for: "where it opens: the field, the sections", sample: screen(CELLS.rest) },
    { name: "Searching", for: "every setting matching “lo”, path and value", sample: screen(() => settingsWindow({ query: "lo", keyboard: "home" })) },
    { name: "Found by a tip", for: "“super ctrl i”: Keep awake, by its tip's keys", sample: screen(() => settingsWindow({ query: "super ctrl i", keyboard: "home" })) },
    { name: "Found by a section's tip", for: "“super n”: Notifications › Popups, by its heading's tip", sample: screen(() => settingsWindow({ query: "super n", keyboard: "home" })) },
    { name: "Found by a line", for: "“stutter”: by the words under its name", sample: screen(() => settingsWindow({ query: "stutter", keyboard: "home" })) },
    { name: "Landed from search", for: "Power › Sleep & lock, Lock after lit", sample: screen(() => settingsWindow(landedOn(POWER, "lock"))) },
    { name: "From the Command menu", for: "“pointer size”: the window opens on its row", sample: screen(() => settingsWindow(landedOn(APPEARANCE, "cursor-size"))) },
    { name: "A section, scrolled", for: "Power at Performance; the bar beside it", sample: screen(() => settingsWindow({ section: POWER, sub: 2 })) },
    { name: "Find on a page", for: "“after” on Power: three rows, the second", sample: screen(() => settingsWindow({ section: POWER, find: "after", findAt: 1, keyboard: "find" })) },
  ],
  sizes: [
    { name: "Minimum", for: "720 × 480: the page keeps 456", sample: screen(() => settingsWindow({ section: POWER }, MINIMUM)) },
    { name: "Default", for: "1120 × 720, where it opens floating", sample: screen(() => settingsWindow({ section: POWER })) },
    { name: "Laptop", for: "1280 × 800, tiled alone", sample: screen(() => settingsWindow({ section: POWER }, LAPTOP)) },
    { name: "Full HD", for: "1920 × 1080: rows at their widest", sample: screen(() => settingsWindow({ section: POWER }, FULL_HD)) },
  ],
  states: {
    shown: ["rest", "selected", "empty"],
    absent: {
      lit: "its rows and fields light on their own pages",
      pressed: "its rows and controls press on their own pages",
      disabled: "a row disables itself; Power shows one",
      loading: "every setting is read as the window opens",
      error: "a field says its own; Power shows one",
    },
    rows: [{ name: "The window", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "Home: the field, then the fourteen sections",
      selected: "Power current, Sleep & lock under the bar",
      empty: "nothing found: a line under the field",
    },
  },
  focus: doesntApply("no line of its own: its rows, wells, fields and controls light their own."),
  content: {
    text: doesntApply("the words are the settings' own; each row clamps its line."),
    missingPicture: doesntApply("every section has its symbol; it draws no picture."),
    counts: [
      { name: "Nothing found", for: "“teleport”: a line says so", sample: screen(CELLS.empty) },
      { name: "One found", for: "“charge”: Charge limit alone", sample: screen(() => settingsWindow({ query: "charge", keyboard: "home" })) },
      { name: "Fourteen sections", for: "at the minimum the sidebar scrolls", sample: screen(() => settingsWindow({ section: APPS }, MINIMUM)) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its window, on Home, nothing moving."),
    leaves: doesntApply("it goes with its window."),
    between: turnMoves({
      section: () => settingsWindow({ section: POWER }),
      home: () => settingsWindow(),
      landed: () => settingsWindow(landedOn(POWER, "lock")),
    }),
  },
  keyboard: {
    keys: [
      { keys: ["Super+Comma"], does: "opens the window on Home, in the field" },
      { keys: ["Tab", "Shift+Tab"], does: "round the stops: field or find box, sidebar, page" },
      { keys: ["↑", "↓", "j", "k"], does: "through the sidebar's or the page's rows" },
      { keys: ["Enter"], does: "Home: the result; a row: its control" },
      { keys: ["←", "→"], does: "on a choice or slider: the value; else sidebar and page" },
      { keys: ["Ctrl+Page Up", "Ctrl+Page Down"], does: "the section before or after" },
      { keys: ["/", "Ctrl+F"], does: "Home's field; on a section, the find box" },
      { keys: ["Esc"], does: "a sub-page: back; else the template's" },
      { keys: ["Enter", "Space"], does: "held on a reset: row, heading or page back" },
    ],
    order: "Home's field or the find box, the sidebar, then the page's wells in reading order.",
    lands: "Opened plain, in Home's field; from the Command menu or a result, on the setting's row.",
  },
  measures: {
    samples: nonEmpty([{ name: "Power at the default size", for: "the sidebar, the rows and their controls", sample: CELLS.selected }]),
    measures: nonEmpty([
      { part: "Sidebar's width", token: "size-sidebar" },
      { part: "A setting's row", token: "size-row_two_line" },
      { part: "The control column", token: "size-menu" },
      { part: "Words to control", token: "space-s12" },
      { part: "A choice or select", token: "size-control_sm" },
      { part: "Wells apart", token: "space-s16" },
      { part: "Sub-sections apart", token: "space-s32" },
      { part: "A page turn's travel", token: "space-s24" },
      { part: "A row's reset; a heading's Reset", token: "size-control_sm" },
      { part: "The default window", token: "size-app_width" },
      { part: "The least window", token: "size-app_min_width" },
    ]),
    reflow: { note: "The sidebar keeps its width; the page takes the rest; controls keep their column.", sample: () => bareWindow({ section: POWER }) },
  },
});
