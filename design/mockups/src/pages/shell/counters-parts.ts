// Counters (#394), as the shell is to draw them: the bar's pill, the sheet that grows down out of the
// bar's corner as the bell's peek does, a counter's row, its right-click menu with Auto reset in
// place, and the history page inside the sheet. This file is no page. The data is the shots' day:
// Thursday 12 November 2026, 14:32.

import "./counters.css";
import { type Pinned, type Shown, stateOptions } from "../../lib/control.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { withHint } from "../../lib/hint.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { filling, row, stack } from "../../lib/layout.ts";
import { flush, listRow, listRows, scrolled, well } from "../../lib/list.ts";
import { type MenuEntry, menu, menuSeparator } from "../../lib/menu.ts";
import { pill } from "../../lib/pill.ts";
import { floatUnder, type PromptHeld, prompt } from "../../lib/prompt.ts";
import { rollingNumber } from "../../lib/rolling-number.ts";
import { hang, screen, sheet } from "../../lib/sheet.ts";
import { tabs } from "../../lib/tabs.ts";
import { clamped, text } from "../../lib/text.ts";
import { popover } from "../../lib/tooltip.ts";
import { barButtons } from "../templates/panels.ts";
import { glyph, wallpaper } from "./bar-parts.ts";

// ---------------------------------------------------------------- the data --

export type ChangeKind = "+1" | "−1" | "set" | "reset" | "auto reset";

export interface Change {
  /** Minutes since the shots' day began, 0:00 on Thursday 12 November 2026; earlier days negative. */
  readonly at: number;
  readonly kind: ChangeKind;
  /** The value after it. */
  readonly value: number;
}

export interface Counter {
  readonly name: string;
  readonly value: number;
  /** Its auto reset rule as it reads back: "every day at 9:00". */
  readonly rule?: string;
  /** It stops at this and + resets it. */
  readonly cap?: number;
  readonly log?: readonly Change[];
}

/** The minutes of the shots' moment: 14:32. */
export const NOW = 14 * 60 + 32;

/** A log of `count` changes ending at `value`, a few hours apart, the same on every machine. */
function logOf(count: number, value: number, cap?: number): Change[] {
  const changes: Change[] = [];
  let seed = 7;
  const next = () => (seed = (seed * 48271) % 2147483647) / 2147483647;
  let at = NOW - 10;
  let after = value;
  for (let i = 0; i < count; i++) {
    const draw = next();
    const kind: ChangeKind = i === 0 ? "set" : draw < 0.74 ? "+1" : draw < 0.88 ? "−1" : draw < 0.95 ? "reset" : "auto reset";
    changes.push({ at, kind, value: after });
    // The value before this change, walking back.
    if (kind === "+1") after = Math.max(0, after - 1);
    else if (kind === "−1") after += 1;
    else after = Math.min(cap ?? 99, Math.floor(next() * 9) + 1);
    at -= Math.floor(next() * 300) + 40;
  }
  return changes;
}

export const COUNTERS: readonly Counter[] = [
  { name: "Coffee", value: 3, rule: "every day at 9:00", log: logOf(64, 3) },
  { name: "Glasses of water", value: 7, cap: 8, log: logOf(40, 7, 8) },
  { name: "Push-ups", value: 120, rule: "every Monday at 0:00", log: logOf(80, 120) },
  { name: "Counter", value: 0, log: [] },
];

/** A name long enough to be cut. */
export const LONG_NAME = "Times I reached for the phone before getting out of bed this week";

/** Twelve counters: more than the sheet shows at once. */
export const MANY: readonly Counter[] = [
  ...COUNTERS,
  ...["Steps upstairs", "Pomodoros", "Books this year", "Bugs fixed", "Walks", "Calls home", "Stretches", "Plants watered"].map((name, index) => ({ name, value: (index * 7) % 23, log: [] })),
];

/** The rows' value box: as wide as the widest value or stop any of them shows, every figure a 0. */
export function widestOf(counters: readonly Counter[]): string {
  const most = Math.max(1, ...counters.map((each) => Math.max(String(each.value).length, String(each.cap ?? 0).length)));
  return "0".repeat(most);
}

/** The second line of a counter's row: its rule, or where it stops; none without either. */
export function ruleLine(counter: Counter): string | undefined {
  if (counter.cap !== undefined) return `Resets at ${counter.cap}`;
  return counter.rule ? `Resets ${counter.rule}` : undefined;
}

// ---------------------------------------------------------------- the pill --

/** The bar's pill: the Tabler `tallymarks` symbol alone. */
export function countersPill(state: Shown = "rest"): HTMLElement {
  const made = pill({ content: [glyph("tallymarks")], lone: true, interactive: true, tooltip: "Counters", ...stateOptions(state) });
  made.classList.add("ct-pill");
  return made;
}

// ---------------------------------------------------------------- a row --

export interface RowOptions {
  /** The value box's widest value, every figure a 0. */
  readonly widest: string;
  readonly current?: boolean;
  readonly pinned?: "lit" | "pressed";
  /** Its place in the list, which a still's prompt finds it by. */
  readonly index?: number;
  /** Its row's menu is open: the row stays lit under it. */
  readonly menuOpen?: boolean;
  /** One of its buttons pinned: the − or the +. */
  readonly button?: { readonly which: "minus" | "plus"; readonly state: Pinned };
  readonly onChange?: (value: number, kind: ChangeKind) => void;
  readonly onRename?: (name: string) => void;
  /** Its name or its value asked for: the sheet floats a prompt under `anchor`. */
  readonly onEdit?: (what: Editing, anchor: HTMLElement, row: CounterRow) => void;
}

