// The ladder a part rises on inside its plane, a window or a shell panel (`[elevation]`,
// `docs/DESIGN.md`, "Elevation"): `lib.css`'s `.k-raised` draws a step's two shadow layers and the
// rim, and lifts the fill one container role at night. Only what floats above the page rises:
// a card is part of it.

/**
 * A step over the plane's surface: 1 a notification tucked under another or a card being dragged,
 * 2 a menu, a popover or a toast, 3 a dialog, 4 a tooltip. A card at rest is flat on the page.
 */
export type Step = 1 | 2 | 3 | 4;

/** Raises `element` to `step`; its CSS names its day fill as `--fill` and the night's as `--fill-night`. */
export function raise<E extends HTMLElement>(element: E, step: Step): E {
  element.classList.add("k-raised");
  element.dataset.step = String(step);
  return element;
}
