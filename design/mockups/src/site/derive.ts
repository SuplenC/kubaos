// The derived colour roles CSS can't compute, for any palette: each a recipe over the palette's own
// tones (`[color.derived]` in the tokens), by `kubaos-tokens`' arithmetic
// (`crates/kubaos-tokens/src/model.rs`, `oklch.rs`), so a theme the toolbar paints in gets the
// same unlit fills, lit fills and fitted labels the tokens' CSS gives the default palette.
//
// A recipe starts from a role, mixed toward white or black in sRGB or as it is, may move its OKLCH
// lightness and chroma (fitted back into sRGB by lowering its chroma alone, so its hue and
// lightness hold), and may fit its lightness until it holds 4.5:1 on every background it is text on.

import { words } from "../lib/tokens.ts";
import { ratio } from "./contrast.ts";
import type { Palette } from "./recolour.ts";
import type { Mode } from "./settings.ts";

/** One role's recipe in one mode, as the tokens' CSS writes it. */
interface Recipe {
  readonly mix?: string;
  readonly white?: number;
  readonly black?: number;
  readonly from?: string;
  readonly lightness?: number;
  readonly at_lightness?: number;
  readonly lighter?: number;
  readonly darker?: number;
  readonly chroma?: number;
  readonly text_on?: readonly string[];
  /** A tone mixed toward white or black alone: a `color-mix` formula in the tokens' CSS. */
  readonly formula: boolean;
}

const recipes = new Map<Mode, Readonly<Record<string, Recipe>>>();

function recipesOf(mode: Mode): Readonly<Record<string, Recipe>> {
  const known = recipes.get(mode);
  if (known) return known;
  const read = JSON.parse(words(`derived-${mode}`)) as Record<string, Recipe>;
  recipes.set(mode, read);
  return read;
}

/** Forgets the recipes read: the tokens changed. */
export function forgetRecipes(): void {
  recipes.clear();
}

/** How many halvings a fit searches by: far below an 8 bit channel's step. */
const HALVINGS = 40;

type Lch = { l: number; c: number; h: number };

const linear = (v: number) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
const encoded = (v: number) => (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055);

function toLch(hex: string): Lch {
  const [r, g, b] = [1, 3, 5].map((at) => linear(Number.parseInt(hex.slice(at, at + 2), 16) / 255)) as [number, number, number];
  const long = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const medium = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const short = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const l = 0.2104542553 * long + 0.793617785 * medium - 0.0040720468 * short;
  const greenRed = 1.9779984951 * long - 2.428592205 * medium + 0.4505937099 * short;
  const blueYellow = 0.0259040371 * long + 0.7827717662 * medium - 0.808675766 * short;
  const c = Math.hypot(greenRed, blueYellow);
  const h = c < 1e-6 ? 0 : (((Math.atan2(blueYellow, greenRed) * 180) / Math.PI) % 360 + 360) % 360;
  return { l, c, h };
}

function linearOf({ l, c, h }: Lch): [number, number, number] {
  const greenRed = c * Math.cos((h * Math.PI) / 180);
  const blueYellow = c * Math.sin((h * Math.PI) / 180);
  const long = (l + 0.3963377774 * greenRed + 0.2158037573 * blueYellow) ** 3;
  const medium = (l - 0.1055613458 * greenRed - 0.0638541728 * blueYellow) ** 3;
  const short = (l - 0.0894841775 * greenRed - 1.291485548 * blueYellow) ** 3;
  return [
    4.0767416621 * long - 3.3077115913 * medium + 0.2309699292 * short,
    -1.2684380046 * long + 2.6097574011 * medium - 0.3413193965 * short,
    -0.0041960863 * long - 0.7034186147 * medium + 1.707614701 * short,
  ];
}

const inGamut = (lch: Lch) => linearOf(lch).every((v) => v >= -1e-9 && v <= 1 + 1e-9);

