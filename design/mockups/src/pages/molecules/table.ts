// The Table page, with TableRow: `docs/design/components/table.md`. Readings in columns, a compact
// row each, numbers lined up at their column's end: the system monitor's processes.

import type { Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { type Column, type TablePinned, type TableRowSpec, table } from "../../lib/table.ts";
import { text } from "../../lib/text.ts";
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
    if (had) shown.focus({ focusVisible: true } as FocusOptions);
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
  summary: "Readings in columns: a header, then a compact row per entry, its numbers lined up at their column's end.",
  live: () => picking(),
  description: [
    "Rust `table::table` and `table::table_row`, QML `Table` and `TableRow`. Press a row, or Tab in and use ↑ ↓ and Enter, to pick it. The system monitor's processes.",
    "A header in `label`, `on_surface_variant`, `space.s4` above and below, then a row per entry: a line of `body` with `space.s4` above and below (`size.row_compact`, 28), so many fit, with no gap between rows. Columns take set shares of the width (the name 2, each number 1), `space.s12` apart and `space.s12` in from the row's ends, split in whole device pixels so no two differ by more than one past their shares. Words sit at their column's start on one line, `on_surface`, ending in an ellipsis where they don't fit; numbers sit at their column's end in `mono_large`, `on_surface_variant`, so their digits line up, as their header does.",
    "The whole row is one control at the row radius (`radius.row`, 8), lit as a Text button is: the pointer on it, or the keyboard on it, lights its tube inside its edge, `primary` at `focus.tube`, with the glow outside at night; pressed, the tube runs white-hot and the glow brightens, and by day the tone tints it. The pointer moving onto a row puts the keyboard on it, so one row is lit at a time. The picked row (the process whose details show) is `secondary_container`, its words `on_secondary_container` and its numbers that at `opacity.subdued_text`; lit, its tube burns on that fill, as a Tonal button's; pressed, the container lifts to `secondary_container_pressed`.",
    "It doesn't sort: the system monitor orders its processes by CPU itself. The table is one stop for Tab; the keys move the lit row and Enter picks it.",
  ],
  variants: [
    { name: "Processes", for: "Readings that change in place, many to a panel: the name, CPU and memory.", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES)) },
    { name: "With the keyboard", for: "The row the keyboard is on lit, the picked one on the secondary container.", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES, { current: 3, pinned: "lit" })) },
  ],
  sizes: doesntApply("a row is always `size.row_compact` tall; a table is as wide as its place, its columns that place's shares."),
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "a row is a reading, never disabled; a process that can't be acted on still shows its details.",
      selected: "a picked row is its own row of the matrix: Picked.",
      loading: "the first reading is there as the panel opens; a table still waiting has its rows' places as skeletons in its owner.",
      empty: "a table with no rows shows its header and an EmptyState in its owner.",
      error: "a reading can't be wrong; a source that failed is an ErrorBlock in its owner.",
    },
    rows: nonEmpty(MATRIX.map(({ name, spec }) => ({ name, cell: (state: MatrixState) => cell(spec, state) }))),
    described: {
      rest: "No fill, its words `on_surface` and its numbers `on_surface_variant`; picked, `secondary_container` under `on_secondary_container`. By day the same roles in daylight colours.",
      lit: "The pointer or the keyboard on it: its tube inside its edge, `primary` at `focus.tube`, the glow outside at night; by day the tube with no glow. Picked, the tube burns on its fill.",
      pressed: "Lit and pressed: the tube runs white-hot and the glow brightens; by day the tone tints the row at `opacity.press_tint`. Picked, the container lifts to `secondary_container_pressed`. Letting go picks it.",
    },
  },
  focus: {
    tubes: [
      { name: "A row", tone: "accent", filled: false, content: false },
      { name: "The picked row", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the row's edge at the row radius, the glow outside it, over the rows round it.",
      signsOff: "By day always, and at night while a power change has the lights out: the tube is solid glass, `primary_glass`, with no glow.",
    },
    signsOff: (index: number) => cell(MATRIX[index]!.spec, "signs-off"),
  },
  content: {
    text: [{ name: "A process's name", words: NAME, sample: (words: string) => within(WELL_ROOM, table(COLUMNS, [{ cells: [words, "0.6%", "96 MB"] }, PROCESSES[0]!])) }],
    missingPicture: doesntApply("a table draws no pictures: its cells are words and numbers."),
    counts: [
      { name: "One", for: "A single reading under its header.", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES.slice(0, 1))) },
      { name: "Many", for: "Rows with no gap between them, so many fit.", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES)) },
      { name: "The widest numbers", for: "\"100.0%\" and \"1023 MB\": the numbers' columns are wide enough for them in `mono_large`.", sample: () => within(WELL_ROOM, table(COLUMNS, [{ cells: ["stress-ng", "100.0%", "1023 MB"] }, PROCESSES[3]!])) },
    ],
  },
  motion: {
    arrives: doesntApply("a table comes with its panel; new readings change the cells in place, with no move."),
    leaves: doesntApply("it goes with its panel."),
    between: [
      { name: "Focus arrives", what: "The pointer or the keyboard onto a row: it lights up over `short` on `standard`, its tube and glow fading in.", tracks: lights(".k-table-row", "a process", 0, 1), reduced: "same", back: "plays", sample: () => alone(PROCESSES[0]!) },
      { name: "Pressed", what: "A lit row pressed: the tube runs white-hot over `micro`, and back as it comes up; letting go picks it.", tracks: press(".k-table-row", "a process"), reduced: "same", back: "plays", sample: () => alone(PROCESSES[0]!, "lit") },
      ...powerMoves(() => within(WELL_ROOM, table(COLUMNS, PROCESSES.slice(0, 3), { current: 0, pinned: "lit" }))),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↓", "↑", "j", "k", "Ctrl+N", "Ctrl+P"], does: "The next or the row before, round from either end." },
      { keys: ["Home", "End"], does: "The first or the last row." },
      { keys: ["Enter"], does: "Picks the row, once per press: its details show." },
    ],
    order: "The table is one stop for Tab; the keys move inside it.",
    lands: "Tabbing in, on the picked row, else the row it was last on, else the first.",
  },
  measures: {
    samples: nonEmpty([{ name: "Processes", for: "The header's and a row's height and padding, the columns' shares and gaps, the numbers at their ends.", sample: () => within(WELL_ROOM, table(COLUMNS, PROCESSES.slice(0, 3))) }]),
    measures: [
      { what: "A row's height.", token: "size-row_compact" },
      { what: "Above and below a row's line and the header's.", token: "space-s4" },
      { what: "Between columns, and in from the row's ends.", token: "space-s12" },
      { what: "A row's words, `body`'s line.", token: "type-body-line" },
      { what: "A number's line, `mono_large`'s.", token: "type-mono_large-line" },
      { what: "The header's line, `label`'s.", token: "type-label-line" },
      { what: "A row's corners.", token: "radius-row" },
    ],
    reflow: {
      described: "A table fills its place at every window size, its columns that width's shares: wider, the names show whole; narrow, they end in an ellipsis and the numbers keep their place at their columns' ends. Here as wide as the window, then in half a panel.",
      sample: () => stack("s16", "start", within("100%", table(COLUMNS, PROCESSES.slice(0, 3))), within("calc(var(--size-panel) / 2)", table(COLUMNS, PROCESSES.slice(0, 3))), text("As wide as the window, then half a panel.", "small", "on_surface_variant")),
    },
  },
});
