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
// `carry` moves one card's live sky into another's place, its words travelling on it.
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
  /**
   * Its box: its owner gives it its width; its height and corners are its own. A card that took
   * another's sky (`carry`) has that sky's box from then on.
   */
  readonly element: HTMLElement;
  /** Its sky, or none on a card drawn on the plain surface, or one whose sky was carried off. */
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

/** Names `element` as a piece of a card's words, which `carry` takes to the piece of that name. */
function piece<E extends HTMLElement>(element: E, name: string): E {
  element.dataset.piece = name;
  return element;
}

/** The large temperature, and beside it the condition over one small line, the piece `lineName`. */
function nowWords(temperature: string, condition: string, line: string, onSky: boolean, lineName: string): HTMLElement {
  const figure = piece(inked(said(temperature, "hero_number", onSky), "hero_number", temperature), "temperature");
  figure.classList.add("k-weather-sits");
  const lines = document.createElement("div");
  lines.className = "k-weather-lines";
  lines.append(piece(said(condition, "subheading", onSky), "condition"), piece(said(line, "small", onSky), lineName));
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

/** What a card is made of, kept so `carry` can hand its sky to another card. */
interface Inside {
  element: HTMLElement;
  sky: WeatherSky | null;
  readonly words: HTMLElement;
  readonly part: string;
  readonly still: boolean;
  readonly onPress: (() => void) | undefined;
}

const insides = new WeakMap<WeatherCard, Inside>();

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
  const inside: Inside = { element, sky, words, part: spec.kind, still: spec.sky?.still ?? false, onPress: spec.onPress };
  const card: Built = {
    get element() {
      return inside.element;
    },
    get sky() {
      return inside.sky;
    },
    words,
    dispose: () => inside.sky?.dispose(),
  };
  insides.set(card, inside);
  return card;
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
  else card = made({ name: "HomeWeatherCard", kind: "home", sky: home.sky, words: [nowWords(home.temperature, home.condition, homeLine(home), true, "line")], dimmed: home.age !== undefined, ...press });
  return Object.assign(card, {
    show(next: HomeState & { readonly state: "reading" }) {
      if (!card.sky) return;
      retell(card, next.sky, nowWords(next.temperature, next.condition, homeLine(next), true, "line"), next.age !== undefined);
    },
  });
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
  return nowWords(options.temperature, options.condition, `Feels like ${options.feels} · ${options.high} / ${options.low}`, true, "feels");
}

