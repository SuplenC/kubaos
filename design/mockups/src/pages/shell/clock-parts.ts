// The clock popup, a DRAFT, from the library's parts with the shots' stand-ins: Tuesday 27 October
// 2026, 21:47 in Warsaw, home, and the cities of `weather-data.ts`. One panel, as tall as what it
// holds, in two views:
//
// - The clock view: the time large and the date under it beside home's weather card; the other
//   cities' cards; the month, and beside it the picked day.
// - The weather view: Back and the city's name, its banner, its warning, twelve hours, six tiles,
//   the week, and where the forecast comes from.
//
// A press on a weather card turns the panel to that city's weather: the card grows into the
// banner's place while the clock view fades, and the weather view's rows rise in one after
// another; Back plays it backwards into the card it came from. Reduced motion swaps the views.
// The clock's pages build their samples from these. This file is no page.
//
// The clock popup's old settings page stays below (`clockSettingsOnScreen`) for its own page,
// until Settings takes the clock's settings.

import "./clock.css";
import { banner } from "../../lib/banner.ts";
import { button } from "../../lib/button.ts";
import { calendar, type Day } from "../../lib/calendar.ts";
import { card } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { descriptionList } from "../../lib/description.ts";
import { field } from "../../lib/field.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { icon } from "../../lib/icon.ts";
import { container, fill, filling, flow, row, stack } from "../../lib/layout.ts";
import { select } from "../../lib/select.ts";
import { hang, sheet } from "../../lib/sheet.ts";
import { tabs } from "../../lib/tabs.ts";
import { clamped, text } from "../../lib/text.ts";
import { cubic, durationMs, length } from "../../lib/tokens.ts";
import { type CardSky, cityCard, cityRow, type HomeState, homeWeatherCard, MOST_CITIES, type WeatherCard, weatherBanner, type WeatherBannerOptions } from "../../lib/weather-card.ts";
import { settings } from "../../site/settings.ts";
import { barScreen, EVERYDAY } from "./bar-parts.ts";
import { TALL } from "./control-center-parts.ts";
import { type City, CITIES, CONDITIONS, clockOf, dayName, dayRange, degrees, HOME, hourOf, hoursOf, offsetOf, readingsOf, symbolOf, tilesOf, UPDATED } from "./weather-data.ts";

export { TALL };

/** Today, at the shots' moment. */
const TODAY: Day = { year: 2026, month: 10, day: 27 };

/** Poland's holidays round the shots' day: home's country's, as `kubaos-clock` reads them. */
const HOLIDAYS: Readonly<Record<string, string>> = {
  "2026-11-01": "All Saints' Day",
  "2026-11-11": "Independence Day",
  "2026-12-25": "Christmas Day",
  "2026-12-26": "Second Day of Christmas",
};

/** Whose weather: home's, or a city's by its index in `CITIES`. */
export type Whose = "home" | number;

/** What home's card shows: a reading, offline a while, offline past a day, or no home city. */
export type HomeShown = "reading" | "aged" | "no connection" | "no city";

export interface ClockOptions {
  /** The view open: the clock's, or a city's weather. */
  readonly view?: "clock" | "weather";
  /** Whose weather the weather view shows. */
  readonly whose?: Whose;
  /** How many other cities, of `CITIES`: none to eight. */
  readonly cities?: number;
  /** The cities' order to take them in, by their index in `CITIES`, for a sample of long names. */
  readonly order?: readonly number[];
  readonly home?: HomeShown;
  /** The picked day in the month. */
  readonly picked?: Day;
  /** The weather view's day, from today. */
  readonly day?: number;
  /** Takes presses: the cards open the weather, Back goes back, the week and the month pick. */
  readonly live?: boolean;
  /** Drawn this far through the turn to the weather view, 0 to 1: a picture of it moving. */
  readonly turning?: number;
}

/** A duration token in milliseconds under the speed setting; none with reduced motion. */
function ms(name: string): number {
  return settings().reduced ? 0 : durationMs(name) / settings().speed;
}

