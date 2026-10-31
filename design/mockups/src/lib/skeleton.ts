// The shape of what is loading, held in its place: `kubaos-ui`'s `skeleton::skeleton`,
// `KubaOS.Components`' `Skeleton` (`docs/design/components/skeleton.md`). Faint glass of the
// accent that breathes while it shows, warming up as a sign's gas does (`.k-skeleton` in
// `lib.css`).

import type { TypeStyle } from "./text.ts";
import { probe } from "./probe.ts";

/** What a skeleton stands in for. */
export type Bone =
  | { readonly bone: "line"; readonly style: TypeStyle }
  | { readonly bone: "circle"; readonly size: "s16" | "s20" | "s24" | "s32" | "s48" }
  | { readonly bone: "block"; readonly size: string; readonly radius: "small" | "control" | "surface" | "full" };

export interface SkeletonOptions {
  /** Held still whatever happens: a sample a motion player moves, or a wait that failed. */
  readonly still?: boolean;
}

/**
 * A skeleton of `bone`: a line as wide as its room and its style's line tall at the small radius,
 * a circle an icon size across, or a block as wide as its room, `size` (a size token's name) tall.
 */
export function skeleton(bone: Bone, options: SkeletonOptions = {}): HTMLElement {
  const shape = document.createElement("span");
  shape.className = "k-skeleton";
  shape.dataset.bone = bone.bone;
  shape.toggleAttribute("data-still", options.still ?? false);
  switch (bone.bone) {
    case "line":
      shape.style.setProperty("--height", `var(--type-${bone.style}-line)`);
      shape.style.setProperty("--corner", "var(--radius-small)");
      break;
    case "circle":
      shape.style.setProperty("--height", `var(--icon-${bone.size})`);
      shape.style.setProperty("--corner", "var(--radius-full)");
      break;
    case "block":
      shape.style.setProperty("--height", `var(--size-${bone.size})`);
      shape.style.setProperty("--corner", `var(--radius-${bone.radius})`);
      break;
  }
  return probe(shape, { role: "component", name: "Skeleton", variant: bone.bone, fixed: { width: false, height: false } });
}
