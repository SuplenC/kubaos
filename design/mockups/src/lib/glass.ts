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

/** An edge light: the lit line under a bar item whose card is open, or its echo on the card. */
export function edgeLight(): HTMLElement {
  const element = document.createElement("span");
  element.className = "k-edge-light";
  return element;
}