const easing = (name: string) => `cubic-bezier(${cubic(`easing-${name}`).join(", ")})`;

const DATE = new Intl.DateTimeFormat("en-GB", { weekday: "long", day: "numeric", month: "long", year: "numeric", timeZone: "UTC" });
const DAY_TITLE = new Intl.DateTimeFormat("en-GB", { weekday: "long", day: "numeric", month: "long", timeZone: "UTC" });

const utc = (day: Day) => Date.UTC(day.year, day.month - 1, day.day);
const DAY_MS = 86_400_000;

/** The ISO week of `day`. */
function weekOf(day: Day): number {
  const at = new Date(utc(day));
  const thursday = utc(day) + (3 - ((at.getUTCDay() + 6) % 7)) * DAY_MS;
  const year = new Date(thursday).getUTCFullYear();
  return 1 + Math.floor((thursday - Date.UTC(year, 0, 1)) / (7 * DAY_MS));
}

/** `day`'s place in its year, from 1. */
const dayOfYear = (day: Day) => 1 + Math.round((utc(day) - Date.UTC(day.year, 0, 1)) / DAY_MS);

const holidayOf = (day: Day) => HOLIDAYS[`${day.year}-${String(day.month).padStart(2, "0")}-${String(day.day).padStart(2, "0")}`];

/** Working days after `day` to its month's end: Monday to Friday, holidays left out. */
function workingDaysLeft(day: Day): number {
  const last = new Date(Date.UTC(day.year, day.month, 0)).getUTCDate();
  let count = 0;
  for (let d = day.day + 1; d <= last; d++) {
    const at = { ...day, day: d };
    const weekday = new Date(utc(at)).getUTCDay();
    if (weekday !== 0 && weekday !== 6 && !holidayOf(at)) count++;
  }
  return count;
}

/** The city `whose` names. */
const placeOf = (whose: Whose): City => (whose === "home" ? HOME : CITIES[whose]!);

/** A city's sky on a day from today: now's hour today, midday on another. */
function skyOf(place: City, day: number, seed: number): CardSky {
  return { hour: day === 0 ? hourOf(place) : 13, weather: place.week[day]!, readings: readingsOf(place, day), seed };
}

/** Home's card as `shown` says, or none with no home city. */
function homeState(shown: HomeShown): HomeState {
  if (shown === "no city") return { state: "no city" };
  if (shown === "no connection") return { state: "no connection", place: HOME.name };
  return {
    state: "reading",
    sky: skyOf(HOME, 0, 3),
    place: HOME.name,
    temperature: degrees(HOME.temperature),
    condition: CONDITIONS[HOME.weather],
    high: degrees(HOME.high),
    low: degrees(HOME.low),
    ...(shown === "aged" ? { age: "2 h ago" } : {}),
  };
}

/** The picked day beside the month: its date, its week, its day of the year, the working days left, its holiday, its forecast. */
function pickedDay(day: Day): HTMLElement {
  const today = utc(day) === utc(TODAY);
  const title = text(today ? `Today, ${DAY_TITLE.format(utc(day))}` : DAY_TITLE.format(utc(day)), "body_strong");
  const holiday = holidayOf(day);
  const facts = descriptionList([
    { label: "Week", value: String(weekOf(day)) },
    { label: "Day of the year", value: String(dayOfYear(day)) },
    { label: "Working days left", value: String(workingDaysLeft(day)) },
    ...(holiday ? [{ label: "Holiday", value: holiday }] : []),
  ]);
  const ahead = Math.round((utc(day) - utc(TODAY)) / DAY_MS);
  const parts: HTMLElement[] = [filling(title), filling(facts)];
  if (ahead >= 0 && ahead < 7) {
    const weather = HOME.week[ahead]!;
    const [high, low] = dayRange(HOME, ahead);
    parts.push(row("s8", "centre", icon(symbolOf(weather), "var(--type-body-icon)", "var(--color-on_surface_variant)"), text(`${degrees(high)} / ${degrees(low)} · ${CONDITIONS[weather]}`, "body", "on_surface_variant")));
  }
  const box = stack("s12", "start", ...parts);
  box.classList.add("ck-picked");
  return box;
}

