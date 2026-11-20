// The WeatherSky page, a DRAFT: the sky behind the clock popup's weather cards, live at the three
// sizes the popup draws it at, with a control for the hour, the weather and every reading; the
// nine weathers by day, at dusk and by night; the still card and reduced motion.

import "./weather-sky.css";
import { list, listRow } from "../../lib/list.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { select } from "../../lib/select.ts";
import { slider } from "../../lib/slider.ts";
import { tabs } from "../../lib/tabs.ts";
import { text } from "../../lib/text.ts";
import { type Hail, type Readings, TYPICAL, type Weather, weatherSky, type WeatherSky, WEATHERS } from "../../lib/weather-sky.ts";
import { doesntApply, nonEmpty, page, type Variant } from "../../site/page.ts";

/** Each weather's name, as a card says it. */
const NAMES: Record<Weather, string> = {
  clear: "Clear",
  fair: "Fair",
  partly: "Partly cloudy",
  overcast: "Overcast",
  fog: "Fog",
  rain: "Rain",
  thunderstorm: "Thunderstorm",
  snow: "Snow",
  hail: "Hail",
};

type Size = "home" | "banner" | "city";

interface CardOptions {
  readonly size: Size;
  readonly hour: number;
  readonly weather: Weather;
  readonly readings?: Partial<Readings>;
  readonly still?: boolean;
  readonly reduced?: boolean;
  readonly seed?: number;
}

/** A sample card: the sky at `size`, and the stand-in text what falls lands on. */
interface Card {
  readonly box: HTMLElement;
  readonly sky: WeatherSky;
  /** Writes the condition word for `weather`. */
  readonly say: (weather: Weather) => void;
}

function card(options: CardOptions): Card {
  const sky = weatherSky({
    hour: options.hour,
    weather: options.weather,
    ...(options.readings ? { readings: options.readings } : {}),
    ...(options.still ? { still: true } : {}),
    ...(options.reduced !== undefined ? { reduced: options.reduced } : {}),
    seed: options.seed ?? 0,
  });
  sky.element.classList.add("ws-card");
  sky.element.dataset.size = options.size;
  const city = options.size === "city";
  const words = document.createElement("div");
  words.className = "ws-text";
  const lines = document.createElement("div");
  lines.className = "ws-lines";
  const figure = text("12°", city ? "headline" : "hero_number");
  const line = text("Warsaw · 14° / 7°", city ? "caption" : "small");
  const say = (weather: Weather) => lines.replaceChildren(text(NAMES[weather], city ? "label" : "subheading"), line);
  say(options.weather);
  words.append(figure, lines);
  sky.content.append(words);
  sky.landOn([words]);
  return { box: sky.element, sky, say };
}

/** The hour as a clock reads it, by the locale. */
const CLOCK = new Intl.DateTimeFormat("en-GB", { hour: "2-digit", minute: "2-digit", timeZone: "UTC" });
const clockOf = (hour: number) => CLOCK.format(new Date(Math.round(hour * 60) * 60_000));

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

// The icons for humidity, rain and snow are stand-ins: the symbol set has no drop or flake yet.
const DIALS: readonly Dial[] = [
  { key: "wind", name: "Wind", icon: "mode_fan", min: 0, max: 25, step: 0.5, format: unit("meter-per-second", 1) },
  { key: "visibility", name: "Visibility", icon: "visibility", min: 0.5, max: 40, step: 0.5, format: unit("kilometer", 1) },
  { key: "uv", name: "UV index", icon: "light_mode", min: 0, max: 11, step: 1, format: new Intl.NumberFormat("en-GB") },
  { key: "humidity", name: "Humidity", icon: "wave_sine", min: 10, max: 100, step: 1, format: unit("percent") },
  { key: "air", name: "Air quality", icon: "eco", min: 0, max: 150, step: 1, format: new Intl.NumberFormat("en-GB") },
  { key: "feels", name: "Feels like", icon: "thermostat", min: -15, max: 38, step: 1, format: unit("celsius") },
  { key: "rain", name: "Rain", icon: "arrow_downward", min: 0, max: 30, step: 0.1, format: unit("millimeter-per-hour", 1) },
  { key: "snow", name: "Snow", icon: "arrow_downward", min: 0, max: 5, step: 0.1, format: unit("centimeter-per-hour", 1) },
];

