// Holding to act: the one hold every held part times (a held menu item, a held `Button`, a held
// `IconButton`). Pressed, `--hold` runs from where it is to 1 over what is left of `duration.hold`,
// on `linear`, and the part acts once it gets there; let go, left or blurred before then, it runs
// back to 0 over `short` on `standard`. `data-holding` is on while it is held. The speed setting
// divides the hold as it does every duration; reduced motion keeps it, since a hold is a timer, not
// a move.

import { durationMs, raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return durationMs(name) / settings().speed;
}

/** What holds a held part: pressing starts its hold, letting go runs it back. */
export interface Holding {
  readonly press: () => void;
  readonly release: () => void;
}

export interface HoldOptions {
  /** Once it has acted, it runs back to empty over `short`: a button that stays. Otherwise it stays full, as a menu item does while its menu closes. */
  readonly empties?: boolean;
}

/**
 * Wires `element` to be held by the pointer: `press` runs `--hold` on to 1 and calls `taken` once
 * it gets there; `release` before then runs it back to 0. The keys are the owner's (a menu holds
 * its current item with Enter), or `heldByKeys`'s.
 */
export function holdable(element: HTMLElement, taken: () => void, options: HoldOptions = {}): Holding {
  let going: Animation | null = null;
  const now = () => Number.parseFloat(getComputedStyle(element).getPropertyValue("--hold")) || 0;
  const settle = (to: number) => {
    going?.cancel();
    going = null;
    element.style.setProperty("--hold", String(to));
  };
  const back = () => {
    const from = now();
    settle(from);
    going = element.animate([{ "--hold": from }, { "--hold": 0 }], { duration: ms("short"), easing: raw("easing-standard"), fill: "forwards" });
    going.finished.then(
      () => settle(0),
      () => {},
    );
  };
  const press = () => {
    if (element.hasAttribute("data-holding") || (element as HTMLButtonElement).disabled) return;
    const from = now();
    settle(from);
    element.dataset.holding = "";
    going = element.animate([{ "--hold": from }, { "--hold": 1 }], { duration: ms("hold") * (1 - from), easing: "linear", fill: "forwards" });
    going.finished.then(
      () => {
        settle(1);
        delete element.dataset.holding;
        taken();
        if (options.empties) back();
      },
      () => {},
    );
  };
  const release = () => {
    if (!element.hasAttribute("data-holding")) return;
    delete element.dataset.holding;
    back();
  };
  element.addEventListener("pointerdown", (event) => {
    if (event.button === 0) press();
  });
  element.addEventListener("pointerup", release);
  element.addEventListener("pointerleave", release);
  element.addEventListener("pointercancel", release);
  return { press, release };
}

/**
 * Holds `element`, a held button, from the keyboard too: Enter or Space kept down is the hold, and
 * the keyboard going elsewhere lets it go. The keys are the hold's: nothing round it takes them, and
 * a press never acts at once.
 */
export function heldByKeys(element: HTMLElement, holding: Holding): void {
  const holds = (event: KeyboardEvent) => event.key === "Enter" || event.key === " ";
  element.addEventListener("keydown", (event) => {
    if (!holds(event)) return;
    event.preventDefault();
    event.stopPropagation();
    if (!event.repeat) holding.press();
  });
  element.addEventListener("keyup", (event) => {
    if (!holds(event)) return;
    event.preventDefault();
    event.stopPropagation();
    holding.release();
  });
  element.addEventListener("blur", holding.release);
}
