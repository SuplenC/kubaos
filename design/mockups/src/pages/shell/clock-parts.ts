// The clock popup, a DRAFT, from the library's parts with the shots' stand-ins: Tuesday 27 October
// 2026, 21:47 in Warsaw, home, and the cities of `weather-data.ts`. One panel, as tall as what it
// holds, in two views:
//
// - The clock view: the time large and the date under it beside home's weather card; the other
//   cities' cards; the month, and beside it the picked day.
// - The weather view: Back and the city's name, its banner, its warning, twelve hours, six tiles,
//   the week, and where the forecast comes from.
//
// A press on a weather card turns the panel to that city's weather: the views change as a side
// panel's content changes view (the clock view slides and fades out to the left, the weather view
// slides and fades in from the right) while the card's live sky grows into the banner's place;
// Back mirrors it, the sky shrinking into the card it came from. Reduced motion swaps the views.
// The clock's pages build their samples from these. This file is no page.
//
// The clock's settings live in the Settings app's Date & time section
// (`apps/settings/sections/date-time.ts`); the popup holds none.

import "./clock.css";
import { banner } from "../../lib/banner.ts";
import { button } from "../../lib/button.ts";
import { calendar, type Day } from "../../lib/calendar.ts";
import { card } from "../../lib/card.ts";
import { descriptionList } from "../../lib/description.ts";
import { icon } from "../../lib/icon.ts";
import { container, fill, filling, row, stack } from "../../lib/layout.ts";
import { hang, sheet, swapViews } from "../../lib/sheet.ts";
import { dayStrip } from "../../lib/day-strip.ts";
import { clamped, text } from "../../lib/text.ts";
import { cubic, durationMs } from "../../lib/tokens.ts";
import { type CardSky, carry, cityCard, cityRow, type HomeState, homeWeatherCard, MOST_CITIES, type WeatherCard, weatherBanner, type WeatherBannerOptions } from "../../lib/weather-card.ts";
import { TYPICAL, type Weather } from "../../lib/weather-sky.ts";
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
  /** Home in another weather than its own: a sample of the turn in rain. */
  readonly homeWeather?: Weather;
  /** The picture of the turn is of the way back to the clock. */
  readonly turningBack?: boolean;
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

/** How many days `year` has. */
const daysIn = (year: number) => Math.round((Date.UTC(year + 1, 0, 1) - Date.UTC(year, 0, 1)) / DAY_MS);

const PLURAL = new Intl.PluralRules("en-GB");

/** The working days left in a month, a message by the count's plural form. */
const WORKING = { one: "{count} working day", other: "{count} working days" } as const;

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
/** Home as `options` show it: its own weather, or the one a sample asks for. */
function homeOf(options: ClockOptions): City {
  const weather = options.homeWeather;
  if (!weather) return HOME;
  return { ...HOME, weather, week: [weather, ...HOME.week.slice(1)] as unknown as City["week"], readings: { ...HOME.readings, ...TYPICAL[weather].readings, ...(weather === "rain" ? { rain: 7.5 } : {}) } };
}

const placeOf = (whose: Whose, options: ClockOptions): City => (whose === "home" ? homeOf(options) : CITIES[whose]!);

/** A city's sky on a day from today: now's hour today, midday on another. */
function skyOf(place: City, day: number, seed: number): CardSky {
  return { hour: day === 0 ? hourOf(place) : 13, weather: place.week[day]!, readings: readingsOf(place, day), seed };
}

/** Home's card as `shown` says, or none with no home city. */
function homeState(shown: HomeShown, home: City): HomeState {
  if (shown === "no city") return { state: "no city" };
  if (shown === "no connection") return { state: "no connection", place: home.name };
  return {
    state: "reading",
    sky: skyOf(home, 0, 3),
    place: home.name,
    temperature: degrees(home.temperature),
    condition: CONDITIONS[home.weather],
    high: degrees(home.high),
    low: degrees(home.low),
    ...(shown === "aged" ? { age: "2 h ago" } : {}),
  };
}

