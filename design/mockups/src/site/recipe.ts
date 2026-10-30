// A move's recipe: its tracks resolved for the settings picked and laid on the player's loop, as
// the keyframes the player animates, and as the JSON a library reads to build the same move. Both
// come from the page's `Move`, so what plays and what is written down are one thing.

import { length, number, raw } from "../lib/tokens.ts";
import type { Duration, Easing, Move, NonEmpty, Track, Value } from "./page.ts";
import { settings } from "./settings.ts";

/** One frame at 60 Hz, in milliseconds: the step the plot's arrow keys take. */
export const FRAME = 1000 / 60;

/** How long the player holds each end of a move: long enough to see the state. */
export const HOLD: Duration = "scene";

/**
 * A duration token in milliseconds, as the tokens write it; the browser hands it back in seconds.
 */
export function milliseconds(duration: Duration): number {
  const value = raw(`duration-${duration}`);
  const amount = Number.parseFloat(value);
  return Math.round(value.endsWith("ms") ? amount : amount * 1000);
}

/** A curve token's four numbers. */
export function bezier(easing: Easing): number[] {
  const inside = raw(`easing-${easing}`).replace(/^cubic-bezier\((.*)\)$/, "$1");
  return inside.split(",").map((n) => Number.parseFloat(n));
}

/**
 * How far along a curve token is at `x`, a share of its duration: the cubic Bézier solved for its
 * y at that x as Chromium does (`gfx::CubicBezier`), Newton's method then bisection, to 1e-7. An
 * overshooting curve (`pop`, `spring`) goes past 1.
 */
export function eased(easing: Easing, x: number): number {
  const [x1, y1, x2, y2] = bezier(easing) as [number, number, number, number];
  const at = (a: number, b: number, t: number) => 3 * a * t * (1 - t) ** 2 + 3 * b * t * t * (1 - t) + t ** 3;
  const slope = (a: number, b: number, t: number) => 3 * a * (1 - t) ** 2 + 6 * (b - a) * t * (1 - t) + 3 * (1 - b) * t * t;
  if (x <= 0) return 0;
  if (x >= 1) return 1;
  let t = x;
  for (let i = 0; i < 8; i++) {
    const error = at(x1, x2, t) - x;
    if (Math.abs(error) < 1e-7) return at(y1, y2, t);
    const d = slope(x1, x2, t);
    if (Math.abs(d) < 1e-6) break;
    t -= error / d;
  }
  let low = 0;
  let high = 1;
  t = x;
  while (high - low > 1e-7) {
    if (at(x1, x2, t) < x) low = t;
    else high = t;
    t = (low + high) / 2;
  }
  return at(y1, y2, t);
}

/** The lowest and highest a curve token goes, over its whole duration. */
export function reach(easing: Easing): { low: number; high: number } {
  const [, y1, , y2] = bezier(easing) as [number, number, number, number];
  let low = 0;
  let high = 1;
  for (let i = 1; i < 1000; i++) {
    const t = i / 1000;
    const y = 3 * y1 * t * (1 - t) ** 2 + 3 * y2 * t * t * (1 - t) + t ** 3;
    low = Math.min(low, y);
    high = Math.max(high, y);
  }
  return { low, high };
}

/** `neon.strike`'s steps: the tube's brightness at each even step of its duration. */
export function strikeSteps(): number[] {
  const count = number("neon-strike-steps");
  return Array.from({ length: count }, (_, i) => number(`neon-strike-${i}`));
}

/** A value's token, as the tokens name it, or null for a plain number. */
export function tokenOf(value: Value): string | null {
  if (typeof value === "number") return null;
  if ("opacity" in value) return `opacity.${value.opacity}`;
  if ("color" in value) return `color.${value.color}`;
  return value.length.replace("-", ".");
}

