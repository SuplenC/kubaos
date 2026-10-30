// Corners by role, and the corner a shape takes when it fills another less an inset:
// `kubaos-ui`'s `look::concentric` and `look::nested`. The two corners share a centre, the inner
// radius the outer's less the inset; a pill in a pill is a pill; an inset that reaches the outer
// radius leaves nothing to round, and the nesting is refused.

import { length } from "./tokens.ts";

/** A corner: a radius role's, or one nested from it. */
export interface Corner {
  /** The corner as CSS, from the tokens: `var(--radius-control)`, or a role less a step. */
  readonly css: string;
  /** Its radius in logical pixels, or null for a pill's. */
  readonly px: number | null;
  /** How it is worked out, as the page says it. */
  readonly words: string;
}

/** A radius role's corner: `small`, `control`, `surface` or `full`. */
export function role(name: "small" | "control" | "surface" | "full"): Corner {
  return {
    css: `var(--radius-${name})`,
    px: name === "full" ? null : length(`radius-${name}`),
    words: `radius.${name}`,
  };
}

/**
 * The corner of a shape filling one with corner `outer` less the spacing step `inset` on every
 * side, or null where the inset reaches the outer radius.
 */
export function concentric(outer: Corner, inset: string): Corner | null {
  if (outer.px === null) return outer;
  const by = length(`space-${inset}`);
  if (by >= outer.px) return null;
  return { css: `calc(${outer.css} - var(--space-${inset}))`, px: outer.px - by, words: `${outer.words} − space.${inset}` };
}

/** `concentric` for a nesting that must resolve: one that doesn't fails the page as it is drawn. */
export function nested(outer: Corner, inset: string): Corner {
  const inner = concentric(outer, inset);
  if (!inner) throw new Error(`space.${inset} reaches ${outer.words}: the corners can't be concentric`);
  return inner;
}