/** The weather view's banner. */
export function weatherBanner(options: WeatherBannerOptions): WeatherBanner {
  const card = made({ name: "WeatherBanner", kind: "banner", sky: options.sky, words: [bannerWords(options)] });
  return Object.assign(card, { show: (next: WeatherBannerOptions) => retell(card, next.sky, bannerWords(next)) });
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
  const temperature = piece(said(options.temperature, "heading", true), "temperature");
  temperature.classList.add("k-weather-temperature");
  const name = piece(clamped(options.name, "body", "on_picture"), "name");
  name.classList.add("k-weather-name");
  const time = piece(inked(said(options.time, "headline", true), "headline", options.time), "time");
  time.classList.add("k-weather-sits");
  const offset = piece(said(options.offset, "small", true), "offset");
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

/** How a carry moves: the box's duration and curve, and the fade of a piece found at one end only. */
export interface CarryTiming {
  /** Milliseconds the box and the pieces it shares take, on `easing` (CSS). */
  readonly duration: number;
  readonly easing: string;
  /** Milliseconds a piece at one end only takes to fade, on `fadeEasing` (CSS). */
  readonly fade: number;
  readonly fadeEasing: string;
}

/** A carry under way: its moves, and what lands it once they are done. */
export interface Carrying {
  readonly moves: Animation[];
  /** Puts the carried sky in the place of the card it became, with that card's words. */
  land(): void;
}

/** `element`'s box in `within`'s own pixels: the page is zoomed. */
function boxIn(element: Element, within: HTMLElement): { x: number; y: number; w: number; h: number } {
  const outer = within.getBoundingClientRect();
  const own = element.getBoundingClientRect();
  const zoom = outer.width / Math.max(1, within.offsetWidth) || 1;
  return { x: (own.left - outer.left) / zoom, y: (own.top - outer.top) / zoom, w: own.width / zoom, h: own.height / zoom };
}

type Box = ReturnType<typeof boxIn>;

const placed = (box: Box) => ({ left: `${box.x}px`, top: `${box.y}px`, width: `${box.w}px`, height: `${box.h}px` });

/** Each named piece of `words`, by its name. */
function piecesOf(words: HTMLElement): Map<string, HTMLElement> {
  return new Map([...words.querySelectorAll<HTMLElement>("[data-piece]")].map((each) => [each.dataset.piece ?? "", each]));
}

/** A layer holding a card's words laid out in `box`, over the sky, nothing cut. */
function carrier(part: string, words: HTMLElement, box: Box): HTMLElement {
  const layer = document.createElement("div");
  layer.className = "k-weather-card k-weather-carrier";
  layer.dataset.part = part;
  Object.assign(layer.style, placed(box));
  layer.append(words);
  return layer;
}

/** Where each named piece of `words` sits in `within` now. */
function piecesAt(words: HTMLElement, within: HTMLElement): Map<string, { readonly x: number; readonly y: number; readonly size: number }> {
  return new Map(
    [...piecesOf(words)].map(([name, each]) => {
      const at = boxIn(each, within);
      return [name, { x: at.x, y: at.y, size: Number(each.dataset.size ?? 1) }] as const;
    }),
  );
}

/**
 * Carries `from`'s live sky into `to`'s place, inside `within`: the one sky changes box (place,
 * size and corners) to `to`'s, drawing every frame as it goes, what falls included, and stays as
 * `to`'s sky when it lands; `to`'s own sky is let go. The words ride on top, never cut by the box:
 * a piece both cards have (the temperature, the condition) travels from its place and size on the
 * one to its place and size on the other, on the box's curve; a piece only `from` has fades out as
 * the move starts, and one only `to` has fades in as it lands. `from`'s place keeps its room,
 * empty, until its owner takes it away. Plain cards carry nothing.
 *
 * Everything that moves is the compositor's: the sky's element is sized once to the larger box and
 * moved and clipped (`transform`, `clip-path`), its buffers never resized, the shader laid out on
 * the part in view (`WeatherSky.follow`); each piece of the words, laid out once in its own card's
 * box, moves by `transform` alone, on a straight line between where its box's move takes it at the
 * start and at the end, which is exactly where it would sit in the box between them, since a
 * piece's place in a box is a straight function of the box's size. Nothing is laid out again
 * until it lands; meanwhile what falls lands on nothing.
 */
export function carry(from: WeatherCard, to: WeatherCard, within: HTMLElement, timing: CarryTiming): Carrying {
  const a = insides.get(from);
  const b = insides.get(to);
  const sky = a?.sky;
  if (!a || !b || !sky || !b.sky) return { moves: [], land: () => {} };
  const start = boxIn(a.element, within);
  const end = boxIn(b.element, within);
  const startRadius = getComputedStyle(a.element).borderTopLeftRadius;
  const endRadius = getComputedStyle(b.element).borderTopLeftRadius;
  const carried = a.element;
  const width = Math.max(start.w, end.w);
  const height = Math.max(start.h, end.h);

  // `from`'s place keeps its room; `to`'s own sky goes, its place kept unseen until the landing.
  const room = document.createElement("div");
  Object.assign(room.style, { flex: "0 0 auto", width: `${start.w}px`, height: `${start.h}px` });
  carried.replaceWith(room);
  b.sky.dispose();
  b.element.style.visibility = "hidden";
  if (a.onPress) carried.removeEventListener("click", a.onPress);

  // The words, each card's laid out once in its own box: measured there and at the other end.
  const leaving = carrier(a.part, a.words, start);
  const coming = carrier(b.part, b.words, start);
  carried.classList.add("k-weather-carried");
  Object.assign(carried.style, { left: "0px", top: "0px", width: `${width}px`, height: `${height}px`, borderRadius: "0" });
  within.append(carried, leaving, coming);
  // Two layouts: each card's words read at the one end, then at the other, where they stay.
  const fromAtStart = piecesAt(a.words, within);
  const toAtStart = piecesAt(b.words, within);
  Object.assign(leaving.style, placed(end));
  Object.assign(coming.style, placed(end));
  const fromAtEnd = piecesAt(a.words, within);
  const toAtEnd = piecesAt(b.words, within);
  Object.assign(leaving.style, placed(start));

  const moving = { duration: timing.duration, easing: timing.easing, fill: "both" as const };
  const clip = (box: Box, radius: string) => `inset(0px ${width - box.w}px ${height - box.h}px 0px round ${radius})`;
  const skyMove = carried.animate(
    [
      { transform: `translate(${start.x}px, ${start.y}px)`, clipPath: clip(start, startRadius) },
      { transform: `translate(${end.x}px, ${end.y}px)`, clipPath: clip(end, endRadius) },
    ],
    moving,
  );
  // The part of the sky in view, for the shader: the box the clip shows, on the move's own curve.
  const eased = () => Number(skyMove.effect?.getComputedTiming().progress ?? 0);
  sky.follow({ width, height, visible: () => ({ width: start.w + (end.w - start.w) * eased(), height: start.h + (end.h - start.h) * eased() }) });
  sky.setStill(false);
  sky.landOn([]);

  const moves: Animation[] = [skyMove];
  const shift = (x: number, y: number, scale = 1) => `translate(${x}px, ${y}px)${scale === 1 ? "" : ` scale(${scale})`}`;
  const fromPieces = piecesOf(a.words);
  const toPieces = piecesOf(b.words);
  for (const [name, mine] of fromPieces) {
    const s0 = fromAtStart.get(name)!;
    const e0 = fromAtEnd.get(name)!;
    const theirs = toPieces.get(name);
    mine.style.transformOrigin = "0 0";
    if (!theirs) {
      moves.push(mine.animate([{ transform: "none" }, { transform: shift(e0.x - s0.x, e0.y - s0.y) }], moving));
      moves.push(mine.animate([{ opacity: 1 }, { opacity: 0 }], { duration: timing.fade, easing: timing.fadeEasing, fill: "both" }));
      continue;
    }
    const s1 = toAtStart.get(name)!;
    const e1 = toAtEnd.get(name)!;
    const scale = s0.size / s1.size;
    theirs.style.transformOrigin = "0 0";
    // The same words in the same style are one piece: it rides alone and the other waits unseen,
    // since two copies fading across each other dim where they overlap. Unlike pieces both ride
    // one path, from this one's place and size to the other's, and cross-fade on the way.
    const ride = shift(e1.x - s0.x, e1.y - s0.y, 1 / scale);
    if (scale === 1 && mine.textContent === theirs.textContent) {
      moves.push(mine.animate([{ transform: "none" }, { transform: ride }], moving), theirs.animate([{ opacity: 0 }, { opacity: 0 }], moving));
      continue;
    }
    moves.push(
      mine.animate([{ transform: "none", opacity: 1 }, { transform: ride, opacity: 0 }], moving),
      theirs.animate([{ transform: shift(s0.x - e1.x, s0.y - e1.y, scale), opacity: 0 }, { transform: "none", opacity: 1 }], moving),
    );
  }
  for (const [name, theirs] of toPieces) {
    if (fromPieces.has(name)) continue;
    const s1 = toAtStart.get(name)!;
    const e1 = toAtEnd.get(name)!;
    moves.push(theirs.animate([{ transform: shift(s1.x - e1.x, s1.y - e1.y) }, { transform: "none" }], moving));
    moves.push(theirs.animate([{ opacity: 0 }, { opacity: 1 }], { duration: timing.fade, delay: Math.max(0, timing.duration - timing.fade), easing: timing.fadeEasing, fill: "both" }));
  }

  return {
    moves,
    land() {
      for (const move of moves) move.cancel();
      for (const each of [...fromPieces.values(), ...toPieces.values()]) each.style.transformOrigin = "";
      carried.classList.remove("k-weather-carried");
      for (const property of ["left", "top", "width", "height", "borderRadius"] as const) carried.style[property] = "";
      carried.dataset.part = b.part;
      carried.toggleAttribute("data-pressable", b.onPress !== undefined);
      if (b.onPress) carried.addEventListener("click", b.onPress);
      b.element.replaceWith(carried);
      sky.content.replaceChildren(b.words);
      leaving.remove();
      coming.remove();
      sky.follow(null);
      sky.landOn([b.words]);
      sky.setStill(b.still);
      b.element = carried;
      b.sky = sky;
      a.sky = null;
      a.element = room;
    },
  };
}
