// A month as six weeks, each led by its ISO week number: `kubaos-ui`'s `calendar::calendar`,
// `KubaOS.Components`' `Calendar` (`docs/design/components/calendar.md`). The clock popup's.
//
// Between two arrows (Text icon buttons, sm) the title, `subheading` at the control radius, opens
// the year: the twelve months three to a row. Under them the weekdays' names (`label`, weekends in
// `tertiary`) and six weeks: a week number (`caption`) in a column a small control wide, then seven
// days, each a regular control (`size.control_md`) with a small one's disc (`size.control_sm`), a
// pill. Today is filled with `primary`; the picked day is ringed with it in the thin stroke; a
// holiday has a `tertiary` dot under its number and its name in a hint; the days of the months
// around are dimmed to `opacity.disabled_content`. The first weekday is the owner's.
//
// A day is a control in the state model (`.k-control` in `lib.css`): lit, its tube burns inside its
// disc's edge, today's core inside its fill. The month grid is one stop for Tab: the arrows move a
// day or a week, Page Up and Page Down a month, Home and End to the week's ends, and the pointer
// moving onto a day puts the keyboard there, so one day is lit at a time. A new month slides in from
// the side it lies on over `short` on `emphasized_decel`.

import { type Pinned, pin, sideways } from "./control.ts";
import { focusRing } from "./focus.ts";
import { withHint } from "./hint.ts";
import { iconButton } from "./icon-button.ts";
import { filling, row } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { raw } from "./tokens.ts";
import { text } from "./text.ts";
import { settings } from "../site/settings.ts";

/** A date, its month 1 to 12. */
export interface Day {
  readonly year: number;
  readonly month: number;
  readonly day: number;
}

export interface CalendarOptions {
  /** The month shown, by its year and month; a date's day is ignored. */
  readonly shown: { readonly year: number; readonly month: number };
  readonly today: Day;
  readonly picked?: Day;
  /** The first weekday, 0 Sunday to 6 Saturday: the owner's, from the clock's settings. */
  readonly firstWeekday?: number;
  /** The locale the names are in. */
  readonly locale?: string;
  /** Holidays' names by their date, `2026-11-01`. */
  readonly holidays?: Readonly<Record<string, string>>;
  /** Open on the year's twelve months. */
  readonly year?: boolean;
  /** The day the keyboard is on, pinned lit (or `pinned`'s state) whatever the pointer does. */
  readonly current?: Day;
  readonly pinned?: Pinned;
  /** The side the shown month came from, for a move's sample: 1 the next, -1 the last. */
  readonly from?: 1 | -1;
}

const DAY_MS = 86_400_000;

function utc(day: Day): number {
  return Date.UTC(day.year, day.month - 1, day.day);
}

function dayOf(ms: number): Day {
  const date = new Date(ms);
  return { year: date.getUTCFullYear(), month: date.getUTCMonth() + 1, day: date.getUTCDate() };
}

function add(day: Day, days: number): Day {
  return dayOf(utc(day) + days * DAY_MS);
}

function same(a: Day | undefined, b: Day | undefined): boolean {
  return !!a && !!b && a.year === b.year && a.month === b.month && a.day === b.day;
}

function weekday(day: Day): number {
  return new Date(utc(day)).getUTCDay();
}

/** `2026-11-01`: a holiday's key. */
export function key(day: Day): string {
  return `${day.year}-${String(day.month).padStart(2, "0")}-${String(day.day).padStart(2, "0")}`;
}

/** The ISO week number of `day`. */
function isoWeek(day: Day): number {
  const thursday = add(day, 3 - ((weekday(day) + 6) % 7));
  const first = { year: thursday.year, month: 1, day: 1 };
  return 1 + Math.floor((utc(thursday) - utc(first)) / (7 * DAY_MS));
}

/** The month `by` months from `month`, its day held to the new month's last. */
function shiftMonth(day: Day, by: number): Day {
  const index = day.year * 12 + day.month - 1 + by;
  const year = Math.floor(index / 12);
  const month = (index % 12) + 1;
  const last = new Date(Date.UTC(year, month, 0)).getUTCDate();
  return { year, month, day: Math.min(day.day, last) };
}

/** Six weeks from the first weekday on or before the month's first day. */
function weeks(year: number, month: number, first: number): Day[][] {
  const start = { year, month, day: 1 };
  const begin = add(start, -((weekday(start) - first + 7) % 7));
  return Array.from({ length: 6 }, (_, w) => Array.from({ length: 7 }, (_, d) => add(begin, w * 7 + d)));
}

