// A panel growing out of an inner edge of the screen frame: `kubaos-ui`'s `sheet::sheet`,
// `KubaOS.Components`' `Sheet` (`docs/design/components/sheet.md`), on a screen with its frame.
//
// The frame and every sheet are one surface: their fills are opaque `surface` inside one layer
// drawn at `opacity.frame`, so where a sheet meets the frame, or covers its rounded inner corner,
// the glass never doubles up, and the layer casts `shadow.panel` once. A sheet's content sits
// above that layer at full opacity, in a box the same size as its surface.
//
// A sheet hangs from the frame's top edge. Where it meets the edge it curves into it through a
// concave fillet of the surface radius, the frame's own inner corner, and its far corners take
// the surface radius; one flush with the frame's right edge curves into that edge below it
// instead. Its depth is `--reveal` of its content's height: opening, `--reveal` goes 0 to 1 over
// `long` on `emphasized_decel`, the content riding on its far edge so it slides out from under the
// frame, and fading in a beat after the surface moves, from a fifth of the way to three quarters;
// closing, it goes back over `medium` on `emphasized_accel`. The fillets and corners shrink with
// the depth, so a sheet starting at nothing grows smoothly out of a flat edge.

import "./sheet.css";
import { probe } from "./probe.ts";
import { raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

export interface ScreenOptions {
  /** Its size, a length's CSS each: a screen of the toolbar's window size, or a sample's room. */
  readonly width: string;
  readonly height: string;
  /** The wallpaper behind the frame. */
  readonly wallpaper: string;
  /** What the bar shows, laid along it. */
  readonly bar?: readonly Element[];
}

/** A screen: its wallpaper, the frame's glass layer and the layer above it for what sits on the glass. */
export interface Screen {
  readonly element: HTMLElement;
  /** The layer drawn at the frame's glass: the frame, then each sheet's surface. */
  readonly glass: HTMLElement;
  /** The layer above it, at full opacity: the bar's content and each sheet's content. */
  readonly over: HTMLElement;
}

export function screen(options: ScreenOptions): Screen {
  const element = document.createElement("span");
  element.className = "k-screen";
  // A whole screen: a motion player draws it as it is laid out.
  element.dataset.whole = "";
  element.style.setProperty("--screen-width", options.width);
  element.style.setProperty("--screen-height", options.height);
  const picture = document.createElement("img");
  picture.className = "k-screen-wallpaper";
  picture.src = options.wallpaper;
  picture.alt = "";
  const dim = document.createElement("span");
  dim.className = "k-screen-dim";
  const glass = document.createElement("span");
  glass.className = "k-screen-glass";
  const frame = document.createElement("span");
  frame.className = "k-frame";
  glass.append(frame);
  const over = document.createElement("span");
  over.className = "k-screen-over";
  const bar = document.createElement("span");
  bar.className = "k-bar";
  bar.append(...(options.bar ?? []));
  over.append(bar);
  element.append(picture, dim, glass, over);
  return { element, glass, over };
}

/** The frame's edge a sheet grows out of. */
export type Edge = "right" | "top" | "bottom";

export interface SheetOptions {
  /** What it holds, which keeps its own padding: a PanelPage. */
  readonly content: HTMLElement;
  /**
   * The right edge, hanging from the bar, as the system monitor, the notifications and the control
   * center do; or the middle of the top or the bottom edge, as the clock and the wallpaper picker.
   */
  readonly edge: Edge;
  /** How far open it is drawn, 0 to 1: a picture of it opening. */
  readonly reveal?: number;
}

/** A sheet: its surface for the glass layer, and its content's box for the layer above. */
export interface Sheet {
  readonly surface: HTMLElement;
  readonly box: HTMLElement;
  /** Both, which a move of the sheet animates together. */
  readonly parts: readonly HTMLElement[];
}

export function sheet(options: SheetOptions): Sheet {
  const surface = document.createElement("span");
  surface.className = "k-sheet-surface k-sheet-part";
  const box = document.createElement("span");
  box.className = "k-sheet k-sheet-part";
  const rider = document.createElement("span");
  rider.className = "k-sheet-rider";
  rider.append(options.content);
  box.append(rider);
  const parts = [surface, box];
  for (const part of parts) {
    part.dataset.edge = options.edge;
    if (options.reveal !== undefined) part.style.setProperty("--reveal", String(options.reveal));
  }
  // The sheet's depth and its span along the edge are its content's size, which only layout knows,
  // and a right-edge sheet as tall as the frame's room is flush with its bottom edge too.
  const measure = () => {
    const [along, away] = options.edge === "right" ? [rider.offsetHeight, rider.offsetWidth] : [rider.offsetWidth, rider.offsetHeight];
    const holder = box.parentElement;
    // The frame's side, read back as the gap the box keeps from the screen's right edge.
    const side = holder ? holder.clientWidth - box.offsetLeft - box.offsetWidth : 0;
    const room = holder ? holder.clientHeight - box.offsetTop - side : Number.POSITIVE_INFINITY;
    const flush = options.edge === "right" && along >= room - 0.5;
    for (const part of parts) {
      part.style.setProperty("--sheet-depth", `${away}px`);
      part.style.setProperty("--sheet-span", `${along}px`);
      part.toggleAttribute("data-flush-end", flush);
    }
  };
  new ResizeObserver(measure).observe(rider);
  probe(box, { role: "component", name: "Sheet", variant: "", fixed: { width: false, height: false, holds: true } });
  return { surface, box, parts };
}

/** `sheet` placed on `on`: its surface into the glass layer, its content above it. */
export function hang(on: Screen, each: Sheet): void {
  on.glass.append(each.surface);
  on.over.append(each.box);
}

/** One of a set of twin panels: what it holds, and where its button is along the bar. */
export interface Twin {
  readonly content: HTMLElement;
  /** Its button's place along the bar against its twins': a lower one is further left. */
  readonly side: number;
  /** Puts the keyboard where it lands as the panel opens: its first thing to do. */
  readonly land?: () => void;
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
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/**
 * Twins on `on`'s right edge: one asked for while another is open takes the surface over where it
 * is, and only the content changes. The old content slides `space.s24` away from the new one's
 * button and fades over `micro`; the new one slides in from its button's side over `short`,
 * `duration.swap_lag` later, and fades in over `duration.swap_fade` once the old is gone. Asked for
 * again before the new content has begun to show, the old one turns back from where it is over
 * `short`. A third asked for mid-swap takes over from the one arriving, and the one still leaving
 * goes at once. With reduce motion the content swaps at once.
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
      current = to;
      const decel = raw("easing-emphasized_decel");
      const standard = raw("easing-standard");
      const style = getComputedStyle(back);
      moving = [
        back.animate([{ "--swap-shift": style.getPropertyValue("--swap-shift") }, { "--swap-shift": "0" }], { duration: ms("short"), easing: decel, fill: "forwards" }),
        back.animate([{ "--swap-presence": style.getPropertyValue("--swap-presence") }, { "--swap-presence": "1" }], { duration: ms("short"), easing: standard, fill: "forwards" }),
      ];
      panels[to]!.land?.();
      return;
    }
    settleSwap();
    if (settings().reduced) {
      show(to);
      return;
    }
    const toward = Math.sign(panels[to]!.side - panels[from]!.side);
    const old = panels[from]!.content;
    old.dataset.leaving = "";
    const arriving = panels[to]!.content;
    arriving.removeAttribute("data-leaving");
    rider.append(arriving);
    leaving = { index: from, element: old };
    current = to;
    started = performance.now();
    const accel = raw("easing-emphasized_accel");
    const fade = raw("easing-standard_accel");
    const decel = raw("easing-emphasized_decel");
    const standard = raw("easing-standard");
    moving = [
      old.animate([{ "--swap-shift": "0" }, { "--swap-shift": String(-toward) }], { duration: ms("micro"), easing: accel, fill: "forwards" }),
      old.animate([{ "--swap-presence": "1" }, { "--swap-presence": "0" }], { duration: ms("micro"), easing: fade, fill: "forwards" }),
      arriving.animate([{ "--swap-shift": String(toward) }, { "--swap-shift": "0" }], { duration: ms("short"), delay: ms("swap_lag"), easing: decel, fill: "backwards" }),
      arriving.animate([{ "--swap-presence": "0" }, { "--swap-presence": "1" }], { duration: ms("swap_fade"), delay: ms("micro"), easing: standard, fill: "backwards" }),
    ];
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
