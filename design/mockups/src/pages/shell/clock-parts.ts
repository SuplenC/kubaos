// The clock popup as `config/quickshell/modules/clock/` draws it, from the library's parts, with the
// shots' stand-ins (`test/shots/fakes`): Tuesday 27 October 2026, 21:47:32, Tokyo as an extra clock,
// Vacation pinned to the bar and Release, a timer running. The face, the extra clocks, the search
// and its answer, the month and the picked day, countdowns, the tools, and the settings page. The
// clock's pages build their samples from these. This file is no page.

import "./clock.css";
import { card } from "../../lib/card.ts";
import { calendar } from "../../lib/calendar.ts";
import { chip } from "../../lib/chip.ts";
import { field } from "../../lib/field.ts";
import { figure } from "../../lib/figure.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { icon } from "../../lib/icon.ts";
import { keyHints } from "../../lib/keycap.ts";
import { container, fill, filling, flow, row, stack } from "../../lib/layout.ts";
import { progressBar } from "../../lib/progress.ts";
import { select } from "../../lib/select.ts";
import { hang, sheet } from "../../lib/sheet.ts";
import { tabs } from "../../lib/tabs.ts";
import { clamped, text } from "../../lib/text.ts";
import { barScreen, CLOCK, EVERYDAY } from "./bar-parts.ts";
import { TALL } from "./control-center-parts.ts";

export { TALL };

/** The shots' frozen moment. */
const NOW = { hhmm: "21:47", ss: "32", date: "Tuesday, 27 October 2026", day: "Day 300 · Week 44" } as const;

/** An extra clock: its label, its time there, how far off, and day or night there. */
export interface Zone {
  readonly label: string;
  readonly name: string;
  readonly time: string;
  readonly relative: string;
  readonly daylight: "light_mode" | "wb_twilight" | "dark_mode";
}

export const ZONES: readonly Zone[] = [
  { label: "Tokyo", name: "Asia/Tokyo", time: "05:47", relative: "+8 h", daylight: "wb_twilight" },
  { label: "New York", name: "America/New_York", time: "16:47", relative: "−5 h", daylight: "light_mode" },
  { label: "Los Angeles", name: "America/Los_Angeles", time: "13:47", relative: "−8 h", daylight: "light_mode" },
  { label: "Auckland", name: "Pacific/Auckland", time: "09:47", relative: "+12 h", daylight: "light_mode" },
];

export interface Countdown {
  readonly name: string;
  readonly left: string;
  readonly at: string;
  readonly passed?: boolean;
}

export const COUNTDOWNS: readonly Countdown[] = [
  { name: "Vacation", left: "52 d 2 h", at: "19 Dec" },
  { name: "Release", left: "6 d 2 h", at: "3 Nov" },
  { name: "Conference", left: "", at: "12 Oct", passed: true },
];

export type Tool = "timer" | "stopwatch" | "alarm";

export interface ClockOptions {
  readonly page?: "main" | "settings";
  readonly analog?: boolean;
  /** How many extra clocks, of `ZONES`. */
  readonly clocks?: number;
  /** What is typed in the search; a typed question shows its answer. */
  readonly query?: string;
  /** The answer copied, its line in place of the key hints. */
  readonly copied?: boolean;
  /** The search lit with the keyboard in it. */
  readonly searching?: boolean;
  readonly countdowns?: number;
  /** The pinned countdown, by name, or none. */
  readonly pinned?: string | null;
  readonly countdownError?: string;
  readonly tool?: Tool;
  /** The timer set and running, set and paused, or not set. */
  readonly timer?: "running" | "paused" | "none";
  readonly laps?: number;
  readonly alarms?: number;
  readonly toolError?: string;
  /** The calendar open on the year. */
  readonly year?: boolean;
  /** The picked day, in October 2026. */
  readonly picked?: number;
  /** kubaos-clock not running: the line that says so at the foot. */
  readonly unavailable?: boolean;
  /** The settings' holidays select drawn open. */
  readonly holidaysOpen?: boolean;
  /** A long day's answer and title, for the content's extremes. */
  readonly answer?: { readonly title: string; readonly detail: string };
  /** Answers the pointer: the gear, the tools' tabs, the search. */
  readonly live?: boolean;
}

