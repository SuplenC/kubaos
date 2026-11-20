// The WeatherSky, a DRAFT part: a live sky for an hour and a weather, drawn behind a weather card's
// text, with the rain, snow or hail the forecast gives falling across the card and landing on the
// tops of its figures and words. The sky is our shader (`weather-sky-gl.ts`); what falls is
// `atmos-fx` (MIT), loaded only once something falls.
//
// The hour and every reading change what is drawn (`target`), and the sky eases every value toward
// its new target, so a change of weather is a blend. How much falls is the forecast's own amount,
// on curves Kuba approved by eye: millimetres of rain, centimetres of snow, hail slight or heavy.
//
// A live sky draws only while it is on screen and the page is shown. Still (a small city card),
// or with reduced motion, it is one frame: nothing falls, no lightning, nothing moves.

import "./weather-sky.css";
import type { AtmosphereController, AtmosphereOptions } from "atmos-fx";
import { probe, variant } from "./probe.ts";
import { holdPainter, type PainterHold, type SkyFrame } from "./weather-sky-gl.ts";
import { DAY, type Readings, type SkyOf, TYPICAL, type Weather } from "./weather-sky-data.ts";
import { onChange, settings } from "../site/settings.ts";

export { type Hail, type Readings, TYPICAL, type Weather, WEATHERS } from "./weather-sky-data.ts";

export interface WeatherSkyOptions {
  /** The hour, 0 to 24, fractions for minutes. */
  readonly hour: number;
  readonly weather: Weather;
  /** The readings the forecast gives; each one left out is the weather's typical value. */
  readonly readings?: Partial<Readings>;
  /** One frame: nothing falls and nothing moves. The small city cards. */
  readonly still?: boolean;
  /** The elements whose text what falls lands on: one box per figure or word, fitted to its ink. */
  readonly landOn?: readonly Element[];
  /**
   * Where its clouds and stars stand at the start, in seconds of its clock: skies side by side take
   * different seeds, so no two are the same picture. 0 unless given.
   */
  readonly seed?: number;
  /** Reduced motion whatever the site's setting: for a sample of it. The site's setting unless given. */
  readonly reduced?: boolean;
}

/** A WeatherSky on the page, and how its owner changes it. */
export interface WeatherSky {
  /** Its box, which its owner sizes and rounds; it fills it. */
  readonly element: HTMLElement;
  /**
   * The layer its owner lays its text in, over its box: what falls behind the text is under it,
   * what falls in front is over it.
   */
  readonly content: HTMLElement;
  /** Moves to `hour`; `jump` lands there at once, else it blends as a weather does. */
  setHour(hour: number, jump?: boolean): void;
  /** Blends to `weather`, with its typical readings but those given. */
  setWeather(weather: Weather, readings?: Partial<Readings>): void;
  /** Blends to these readings, the others kept. */
  setReadings(readings: Partial<Readings>): void;
  /** What falls lands on these elements' text from now on. */
  landOn(elements: readonly Element[]): void;
  /** Holds it still where it is, until `resume`: a popup shut. */
  pause(): void;
  resume(): void;
  /** Stops it, lets its share of the sky's context and the precipitation's layers go, and removes nothing else. */
  dispose(): void;
}

type Vec3 = readonly [number, number, number];

/** Every value the sky eases toward: what the shader draws, and how the clouds and lightning run. */
interface Target extends SkyFrame {
  readonly thunder: number;
  /** How fast the clouds drift, in the shader's units a second. */
  readonly drift: number;
}

const mix = (a: number, b: number, t: number) => a + (b - a) * t;
const mix3 = (a: Vec3, b: Vec3, t: number): Vec3 => [mix(a[0], b[0], t), mix(a[1], b[1], t), mix(a[2], b[2], t)];
const unit = (v: number) => Math.max(0, Math.min(1, v));

