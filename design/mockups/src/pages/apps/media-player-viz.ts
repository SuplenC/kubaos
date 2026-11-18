// The visualisers a song can play behind the player's controller, by their kind, in the order a
// key steps through them. A DRAFT: each in its own file, the engine in `media-player-gl.ts`.

import type { Visual, VizKind } from "./media-player-gl.ts";
import { EMBERS } from "./media-player-viz-embers.ts";
import { SILK } from "./media-player-viz-silk.ts";
import { TUBE } from "./media-player-viz-tube.ts";
import { WATER } from "./media-player-viz-water.ts";

export const VISUALS: Readonly<Record<VizKind, Visual>> = { water: WATER, silk: SILK, tube: TUBE, embers: EMBERS };

/** Every choice, in the order the list and the key step through them; Off last: the still cover. */
export const CHOICES = ["water", "silk", "tube", "embers", "off"] as const;
export type VizChoice = (typeof CHOICES)[number];

/** A choice's name, as the menu and the switches say it. */
export function vizName(choice: VizChoice): string {
  return choice === "off" ? "Off" : VISUALS[choice].name;
}
