// A move's recipe: its tracks resolved for the settings picked and laid on the player's loop, as
// the keyframes the player animates, and as the JSON a library reads to build the same move. Both
// come from the page's `Move`, so what plays and what is written down are one thing.

import { length, number, raw } from "../lib/tokens.ts";
import type { Duration, Easing, Move, NonEmpty, Track, Value } from "./page.ts";
import { settings } from "./settings.ts";

/** One frame at 60 Hz, in milliseconds: the scrubber's step. */
export const FRAME = 1000 / 60;

/** How long the player holds each end of a move: long enough to see the state. */
export const HOLD: Duration = "scene";

/** A duration token in milliseconds, as the tokens write it; the browser hands it back in seconds. */
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

/**
 * The set the settings picked play. A move that strikes says what reduced motion and the stutter
 * off make of it, as both turn a strike into a fade.
 */
export function playing(move: Move): TrackSet {
  const strikes = (list: readonly Track[]) => list.some((track) => track.curve === "neon.strike");
  if (strikes(move.tracks) && (move.reduced === "same" || !move.unstuttered)) {
    throw new Error(`${move.name} strikes, so it names its reduced and its unstuttered tracks`);
  }
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

/** The keyframes of one track over the whole loop, which the player plays and seeks. */
export function keyframes(timed: Timed, total: number): Keyframe[] {
  const { track } = timed;
  const from = css(track.from);
  const to = css(track.to);
  const stops: { at: number; value: number | string; easing: string }[] = [];
  const at = (ms: number, value: number | string, easing = "linear") => stops.push({ at: ms, value, easing });
  const segment = (span: Span, a: number | string, b: number | string, forward: boolean) => {
    at(span.start, a);
    if (track.curve === "neon.strike") {
      const count = strikeSteps().length;
      strikeSteps().forEach((step, i) => {
        const value = between(track, forward ? step : 1 - step);
        at(span.start + (i * (span.end - span.start)) / count, value, "step-end");
      });
    } else {
      at(span.start, a, `cubic-bezier(${bezier(track.curve).join(", ")})`);
    }
    at(span.end, b);
  };
  at(0, from);
  segment(timed.forward, from, to, true);
  if (timed.back) {
    segment(timed.back, to, from, false);
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
    back: move.back,
  };
}