/** `lch` in sRGB, `#rrggbb`, its chroma lowered at the same hue and lightness until it fits. */
function toHex(lch: Lch): string {
  const fitted = { ...lch, l: Math.min(1, Math.max(0, lch.l)), c: Math.max(0, lch.c) };
  if (!inGamut(fitted)) {
    let [inside, outside] = [0, fitted.c];
    for (let i = 0; i < HALVINGS; i++) {
      const middle = (inside + outside) / 2;
      if (inGamut({ ...fitted, c: middle })) inside = middle;
      else outside = middle;
    }
    fitted.c = inside;
  }
  const byte = (v: number) => Math.min(255, Math.max(0, Math.floor(encoded(Math.min(1, Math.max(0, v))) * 255 + 0.5)));
  return `#${linearOf(fitted).map((v) => byte(v).toString(16).padStart(2, "0")).join("")}`;
}

/** `hex` taken `t` of the way to `toward`, each channel rounded half up. */
function mixed(hex: string, toward: "white" | "black", t: number): string {
  const channel = (at: number) => {
    const c = Number.parseInt(hex.slice(at, at + 2), 16);
    const v = toward === "white" ? c + (255 - c) * t : c * (1 - t);
    return Math.min(255, Math.max(0, Math.floor(v + 0.5)));
  };
  return `#${[1, 3, 5].map((at) => channel(at).toString(16).padStart(2, "0")).join("")}`;
}

/** `colour`'s lightness moved only as far as it takes to hold 4.5:1 on every one of `backgrounds`. */
function fitText(colour: string, backgrounds: readonly string[]): string {
  const holds = (c: string) => backgrounds.every((bg) => ratio(c, bg) >= 4.5);
  if (holds(colour)) return colour;
  const start = toLch(colour);
  const lighter = backgrounds.every((bg) => toLch(bg).l <= start.l);
  const at = (l: number) => toHex({ ...start, l });
  const first = lighter ? 1 : 0;
  const end = [first, 1 - first].find((each) => holds(at(each)));
  if (end === undefined) return at(first);
  let [failing, holding] = [start.l, end];
  for (let i = 0; i < HALVINGS; i++) {
    const middle = (failing + holding) / 2;
    if (holds(at(middle))) holding = middle;
    else failing = middle;
  }
  return at(holding);
}

function compute(recipe: Recipe, role: (name: string) => string | undefined): string | undefined {
  let colour: string | undefined;
  if (recipe.mix !== undefined) {
    const tone = role(recipe.mix);
    if (tone === undefined) return undefined;
    colour = recipe.white !== undefined ? mixed(tone, "white", recipe.white) : mixed(tone, "black", recipe.black ?? 0);
  } else if (recipe.from !== undefined) {
    colour = role(recipe.from);
  }
  if (colour === undefined) return undefined;
  const moves = [recipe.lightness, recipe.at_lightness, recipe.lighter, recipe.darker, recipe.chroma].some((each) => each !== undefined);
  if (moves) {
    const lch = toLch(colour);
    if (recipe.at_lightness !== undefined) lch.l = recipe.at_lightness;
    lch.l = lch.l * (recipe.lightness ?? 1) + (recipe.lighter ?? 0) - (recipe.darker ?? 0);
    lch.c *= recipe.chroma ?? 1;
    colour = toHex(lch);
  }
  if (recipe.text_on) {
    const backgrounds = recipe.text_on.map(role);
    if (backgrounds.some((each) => each === undefined)) return undefined;
    colour = fitText(colour, backgrounds as string[]);
  }
  return colour;
}

/**
 * The derived roles of `palette`, a palette of `mode`, that CSS can't compute, from its own tones:
 * what a page painted in it sets. The roles that are a tone mixed toward white or black are left
 * to the tokens' `color-mix`, which follows the tone wherever it is set.
 */
export function derived(palette: Palette, mode: Mode): Palette {
  const all = recipesOf(mode);
  const out: Record<string, string> = {};
  const reading = new Set<string>();
  const role = (name: string): string | undefined => {
    const known = out[name] ?? (all[name] ? undefined : palette[name]);
    if (known !== undefined || !all[name] || reading.has(name)) return known;
    reading.add(name);
    const colour = compute(all[name], role);
    reading.delete(name);
    if (colour !== undefined) out[name] = colour;
    return colour;
  };
  for (const name of Object.keys(all)) role(name);
  return Object.fromEntries(Object.entries(out).filter(([name]) => !all[name]!.formula));
}
