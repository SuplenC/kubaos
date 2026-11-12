// Small helpers for the page's own words and boxes: the parts the mocked elements sit among.

import { advance, centreFor, type TypeStyle, typeset } from "../lib/text.ts";

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
  const cell = sampleBox(el("div", "page-cell", mock));
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
  centreFor(element, style, words);
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

/**
 * Tells the page that what is drawn in `from` changed its layout (a stage picked another sample, a
 * player another move), so what script lays out is laid out again (`site/settle.ts`).
 */
export function relayout(from: Element): void {
  from.dispatchEvent(new Event(RELAYOUT, { bubbles: true }));
}

export const RELAYOUT = "page-relayout";

/** Each sample drawn scaled down, by its inner box: the box that holds it and by how much. */
const scaled = new WeakMap<Element, { readonly outer: HTMLElement; readonly by: number }>();

const growing = new ResizeObserver((entries) => {
  for (const { target } of entries) {
    const fit = scaled.get(target);
    if (!fit || !target.isConnected) {
      growing.unobserve(target);
      continue;
    }
    fit.outer.style.height = `${Math.floor((target as HTMLElement).offsetHeight * fit.by)}px`;
  }
});

/**
 * Keeps `outer` as tall as `inner` drawn at `by`, for as long as it is shown. A transform takes no
 * room, so `outer` holds it; a sample that grows while shown (a log opening under its buttons)
 * grows the box with it rather than spilling over the page under it. `by` of 1 lets it go.
 * `height`, `inner`'s own, read with the other boxes, saves a layout.
 */
export function holdScaled(outer: HTMLElement, inner: HTMLElement, by: number, height?: number): void {
  if (by === 1) {
    scaled.delete(inner);
    growing.unobserve(inner);
    return;
  }
  scaled.set(inner, { outer, by });
  outer.style.height = `${Math.floor((height ?? inner.offsetHeight) * by)}px`;
  growing.observe(inner);
}

// Every sample's box is `data-away` while none of it is in the viewport, which holds its endless
// loops (`site.css`): a page scrolled past its samples asks for no frames.
const seen = new IntersectionObserver((entries) => {
  for (const entry of entries) (entry.target as HTMLElement).toggleAttribute("data-away", !entry.isIntersecting);
});

/**
 * Marks `box` as holding a live sample: a wheel over it scrolls the page until the reader clicks
 * into it (`site/scroll.ts`), and its endless loops hold while it is out of view.
 */
export function sampleBox<E extends HTMLElement>(box: E): E {
  box.dataset.sample = "";
  seen.observe(box);
  return box;
}
