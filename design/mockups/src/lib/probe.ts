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

/** Marks `element` as `role`. */
export function probe<E extends HTMLElement | SVGElement>(element: E, role: Role): E {
  element.dataset.probe = JSON.stringify(role);
  return element;
}

/** A component's variant: its words that apply, in order, joined by spaces. */
export function variant(...words: (string | false | undefined)[]): string {
  return words.filter((word): word is string => typeof word === "string" && word !== "").join(" ");
}