/** What a counter's prompt changes: its name, or its value. */
export type Editing = "name" | "value";

export interface CounterRow {
  readonly element: HTMLElement;
  readonly add: (by: 1 | -1) => void;
  readonly reset: () => void;
  readonly set: (value: number, kind?: ChangeKind) => void;
  /** Asks for its name: F2, a double-click on it, Rename in its menu. */
  readonly rename: () => void;
  /** Asks for its value: Shift+F2, a double-click on it. */
  readonly edit: () => void;
  readonly setName: (name: string) => void;
  readonly name: () => string;
  readonly value: () => number;
}

/**
 * A counter's row: a two-line ListRow, its name and its rule's line, then at its end the value, −
 * (Tonal, disabled at 0) and + (Filled), which turns into a reset at the counter's stop.
 */
export function counterRow(counter: Counter, options: RowOptions): CounterRow {
  let value = counter.value;
  const rule = ruleLine(counter);
  const element = listRow({ title: counter.name, ...(rule ? { subtitle: rule, subtitleIcon: "repeat" } : { tall: true }) }, options.current ?? false, options.pinned);
  element.classList.add("ct-row");
  element.toggleAttribute("data-menu", options.menuOpen ?? false);
  const line = element.firstElementChild as HTMLElement;

  if (options.index !== undefined) element.dataset.index = String(options.index);

  // The name: a hint says a double-click renames it, after the whole name when it is cut.
  let name = counter.name;
  const nameBox = document.createElement("span");
  nameBox.className = "ct-name";
  const nameOf = (words: string) => {
    const made = clamped(words, "body_strong");
    made.classList.add("k-list-title");
    made.style.color = "var(--title)";
    requestAnimationFrame(() => {
      const cut = made.scrollWidth > made.clientWidth + 0.5;
      withHint(nameBox, cut ? `${words}. Double-click to rename` : "Double-click to rename", { held: "start" });
    });
    return made;
  };
  nameBox.append(nameOf(name));
  element.querySelector<HTMLElement>(".k-list-title")!.replaceWith(nameBox);

  // The value, rolling; a double-click asks for a new one.
  const rolling = rollingNumber({ value, widest: options.widest });
  rolling.element.classList.add("ct-value");
  withHint(rolling.element, "Double-click to set");
  const pinnedOf = (which: "minus" | "plus") => (options.button?.which === which ? { pinned: options.button.state } : {});
  const minus = iconButton({ icon: "remove", label: "−1", kind: "Tonal", size: "Sm", disabled: value === 0, ...pinnedOf("minus") });
  const full = () => counter.cap !== undefined && value >= counter.cap;
  const plusOf = () =>
    full()
      ? iconButton({ icon: "restart_alt", label: `Reached ${counter.cap}: reset to 0`, kind: "Filled", size: "Sm", ...pinnedOf("plus") })
      : iconButton({ icon: "add", label: "+1", kind: "Filled", size: "Sm", ...pinnedOf("plus") });
  let plus = plusOf();
  const buttons = row("s8", "centre", minus, plus);
  buttons.classList.add("ct-buttons");
  line.append(rolling.element, buttons);

  function set(to: number, kind: ChangeKind): void {
    const before = full();
    value = Math.max(0, Math.min(to, counter.cap ?? Number.POSITIVE_INFINITY));
    rolling.set(value);
    minus.disabled = value === 0;
    if (before !== full()) {
      const fresh = plusOf();
      plus.replaceWith(fresh);
      plus = fresh;
      wire();
    }
    options.onChange?.(value, kind);
  }
  const add = (by: 1 | -1) => (by > 0 && full() ? set(0, "reset") : set(value + by, by > 0 ? "+1" : "−1"));
  function wire(): void {
    plus.addEventListener("click", (event) => {
      event.stopPropagation();
      add(1);
    });
  }
  wire();
  minus.addEventListener("click", (event) => {
    event.stopPropagation();
    add(-1);
  });
  // The buttons take their own pointer: a press on them neither lights nor takes the row.
  for (const each of [minus, buttons]) each.addEventListener("dblclick", (event) => event.stopPropagation());

  const made: CounterRow = {
    element,
    add,
    reset: () => set(0, "reset"),
    set: (to, kind = "set") => set(to, kind),
    rename: () => options.onEdit?.("name", nameBox.firstElementChild as HTMLElement, made),
    edit: () => options.onEdit?.("value", rolling.element, made),
    setName: (words) => {
      name = words;
      nameBox.replaceChildren(nameOf(words));
      nameBox.classList.remove("k-hinted");
      options.onRename?.(words);
    },
    name: () => name,
    value: () => value,
  };
  nameBox.addEventListener("dblclick", (event) => {
    event.stopPropagation();
    made.rename();
  });
  rolling.element.addEventListener("dblclick", (event) => {
    event.stopPropagation();
    made.edit();
  });
  return made;
}

/** Why a name can't be taken, or null: it can't be empty. */
export function checkName(typed: string): string | null {
  return typed.trim() === "" ? "A name can't be empty." : null;
}