/** A view drawn, with the cards it holds, so it can let their skies go. */
interface View {
  readonly element: HTMLElement;
  readonly cards: WeatherCard[];
  /** The card each city's weather turns from and back into, by whose it is. */
  readonly cardOf: (whose: Whose) => HTMLElement | null;
  /** The rows that rise in one after another, top to bottom. */
  readonly rows: HTMLElement[];
}

function clockView(options: ClockOptions, open: (whose: Whose, from: HTMLElement) => void): View {
  const cards: WeatherCard[] = [];
  const byWhose = new Map<Whose, HTMLElement>();
  const time = stack("s4", "start", text("21:47", "hero_number"), text(DATE.format(utc(TODAY)), "body_large", "on_surface_variant"));
  time.classList.add("ck-time");
  const shown = options.home ?? "reading";
  const home = homeWeatherCard({ home: homeState(shown), ...(options.live && shown !== "no city" ? { onPress: () => open("home", home.element) } : {}) });
  cards.push(home);
  if (shown !== "no city") byWhose.set("home", home.element);
  const top = document.createElement("div");
  top.className = "ck-top";
  top.append(time, home.element);
  const rows: HTMLElement[] = [top];
  const order = options.order ?? CITIES.map((_, i) => i);
  const count = Math.min(MOST_CITIES, options.cities ?? 4);
  if (count > 0) {
    const made = order.slice(0, count).map((index) => {
      const place = CITIES[index]!;
      const each = cityCard({
        sky: skyOf(place, 0, index * 5),
        name: place.name,
        time: clockOf(hourOf(place)),
        temperature: degrees(place.temperature),
        offset: offsetOf(place),
        ...(options.live ? { onPress: () => open(index, each.element) } : {}),
      });
      byWhose.set(index, each.element);
      cards.push(each);
      return each.element;
    });
    rows.push(cityRow(made));
  }
  let picked = options.picked ?? TODAY;
  let details = pickedDay(picked);
  const month = calendar({
    shown: { year: picked.year, month: picked.month },
    today: TODAY,
    picked,
    firstWeekday: 1,
    holidays: HOLIDAYS,
    weekNumbers: false,
    weekLine: true,
    onPick: (day) => {
      picked = day;
      const fresh = pickedDay(day);
      details.replaceWith(fresh);
      details = fresh;
    },
  });
  const lower = document.createElement("div");
  lower.className = "ck-lower";
  lower.append(month, details);
  rows.push(lower);
  const element = document.createElement("div");
  element.className = "ck-view";
  element.append(...rows);
  return { element, cards, cardOf: (whose) => byWhose.get(whose) ?? null, rows };
}

/** A city's twelve hours on a day: hour, symbol, temperature, chance of rain. */
function hoursRow(place: City, day: number): HTMLElement {
  const box = document.createElement("div");
  box.className = "ck-hours";
  for (const hour of hoursOf(place, day)) {
    const cell = stack("s4", "centre", text(hour.label, "small", "on_surface_variant"), icon(hour.symbol, "var(--type-body-icon)", "var(--color-on_surface)"), text(hour.temperature, "body_strong"), text(hour.rain, "small", "cold"));
    cell.classList.add("ck-hour");
    box.append(cell);
  }
  return box;
}

/** The six tiles: wind, humidity, UV index, air quality, sunrise, sunset. */
function tilesRow(place: City, day: number): HTMLElement {
  const box = document.createElement("div");
  box.className = "ck-tiles";
  for (const tile of tilesOf(place, day)) box.append(card({ header: { glyph: tile.symbol, title: tile.value, subtitle: tile.name } }));
  return box;
}