/** What the sky draws at `hour` under `sky`, with `readings`. */
function target(hourOf: number, sky: SkyOf, readings: Readings): Target {
  const hour = ((hourOf % 24) + 24) % 24;
  let next = DAY.findIndex((key) => key.hour > hour);
  if (next < 1) next = DAY.length - 1;
  const a = DAY[next - 1]!;
  const b = DAY[next]!;
  const t = (hour - a.hour) / (b.hour - a.hour);
  const g = sky.gloom;
  const grey = (c: Vec3): Vec3 => {
    const l = (c[0] + c[1] + c[2]) / 3;
    const greyed = mix3(c, [l * 0.92, l * 0.97, l * 1.05], g * 0.85);
    const dim = mix(1, 0.55, g);
    return [greyed[0] * dim, greyed[1] * dim, greyed[2] * dim];
  };
  const scaled = (c: Vec3, by: number): Vec3 => [c[0] * by, c[1] * by, c[2] * by];
  // How the air feels tints everything: cold is crisper and bluer, heat is warmer and softer.
  const heat = Math.max(-1, Math.min(1, (readings.feels - 10) / 22));
  const feel = (c: Vec3): Vec3 => [c[0] * (1 + 0.1 * heat), c[1], c[2] * (1 - 0.12 * heat)];
  const stars = mix(a.stars, b.stars, t);
  const day = (hour - 6) / 13;
  const up = Math.sin(Math.PI * day);
  const night = ((hour + 5) % 24) / 11;
  const humid = unit((readings.humidity - 30) / 70);
  const murk = unit(1 - Math.log(Math.max(readings.visibility, 0.5)) / Math.log(40));
  return {
    top: feel(grey(mix3(a.top, b.top, t))),
    bottom: feel(grey(mix3(a.horizon, b.horizon, t))),
    lit: scaled(grey(mix3(a.cloudLit, b.cloudLit, t)), mix(1, 0.8, g)),
    shade: scaled(grey(mix3(a.cloudShade, b.cloudShade, t)), mix(1, 0.7, g)),
    sun: mix3(a.sun, b.sun, t),
    sunAt: [mix(-1.4, 1.4, day), mix(-0.75, 0.32, Math.max(0, up))],
    sunUp: unit(up * 4 + 0.6) * (1 - g * 0.9) * (day > -0.05 && day < 1.05 ? 1 : 0),
    moonAt: [mix(-0.4, 1.9, night), mix(-0.6, 0.3, Math.max(0, Math.sin(Math.PI * night)))],
    moonUp: stars * (1 - sky.cover * 0.55) * (1 - g),
    stars: stars * (1 - g) * (1 - murk * 0.8),
    cover: sky.cover,
    fog: sky.fog,
    haze: murk * 0.75 + humid * 0.1 + Math.max(0, heat) * 0.08,
    glare: unit(readings.uv / 11),
    halo: humid,
    smog: unit((readings.air - 40) / 110),
    thunder: sky.thunder,
    drift: 0.3 + readings.wind / 3.5,
  };
}

/** `now` a share `k` of the way to `goal`, every value. */
function eased(now: Target, goal: Target, k: number): Target {
  const out: Record<string, unknown> = {};
  for (const key of Object.keys(goal) as (keyof Target)[]) {
    const from = now[key];
    const to = goal[key];
    out[key] = Array.isArray(to) ? (to as readonly number[]).map((v, i) => mix((from as readonly number[])[i]!, v, k)) : mix(from as number, to as number, k);
  }
  // Every key of `Target` was copied from `goal` and eased, so the record is a whole target.
  return out as unknown as Target;
}

// How much falls is the forecast's amount, on the library's own options. Its particle count is its
// rate times the card's area times `density`, and density stops at 1, so each kind is a curve from
// the amount to density, with speed (and with it the rain's streak length) rising alongside:
//   rain, mm/h over 0-30: density (mm/30)^0.6, speed 0.8 + 0.5 (mm/30)^0.5. 0.2 is a few drops, 2.5
//     light, 7.5 moderate, 15 heavy, 30 a downpour; a thunderstorm is just more millimetres.
//   snow, cm/h over 0-5: density (cm/5)^0.374, speed 0.36 + 0.14 (cm/5)^0.5. Snow starts higher than
//     rain: its flakes are few and small, so 0.2 cm/h is already about a third of the way.
//   hail, which the forecast gives no amount: slight 0.55 at its own speed 0.92, heavy 1 at 1.1.
type Falling = "rain" | "snow" | "hail";
const CURVE: Record<Falling, (r: Readings) => readonly [number, number]> = {
  rain: (r) => {
    const x = Math.min(1, r.rain / 30);
    return [x ** 0.6, 0.8 + 0.5 * x ** 0.5];
  },
  snow: (r) => {
    const x = Math.min(1, r.snow / 5);
    return [x ** 0.374, 0.36 + 0.14 * x ** 0.5];
  },
  hail: (r) => (r.hail === "heavy" ? [1, 1.1] : [0.55, 0.92]),
};

// Those densities are the home card's, the smallest live one. A bigger card shows more of the same
// weather by area alone, so it gets less density: (home card's area / this card's area)^0.22.
const HOME_AREA = 370 * 112;