/** Why a count can't be taken, or null: a whole number, 0 or more. */
export function checkCount(typed: string): string | null {
  return /^\s*\d+\s*$/.test(typed) ? null : "A whole number, 0 or more.";
}

/** Why a stop can't be taken, or null: a whole number, 1 or more. */
export function checkStop(typed: string): string | null {
  return /^\s*\d+\s*$/.test(typed) && Number(typed) > 0 ? null : "A whole number, 1 or more.";
}

export interface AskOptions {
  readonly typed?: string;
  readonly pinned?: Pinned;
  readonly invalid?: boolean;
  readonly onSave?: (typed: string) => void;
  readonly onCancel?: () => void;
}

/** The prompt for a counter's name or value, `typed` in it, all of it selected. */
export function askFor(what: Editing, counter: Counter, options: AskOptions = {}): HTMLElement {
  const typed = options.typed ?? (what === "name" ? counter.name : String(counter.value));
  return prompt({
    label: what === "name" ? "Rename" : "Set value",
    value: typed,
    check: what === "name" ? checkName : checkCount,
    ...(options.pinned ? { pinned: options.pinned } : {}),
    ...(options.invalid ? { invalid: true } : {}),
    ...(options.onSave ? { onSave: options.onSave } : {}),
    ...(options.onCancel ? { onCancel: options.onCancel } : {}),
  }).element;
}

// ---------------------------------------------------------------- the list --

/** The most rows the sheet shows before its list scrolls. */
export const MOST_ROWS = 8;

export interface ListOptions {
  readonly current?: number;
  readonly pinned?: "lit" | "pressed";
  readonly menuOpen?: number;
  readonly button?: { readonly index: number; readonly which: "minus" | "plus"; readonly state: Pinned };
  /** Scrolled this share of the way down. */
  readonly down?: number;
  readonly live?: LiveHooks;
}

/** What a live list tells its sheet. */
export interface LiveHooks {
  readonly onMenu: (index: number, at: { x: number; y: number }, rows: readonly CounterRow[]) => void;
  readonly onChange: (index: number, value: number, kind: ChangeKind) => void;
  readonly onRename: (index: number, name: string) => void;
  readonly onEdit: (index: number, what: Editing, anchor: HTMLElement, row: CounterRow) => void;
}

/** The counters' rows, flush on the sheet; past `MOST_ROWS` they scroll. */
export function countersList(counters: readonly Counter[], options: ListOptions = {}): { element: HTMLElement; rows: CounterRow[] } {
  const widest = widestOf(counters);
  const rows = counters.map((counter, index) =>
    counterRow(counter, {
      widest,
      current: options.current === index,
      ...(options.current === index && options.pinned ? { pinned: options.pinned } : {}),
      index,
      menuOpen: options.menuOpen === index,
      ...(options.button?.index === index ? { button: { which: options.button.which, state: options.button.state } } : {}),
      ...(options.live
        ? {
            onChange: (value: number, kind: ChangeKind) => options.live!.onChange(index, value, kind),
            onRename: (name: string) => options.live!.onRename(index, name),
            onEdit: (what: Editing, anchor: HTMLElement, row: CounterRow) => options.live!.onEdit(index, what, anchor, row),
          }
        : {}),
    }),
  );
  const column = stack("s4", "start", ...rows.map((each) => each.element));
  column.classList.add("k-list-rows");
  let box: HTMLElement;
  let pane: HTMLElement | null = null;
  if (counters.length > MOST_ROWS) {
    const made = scrolled("ScrollingList", column, { height: `calc(${MOST_ROWS} * var(--size-row_two_line) + ${MOST_ROWS - 1} * var(--space-s4) + 2 * var(--space-s12))` });
    box = made.box;
    pane = made.pane;
    if (options.down !== undefined) {
      const down = options.down;
      requestAnimationFrame(() => {
        pane!.scrollTop = Math.round((pane!.scrollHeight - pane!.clientHeight) * down);
        made.thumb();
      });
    }
  } else box = well("List", column);
  flush(box);
  const keys = listRows(box, pane, () => rows.map((each) => each.element), {
    ...(options.current !== undefined ? { current: options.current } : {}),
    ...(options.pinned ? { pinned: options.pinned } : {}),
    vimKeys: true,
  });
  if (options.live) {
    const live = options.live;
    rows.forEach((each, index) => {
      each.element.addEventListener("contextmenu", (event) => {
        event.preventDefault();
        live.onMenu(index, { x: event.clientX, y: event.clientY }, rows);
      });
    });
    box.addEventListener("keydown", (event) => {
      const at = rows.findIndex((each) => each.element === keys.current());
      const counter = rows[at];
      if (!counter) return;
      if (event.key === "+" || event.key === "=") counter.add(1);
      else if (event.key === "-") counter.add(-1);
      else if (event.key === "F2" && event.shiftKey) counter.edit();
      else if (event.key === "F2") counter.rename();
      else if (event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey)) {
        const box_ = counter.element.getBoundingClientRect();
        live.onMenu(at, { x: box_.left + box_.width / 2, y: box_.bottom }, rows);
      } else return;
      event.preventDefault();
      event.stopPropagation();
    });
  }
  return { element: box, rows };
}

// ---------------------------------------------------------------- the menu --

