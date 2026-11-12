// The depth layers of the wallpaper picker's pictures, which the lock clock can stand behind, step
// by step: each step is what stands in front of the clock when it stands behind that many layers,
// nearest first. The layers are made once, offline, from the depth map the depth tool made of each
// picture (`assets/lock/CREDITS.md`); the shipped cut-out is one of the steps at its own quality,
// and everything nearer than it stands in front of the clock too.

import tower1 from "../../assets/lock/tokyo-tower-blue-hour-layer-1.webp?url";
import tower2 from "../../assets/lock/tokyo-tower-blue-hour-layer-2.webp?url";
import tower3 from "../../assets/lock/tokyo-tower-blue-hour-layer-3.webp?url";
import type { Layer, Wallpaper } from "../../lib/lock.ts";
import { PURPLE_HOUR, TOWER } from "../templates/locks.ts";

/** A picture's steps, and which of them puts the shipped cut-out in front of the clock. */
export interface Steps {
  /** Step k (from 1) is `steps[k - 1]`: everything in front of the clock behind k layers. */
  readonly steps: readonly (readonly Layer[])[];
  /** The step behind the shipped cut-out: the clock's place by default. Null with no cut-out. */
  readonly cut: number | null;
}

const far = (mask: string): Layer => ({ mask, near: false });
const city = (wallpaper: Wallpaper): Layer => ({ mask: wallpaper.depth!.cutout, near: true });

// Tokyo Tower: two bands of the city nearer than the tower, the tower's own cut-out at its depth,
// and the band past it. Shinjuku's city is one layer, its cut-out against the sky.
const KNOWN = new Map<string, Steps>([
  [TOWER.picture, { steps: [[far(tower1)], [far(tower2)], [city(TOWER), far(tower2)], [city(TOWER), far(tower3)]], cut: 3 }],
  [PURPLE_HOUR.picture, { steps: [[city(PURPLE_HOUR)]], cut: 1 }],
]);

const NONE: Steps = { steps: [], cut: null };

/** `wallpaper`'s steps: none for a picture with no depth map. */
export function stepsOf(wallpaper: Wallpaper): Steps {
  return KNOWN.get(wallpaper.picture) ?? NONE;
}