/** The library's density and speed move in steps of 0.02: each change of its particle count rebuilds its particles. */
const stepped = (v: number) => Math.round(v * 50) / 50;

// The library's wind is a sideways push. Its own rain uses about -0.12, which barely leans a streak;
// a clear slant needs about -2, so 1 m/s is -0.1 here: a breeze leans the rain a little, a gale a lot.
const windOf = (r: Readings) => -Math.round(Math.min(25, r.wind) * 2) / 20;

/**
 * What falls: hail first, as the forecast's code puts it; then snow or rain, whichever brings more
 * water (a centimetre of snow is about 0.7 mm of it). With nothing falling in a wet sky the
 * atmosphere stays, empty; in a dry sky it goes.
 */
function falling(sky: SkyOf, r: Readings): Falling | null {
  if (r.hail !== "none") return "hail";
  if (r.snow > 0 && r.snow * 0.7 >= r.rain) return "snow";
  if (r.rain > 0) return "rain";
  if (sky.snow > 0) return "snow";
  if (sky.rain > 0 || sky.thunder > 0) return "rain";
  return null;
}

let library: Promise<typeof import("atmos-fx")> | null = null;

/** The precipitation's library, loaded the first time anything falls. */
function atmosFx(): Promise<typeof import("atmos-fx")> {
  library ??= import("atmos-fx");
  return library;
}

/** Every live sky that wants frames, drawn from one frame callback. */
const running = new Set<Sky>();
let frameId = 0;

function tick(now: number): void {
  frameId = 0;
  for (const sky of running) sky.frame(now);
  if (running.size > 0) frameId = requestAnimationFrame(tick);
}

function wake(): void {
  if (!frameId && running.size > 0) frameId = requestAnimationFrame(tick);
}

const ink = document.createElement("canvas").getContext("2d");

/** The scale `part` is drawn at: the page's, or that of a box drawn at a scale of its own. */
function scaleOf(part: HTMLElement): number {
  return Number.parseFloat(getComputedStyle(part).getPropertyValue("--scale")) || settings().scale;
}

class Sky implements WeatherSky {
  readonly element = document.createElement("div");
  readonly content = document.createElement("div");
  private readonly canvas = document.createElement("canvas");
  private readonly hits = document.createElement("div");
  private readonly painter: PainterHold = holdPainter();
  private readonly into: CanvasRenderingContext2D | null;
  private readonly still: boolean;
  private readonly forcedReduced: boolean | undefined;
  private hour: number;
  private weather: Weather;
  private readings: Readings;
  private goal: Target;
  private now: Target;
  private time: number;
  private drift = 0;
  private flash = 0;
  private echo = 0;
  private nextFlash = 0;
  private last = -1;
  private seen = false;
  private paused = false;
  private disposed = false;
  private targets: readonly Element[] = [];
  private atmos: AtmosphereController | null = null;
  private atmosKey = "";
  private loading = 0;
  private refit = 0;
  private readonly observers: { disconnect(): void }[] = [];
  private readonly unlisten: () => void;

  constructor(options: WeatherSkyOptions) {
    this.still = options.still ?? false;
    this.forcedReduced = options.reduced;
    this.hour = options.hour;
    this.weather = options.weather;
    this.readings = { ...TYPICAL[options.weather].readings, ...options.readings };
    this.goal = target(this.hour, TYPICAL[this.weather].sky, this.readings);
    this.now = this.goal;
    this.time = options.seed ?? 0;
    this.element.className = "k-weather-sky";
    this.canvas.className = "k-weather-sky-picture";
    this.content.className = "k-weather-sky-content";
    this.hits.className = "k-weather-sky-hits";
    this.element.append(this.canvas, this.content, this.hits);
    this.into = this.canvas.getContext("2d", { alpha: false });
    probe(this.element, { role: "component", name: "WeatherSky", variant: variant(this.still && "still"), fixed: { width: false, height: false } });

    const sized = new ResizeObserver(() => {
      this.size();
      this.fitLater();
      this.fall();
    });
    sized.observe(this.element);
    const seen = new IntersectionObserver((entries) => {
      this.seen = entries.some((entry) => entry.isIntersecting);
      this.run();
    });
    seen.observe(this.element);
    const shown = () => this.run();
    document.addEventListener("visibilitychange", shown);
    this.observers.push(sized, seen, { disconnect: () => document.removeEventListener("visibilitychange", shown) });
    this.unlisten = onChange((_, changed) => {
      if (changed.has("reduced")) this.run();
    });
    if (options.landOn) this.landOn(options.landOn);
  }

