// The cards that lie on a WeatherSky, DRAFT parts (`docs/design/components/weather-sky.md`): the
// clock popup mounts them and lays them out.
//
// - HomeWeatherCard: home's weather now on a live sky, the temperature large and beside it the
//   condition over "place · high / low"; what falls lands on its words. With no home city it reads
//   "Set your city" on the plain surface; offline, the last reading dimmed with its age, and past a
//   day "No connection" on the surface.
// - WeatherBanner: a city's weather on a live sky, at the head of the weather view: the
//   temperature large, the condition over "Feels like X° · high / low".
// - CityCard: another city on a still sky: its name bottom left over its time, the offset bottom
//   right on the time's line, the temperature top right; the name one line, cut with an ellipsis.
//   CityRow lays one to eight out: one to four share a row, five to eight take a second row of the
//   rest, each row's cards sharing it equally.
//
// Words on a sky are `on_picture`, white in both schemes, with the approved tight shadow and
// nothing else over the picture. They are set in from the card's edges by their ink: the large
// figures' ink sits on the small line's baseline, which stands as far above the card's bottom as
// the ink stands in from its left (`weather-card.css`).

import "./weather-card.css";
import { probe, variant } from "./probe.ts";
import { clamped, inkLeft, text, type TypeStyle } from "./text.ts";
import { type Readings, type Weather, weatherSky, type WeatherSky } from "./weather-sky.ts";

/** A card on the page. */
export interface WeatherCard {
  /** Its box: its owner gives it its width; its height and corners are its own. */
  readonly element: HTMLElement;
  /** Its sky, or none on a card drawn on the plain surface. */
  readonly sky: WeatherSky | null;
  /** Stops its sky and lets it go. */
  dispose(): void;
}

/** The sky a card draws: the hour there, its weather and the forecast's readings. */
export interface CardSky {
  readonly hour: number;
  readonly weather: Weather;
  readonly readings?: Partial<Readings>;
  /** Where its clouds start: cards side by side take different seeds. */
  readonly seed?: number;
  /** Reduced motion whatever the site's setting: for a sample of it. */
  readonly reduced?: boolean;
}

/** `words` in `style`: `on_picture` on a sky, the surface's own on a plain card. */
function said(words: string, style: TypeStyle, onSky: boolean, quiet = false): HTMLElement {
  return text(words, style, onSky ? "on_picture" : quiet ? "on_surface_variant" : "on_surface");
}

/** Marks `element` to be set in by its ink, for the CSS. */
function inked(element: HTMLElement, style: TypeStyle, words: string): HTMLElement {
  element.style.setProperty("--ink-left", String(inkLeft(style, words)));
  element.classList.add("k-weather-inked");
  return element;
}

/** The large temperature, and beside it the condition over one small line. */
function nowWords(temperature: string, condition: string, line: string, onSky: boolean): HTMLElement {
  const figure = inked(said(temperature, "hero_number", onSky), "hero_number", temperature);
  figure.classList.add("k-weather-sits");
  const lines = document.createElement("div");
  lines.className = "k-weather-lines";
  lines.append(said(condition, "subheading", onSky), said(line, "small", onSky));
  const box = document.createElement("div");
  box.className = "k-weather-now";
  box.append(figure, lines);
  return box;
}

interface Made {
  readonly name: string;
  readonly kind: string;
  readonly sky?: CardSky & { readonly still?: boolean };
  readonly words: readonly HTMLElement[];
  readonly dimmed?: boolean;
  readonly onPress?: () => void;
}

/** A card as `made` builds it, with its words' layer for a part to write again. */
interface Built extends WeatherCard {
  readonly words: HTMLElement;
}

function made(spec: Made): Built {
  const sky = spec.sky
    ? weatherSky({ hour: spec.sky.hour, weather: spec.sky.weather, ...(spec.sky.readings ? { readings: spec.sky.readings } : {}), ...(spec.sky.still ? { still: true } : {}), ...(spec.sky.reduced !== undefined ? { reduced: spec.sky.reduced } : {}), seed: spec.sky.seed ?? 0 })
    : null;
  const element = sky?.element ?? document.createElement("div");
  element.classList.add("k-weather-card");
  element.dataset.part = spec.kind;
  element.toggleAttribute("data-plain", !sky);
  const words = document.createElement("div");
  words.className = "k-weather-words";
  words.toggleAttribute("data-dimmed", spec.dimmed ?? false);
  words.append(...spec.words);
  if (sky) {
    sky.content.append(words);
    sky.landOn([words]);
  } else element.append(words);
  if (spec.onPress) {
    element.dataset.pressable = "";
    element.addEventListener("click", spec.onPress);
  }
  probe(words, { role: "component", name: spec.name, variant: variant(!sky && "plain", spec.dimmed && "dimmed"), fixed: { width: false, height: true } });
  return { element, sky, words, dispose: () => sky?.dispose() };
}

/** Blends `card`'s sky to `sky` and writes its words anew. */
function retell(card: Built, sky: CardSky, words: HTMLElement, dimmed = false): void {
  card.sky?.setHour(sky.hour);
  card.sky?.setWeather(sky.weather, sky.readings ?? {});
  card.words.replaceChildren(words);
  card.words.toggleAttribute("data-dimmed", dimmed);
}

