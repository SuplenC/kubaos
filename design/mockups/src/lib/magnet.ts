// The magnet every carried thing snaps by (the Overview's workspaces, the bar's workspaces): each
// place it can land in has a zone, a rectangle of the place's shape centred in it, `scale.snap` of
// its width and height. The pointer in a zone proposes that place, and the thing snaps into it;
// in none, it is at the hand. Where two zones meet, the nearer middle wins. The same boundary takes
// it in and lets it out, with nothing remembered between moves.

import { number } from "./tokens.ts";

export interface Rect {
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
}

export interface Point {
  readonly x: number;
  readonly y: number;
}

/** A place's zone, the place it lies in, and what it proposes. */
export interface Zone<T> extends Rect {
  readonly place: Rect;
  readonly target: T;
}

/**
 * The share of a place's sides its zone takes: `scale.snap`, its shorter side never less than
 * `least` (in the place's units), and never past the place.
 */
export function catchShare(place: Rect, least = 0): number {
  return Math.min(1, Math.max(number("scale-snap"), least / Math.min(place.w, place.h)));
}

/** The zone of `place`, `share` of its sides centred in it, proposing `target`. */
export function zoneOf<T>(place: Rect, target: T, share = catchShare(place)): Zone<T> {
  return { x: place.x + (place.w * (1 - share)) / 2, y: place.y + (place.h * (1 - share)) / 2, w: place.w * share, h: place.h * share, place, target };
}

/** Whether `point` is in `box`: its start edges in, its end edges out, so two boxes that touch never share a point. */
export function inside(point: Point, box: Rect): boolean {
  return point.x >= box.x && point.x < box.x + box.w && point.y >= box.y && point.y < box.y + box.h;
}

/** What the zone `point` is in proposes, the nearest middle's where zones meet; null in none. */
export function caught<T>(point: Point, zones: readonly Zone<T>[]): T | null {
  let best: { target: T; off: number } | null = null;
  for (const zone of zones) {
    if (!inside(point, zone)) continue;
    const off = Math.hypot(point.x - (zone.x + zone.w / 2), point.y - (zone.y + zone.h / 2));
    if (!best || off < best.off) best = { target: zone.target, off };
  }
  return best ? best.target : null;
}