  private get reduced(): boolean {
    return this.forcedReduced ?? settings().reduced;
  }

  /** Whether it is on the page, on screen, not paused, and the page is shown. */
  private get drawing(): boolean {
    return this.element.isConnected && this.seen && !this.paused && document.visibilityState === "visible";
  }

  /** Whether it moves: a live sky, motion not reduced. */
  private get moving(): boolean {
    return !this.still && !this.reduced;
  }

  setHour(hour: number, jump = false): void {
    this.hour = hour;
    this.aim(jump);
  }

  setWeather(weather: Weather, readings: Partial<Readings> = {}): void {
    this.weather = weather;
    this.readings = { ...TYPICAL[weather].readings, ...readings };
    this.aim(false);
  }

  setReadings(readings: Partial<Readings>): void {
    this.readings = { ...this.readings, ...readings };
    this.aim(false);
  }

  landOn(elements: readonly Element[]): void {
    this.targets = elements;
    const text = new MutationObserver(() => this.fitLater());
    const boxes = new ResizeObserver(() => this.fitLater());
    for (const element of elements) {
      text.observe(element, { characterData: true, childList: true, subtree: true });
      boxes.observe(element);
    }
    this.observers.push(text, boxes);
    void document.fonts.ready.then(() => this.fitLater());
    this.fitLater();
  }

  pause(): void {
    this.paused = true;
    this.run();
  }

  resume(): void {
    this.paused = false;
    this.run();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    running.delete(this);
    cancelAnimationFrame(this.refit);
    for (const observer of this.observers) observer.disconnect();
    this.unlisten();
    this.atmos?.destroy();
    this.atmos = null;
    this.loading++;
    this.painter.release();
  }

  /** A new target from the hour, the weather and the readings; a still sky, or `jump`, lands on it. */
  private aim(jump: boolean): void {
    this.goal = target(this.hour, TYPICAL[this.weather].sky, this.readings);
    if (jump || !this.moving) {
      this.now = this.goal;
      this.draw();
    }
    this.fall();
  }

  /** Starts or stops its frames and what falls, as its state asks. */
  private run(): void {
    if (this.disposed) return;
    const live = this.moving && this.drawing;
    if (live && !running.has(this)) {
      this.last = -1;
      running.add(this);
      wake();
    } else if (!live && running.delete(this)) {
      this.last = -1;
    }
    if (!this.moving) {
      this.now = this.goal;
      this.flash = 0;
      this.draw();
    }
    this.fall();
  }

  /** One frame of a live sky: every value eased toward its target, the lightning, then drawn. */
  frame(at: number): void {
    if (!this.element.isConnected) {
      this.run();
      return;
    }
    const dt = this.last < 0 ? 0 : (Math.min(0.1, (at - this.last) / 1000) * settings().speed);
    this.last = at;
    this.time += dt;
    this.now = eased(this.now, this.goal, 1 - Math.exp(-dt * 2.2));
    this.drift += dt * this.now.drift;
    // Lightning: a flash at a random moment, a second flicker after, both dying fast.
    if (this.now.thunder > 0.3 && this.time > this.nextFlash) {
      this.flash = 1;
      this.echo = this.time + 0.12;
      this.nextFlash = this.time + 2.5 + Math.random() * 6;
    }
    if (this.echo && this.time > this.echo) {
      this.flash = Math.max(this.flash, 0.7);
      this.echo = 0;
    }
    this.flash *= Math.exp(-dt * 9);
    this.draw();
  }

  private draw(): void {
    if (this.disposed || !this.into || this.canvas.width < 2) return;
    this.painter.paint(this.into, this.now, this.time, this.drift, this.flash * this.now.thunder, this.canvas.width, this.canvas.height);
  }

  /** Its canvas in device pixels, at most two to a CSS pixel, drawn again at once. */
  private size(): void {
    const box = this.element.getBoundingClientRect();
    const ratio = Math.min(devicePixelRatio, 2);
    const width = Math.max(1, Math.round(box.width * ratio));
    const height = Math.max(1, Math.round(box.height * ratio));
    if (width === this.canvas.width && height === this.canvas.height) return;
    this.canvas.width = width;
    this.canvas.height = height;
    this.draw();
  }