/** The analog face: twelve ticks, the hour in `primary`, the minute in `on_surface`, the second in `tertiary`. */
function analogFace(): HTMLElement {
  const face = document.createElement("span");
  face.className = "ck-face";
  for (let index = 0; index < 12; index++) {
    const tick = document.createElement("span");
    tick.className = "ck-tick";
    tick.toggleAttribute("data-quarter", index % 3 === 0);
    tick.style.setProperty("--turn", `${index * 30}deg`);
    face.append(tick);
  }
  const hand = (name: string, turn: number) => {
    const each = document.createElement("span");
    each.className = `ck-hand ck-hand-${name}`;
    each.style.setProperty("--turn", `${turn}deg`);
    return each;
  };
  const pin = document.createElement("span");
  pin.className = "ck-pin";
  // 21:47:32.
  face.append(hand("hour", (9 + 47 / 60) * 30), hand("minute", (47 + 32 / 60) * 6), hand("second", 32 * 6), pin);
  return face;
}

/** The time to the second, the date with its kanji, the day and the week; the gear at the end. */
function head(options: ClockOptions, toggle: () => void): HTMLElement {
  // The seconds on the time's baseline: a box of its own, so the row's shared line leaves it be.
  const hero = text(NOW.hhmm, "hero_number");
  const seconds = figure(NOW.ss, "headline", "on_surface_variant");
  seconds.style.setProperty("--time-mass", hero.dataset.mass ?? "0");
  seconds.style.setProperty("--mass-seconds", (seconds.firstElementChild as HTMLElement | null)?.dataset.mass ?? "0");
  const time = document.createElement("span");
  time.className = "ck-time";
  time.append(hero, seconds);
  const date = row("s8", "centre", text(CLOCK.kanji, "accent", "tertiary"), text(NOW.date, "body_large"));
  const words = stack("s4", "start", time, date, text(NOW.day, "small", "on_surface_variant"));
  const left = row("s20", "centre", ...(options.analog ? [analogFace()] : []), words);
  const settings = options.page === "settings";
  const gear = iconButton({ icon: settings ? "close" : "settings", label: settings ? "Back" : "Calendar settings", kind: settings ? "Filled" : "Text" });
  gear.addEventListener("click", toggle);
  return row(0, "start", left, fill(), gear);
}

/** An extra clock: day or night there, its label over its time, how far off. */
function zone(each: Zone): HTMLElement {
  const words = stack(0, "start", text(each.label, "caption", "on_surface_variant"), figure(each.time, "body_strong"));
  const box = row("s8", "centre", icon(each.daylight, "var(--icon-s20)", `var(--color-${each.daylight === "light_mode" ? "primary" : "tertiary"})`), words, text(each.relative, "small", "on_surface_variant"));
  box.classList.add("ck-zone");
  return box;
}

/** What the search comes to, in a card under it, opening as a log does. */
function answer(options: ClockOptions): HTMLElement {
  const said = options.answer ?? { title: "53 days", detail: "From today to Saturday 19 December 2026" };
  const foot = options.copied ? text("Copied 53 days", "caption", "tertiary") : keyHints([["Enter", "copies"], ["Ctrl+Enter", "copies ISO"]]);
  const box = document.createElement("span");
  box.className = "ck-answer";
  const inner = card({ body: [stack("s4", "start", filling(clamped(said.title, "heading", "primary")), filling(clamped(said.detail, "body", "on_surface_variant", 2)), foot)] });
  box.append(inner);
  new ResizeObserver(() => box.style.setProperty("--natural", `${inner.offsetHeight}px`)).observe(inner);
  return box;
}

/** The picked day: its title, how far and its week, a holiday, the extra clocks' times. */
function dayDetails(options: ClockOptions): HTMLElement {
  const today = (options.picked ?? 27) === 27;
  const holiday = options.picked === 1;
  const lines = [
    text(today ? "Today, Tuesday 27 October" : holiday ? "Sunday 1 November" : `${options.picked} October`, "body_strong"),
    clamped(today ? "Week 44 · day 300 · 3 working days to Friday" : holiday ? "In 5 days · week 44 · day 305" : "Week 44", "small", "on_surface_variant"),
    ...(holiday ? [text("All Saints' Day", "small", "tertiary")] : []),
    ...((options.clocks ?? 1) > 0 ? [clamped(ZONES.slice(0, options.clocks ?? 1).map((each) => `${each.label} ${each.time}`).join("  ·  "), "small", "on_surface_variant")] : []),
  ];
  return stack("s4", "start", ...lines.map(filling));
}

