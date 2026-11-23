// The shell's glass: the one material of the bar, its cards and every surface floating over the
// desktop (`glass.css` draws it). A surface is glass by this module alone, never by its own fill,
// edge or shadow, so the glass setting and the day's ring reach every one of them.
//
// The edge light is the bar's answer to the pointer: under an item that opens a card or a menu
// (`opens`), a stretch of the Tabs' rail along the glass's lower edge, the item's width and
// `size.gap` either side. Hovered it shows dimmer, `primary` at `opacity.track_lit`, fading in over
// `short` on `standard` and travelling from item to item the same way; pressed it goes to full
// brightness while the item's content dips to `scale.press_dip` over `micro`; while the item's card
// is open the screen's own light lies there (`sheet.ts`). Near a rounded end it stays straight and
// stops where the curve begins. The workspaces and the Command menu button take none.

import "./glass.css";
import { durationMs, length } from "./tokens.ts";
import { settings } from "../site/settings.ts";

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

/** Marks `item` as one that opens a card or a menu: the edge light answers the pointer on it. */
export function opens<T extends HTMLElement>(item: T): T {
  item.dataset.opens = "";
  return item;
}

/** Where the light under `item` runs, in `host`'s own pixels: its ends and the line it lies on. */
export interface Stretch {
  readonly from: number;
  readonly to: number;
  /** The glass's lower edge, where the light's lower edge lies. */
  readonly edge: number;
}

/**
 * The stretch under `item` in `host`'s pixels, whatever zoom the page is drawn at: the item's width
 * and `gap` either side, stopped where the rounded ends of its glass begin to curve. Null for an
 * item on no glass.
 */
export function stretchUnder(item: Element, host: HTMLElement, gap: number): Stretch | null {
  const surface = item.closest(".k-glass");
  if (!surface) return null;
  const outer = host.getBoundingClientRect();
  const zoom = host.offsetWidth ? outer.width / host.offsetWidth : 1;
  const own = item.getBoundingClientRect();
  const round = surface.getBoundingClientRect();
  const radius = Number.parseFloat(getComputedStyle(surface).borderTopLeftRadius) || 0;
  const curve = Math.min(radius * zoom, round.height / 2);
  const from = Math.max(own.left - gap * zoom, round.left + curve);
  const to = Math.min(own.right + gap * zoom, round.right - curve);
  return { from: (from - outer.left) / zoom, to: (Math.max(from, to) - outer.left) / zoom, edge: (round.bottom - outer.top) / zoom };
}

/** The light's place along `host`, as the custom properties `glass.css` draws it from. */
function place(light: HTMLElement, at: Stretch): void {
  light.style.setProperty("--rail-from", `${at.from}px`);
  light.style.setProperty("--rail-width", `${at.to - at.from}px`);
  light.style.setProperty("--rail-edge", `${at.edge}px`);
}

/**
 * The pointer's edge light over every item marked `opens` inside `host`, a positioned element round
 * the bar's glass: one light, which comes in under the item the pointer is on, travels to the next
 * one, goes to full brightness while the item is pressed, and goes when the pointer leaves. An
 * item whose card is open (`data-card-open`) already has its light and takes none. A pinned item
 * (`data-pinned` hover or pressed, on it or a part of it) shows its light still, once it is laid out.
 */
export function edgeRail(host: HTMLElement): HTMLElement {
  const light = edgeLight();
  light.classList.add("k-edge-hover");
  host.append(light);
  // `size.gap` in the page's CSS pixels, at the reader's scale.
  const gap = () => length("size-gap") * settings().scale;
  let under: HTMLElement | null = null;
  /** When the light last began to go: until it has, the next item's light travels from it. */
  let went = Number.NEGATIVE_INFINITY;

  const show = (item: HTMLElement | null) => {
    if (item === under) return;
    const shown = light.hasAttribute("data-shown") || performance.now() - went < durationMs("short") / settings().speed;
    under = item;
    if (!item) {
      if (light.hasAttribute("data-shown")) went = performance.now();
      delete light.dataset.shown;
      delete light.dataset.pressed;
      return;
    }
    const at = stretchUnder(item, host, gap());
    if (!at) return;
    // From nothing it fades in where the item is; from another item, or one it is still leaving,
    // it travels there.
    light.toggleAttribute("data-travel", shown);
    place(light, at);
    light.dataset.shown = "";
  };
  const itemOf = (target: EventTarget | null) => {
    const item = target instanceof Element ? target.closest<HTMLElement>("[data-opens]") : null;
    return item && host.contains(item) && !item.hasAttribute("data-card-open") ? item : null;
  };
  host.addEventListener("pointerover", (event) => show(itemOf(event.target)));
  host.addEventListener("pointerleave", () => show(null));
  host.addEventListener("pointerdown", (event) => {
    const item = itemOf(event.target);
    if (!item || event.button !== 0) return;
    show(item);
    light.dataset.pressed = "";
  });
  const release = () => delete light.dataset.pressed;
  host.addEventListener("pointerup", release);
  host.addEventListener("pointercancel", release);
  // A press that opens the item's card hands the light to the screen's, lit under the open card.
  host.addEventListener("click", () => requestAnimationFrame(() => {
    if (under?.hasAttribute("data-card-open")) show(null);
  }));

  // A pinned item shows its light as a still once it is on the page and laid out.
  const settle = (tries: number) => {
    const pinned = host.querySelector<HTMLElement>('[data-opens]:is([data-pinned="hover"], [data-pinned="pressed"], :has([data-pinned="hover"], [data-pinned="pressed"]))');
    if (!pinned) return;
    if (!host.isConnected || host.offsetWidth === 0) {
      if (tries > 0) requestAnimationFrame(() => settle(tries - 1));
      return;
    }
    show(pinned);
    light.removeAttribute("data-travel");
    light.toggleAttribute("data-pressed", pinned.matches('[data-pinned="pressed"], :has([data-pinned="pressed"])'));
  };
  requestAnimationFrame(() => settle(300));
  // The bar's layout moves as it settles (fonts, a card's swap): the light keeps to its item.
  new ResizeObserver(() => {
    if (!under) return;
    const at = stretchUnder(under, host, gap());
    if (at) place(light, at);
  }).observe(host);
  return light;
}