  /** Sets what falls, creating, changing or taking down the library's atmosphere. */
  private fall(): void {
    if (this.disposed) return;
    const kind = falling(TYPICAL[this.weather].sky, this.readings);
    // Taken off the page, it lets the library go: put back, it is seen again and makes another.
    const wanted = kind && this.moving && this.element.isConnected;
    if (!wanted) {
      this.atmos?.destroy();
      this.atmos = null;
      this.atmosKey = "";
      this.loading++;
      return;
    }
    const [density, speed] = CURVE[kind](this.readings);
    const options: AtmosphereOptions = { preset: kind, density: stepped(Math.min(1, density * this.cardFactor())), speed: stepped(speed), wind: windOf(this.readings) };
    const live = this.drawing;
    const key = JSON.stringify(options);
    if (this.atmos) {
      if (key !== this.atmosKey) this.atmos.update(options);
      this.atmosKey = key;
      if (live) this.atmos.resume();
      else this.atmos.pause();
      return;
    }
    // An atmosphere is made only for a sky that is drawing: one scrolled past waits until it is seen.
    if (!live) return;
    const ticket = ++this.loading;
    void atmosFx().then(({ createAtmosphere }) => {
      if (ticket !== this.loading || this.disposed || this.atmos) return;
      this.atmos = createAtmosphere(this.element, options);
      this.atmosKey = key;
      this.atmos.start();
      this.fall();
    });
  }

  /** The density factor for this card's area, in logical pixels. */
  private cardFactor(): number {
    const scale = scaleOf(this.element);
    const area = (this.element.offsetWidth / scale) * (this.element.offsetHeight / scale);
    return Math.min(1, (HOME_AREA / Math.max(1, area)) ** 0.22);
  }

  private fitLater(): void {
    if (this.refit || this.disposed) return;
    this.refit = requestAnimationFrame(() => {
      this.refit = 0;
      this.fit();
    });
  }

  /**
   * One invisible box per big figure, and per smaller word, sized to its ink, for what falls to
   * land on. The library collides with an element's whole box, and a text box reaches well above
   * the ink: so each box takes its left edge and line from the layout (a Range over the
   * characters) and its ink's extent from the font's own metrics. The boxes sit side by side,
   * never nested, and none is wider than one word.
   */
  private fit(): void {
    this.hits.replaceChildren();
    if (!ink || this.still) return;
    const base = this.element.getBoundingClientRect();
    // The page's box is zoomed to the display's ratio: rectangles are in the viewport's pixels,
    // the boxes in the sky's own.
    const zoom = base.width / Math.max(1, this.element.offsetWidth) || 1;
    const scale = scaleOf(this.element);
    for (const element of this.targets) {
      const walk = document.createTreeWalker(element, NodeFilter.SHOW_TEXT);
      for (let node = walk.nextNode() as Text | null; node; node = walk.nextNode() as Text | null) {
        const holder = node.parentElement;
        if (!holder) continue;
        const style = getComputedStyle(holder);
        ink.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
        const perGlyph = Number.parseFloat(style.fontSize) / scale >= 32;
        const parts: [number, string][] = perGlyph ? [...node.data].map((c, i) => [i, c]) : [...node.data.matchAll(/\S+/g)].map((m) => [m.index, m[0]]);
        for (const [at, words] of parts) {
          if (!words.trim()) continue;
          const range = document.createRange();
          range.setStart(node, at);
          range.setEnd(node, at + words.length);
          const r = range.getBoundingClientRect();
          if (r.width === 0) continue;
          const m = ink.measureText(words);
          const baseline = (r.top - base.top) / zoom + m.fontBoundingBoxAscent;
          const box = document.createElement("div");
          box.className = "k-weather-sky-hit";
          box.setAttribute("data-atmos-collision", "");
          box.setAttribute("data-atmos-solid", "");
          box.setAttribute("data-atmos-liquid-dripping", "false");
          box.style.left = `${(r.left - base.left) / zoom - m.actualBoundingBoxLeft}px`;
          box.style.top = `${baseline - m.actualBoundingBoxAscent}px`;
          box.style.width = `${m.actualBoundingBoxLeft + m.actualBoundingBoxRight}px`;
          box.style.height = `${m.actualBoundingBoxAscent + m.actualBoundingBoxDescent}px`;
          this.hits.append(box);
        }
      }
    }
  }
}

/** A WeatherSky for `options`; its owner sizes `element`, lays its text in `content`, and disposes it. */
export function weatherSky(options: WeatherSkyOptions): WeatherSky {
  return new Sky(options);
}