function bannerOf(place: City, day: number): WeatherBannerOptions {
  const [high, low] = dayRange(place, day);
  const r = readingsOf(place, day);
  return { sky: skyOf(place, day, 7), temperature: degrees(day === 0 ? place.temperature : high), condition: CONDITIONS[place.week[day]!], feels: degrees(r.feels), high: degrees(high), low: degrees(low) };
}

function weatherView(options: ClockOptions, back: () => void): View & { readonly banner: HTMLElement } {
  const place = placeOf(options.whose ?? "home");
  let day = options.day ?? 0;
  const head = row("s12", "centre", button({ label: "Clock", kind: "Text", icon: "chevron_left", size: "Sm" }), text(place.name, "heading"));
  head.querySelector("button")?.addEventListener("click", () => options.live && back());
  const big = weatherBanner(bannerOf(place, day));
  const rows: HTMLElement[] = [head];
  if (typeof place.warning === "object" && "title" in place.warning) rows.push(banner({ tone: "Warning", icon: "warning", title: place.warning.title, body: place.warning.body }));
  else if (typeof place.warning === "object") rows.push(filling(clamped(place.warning.noSource, "small", "on_surface_variant")));
  let hours = hoursRow(place, day);
  let tiles = tilesRow(place, day);
  const week = document.createElement("div");
  week.className = "ck-week";
  const ranges = document.createElement("div");
  ranges.className = "ck-ranges";
  for (let d = 0; d < 7; d++) {
    const [high, low] = dayRange(place, d);
    ranges.append(text(`${degrees(high)} / ${degrees(low)}`, "small", "on_surface_variant"));
  }
  week.append(
    tabs({
      choices: Array.from({ length: 7 }, (_, d) => ({ label: dayName(d), icon: symbolOf(place.week[d]!) })),
      picked: day,
      onPick: (index) => {
        day = index;
        big.show(bannerOf(place, day));
        const freshHours = hoursRow(place, day);
        hours.replaceWith(freshHours);
        hours = freshHours;
        const freshTiles = tilesRow(place, day);
        tiles.replaceWith(freshTiles);
        tiles = freshTiles;
      },
    }),
    ranges,
  );
  const foot = row(0, "centre", text("Weather from Open-Meteo", "small", "on_surface_variant"), fill(), text(`Updated ${UPDATED}`, "small", "on_surface_variant"));
  rows.push(hours, tiles, week, filling(foot));
  const element = document.createElement("div");
  element.className = "ck-view";
  element.append(head, big.element, ...rows.slice(1));
  return { element, cards: [big], cardOf: () => null, rows, banner: big.element };
}

/** The popup, and how its owner turns it. */
export interface ClockPopup {
  readonly element: HTMLElement;
  /** Turns to `whose`'s weather, from its card where it has one. */
  toWeather(whose: Whose): void;
  /** Back to the clock view. */
  toClock(): void;
}

/** `element`'s box in `within`'s own pixels: the page is zoomed. */
function boxIn(element: HTMLElement, within: HTMLElement): { readonly x: number; readonly y: number; readonly w: number; readonly h: number } {
  const outer = within.getBoundingClientRect();
  const own = element.getBoundingClientRect();
  const zoom = outer.width / Math.max(1, within.offsetWidth) || 1;
  return { x: (own.left - outer.left) / zoom, y: (own.top - outer.top) / zoom, w: own.width / zoom, h: own.height / zoom };
}

/**
 * The turn between the views, played forwards (to the weather) or backwards (to the clock): the
 * card grows into the banner's place, clipped to the card's size and moved onto it at the start;
 * the clock view fades; the weather view's rows rise `space.s16` one after another, `micro` apart;
 * the panel's height eases from the one view's to the other's. Every part over `long` on
 * `emphasized_decel`, the fade over `short`. Played backwards, it runs the same curves in reverse.
 */
