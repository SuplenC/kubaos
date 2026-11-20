// What every control in the state model shares (`docs/DESIGN.md`, "States"): the states a sample
// is pinned in, as the specimens pin them, and how a pinned state is set on a control. The look of
// each state is `.k-control`'s in `lib.css`.

/**
 * A state shown whatever the pointer and the keyboard do, as the specimens pin them: lit (focused
 * from the keyboard) in the look the scene gives it, lit with the signs off whatever the scene,
 * hovered (the pointer over it, quiet), pressed by the pointer (quiet, a step further), or hovered
 * and focused at once (the neon over the hovered fill).
 */
export type Pinned = "lit" | "signs-off" | "hover" | "pressed" | "focus-hover";

/** Every state a sample is drawn in: at rest, disabled, hovered (as the site's states name it), or pinned. */
export type Shown = "rest" | "disabled" | "hovered" | Pinned;

/** The options a control takes for a state: disabled, or pinned. */
export function stateOptions(state: Shown): { disabled?: boolean; pinned?: Pinned } {
  if (state === "rest") return {};
  if (state === "hovered") return { pinned: "hover" };
  return state === "disabled" ? { disabled: true } : { pinned: state };
}

/** Marks `element`, a control, as pinned in `pinned`, when it is. */
export function pin(element: HTMLElement, pinned: Pinned | undefined): void {
  if (pinned) element.dataset.pinned = pinned;
}

/** A wheel's notch, as the toolkits count it: Qt's `angleDelta` and a browser's line of 100. */
const NOTCH = 100;

/** The least a single wheel event turns for a mouse's notch, not a touchpad's glide. */
const CLICK = 40;

/**
 * Calls `step` with whole notches as the wheel turns sideways over `element` (a tilt, Shift and the
 * wheel, or a touchpad's sideways swipe), right positive. A turn up or down is left to scroll the
 * page. The page's box is zoomed to the display's ratio, which shrinks the deltas the browser
 * reports by that ratio, so they are scaled back first. A mouse's notch steps once at once,
 * whatever its size on this system; a touchpad's stream of small turns adds up to a notch, so it
 * never races.
 */
export function sideways(element: HTMLElement, step: (notches: number) => void): void {
  let turned = 0;
  element.addEventListener(
    "wheel",
    (event) => {
      if (event.deltaX === 0) return;
      event.preventDefault();
      const delta = event.deltaMode === WheelEvent.DOM_DELTA_PIXEL ? event.deltaX * devicePixelRatio : Math.sign(event.deltaX) * NOTCH;
      if (Math.abs(delta) >= CLICK) {
        turned = 0;
        step(Math.sign(delta) * Math.max(1, Math.round(Math.abs(delta) / NOTCH)));
        return;
      }
      turned += delta;
      const notches = Math.trunc(turned / NOTCH);
      if (notches === 0) return;
      turned -= notches * NOTCH;
      step(notches);
    },
    { passive: false },
  );
}
