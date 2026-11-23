// The desktop and the cards floating over it: `kubaos-ui`'s `sheet::sheet`, `KubaOS.Components`'
// `Sheet` (`docs/design/components/sheet.md`), on a screen with no frame round it.
//
// The screen is its wallpaper, the bar along its top in one of three layouts, and the cards. The
// bar and every card are the shell's glass (`glass.ts`), each its own surface over the desktop; the
// desktop never dims under a card. The layouts hold the same items in a row `size.bar_row` tall:
// Bar, a band `size.bar` tall touching the top edge, full width; Floating, one strip
// `size.bar_strip` tall `size.gap` from the top and the sides; Islands, its groups each a strip of
// their own, as far from the edges. Everything else on the desktop keeps `size.gap` from the
// screen's edges, from the bar and from each other.
//
// A card opens `size.gap` under the bar, centred under the bar item that opened it and stopped
// `size.gap` from the screen's sides; one with no item stands at the right, or in the middle for
// the clock's. It is its content's size, its corners `radius.card`, and it grows no further than
// `size.gap` above the screen's bottom, where its content scrolls. Opening, `--reveal` goes 0 to 1
// over `long` on `emphasized_decel`: the card fades in as it settles `space.s8` down into place;
// closing, back over `medium` on `emphasized_accel`. The item under which it opened carries the edge
// light along the bar's lower edge, echoed along the card's top edge between its corners.

