// The ladder a part rises on inside its plane, a window or a shell panel (`[elevation]`,
// `docs/DESIGN.md`, "Elevation"): `lib.css`'s `.k-raised` draws a step's two shadow layers and the
// rim, and lifts the fill one container role at night.

/** A step over the plane's surface: 1 a card, 2 a menu or a popover, 3 a dialog, 4 a tooltip. */
export type Step = 1 | 2 | 3 | 4;

/** Raises `element` to `step`; its CSS names its day fill as `--fill` and the night's as `--fill-night`. */
export function raise<E extends HTMLElement>(element: E, step: Step): E {
  element.classList.add("k-raised");
  element.dataset.step = String(step);
  return element;
}
