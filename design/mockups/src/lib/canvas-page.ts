// A canvas of objects drawn to scale, with the selected one's settings: the template `CanvasPage`
// (`docs/design/components/canvas-page.md`), a region inside a SidebarPage page, Settings ›
// Displays' screens. Its look is `canvas-page.css`.
//
// The canvas is a well in `surface_container_lowest` at the content radius, its sunken hairline
// round it and a faint dot on every `space.s16` of its grid. Its toolbar is its top band, `space.s12`
// in: the current desk as a chip, then its actions at the far end; its key hints are its bottom
// band. Between them the objects are drawn to scale, fitted to the room less `space.s24` all round,
// and fitted again on every resize, frame by frame. Dragging one tracks the pointer, its edges and
// centre snapping to its neighbours' with a `primary` guide while a snap holds.
//
// On a page under `size.canvas_wide` the canvas spans the page, `aspect.canvas` of its width tall,
// capped so the settings' heading under it stays in view; from it the settings go into a column
// `size.canvas_settings` wide beside the canvas, which takes all the height left.

import "./canvas-page.css";
import { button } from "./button.ts";
import { chip } from "./chip.ts";
import { focusRing } from "./focus.ts";
import { keyHints } from "./keycap.ts";
import { fill, filling, row, section, stack } from "./layout.ts";
import { scaleOf } from "./list.ts";
import { probe } from "./probe.ts";
import { scroll } from "./scroll.ts";
import { type RegionHost, settingsWell } from "./sidebar-page.ts";
import { advance, text } from "./text.ts";
import { length, raw } from "./tokens.ts";
import type { ListRowSpec } from "./list.ts";

/** An object on the canvas: a screen, in logical pixels, where it sits in the arrangement. */
export interface Screen {
  readonly id: string;
  readonly name: string;
  /** The number Identify shows on it, and the canvas on its box. */
  readonly number: number;
  readonly width: number;
  readonly height: number;
  /** The screen's scale: its size in logical pixels is its mode over it. */
  readonly scale: number;
  x: number;
  y: number;
}

export interface CanvasOptions {
  readonly screens: readonly Screen[];
  /** The selected object, by index. */
  readonly selected?: number;
  /** The desk the arrangement is for, on the toolbar's chip. */
  readonly desk: string;
  /** The canvas pinned as with the keyboard on it: a still sample. */
  readonly keyboard?: boolean;
  /** A drag held mid-way for a still sample: the object at index `drag` held with its guides. */
  readonly held?: { readonly index: number; readonly x: number; readonly y: number };
  /** What the selection changing does: the settings under the canvas follow it. */
  readonly onSelect?: (screen: Screen) => void;
  /** Called every frame an object moves, so its numbers follow it live. */
  readonly onMove?: (screen: Screen) => void;
  /** Enter on an object: its settings. */
  readonly onEnter?: () => void;
}

/** The canvas, and what the page drives it with. */
export interface Canvas {
  readonly element: HTMLElement;
  readonly screens: () => readonly Screen[];
  readonly selected: () => Screen | undefined;
  /** A screen plugged in: it fades in and the arrangement fits again. */
  readonly plug: (screen: Screen) => void;
  /** A screen unplugged: it fades out and the arrangement fits again. */
  readonly unplug: (id: string) => void;
}

/** An edge or the centre of an object along one axis, in logical pixels. */
function marks(start: number, size: number): number[] {
  return [start, start + size / 2, start + size];
}

