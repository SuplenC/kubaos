// Counters (#394): named counters kept in the bar. Its pill opens a sheet that grows down out of the
// bar's corner as the bell's peek does, a row per counter; right-click a row for its menu, Auto reset
// in its place; its history opens inside the sheet. Not built yet.

import type { Words } from "../../lib/content.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page, screen } from "../../site/page.ts";
import { PROMPT_CLOSES, PROMPT_INVALID, PROMPT_OPENS } from "../molecules/prompt.ts";
import {
  anchorOf,
  askFor,
  askNew,
  listPage,
  askStop,
  autoMenu,
  type Counter,
  COUNTERS,
  counterMenu,
  countersList,
  type Editing,
  countersOnScreen,
  countersPill,
  historyPage,
  liveCounters,
  LONG_NAME,
  MANY,
  rowAlone,
  sheetBox,
} from "./counters-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "open" | "disabled" | "empty" | "error";

/** The water glass at its stop: + is a reset. */
const STOPPED: readonly Counter[] = COUNTERS.map((each) => (each.cap !== undefined ? { ...each, value: each.cap } : each));

/** The menu's place on the screen: under the pointer on the second row. */
const AT = { x: "calc(var(--space-s64) * 2)", y: "calc(var(--size-bar) + var(--size-row_two_line) + var(--space-s64))" };

/** The sheet holding the list, `options` on it. */
const listed = (counters: readonly Counter[] = COUNTERS, options: Parameters<typeof countersList>[1] = {}) => sheetBox(listPage(countersList(counters, options).element));

const onList = screen((counters: readonly Counter[] = COUNTERS, options: Parameters<typeof countersList>[1] = {}) => countersOnScreen({ content: listed(counters, options) }));

const withMenu = screen((card: HTMLElement, row = 1) => countersOnScreen({ content: listed(COUNTERS, { menuOpen: row }), over: { card, ...AT } }));

const withHistory = screen((counter: Counter, options: Parameters<typeof historyPage>[1] = {}) => countersOnScreen({ content: sheetBox(historyPage(counter, options)), height: "calc(var(--size-app_min_height) + var(--space-s64) * 3)" }));

/** The list with a prompt for Coffee's name or value under it, `typed` in it, the keyboard in it. */
const withAsk = screen((what: Editing, options: { typed?: string; invalid?: boolean; counters?: readonly Counter[] } = {}) =>
  countersOnScreen({
    content: listed(options.counters ?? COUNTERS),
    prompt: {
      card: askFor(what, (options.counters ?? COUNTERS)[0]!, { pinned: "lit", ...(options.typed !== undefined ? { typed: options.typed } : {}), ...(options.invalid ? { invalid: true } : {}) }),
      anchor: anchorOf(0, what),
      held: what === "value" ? "end" : "start",
    },
  }),
);

/** Auto reset over the water glass's row, with the prompt for its stop under N. */
const withStop = screen((options: { typed?: string; invalid?: boolean } = {}) =>
  countersOnScreen({
    content: listed(COUNTERS, { menuOpen: 1 }),
    over: { card: autoMenu(COUNTERS[1]!, { current: 2 }), ...AT },
    prompt: {
      card: askStop(COUNTERS[1]!, { pinned: "lit", ...(options.typed !== undefined ? { typed: options.typed } : {}), ...(options.invalid ? { invalid: true } : {}) }),
      anchor: ".ct-cap",
      held: "end",
    },
  }),
);

/** The list with the prompt for a new counter's name under New counter. */
const withNew = screen((options: { typed?: string; invalid?: boolean } = {}) =>
  countersOnScreen({
    prompt: {
      card: askNew({ pinned: "lit", ...(options.typed !== undefined ? { typed: options.typed } : {}), ...(options.invalid ? { invalid: true } : {}) }),
      anchor: ".ct-create",
    },
  }),
);

const live = screen(() => liveCounters());

const ON = "the counters' sheet";

