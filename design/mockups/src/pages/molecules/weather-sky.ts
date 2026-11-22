// The WeatherSky page, a DRAFT: the sky behind the clock popup's weather, and the three cards that
// lie on it (`lib/weather-card.ts`): home's card and the weather view's banner live, a city's card
// still, live with a control for the hour, the weather and every reading; the nine weathers by
// day, at dusk and by night; home's card with no city and offline; one to eight cities.

import "./weather-sky.css";
import { list, listRow } from "../../lib/list.ts";
import { stack, within } from "../../lib/layout.ts";
import { select } from "../../lib/select.ts";
import { slider } from "../../lib/slider.ts";
import { tabs } from "../../lib/tabs.ts";
import { type CardSky, cityCard, cityRow, type HomeState, homeWeatherCard, weatherBanner, type WeatherBannerOptions } from "../../lib/weather-card.ts";
import { type Hail, type Readings, TYPICAL, type Weather, WEATHERS } from "../../lib/weather-sky.ts";
import type { Pinned } from "../../lib/control.ts";
import { doesntApply, nonEmpty, page, type Variant } from "../../site/page.ts";
import { CITIES, CONDITIONS, clockOf, degrees, HOME, hourOf, offsetOf } from "../shell/weather-data.ts";

/** Home's reading at `sky`: Warsaw's numbers, the condition `sky`'s weather's. */
function homeAt(sky: CardSky, age?: string): HomeState & { readonly state: "reading" } {
  return { state: "reading", sky, place: HOME.name, temperature: degrees(HOME.temperature), condition: CONDITIONS[sky.weather], high: degrees(HOME.high), low: degrees(HOME.low), ...(age ? { age } : {}) };
}

function bannerAt(sky: CardSky): WeatherBannerOptions {
  return { sky, temperature: degrees(HOME.temperature), condition: CONDITIONS[sky.weather], feels: degrees(HOME.feels), high: degrees(HOME.high), low: degrees(HOME.low) };
}

/** A sample's box, its width the popup's room for it: the banner the column, home's card a panel. */
function sized<E extends HTMLElement>(element: E, part: "banner" | "home" | "city"): E {
  element.classList.add("ws-sample");
  element.dataset.sample = part;
  return element;
}

/** A press for a sample: the popup's opens a view; here a card only shows it takes one. */
const press = () => {};

const home = (sky: CardSky, age?: string, pinned?: Pinned) => sized(homeWeatherCard({ home: homeAt(sky, age), onPress: press, ...(pinned ? { pinned } : {}) }).element, "home");
const banner = (sky: CardSky) => sized(weatherBanner(bannerAt(sky)).element, "banner");

/** A city's card as the popup draws it, by its index in the sample cities. */
function city(index: number, sky?: Partial<CardSky>, pinned?: Pinned): HTMLElement {
  const place = CITIES[index]!;
  return cityCard({ sky: { hour: hourOf(place), weather: place.weather, ...(place.readings ? { readings: place.readings } : {}), seed: index * 5, ...sky }, name: place.name, time: clockOf(hourOf(place)), temperature: degrees(place.temperature), offset: offsetOf(place), onPress: press, ...(pinned ? { pinned } : {}) }).element;
}

type CardState = "rest" | "hovered" | "pressed" | "lit";

/** A card's state as the specimens pin it. */
const PINS: Record<CardState, Pinned | undefined> = { rest: undefined, hovered: "hover", pressed: "pressed", lit: "lit" };

/** A reading's control: its range, its step, and how its value reads. */
interface Dial {
  readonly key: Exclude<keyof Readings, "hail">;
  readonly name: string;
  readonly icon: string;
  readonly min: number;
  readonly max: number;
  readonly step: number;
  readonly format: Intl.NumberFormat;
}

const unit = (unit: string, digits = 0) => new Intl.NumberFormat("en-GB", { style: "unit", unit, maximumFractionDigits: digits, minimumFractionDigits: digits });

