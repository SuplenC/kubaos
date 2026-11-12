// Readings at a glance, as the system monitor shows them: `kubaos-ui`'s `meter::meter` and
// `meter::sparkline`, `KubaOS.Components`' `Meter` and `Sparkline` (`docs/design/components/meter.md`,
// `sparkline.md`). `feedback.css` draws them.
//
// A meter is a reading in parts on a progress bar's track: `size.progress` tall, a pill, each part
// a share and its colour, left to right, every part from the left edge to where it ends so each
// keeps the bar's rounded ends. A new reading eases each part over `short` on `linear`.
//
// A live meter is a mic's level as it comes, one `primary` part with no easing, and a peak tick: a
// `stroke.thin` line across the bar at the highest level lately, which holds for `slow` and then
// falls back at the whole bar per `scene`, never under the level. Muted, it is the track alone.
//
// A sparkline is the last minute of a reading as a line in the thin stroke, in whole device pixels,
// filled under it with the line's colour at `opacity.tint`, each sample placed by its time with the
// newest at the right edge; a second reading over it, unfilled, in `tertiary`. It draws when a
// sample comes, never per frame.

import "./feedback.css";
import { probe } from "./probe.ts";
import { length, raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** A reading's colour step against its threshold: fine, close to it, over it. */
export type Level = 0 | 1 | 2;

/**
 * The step of `value` against `threshold`, close within `margin` of it, as the system monitor works
 * it out (`config/quickshell/services/sysmon.js`).
 */
export function level(value: number, threshold: number, margin: number): Level {
  return value > threshold ? 2 : value > threshold - margin ? 1 : 0;
}

/** A reading's colour role at `step`, or `fine` while it is fine. */
export function tint(step: Level, fine: string): string {
  return step === 2 ? "error" : step === 1 ? "warning" : fine;
}

export interface MeterPart {
  /** Its share of the bar, 0 to 1. */
  readonly share: number;
  /** Its colour role. */
  readonly color: string;
}

/** Sets `bar`'s parts' shares, each easing to its new end. */
export function setMeter(bar: HTMLElement, parts: readonly MeterPart[]): void {
  const fills = [...bar.querySelectorAll<HTMLElement>(".k-meter-part")].reverse();
  let end = 0;
  parts.forEach((part, index) => {
    end = Math.min(1, end + Math.max(0, part.share));
    const fill = fills[index];
    if (!fill) return;
    fill.style.setProperty("--reach", String(end));
    fill.style.background = `var(--color-${part.color})`;
  });
}

export function meter(parts: readonly MeterPart[]): HTMLElement {
  const bar = document.createElement("span");
  bar.className = "k-meter";
  bar.setAttribute("role", "meter");
  // The last part first, so each one's rounded end shows over the one after it.
  for (let index = 0; index < parts.length; index++) {
    const fill = document.createElement("span");
    fill.className = "k-meter-part";
    bar.append(fill);
  }
  setMeter(bar, parts);
  return probe(bar, { role: "component", name: "Meter", variant: "", fixed: { width: false, height: true } });
}

export interface LiveMeter {
  readonly bar: HTMLElement;
  /** A level, 0 to 1, read at `now` in milliseconds: the part jumps to it, the peak holds or falls. */
  readonly feed: (level: number, now: number) => void;
  /** Muted, it is empty and takes no level; unmuted, it starts again from nothing. */
  readonly mute: (muted: boolean) => void;
}

/** The duration token `name` in milliseconds under the speed setting. */
function duration(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

export function liveMeter(level = 0, peak = level): LiveMeter {
  const bar = meter([{ share: level, color: "primary" }]);
  bar.dataset.live = "";
  const part = bar.querySelector<HTMLElement>(".k-meter-part")!;
  const tick = document.createElement("span");
  tick.className = "k-meter-peak";
  bar.append(tick);
  let muted = false;
  // The peak last caught and when: it holds there, then falls from it.
  let top = Math.max(level, peak);
  let since = Number.NEGATIVE_INFINITY;

  /** Where the tick is at `now`: the peak held, or falling from it, never under `under`. */
  function peakAt(now: number, under: number): number {
    const fallen = Math.max(0, now - since - duration("slow")) / duration("scene");
    return Math.max(under, top - fallen);
  }

  function draw(reach: number, at: number): void {
    part.style.setProperty("--reach", String(reach));
    bar.style.setProperty("--meter-peak", String(at));
    bar.toggleAttribute("data-peaked", at > 0);
  }

  draw(level, top);
  return {
    bar,
    feed(next, now) {
      if (muted) return;
      const reach = Math.max(0, Math.min(1, next));
      if (reach >= peakAt(now, 0)) {
        top = reach;
        since = now;
      }
      draw(reach, peakAt(now, reach));
    },
    mute(on) {
      muted = on;
      top = 0;
      since = Number.NEGATIVE_INFINITY;
      draw(0, 0);
    },
  };
}

/** A sample: its time in milliseconds and its value. */
export type Sample = readonly [time: number, value: number];

export interface SparklineOptions {
  readonly samples: readonly Sample[];
  /** A second reading over the first, unfilled, in `tertiary`: upload over download. */
  readonly second?: readonly Sample[];
  /** The top of the scale, or 0 to fit the highest sample with room above it. */
  readonly max?: number;
  /** The line's colour role. */
  readonly color?: string;
  /** Inline in the bar, a line of small text tall, in place of `size.sparkline`. */
  readonly inline?: boolean;
}

/** The top of a sparkline's scale unless set: a percentage's. */
export const DEFAULT_MAX = 100;
/** How far above the highest sample a fitted scale's top is. */
const FIT_HEADROOM = 1.15;
/** The time across a sparkline: the last minute. */
export const SPAN = 60_000;
/** A second in milliseconds: the system monitor samples once a second while its panel is open. */
export const SECOND = 1000;

const SVG = "http://www.w3.org/2000/svg";

export interface Sparkline {
  readonly box: HTMLElement;
  /** Draws it again with `samples` (and `second`): a sample came. */
  readonly update: (samples: readonly Sample[], second?: readonly Sample[]) => void;
}

/** The newest sample's time in `series`, or 0. */
function newest(series: readonly Sample[]): number {
  return series.at(-1)?.[0] ?? 0;
}

export function sparkline(options: SparklineOptions): Sparkline {
  const box = document.createElement("span");
  box.className = "k-sparkline";
  box.toggleAttribute("data-inline", options.inline ?? false);
  box.style.setProperty("--line-colour", `var(--color-${options.color ?? "primary"})`);
  const drawing = document.createElementNS(SVG, "svg");
  const [area, line, second] = ["k-sparkline-area", "k-sparkline-line", "k-sparkline-second"].map((name) => {
    const path = document.createElementNS(SVG, "path");
    path.setAttribute("class", name);
    drawing.append(path);
    return path;
  }) as [SVGPathElement, SVGPathElement, SVGPathElement];
  box.append(drawing);
  let samples = options.samples;
  let seconds = options.second ?? [];

  /** Draws both readings in the box's device pixels, the line's middle inside its edges. */
  function draw(): void {
    const width = box.clientWidth;
    const height = box.clientHeight;
    if (width === 0 || height === 0) return;
    drawing.setAttribute("viewBox", `0 0 ${width} ${height}`);
    const stroke = Math.max(1, Math.round(length("stroke-thin") * Number.parseFloat(getComputedStyle(box).getPropertyValue("--scale"))));
    const max = options.max ?? DEFAULT_MAX;
    const top = max > 0 ? max : Math.max(1, ...samples.map((each) => each[1]), ...seconds.map((each) => each[1])) * FIT_HEADROOM;
    const end = Math.max(newest(samples), newest(seconds));
    const x = (time: number) => width - ((end - time) / SPAN) * width;
    const y = (value: number) => height - stroke / 2 - Math.min(1, Math.max(0, value / top)) * (height - stroke);
    const path = (series: readonly Sample[]) => series.map(([time, value], index) => `${index === 0 ? "M" : "L"}${x(time)} ${y(value)}`).join(" ");
    line.setAttribute("d", samples.length < 2 ? "" : path(samples));
    area.setAttribute("d", samples.length < 2 ? "" : `${path(samples)} L${x(newest(samples))} ${height} L${x(samples[0]![0])} ${height} Z`);
    second.setAttribute("d", seconds.length < 2 ? "" : path(seconds));
    for (const each of [line, second]) each.style.strokeWidth = `${stroke}px`;
  }

  new ResizeObserver(draw).observe(box);
  return {
    box: probe(box, { role: "component", name: "Sparkline", variant: options.inline ? "inline" : "", fixed: { width: false, height: true } }),
    update(next, nextSecond) {
      samples = next;
      seconds = nextSecond ?? seconds;
      draw();
    },
  };
}

/**
 * A minute of a reading, one sample a second ending at `end`: `shape` gives each sample's value
 * from its second, so a sample's graph is the same whenever it is drawn.
 */
export function minute(end: number, shape: (second: number) => number): Sample[] {
  const count = SPAN / SECOND;
  return Array.from({ length: count + 1 }, (_, index) => {
    const second = Math.floor(end / SECOND) - count + index;
    return [second * SECOND, shape(second)] as const;
  });
}
