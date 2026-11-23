// The values a Settings window holds: Appearance's and Power's by name, and each other section's
// own under its key (`own`), every one starting at KubaOS's default. Changing one changes that window
// alone, except the motion settings, which are the site's own (the toolbar's), so the page moves as
// they say.

import { number } from "../../../lib/tokens.ts";
import type { Mode } from "../../../site/settings.ts";

/** The glass setting's stops, a percent each, from `opacity.glass_floor` to fully opaque. */
export function glassSteps(): number[] {
  const floor = Math.round(number("opacity-glass_floor") * 100);
  return Array.from({ length: 101 - floor }, (_, i) => floor + i);
}

/** The stop of `glassSteps()` an opacity token names. */
function glassStop(token: "glass" | "glass_day"): number {
  return glassSteps().indexOf(Math.round(number(`opacity-${token}`) * 100));
}

/** Every value a window shows, as it opens: KubaOS's defaults. */
export interface Values {
  /** Mode: Dark, Light, or by the sun. */
  mode: number;
  switchMove: number;
  /** The mode the window is drawn in now. */
  drawn: Mode;
  dayWallpaper: number;
  wallpaperMotion: number;
  /** Glass opacity's stop by night and by day, of `glassSteps()`. */
  glass: [number, number];
  accentFrom: number;
  preset: number;
  cold: number;
  menuMotion: number;
  cursorStyle: number;
  cursorSize: number;
  chargeLimit: number;
  lowStop: number;
  lowAt: number;
  criticalWarning: boolean;
  criticalAt: number;
  batteryNotes: boolean;
  source: number;
  lock: [number, number];
  screensOff: [number, number];
  suspend: [number, number];
  lid: [number, number];
  keepAwake: boolean;
  keepFor: string;
  stopOn: [number, number];
  scheduler: number;
  yields: boolean;
  panelSaving: boolean;
  tuned: number;
  panel: number[];
  aspm: number[];
  cpuMax: string[];
  /** The window's frame, once drawn: what the mode recolours. */
  window: HTMLElement | null;
  /** The sub-page drawn open as the window opens, by its id: a sample's. */
  opened?: string;
  /** Each section's own values, by the section's key, made as the section is first drawn (`own`). */
  sections: Record<string, unknown>;
  /** Values a sample starts a section's own with, by the section's key, over its defaults. */
  seed?: Record<string, object>;
}

export function defaults(): Values {
  // The tokens are read once the page's CSS is there, not as the module loads: `D` is made then.
  let glass: [number, number] | null = null;
  return {
    mode: 0,
    switchMove: 0,
    drawn: "dark",
    dayWallpaper: 0,
    wallpaperMotion: 1,
    get glass(): [number, number] {
      glass ??= [glassStop("glass"), glassStop("glass_day")];
      return glass;
    },
    set glass(stops: [number, number]) {
      glass = stops;
    },
    accentFrom: 0,
    preset: 0,
    cold: 0,
    menuMotion: 0,
    cursorStyle: 0,
    cursorSize: 2,
    chargeLimit: 2,
    lowStop: 0,
    lowAt: 3,
    criticalWarning: true,
    criticalAt: 2,
    batteryNotes: true,
    source: 0,
    lock: [3, 3],
    screensOff: [4, 4],
    suspend: [3, 6],
    lid: [0, 2],
    keepAwake: false,
    keepFor: "",
    stopOn: [1, 2],
    scheduler: 0,
    yields: true,
    panelSaving: true,
    tuned: 0,
    panel: [0, 1, 1, 2, 2],
    aspm: [3, 2, 0, 1, 1],
    cpuMax: ["1600", "0", "0", "0", "0"],
    window: null,
    sections: {},
  };
}

/** KubaOS's defaults: what every reset puts back. */
export const D = defaults();

/**
 * Section `key`'s own values in `values`: made from `make`, its defaults, the first time the section
 * is drawn, with the sample's seed for it over them, then kept for the window.
 */
export function own<T extends object>(values: Values, key: string, make: () => T): T {
  const kept = values.sections[key] as T | undefined;
  if (kept) return kept;
  const made = { ...make(), ...(values.seed?.[key] ?? {}) } as T;
  values.sections[key] = made;
  return made;
}