import "./sheet.css";
import { edgeLight, glass } from "./glass.ts";
import { probe } from "./probe.ts";
import { durationMs, length, raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** How the bar lies along the screen's top: a band, one floating strip, or a strip per group. */
export type BarLayout = "bar" | "floating" | "islands";

/** The layouts, the setting's default first. */
export const BAR_LAYOUTS: readonly BarLayout[] = ["bar", "floating", "islands"];

export interface ScreenOptions {
  /** Its size, a length's CSS each: a screen of the toolbar's window size, or a sample's room. */
  readonly width: string;
  readonly height: string;
  /** The desktop's wallpaper. */
  readonly wallpaper: string;
  /** What the bar shows, laid along it. */
  readonly bar?: readonly Element[];
  /** How the bar lies: `bar`, the setting's default, unless the sample shows another. */
  readonly layout?: BarLayout;
  /**
   * A dialog's dim over the desktop while something is open over it, `scrim` at `opacity.scrim`,
   * as the power menu asks what to do with the session. A card never dims it.
   */
  readonly dim?: "dialog";
}

/** A screen: its wallpaper, the layer of the cards' glass, and the layer above it. */
export interface Screen {
  readonly element: HTMLElement;
  /** The layer the cards' glass lies in. */
  readonly glass: HTMLElement;
  /** The layer above it: the bar, and each card's content. */
  readonly over: HTMLElement;
}

/**
 * The bar's islands: a row of groups (the bar's left, its clock and its right) takes a strip each,
 * and a bar of loose items takes one strip round them all.
 */
function islands(items: readonly Element[]): Element[] {
  const island = (...held: Element[]) => {
    const element = glass(document.createElement("span"));
    element.className += " k-bar-island";
    element.append(...held);
    return element;
  };
  const only = items.length === 1 ? items[0]! : null;
  if (only?.classList.contains("shell-bar-row")) {
    only.replaceChildren(...[...only.children].map((group) => island(group)));
    return [only];
  }
  return items.length ? [island(...items)] : [];
}

export function screen(options: ScreenOptions): Screen {
  const layout = options.layout ?? "bar";
  const element = document.createElement("span");
  element.className = "k-screen";
  // A whole screen: a motion player draws it as it is laid out.
  element.dataset.whole = "";
  element.dataset.layout = layout;
  element.style.setProperty("--screen-width", options.width);
  element.style.setProperty("--screen-height", options.height);
  const picture = document.createElement("img");
  picture.className = "k-screen-wallpaper";
  picture.src = options.wallpaper;
  picture.alt = "";
  const dim = document.createElement("span");
  dim.className = "k-screen-dim";
  if (options.dim === "dialog") element.dataset.dim = "dialog";
  const glassLayer = document.createElement("span");
  glassLayer.className = "k-screen-glass";
  const over = document.createElement("span");
  over.className = "k-screen-over";
  const bar = document.createElement("span");
  bar.className = "k-bar";
  if (layout === "islands") bar.append(...islands(options.bar ?? []));
  else {
    glass(bar, layout === "bar");
    bar.append(...(options.bar ?? []));
  }
  const light = edgeLight();
  light.classList.add("k-bar-light");
  over.append(bar, light);
  element.append(picture, dim, glassLayer, over);
  return { element, glass: glassLayer, over };
}

/** Where a card stands: under the bar, or above the screen's bottom edge. */
export type Edge = "right" | "top" | "bottom";

export interface SheetOptions {
  /** What it holds, which keeps its own padding: a PanelPage. */
  readonly content: HTMLElement;
  /**
   * Under the bar (`right` and `top`), or `size.gap` above the screen's bottom edge, centred
   * (`bottom`). Under the bar with no `under` item, a `right` card stands at the screen's right and
   * a `top` one in its middle, as the clock's.
   */
  readonly edge: Edge;
  /** The bar item it opened from: it centres under it, and the edge light lies under it. */
  readonly under?: Element;
  /** How far open it is drawn, 0 to 1: a picture of it opening. */
  readonly reveal?: number;
  /** Under the bar at the screen's right, as the popup stack and the bell's peek stand. */
  readonly corner?: boolean;
  /** How far further down it starts (a length's CSS): a stack under another card. */
  readonly start?: string;
  /**
   * Its corners (a length's CSS) when what it holds is not a card's content boxes: concentric with
   * its own controls, their radius and its content's padding. `radius.card` otherwise.
   */
  readonly rounding?: string;
}

/** A card: its glass for the glass layer, and its content's box for the layer above. */
export interface Sheet {
  readonly surface: HTMLElement;
  readonly box: HTMLElement;
  /** Both, which a move of the card animates together. */
  readonly parts: readonly HTMLElement[];
  /** Puts the card under another bar item, or none; it moves there as its settings say. */
  readonly anchor: (item: Element | null) => void;
}

/** What each card does once it is hung on a screen: follows the screen's size and its bar's. */
const placers = new WeakMap<HTMLElement, (on: Screen) => void>();

/** `element`'s left and right edges in `on`'s own pixels, whatever zoom the page is drawn at. */
function edgesIn(element: Element, on: HTMLElement): [number, number] {
  const outer = on.getBoundingClientRect();
  const inner = element.getBoundingClientRect();
  const zoom = on.offsetWidth ? outer.width / on.offsetWidth : 1;
  return [(inner.left - outer.left) / zoom, (inner.right - outer.left) / zoom];
}

export function sheet(options: SheetOptions): Sheet {
  const surface = glass(document.createElement("span"));
  surface.classList.add("k-sheet-surface", "k-sheet-part");
  const box = document.createElement("span");
  box.className = "k-sheet k-sheet-part";
  const rider = document.createElement("span");
  rider.className = "k-sheet-rider";
  rider.append(options.content);
  const echo = edgeLight();
  echo.classList.add("k-sheet-echo");
  box.append(rider, echo);
  const parts = [surface, box];
  const edge = options.edge === "bottom" ? "bottom" : "top";
  for (const part of parts) {
    part.dataset.edge = edge;
    if (options.start !== undefined && edge === "top") part.style.setProperty("--sheet-start", options.start);
    if (options.reveal !== undefined) part.style.setProperty("--reveal", String(options.reveal));
    if (options.rounding !== undefined) part.style.setProperty("--sheet-corner", options.rounding);
  }
  let under: Element | null = options.under ?? null;
  // The card's size is its content's, and its place its item's, which only layout knows.
  const place = () => {
    const room = box.closest<HTMLElement>(".k-screen");
    const width = room?.clientWidth ?? 0;
    const edges = room && under ? edgesIn(under, room) : null;
    const centre = edges ? (edges[0] + edges[1]) / 2 : options.edge === "right" || options.corner ? width : width / 2;
    for (const part of parts) {
      part.style.setProperty("--sheet-span", `${rider.offsetWidth}px`);
      part.style.setProperty("--sheet-depth", `${rider.offsetHeight}px`);
      part.style.setProperty("--card-centre", `${centre}px`);
    }
    if (room && edges) {
      room.style.setProperty("--item-start", `${edges[0]}px`);
      room.style.setProperty("--item-end", `${edges[1]}px`);
    }
    box.toggleAttribute("data-under", edges !== null);
    room?.toggleAttribute("data-lit", edges !== null);
  };
  const watch = new ResizeObserver(place);
  watch.observe(rider);
  placers.set(box, (room) => {
    watch.observe(room.element);
    const bar = room.over.querySelector(".k-bar");
    if (bar) watch.observe(bar);
    place();
  });
  probe(box, { role: "component", name: "Sheet", variant: "", fixed: { width: false, height: false, holds: true } });
  const anchor = (item: Element | null) => {
    under = item;
    place();
  };
  return { surface, box, parts, anchor };
}

/** `sheet` placed on `on`: its glass into the glass layer, its content above it. */
export function hang(on: Screen, each: Sheet): void {
  on.glass.append(each.surface);
  on.over.append(each.box);
  const reveal = each.box.style.getPropertyValue("--reveal");
  if (reveal !== "") on.element.style.setProperty("--light-shown", reveal);
  placers.get(each.box)?.(on);
}

export interface Twin {
  readonly content: HTMLElement;
  /** Its button's place along the bar against its twins': a lower one is further left. */
  readonly side: number;
  /** Puts the keyboard where it lands as the panel opens: its first thing to do. */
  readonly land?: () => void;
  /** The bar item that opens it, which the card stands under. */
  readonly under?: Element;
}

/** Twins sharing one sheet, and what opens, swaps and closes them. */
export interface Twins {
  readonly sheet: Sheet;
  /** Opens `index`, taking the surface over from the twin that is open; again, closes it. */
  readonly toggle: (index: number) => void;
  readonly close: () => void;
  /** Which is open, or null. */
  readonly open: () => number | null;
}

/** The duration token `name` in milliseconds under the speed setting. */
function ms(name: string): number {
  return durationMs(name) / settings().speed;
}

/**
 * A panel's content changing view, the panels' slide and fade: the leaving view slides
 * `space.s24` away from where the arriving one comes from over `micro` on `emphasized_accel`,
 * fading on `standard_accel`; the arriving one slides in that far from its side over `short` on
 * `emphasized_decel`, `duration.swap_lag` later, and fades in on `standard` over
 * `duration.swap_fade` once the leaving one is gone, so the two are never drawn over each other.
 * `toward` is the side the arriving view comes from: 1 the right, a view further in; -1 the left,
 * coming back. Only `translate` and `opacity` move, so the compositor plays it. Each animation
 * holds where it lands; its owner settles them. `leaving` may be none: a view arriving alone.
 */
export function swapViews(leaving: HTMLElement | null, arriving: HTMLElement, toward: 1 | -1): Animation[] {
  const travel = Math.round(length("space-s24"));
  const moves: Animation[] = [];
  if (leaving) {
    moves.push(
      leaving.animate([{ translate: "0px 0px" }, { translate: `${-toward * travel}px 0px` }], { duration: ms("micro"), easing: raw("easing-emphasized_accel"), fill: "forwards" }),
      leaving.animate([{ opacity: 1 }, { opacity: 0 }], { duration: ms("micro"), easing: raw("easing-standard_accel"), fill: "forwards" }),
    );
  }
  moves.push(
    arriving.animate([{ translate: `${toward * travel}px 0px` }, { translate: "0px 0px" }], { duration: ms("short"), delay: ms("swap_lag"), easing: raw("easing-emphasized_decel"), fill: "both" }),
    arriving.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("swap_fade"), delay: ms("micro"), easing: raw("easing-standard"), fill: "both" }),
  );
  return moves;
}