/** A counter's menu: Rename, Reset now (not at 0), Auto reset, History, then Hold to remove. */
export function menuEntries(counter: Counter): MenuEntry[] {
  return [
    { label: "Rename", icon: "draw" },
    { label: "Reset now", icon: "restart_alt", disabled: counter.value === 0 },
    { label: "Auto reset", icon: "repeat", submenu: true },
    { label: "History", icon: "history" },
    "separator",
    { label: "Hold to remove", icon: "delete", held: true },
  ];
}

/** What a counter's menu tells the sheet. */
export interface MenuHooks {
  readonly onTake: (what: "rename" | "reset" | "auto" | "history" | "remove") => void;
  readonly onLeave: () => void;
}

const TAKES = ["rename", "reset", "auto", "history", "", "remove"] as const;

/** A counter's menu on its popover. */
export function counterMenu(counter: Counter, options: { current?: number; holding?: number; hooks?: MenuHooks } = {}): HTMLElement {
  const entries = menuEntries(counter).map((entry) => (entry !== "separator" && entry.held && options.holding !== undefined ? { ...entry, holding: options.holding } : entry));
  const list = menu(entries, {
    ...(options.current !== undefined ? { current: options.current, ...(options.holding !== undefined ? { pinned: "lit" as const } : {}) } : {}),
    ...(options.hooks
      ? {
          onTake: (index: number) => {
            const what = TAKES[index];
            if (what) options.hooks!.onTake(what);
          },
          onLeave: options.hooks.onLeave,
        }
      : {}),
  });
  const card = popover(2, list);
  card.classList.add("ct-menu");
  return card;
}

// ---------------------------------------------------------------- auto reset --

/** What the type box read: the rule and its next reset, or nothing it can read. */
export interface Reading {
  readonly rule: string;
  readonly next: string;
}

const DAYS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/** The shots' day as a date: Thursday 12 November 2026. */
function day(offset: number): string {
  const date = new Date(2026, 10, 12 + offset);
  return `${DAYS[date.getDay()]!.slice(0, 3)} ${date.getDate()} ${MONTHS[date.getMonth()]!.slice(0, 3)}`;
}

function ordinal(n: number): string {
  const tens = n % 100;
  return `${n}${tens >= 11 && tens <= 13 ? "th" : (["th", "st", "nd", "rd"][n % 10] ?? "th")}`;
}

/**
 * The rule `typed` reads as, the way the clock's query box reads a date: a stand-in for the
 * recurrence reader `kubaos-clock` is to have. Null where it can't read it.
 */
export function readRule(typed: string): Reading | null {
  let words = typed.toLowerCase().trim().replace(/\beveryday\b|\bdaily\b/, "every day").replace(/\bweekly\b/, "every week").replace(/\bmonthly\b/, "every month");
  if (!words) return null;
  let minutes = 0;
  let timed = false;
  const time = words.match(/(?:\bat\s*)?(\d{1,2})(?::(\d{2}))?\s*(am|pm)?$/);
  const ordinalEnd = /\d+(st|nd|rd|th)$/.test(words);
  const every = words.match(/^every (\d+) (hour|minute)s?$/);
  if (time && !every && !ordinalEnd) {
    let hours = Number(time[1]);
    if (time[3] === "pm" && hours < 12) hours += 12;
    if (time[3] === "am" && hours === 12) hours = 0;
    const mins = Number(time[2] ?? 0);
    if (hours > 23 || mins > 59) return null;
    minutes = hours * 60 + mins;
    timed = true;
    words = words.slice(0, time.index).replace(/\s*\bat\s*$/, "").trim();
  }
  const clock = `${Math.floor(minutes / 60)}:${String(minutes % 60).padStart(2, "0")}`;
  if (every) {
    const n = Number(every[1]);
    const step = every[2] === "hour" ? n * 60 : n;
    const next = NOW + step;
    return { rule: `every ${n} ${every[2]}${n === 1 ? "" : "s"}`, next: `today ${Math.floor(next / 60) % 24}:${String(next % 60).padStart(2, "0")}` };
  }
  if (words === "every day" || (words === "" && timed)) return { rule: `every day at ${clock}`, next: `${minutes > NOW ? "today" : `tomorrow, ${day(1)},`} ${clock}` };
  const weekday = words.match(/^(?:every )?(sun|mon|tue|wed|thu|fri|sat)[a-z]*?s?$/);
  if (weekday) {
    const index = DAYS.findIndex((each) => each.toLowerCase().startsWith(weekday[1]!));
    const ahead = (index - 4 + 7) % 7 || (minutes > NOW ? 0 : 7);
    return { rule: `every ${DAYS[index]} at ${clock}`, next: `${day(ahead)}, ${clock}` };
  }
  if (words === "every week") return { rule: `every Monday at ${clock}`, next: `${day(4)}, ${clock}` };
  const monthly = words.match(/^(?:every month|(?:the )?(\d+)(?:st|nd|rd|th)?(?: of (?:the|every) month)?)$/);
  if (monthly) {
    const n = Number(monthly[1] ?? 1);
    if (n < 1 || n > 31) return null;
    const ahead = n > 12 ? n - 12 : n - 12 + 30;
    return { rule: `on the ${ordinal(n)} of every month at ${clock}`, next: `${day(ahead)}, ${clock}` };
  }
  return null;
}