/** A countdown: pinned or not, its name over what's left and when, Remove at its end. */
function countdown(each: Countdown, pinned: boolean): HTMLElement {
  const words = stack(0, "start", filling(clamped(each.name, "body_strong")), filling(clamped(each.passed ? `${each.at} · passed` : `in ${each.left} · ${each.at}`, "caption", "on_surface_variant")));
  const remove = iconButton({ icon: "close", label: "Remove", kind: "Text", size: "Sm" });
  return row("s8", "centre", icon(pinned ? "push_pin" : "hourglass_top", "var(--icon-s20)", "var(--color-tertiary)"), filling(words), remove);
}

function countdowns(options: ClockOptions): HTMLElement {
  const pinned = options.pinned === undefined ? "Vacation" : options.pinned;
  const shown = COUNTDOWNS.slice(0, options.countdowns ?? 2);
  return stack(
    "s8",
    "start",
    text("Countdowns", "body_strong"),
    ...shown.map((each) => filling(countdown(each, each.name === pinned))),
    filling(field({ icon: "more_time", placeholder: "Vacation: 19 dec" })),
    ...(options.countdownError ? [filling(clamped(options.countdownError, "small", "error", 2))] : []),
  );
}

/** A tool's page, all three one height: the alarms' field over three alarms. */
function toolPage(tool: Tool, options: ClockOptions): HTMLElement {
  const quiet = (name: string, label: string, kind: "Filled" | "Tonal") => iconButton({ icon: name, label, kind });
  let content: HTMLElement;
  if (tool === "timer") {
    const timer = options.timer ?? "running";
    content =
      timer === "none"
        ? stack(
            "s8",
            "start",
            filling(field({ icon: "timer", placeholder: "25m, 1h30m, 90 seconds" })),
            row(
              "s8",
              "centre",
              ...["5m", "10m", "25m", "1h"].map((label) => chip({ label, press: "press", tabStop: false })),
            ),
          )
        : stack(
            "s8",
            "centre",
            figure(timer === "running" ? "12:08" : "12:08", "headline", timer === "running" ? "on_surface" : "on_surface_variant"),
            filling(progressBar({ fraction: 0.52 })),
            row("s8", "centre", quiet(timer === "running" ? "pause" : "play_arrow", timer === "running" ? "Pause" : "Resume", "Filled"), quiet("close", "Cancel", "Tonal")),
          );
  } else if (tool === "stopwatch") {
    const laps = ["Lap 2  1:02.40", "Lap 1  0:58.12"].slice(0, options.laps ?? 2);
    content = stack("s8", "centre", figure("2:00.52", "headline"), row("s8", "centre", quiet("pause", "Stop", "Filled"), quiet("flag", "Lap", "Tonal"), quiet("restart_alt", "Reset", "Tonal")), ...laps.map((each) => figure(each, "small", "on_surface_variant")));
  } else {
    const alarms = ["7:30", "Friday 17:00", "6:00 PM"].slice(0, options.alarms ?? 1);
    const rows = alarms.map((label) => filling(row("s8", "centre", icon("alarm", "var(--icon-s20)", "var(--color-tertiary)"), text(label, "body"), fill(), iconButton({ icon: "close", label: "Remove", kind: "Text", size: "Sm" }))));
    content = stack("s8", "start", filling(field({ icon: "alarm_add", placeholder: "7:30, 6pm, friday 17:00" })), ...(rows.length ? rows : [text("No alarms set", "small", "on_surface_variant")]));
  }
  content.classList.add("ck-tool");
  return content;
}

function tools(options: ClockOptions): HTMLElement {
  const order: Tool[] = ["timer", "stopwatch", "alarm"];
  let current = options.tool ?? "timer";
  const room = document.createElement("span");
  room.className = "ck-tool-room";
  const draw = () => room.replaceChildren(toolPage(current, options));
  draw();
  const bar = tabs({
    choices: [{ label: "Timer" }, { label: "Stopwatch" }, { label: "Alarm" }],
    picked: order.indexOf(current),
    onPick: (index) => {
      current = order[index]!;
      draw();
    },
  });
  return stack("s12", "start", filling(bar), filling(room), ...(options.toolError ? [filling(clamped(options.toolError, "small", "error", 2))] : []));
}

