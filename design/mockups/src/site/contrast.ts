// The contrast pairs (`[contrast]` in the tokens) and their ratios in any palette, by the
// arithmetic `kubaos-design contrast` reads them with (`crates/kubaos-tokens/src/contrast.rs`): the
// foreground at its opacity blended over the opaque background in gamma-encoded sRGB, as the
// toolkits blend, then WCAG 2.1's ratio of the two.

import { number, raw, words } from "../lib/tokens.ts";
import type { Palette } from "./recolour.ts";

/** What a pair needs, as `[contrast]` names it. */
export type Need = "text" | "large_text" | "graphic";

/** One foreground role drawn on one background role. */
export interface Pair {
  readonly fg: string;
  readonly bg: string;
  readonly need: Need;
  /** The lowest ratio that passes: 4.5 for text, 3 for large text and graphics. */
  readonly minimum: number;
  /** The opacity the foreground is drawn at, by its token's name, or null for none. */
  readonly opacity: string | null;
  /** Where the pair is drawn. */
  readonly use: string;
}

/** Every pair the tokens list, in their order. */
export function pairs(): Pair[] {
  const count = Number(raw("contrast-pairs"));
  return Array.from({ length: count }, (_, i) => {
    const opacity = words(`contrast-${i}-opacity`);
    return {
      fg: words(`contrast-${i}-fg`),
      bg: words(`contrast-${i}-bg`),
      need: words(`contrast-${i}-need`) as Need,
      minimum: number(`contrast-${i}-minimum`),
      opacity: opacity === "" ? null : opacity,
      use: words(`contrast-${i}-use`),
    };
  });
}

/** The roles drawn only as decoration, which need no contrast. */
export function decorative(): string[] {
  return words("contrast-decorative").split(" ");
}

type Srgb = readonly [number, number, number];

function channels(hex: string): Srgb {
  return [1, 3, 5].map((at) => Number.parseInt(hex.slice(at, at + 2), 16) / 255) as unknown as Srgb;
}

function luminance(c: Srgb): number {
  const linear = (v: number) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  return 0.2126 * linear(c[0]) + 0.7152 * linear(c[1]) + 0.0722 * linear(c[2]);
}

/** WCAG 2.1's ratio of `fg` at `alpha` over the opaque `bg`, both `#rrggbb`: 1 to 21. */
export function ratio(fg: string, bg: string, alpha = 1): number {
  const f = channels(fg);
  const b = channels(bg);
  const seen = [0, 1, 2].map((i) => b[i]! + (f[i]! - b[i]!) * alpha) as unknown as Srgb;
  const [la, lb] = [luminance(seen), luminance(b)];
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

/** `pair`'s ratio in `palette`, or null where the palette lacks one of its roles. */
export function ratioIn(pair: Pair, palette: Palette): number | null {
  const fg = palette[pair.fg];
  const bg = palette[pair.bg];
  if (!fg || !bg) return null;
  return ratio(fg, bg, pair.opacity ? number(`opacity-${pair.opacity}`) : 1);
}