const HAILS: readonly Hail[] = ["none", "slight", "heavy"];

/** A settings row: its name, and its control in the control column. */
function ending(title: string, control: HTMLElement): HTMLElement {
  const made = listRow({ title });
  made.firstElementChild?.append(within("var(--size-menu)", control));
  return made;
}

/**
 * Live: the banner and the home card live and the city card still, one control for the hour, the
 * weather and each reading. Picking a weather sets its typical readings; the hour lands at once.
 */
function live(): HTMLElement {
  let weather: Weather = "rain";
  let readings: Readings = { ...TYPICAL[weather].readings, rain: 7.5 };
  let hour = 14;
  const cards = [
    card({ size: "banner", hour, weather, readings, seed: 11 }),
    card({ size: "home", hour, weather, readings, seed: 23 }),
    card({ size: "city", hour, weather, readings, still: true, seed: 37 }),
  ];
  const dials = new Map<Dial["key"], HTMLElement>();
  const rebuildDials: (() => void)[] = [];
  const hourDial = slider({
    icon: "schedule",
    size: "Compact",
    value: hour / 24,
    number: (v) => clockOf(v * 24),
    widest: "00:00",
    changed: (v) => {
      hour = Math.round(v * 24 * 20) / 20;
      for (const each of cards) each.sky.setHour(hour, true);
    },
  });
  const dialOf = (dial: Dial) => {
    const range = dial.max - dial.min;
    const at = (value: number) => (value - dial.min) / range;
    return slider({
      icon: dial.icon,
      size: "Compact",
      value: at(readings[dial.key]),
      number: (v) => dial.format.format(dial.min + Math.round((v * range) / dial.step) * dial.step),
      widest: dial.format.format(dial.max),
      changed: (v) => {
        const value = dial.min + Math.round((v * range) / dial.step) * dial.step;
        readings = { ...readings, [dial.key]: value };
        for (const each of cards) each.sky.setReadings({ [dial.key]: value });
      },
    });
  };
  const hailOf = () =>
    tabs({
      choices: [{ label: "None" }, { label: "Slight" }, { label: "Heavy" }],
      picked: HAILS.indexOf(readings.hail),
      onPick: (index) => {
        const hail = HAILS[index] ?? "none";
        readings = { ...readings, hail };
        for (const each of cards) each.sky.setReadings({ hail });
      },
    });
  const rows = list([
    ending("Hour", hourDial),
    ending(
      "Weather",
      select({
        options: WEATHERS.map((w) => NAMES[w]),
        picked: WEATHERS.indexOf(weather),
        onPick: (index) => {
          weather = WEATHERS[index] ?? "clear";
          readings = { ...TYPICAL[weather].readings };
          for (const each of cards) {
            each.sky.setWeather(weather);
            each.say(weather);
          }
          for (const rebuild of rebuildDials) rebuild();
        },
      }),
    ),
    ...DIALS.map((dial) => {
      const made = ending(dial.name, dialOf(dial));
      dials.set(dial.key, made);
      rebuildDials.push(() => {
        const fresh = ending(dial.name, dialOf(dial));
        dials.get(dial.key)?.replaceWith(fresh);
        dials.set(dial.key, fresh);
      });
      return made;
    }),
    (() => {
      let made = ending("Hail", hailOf());
      rebuildDials.push(() => {
        const fresh = ending("Hail", hailOf());
        made.replaceWith(fresh);
        made = fresh;
      });
      return made;
    })(),
  ]);
  return stack("s24", "start", stack("s16", "start", cards[0]!.box, row("s16", "end", cards[1]!.box, cards[2]!.box)), within("var(--size-column)", rows));
}

/** A still card at `hour` in `weather`, for the tables. */
const still = (size: Size, hour: number, weather: Weather, seed = 0) => () => card({ size, hour, weather, still: true, seed }).box;

