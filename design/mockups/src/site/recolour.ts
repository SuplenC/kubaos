// Every colour role moving from one palette to another, as the theme change and the dark ⇄ light
// switch move them (`docs/DESIGN.md`, "Motion"): in OKLCH, along a path that says when each of a
// colour's lightness, chroma and hue moves over the fade. A path is data, so the recipe a library
// reads names the same windows the mockup plays.
//
// One number drives it: `--recolour`, 0 at the old palette and 1 at the new, which the player
// animates on its easing curve. Each role's colour is a CSS formula of it, so a frame held on the
// plot is exactly the colour that plays there, and the derived roles (the cores) follow their
// tone through it.

import { derived, forgetRecipes } from "./derive.ts";
import { THEMES } from "./themes.ts";
import type { Mode } from "./settings.ts";

/** A colour role's value in each role, `#rrggbb`, by role name. */
export type Palette = Readonly<Record<string, string>>;

/** A stretch of the fade, as shares of it from 0 to 1. */
export type Window = readonly [number, number];

/** How a colour's chroma goes from its old value to its new one. */
export type Chroma =
  /** Drains to grey over `drain`, then rises to the new value over `rise`. */
  | { readonly grey: { readonly drain: Window; readonly rise: Window } }
  /** Straight from the old value to the new over `window`. */
  | { readonly window: Window };

/** How a colour's hue goes from its old value to its new one. */
export type Hue =
  /** Jumps at `at`, where the colour is grey, so it passes through no in-between hue. */
  | { readonly jumpsAt: number }
  /** Turns the short way round over `window`. */
  | { readonly window: Window };

export interface PathSpec {
  readonly lightness: Window;
  readonly chroma: Chroma;
  readonly hue: Hue;
  /** The path in words, as the recipe says it. */
  readonly words: string;
}

/** The paths, by name. */
export const PATHS = {
  "through grey": {
    lightness: [0, 1],
    chroma: { grey: { drain: [0, 0.5], rise: [0.5, 1] } },
    hue: { jumpsAt: 0.5 },
    words:
      "Lightness moves over the whole fade; chroma drains to grey over the first half and the new chroma seeps in over the second; the hue jumps at the grey middle, so nothing passes through an odd in-between colour.",
  },
  dawn: {
    lightness: [0, 0.6],
    chroma: { window: [0.4, 1] },
    hue: { window: [0.4, 1] },
    words:
      "Lightness moves first, over the first 60% of the fade, while each colour keeps its chroma and hue; chroma and hue then settle over the last 60%, the hue the short way round, so the room brightens or darkens before the colours are seen to change.",
  },
} as const satisfies Record<string, PathSpec>;

export type ColourPath = keyof typeof PATHS;

/** A colour in OKLCH: lightness 0 to 1, chroma, hue in degrees (NaN for a grey). */
interface Lch {
  readonly l: number;
  readonly c: number;
  readonly h: number;
}

