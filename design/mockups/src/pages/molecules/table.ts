// The Table page, with TableRow: `docs/design/components/table.md`. Readings in columns, a compact
// row each, numbers lined up at their column's end: the system monitor's processes.

import type { Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { type Column, type TablePinned, type TableRowSpec, table } from "../../lib/table.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { WELL_ROOM } from "./list.ts";

/** The system monitor's columns: the name twice as wide as each number. */
const COLUMNS: readonly Column[] = [
  { title: "Name", weight: 2 },
  { title: "CPU", weight: 1, numeric: true },
  { title: "Memory", weight: 1, numeric: true },
];

const PROCESSES: readonly TableRowSpec[] = [
  { cells: ["firefox", "12.4%", "2104 MB"] },
  { cells: ["Hyprland", "3.1%", "310 MB"], picked: true },
  { cells: ["quickshell", "1.8%", "182 MB"] },
  { cells: ["kitty", "0.6%", "96 MB"] },
  { cells: ["pipewire", "0.4%", "24 MB"] },
  { cells: ["systemd", "0.0%", "12 MB"] },
];

const NAME: Words = { short: "kitty", long: "firefox -contentproc -isForBrowser -prefsLen 31744" };

/** The processes, picking a row as a press or Enter does. */
function picking(width = WELL_ROOM): HTMLElement {
  const room = within(width);
  const draw = (rows: readonly TableRowSpec[], current?: number) => {
    const had = room.querySelector(".k-table") === document.activeElement;
    const shown = table(COLUMNS, rows, {
      ...(current !== undefined ? { current } : {}),
      onPick: (index) => draw(rows.map((row, at) => ({ ...row, picked: at === index })), index),
    });
    room.replaceChildren(shown);
    if (had) shown.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
  };
  draw(PROCESSES);
  return room;
}

/** One row under its header, pinned in a state when `pinned`. */
function alone(spec: TableRowSpec, pinned?: TablePinned): HTMLElement {
  return within(WELL_ROOM, table(COLUMNS, [spec], pinned ? { current: 0, pinned } : {}));
}

const MATRIX = [
  { name: "A process", spec: PROCESSES[0]! },
  { name: "Picked", spec: { ...PROCESSES[1]! } },
] as const;

type MatrixState = "rest" | "lit" | "pressed";

function cell(spec: TableRowSpec, state: MatrixState | "signs-off"): HTMLElement {
  return alone(spec, state === "rest" ? undefined : state);
}

export default page({
  level: "molecules",
  slug: "table",
  title: "Table and TableRow",
  lede: "Readings in columns: a header, then a compact row per entry, numbers lined up at their column's end.",
  code: { rust: "table::table, table::table_row", qml: "Table, TableRow", unbuilt: "tube and keys: not built yet" },
  stage: { live: () => picking("var(--size-column)"), hint: "press a row, or Tab, ↑ ↓ and Enter", daylight: true },
  variants: [
    { name: "Processes", for: "many to a panel: name, CPU and memory", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES)) },
    { name: "With the keyboard", for: "keyboard's row lit, the pick on the secondary container", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES, { current: 3, pinned: "lit" })) },
  ],
  sizes: doesntApply("a row is `size.row_compact` tall; a table is as wide as its place."),
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "a row is a reading, never disabled",
      selected: "a row of the matrix: Picked",
      loading: "its owner holds skeletons in the rows' places",
      empty: "its header and an `EmptyState` in its owner",
      error: "an `ErrorBlock` in its owner",
    },
    rows: nonEmpty(MATRIX.map(({ name, spec }) => ({ name, cell: (state: MatrixState) => cell(spec, state) }))),
    looks: {
      rest: "no fill; picked, `secondary_container`",
      lit: "tube inside its edge, glow at night; picked, on its fill",
      pressed: "tube white-hot; picked lifts to `secondary_container_pressed`",
    },
  },
  focus: {
    tubes: [
      { name: "A row", tone: "accent", filled: false, content: false },
      { name: "The picked row", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "the tube inside the row's edge, the glow over its neighbours",
      signsOff: "the tube as `primary_glass`, no glow",
    },
    signsOff: (index: number) => cell(MATRIX[index]!.spec, "signs-off"),
  },
  content: {
    text: [{ name: "A process's name", words: NAME, sample: (words: string) => within(WELL_ROOM, table(COLUMNS, [{ cells: [words, "0.6%", "96 MB"] }, PROCESSES[0]!])) }],
    missingPicture: doesntApply("a table draws no pictures."),
    counts: [
      { name: "One", for: "a single reading under its header", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES.slice(0, 1))) },
      { name: "Many", for: "no gap between rows, so many fit", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES)) },
      { name: "The widest numbers", for: "`100.0%`, `1023 MB` fit their columns in `mono_large`", sample: () => within(WELL_ROOM, table(COLUMNS, [{ cells: ["stress-ng", "100.0%", "1023 MB"] }, PROCESSES[3]!])) },
    ],
  },
  motion: {
    arrives: doesntApply("a table comes with its panel; readings change in place, with no move."),
    leaves: doesntApply("it goes with its panel."),
    between: [
      { name: "Focus arrives", what: "It lights up over `short` on `standard`, its tube and glow fading in.", tracks: lights(".k-table-row", "a process", 0, 1), reduced: "same", back: "plays", sample: () => alone(PROCESSES[0]!) },
      { name: "Pressed", what: "The tube runs white-hot over `micro`, and back as it comes up; letting go picks it.", tracks: press(".k-table-row", "a process"), reduced: "same", back: "plays", sample: () => alone(PROCESSES[0]!, "lit") },
      ...powerMoves(() => within(WELL_ROOM, table(COLUMNS, PROCESSES.slice(0, 3), { current: 0, pinned: "lit" }))),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↓", "↑", "j", "k", "Ctrl+N", "Ctrl+P"], does: "next or previous row, wrapping round" },
      { keys: ["Home", "End"], does: "the first or last row" },
      { keys: ["Enter"], does: "picks it: its details show" },
    ],
    order: "The table is one stop for Tab; the keys move inside it.",
    lands: "On the picked row, else the one it was last on, else the first.",
  },
  measures: {
    samples: nonEmpty([{ name: "Processes", for: "header and row height, column shares, gaps, numbers", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES.slice(0, 3))) }]),
    measures: [
      { part: "Row height", token: "size-row_compact" },
      { part: "Row and header, above and below", token: "space-s4" },
      { part: "Column gap, row ends", token: "space-s12" },
      { part: "Words' line", token: "type-body-line" },
      { part: "Number's line", token: "type-mono_large-line" },
      { part: "Header's line", token: "type-label-line" },
      { part: "Row corners", token: "radius-row" },
    ],
    reflow: {
      note: "It fills its place, columns by share: narrow, names end in an ellipsis and numbers keep their place.",
      sample: () => stack("s16", "start", within("100%", table(COLUMNS, PROCESSES.slice(0, 3))), within("calc(var(--size-panel) / 2)", table(COLUMNS, PROCESSES.slice(0, 3)))),
    },
  },
});