/** The nine weathers at `hour`, one still home card each. */
function weathersAt(hour: number): ReturnType<typeof nonEmpty<Variant>> {
  return nonEmpty(WEATHERS.map((weather, i) => ({ name: NAMES[weather], for: `typical readings, ${clockOf(hour)}`, sample: still("home", hour, weather, i * 7) })));
}

/** A row of eight still cities, as the popup's second row of cities draws them. */
function cities(): HTMLElement {
  const places: readonly [number, Weather][] = [
    [21, "fair"],
    [8, "rain"],
    [13, "overcast"],
    [13, "snow"],
    [9, "partly"],
    [4, "fog"],
    [23, "clear"],
    [15, "clear"],
  ];
  return stack("s8", "start", ...[places.slice(0, 4), places.slice(4)].map((half) => row("s8", "start", ...half.map(([hour, weather], i) => card({ size: "city", hour, weather, still: true, seed: i * 5 }).box))));
}

export default page({
  level: "molecules",
  slug: "weather-sky",
  title: "WeatherSky",
  lede: "A live sky for an hour and a weather, with the forecast's rain, snow or hail landing on the card's text.",
  code: { unbuilt: "Rust and QML: not built yet" },
  stage: { live, hint: "pick a weather, drag the readings", daylight: true },
  variants: [
    { name: "Live", for: "the popup's open cards: it moves, things fall", sample: () => card({ size: "home", hour: 14, weather: "rain", readings: { rain: 7.5 }, seed: 3 }).box },
    { name: "Still", for: "the small city cards: one frame, nothing falls", sample: still("city", 14, "rain", 3) },
    { name: "Reduced motion", for: "a still sky, nothing falls, no lightning", sample: () => card({ size: "home", hour: 14, weather: "thunderstorm", reduced: true, seed: 3 }).box },
  ],
  sizes: [
    { name: "Banner", for: "the weather view's banner, live", sample: () => card({ size: "banner", hour: 14, weather: "snow", seed: 5 }).box },
    { name: "Home card", for: "the clock view's weather card, live", sample: () => card({ size: "home", hour: 14, weather: "snow", seed: 5 }).box },
    { name: "City card", for: "a city beside the clock, still", sample: still("city", 14, "snow", 5) },
  ],
  groups: [
    { title: "By day, 13:00", rows: weathersAt(13) },
    { title: "At dusk, 19:00", rows: weathersAt(19) },
    { title: "By night, 23:00", rows: weathersAt(23) },
  ],
  states: {
    shown: ["rest"],
    absent: {
      lit: "it takes no pointer or keyboard",
      pressed: "it takes no press",
      disabled: "nothing to disable: reduced motion stills it",
      selected: "it holds no choice",
      loading: "drawn at once; the precipitation follows",
      empty: "every weather has a sky",
      error: "with no WebGL its box stays dark",
    },
    rows: nonEmpty([
      { name: "Live", cell: () => card({ size: "home", hour: 14, weather: "rain", readings: { rain: 7.5 }, seed: 9 }).box },
      { name: "Still", cell: still("city", 14, "rain", 9) },
    ]),
    looks: { rest: "the sky for its hour and weather" },
  },
  focus: doesntApply("not a control: it has no line."),
  content: {
    text: doesntApply("the text is its owner's; what falls lands on any."),
    missingPicture: doesntApply("it draws its picture itself; there is nothing to miss."),
    counts: [{ name: "Eight cities", for: "eight still skies, one WebGL context", sample: cities }],
  },
  motion: {
    arrives: doesntApply("it is drawn with its card."),
    leaves: doesntApply("it goes with its card."),
    between: doesntApply("a change of weather blends on the sky's own clock; try it in Live."),
  },
  keyboard: doesntApply("it takes no keys; its owner's controls do."),
  measures: {
    samples: nonEmpty([{ name: "Home card", for: "the sample's inset and corner", sample: () => card({ size: "home", hour: 14, weather: "fair", seed: 1 }).box }]),
    measures: [
      { part: "Sample text inset", token: "space-s16" },
      { part: "Sample corner", token: "radius-content" },
      { part: "Text halo", token: "size-halo" },
    ],
    reflow: doesntApply("it fills whatever box its owner gives it."),
  },
});