/** The phrases shown when the type box can't read what is in it. */
export const EXAMPLES = ["every day at 9", "mondays 8am", "every 2 hours", "1st of the month"];

export interface AutoOptions {
  readonly typed?: string;
  /** The type box pinned lit, as with the keyboard in it. */
  readonly fieldLit?: boolean;
  /** Drawn after Enter on what it can't read. */
  readonly invalid?: boolean;
  /** The item the keyboard is on among Back, Never and When it reaches. */
  readonly current?: number;
  readonly hooks?: AutoHooks;
}

export interface AutoHooks {
  readonly onBack: () => void;
  readonly onRule: (rule: string | null) => void;
  /** When it reaches taken: the sheet asks for N under `anchor`. */
  readonly onReach: (anchor: HTMLElement) => void;
  readonly onLeave: () => void;
}

/** The phrases it reads, as the type box offers them. */
const TRY = `Try “${EXAMPLES.join("”, “")}”`;

/** What the type box's Note says while it can be taken: the rule and its next reset, or the phrases. */
function reading(typed: string): string {
  const read = readRule(typed);
  return read ? `Resets ${read.rule}. Next: ${read.next}` : TRY;
}

/** Why the type box can't be taken, or null. */
function checkRule(typed: string): string | null {
  if (readRule(typed)) return null;
  return typed.trim() === "" ? TRY : `Can't read that. ${TRY}`;
}

/**
 * Auto reset, in the menu's place: Back, the type box (a Prompt inside the menu's popover), then
 * Never and When it reaches N, the counter's rule checked.
 */
export function autoMenu(counter: Counter, options: AutoOptions = {}): HTMLElement {
  const typed = options.typed ?? (counter.rule ?? "");
  const hooks = options.hooks;
  const back = menu([{ label: "Back", icon: "arrow_back" }], {
    ...(options.current === 0 ? { current: 0 } : {}),
    ...(hooks ? { onTake: hooks.onBack, onLeave: hooks.onLeave } : {}),
  });
  const box = prompt({
    label: "When it resets",
    value: typed,
    place: "menu",
    check: checkRule,
    note: reading,
    ...(options.fieldLit ? { pinned: "lit" as const } : {}),
    ...(options.invalid ? { invalid: true } : {}),
    ...(hooks ? { onSave: (value: string) => hooks.onRule(readRule(value)!.rule), onCancel: hooks.onBack } : {}),
  });
  // N at When it reaches' end, before the check's room; taking the item asks for it.
  const cap = text(String(counter.cap ?? 10), "mono_large");
  cap.style.color = "var(--ink)";
  cap.classList.add("ct-cap");
  const choices = menu(
    [
      { label: "Never", checked: !counter.rule && counter.cap === undefined },
      { label: "When it reaches", checked: counter.cap !== undefined },
    ],
    {
      ...(options.current !== undefined && options.current > 0 ? { current: options.current - 1 } : {}),
      ...(hooks
        ? {
            onTake: (index: number) => (index === 0 ? hooks.onRule(null) : hooks.onReach(cap)),
            onLeave: hooks.onLeave,
          }
        : {}),
    },
  );
  (choices.children[1] as HTMLElement).querySelector(".k-menu-words")?.after(cap);
  const column = stack(0, "start", filling(back), filling(menuSeparator()), filling(box.element), filling(menuSeparator()), filling(choices));
  const card = popover(2, column);
  card.classList.add("ct-menu", "ct-auto");
  return card;
}

/** The prompt for the number When it reaches stops at, `typed` in it, all of it selected. */
export function askStop(counter: Counter, options: AskOptions = {}): HTMLElement {
  return prompt({
    label: "When it reaches",
    value: options.typed ?? String(counter.cap ?? 10),
    check: checkStop,
    ...(options.pinned ? { pinned: options.pinned } : {}),
    ...(options.invalid ? { invalid: true } : {}),
    ...(options.onSave ? { onSave: options.onSave } : {}),
    ...(options.onCancel ? { onCancel: options.onCancel } : {}),
  }).element;
}

// ---------------------------------------------------------------- history --

export type Grouping = "change" | "day" | "week" | "month";
export type Sorting = "newest" | "oldest" | "most";

const GROUPS: readonly { readonly key: Grouping; readonly label: string }[] = [
  { key: "change", label: "Each change" },
  { key: "day", label: "Day" },
  { key: "week", label: "Week" },
  { key: "month", label: "Month" },
];

const KIND_WORDS: Record<ChangeKind, string> = { "+1": "+1", "−1": "−1", set: "Set", reset: "Reset", "auto reset": "Auto reset" };

/** A minute of the shots' calendar as the history's rows say it. */
function when(at: number): { readonly date: string; readonly time: string; readonly offset: number } {
  const offset = Math.floor(at / 1440);
  const minutes = at - offset * 1440;
  return { date: day(offset), time: `${Math.floor(minutes / 60)}:${String(minutes % 60).padStart(2, "0")}`, offset };
}

/** One history row: when, what, and the value after it (or how much a group added). */
interface Line {
  readonly title: string;
  readonly what: string;
  readonly value: string;
  readonly at: number;
  readonly amount: number;
}