/** The duration token `name` in milliseconds, as the page plays it. */
function motionMs(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`));
}

/** The numbers an object shows under its own: its size in logical pixels. */
export function sizeWords(screen: Screen): string {
  return `${Math.round(screen.width / screen.scale)} × ${Math.round(screen.height / screen.scale)}`;
}

export function canvas(options: CanvasOptions): Canvas {
  let screens = options.screens.map((each) => ({ ...each }));
  let picked = options.selected ?? 0;

  const element = document.createElement("span");
  element.className = "k-cv k-sunken";
  element.tabIndex = 0;
  element.setAttribute("role", "listbox");
  element.setAttribute("aria-label", "Screens");
  const edge = document.createElement("span");
  edge.className = "k-cv-edge";

  const desk = chip({ label: options.desk, icon: "desktop_windows" });
  const bar = row("s8", "centre", desk, fill(), button({ label: "Identify", kind: "Text", size: "Sm" }), button({ label: "Mirror", kind: "Text", size: "Sm" }));
  bar.classList.add("k-cv-band");
  const stage = document.createElement("span");
  stage.className = "k-cv-stage";
  const hints = keyHints([
    ["← →", "Pick"],
    ["Shift+← →", "Move to the next snap"],
    ["Enter", "Its settings"],
  ]);
  const foot = row(0, "centre", hints);
  foot.classList.add("k-cv-band");
  const guideX = document.createElement("span");
  guideX.className = "k-cv-guide";
  guideX.dataset.axis = "x";
  const guideY = document.createElement("span");
  guideY.className = "k-cv-guide";
  guideY.dataset.axis = "y";
  stage.append(guideX, guideY);
  element.append(bar, stage, foot, edge);
  focusRing(element, "content", "accent", false);
  if (options.keyboard) element.dataset.pinned = "lit";

  const boxes = new Map<string, HTMLElement>();
  /** The fit: stage pixels per logical pixel, and where the arrangement's corner lands. */
  let fit = { factor: 1, left: 0, top: 0, minX: 0, minY: 0 };
  let dragging = false;

  function box(screen: Screen): HTMLElement {
    const known = boxes.get(screen.id);
    if (known) return known;
    const made = document.createElement("span");
    made.className = "k-cv-screen k-control";
    made.setAttribute("role", "option");
    made.dataset.n = String(screen.number);
    const number = text(String(screen.number), "headline");
    number.classList.add("k-cv-number");
    const size = text(sizeWords(screen), "mono", "on_surface_variant");
    size.classList.add("k-cv-size");
    made.append(stack(0, "centre", number, size));
    focusRing(made, "control", "accent", false);
    made.addEventListener("pointerdown", (event) => drag(event, screen.id));
    boxes.set(screen.id, made);
    stage.append(made);
    return probe(made, { role: "component", name: "CanvasObject", variant: "", fixed: { width: false, height: false } });
  }

  /** The arrangement fitted to the stage less `space.s24` all round, every box on whole device pixels. */
  function refit(): void {
    if (dragging) return;
    const scale = scaleOf(element);
    const margin = length("space-s24") * scale;
    const room = { width: stage.clientWidth - 2 * margin, height: stage.clientHeight - 2 * margin };
    if (screens.length === 0 || room.width <= 0 || room.height <= 0) return place();
    const logical = screens.map((s) => ({ x: s.x, y: s.y, w: s.width / s.scale, h: s.height / s.scale }));
    const minX = Math.min(...logical.map((s) => s.x));
    const minY = Math.min(...logical.map((s) => s.y));
    const width = Math.max(...logical.map((s) => s.x + s.w)) - minX;
    const height = Math.max(...logical.map((s) => s.y + s.h)) - minY;
    const factor = Math.min(room.width / width, room.height / height);
    fit = {
      factor,
      minX,
      minY,
      left: Math.round(margin + (room.width - width * factor) / 2),
      top: Math.round(margin + (room.height - height * factor) / 2),
    };
    place();
  }

  /** Each object's box from the fit: its edges rounded, so two that touch share a device pixel. */
  function place(): void {
    for (const [index, screen] of screens.entries()) {
      const made = box(screen);
      const w = screen.width / screen.scale;
      const h = screen.height / screen.scale;
      const left = Math.round(fit.left + (screen.x - fit.minX) * fit.factor);
      const top = Math.round(fit.top + (screen.y - fit.minY) * fit.factor);
      const right = Math.round(fit.left + (screen.x + w - fit.minX) * fit.factor);
      const bottom = Math.round(fit.top + (screen.y + h - fit.minY) * fit.factor);
      made.style.left = `${left}px`;
      made.style.top = `${top}px`;
      made.style.width = `${right - left}px`;
      made.style.height = `${bottom - top}px`;
      made.setAttribute("aria-selected", String(index === picked));
      // An object too small for its size shows its number alone.
      const scale = scaleOf(element);
      const words = made.querySelector<HTMLElement>(".k-cv-size");
      const room = (advance("mono", sizeWords(screen)) + 2 * length("space-s8")) * scale;
      const tall = (length("type-headline-line") + length("type-mono-line") + 2 * length("space-s8")) * scale;
      if (words) words.hidden = right - left < room || bottom - top < tall;
    }
  }

  // Every object has its box from the start, so a motion player finds its parts; the fit follows
  // once the stage is laid out.
  place();
  new ResizeObserver(refit).observe(stage);

  /** Where a snap holds along one axis: the move to it, and the guide's place in logical pixels. */
  function snap(mine: number[], others: number[][], reach: number): { by: number; at: number } | null {
    let best: { by: number; at: number } | null = null;
    for (const theirs of others) {
      for (const a of mine) {
        for (const b of theirs) {
          const by = b - a;
          if (Math.abs(by) <= reach && (!best || Math.abs(by) < Math.abs(best.by))) best = { by, at: b };
        }
      }
    }
    return best;
  }

  /** Moves object `index` to `x`, `y`, snapped to its neighbours, the guides shown while a snap holds. */
  function moveTo(index: number, x: number, y: number): void {
    const screen = screens[index]!;
    const w = screen.width / screen.scale;
    const h = screen.height / screen.scale;
    const others = screens.filter((_, at) => at !== index);
    const reach = (length("space-s8") * scaleOf(element)) / fit.factor;
    const sx = snap(marks(x, w), others.map((o) => marks(o.x, o.width / o.scale)), reach);
    const sy = snap(marks(y, h), others.map((o) => marks(o.y, o.height / o.scale)), reach);
    screen.x = Math.round(x + (sx?.by ?? 0));
    screen.y = Math.round(y + (sy?.by ?? 0));
    guide(guideX, sx?.at, "x");
    guide(guideY, sy?.at, "y");
    place();
    options.onMove?.(screen);
  }

  function guide(line: HTMLElement, at: number | undefined, axis: "x" | "y"): void {
    line.toggleAttribute("data-shown", at !== undefined);
    if (at === undefined) return;
    const offset = axis === "x" ? fit.left + (at - fit.minX) * fit.factor : fit.top + (at - fit.minY) * fit.factor;
    line.style.setProperty("--guide", `${Math.round(offset)}px`);
  }

  /** Picks object `index`: its tube lights and the settings under the canvas follow. */
  function select(index: number): void {
    picked = Math.max(0, Math.min(screens.length - 1, index));
    place();
    const screen = screens[picked];
    if (screen) options.onSelect?.(screen);
  }

  // The page's box is zoomed to the display's ratio, so the pointer's viewport pixels are brought
  // back to the stage's own by its layout width over its drawn width, and then to logical pixels
  // of the arrangement by the fit.
  function drag(event: PointerEvent, id: string): void {
    const index = screens.findIndex((each) => each.id === id);
    if (index < 0 || event.button !== 0) return;
    event.preventDefault();
    element.focus({ preventScroll: true });
    if (index !== picked) select(index);
    const made = box(screens[index]!);
    made.setPointerCapture(event.pointerId);
    dragging = true;
    element.dataset.dragging = "";
    const ratio = stage.offsetWidth / Math.max(1, stage.getBoundingClientRect().width);
    const from = { x: event.clientX, y: event.clientY, sx: screens[index]!.x, sy: screens[index]!.y };
    const follow = (moved: PointerEvent) => {
      const dx = ((moved.clientX - from.x) * ratio) / fit.factor;
      const dy = ((moved.clientY - from.y) * ratio) / fit.factor;
      moveTo(index, from.sx + dx, from.sy + dy);
    };
    const end = () => {
      made.removeEventListener("pointermove", follow);
      dragging = false;
      delete element.dataset.dragging;
      guide(guideX, undefined, "x");
      guide(guideY, undefined, "y");
      glideRefit();
    };
    made.addEventListener("pointermove", follow);
    made.addEventListener("pointerup", end, { once: true });
    made.addEventListener("pointercancel", end, { once: true });
  }

  /** The arrangement fitted again after it changed, its objects gliding to their new boxes. */
  function glideRefit(): void {
    element.dataset.refit = "";
    refit();
    setTimeout(() => delete element.dataset.refit, motionMs("medium"));
  }

  /** The next place along an axis where an edge or the centre meets a neighbour's, past where it is. */
  function nextSnap(index: number, axis: "x" | "y", toward: 1 | -1): number | null {
    const screen = screens[index]!;
    const at = axis === "x" ? screen.x : screen.y;
    const size = axis === "x" ? screen.width / screen.scale : screen.height / screen.scale;
    let best: number | null = null;
    for (const [other, each] of screens.entries()) {
      if (other === index) continue;
      const theirs = axis === "x" ? marks(each.x, each.width / each.scale) : marks(each.y, each.height / each.scale);
      for (const a of marks(0, size)) {
        for (const b of theirs) {
          const to = Math.round(b - a);
          if ((to - at) * toward <= 0) continue;
          if (best === null || Math.abs(to - at) < Math.abs(best - at)) best = to;
        }
      }
    }
    return best;
  }

  element.addEventListener("keydown", (event) => {
    const key = event.key;
    const lower = key.toLowerCase();
    const shift = event.shiftKey;
    const vertical = key === "ArrowUp" || key === "ArrowDown" || lower === "j" || lower === "k";
    const back = key === "ArrowLeft" || key === "ArrowUp" || lower === "h" || lower === "k";
    const moves = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(key) || ["h", "j", "k", "l"].includes(lower);
    // H, J, K and L are the Neovim keys with Shift held.
    const upper = key.length === 1 && key !== lower;
    if (moves && (shift || upper)) {
      const to = nextSnap(picked, vertical ? "y" : "x", back ? -1 : 1);
      if (to === null) return event.preventDefault();
      const screen = screens[picked]!;
      if (vertical) screen.y = to;
      else screen.x = to;
      options.onMove?.(screen);
      glideRefit();
    } else if (!vertical && moves) {
      // The objects in reading order, left to right and top to bottom.
      const order = screens.map((_, at) => at).sort((a, b) => screens[a]!.x - screens[b]!.x || screens[a]!.y - screens[b]!.y);
      const at = order.indexOf(picked);
      select(order[(at + (back ? -1 : 1) + order.length) % order.length]!);
    } else if (key === "Enter") {
      if (!event.repeat) options.onEnter?.();
    } else return;
    event.preventDefault();
    event.stopPropagation();
  });

  if (options.held) {
    const held = options.held;
    requestAnimationFrame(() => {
      refit();
      dragging = true;
      moveTo(held.index, held.x, held.y);
      dragging = false;
    });
  }

  probe(element, { role: "component", name: "Canvas", variant: "", fixed: { width: false, height: false, holds: true } });
  return {
    element,
    screens: () => screens,
    selected: () => screens[picked],
    plug: (screen) => {
      screens = [...screens, { ...screen }];
      const made = box(screen);
      made.dataset.arriving = "";
      glideRefit();
      requestAnimationFrame(() => requestAnimationFrame(() => delete made.dataset.arriving));
    },
    unplug: (id) => {
      const made = boxes.get(id);
      const index = screens.findIndex((each) => each.id === id);
      if (!made || index < 0) return;
      made.dataset.leaving = "";
      screens = screens.filter((each) => each.id !== id);
      boxes.delete(id);
      if (picked >= screens.length) picked = Math.max(0, screens.length - 1);
      setTimeout(() => made.remove(), motionMs("medium"));
      glideRefit();
      const screen = screens[picked];
      if (screen) options.onSelect?.(screen);
    },
  };
}

export interface RegionOptions extends Omit<CanvasOptions, "onSelect" | "onMove" | "onEnter"> {
  /** The selected object's settings, its first sub-section: rows drawn from it. */
  readonly settings: (screen: Screen) => readonly (readonly ListRowSpec[])[];
  /** The sub-sections after the selected object's, which don't follow the selection. */
  readonly after?: readonly { readonly title: string; readonly body?: string; readonly wells: readonly (readonly ListRowSpec[])[] }[];
  /** The page's width it lays out for, held: a still sample of either layout. */
  readonly wide?: boolean;
}

/** The numbers line under the selected object's name: its size, scale and where it sits. */
export function numbers(screen: Screen): string {
  return `${sizeWords(screen)} · ${screen.width} × ${screen.height} at ${screen.scale}× · at ${screen.x}, ${screen.y}`;
}

/**
 * CanvasPage in a SidebarPage page: the canvas, then the selected object's settings, below it or
 * beside it by the page's width. Returns the region the page draws under its header.
 */
export function canvasRegion(host: RegionHost, options: RegionOptions): HTMLElement {
  const region = document.createElement("span");
  region.className = "k-cvp";
  const column = stack("s32", "start");
  column.classList.add("k-cvp-settings");
  const scrolled = scroll(column, { height: "100%", above: 0, below: 0, noGutter: true });
  scrolled.box.classList.add("k-cvp-column");

  const settingsOf = (screen: Screen) => {
    const box = section(screen.name, numbers(screen), ...options.settings(screen).map(settingsWell));
    box.classList.add("k-cvp-selected");
    return filling(box);
  };
  const after = (options.after ?? []).map((each) => filling(section(each.title, each.body, ...each.wells.map(settingsWell))));

  const drawn = canvas({
    ...options,
    onSelect: (screen) => {
      column.firstElementChild?.replaceWith(settingsOf(screen));
      host.rename(0, screen.name);
      host.refresh();
    },
    onMove: (screen) => {
      // The section's second line of words: its numbers, under its name.
      const line = column.querySelectorAll(".k-cvp-selected .k-text")[1];
      if (line) line.textContent = numbers(screen);
    },
    onEnter: () => host.enter(0),
  });
  const first = drawn.selected();
  column.append(...(first ? [settingsOf(first)] : []), ...after);
  if (first) host.rename(0, first.name);
  region.append(drawn.element, scrolled.box);
  region.dataset.layout = options.wide ? "wide" : "narrow";

  /** Lays the region out for the page's width: below the canvas, or the column beside it. */
  const layout = () => {
    const pageBox = region.closest<HTMLElement>(".k-page");
    const pane = region.closest<HTMLElement>(".k-scroll-view");
    if (!pageBox || !pane) return;
    const scale = scaleOf(region);
    const wide = options.wide ?? pageBox.clientWidth / scale >= length("size-canvas_wide");
    region.dataset.layout = wide ? "wide" : "narrow";
    const paneBox = pane.getBoundingClientRect();
    const ratio = pane.offsetHeight / Math.max(1, paneBox.height);
    const top = (region.getBoundingClientRect().top - paneBox.top) * ratio + pane.scrollTop;
    const foot = length("space-s24") * scale;
    if (wide) {
      const height = Math.max(0, Math.round(pane.clientHeight - top - foot));
      drawn.element.style.height = `${height}px`;
      scrolled.box.style.setProperty("--height", `${height}px`);
      host.spyOn(scrolled.view, 0);
    } else {
      const [across, down] = raw("aspect-canvas").split("/").map((n) => Number.parseFloat(n)) as [number, number];
      const shaped = Math.round((drawn.element.clientWidth * down) / across);
      const heading = (length("space-s32") + length("type-heading-line")) * scale;
      const cap = Math.round(pane.clientHeight - top - heading - foot);
      drawn.element.style.height = `${Math.max(0, Math.min(shaped, cap))}px`;
      scrolled.box.style.setProperty("--height", "auto");
      host.spyOn(null);
    }
  };
  const observer = new ResizeObserver(() => requestAnimationFrame(layout));
  requestAnimationFrame(() => {
    const pane = region.closest(".k-scroll-view");
    if (pane) observer.observe(pane);
    observer.observe(region);
    layout();
  });
  (region as HTMLElement & { canvas?: Canvas }).canvas = drawn;
  return probe(region, { role: "component", name: "CanvasPage", variant: "", fixed: { width: false, height: false, holds: true } });
}

/** The canvas a region holds, for the page's own controls: plugging a screen in and out. */
export function canvasOf(region: Element | null): Canvas | undefined {
  return (region as (HTMLElement & { canvas?: Canvas }) | null)?.canvas;
}