/** What home's card shows: a reading, no home city, or no connection for over a day. */
export type HomeState =
  | {
      readonly state: "reading";
      readonly sky: CardSky;
      readonly place: string;
      readonly temperature: string;
      readonly condition: string;
      readonly high: string;
      readonly low: string;
      /** How old the reading is, offline: "2 h ago". Its words are then dimmed. */
      readonly age?: string;
    }
  | { readonly state: "no city" }
  | { readonly state: "no connection"; readonly place: string };

export interface HomeWeatherCardOptions {
  readonly home: HomeState;
  /** What a press does: the popup opens home's weather. */
  readonly onPress?: () => void;
}

/** Home's card on the page. */
export interface HomeWeatherCard extends WeatherCard {
  /**
   * Shows another reading: its sky blends there, its words are written anew. A card made with a
   * reading only takes readings; no home city or no connection is a card of its own.
   */
  show(home: HomeState & { readonly state: "reading" }): void;
}

/** "place · high / low", and the reading's age when it is old. */
function homeLine(home: HomeState & { readonly state: "reading" }): string {
  return home.age ? `${home.place} · ${home.high} / ${home.low} · ${home.age}` : `${home.place} · ${home.high} / ${home.low}`;
}

/** Home's weather card. */
export function homeWeatherCard(options: HomeWeatherCardOptions): HomeWeatherCard {
  const home = options.home;
  const press = options.onPress ? { onPress: options.onPress } : {};
  let card: Built;
  if (home.state === "no city") card = made({ name: "HomeWeatherCard", kind: "home", words: [plain("Set your city")], ...press });
  else if (home.state === "no connection") card = made({ name: "HomeWeatherCard", kind: "home", words: [plain("No connection", home.place)], ...press });
  else card = made({ name: "HomeWeatherCard", kind: "home", sky: home.sky, words: [nowWords(home.temperature, home.condition, homeLine(home), true)], dimmed: home.age !== undefined, ...press });
  return {
    ...card,
    show(next) {
      if (!card.sky) return;
      retell(card, next.sky, nowWords(next.temperature, next.condition, homeLine(next), true), next.age !== undefined);
    },
  };
}

/** Words alone on the plain surface: a title over a quiet line. */
function plain(title: string, line?: string): HTMLElement {
  const lines = document.createElement("div");
  lines.className = "k-weather-lines k-weather-plain";
  lines.append(said(title, "subheading", false), ...(line ? [said(line, "small", false, true)] : []));
  return lines;
}

export interface WeatherBannerOptions {
  readonly sky: CardSky;
  readonly temperature: string;
  readonly condition: string;
  readonly feels: string;
  readonly high: string;
  readonly low: string;
}

/** The banner on the page. */
export interface WeatherBanner extends WeatherCard {
  /** Shows another reading or day: its sky blends there, its words are written anew. */
  show(options: WeatherBannerOptions): void;
}

function bannerWords(options: WeatherBannerOptions): HTMLElement {
  return nowWords(options.temperature, options.condition, `Feels like ${options.feels} · ${options.high} / ${options.low}`, true);
}

/** The weather view's banner. */
export function weatherBanner(options: WeatherBannerOptions): WeatherBanner {
  const card = made({ name: "WeatherBanner", kind: "banner", sky: options.sky, words: [bannerWords(options)] });
  return { ...card, show: (next) => retell(card, next.sky, bannerWords(next)) };
}

export interface CityCardOptions {
  readonly sky: CardSky;
  readonly name: string;
  readonly time: string;
  readonly temperature: string;
  /** How far its clock is from home's: "+8 h". */
  readonly offset: string;
  /** What a press does: the popup opens that city's weather. */
  readonly onPress?: () => void;
}

/** A city's card, on a still sky. */
export function cityCard(options: CityCardOptions): WeatherCard {
  const temperature = said(options.temperature, "heading", true);
  temperature.classList.add("k-weather-temperature");
  const name = clamped(options.name, "body", "on_picture");
  name.classList.add("k-weather-name");
  const time = inked(said(options.time, "headline", true), "headline", options.time);
  time.classList.add("k-weather-sits");
  const offset = said(options.offset, "small", true);
  offset.classList.add("k-weather-sits");
  const line = document.createElement("div");
  line.className = "k-weather-time-line";
  line.append(time, offset);
  const place = document.createElement("div");
  place.className = "k-weather-place";
  place.append(name, line);
  return made({ name: "CityCard", kind: "city", sky: { ...options.sky, still: true }, words: [temperature, place], ...(options.onPress ? { onPress: options.onPress } : {}) });
}

/** The most cities a row of cards holds, and the most cards in all. */
export const CITIES_A_ROW = 4;
export const MOST_CITIES = 8;

/**
 * One to eight city cards: up to four in the first row, the rest in a second, each row's cards
 * sharing its width equally, so a short last row stretches its cards. None draws nothing.
 */
export function cityRow(cards: readonly HTMLElement[]): HTMLElement {
  const box = document.createElement("div");
  box.className = "k-city-row";
  const shown = cards.slice(0, MOST_CITIES);
  for (const part of [shown.slice(0, CITIES_A_ROW), shown.slice(CITIES_A_ROW)]) {
    if (!part.length) continue;
    const line = document.createElement("div");
    line.className = "k-city-line";
    line.append(...part);
    box.append(line);
  }
  return probe(box, { role: "component", name: "CityRow", variant: String(shown.length), fixed: { width: false, height: false } });
}