const DIALS: readonly Dial[] = [
  { key: "wind", name: "Wind", icon: "air", min: 0, max: 25, step: 0.5, format: unit("meter-per-second", 1) },
  { key: "visibility", name: "Visibility", icon: "visibility", min: 0.5, max: 40, step: 0.5, format: unit("kilometer", 1) },
  { key: "uv", name: "UV index", icon: "light_mode", min: 0, max: 11, step: 1, format: new Intl.NumberFormat("en-GB") },
  { key: "humidity", name: "Humidity", icon: "humidity", min: 10, max: 100, step: 1, format: unit("percent") },
  { key: "air", name: "Air quality", icon: "eco", min: 0, max: 150, step: 1, format: new Intl.NumberFormat("en-GB") },
  { key: "feels", name: "Feels like", icon: "thermostat", min: -15, max: 38, step: 1, format: unit("celsius") },
  { key: "rain", name: "Rain", icon: "rainy", min: 0, max: 30, step: 0.1, format: unit("millimeter-per-hour", 1) },
  { key: "snow", name: "Snow", icon: "weather_snowy", min: 0, max: 5, step: 0.1, format: unit("centimeter-per-hour", 1) },
];

const HAILS: readonly Hail[] = ["none", "slight", "heavy"];

/** A settings row: its name, and its control in the control column. */
function ending(title: string, control: HTMLElement): HTMLElement {
  const made = listRow({ title });
  made.firstElementChild?.append(within("var(--size-menu)", control));
  return made;
}

/**
 * Live: the banner and home's card live, Tokyo's card still, one control for the hour, the weather
 * and each reading. Picking a weather sets its typical readings; the hour lands at once.
 */
function live(): HTMLElement {
  let weather: Weather = "rain";
  let readings: Readings = { ...TYPICAL[weather].readings, rain: 7.5 };
  let hour = 14;
  const skyNow = (): CardSky => ({ hour, weather, readings, seed: 11 });
  const big = weatherBanner(bannerAt(skyNow()));
  const small = homeWeatherCard({ home: homeAt(skyNow()) });
  const tokyo = cityCard({ sky: { hour, weather, readings, seed: 37 }, name: "Tokyo", time: clockOf(hour), temperature: degrees(18), offset: "+8 h" });
  const tell = () => {
    big.show(bannerAt(skyNow()));
    small.show(homeAt(skyNow()));
    tokyo.sky?.setHour(hour, true);
    tokyo.sky?.setWeather(weather, readings);
  };
  const dials = new Map<Dial["key"], HTMLElement>();
  const rebuild: (() => void)[] = [];
  const hourDial = slider({
    icon: "schedule",
    size: "Compact",
    value: hour / 24,
    number: (v) => clockOf(v * 24),
    widest: "00:00",
    changed: (v) => {
      hour = Math.round(v * 24 * 20) / 20;
      for (const each of [big, small, tokyo]) each.sky?.setHour(hour, true);
    },
  });
  const dialOf = (dial: Dial) => {
    const range = dial.max - dial.min;
    return slider({
      icon: dial.icon,
      size: "Compact",
      value: (readings[dial.key] - dial.min) / range,
      number: (v) => dial.format.format(dial.min + Math.round((v * range) / dial.step) * dial.step),
      widest: dial.format.format(dial.max),
      changed: (v) => {
        readings = { ...readings, [dial.key]: dial.min + Math.round((v * range) / dial.step) * dial.step };
        tell();
      },
    });
  };
  const hailOf = () =>
    tabs({
      choices: [{ label: "None" }, { label: "Slight" }, { label: "Heavy" }],
      picked: HAILS.indexOf(readings.hail),
      onPick: (index) => {
        readings = { ...readings, hail: HAILS[index] ?? "none" };
        tell();
      },
    });
  let hailRow = ending("Hail", hailOf());
  rebuild.push(() => {
    const fresh = ending("Hail", hailOf());
    hailRow.replaceWith(fresh);
    hailRow = fresh;
  });
  const rows = list([
    ending("Hour", hourDial),
    ending(
      "Weather",
      select({
        options: WEATHERS.map((w) => CONDITIONS[w]),
        picked: WEATHERS.indexOf(weather),
        onPick: (index) => {
          weather = WEATHERS[index] ?? "clear";
          readings = { ...TYPICAL[weather].readings };
          tell();
          for (const each of rebuild) each();
        },
      }),
    ),
    ...DIALS.map((dial) => {
      const made = ending(dial.name, dialOf(dial));
      dials.set(dial.key, made);
      rebuild.push(() => {
        const fresh = ending(dial.name, dialOf(dial));
        dials.get(dial.key)?.replaceWith(fresh);
        dials.set(dial.key, fresh);
      });
      return made;
    }),
    hailRow,
  ]);
  return stack("s24", "start", stack("s8", "start", sized(big.element, "banner"), sized(small.element, "home"), sized(tokyo.element, "city")), within("var(--size-column)", rows));
}

