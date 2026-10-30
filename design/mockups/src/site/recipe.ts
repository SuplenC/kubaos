// A move's recipe: its tracks resolved for the settings picked and laid on the player's loop, as
// the keyframes the player animates, and as the JSON a library reads to build the same move. Both
// come from the page's `Move`, so what plays and what is written down are one thing.

import { cubic, length, number, raw } from "../lib/tokens.ts";
import type { Curve, Duration, Easing, Move, NonEmpty, Strike, Track, Value } from "./page.ts";
import { PATHS } from "./recolour.ts";
import { type Mode, settings } from "./settings.ts";

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
  return cubic(`easing-${easing}`);
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

/** Whether a curve is the neon sign's stutter, either way. */
export function struck(curve: Curve): curve is Strike {
  return curve === "neon.strike" || curve === "neon.strike.reversed";
}

/**
 * A stuttering track's progress from its `from` (0) to its `to` (1) at each even step: a strike's
 * brightness, or for one switching off, the strike's steps played backwards, the tube's brightness
 * at each step then `1 - progress`.
 */
export function stepsOf(curve: Strike): number[] {
  const values = strikeSteps();
  return curve === "neon.strike" ? values : values.map((_, i) => 1 - values[values.length - 1 - i]!);
}

/** A track's cue in milliseconds at speed 1: the sum of its phases. */
export function cueOf(track: Track): number {
  if (!track.cue) return 0;
  const phases: readonly Duration[] = typeof track.cue === "string" ? [track.cue] : track.cue;
  return phases.reduce((sum, phase) => sum + milliseconds(phase), 0);
}

