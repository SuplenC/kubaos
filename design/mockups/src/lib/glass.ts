// The shell's glass: the one material of the bar, its cards and every surface floating over the
// desktop (`glass.css` draws it). A surface is glass by this module alone, never by its own fill,
// edge or shadow, so the glass setting and the day's ring reach every one of them.

import "./glass.css";

/** Makes `element` a glass surface; `flush` for one lying along the screen's edges (the band). */
export function glass<T extends HTMLElement>(element: T, flush = false): T {
  element.classList.add("k-glass");
  element.toggleAttribute("data-flush", flush);
  return element;
}

/**
 * Makes `element` an app's window on the glass, at the surface radius with the glass's own edge;
 * `active`, the window the keyboard is in, its lit tube on that edge.
 */
export function glassWindow<T extends HTMLElement>(element: T, active: boolean): T {
  glass(element);
  element.classList.add("k-glass-window");
  element.toggleAttribute("data-active", active);
  return element;
}

/**
 * Makes `element` a part that turns to glass as it opens out of a control, a select's list: its CSS
 * names how far open it is as `--glass-rise` and its shut fill as `--glass-from`.
 */
export function glassRising<T extends HTMLElement>(element: T): T {
  element.classList.add("k-glass-rise");
  return element;
}

/** An edge light: the lit line under a bar item whose card is open, or its echo on the card. */
export function edgeLight(): HTMLElement {
  const element = document.createElement("span");
  element.className = "k-edge-light";
  return element;
}