/** The nine weathers at `hour`, one still city card each, named for its weather. */
function weathersAt(hour: number): ReturnType<typeof nonEmpty<Variant>> {
  return nonEmpty(
    WEATHERS.map((weather, i) => ({
      name: CONDITIONS[weather],
      for: `typical readings, ${clockOf(hour)}`,
      sample: () => sized(cityCard({ sky: { hour, weather, seed: i * 7 }, name: CONDITIONS[weather], time: clockOf(hour), temperature: degrees(TYPICAL[weather].readings.feels), offset: "" }).element, "city"),
    })),
  );
}

/** The first `count` sample cities in their row, at the popup's width. */
const cities = (count: number) => () => sized(cityRow(CITIES.slice(0, count).map((_, i) => city(i))), "banner");

const NIGHT = 22;
const DAY = 14;

export default page({
  level: "molecules",
  slug: "weather-sky",
  title: "WeatherSky",
  lede: "A live sky for an hour and a weather, and the cards that lie on it, what falls landing on their words.",
  code: { unbuilt: "Rust and QML: not built yet" },
  stage: { live, hint: "pick a weather, drag the readings", daylight: true },
  variants: [
    { name: "HomeWeatherCard", for: "home's weather, live; a press opens it", sample: () => home({ hour: DAY, weather: "rain", readings: { rain: 7.5 }, seed: 3 }) },
    { name: "WeatherBanner", for: "the weather view's head, live", sample: () => banner({ hour: DAY, weather: "snow", seed: 5 }) },
    { name: "CityCard", for: "a city beside the clock, still", sample: () => sized(city(0), "city") },
    { name: "Reduced motion", for: "a still sky, nothing falls, no lightning", sample: () => home({ hour: DAY, weather: "thunderstorm", reduced: true, seed: 3 }) },
  ],
  sizes: doesntApply("each card's height is its own token; its width is its room."),
  groups: [
    { title: "By day, 14:00", rows: weathersAt(DAY) },
    { title: "At dusk, 19:00", rows: weathersAt(19) },
    { title: "By night, 22:00", rows: weathersAt(NIGHT) },
    {
      title: "CityCard's states",
      rows: nonEmpty(
        (["rest", "hovered", "pressed", "lit"] as const).map((state) => ({ name: state === "lit" ? "Keyboard focus" : state[0]!.toUpperCase() + state.slice(1), for: { rest: "a still sky", hovered: "the veil over the sky", pressed: "the veil a step lighter", lit: "the neon line inside its edge" }[state], sample: () => sized(city(0, {}, PINS[state]), "city") })),
      ),
    },
    {
      title: "Home's card",
      rows: nonEmpty([
        { name: "A reading", for: "live; place · high / low", sample: () => home({ hour: NIGHT, weather: "partly", seed: 2 }) },
        { name: "Offline, 2 h", for: "the last reading dimmed, its age", sample: () => home({ hour: NIGHT, weather: "partly", seed: 2 }, "2 h ago") },
        { name: "Offline over a day", for: "No connection, on the surface", sample: () => sized(homeWeatherCard({ home: { state: "no connection", place: HOME.name } }).element, "home") },
        { name: "No home city", for: "Set your city, on the surface", sample: () => sized(homeWeatherCard({ home: { state: "no city" } }).element, "home") },
        { name: "Hail", for: "what falls is hail", sample: () => home({ hour: DAY, weather: "hail", seed: 4 }) },
      ]),
    },
  ],
  states: {
    shown: ["rest", "hovered", "pressed", "lit", "empty", "error"],
    absent: {
      disabled: "nothing to disable: reduced motion stills it",
      selected: "it holds no choice",
      loading: "drawn at once; the precipitation follows",
    },
    rows: nonEmpty([
      {
        name: "HomeWeatherCard",
        cell: (state: CardState | "empty" | "error") =>
          state === "empty" || state === "error"
            ? sized(homeWeatherCard({ home: state === "empty" ? { state: "no city" } : { state: "no connection", place: HOME.name }, onPress: press }).element, "home")
            : home({ hour: DAY, weather: "rain", readings: { rain: 7.5 }, seed: 9 }, undefined, PINS[state]),
      },
    ]),
    looks: {
      rest: "the sky for its hour and weather",
      hovered: "a faint light veil over the sky",
      pressed: "the veil a step lighter",
      lit: "the accent's neon line inside its edge",
      empty: "no home city: Set your city",
      error: "offline past a day: No connection",
    },
  },
  focus: {
    tubes: [{ name: "HomeWeatherCard, CityCard", tone: "accent", filled: false, content: false }],
    looks: { lit: "the line inside the edge; the glow cut by the card", signsOff: "the line as glass, no glow" },
    signsOff: () => home({ hour: DAY, weather: "rain", readings: { rain: 7.5 }, seed: 9 }, undefined, "signs-off"),
  },
  content: {
    text: doesntApply("a city's long name is cut with an ellipsis: see Counts."),
    missingPicture: doesntApply("it draws its picture itself; there is nothing to miss."),
    counts: [
      { name: "One city", for: "the row to itself", sample: cities(1) },
      { name: "Two", for: "sharing the row", sample: cities(2) },
      { name: "Three", for: "sharing the row", sample: cities(3) },
      { name: "Four", for: "a long name cut", sample: () => sized(cityRow([0, 1, 4, 5].map((i) => city(i))), "banner") },
      { name: "Six", for: "a second row, stretched", sample: cities(6) },
      { name: "Eight, the most", for: "two rows of four", sample: cities(8) },
    ],
  },
  motion: {
    arrives: doesntApply("it is drawn with its card."),
    leaves: doesntApply("it goes with its card."),
    between: doesntApply("a change of weather blends on the sky's own clock; try it in Live."),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "to a pressable card and on" },
      { keys: ["Enter", "Space"], does: "presses the card: its owner opens its weather" },
    ],
    order: "A pressable card is one stop, in its owner's reading order; the banner and a card without a press take none.",
    lands: "Its owner says where the keyboard lands; the card itself never takes it unasked.",
  },
  measures: {
    samples: nonEmpty([
      { name: "HomeWeatherCard", for: "ink insets, heights", sample: () => home({ hour: DAY, weather: "fair", seed: 1 }) },
      { name: "CityCard", for: "ink insets, heights", sample: () => sized(city(5), "city") },
    ]),
    measures: [
      { part: "Home, banner: ink inset", token: "space-s16" },
      { part: "City: ink inset", token: "space-s12" },
      { part: "Home's card height", token: "size-weather_card" },
      { part: "Banner height", token: "size-weather_banner" },
      { part: "City card height", token: "size-city_card" },
      { part: "Between cities", token: "space-s8" },
    ],
    reflow: doesntApply("its width is its room; its height its own."),
  },
});