/** A track's cue as its phases' tokens, empty for one that starts with the move. */
export function cuePhases(track: Track): readonly Duration[] {
  if (!track.cue) return [];
  return typeof track.cue === "string" ? [track.cue] : track.cue;
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

/**
 * Which of a move's track lists plays: its own, reduced motion's, light mode's, or the stutter's
 * off one.
 */
export type TrackSet = "tracks" | "reduced" | "daylight" | "unstuttered";

/** The tracks of `set` in `move`. */
export function tracksOf(move: Move, set: TrackSet): NonEmpty<Track> | null {
  if (set === "tracks") return move.tracks;
  if (set === "reduced") return move.reduced === "same" ? null : move.reduced;
  if (set === "daylight") return move.daylight ?? null;
  return move.unstuttered ?? null;
}

/**
 * The set the settings picked play, in `mode`, the one picked by default: reduced motion's first,
 * then light mode's, then the stutter's.
 */
export function playing(move: Move, mode: Mode = settings().mode): TrackSet {
  const { reduced, stutter } = settings();
  if (reduced && move.reduced !== "same") return "reduced";
  if (mode === "light" && move.daylight) return "daylight";
  if (reduced) return "tracks";
  if (!stutter && move.unstuttered) return "unstuttered";
  return "tracks";
}

/** A stretch of the loop, in milliseconds from its start. */
export interface Span {
  readonly start: number;
  readonly end: number;
}

/**
 * Each track of `tracks` that shares its target and property with an earlier one, and that earlier
 * track: the value before its cue is whatever the earlier one left. Such tracks are listed in cue
 * order, take numbers, and never overlap; a recipe that breaks that is refused.
 */
export function leads(tracks: readonly Track[]): Map<Track, Track> {
  const last = new Map<string, Track>();
  const found = new Map<Track, Track>();
  for (const track of tracks) {
    const key = JSON.stringify([track.target, track.property]);
    const lead = last.get(key);
    last.set(key, track);
    if (!lead) continue;
    const named = `${track.part}'s ${track.property} on ${track.target}`;
    if (cueOf(lead) + milliseconds(lead.duration) > cueOf(track)) throw new Error(`${named} starts before the earlier track on it ends, or is listed before it`);
    for (const each of [lead, track]) {
      if (each.colours || typeof css(each.from) !== "number" || typeof css(each.to) !== "number") throw new Error(`${named} shares its property with another track, which needs numbers`);
    }
    found.set(track, lead);
  }
  return found;
}

/** A track on the loop: when it plays forward, and back if the move plays back. */
export interface Timed {
  readonly track: Track;
  readonly forward: Span;
  readonly back: Span | null;
  /** The earlier track on the same target and property, whose value holds until this one's cue. */
  readonly lead: Timed | null;
}

/** The player's loop of a move at the settings picked, in `mode`: hold, move, hold, back. */
export interface Loop {
  readonly set: TrackSet;
  readonly total: number;
  readonly phases: readonly { readonly name: string; readonly span: Span }[];
  readonly timed: readonly Timed[];
}

export function loop(move: Move, mode: Mode = settings().mode): Loop {
  const set = playing(move, mode);
  const tracks = tracksOf(move, set) ?? move.tracks;
  // The speed setting divides every duration and cue, as `kubaos_ui::motion::Motion` does; a hold
  // is no motion, so it stays.
  const speed = settings().speed;
  const hold = milliseconds(HOLD);
  const timing = tracks.map((track) => ({
    track,
    cue: cueOf(track) / speed,
    duration: milliseconds(track.duration) / speed,
  }));
  const parts = Math.max(...timing.map((t) => t.cue + t.duration));
  // The closing beat belongs to the move, but reduced motion's moves have none: they are done at once.
  const tail = move.tail && set !== "reduced" ? milliseconds(move.tail) / speed : 0;
  const length = parts + tail;
  const backStart = hold + length + hold;
  const plays = move.back === "plays";
  const total = plays ? backStart + length : backStart;
  const led = leads(tracks);
  // Played back, a later track would go back before the earlier one it follows has.
  if (plays && led.size > 0) throw new Error(`${move.name} plays back, so no two of its tracks may share a property`);
  const timed: Timed[] = [];
  for (const { track, cue, duration } of timing) {
    const lead = led.get(track);
    timed.push({
      track,
      forward: { start: hold + cue, end: hold + cue + duration },
      back: plays ? { start: backStart + cue, end: backStart + cue + duration } : null,
      lead: lead ? timed.find((each) => each.track === lead)! : null,
    });
  }
  const phases = [
    { name: "Hold the first state", span: { start: 0, end: hold } },
    { name: "The move", span: { start: hold, end: hold + parts } },
    ...(tail > 0 ? [{ name: `The closing beat, \`duration.${move.tail}\``, span: { start: hold + parts, end: hold + length } }] : []),
    { name: "Hold the last state", span: { start: hold + length, end: backStart } },
    ...(plays ? [{ name: "The move back", span: { start: backStart, end: total } }] : []),
  ];
  return { set, total, phases, timed };
}

/** The moments a track changes value: each step of a strike, or its start and end. */
export function steps(timed: Timed): Span[] {
  const spans = [timed.forward, ...(timed.back ? [timed.back] : [])];
  if (!struck(timed.track.curve)) return spans;
  const count = strikeSteps().length;
  return spans.flatMap((span) => {
    const step = (span.end - span.start) / count;
    return Array.from({ length: count }, (_, i) => ({ start: span.start + i * step, end: span.start + (i + 1) * step }));
  });
}

/**
 * `lead`'s progress as `timed`'s: the same value, on `timed`'s scale from its `from` to its `to`.
 * Both take numbers (`leads`).
 */
export function toward(lead: Track, track: Track): (part: number) => number {
  const a = between(lead, 0);
  const b = between(lead, 1);
  const from = between(track, 0);
  const to = between(track, 1);
  return (part) => (a + (b - a) * part - from) / (to - from);
}

/**
 * How far `timed` is from its `from` (0) to its `to` (1) at `t` on the loop, as its keyframes play
 * it: before its start, its lead's value or its own `from`; along its curve or `neon.strike`'s
 * steps, held, then back along the same curve, or held to the loop's end when the move cuts back.
 */
export function progress(timed: Timed, t: number): number {
  const along = (span: Span) => (t - span.start) / (span.end - span.start);
  const { track, forward, back, lead } = timed;
  if (t < forward.start) return lead ? toward(lead.track, track)(progress(lead, t)) : 0;
  if (t < forward.end) {
    if (!struck(track.curve)) return eased(track.curve, along(forward));
    const values = stepsOf(track.curve);
    return values[Math.min(values.length - 1, Math.floor(along(forward) * values.length))]!;
  }
  if (!back || t < back.start) return 1;
  // A move that plays back is eased, never struck (`page.ts`, `Struck`).
  if (t < back.end && !struck(track.curve)) return 1 - eased(track.curve, along(back));
  return 0;
}

interface Stop {
  readonly at: number;
  readonly value: number | string;
  readonly easing: string;
}

/**
 * The keyframes of one track over the whole loop, which the player plays and seeks. Before its cue
 * they play its lead's up to then, so a later track on the same property, which wins over the
 * earlier, starts from what the earlier left.
 */
export function keyframes(timed: Timed, total: number): Keyframe[] {
  return stopsOf(timed, total).map((stop) => ({ offset: Math.min(1, stop.at / total), easing: stop.easing, [timed.track.property]: stop.value }));
}

function stopsOf(timed: Timed, total: number): Stop[] {
  const { track, lead } = timed;
  const from = css(track.from);
  const to = css(track.to);
  const stops: Stop[] = [];
  const at = (ms: number, value: number | string, easing = "linear") => stops.push({ at: ms, value, easing });
  // A strike only ever plays forward: a move that strikes cuts back (`page.ts`, `Struck`).
  const segment = (span: Span, a: number | string, b: number | string) => {
    at(span.start, a);
    if (struck(track.curve)) {
      const values = stepsOf(track.curve);
      values.forEach((step, i) => {
        at(span.start + (i * (span.end - span.start)) / values.length, between(track, step), "step-end");
      });
    } else {
      at(span.start, a, `cubic-bezier(${bezier(track.curve).join(", ")})`);
    }
    at(span.end, b);
  };
  if (lead) {
    stops.push(...stopsOf(lead, total).filter((stop) => stop.at < timed.forward.start));
    // The lead holds its last value up to the cue.
    at(timed.forward.start, css(lead.track.to));
  } else {
    at(0, from);
  }
  segment(timed.forward, from, to);
  if (timed.back) {
    segment(timed.back, to, from);
    at(total, from);
  } else {
    // Cut back: the last state holds to the end, and the loop starts again on the first.
    at(total, to);
  }
  return stops;
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
  const curve = struck(track.curve)
    ? {
        token: "neon.strike",
        ...(track.curve === "neon.strike.reversed" ? { reversed: "its steps played backwards: a tube switching off" } : {}),
        keyframes: [
          ...stepsOf(track.curve).map((step, i, all) => ({ ms: (i * duration) / all.length, value: between(track, step) })),
          { ms: duration, value: between(track, 1) },
        ],
        stepped: "each value holds until the next",
      }
    : { token: `easing.${track.curve}`, cubicBezier: bezier(track.curve) };
  const phases = cuePhases(track);
  return {
    part: track.part,
    on: track.on,
    property: track.property,
    from: valueJson(track.from),
    to: valueJson(track.to),
    duration: { token: `duration.${track.duration}`, ms: duration },
    cue: phases.length > 0 ? { phases: phases.map((phase) => ({ token: `duration.${phase}`, ms: milliseconds(phase) })), ms: cueOf(track) } : { ms: 0 },
    curve,
    ...(track.colours
      ? {
          colours: {
            roles: "every colour role, each from its value in `from` to its value in `to`, in OKLCH; the derived roles follow their tone",
            from: track.colours.from,
            to: track.colours.to,
            path: { name: track.colours.path, ...PATHS[track.colours.path] },
            progress: "the property, eased on the curve, is how far along the path every role is",
          },
        }
      : {}),
  };
}

/** `move`'s recipe as JSON data: what a library builds the same move from. */
export function recipe(move: Move): Record<string, unknown> {
  return {
    name: move.name,
    what: move.what,
    tracks: move.tracks.map(trackJson),
    ...(move.tail ? { tail: { token: `duration.${move.tail}`, ms: milliseconds(move.tail), what: "a still beat after the last part, part of the move" } } : {}),
    reduced: move.reduced === "same" ? "same" : move.reduced.map(trackJson),
    ...(move.daylight ? { daylight: move.daylight.map(trackJson) } : {}),
    ...(move.unstuttered ? { unstuttered: move.unstuttered.map(trackJson) } : {}),
    speed: "every duration, cue and beat divided by the speed setting, 0.5 to 2",
    player: {
      hold: { token: `duration.${HOLD}`, ms: milliseconds(HOLD) },
      loop: "hold the first state, the move, hold the last state, then back",
      back: move.back,
    },
  };
}