/** The settings page: holidays, first weekday, time format, date order, analog face, the bar's countdown, extra clocks. */
function settingsPage(options: ClockOptions): HTMLElement {
  const choice = (label: string, picked: boolean) => chip({ label, kind: picked ? "Selected" : "Neutral", press: "press", tabStop: false });
  const setting = (title: string, note: string | null, ...choices: HTMLElement[]) =>
    stack("s8", "start", filling(stack("s4", "start", text(title, "body_strong"), ...(note ? [text(note, "small", "on_surface_variant")] : []))), ...choices.map(filling));
  const zones = ZONES.slice(0, options.clocks ?? 1);
  return stack(
    "s16",
    "start",
    ...[
      setting("Holidays", "Marked in the calendar and left out of working days", select({ options: ["None", "Germany", "Japan", "Poland", "United Kingdom", "United States"], picked: 3, icon: "flag", ...(options.holidaysOpen ? { open: true } : {}) })),
      setting("First day of the week", null, row("s4", "centre", ...["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map((day, index) => choice(day, index === 0)))),
      setting("Time format", "How the bar, the lock screen and this popup write the time", row("s4", "centre", choice("Locale", true), choice("24-hour", false), choice("12-hour", false))),
      setting("Date order", null, row("s4", "centre", choice("Locale", true), choice("Day first", false), choice("Month first", false))),
      setting("Analog face", null, row("s4", "centre", choice("Off", !options.analog), choice("On", options.analog ?? false))),
      setting("Countdown in the bar", null, flow("s4", choice("None", options.pinned === null), ...COUNTDOWNS.slice(0, options.countdowns ?? 2).map((each) => choice(each.name, each.name === (options.pinned === undefined ? "Vacation" : options.pinned))))),
      setting(
        "Extra clocks",
        "Up to four, by city or IANA name",
        ...zones.map((each) => row(0, "centre", text(`${each.label}  ·  ${each.name}`, "body"), fill(), iconButton({ icon: "close", label: "Remove", kind: "Text", size: "Sm" }))),
        ...(zones.length < 4 ? [field({ icon: "add_location_alt", placeholder: "Tokyo, New York, America/Chicago" })] : []),
      ),
      clamped("Kept in ~/.config/kubaos/desktop/clock/clock.toml", "small", "on_surface_variant"),
    ].map(filling),
  );
}

/** The popup's content: `space.s20` above and below, the container padding at its sides. */
export function clockPopup(options: ClockOptions = {}): HTMLElement {
  let page = options.page ?? "main";
  const body = document.createElement("span");
  body.className = "ck-body";
  const box = container(body);
  box.classList.add("ck-popup");

  function draw(): void {
    const shown: ClockOptions = { ...options, page };
    const parts: HTMLElement[] = [
      head(shown, () => {
        if (!options.live) return;
        page = page === "main" ? "settings" : "main";
        draw();
      }),
    ];
    if (page === "main") {
      const clocks = ZONES.slice(0, options.clocks ?? 1);
      if (clocks.length) parts.push(flow("s8", ...clocks.map(zone)));
      const query = options.query ?? "";
      const search = field({ icon: "event", placeholder: "next friday, days until christmas, 12 mar to 4 jul, 15:00 tokyo", value: query, ...(options.searching || query ? { pinned: "lit" as const } : {}) });
      parts.push(stack("s8", "start", filling(search), ...(query ? [filling(answer(options))] : [])));
      const month = calendar({ shown: { year: 2026, month: 10 }, today: { year: 2026, month: 10, day: 27 }, picked: { year: 2026, month: 10, day: options.picked ?? 27 }, firstWeekday: 1, holidays: { "2026-11-01": "All Saints' Day" }, ...(options.year ? { year: true } : {}) });
      const left = stack("s12", "start", month, filling(dayDetails(options)));
      left.classList.add("ck-month");
      const side = stack("s16", "start", filling(countdowns(options)), filling(tools(options)));
      side.classList.add("ck-side");
      parts.push(row("s20", "start", left, side));
    } else {
      parts.push(settingsPage(options));
    }
    if (options.unavailable) parts.push(clamped("kubaos-clock isn't running: the calendar and the search need it", "small", "error"));
    body.replaceChildren(...parts.map(filling));
  }

  draw();
  return box;
}

/** A screen `width` logical pixels wide, its bar given way as that width needs, the popup open on the top edge. */
export function clockOnScreen(options: ClockOptions = {}, width = 1440, height = TALL, reveal?: number): HTMLElement {
  const room = barScreen(width, height, { content: { ...EVERYDAY, countdown: true } });
  room.element.dataset.open = "";
  hang(room, sheet({ content: clockPopup(options), edge: "top", ...(reveal !== undefined ? { reveal } : {}) }));
  return room.element;
}