/**
 * Twins under the bar sharing one card: one asked for while another is open takes the card over,
 * which moves and resizes to its place under the new item over `long` on `emphasized_decel`, while
 * the edge light travels with it, its leading edge over `medium` and its trailing one over `long`.
 * The old content slides `space.s24` away from the new one's button and fades over `micro`; the new
 * one slides in from its button's side over `short`, `duration.swap_lag` later, and fades in over
 * `duration.swap_fade` once the old is gone. Asked for again before the new content has begun to
 * show, the old one turns back from where it is over `short`. A third asked for mid-swap takes over
 * from the one arriving, and the one still leaving goes at once. With reduce motion the content
 * swaps at once.
 */
export function twins(on: Screen, panels: readonly Twin[]): Twins {
  const shared = sheet({ content: document.createElement("span"), edge: "right" });
  hang(on, shared);
  for (const part of shared.parts) part.dataset.live = "";
  const rider = shared.box.querySelector<HTMLElement>(".k-sheet-rider")!;
  rider.replaceChildren();
  let current: number | null = null;
  /** The content on its way out in a swap, and the animations moving the two. */
  let leaving: { index: number; element: HTMLElement } | null = null;
  let started = 0;
  let moving: Animation[] = [];
  let travelling = 0;

  /** The card and the light travelling to `index`'s item, toward the side its button is on. */
  function travel(index: number, toward: number): void {
    window.clearTimeout(travelling);
    on.element.dataset.toward = toward < 0 ? "left" : "right";
    on.element.dataset.swapping = "";
    shared.anchor(panels[index]!.under ?? null);
    travelling = window.setTimeout(() => delete on.element.dataset.swapping, ms("long"));
  }

  function settleSwap(): void {
    for (const each of moving) each.cancel();
    moving = [];
    leaving?.element.remove();
    leaving = null;
    for (const child of rider.children) {
      (child as HTMLElement).style.removeProperty("--swap-shift");
      (child as HTMLElement).style.removeProperty("--swap-presence");
    }
  }

  function show(index: number): void {
    const content = panels[index]!.content;
    content.removeAttribute("data-leaving");
    rider.replaceChildren(content);
    current = index;
    shared.anchor(panels[index]!.under ?? null);
    on.element.dataset.open = "";
    for (const part of shared.parts) part.dataset.shown = "";
    // The size is settled before it can ease: a sheet opening takes its content's size at once.
    requestAnimationFrame(() => {
      for (const part of shared.parts) part.dataset.settled = "";
    });
    panels[index]!.land?.();
  }

  function close(): void {
    settleSwap();
    current = null;
    delete on.element.dataset.open;
    for (const part of shared.parts) {
      delete part.dataset.shown;
      delete part.dataset.settled;
    }
  }

  function swap(to: number): void {
    const from = current!;
    const elapsed = performance.now() - started;
    // Asked for again before the new content has begun to show: the old content turns back.
    if (leaving && leaving.index === to && elapsed < ms("micro")) {
      const back = leaving.element;
      for (const each of moving) each.cancel();
      moving = [];
      rider.querySelector(":scope > :not([data-leaving])")?.remove();
      back.removeAttribute("data-leaving");
      leaving = null;
      travel(to, panels[to]!.side - panels[from]!.side);
      current = to;
      const decel = raw("easing-emphasized_decel");
      const standard = raw("easing-standard");
      const style = getComputedStyle(back);
      moving = [
        back.animate([{ translate: style.translate === "none" ? "0px 0px" : style.translate }, { translate: "0px 0px" }], { duration: ms("short"), easing: decel, fill: "forwards" }),
        back.animate([{ opacity: style.opacity }, { opacity: 1 }], { duration: ms("short"), easing: standard, fill: "forwards" }),
      ];
      panels[to]!.land?.();
      return;
    }
    settleSwap();
    const toward = Math.sign(panels[to]!.side - panels[from]!.side);
    travel(to, toward);
    if (settings().reduced) {
      show(to);
      return;
    }
    const old = panels[from]!.content;
    old.dataset.leaving = "";
    const arriving = panels[to]!.content;
    arriving.removeAttribute("data-leaving");
    rider.append(arriving);
    leaving = { index: from, element: old };
    current = to;
    started = performance.now();
    moving = swapViews(old, arriving, toward < 0 ? -1 : 1);
    const last = moving[2]!;
    last.finished.then(() => {
      if (leaving?.element === old) settleSwap();
    }, () => {});
    panels[to]!.land?.();
  }

  function toggle(index: number): void {
    if (current === index && !leaving) close();
    else if (current === null) show(index);
    else if (current !== index || leaving) swap(index);
  }

  on.element.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || current === null) return;
    event.preventDefault();
    close();
  });
  on.element.querySelector(".k-screen-dim")?.addEventListener("click", close);
  return { sheet: shared, toggle, close, open: () => current };
}