const OPENS: Move = {
  name: "Opens",
  what: "The pill clicked: the sheet grows down from the corner as the bell's peek does.",
  tracks: [{ part: "the sheet", on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the sheet", on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => onList(),
};

const CLOSES: Move = {
  name: "Closes",
  what: "The pill again, a click outside, or Esc: it goes back up into the bar.",
  tracks: [{ part: "the sheet", on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: [{ part: "the sheet", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => onList(),
};

const COUNTS: Move = {
  name: "+1",
  what: "+ pressed: only the value's changing figures roll, up, over `short` on `standard`.",
  tracks: [
    { part: "the figure leaving", on: "Coffee's value", target: ".k-roll-figure[data-out]", property: "--roll", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the figure arriving", on: "Coffee's value", target: ".k-roll-figure[data-in]", property: "--roll", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [
    { part: "the figure leaving", on: "Coffee's value", target: ".k-roll-figure[data-out]", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the figure arriving", on: "Coffee's value", target: ".k-roll-figure[data-in]", property: "--roll", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  back: "cuts",
  sample: () => {
    const sample = rowAlone(COUNTERS[0]!, { widest: "000" });
    // The value caught as it starts to roll from 3 to 4.
    const slot = sample.querySelector<HTMLElement>(".k-roll-slot:last-child");
    const old = slot?.querySelector<HTMLElement>(".k-roll-figure");
    if (slot && old) {
      slot.style.setProperty("--way", "1");
      const fresh = old.cloneNode(true) as HTMLElement;
      fresh.textContent = "4";
      old.dataset.out = "";
      fresh.dataset.in = "";
      slot.append(fresh);
    }
    return sample;
  },
};

const HELD: Move = {
  name: "Held to remove",
  what: "Hold to remove held: `error` sweeps across it over `hold`; at its end the counter goes.",
  tracks: [{ part: "the sweep", on: "Hold to remove", target: ".k-menu-item[data-held]", property: "--hold", from: 0, to: 1, duration: "hold", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: () => counterMenu(COUNTERS[1]!, { current: 5, holding: 0 }),
};

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return onList();
    case "lit":
      return onList(COUNTERS, { current: 0 });
    case "pressed":
      return onList(COUNTERS, { current: 0, button: { index: 0, which: "plus", state: "pressed" } });
    case "open":
      return withMenu(counterMenu(COUNTERS[1]!, { current: 0 }));
    case "disabled":
      return onList(COUNTERS, { current: 3 });
    case "empty":
      return countersOnScreen({ content: null, none: true });
    case "error":
      return withAsk("name", { typed: "", invalid: true });
  }
}

const NAME: Words = { short: "Coffee", long: LONG_NAME };

export default page<Shown>({
  level: "shell",
  slug: "counters",
  title: "Counters",
  lede: "Named counters in the bar: a pill opens a sheet of rows, each counted with − and +.",
  code: { unbuilt: "QML: not built yet" },
  from: nonEmpty(["pill", "list", "icon-button", "rolling-number", "prompt", "popover", "menu", "field", "tabs", "empty-state"]),
  stage: { live, hint: "pill opens it; right-click a row; + − F2 Shift+F2 Menu", daylight: true },
  variants: [
    { name: "Open", for: "a row per counter, its rule under its name", sample: () => onList() },
    { name: "At its stop", for: "8 of 8: + turns into a reset", sample: () => onList(STOPPED) },
    { name: "Its menu", for: "right-click: the row stays lit under it", sample: () => withMenu(counterMenu(COUNTERS[1]!, { current: 0 })) },
    { name: "Holding Remove", for: "the sweep part-way; letting go runs it back", sample: () => withMenu(counterMenu(COUNTERS[1]!, { current: 5, holding: 0.45 })) },
    { name: "Auto reset, read", for: "what it read and the next reset", sample: () => withMenu(autoMenu(COUNTERS[0]!, { typed: "every day at 9", fieldLit: true })) },
    { name: "Auto reset, unread", for: "typing: the phrases it reads", sample: () => withMenu(autoMenu(COUNTERS[0]!, { typed: "whenever I feel like it", fieldLit: true })) },
    { name: "Auto reset, can't read", for: "Enter on it: `Invalid`, the reason", sample: () => withMenu(autoMenu(COUNTERS[0]!, { typed: "whenever I feel like it", fieldLit: true, invalid: true })) },
    { name: "Auto reset, empty", for: "the phrases it reads", sample: () => withMenu(autoMenu(COUNTERS[3]!, { typed: "", fieldLit: true })) },
    { name: "When it reaches, asked", for: "the item taken: a Prompt under N", sample: () => withStop() },
    { name: "When it reaches, can't take", for: "0 or not a number: `Invalid`", sample: () => withStop({ typed: "0", invalid: true }) },
    { name: "Renaming", for: "double-click or F2: a Prompt under the name", sample: () => withAsk("name") },
    { name: "Renaming, empty", for: "Enter on it: `Invalid`, the reason", sample: () => withAsk("name", { typed: "", invalid: true }) },
    { name: "Setting the value", for: "double-click it or Shift+F2: under the value", sample: () => withAsk("value") },
    { name: "New counter", for: "under the list: a Prompt for its name", sample: () => withNew() },
    { name: "New counter, empty", for: "Enter on it: `Invalid`, the reason", sample: () => withNew({ typed: "", invalid: true }) },
    { name: "Setting the value, not a count", for: "Enter on it: `Invalid`, the reason", sample: () => withAsk("value", { typed: "3.5", invalid: true }) },
    { name: "History", for: "every change, newest first", sample: () => withHistory(COUNTERS[0]!) },
    { name: "History by day", for: "each day's changes and how much it added", sample: () => withHistory(COUNTERS[0]!, { group: "day" }) },
    { name: "History by week, most added", for: "the busiest week first", sample: () => withHistory(COUNTERS[2]!, { group: "week", sort: "most" }) },
    { name: "History by month", for: "a month a row", sample: () => withHistory(COUNTERS[2]!, { group: "month" }) },
    { name: "History, empty", for: "a counter never changed", sample: () => withHistory(COUNTERS[3]!) },
  ],
  sizes: doesntApply("one size: the sheet `size.panel` wide, rows `size.row_two_line`."),
  states: {
    shown: ["rest", "lit", "pressed", "open", "disabled", "empty", "error"],
    absent: {
      selected: "no counter is picked; the keyboard's row is lit",
      loading: "the counters are read before the pill shows",
    },
    rows: [{ name: "The sheet", cell }],
    looks: {
      rest: "the pill lit while it is open; the rows",
      lit: "the keyboard's row on `secondary_container`",
      pressed: "+ pressed, the value rolling",
      open: "a row's menu at the pointer",
      disabled: "at 0: − dimmed, Reset now dimmed",
      empty: "no counters: no pill, no sheet",
      error: "a Prompt's value it can't take: `Invalid`, the reason",
    },
  },
  focus: doesntApply("no line of its own: the List's well, the Menu, the Field and the Tabs light theirs."),
  content: {
    text: [
      { name: "A counter's name", words: NAME, sample: (words: string) => onList([{ ...COUNTERS[0]!, name: words }, ...COUNTERS.slice(1)]) },
      { name: "A name, renaming", words: NAME, sample: (words: string) => withAsk("name", { counters: [{ ...COUNTERS[0]!, name: words }, ...COUNTERS.slice(1)] }) },
    ],
    missingPicture: doesntApply("its symbols are fixed; it shows no picture."),
    counts: [
      { name: "None", for: "the pill is gone", sample: () => countersOnScreen({ content: null, none: true }) },
      { name: "One", for: "one row; the sheet as tall as it", sample: () => onList([COUNTERS[3]!]) },
      { name: "Twelve", for: "eight show; the list scrolls", sample: () => onList(MANY, { down: 0.5 }) },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: [
      { ...PROMPT_OPENS, name: "A prompt opens", sample: () => withAsk("name") },
      { ...PROMPT_CLOSES, name: "A prompt closes", sample: () => withAsk("name") },
      { ...PROMPT_INVALID, name: "A prompt can't take it", sample: () => withAsk("name", { typed: "" }) },
      COUNTS,
      HELD,
      ...powerMoves(() => onList(COUNTERS, { current: 0 })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "j", "k"], does: "the rows, round" },
      { keys: ["+", "="], does: "+1, or reset at its stop" },
      { keys: ["-"], does: "−1; nothing at 0" },
      { keys: ["F2"], does: "asks for the row's name, under it" },
      { keys: ["Shift+F2"], does: "asks for its value, under it" },
      { keys: ["Enter"], does: "in a Prompt: saves, or shows why it can't" },
      { keys: ["Esc"], does: "cancels a Prompt, closes the menu, leaves history, closes" },
      { keys: ["Menu", "Shift+F10"], does: "the row's menu, under it" },
      { keys: ["Enter held"], does: "on Hold to remove: holds it" },
    ],
    order: "The list, one stop; a Prompt, its Field alone; its menu, Auto reset's Back, type box and choices; history's Back, Group, Sort, list.",
    lands: "Opening, on the list; a Prompt, in its Field, all selected, then back on the list; history, on Back.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A row", for: "name, rule, value, − and +", sample: () => rowAlone(COUNTERS[0]!, { widest: "000" }) },
      { name: "The bar's pill", for: "a lone symbol", sample: () => countersPill() },
      { name: "The menu", for: "the library's popover and items", sample: () => counterMenu(COUNTERS[1]!) },
      { name: "Auto reset", for: "the type box, a Prompt in the menu", sample: () => autoMenu(COUNTERS[0]!, { typed: "every day at 9" }) },
      { name: "A Prompt", for: "the library's popover, label, Field, Note", sample: () => askFor("name", COUNTERS[0]!, { typed: "", invalid: true, pinned: "lit" }) },
      { name: "History", for: "its head, tabs and rows", sample: () => sheetBox(historyPage(COUNTERS[0]!)) },
    ]),
    measures: [
      { part: "Sheet width", token: "size-panel" },
      { part: "Sheet inside, well inside", token: "space-s8" },
      { part: "Row", token: "size-row_two_line" },
      { part: "Between rows", token: "space-s4" },
      { part: "Row sides", token: "space-s16" },
      { part: "Row's parts", token: "space-s12" },
      { part: "− and +", token: "size-control_sm" },
      { part: "Between − and +", token: "space-s8" },
      { part: "Value line", token: "type-mono_large-line" },
      { part: "History row", token: "size-row" },
      { part: "Menu and Prompt width", token: "size-menu" },
      { part: "Prompt under its text", token: "space-s8" },
    ],
    reflow: doesntApply("it keeps the corner and `size.panel` on every screen; past eight rows the list scrolls."),
  },
});
