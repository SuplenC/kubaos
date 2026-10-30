// What every control in the state model shares (`docs/DESIGN.md`, "States"): the states a sample
// is pinned in, as the specimens pin them, and how a pinned state is set on a control. The look of
// each state is `.k-control`'s in `lib.css`.

/**
 * A state shown whatever the pointer and the keyboard do, as the specimens pin them: lit in the
 * look the scene gives it, lit with the signs off whatever the scene, or pressed.
 */
export type Pinned = "lit" | "signs-off" | "pressed";

/** Every state a sample is drawn in: at rest, disabled, or pinned. */
export type Shown = "rest" | "disabled" | Pinned;

/** The options a control takes for a state: disabled, or pinned. */
export function stateOptions(state: Shown): { disabled?: boolean; pinned?: Pinned } {
  if (state === "rest") return {};
  return state === "disabled" ? { disabled: true } : { pinned: state };
}

/** Marks `element`, a control, as pinned in `pinned`, when it is. */
export function pin(element: HTMLElement, pinned: Pinned | undefined): void {
  if (pinned) element.dataset.pinned = pinned;
}