function turn(body: HTMLElement, clock: View, weather: View & { readonly banner: HTMLElement }, from: HTMLElement | null, back: boolean): Animation[] {
  const long = ms("long");
  const short = ms("short");
  const micro = ms("micro");
  const decel = easing("emphasized_decel");
  const direction: PlaybackDirection = back ? "reverse" : "normal";
  const played: Animation[] = [];
  const clockHeight = clock.element.offsetHeight;
  const weatherHeight = weather.element.offsetHeight;
  played.push(body.animate([{ height: `${clockHeight}px` }, { height: `${weatherHeight}px` }], { duration: long, easing: decel, direction, fill: "both" }));
  played.push(clock.element.animate([{ opacity: 1 }, { opacity: 0 }], { duration: short, easing: easing("standard"), direction, fill: "both" }));
  if (from) {
    const card = boxIn(from, body);
    const banner = boxIn(weather.banner, body);
    const cardRadius = getComputedStyle(from).borderTopLeftRadius;
    const bannerRadius = getComputedStyle(weather.banner).borderTopLeftRadius;
    played.push(
      weather.banner.animate(
        [
          { translate: `${card.x - banner.x}px ${card.y - banner.y}px`, clipPath: `inset(0 ${banner.w - card.w}px ${banner.h - card.h}px 0 round ${cardRadius})` },
          { translate: "0 0", clipPath: `inset(0 0 0 0 round ${bannerRadius})` },
        ],
        { duration: long, easing: decel, direction, fill: "both" },
      ),
    );
  }
  const rise = length("space-s16");
  weather.rows.forEach((each, index) => {
    played.push(each.animate([{ opacity: 0, translate: `0 ${rise}px` }, { opacity: 1, translate: "0 0" }], { duration: long, delay: back ? 0 : index * micro, endDelay: back ? index * micro : 0, easing: decel, direction, fill: "both" }));
  });
  return played;
}

/** The popup's content: the container padding at its sides, `space.s20` above and below. */
export function clockPopupOf(options: ClockOptions = {}): ClockPopup {
  const body = document.createElement("div");
  body.className = "ck-body";
  const box = container(body);
  box.classList.add("ck-popup");
  let busy = false;
  let shown: View;
  let whose: Whose = options.whose ?? "home";

  const freshClock = () => clockView(options, (to, from) => go(to, from));
  const freshWeather = (to: Whose) => weatherView({ ...options, whose: to, day: 0 }, () => back());
  const free = (view: View) => {
    for (const each of view.cards) each.dispose();
    view.element.remove();
  };

  /** Puts `top` over `under` while they turn, and settles on `stays` after. */
  function overlay(under: View, top: View, played: Animation[], stays: View): void {
    busy = true;
    top.element.classList.add("ck-turning");
    void Promise.all(played.map((each) => each.finished)).then(
      () => {
        for (const each of played) each.cancel();
        top.element.classList.remove("ck-turning");
        free(stays === under ? top : under);
        shown = stays;
        busy = false;
      },
      () => (busy = false),
    );
  }

  function go(to: Whose, from: HTMLElement | null): void {
    if (busy || !shown.element.isConnected) return;
    whose = to;
    const clock = shown;
    const weather = freshWeather(to);
    body.append(weather.element);
    weather.element.classList.add("ck-turning");
    if (settings().reduced) {
      free(clock);
      weather.element.classList.remove("ck-turning");
      shown = weather;
      return;
    }
    overlay(clock, weather, turn(body, clock, weather, from, false), weather);
  }

  function back(): void {
    if (busy) return;
    const weather = shown as View & { readonly banner: HTMLElement };
    const clock = freshClock();
    body.prepend(clock.element);
    weather.element.classList.add("ck-turning");
    if (settings().reduced) {
      free(weather);
      shown = clock;
      return;
    }
    overlay(clock, weather, turn(body, clock, weather, clock.cardOf(whose), true), clock);
  }

  if (options.turning !== undefined) {
    // A picture of the turn: both views drawn, every part held where it is that far through.
    const clock = freshClock();
    body.append(clock.element);
    shown = clock;
    requestAnimationFrame(() => {
      const weather = freshWeather(whose);
      weather.element.classList.add("ck-turning");
      body.append(weather.element);
      const played = turn(body, clock, weather, clock.cardOf(whose), false);
      // One moment for every part: that far through the whole turn, its last row's rise included.
      const whole = Math.max(...played.map((each) => Number(each.effect?.getComputedTiming().endTime ?? 0)));
      for (const each of played) {
        each.pause();
        each.currentTime = (options.turning ?? 0) * whole;
      }
    });
  } else if ((options.view ?? "clock") === "weather") {
    shown = weatherView(options, () => back());
    body.append(shown.element);
  } else {
    shown = freshClock();
    body.append(shown.element);
  }

  return {
    element: box,
    toWeather: (to) => {
      if (shown.rows[0]?.classList.contains("ck-top")) go(to, shown.cardOf(to));
    },
    toClock: () => {
      if (!shown.rows[0]?.classList.contains("ck-top")) back();
    },
  };
}

