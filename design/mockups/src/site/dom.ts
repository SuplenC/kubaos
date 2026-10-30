// Small helpers for the page's own words and boxes: the parts the mocked elements sit among.

import { advance, type TypeStyle, typeset } from "../lib/text.ts";

/** An element of `tag` with `className`, holding `children`. */
export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className: string,
  ...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tag);
  element.className = className;
  element.append(...children);
  return element;
}

/** A grid cell holding a mock, which `settle` centres down its row on a whole device pixel. */
export function mockCell(mock: Element): HTMLElement {
  const cell = el("div", "page-cell", mock);
  cell.dataset.mock = "";
  return cell;
}

/**
 * One line of the page's own words in a type style, as wide as a mocked text is, so what follows
 * it on the line starts on a whole device pixel. It carries no probe: it isn't part of the mock.
 */
export function label(words: string, style: TypeStyle, color = "on_surface_variant"): HTMLElement {
  const element = el("span", "k-text", words);
  typeset(element, style);
  element.style.setProperty("--natural", String(advance(style, words)));
  element.style.color = `var(--color-${color})`;
  return element;
}

/** A paragraph of the page's own words, wrapping in the room it has. */
export function prose(words: string, style: TypeStyle = "body", color = "on_surface"): HTMLElement {
  const element = el("p", "page-prose", ...inline(words));
  typeset(element, style);
  element.style.color = `var(--color-${color})`;
  return element;
}

/** `words` with each `backticked` span set as code. */
function inline(words: string): (Node | string)[] {
  return words.split(/(`[^`]+`)/).map((part) => (part.startsWith("`") ? el("code", "page-code", part.slice(1, -1)) : part));
}