/** The picked day beside the month: its date, its week, its day of the year, the working days left, its holiday, its forecast. */
function pickedDay(day: Day): HTMLElement {
  const today = utc(day) === utc(TODAY);
  const title = text(today ? `Today, ${DAY_TITLE.format(utc(day))}` : DAY_TITLE.format(utc(day)), "body_strong");
  const holiday = holidayOf(day);
  // Labels of one short word, so none wraps in the list's label column at the popup's width; the
  // values carry the rest of the words.
  const left = workingDaysLeft(day);
  const facts = descriptionList([
    { label: "Week", value: String(weekOf(day)) },
    { label: "Day", value: `${dayOfYear(day)} of ${daysIn(day.year)}` },
    { label: "Left", value: WORKING[PLURAL.select(left) === "one" ? "one" : "other"].replace("{count}", String(left)) },
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
  readonly cardOf: (whose: Whose) => WeatherCard | null;
  /** The view's rows, top to bottom. */
  readonly rows: HTMLElement[];
}

function clockView(options: ClockOptions, open: (whose: Whose, from: WeatherCard) => void): View {
  const cards: WeatherCard[] = [];
  const byWhose = new Map<Whose, WeatherCard>();
  const time = stack("s4", "start", text("21:47", "hero_number"), text(DATE.format(utc(TODAY)), "body_large", "on_surface_variant"));
  time.classList.add("ck-time");
  const shown = options.home ?? "reading";
  const home = homeWeatherCard({ home: homeState(shown, homeOf(options)), ...(options.live && shown !== "no city" ? { onPress: () => open("home", home) } : {}) });
  cards.push(home);
  if (shown !== "no city") byWhose.set("home", home);
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
        ...(options.live ? { onPress: () => open(index, each) } : {}),
      });
      byWhose.set(index, each);
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

function weatherView(options: ClockOptions, back: () => void): View & { readonly banner: WeatherCard } {
  const place = placeOf(options.whose ?? "home", options);
  let day = options.day ?? 0;
  const head = row("s12", "centre", button({ label: "Clock", kind: "Text", icon: "chevron_left", size: "Sm" }), text(place.name, "heading"));
  head.querySelector("button")?.addEventListener("click", () => options.live && back());
  const big = weatherBanner(bannerOf(place, day));
  const rows: HTMLElement[] = [head];
  if (typeof place.warning === "object" && "title" in place.warning) rows.push(banner({ tone: "Warning", icon: "warning", title: place.warning.title, body: place.warning.body }));
  else if (typeof place.warning === "object") rows.push(filling(clamped(place.warning.noSource, "small", "on_surface_variant")));
  let hours = hoursRow(place, day);
  let tiles = tilesRow(place, day);
  const week = dayStrip({
    days: Array.from({ length: 7 }, (_, d) => {
      const [high, low] = dayRange(place, d);
      return { name: dayName(d), symbol: symbolOf(place.week[d]!), high: degrees(high), low: degrees(low) };
    }),
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
  });
  const foot = row(0, "centre", text("Weather from Open-Meteo", "small", "on_surface_variant"), fill(), text(`Updated ${UPDATED}`, "small", "on_surface_variant"));
  rows.push(hours, tiles, week, filling(foot));
  const element = document.createElement("div");
  element.className = "ck-view";
  element.append(head, big.element, ...rows.slice(1));
  return { element, cards: [big], cardOf: () => null, rows, banner: big };
}

/** The popup, and how its owner turns it. */
export interface ClockPopup {
  readonly element: HTMLElement;
  /** Turns to `whose`'s weather, from its card where it has one. */
  toWeather(whose: Whose): void;
  /** Back to the clock view. */
  toClock(): void;
}

/** One part's move, running forwards from its first keyframe to its last, held at both ends. */
function moved(element: Element, frames: Keyframe[], duration: number, curve: string, delay = 0): Animation {
  return element.animate(frames, { duration, delay, easing: easing(curve), fill: "both" });
}

/** A turn's moves, and what settles it once they are done. */
interface Turning {
  readonly played: Animation[];
  readonly land: () => void;
}

/**
 * The turn between the views, each way a move of its own that runs forwards, the two views never
 * both to be read at once. The views change as a panel's content changes view (`swapViews`): to
 * the weather, the clock view slides `space.s24` to the left and fades, and the weather view slides
 * in from the right and fades in once it has gone; back to the clock, the same mirrored.
 *
 * The card's own live sky is the one thing that carries across: it grows into the banner's place,
 * or the banner's shrinks back into the card's (`carry`), over `long` on `emphasized_decel`, still
 * drawing, its words travelling on it, sliding with neither view. Its two boxes are read in the
 * views' resting layouts, before either moves. The panel's height eases from the one view's to the
 * other's over `long` on `emphasized_decel`.
 */
function turn(body: HTMLElement, clock: View, weather: View & { readonly banner: WeatherCard }, from: WeatherCard | null, back: boolean): Turning {
  const long = ms("long");
  const short = ms("short");
  // The sky's two boxes first, in the resting layouts.
  const carried = from ? carry(back ? weather.banner : from, back ? from : weather.banner, body, { duration: long, easing: easing("emphasized_decel"), fade: short, fadeEasing: easing("standard") }) : null;
  const heights = [{ height: `${clock.element.offsetHeight}px` }, { height: `${weather.element.offsetHeight}px` }];
  const played: Animation[] = [moved(body, back ? heights.reverse() : heights, long, "emphasized_decel")];
  played.push(...(back ? swapViews(weather.element, clock.element, -1) : swapViews(clock.element, weather.element, 1)));
  // Only the carried sky moves: the leaving clock view's others hold still while it goes.
  if (!back) for (const card of clock.cards) if (card !== from) card.sky?.pause();
  return carried ? { played: [...played, ...carried.moves], land: carried.land } : { played, land: () => {} };
}

/** Runs `then` once the views just drawn have laid out and drawn their skies. */
function drawn(then: () => void): void {
  requestAnimationFrame(() => requestAnimationFrame(then));
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

  /**
   * Turns between `clock` and `weather`, `weather` lying over `clock` meanwhile, and settles on
   * the weather or, `back`, on the clock. `hold` draws it held that far through, a picture of it.
   */
  function play(clock: View, weather: View & { readonly banner: WeatherCard }, from: WeatherCard | null, back: boolean, hold?: number): void {
    busy = true;
    // Until the move starts, the view it starts from is all there is: the other is drawn unseen
    // and the panel keeps the first's height.
    body.style.height = `${(back ? weather : clock).element.offsetHeight}px`;
    weather.element.classList.add("ck-turning");
    (back ? clock : weather).element.style.opacity = "0";
    drawn(() => {
      const { played, land } = turn(body, clock, weather, from, back);
      body.style.height = "";
      clock.element.style.opacity = "";
      weather.element.style.opacity = "";
      if (hold !== undefined) {
        // One moment for every part: that far through the whole turn, its last part included.
        const whole = Math.max(...played.map((each) => Number(each.effect?.getComputedTiming().endTime ?? 0)));
        for (const each of played) {
          each.pause();
          each.currentTime = hold * whole;
        }
        return;
      }
      void Promise.all(played.map((each) => each.finished)).then(
        () => {
          for (const each of played) each.cancel();
          land();
          weather.element.classList.remove("ck-turning");
          free(back ? weather : clock);
          shown = back ? clock : weather;
          busy = false;
        },
        () => (busy = false),
      );
    });
  }

  function go(to: Whose, from: WeatherCard | null): void {
    if (busy || !shown.element.isConnected) return;
    whose = to;
    const clock = shown;
    const weather = freshWeather(to);
    body.append(weather.element);
    if (settings().reduced) {
      free(clock);
      shown = weather;
      return;
    }
    play(clock, weather, from, false);
  }

  function back(): void {
    if (busy) return;
    const weather = shown as View & { readonly banner: WeatherCard };
    const clock = freshClock();
    body.prepend(clock.element);
    if (settings().reduced) {
      free(weather);
      shown = clock;
      return;
    }
    play(clock, weather, clock.cardOf(whose), true);
  }

  if (options.turning !== undefined) {
    // A picture of the turn, there or back: both views drawn, every part held that far through.
    const clock = freshClock();
    const weather = freshWeather(whose);
    body.append(clock.element, weather.element);
    shown = options.turningBack ? weather : clock;
    play(clock, weather, clock.cardOf(whose), options.turningBack ?? false, options.turning);
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
    // The pill's weather opens home's weather; anywhere else on the pill opens the clock.
    room.element.querySelector(".shell-clock-pill")?.addEventListener("click", (event) => {
      if ((event.target as Element).closest(".shell-clock-weather")) popup.toWeather("home");
      else popup.toClock();
    });
  }
  return room.element;
}

/** The countdowns the Settings app's "Countdown in the bar" picks from, as the shots' stand-ins hold them. */
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
