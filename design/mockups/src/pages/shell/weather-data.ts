// The clock's weather as the shots' stand-ins hold it: Tuesday 27 October 2026, 21:47 in Warsaw,
// home, and eight other cities with the prototype's weathers. The forecast for each city and day is
// worked out from fixed numbers, so every sample draws the same. This file is no page.
//
// The weather's symbols are KubaOS Symbols: the set's sun and moon, and the weather ones the
// mockups draw in the set's way (`src/assets/symbols/`) until the libraries have them.

import type { Readings, Weather } from "../../lib/weather-sky.ts";
import { TYPICAL } from "../../lib/weather-sky.ts";

/** Each weather's name, as a card says it. */
export const CONDITIONS: Record<Weather, string> = {
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

/** A weather's symbol by day and by night: the sky differs for clear, fair and partly cloudy. */
const SYMBOLS: Record<Weather, readonly [string, string]> = {
  clear: ["light_mode", "dark_mode"],
  fair: ["fair_day", "fair_night"],
  partly: ["partly_cloudy_day", "partly_cloudy_night"],
  overcast: ["cloud", "cloud"],
  fog: ["foggy", "foggy"],
  rain: ["rainy", "rainy"],
  thunderstorm: ["thunderstorm", "thunderstorm"],
  snow: ["weather_snowy", "weather_snowy"],
  hail: ["weather_hail", "weather_hail"],
};

/** `weather`'s symbol at `hour` local time: the night's from 19:00 to 06:00. */
export function symbolOf(weather: Weather, hour = 12): string {
  const night = hour < 6 || hour >= 19;
  return SYMBOLS[weather][night ? 1 : 0];
}

/** The tiles' symbols. */
export const TILE_SYMBOLS = { wind: "air", humidity: "humidity", uv: "light_mode", air: "eco", sunrise: "sunrise", sunset: "sunset" } as const;

/** An official warning: what, until when, and its source's words. */
export interface Warning {
  readonly title: string;
  readonly body: string;
}

/** A city whose weather the clock shows. */
export interface City {
  readonly name: string;
  /** Its time zone's IANA name. */
  readonly iana: string;
  /** Hours from home's clock. */
  readonly offset: number;
  readonly weather: Weather;
  readonly temperature: number;
  readonly high: number;
  readonly low: number;
  readonly feels: number;
  /** Its readings where they differ from its weather's typical ones. */
  readonly readings?: Partial<Readings>;
  /** Its official warning now, none, or its country's having no source of them. */
  readonly warning: Warning | "none" | { readonly noSource: string };
  /** Its weather each day from today, seven. */
  readonly week: readonly [Weather, Weather, Weather, Weather, Weather, Weather, Weather];
  readonly sunrise: string;
  readonly sunset: string;
}

/** Home's clock now, in hours. */
export const NOW_HOUR = 21 + 47 / 60;

export const HOME: City = {
  name: "Warsaw",
  iana: "Europe/Warsaw",
  offset: 0,
  weather: "partly",
  temperature: 12,
  high: 14,
  low: 7,
  feels: 10,
  readings: { wind: 5, humidity: 71, uv: 0, air: 32 },
  warning: { title: "Strong wind until 06:00", body: "Gusts to 90 km/h. IMGW, level 2." },
  week: ["partly", "rain", "hail", "overcast", "clear", "fair", "partly"],
  sunrise: "06:24",
  sunset: "16:18",
};

const WEEK_AFTER: readonly Weather[] = ["fair", "partly", "rain", "overcast", "clear", "snow", "thunderstorm", "fog"];

function city(name: string, iana: string, offset: number, weather: Weather, temperature: number, extra: Partial<City> = {}): City {
  const start = WEEK_AFTER.indexOf(weather);
  const week = Array.from({ length: 7 }, (_, i) => (i === 0 ? weather : WEEK_AFTER[(Math.max(0, start) + i * 3) % WEEK_AFTER.length]!)) as unknown as City["week"];
  return { name, iana, offset, weather, temperature, high: temperature + 3, low: temperature - 5, feels: temperature - 2, warning: "none", week, sunrise: "06:40", sunset: "17:10", ...extra };
}

/** The other cities, in the order the clock lists them; eight is the most. */
export const CITIES: readonly City[] = [
  city("Tokyo", "Asia/Tokyo", 8, "fair", 18, { sunrise: "05:58", sunset: "16:52" }),
  city("New York", "America/New_York", -5, "rain", 9, { readings: { rain: 2.5 } }),
  city("London", "Europe/London", -1, "overcast", 11),
  city("Reykjavik", "Atlantic/Reykjavik", -1, "snow", -2, { sunrise: "09:21", sunset: "17:02" }),
  city("Rio de Janeiro", "America/Sao_Paulo", -4, "partly", 27),
  city("San Francisco Bay Area", "America/Los_Angeles", -9, "fog", 13),
  city("Sydney", "Australia/Sydney", 10, "clear", 22),
  city("Cairo", "Africa/Cairo", 1, "clear", 31, { warning: { noSource: "Egypt has no official weather warnings we can read." } }),
];

/** A city's local hour now. */
export function hourOf(place: City): number {
  return (((NOW_HOUR + place.offset) % 24) + 24) % 24;
}

const CLOCK = new Intl.DateTimeFormat("en-GB", { hour: "2-digit", minute: "2-digit", timeZone: "UTC" });

/** `hour` as a clock reads it. */
export function clockOf(hour: number): string {
  return CLOCK.format(new Date(Math.round(hour * 60) * 60_000));
}

const SIGNED = new Intl.NumberFormat("en-GB", { signDisplay: "always" });

/** How far a city's clock is from home's: "+8 h", "−5 h". */
export function offsetOf(place: City): string {
  return `${SIGNED.format(place.offset).replace("-", "−")} h`;
}

/** A temperature as a card says it: "12°", "−2°". */
export function degrees(value: number): string {
  return `${Math.round(value)}°`.replace("-", "−");
}

/** One hour of the forecast. */
export interface Hour {
  readonly label: string;
  readonly symbol: string;
  readonly temperature: string;
  readonly rain: string;
}

/** A small number from a city, a day and a step, the same every time. */
function wobble(name: string, day: number, step: number): number {
  let h = day * 131 + step * 17;
  for (const c of name) h = (h * 31 + c.charCodeAt(0)) % 9973;
  return (h % 100) / 100;
}

const RAINY: Partial<Record<Weather, number>> = { rain: 0.7, thunderstorm: 0.8, hail: 0.6, snow: 0.6, overcast: 0.25, fog: 0.15, partly: 0.1 };

/** A day's high and low: today's the city's own, the others' near them. */
export function dayRange(place: City, day: number): readonly [number, number] {
  if (day === 0) return [place.high, place.low];
  const shift = Math.round((wobble(place.name, day, 0) - 0.5) * 6);
  return [place.high + shift, place.low + shift];
}

/** A day's twelve hours: today's from now, another day's from 08:00. */
export function hoursOf(place: City, day: number): readonly Hour[] {
  const weather = place.week[day]!;
  const [high, low] = dayRange(place, day);
  const first = day === 0 ? Math.floor(hourOf(place)) : 8;
  return Array.from({ length: 12 }, (_, i) => {
    const hour = (first + i) % 24;
    const warmth = (Math.sin(((hour - 9) / 24) * Math.PI * 2) + 1) / 2;
    const temperature = day === 0 && i === 0 ? place.temperature : low + (high - low) * warmth;
    const chance = Math.round(Math.min(0.95, (RAINY[weather] ?? 0) * (0.6 + wobble(place.name, day, i) * 0.6)) * 20) * 5;
    return { label: day === 0 && i === 0 ? "Now" : String(hour).padStart(2, "0"), symbol: symbolOf(weather, hour), temperature: degrees(temperature), rain: `${chance}%` };
  });
}

const WEEKDAY = new Intl.DateTimeFormat("en-GB", { weekday: "short", timeZone: "UTC" });

/** The name of the day `day` days from today: "Today", then "Wed". */
export function dayName(day: number): string {
  return day === 0 ? "Today" : WEEKDAY.format(Date.UTC(2026, 9, 27 + day));
}

/** A city's readings on a day: its weather's typical ones, today's its own. */
export function readingsOf(place: City, day: number): Readings {
  const weather = place.week[day]!;
  return { ...TYPICAL[weather].readings, ...(day === 0 ? place.readings : {}), feels: day === 0 ? place.feels : dayRange(place, day)[0] - 2 };
}

const COMPASS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

/** The six tiles' values and names for a city on a day. */
export function tilesOf(place: City, day: number): readonly { readonly symbol: string; readonly value: string; readonly name: string }[] {
  const r = readingsOf(place, day);
  const uv = r.uv <= 2 ? "low" : r.uv <= 5 ? "moderate" : r.uv <= 7 ? "high" : "very high";
  const air = r.air <= 50 ? "good" : r.air <= 100 ? "fair" : "poor";
  const from = COMPASS[Math.floor(wobble(place.name, day, 99) * 8)]!;
  return [
    { symbol: TILE_SYMBOLS.wind, value: `${Math.round(r.wind * 3.6)} km/h`, name: `Wind, ${from}` },
    { symbol: TILE_SYMBOLS.humidity, value: `${Math.round(r.humidity)}%`, name: "Humidity" },
    { symbol: TILE_SYMBOLS.uv, value: String(r.uv), name: `UV, ${uv}` },
    { symbol: TILE_SYMBOLS.air, value: String(r.air), name: `Air, ${air}` },
    { symbol: TILE_SYMBOLS.sunrise, value: place.sunrise, name: "Sunrise" },
    { symbol: TILE_SYMBOLS.sunset, value: place.sunset, name: "Sunset" },
  ];
}

/** When the forecast was last read. */
export const UPDATED = "21:40";