/** `log` as the history lists it, grouped and sorted. */
export function historyLines(log: readonly Change[], group: Grouping, sort: Sorting): Line[] {
  let lines: Line[];
  if (group === "change") {
    lines = log.map((change) => {
      const said = when(change.at);
      return { title: `${said.offset === 0 ? "Today" : said.date} · ${said.time}`, what: KIND_WORDS[change.kind], value: String(change.value), at: change.at, amount: change.value };
    });
  } else {
    const keyOf = (at: number): string => {
      const offset = Math.floor(at / 1440);
      const date = new Date(2026, 10, 12 + offset);
      if (group === "day") return offset === 0 ? "Today" : day(offset);
      if (group === "week") {
        const monday = new Date(date);
        monday.setDate(date.getDate() - ((date.getDay() + 6) % 7));
        return `Week of ${monday.getDate()} ${MONTHS[monday.getMonth()]!.slice(0, 3)}`;
      }
      return `${MONTHS[date.getMonth()]} ${date.getFullYear()}`;
    };
    const groups = new Map<string, { at: number; added: number; changes: number }>();
    for (const change of log) {
      const key = keyOf(change.at);
      const sum = groups.get(key) ?? { at: change.at, added: 0, changes: 0 };
      sum.changes += 1;
      if (change.kind === "+1") sum.added += 1;
      if (change.kind === "−1") sum.added -= 1;
      groups.set(key, sum);
    }
    lines = [...groups.entries()].map(([title, sum]) => ({ title, what: `${sum.changes} ${sum.changes === 1 ? "change" : "changes"}`, value: `+${Math.max(0, sum.added)}`, at: sum.at, amount: sum.added }));
  }
  return lines.sort(sort === "newest" ? (a, b) => b.at - a.at : sort === "oldest" ? (a, b) => a.at - b.at : (a, b) => b.amount - a.amount);
}

export interface HistoryOptions {
  readonly group?: Grouping;
  readonly sort?: Sorting;
  readonly down?: number;
  /** Back pinned in a state. */
  readonly back?: Pinned;
  readonly onBack?: () => void;
  readonly live?: boolean;
}

/** The history's rows past which its list scrolls. */
export const HISTORY_ROWS = 8;

/** A counter's history in the sheet: Back and its name, Group and Sort, then every change. */
export function historyPage(counter: Counter, options: HistoryOptions = {}): HTMLElement {
  let group = options.group ?? "change";
  let sort = options.sort ?? "newest";
  const log = counter.log ?? [];
  const backButton = iconButton({ icon: "arrow_back", label: "Back", kind: "Text", size: "Sm", ...(options.back ? { pinned: options.back } : {}) });
  if (options.onBack) backButton.addEventListener("click", options.onBack);
  const head = row("s8", "centre", backButton, filling(clamped(counter.name, "body_strong")), text(`${log.length} ${log.length === 1 ? "change" : "changes"}`, "small", "on_surface_variant"));
  head.classList.add("ct-head");
  const page = document.createElement("span");
  page.className = "ct-history";
  const body = document.createElement("span");
  body.className = "ct-history-body";

  const sortTabs = () =>
    tabs({
      choices: [{ label: "Newest" }, { label: "Oldest" }, { label: group === "change" ? "Highest" : "Most added" }],
      picked: ["newest", "oldest", "most"].indexOf(sort),
      onPick: (index) => {
        sort = (["newest", "oldest", "most"] as const)[index]!;
        draw();
      },
    });
  let sortWell = sortTabs();
  const groupWell = tabs({
    choices: GROUPS.map((each) => ({ label: each.label })),
    picked: GROUPS.findIndex((each) => each.key === group),
    onPick: (index) => {
      group = GROUPS[index]!.key;
      const fresh = sortTabs();
      sortWell.replaceWith(fresh);
      sortWell = fresh;
      draw();
    },
  });

  function draw(): void {
    if (log.length === 0) {
      body.replaceChildren(emptyState({ icon: "history", title: "No changes yet", body: "Every +1, −1, set and reset shows here, with its time." }));
      return;
    }
    const lines = historyLines(log, group, sort);
    const widest = "0".repeat(Math.max(...lines.map((each) => each.value.replace(/\D/g, "").length)) + (group === "change" ? 0 : 1));
    const rows = lines.map((line) => {
      const made = listRow({ title: line.title });
      made.classList.add("ct-line");
      const what = clamped(line.what, "body", "on_surface_variant");
      what.classList.add("ct-what");
      (made.firstElementChild as HTMLElement).append(what, rollingStill(line.value, widest));
      return made;
    });
    const column = stack("s4", "start", ...rows);
    column.classList.add("k-list-rows");
    const height = `calc(${HISTORY_ROWS} * var(--size-row) + ${HISTORY_ROWS - 1} * var(--space-s4) + 2 * var(--space-s12))`;
    const made = rows.length > HISTORY_ROWS ? scrolled("ScrollingList", column, { height }) : { box: well("List", column), pane: null, thumb: () => {} };
    listRows(made.box, made.pane, () => rows, {});
    if (options.down !== undefined && made.pane) {
      const down = options.down;
      const pane = made.pane;
      requestAnimationFrame(() => {
        pane.scrollTop = Math.round((pane.scrollHeight - pane.clientHeight) * down);
        made.thumb();
      });
    }
    body.replaceChildren(made.box);
  }
  draw();
  const controls = stack("s8", "start", filling(groupWell), filling(sortWell));
  page.append(filling(head), filling(controls), filling(body));
  return page;
}