/** The duration token `name` in milliseconds under the speed setting. */
function duration(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** `words` in `style`, centred in its box on whole device pixels, the odd one after it. */
function centred(words: string, style: Parameters<typeof text>[1], color: string): HTMLElement {
  const shown = text(words, style, color);
  shown.classList.add("k-cal-centred");
  return shown;
}

/** What makes a day look as it does. */
export interface DayLook {
  readonly day: Day;
  readonly today?: boolean;
  readonly picked?: boolean;
  readonly weekend?: boolean;
  /** Its holiday's name. */
  readonly holiday?: string;
  /** In the month before or after the one shown. */
  readonly other?: boolean;
}

/**
 * One day's cell: a regular control with a small control's disc; a holiday's name in a hint unless
 * `hinted` is false, for a sample whose hint would float over the row below.
 */
export function dayCell(look: DayLook, pinned?: Pinned, hinted = true): HTMLElement {
  const cell = document.createElement("span");
  cell.className = "k-cal-day k-control";
  cell.dataset.hot = "";
  cell.dataset.date = key(look.day);
  cell.setAttribute("role", "gridcell");
  for (const [name, on] of [
    ["today", look.today],
    ["picked", look.picked],
    ["weekend", look.weekend],
    ["holiday", look.holiday],
    ["other", look.other],
  ] as const) {
    cell.toggleAttribute(`data-${name}`, !!on);
  }
  cell.setAttribute("aria-selected", String(!!look.picked));
  pin(cell, pinned);
  const disc = document.createElement("span");
  disc.className = "k-cal-disc";
  const number = centred(String(look.day.day), look.today ? "body_strong" : "body", "on_surface");
  number.style.color = "var(--num)";
  number.classList.add("k-cal-number");
  disc.append(number);
  if (look.holiday) {
    const dot = document.createElement("span");
    dot.className = "k-cal-dot";
    disc.append(dot);
  }
  focusRing(disc, "full", "accent", !!look.today).dataset.pressable = "";
  cell.append(disc);
  if (look.holiday && hinted) withHint(cell, look.holiday);
  return cell;
}

/** A month of the year view: `size.control_md` tall at the control radius, the one shown filled. */
export function monthCell(name: string, current: boolean, pinned?: Pinned): HTMLElement {
  const cell = document.createElement("span");
  cell.className = "k-cal-month k-control";
  cell.dataset.hot = "";
  cell.toggleAttribute("data-shown", current);
  pin(cell, pinned);
  const words = centred(name, "body_strong", "on_surface");
  words.style.color = "var(--num)";
  cell.append(words);
  focusRing(cell, "control", "accent", current).dataset.pressable = "";
  return cell;
}

/** The title: `subheading` at the control radius, opening the year, Tonal while it is open. */
export function titlePill(words: string, open: boolean, pinned?: Pinned): HTMLElement {
  const title = document.createElement("button");
  title.type = "button";
  title.className = "k-cal-title k-control";
  title.dataset.hot = "";
  title.toggleAttribute("data-open", open);
  title.setAttribute("aria-expanded", String(open));
  pin(title, pinned);
  const words_ = text(words, "subheading");
  words_.style.color = "var(--words)";
  title.append(words_);
  focusRing(title, "control", "accent", false).dataset.pressable = "";
  return title;
}

export function calendar(options: CalendarOptions): HTMLElement {
  const first = options.firstWeekday ?? 1;
  const locale = options.locale ?? "en-GB";
  const holidays = options.holidays ?? {};
  let shown = { ...options.shown };
  let picked = options.picked;
  let current: Day = options.current ?? picked ?? options.today;
  let yearView = options.year ?? false;

  const root = document.createElement("span");
  root.className = "k-calendar";

  const monthTitle = new Intl.DateTimeFormat(locale, { month: "long", year: "numeric", timeZone: "UTC" });
  const shortMonth = new Intl.DateTimeFormat(locale, { month: "short", timeZone: "UTC" });
  const dayName = new Intl.DateTimeFormat(locale, { weekday: "short", timeZone: "UTC" });

  const head = document.createElement("span");
  head.className = "k-cal-head";
  const prev = iconButton({ icon: "chevron_left", label: "Last month", kind: "Text", size: "Sm" });
  const next = iconButton({ icon: "chevron_right", label: "Next month", kind: "Text", size: "Sm" });
  let title = titlePill("", false);
  const spacer = filling(document.createElement("span"));
  const arrows = row(0, "centre", prev, spacer, next);
  head.append(arrows);

  const body = document.createElement("span");
  body.className = "k-cal-body";
  const grid = document.createElement("span");
  grid.className = "k-cal-grid";
  grid.setAttribute("role", "grid");
  const names = document.createElement("span");
  names.className = "k-cal-names";
  const rows = document.createElement("span");
  rows.className = "k-cal-weeks";
  if (options.from) {
    rows.style.setProperty("--from", String(options.from));
    body.dataset.sliding = "";
  }
  grid.append(names, rows);
  const year = document.createElement("span");
  year.className = "k-cal-year";
  year.setAttribute("role", "grid");
  body.append(grid, year);
  root.append(head, body);

  /** The weekdays' names from the first weekday: two letters, as the build has them. */
  function drawNames(): void {
    const week = centred("Wk", "caption", "on_surface_variant");
    const cells = [week];
    for (let n = 0; n < 7; n++) {
      const day = (first + n) % 7;
      // 2023-01-01 was a Sunday.
      const name = [...dayName.format(Date.UTC(2023, 0, 1 + day))].slice(0, 2).join("");
      cells.push(centred(name, "label", day === 0 || day === 6 ? "tertiary" : "on_surface_variant"));
    }
    names.replaceChildren(...cells.map((each) => box(each, "k-cal-name")));
  }

  function box(content: HTMLElement, className: string): HTMLElement {
    const holder = document.createElement("span");
    holder.className = className;
    holder.append(content);
    return holder;
  }

  function drawWeeks(): void {
    const made = weeks(shown.year, shown.month, first).map((days) => {
      const thursday = days.find((day) => weekday(day) === 4)!;
      const number = box(centred(String(isoWeek(thursday)), "caption", "on_surface_variant"), "k-cal-week-number");
      const cells = days.map((day) => {
        const look: DayLook = {
          day,
          today: same(day, options.today),
          picked: same(day, picked) && !same(day, options.today),
          weekend: weekday(day) === 0 || weekday(day) === 6,
          ...(holidays[key(day)] ? { holiday: holidays[key(day)]! } : {}),
          other: day.month !== shown.month,
        };
        const cell = dayCell(look, same(day, options.current) ? (options.pinned ?? "lit") : undefined);
        cell.toggleAttribute("data-current", same(day, current));
        cell.addEventListener("pointermove", () => {
          if (!same(current, day)) moveTo(day, false);
        });
        cell.addEventListener("click", () => pick(day));
        return cell;
      });
      const week = document.createElement("span");
      week.className = "k-cal-week";
      week.setAttribute("role", "row");
      week.append(number, ...cells);
      return week;
    });
    rows.replaceChildren(...made);
  }

  function drawYear(): void {
    const months = Array.from({ length: 12 }, (_, index) => {
      const cell = monthCell(shortMonth.format(Date.UTC(shown.year, index, 1)), index + 1 === shown.month);
      cell.toggleAttribute("data-current", index + 1 === current.month);
      cell.addEventListener("pointermove", () => {
        if (current.month !== index + 1) {
          current = shiftMonth(current, index + 1 - current.month);
          markYear();
        }
      });
      cell.addEventListener("click", () => leaveYear(index + 1));
      return cell;
    });
    year.replaceChildren(...months);
  }

  function markYear(): void {
    [...year.children].forEach((cell, index) => cell.toggleAttribute("data-current", index + 1 === current.month));
  }

  function drawTitle(): void {
    const words = yearView ? String(shown.year) : monthTitle.format(Date.UTC(shown.year, shown.month - 1, 1));
    const made = titlePill(words, yearView, title.dataset.pinned as Pinned | undefined);
    made.addEventListener("click", () => (yearView ? leaveYear(current.month) : openYear()));
    title.replaceWith(made);
    title = made;
    if (!title.isConnected) arrows.insertBefore(title, spacer);
    prev.setAttribute("aria-label", yearView ? "Last year" : "Last month");
    next.setAttribute("aria-label", yearView ? "Next year" : "Next month");
    requestAnimationFrame(placeTitle);
  }

  /** The title in the middle of the header on a whole device pixel, the odd one after it. */
  function placeTitle(): void {
    title.style.left = `${Math.floor((head.clientWidth - title.offsetWidth) / 2)}px`;
  }

  function show(): void {
    root.toggleAttribute("data-year", yearView);
    grid.tabIndex = yearView ? -1 : 0;
    year.tabIndex = yearView ? 0 : -1;
    grid.inert = yearView;
    year.inert = !yearView;
    drawTitle();
    probe(root, { role: "component", name: "Calendar", variant: variant(yearView && "year"), fixed: { width: true, height: true } });
  }

  /** Slides the new month in from the side it lies on, `by` months from the last. */
  function slide(by: number): void {
    if (!root.isConnected || by === 0) return;
    const reduced = settings().reduced;
    const easing = raw("easing-emphasized_decel");
    rows.style.setProperty("--from", String(Math.sign(by)));
    body.dataset.sliding = "";
    const frames = reduced ? [{ opacity: 0 }, { opacity: 1 }] : [{ opacity: 0, "--slide": "1" }, { opacity: 1, "--slide": "0" }];
    const played = rows.animate(frames, { duration: duration("short"), easing });
    played.onfinish = played.oncancel = () => delete body.dataset.sliding;
  }

  function goTo(month: { year: number; month: number }): void {
    const by = month.year * 12 + month.month - (shown.year * 12 + shown.month);
    shown = { ...month };
    drawWeeks();
    drawTitle();
    slide(by);
  }

  /** Puts the keyboard on `day`, showing its month when it lies in another. */
  function moveTo(day: Day, follow = true): void {
    current = day;
    if (follow && (day.month !== shown.month || day.year !== shown.year)) goTo(day);
    for (const cell of rows.querySelectorAll<HTMLElement>(".k-cal-day")) cell.toggleAttribute("data-current", cell.dataset.date === key(day));
  }

  function pick(day: Day): void {
    picked = day;
    current = day;
    if (day.month !== shown.month || day.year !== shown.year) goTo(day);
    for (const cell of rows.querySelectorAll<HTMLElement>(".k-cal-day")) {
      const on = cell.dataset.date === key(day) && !cell.hasAttribute("data-today");
      cell.toggleAttribute("data-picked", on);
      cell.setAttribute("aria-selected", String(on));
      cell.toggleAttribute("data-current", cell.dataset.date === key(day));
    }
  }

  function openYear(): void {
    yearView = true;
    drawYear();
    show();
    year.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
  }

  function leaveYear(month: number): void {
    yearView = false;
    current = shiftMonth({ ...current, year: shown.year }, month - current.month);
    goTo({ year: shown.year, month });
    show();
    grid.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
  }

  prev.addEventListener("click", () => (yearView ? yearBy(-1) : goTo(shiftMonth({ ...shown, day: 1 }, -1))));
  next.addEventListener("click", () => (yearView ? yearBy(1) : goTo(shiftMonth({ ...shown, day: 1 }, 1))));

  function yearBy(by: number): void {
    shown = { year: shown.year + by, month: shown.month };
    drawYear();
    drawTitle();
  }

  grid.addEventListener("keydown", (event) => {
    const k = event.key;
    const days =
      k === "ArrowLeft" || k === "h" ? -1 : k === "ArrowRight" || k === "l" ? 1 : k === "ArrowUp" || k === "k" ? -7 : k === "ArrowDown" || k === "j" ? 7 : 0;
    if (days !== 0) moveTo(add(current, days));
    else if (k === "PageUp" || k === "PageDown") moveTo(shiftMonth(current, k === "PageUp" ? -1 : 1));
    else if (k === "Home" || k === "End") {
      const start = add(current, -((weekday(current) - first + 7) % 7));
      moveTo(k === "Home" ? start : add(start, 6));
    } else if (k === "Enter" || k === " ") {
      if (!event.repeat) pick(current);
    } else return;
    event.preventDefault();
  });

  year.addEventListener("keydown", (event) => {
    const k = event.key;
    const by = k === "ArrowLeft" ? -1 : k === "ArrowRight" ? 1 : k === "ArrowUp" ? -3 : k === "ArrowDown" ? 3 : 0;
    if (by !== 0) {
      const month = Math.max(1, Math.min(12, current.month + by));
      current = shiftMonth(current, month - current.month);
      markYear();
    } else if (k === "PageUp" || k === "PageDown") yearBy(k === "PageUp" ? -1 : 1);
    else if (k === "Enter" || k === " ") {
      if (!event.repeat) leaveYear(current.month);
    } else if (k === "Escape") leaveYear(shown.month);
    else return;
    event.preventDefault();
    event.stopPropagation();
  });

  // Only a sideways turn of the wheel leafs the months: a turn up or down scrolls the page past
  // the calendar, as everywhere else.
  sideways(body, (notches) => {
    if (yearView) yearBy(notches);
    else goTo(shiftMonth({ ...shown, day: 1 }, notches));
  });

  drawNames();
  drawWeeks();
  if (yearView) drawYear();
  show();
  new ResizeObserver(placeTitle).observe(head);
  return root;
}
