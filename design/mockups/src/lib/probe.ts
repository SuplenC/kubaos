// The specimens' probe on a mocked element: what its box is, as `kubaos-ui`'s probes and the QML
// components' `probe` property say it, so the geometry check reads a mockup as it reads a build
// (`docs/DESIGN.md`, "The geometry check"). `site/frame.ts` walks the probes into a frame.

export type Align = "start" | "centre" | "end";

export interface Fixed {
  readonly width: boolean;
  readonly height: boolean;
  readonly holds?: true;
}

/** A role as the frame's JSON spells it; a text's content and natural size are read when walked. */
export type Role =
  | { readonly role: "layout"; readonly axis: "x" | "y"; readonly gap: number; readonly align: Align }
  | { readonly role: "grid"; readonly columns: number; readonly gap: number; readonly align: Align }
  | { readonly role: "component"; readonly name: string; readonly variant: string; readonly fixed: Fixed }
  | { readonly role: "text" }
  | { readonly role: "icon"; readonly name: string }
  | { readonly role: "spacer"; readonly fill: boolean };

export const FIXED_HEIGHT: Fixed = { width: false, height: true };

/**
 * What a probe declares of its box beyond its role, by name, for `site/frame.ts` and the check
 * (`docs/DESIGN.md`, "What a mockup's frame says"). Each is a rule's exception for that box alone.
 */
export interface Declared {
  /**
   * It floats over the page when placed out of the flow: a bubble or a card anchored to a control
   * but drawn in a layer of its own, held to its own boxes rather than its control's.
   */
  readonly floats?: true;
}

/** Marks `element` as `role`, with what it `declares`. */
export function probe<E extends HTMLElement | SVGElement>(element: E, role: Role, declares: Declared = {}): E {
  element.dataset.probe = JSON.stringify({ ...role, ...declares });
  return element;
}

/**
 * Names the tokens script worked `element`'s box out from, which its styles hold only as a number:
 * a menu between one column and two wide, a side a third of its window up to a column. The site's
 * measures light the box for them (`site/parts/uses.ts`).
 */
export function workedOut<E extends HTMLElement>(element: E, ...tokens: string[]): E {
  element.dataset.tokens = tokens.join(" ");
  return element;
}

/** A component's variant: its words that apply, in order, joined by spaces. */
export function variant(...words: (string | false | undefined)[]): string {
  return words.filter((word): word is string => typeof word === "string" && word !== "").join(" ");
}