/** A value or a group's sum still, in a box as wide as `widest`: "+12" in "+00". */
function rollingStill(words: string, widest: string): HTMLElement {
  const sign = words.startsWith("+") ? "+" : "";
  const shown = rollingNumber({ value: Number(words.replace(/\D/g, "")), widest: widest.slice(sign ? 1 : 0) || "0" });
  if (!sign) return shown.element;
  return row(0, "centre", text("+", "mono_large"), shown.element);
}

// ---------------------------------------------------------------- the sheet --

/** The sheet's content box: `size.panel` wide, `space.s8` inside, as the peek's. */
export function sheetBox(...content: Element[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "ct-sheet";
  box.append(...content);
  return box;
}

/** The screen's corner the sheet is drawn in, and its height. */
export const CORNER = "calc(var(--size-panel) + var(--space-s64) * 3)";
export const TALL = "calc(var(--size-app_min_height) + var(--space-s64))";

export interface ScreenOptions {
  readonly content?: HTMLElement | null;
  readonly pill?: Shown;
  /** No counters: no pill. */
  readonly none?: boolean;
  readonly height?: string;
  readonly reveal?: number;
  /** A popover over the sheet, at its place in the screen. */
  readonly over?: { readonly card: HTMLElement; readonly x: string; readonly y: string };
  /** A prompt floating under the text it changes, found by `anchor`, a selector in the screen. */
  readonly prompt?: { readonly card: HTMLElement; readonly anchor: string; readonly held?: PromptHeld };
}

/** The selector of row `index`'s name, or its value, for a still's prompt. */
export function anchorOf(index: number, what: Editing): string {
  return `.ct-row[data-index="${index}"] ${what === "name" ? ".ct-name > .k-list-title" : ".ct-value"}`;
}

/** The bar with the counters' pill, and the sheet open under the corner holding `content`. */
export function countersOnScreen(options: ScreenOptions = {}): HTMLElement {
  const bar = [...(options.none ? [] : [countersPill(options.pill ?? "rest")]), ...barButtons(false)];
  const room = screen({ width: CORNER, height: options.height ?? TALL, wallpaper, bar });
  if (options.content !== null) {
    room.element.dataset.open = "";
    hang(room, sheet({ content: options.content ?? sheetBox(countersList(COUNTERS).element), edge: "top", corner: true, ...(options.reveal !== undefined ? { reveal: options.reveal } : {}) }));
  }
  if (options.over) {
    const card = options.over.card;
    card.style.setProperty("--ct-x", options.over.x);
    card.style.setProperty("--ct-y", options.over.y);
    card.classList.add("ct-placed");
    room.over.append(card);
  }
  if (options.prompt) {
    const { card, anchor, held } = options.prompt;
    const found = room.element.querySelector<HTMLElement>(anchor);
    if (found) floatUnder(card, found, room.over, { held: held ?? "start" });
  }
  return room.element;
}

// ---------------------------------------------------------------- live --

/** The whole surface, live: the pill opens the sheet; rows count; right-click for the menu. */
export function liveCounters(start: readonly Counter[] = COUNTERS): HTMLElement {
  let counters = start.map((each) => ({ ...each, log: [...(each.log ?? [])] }));
  const pillElement = countersPill();
  const room = screen({ width: CORNER, height: TALL, wallpaper, bar: [pillElement, ...barButtons(false)] });
  const box = sheetBox();
  const made = sheet({ content: box, edge: "top", corner: true });
  hang(room, made);
  for (const part of made.parts) part.dataset.live = "";
  let card: HTMLElement | null = null;
  let asking: HTMLElement | null = null;
  let menuRow: HTMLElement | null = null;
  let rows: readonly CounterRow[] = [];

  const open = () => made.parts.some((part) => part.hasAttribute("data-shown"));
  function show(): void {
    room.element.dataset.open = "";
    for (const part of made.parts) part.dataset.shown = "";
    requestAnimationFrame(() => made.parts.forEach((part) => (part.dataset.settled = "")));
  }
  function hide(): void {
    closeAsk();
    closeMenu();
    delete room.element.dataset.open;
    for (const part of made.parts) {
      delete part.dataset.shown;
      delete part.dataset.settled;
    }
  }
  function closeAsk(): void {
    asking?.remove();
    asking = null;
  }
  /** Floats `made` under `anchor`, the keyboard in its Field; the menu, if open, stays under it. */
  function ask(made: HTMLElement, anchor: HTMLElement, held: PromptHeld): void {
    closeAsk();
    asking = made;
    floatUnder(made, anchor, room.over, { held });
    requestAnimationFrame(() => made.querySelector<HTMLElement>(".k-field")?.focus({ preventScroll: true }));
  }
  /** Asks for row `index`'s name or value under `anchor`. */
  function edit(index: number, what: Editing, anchor: HTMLElement, target: CounterRow): void {
    closeMenu();
    const counter = { ...counters[index]!, name: target.name(), value: target.value() };
    const done = () => {
      closeAsk();
      target.element.closest<HTMLElement>(".k-list, [role=listbox]")?.focus({ preventScroll: true });
    };
    ask(
      askFor(what, counter, {
        onCancel: done,
        onSave: (typed) => {
          done();
          if (what === "name") target.setName(typed.trim());
          else target.set(Math.min(Number(typed), counter.cap ?? Number.POSITIVE_INFINITY), "set");
        },
      }),
      anchor,
      what === "value" ? "end" : "start",
    );
  }
  function closeMenu(): void {
    card?.remove();
    card = null;
    menuRow?.removeAttribute("data-menu");
    menuRow = null;
  }
  function log(index: number, kind: ChangeKind, value: number): void {
    const counter = counters[index]!;
    counters[index] = { ...counter, value, log: [{ at: NOW, kind, value }, ...(counter.log ?? [])] };
  }
  function list(): void {
    closeMenu();
    pillElement.hidden = counters.length === 0;
    if (counters.length === 0) {
      hide();
      return;
    }
    const drawn = countersList(counters, {
      live: {
        onChange: (index, value, kind) => log(index, kind, value),
        onRename: (index, name) => (counters[index] = { ...counters[index]!, name }),
        onMenu: (index, at, made) => place(index, at, made),
        onEdit: (index, what, anchor, row) => edit(index, what, anchor, row),
      },
    });
    rows = drawn.rows;
    box.replaceChildren(drawn.element);
  }
  /** The menu's card at `at` in the page, inside the screen. */
  function place(index: number, at: { x: number; y: number }, drawnRows: readonly CounterRow[]): void {
    closeMenu();
    rows = drawnRows;
    menuRow = rows[index]!.element;
    menuRow.dataset.menu = "";
    const counter = counters[index]!;
    const screenBox = room.element.getBoundingClientRect();
    const scale = screenBox.width / room.element.offsetWidth || 1;
    const x = (at.x - screenBox.left) / scale;
    const y = (at.y - screenBox.top) / scale;
    const mainMenu = (): HTMLElement =>
      counterMenu(counter, {
        hooks: {
          onLeave: closeMenu,
          onTake: (what) => {
            const target = rows[index]!;
            if (what === "rename") {
              closeMenu();
              target.rename();
            } else if (what === "reset") {
              closeMenu();
              target.reset();
            } else if (what === "auto") swap(autoView());
            else if (what === "history") {
              closeMenu();
              history(index);
            } else if (what === "remove") {
              counters = counters.filter((_, at_) => at_ !== index);
              list();
            }
          },
        },
      });
    const autoView = (): HTMLElement =>
      autoMenu(counters[index]!, {
        hooks: {
          onBack: () => swap(mainMenu()),
          onLeave: closeMenu,
          onRule: (rule) => {
            const { cap: _cap, ...rest } = counters[index]!;
            counters[index] = rule ? { ...rest, rule } : rest;
            list();
          },
          onReach: (anchor) =>
            ask(
              askStop(counters[index]!, {
                onCancel: closeAsk,
                onSave: (typed) => {
                  closeAsk();
                  const cap = Number(typed);
                  const { rule: _rule, ...rest } = counters[index]!;
                  counters[index] = { ...rest, cap, value: Math.min(rest.value, cap) };
                  list();
                },
              }),
              anchor,
              "end",
            ),
        },
      });
    const swap = (fresh: HTMLElement) => {
      fresh.style.setProperty("--ct-x", `${x}px`);
      fresh.style.setProperty("--ct-y", `${y}px`);
      fresh.classList.add("ct-placed");
      card?.replaceWith(fresh);
      if (!card) room.over.append(fresh);
      card = fresh;
      requestAnimationFrame(() => {
        // Kept inside the screen, `space.s8` from its edges.
        const inside = room.element.offsetWidth - fresh.offsetWidth - fresh.offsetLeft;
        if (inside < 0) fresh.style.setProperty("--ct-x", `${x + inside}px`);
        (fresh.querySelector<HTMLElement>(".k-field") ?? fresh.querySelector<HTMLElement>(".k-menu"))?.focus({ preventScroll: true });
      });
    };
    swap(mainMenu());
  }
  function history(index: number): void {
    box.replaceChildren(historyPage(counters[index]!, { onBack: list, live: true }));
    box.querySelector<HTMLElement>(".k-button")?.focus({ preventScroll: true, focusVisible: false } as FocusOptions);
  }
  pillElement.addEventListener("click", (event) => {
    event.stopPropagation();
    if (open()) hide();
    else {
      list();
      show();
    }
  });
  room.element.addEventListener("pointerdown", (event) => {
    const target = event.target as Node;
    // A press outside a prompt cancels it; outside the menu too, closes the menu.
    if (asking && !asking.contains(target)) {
      closeAsk();
      if (card?.contains(target)) return;
    }
    if (card && !card.contains(target)) closeMenu();
    if (!open() || made.box.contains(target) || pillElement.contains(target) || card?.contains(target)) return;
    hide();
  });
  room.element.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    if (card) closeMenu();
    else if (box.querySelector(".ct-history")) list();
    else hide();
  });
  list();
  show();
  return room.element;
}

/** A row alone, at the sheet's width, for the parts' pages. */
export function rowAlone(counter: Counter, options: Partial<RowOptions> = {}): HTMLElement {
  const made = counterRow(counter, { widest: widestOf([counter]), ...options });
  const column = stack("s4", "start", made.element);
  column.classList.add("k-list-rows");
  return sheetBox(flush(well("List", column)));
}