/** The popup's content alone. */
export function clockPopup(options: ClockOptions = {}): HTMLElement {
  return clockPopupOf(options).element;
}

/**
 * A screen `width` logical pixels wide, its bar given way as that width needs, the popup open on
 * the top edge. Live, the bar's time turns the popup to the clock and its temperature to home's
 * weather.
 */
export function clockOnScreen(options: ClockOptions = {}, width = 1440, height = TALL, reveal?: number): HTMLElement {
  const room = barScreen(width, height, { content: { ...EVERYDAY, ...(options.home === "no city" ? { noHome: true } : {}) } });
  room.element.dataset.open = "";
  const popup = clockPopupOf(options);
  hang(room, sheet({ content: popup.element, edge: "top", ...(reveal !== undefined ? { reveal } : {}) }));
  if (options.live) {
    room.element.querySelector(".shell-clock-capsule")?.addEventListener("click", () => popup.toClock());
    room.element.querySelector(".shell-clock-weather")?.addEventListener("click", () => popup.toWeather("home"));
  }
  return room.element;
}

// The clock popup's old settings, for the Clock settings page until Settings takes them.

/** An extra clock of the old settings: its label and its zone. */
export interface Zone {
  readonly label: string;
  readonly name: string;
}

export const ZONES: readonly Zone[] = [
  { label: "Tokyo", name: "Asia/Tokyo" },
  { label: "New York", name: "America/New_York" },
  { label: "Los Angeles", name: "America/Los_Angeles" },
  { label: "Auckland", name: "Pacific/Auckland" },
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

export interface SettingsOptions {
  readonly analog?: boolean;
  readonly clocks?: number;
  readonly countdowns?: number;
  /** The pinned countdown, by name, or none. */
  readonly pinned?: string | null;
  /** kubaos-clock not running: the line that says so at the foot. */
  readonly unavailable?: boolean;
  /** The holidays select drawn open. */
  readonly holidaysOpen?: boolean;
}

/** The settings: holidays, first weekday, time format, date order, analog face, the bar's countdown, extra clocks. */
function settingsPage(options: SettingsOptions): HTMLElement {
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
      ...(options.unavailable ? [clamped("kubaos-clock isn't running: the calendar and the search need it", "small", "error")] : []),
    ].map(filling),
  );
}

/** The old settings in the popup's sheet, on a screen 1440 wide. */
export function clockSettingsOnScreen(options: SettingsOptions = {}): HTMLElement {
  const body = document.createElement("div");
  body.className = "ck-body";
  body.append(settingsPage(options));
  const box = container(body);
  box.classList.add("ck-popup");
  const room = barScreen(1440, TALL, { content: EVERYDAY });
  room.element.dataset.open = "";
  hang(room, sheet({ content: box, edge: "top" }));
  return room.element;
}