/** A value as the property takes it: a number, or the token's custom property. */
export function css(value: Value): number | string {
  if (typeof value === "number") return value;
  if ("opacity" in value) return number(`opacity-${value.opacity}`);
  if ("color" in value) return `var(--color-${value.color})`;
  return `var(--${value.length})`;
}

/** A track's value `part` of the way from `from` to `to`, for a stepped move: numbers only. */
export function between(track: Track, part: number): number {
  const from = css(track.from);
  const to = css(track.to);
  if (typeof from !== "number" || typeof to !== "number") {
    throw new Error(`${track.part}'s ${track.property} steps through neon.strike, which needs numbers`);
  }
  return from + (to - from) * part;
}

/** Which of a move's track lists plays: its own, reduced motion's, or the stutter's off one. */
export type TrackSet = "tracks" | "reduced" | "unstuttered";

/** The tracks of `set` in `move`. */
export function tracksOf(move: Move, set: TrackSet): NonEmpty<Track> | null {
  if (set === "tracks") return move.tracks;
  if (set === "reduced") return move.reduced === "same" ? null : move.reduced;
  return move.unstuttered ?? null;
}

/** The set the settings picked play. */
export function playing(move: Move): TrackSet {
  const { reduced, stutter } = settings();
  if (reduced) return move.reduced === "same" ? "tracks" : "reduced";
  if (!stutter && move.unstuttered) return "unstuttered";
  return "tracks";
}

/** A stretch of the loop, in milliseconds from its start. */
export interface Span {
  readonly start: number;
  readonly end: number;
}

/** A track on the loop: when it plays forward, and back if the move plays back. */
export interface Timed {
  readonly track: Track;
  readonly forward: Span;
  readonly back: Span | null;
}

/** The player's loop of a move at the settings picked: hold, move, hold, back. */
export interface Loop {
  readonly set: TrackSet;
  readonly total: number;
  readonly phases: readonly { readonly name: string; readonly span: Span }[];
  readonly timed: readonly Timed[];
}

export function loop(move: Move): Loop {
  const set = playing(move);
  const tracks = tracksOf(move, set) ?? move.tracks;
  // The speed setting divides every duration and cue, as `kubaos_ui::motion::Motion` does; a hold
  // is no motion, so it stays.
  const speed = settings().speed;
  const hold = milliseconds(HOLD);
  const timing = tracks.map((track) => ({
    track,
    cue: track.cue ? milliseconds(track.cue) / speed : 0,
    duration: milliseconds(track.duration) / speed,
  }));
  const length = Math.max(...timing.map((t) => t.cue + t.duration));
  const backStart = hold + length + hold;
  const plays = move.back === "plays";
  const total = plays ? backStart + length : backStart;
  const timed = timing.map(({ track, cue, duration }) => ({
    track,
    forward: { start: hold + cue, end: hold + cue + duration },
    back: plays ? { start: backStart + cue, end: backStart + cue + duration } : null,
  }));
  const phases = [
    { name: "Hold the first state", span: { start: 0, end: hold } },
    { name: "The move", span: { start: hold, end: hold + length } },
    { name: "Hold the last state", span: { start: hold + length, end: backStart } },
    ...(plays ? [{ name: "The move back", span: { start: backStart, end: total } }] : []),
  ];
  return { set, total, phases, timed };
}

/** The moments a track changes value: each step of a strike, or its start and end. */
export function steps(timed: Timed): Span[] {
  const spans = [timed.forward, ...(timed.back ? [timed.back] : [])];
  if (timed.track.curve !== "neon.strike") return spans;
  const count = strikeSteps().length;
  return spans.flatMap((span) => {
    const step = (span.end - span.start) / count;
    return Array.from({ length: count }, (_, i) => ({ start: span.start + i * step, end: span.start + (i + 1) * step }));
  });
}

/**
 * How far `timed` is from its `from` (0) to its `to` (1) at `t` on the loop, as its keyframes play
 * it: before its start, along its curve or `neon.strike`'s steps, held, then back along the same
 * curve, or held to the loop's end when the move cuts back.
 */