/** `#rrggbb` in OKLCH (Björn Ottosson's OKLab, from linear sRGB). */
export function oklch(hex: string): Lch {
  const channel = (at: number) => {
    const v = Number.parseInt(hex.slice(at, at + 2), 16) / 255;
    return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  const [r, g, b] = [channel(1), channel(3), channel(5)];
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  const c = Math.hypot(A, B);
  // Below this a colour is grey: its hue is noise, and the other colour's is taken.
  const h = c < 1e-4 ? Number.NaN : ((Math.atan2(B, A) * 180) / Math.PI + 360) % 360;
  return { l: L, c, h };
}

/** A number as a formula writes it. */
const n = (value: number) => String(Math.round(value * 1e6) / 1e6);

/** How far through `window` the fade is, 0 to 1, as CSS: held at each end outside it. */
function through(window: Window): string {
  const [a, b] = window;
  return `clamp(0, calc((var(--recolour) - ${n(a)}) / ${n(b - a)}), 1)`;
}

/** `from` to `to` by `share`, as CSS. */
function mix(from: number, to: number, share: string): string {
  return from === to ? n(from) : `calc(${n(from)} + ${n(to - from)} * ${share})`;
}

/** One role's colour along `path` from `from` to `to`, as a CSS formula of `--recolour`. */
export function formula(path: ColourPath, from: string, to: string): string {
  const spec: PathSpec = PATHS[path];
  const a = oklch(from);
  const b = oklch(to);
  const ha = Number.isNaN(a.h) ? (Number.isNaN(b.h) ? 0 : b.h) : a.h;
  const hb = Number.isNaN(b.h) ? ha : b.h;
  const lightness = mix(a.l, b.l, through(spec.lightness));
  const chroma =
    "grey" in spec.chroma
      ? `calc(${n(a.c)} * (1 - ${through(spec.chroma.grey.drain)}) + ${n(b.c)} * ${through(spec.chroma.grey.rise)})`
      : mix(a.c, b.c, through(spec.chroma.window));
  let hue: string;
  if ("jumpsAt" in spec.hue) {
    hue = mix(ha, hb, `clamp(0, round(down, calc(var(--recolour) / ${n(spec.hue.jumpsAt)}), 1), 1)`);
  } else {
    // The short way round: at most half a turn either way.
    const turn = ((((hb - ha) % 360) + 540) % 360) - 180;
    hue = mix(ha, ha + turn, through(spec.hue.window));
  }
  return `oklch(${lightness} ${chroma} ${hue})`;
}

/** The tokens' fallback palettes read so far, by mode. */
const read = new Map<Mode, Palette>();

/** Every role the tokens give a value in `mode`, read from their CSS: the derived ones are left. */
function fallback(mode: Mode): Palette {
  const known = read.get(mode);
  if (known) return known;
  const probe = document.createElement("div");
  probe.dataset.mode = mode;
  probe.hidden = true;
  document.body.append(probe);
  const style = getComputedStyle(probe);
  // Every role any rule names, read as the mode's box computes it: a role that is a formula
  // computes to it, so only the roles with a value are kept, the derived ones `palette` works out
  // again among them.
  const found: Record<string, string> = {};
  for (const sheet of document.styleSheets) {
    for (const rule of sheet.cssRules) {
      if (!(rule instanceof CSSStyleRule)) continue;
      for (const property of rule.style) {
        if (!property.startsWith("--color-")) continue;
        const value = style.getPropertyValue(property).trim();
        if (/^#[0-9a-f]{6}$/i.test(value)) found[property.slice("--color-".length)] = value;
      }
    }
  }
  probe.remove();
  read.set(mode, found);
  return found;
}

/** Forgets the fallback palettes and the recipes read: the tokens changed. */
export function forgetPalettes(): void {
  read.clear();
  forgetRecipes();
}

/**
 * The palette theme `slug` paints in `mode`: the tokens' fallback, with the theme's matugen roles,
 * and the derived roles CSS can't compute worked out from them.
 */
export function palette(slug: string, mode: Mode): Palette {
  const theme = THEMES.find((each) => each.slug === slug) ?? THEMES[0]!;
  const roles = { ...fallback(mode), ...(theme[mode] ?? {}) };
  return { ...roles, ...derived(roles, mode) };
}

/** A theme's name. */
export function themeName(slug: string): string {
  return (THEMES.find((each) => each.slug === slug) ?? THEMES[0]!).name;
}

/** A palette's name as a recipe says it. */
export function paletteName(slug: string, mode: Mode): string {
  const theme = THEMES.find((each) => each.slug === slug) ?? THEMES[0]!;
  return `${theme.name}, ${mode}`;
}

/** Paints `element` in `to`, each role a CSS variable, so everything inside it takes the palette. */
export function paint(element: HTMLElement, to: Palette): void {
  for (const [role, value] of Object.entries(to)) element.style.setProperty(`--color-${role}`, value);
}

/** Takes the palette painted on `element` off it: it takes its mode's own again. */
export function unpaint(element: HTMLElement): void {
  for (const property of [...element.style]) {
    if (property.startsWith("--color-")) element.style.removeProperty(property);
  }
}

/**
 * Paints `element` on the way from `from` to `to` along `path`: each role a formula of
 * `--recolour`, which an animation on `element` moves from 0 to 1.
 */
export function paintBetween(element: HTMLElement, path: ColourPath, from: Palette, to: Palette): void {
  for (const role of new Set([...Object.keys(from), ...Object.keys(to)])) {
    const a = from[role] ?? to[role]!;
    const b = to[role] ?? a;
    element.style.setProperty(`--color-${role}`, a === b ? a : formula(path, a, b));
  }
}

/**
 * Holds `element`'s unlit glass, every tone's `_unlit`, in `to`'s colours, the palette of `mode`,
 * while the rest recolours: the switch to light, whose glass is daylight's from the first step of
 * the flicker. `unpaint` takes it off with the rest.
 */
export function paintGlass(element: HTMLElement, to: Palette, mode: Mode): void {
  const probe = document.createElement("div");
  probe.dataset.mode = mode;
  probe.hidden = true;
  paint(probe, to);
  document.body.append(probe);
  // Each unlit core computes from its tone where the mode's box is, so the probe's is `to`'s.
  const style = getComputedStyle(probe);
  for (const role of Object.keys(to)) {
    const unlit = style.getPropertyValue(`--color-${role}_unlit`).trim();
    if (unlit) element.style.setProperty(`--color-${role}_unlit`, unlit);
  }
  probe.remove();
}
