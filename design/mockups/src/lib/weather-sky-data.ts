// The WeatherSky's picture data: the sky's colours through the day and what each weather does to
// the sky, with the readings it usually comes with. These colours are a painting's, not the
// interface's: they are never a colour role, they never reach text or a control, and no token is
// routed through them. They are the approved prototype's numbers, kept as they were tuned.

/** The nine weathers a forecast's code is sorted into. */
export const WEATHERS = ["clear", "fair", "partly", "overcast", "fog", "rain", "thunderstorm", "snow", "hail"] as const;
export type Weather = (typeof WEATHERS)[number];

/** Hail: the forecast gives it no amount, only whether it is slight or heavy. */
export type Hail = "none" | "slight" | "heavy";

/** A weather's readings, each as the forecast gives it. */
export interface Readings {
  /** Wind, m/s. */
  readonly wind: number;
  /** Visibility, km. */
  readonly visibility: number;
  /** The UV index, 0 to 11. */
  readonly uv: number;
  /** Relative humidity, %. */
  readonly humidity: number;
  /** The air quality index. */
  readonly air: number;
  /** Feels like, °C. */
  readonly feels: number;
  /** Rain, mm/h. */
  readonly rain: number;
  /** Snowfall, cm/h. */
  readonly snow: number;
  readonly hail: Hail;
}

/**
 * What a weather does to the sky, 0 to 1 each: its cloud cover, how grey and dark it turns, its
 * fog, and whether it is a wet sky of rain or snow and a storm's.
 */
export interface SkyOf {
  readonly cover: number;
  readonly gloom: number;
  readonly fog: number;
  readonly rain: number;
  readonly snow: number;
  readonly thunder: number;
}

/** Each weather's sky, and the readings it usually comes with. */
export const TYPICAL: Readonly<Record<Weather, { readonly sky: SkyOf; readonly readings: Readings }>> = {
  clear: { sky: { cover: 0, gloom: 0, fog: 0, rain: 0, snow: 0, thunder: 0 }, readings: { wind: 2, visibility: 40, uv: 6, humidity: 40, air: 20, feels: 18, rain: 0, snow: 0, hail: "none" } },
  fair: { sky: { cover: 0.3, gloom: 0, fog: 0, rain: 0, snow: 0, thunder: 0 }, readings: { wind: 3, visibility: 35, uv: 5, humidity: 45, air: 20, feels: 16, rain: 0, snow: 0, hail: "none" } },
  partly: { sky: { cover: 0.52, gloom: 0.1, fog: 0, rain: 0, snow: 0, thunder: 0 }, readings: { wind: 4, visibility: 25, uv: 4, humidity: 55, air: 25, feels: 12, rain: 0, snow: 0, hail: "none" } },
  overcast: { sky: { cover: 1, gloom: 0.65, fog: 0.1, rain: 0, snow: 0, thunder: 0 }, readings: { wind: 5, visibility: 15, uv: 2, humidity: 70, air: 30, feels: 10, rain: 0, snow: 0, hail: "none" } },
  fog: { sky: { cover: 0.5, gloom: 0.5, fog: 0.9, rain: 0, snow: 0, thunder: 0 }, readings: { wind: 1, visibility: 0.6, uv: 1, humidity: 97, air: 40, feels: 6, rain: 0, snow: 0, hail: "none" } },
  rain: { sky: { cover: 1, gloom: 0.8, fog: 0.15, rain: 0.7, snow: 0, thunder: 0 }, readings: { wind: 7, visibility: 8, uv: 1, humidity: 90, air: 20, feels: 9, rain: 2.5, snow: 0, hail: "none" } },
  thunderstorm: { sky: { cover: 1, gloom: 1, fog: 0.2, rain: 1, snow: 0, thunder: 1 }, readings: { wind: 12, visibility: 5, uv: 1, humidity: 95, air: 20, feels: 14, rain: 15, snow: 0, hail: "none" } },
  snow: { sky: { cover: 0.95, gloom: 0.5, fog: 0.3, rain: 0, snow: 1, thunder: 0 }, readings: { wind: 3, visibility: 4, uv: 1, humidity: 85, air: 20, feels: -6, rain: 0, snow: 1, hail: "none" } },
  hail: { sky: { cover: 1, gloom: 0.85, fog: 0.15, rain: 0.8, snow: 0, thunder: 0 }, readings: { wind: 11, visibility: 3, uv: 1, humidity: 88, air: 20, feels: 2, rain: 4, snow: 0, hail: "slight" } },
};

/** A colour in linear light, 0 to 1 a channel. */
export type Linear = readonly [number, number, number];

/** One key of the day: the hour, and the sky's colours and its stars' strength then. */
export interface DayKey {
  readonly hour: number;
  readonly top: Linear;
  readonly horizon: Linear;
  readonly cloudLit: Linear;
  readonly cloudShade: Linear;
  readonly sun: Linear;
  readonly stars: number;
}

/** A painted sRGB colour in linear light, as the shader mixes them. */
function linear(hex: string): Linear {
  const channel = (at: number) => (Number.parseInt(hex.slice(at, at + 2), 16) / 255) ** 2.2;
  return [channel(1), channel(3), channel(5)];
}

/** The sky's colours through the day: hour, top, horizon, cloud lit, cloud shaded, sun, stars. */
const KEYS: readonly (readonly [number, string, string, string, string, string, number])[] = [
  [0, "#05070f", "#0d1424", "#2a3147", "#0b0e18", "#000000", 1],
  [4.5, "#070a16", "#141c33", "#2f3650", "#0d101c", "#000000", 1],
  [6, "#27325c", "#e08a5c", "#f2b08c", "#4a4563", "#ff9a55", 0.15],
  [7.5, "#3f78c4", "#b9d2ea", "#ffffff", "#8a98b0", "#ffe2b8", 0],
  [12, "#2f74d0", "#a8cdf0", "#ffffff", "#93a3bb", "#fff1cf", 0],
  [16.5, "#3a76c8", "#b6cfe8", "#fff8ee", "#8c97ad", "#ffe6b8", 0],
  [18.5, "#3a3f78", "#f08a4e", "#ffb27a", "#4d4263", "#ff7a3c", 0.1],
  [19.6, "#151b3d", "#6a4a6e", "#6f5a7c", "#1c1a30", "#c0503a", 0.6],
  [21, "#070a16", "#131b31", "#2f3650", "#0d101c", "#000000", 1],
  [24, "#05070f", "#0d1424", "#2a3147", "#0b0e18", "#000000", 1],
];

/** The day's keys, in hour order from 0 to 24. */
export const DAY: readonly DayKey[] = KEYS.map(([hour, top, horizon, cloudLit, cloudShade, sun, stars]) => ({
  hour,
  top: linear(top),
  horizon: linear(horizon),
  cloudLit: linear(cloudLit),
  cloudShade: linear(cloudShade),
  sun: linear(sun),
  stars,
}));