export function progress(timed: Timed, t: number): number {
  const along = (span: Span) => (t - span.start) / (span.end - span.start);
  const { track, forward, back } = timed;
  if (t < forward.start) return 0;
  if (t < forward.end) {
    if (track.curve !== "neon.strike") return eased(track.curve, along(forward));
    const values = strikeSteps();
    return values[Math.min(values.length - 1, Math.floor(along(forward) * values.length))]!;
  }
  if (!back || t < back.start) return 1;
  // A move that plays back is eased, never struck (`page.ts`, `Struck`).
  if (t < back.end && track.curve !== "neon.strike") return 1 - eased(track.curve, along(back));
  return 0;
}

/** The keyframes of one track over the whole loop, which the player plays and seeks. */
export function keyframes(timed: Timed, total: number): Keyframe[] {
  const { track } = timed;
  const from = css(track.from);
  const to = css(track.to);
  const stops: { at: number; value: number | string; easing: string }[] = [];
  const at = (ms: number, value: number | string, easing = "linear") => stops.push({ at: ms, value, easing });
  // A strike only ever plays forward: a move that strikes cuts back (`page.ts`, `Struck`).
  const segment = (span: Span, a: number | string, b: number | string) => {
    at(span.start, a);
    if (track.curve === "neon.strike") {
      const count = strikeSteps().length;
      strikeSteps().forEach((step, i) => {
        at(span.start + (i * (span.end - span.start)) / count, between(track, step), "step-end");
      });
    } else {
      at(span.start, a, `cubic-bezier(${bezier(track.curve).join(", ")})`);
    }
    at(span.end, b);
  };
  at(0, from);
  segment(timed.forward, from, to);
  if (timed.back) {
    segment(timed.back, to, from);
    at(total, from);
  } else {
    // Cut back: the last state holds to the end, and the loop starts again on the first.
    at(total, to);
  }
  return stops.map((stop) => ({ offset: Math.min(1, stop.at / total), easing: stop.easing, [track.property]: stop.value }));
}

function valueJson(value: Value): Record<string, unknown> {
  const token = tokenOf(value);
  if (typeof value === "number") return { value };
  if ("opacity" in value) return { token, value: number(`opacity-${value.opacity}`) };
  if ("color" in value) return { token };
  return { token, logicalPx: length(value.length) };
}

function trackJson(track: Track): Record<string, unknown> {
  const duration = milliseconds(track.duration);
  const curve =
    track.curve === "neon.strike"
      ? {
          token: "neon.strike",
          keyframes: [
            ...strikeSteps().map((step, i, all) => ({ ms: (i * duration) / all.length, value: between(track, step) })),
            { ms: duration, value: between(track, 1) },
          ],
          stepped: "each value holds until the next",
        }
      : { token: `easing.${track.curve}`, cubicBezier: bezier(track.curve) };
  return {
    part: track.part,
    on: track.on,
    property: track.property,
    from: valueJson(track.from),
    to: valueJson(track.to),
    duration: { token: `duration.${track.duration}`, ms: duration },
    cue: track.cue ? { token: `duration.${track.cue}`, ms: milliseconds(track.cue) } : { ms: 0 },
    curve,
  };
}

/** `move`'s recipe as JSON data: what a library builds the same move from. */
export function recipe(move: Move): Record<string, unknown> {
  return {
    name: move.name,
    what: move.what,
    tracks: move.tracks.map(trackJson),
    reduced: move.reduced === "same" ? "same" : move.reduced.map(trackJson),
    ...(move.unstuttered ? { unstuttered: move.unstuttered.map(trackJson) } : {}),
    speed: "every duration and cue divided by the speed setting, 0.5 to 2",
    player: {
      hold: { token: `duration.${HOLD}`, ms: milliseconds(HOLD) },
      loop: "hold the first state, the move, hold the last state, then back",
      back: move.back,
    },
  };
}
