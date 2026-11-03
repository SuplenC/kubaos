// The StepRailPage template: a step rail beside a page with a footer band, and how it moves between
// its pages (`docs/design/components/step-rail-page.md`). One template with two hosts: in an app's
// window (the updater) the rail and the page fill the window; in Kōbō they are its glass panel over
// the wallpaper and the scrim. Its look is `step-rail.css`.
//
// The rail is `size.rail` wide, `space.s16` in above and below and `space.s12` at its sides, so its
// steps' corners are concentric with its own: the brand or the title, `space.s20`, the steps back
// to back across its width (`Step`, each a mark at its end), scrolling where they don't fit, then
// `space.s16` and at its foot the key hints and, in Kōbō, the power button and the battery. A wire
// runs down the steps' symbols, lit down to the current one; each symbol is a bead on it. The page
// beside it is a clear `Page`: its body, then `space.s16` above the footer band, which has no
// hairline: Back on the left, the one Filled button on the right, and a note just before it.

import "./step-rail.css";
import { button } from "./button.ts";
import type { Pinned } from "./control.ts";
import { icon } from "./icon.ts";
import { keyHints, type KeyHint } from "./keycap.ts";
import { fill, filling, fillingHeight, gap, inset, page, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { scroll } from "./scroll.ts";
import { step, type StepOptions, type StepState } from "./step.ts";
import { text } from "./text.ts";
import { cubic, length, raw } from "./tokens.ts";
import { palette } from "../site/recolour.ts";
import { settings } from "../site/settings.ts";
import { THEMES } from "../site/themes.ts";

/** Where the template is drawn: an app's window, or Kōbō's glass panel over its backdrop. */
export type RailHost = "window" | "kobo";

/** One page of the flow: its step in the rail and what the page holds. */
export interface RailPage {
  /** Its step: a title and a symbol; where it stands comes from the flow. */
  readonly step: Pick<StepOptions, "title" | "icon">;
  /** The page's body, drawn each time the page shows. */
  readonly body: () => HTMLElement;
  /** Continue's words: Continue, Skip, Install. */
  readonly next?: string;
  /** Why Continue does nothing yet: Enter flashes it in the error colour. */
  readonly blocker?: string;
  /** What the footer says before Continue when nothing blocks it: 12 of 40 picked. */
  readonly note?: string;
  /** A page with no footer: the review, whose action is in its body, and the running steps. */
  readonly footless?: boolean;
  /** Going back past it is refused: it can't be undone (the install started). */
  readonly fixed?: boolean;
}

/** What the rail shows at its foot besides the key hints: Kōbō's power button and battery. */
export interface RailFoot {
  readonly power: boolean;
  /** The battery's percent, or none on a desktop. */
  readonly battery?: number;
}

/** The pictures behind Kōbō's panel: the wallpaper, and the same blurred for its glass. */
export interface Backdrop {
  readonly wallpaper: string;
  readonly blurred: string;
}

export interface StepRailOptions {
  readonly host: RailHost;
  /** The brand (Kōbō's lockup and its version line) or the window's title, at the rail's top. */
  readonly brand: HTMLElement;
  readonly pages: readonly RailPage[];
  /** The page showing, by its index. */
  readonly current?: number;
  /** The furthest page reached: a done step up to it takes a press. */
  readonly reached?: number;
  /** The current step failed: its symbol is `error`. */
  readonly failed?: boolean;
  /** The current step is running (the install): no step takes a press until it ends. */
  readonly locked?: boolean;
  /** The keys the current page takes, at the rail's foot. */
  readonly hints: (page: number) => readonly KeyHint[];
  readonly foot?: RailFoot;
  /** Kōbō's wallpaper and its blurred copy, which shows through the glass panel. */
  readonly backdrop?: Backdrop;
  /** The blocker's flash, held this far through, as a still sample of it. */
  readonly nudged?: number;
  /** The current page arriving from this side, held this far in, as a still sample of the slide. */
  readonly arriving?: { readonly forward: boolean; readonly shown: number };
}

/** A rail step's state: done before the current one, the current one, to do after it. */
function stateOf(index: number, current: number): StepState {
  if (index < current) return { is: "done" };
  if (index === current) return { is: "current" };
  return { is: "todo" };
}

/**
 * The first page a done step can go back to: none before a step that can't be undone, nor while
 * the current one runs.
 */
function backTo(pages: readonly RailPage[], current: number, locked: boolean): number {
  if (locked) return current;
  let first = 0;
  for (let index = 0; index < current; index++) if (pages[index]?.fixed) first = index + 1;
  return first;
}

/** Key hints one under the other, `space.s4` apart: each a line of `KeyHints`, its words right after its keys. */
export function hintColumn(hints: readonly KeyHint[]): HTMLElement {
  return stack("s4", "start", ...hints.map((hint) => keyHints([hint])));
}

/** The battery: its symbol and the percent, muted, in `label`. */
function battery(percent: number): HTMLElement {
  const words = new Intl.NumberFormat(undefined, { style: "percent" }).format(percent / 100);
  return row("s4", "centre", icon("battery_full", "var(--icon-s16)", "var(--color-on_surface_variant)"), text(words, "label", "on_surface_variant"));
}

/** The rail: the brand, `space.s20`, the steps on their wire, then the hints and the foot at its bottom. */
function rail(options: StepRailOptions, current: number, reached: number, go: (to: number) => void): HTMLElement {
  const first = backTo(options.pages, current, options.locked ?? false);
  const steps = options.pages.map((each, index) => {
    const pressable = index !== current && index >= first && index <= reached;
    const element = step({
      ...each.step,
      state: stateOf(index, current),
      ...(index === current && options.failed ? { failed: true } : {}),
      ...(index === current && options.locked ? { running: true } : {}),
      box: "rail",
      mark: true,
      pressable,
    });
    if (pressable) {
      element.addEventListener("click", () => go(index));
      element.addEventListener("keydown", (event) => {
        if ((event.key === "Enter" || event.key === " ") && !event.repeat) {
          event.preventDefault();
          go(index);
        }
      });
    }
    return element;
  });
  arrows(steps);
  const column = stack(0, "start", ...steps, wire(steps.length, current));
  column.classList.add("k-rail-steps");
  // Cut at its own box: the brand's `space.s20` and the hints' `space.s16` stay clear of the steps
  // scrolling between them.
  const { box: scroller } = scroll(column, { height: "100%", above: 0, below: 0, noGutter: true });
  const parts: Element[] = [inset(options.brand, 0, "s12"), gap("s20"), fillingHeight(scroller)];
  const hints = options.hints(current);
  if (hints.length > 0) parts.push(gap("s16"), inset(hintColumn(hints), 0, "s12"));
  if (options.foot) {
    const foot: Element[] = [];
    if (options.foot.power) {
      const power = button({ label: "Power", kind: "Text", size: "Sm", icon: "power_settings_new" });
      power.classList.add("k-rail-power");
      foot.push(power);
    }
    foot.push(fill());
    if (options.foot.battery !== undefined) foot.push(battery(options.foot.battery));
    parts.push(gap("s16"), inset(row("s8", "centre", ...foot), 0, "s12"));
  }
  const box = document.createElement("span");
  box.className = "k-rail";
  box.append(fillingHeight(stack(0, "start", ...parts)));
  return probe(box, { role: "component", name: "StepRail", variant: options.host, fixed: { width: true, height: false } });
}

/**
 * The progress wire down the steps' symbols, from the first to the last, lit down to `current`.
 * Each symbol is a bead on it: the wire is cut away round each, so whatever is under the bead (the
 * rail's fill, or the current row's) shows there, glass and all.
 */
function wire(count: number, current: number): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-rail-wire";
  box.style.setProperty("--count", String(count));
  const cut = (index: number) => `radial-gradient(circle at var(--wire-x) calc(${index} * var(--size-row) + var(--wire-top)), transparent var(--bead-r), black var(--bead-r))`;
  box.style.maskImage = Array.from({ length: count }, (_, index) => cut(index)).join(", ");
  const track = document.createElement("span");
  track.className = "k-rail-wire-track";
  const lit = document.createElement("span");
  lit.className = "k-rail-wire-lit";
  lit.style.setProperty("--wire-lit", String(current));
  box.append(track, lit);
  return box;
}

/** How far through a move on `token`'s curve it is when the curve has come `share` of the way. */
function reaching(token: string, share: number): number {
  const [x1 = 0, y1 = 0, x2 = 1, y2 = 1] = cubic(`easing-${token}`);
  const along = (a: number, b: number, t: number) => 3 * a * t * (1 - t) ** 2 + 3 * b * t * t * (1 - t) + t ** 3;
  let low = 0;
  let high = 1;
  for (let round = 0; round < 40; round++) {
    const middle = (low + high) / 2;
    if (along(y1, y2, middle) < share) low = middle;
    else high = middle;
  }
  return along(x1, x2, low);
}

/**
 * The wire's light moving from step `from` to `to`: its lit length grows (or retracts) to the new
 * current symbol over `long` on `emphasized_decel`, and each symbol it reaches lights over `short`
 * as the light gets there (one it leaves goes out as the light leaves it); the new current one
 * lights `medium` in, as the light arrives at its bead. With reduced motion it is the state change
 * alone.
 */
function travel(steps: HTMLElement, from: number, to: number, failed: boolean): void {
  if (settings().reduced || from === to) return;
  const long = timed("long");
  steps.querySelector(".k-rail-wire-lit")?.animate([{ "--wire-lit": from }, { "--wire-lit": to }], { duration: long, easing: curve("emphasized_decel") });
  const rows = [...steps.querySelectorAll<HTMLElement>(":scope > .k-step")];
  const look = (index: number, at: number, fails: boolean) => ({ "--bead-lit": index <= at ? 1 : 0, "--bead-glow": index === at && !fails ? 1 : 0 });
  rows.forEach((each, index) => {
    const before = look(index, from, false);
    const after = look(index, to, failed);
    if (before["--bead-lit"] === after["--bead-lit"] && before["--bead-glow"] === after["--bead-glow"]) return;
    const passed = to > from ? (index - from) / (to - from) : (from - index) / (from - to);
    const delay = index === to ? timed("medium") : reaching("emphasized_decel", Math.max(0, passed)) * long;
    each.querySelector(".k-step-bead")?.animate([before, after], { duration: timed("short"), delay, easing: curve("standard"), fill: "backwards" });
  });
}

/** ↑ and ↓ (k and j) move the keyboard between the steps that take a press. */
function arrows(steps: readonly HTMLElement[]): void {
  const pressable = steps.filter((each) => each.tabIndex === 0);
  pressable.forEach((each, index) => {
    each.addEventListener("keydown", (event) => {
      const by = event.key === "ArrowDown" || event.key === "j" ? 1 : event.key === "ArrowUp" || event.key === "k" ? -1 : 0;
      const next = pressable[index + by];
      if (by === 0 || !next) return;
      event.preventDefault();
      next.focus();
    });
  });
}

/** The footer's buttons pinned in a state, as a state matrix shows them. */
export interface FooterPins {
  readonly back?: Pinned;
  readonly next?: Pinned;
}

/**
 * The footer band: Back on the left, then the note or the blocker, `space.s16` apart, and the one
 * Filled button on the right. A blocker is the `error` symbol and its reason in `small`, flashing
 * in `error` as Enter is refused.
 */
export function footer(spec: RailPage, onBack: () => void = () => {}, onNext: () => void = () => {}, pinned: FooterPins = {}): { band: HTMLElement; nudge: HTMLElement | null } {
  const back = button({ label: "Back", kind: "Text", icon: "arrow_back", ...(pinned.back ? { pinned: pinned.back } : {}) });
  back.classList.add("k-step-back");
  back.addEventListener("click", onBack);
  const parts: Element[] = [back, fill()];
  let nudge: HTMLElement | null = null;
  if (spec.blocker) {
    const reason = text(spec.blocker, "small");
    reason.style.color = "var(--nudge-ink)";
    nudge = row("s8", "centre", icon("error", "var(--type-small-icon)", "var(--nudge-ink)"), reason);
    nudge.classList.add("k-step-blocker");
    parts.push(nudge);
  } else if (spec.note) {
    parts.push(text(spec.note, "small", "on_surface_variant"));
  }
  const next = button({ label: spec.next ?? "Continue", kind: "Filled", icon: "arrow_forward", trailing: true, ...(spec.blocker ? { disabled: true } : {}), ...(pinned.next ? { pinned: pinned.next } : {}) });
  next.addEventListener("click", onNext);
  parts.push(next);
  const band = row("s16", "centre", ...parts);
  band.classList.add("k-step-footer");
  return { band, nudge };
}

/** A duration token in milliseconds at the speed picked, capped at `short` with reduced motion. */
function timed(token: string): number {
  const ms = (name: string) => {
    const value = raw(`duration-${name}`);
    return Number.parseFloat(value) * (value.endsWith("ms") ? 1 : 1000);
  };
  const { reduced, speed } = settings();
  return (reduced ? Math.min(ms(token), ms("short")) : ms(token)) / speed;
}

/** A curve token as CSS. */
function curve(token: string): string {
  return `cubic-bezier(${cubic(`easing-${token}`).join(", ")})`;
}

/**
 * A page arriving: it slides `space.s32` in from the side it comes from and fades in, over `long`
 * on `emphasized_decel`, its offset on whole device pixels; with reduced motion it only fades.
 */
export function arrive(room: HTMLElement, forward: boolean): void {
  room.dataset.from = forward ? "end" : "start";
  const reduced = settings().reduced;
  const frames: Keyframe[] = reduced ? [{ opacity: 0 }, { opacity: 1 }] : [{ opacity: 0, "--slide": 1 }, { opacity: 1, "--slide": 0 }];
  room.animate(frames, { duration: timed("long"), easing: curve("emphasized_decel") });
}

/** Enter refused: the blocker's reason in `error` at once, fading back over `slow`. */
function flash(nudge: HTMLElement): void {
  nudge.animate([{ "--nudge": 1 }, { "--nudge": 0 }], { duration: timed("slow"), easing: curve("standard_accel") });
}

/** The page's room: the clear page, its body above the footer band; what slides and fades. */
function pageRoom(spec: RailPage, onBack: () => void, onNext: () => void): { room: HTMLElement; nudge: HTMLElement | null } {
  const parts: Element[] = [fillingHeight(filling(spec.body()))];
  let nudge: HTMLElement | null = null;
  if (!spec.footless) {
    const made = footer(spec, onBack, onNext);
    nudge = made.nudge;
    parts.push(gap("s16"), filling(made.band));
  }
  const room = document.createElement("span");
  room.className = "k-step-room";
  room.append(page(fillingHeight(stack(0, "start", ...parts)), { clear: true }));
  return { room, nudge };
}

/**
 * The template, live: the rail beside the page. A done step that can be gone back to, Back and Esc
 * go back a page; Continue and Enter go on, or flash the blocker's reason; each page arrives as
 * `arrive` says.
 */
export function stepRailPage(options: StepRailOptions): HTMLElement {
  let current = options.current ?? 0;
  let reached = Math.max(current, options.reached ?? current);
  const box = document.createElement("span");
  box.className = "k-step-rail-page";
  box.dataset.host = options.host;
  // A whole surface: a motion player draws it as it is laid out.
  box.dataset.whole = "";
  box.tabIndex = -1;
  // The blocker of the page showing, which Enter refused flashes.
  const shown: { nudge: HTMLElement | null } = { nudge: null };

  const draw = (from: number | null) => {
    const spec = options.pages[current];
    if (!spec) return;
    const made = pageRoom(spec, () => go(current - 1), next);
    shown.nudge = made.nudge;
    const side = document.createElement("span");
    side.className = "k-step-side";
    side.append(made.room);
    const scrolled = box.querySelector(".k-rail .k-scroll-view")?.scrollTop ?? 0;
    box.replaceChildren(rail(options, current, reached, go), side);
    const steps = box.querySelector<HTMLElement>(".k-rail-steps");
    if (steps) keepInView(steps, current, scrolled);
    if (from === null) return;
    arrive(made.room, current > from);
    if (steps) travel(steps, from, current, options.failed ?? false);
  };

  function go(to: number): void {
    if (to < backTo(options.pages, current, options.locked ?? false) || to < 0 || to >= options.pages.length || to === current) return;
    const from = current;
    current = to;
    reached = Math.max(reached, current);
    draw(from);
  }

  function next(): void {
    const spec = options.pages[current];
    if (spec?.blocker) {
      if (shown.nudge) flash(shown.nudge);
      return;
    }
    go(current + 1);
  }

  box.addEventListener("keydown", (event) => {
    if (event.defaultPrevented || event.repeat) return;
    if ((event.target as Element).closest(".k-step")) return;
    if (event.key === "Enter") next();
    else if (event.key === "Escape") go(current - 1);
    else return;
    event.preventDefault();
  });

  draw(null);
  if (options.nudged !== undefined) shown.nudge?.style.setProperty("--nudge", String(options.nudged));
  if (options.arriving) {
    const room = box.querySelector<HTMLElement>(".k-step-room");
    if (room) {
      room.dataset.from = options.arriving.forward ? "end" : "start";
      room.style.opacity = String(options.arriving.shown);
      room.style.setProperty("--slide", String(1 - options.arriving.shown));
    }
  }
  const framed = options.host === "kobo" && options.backdrop ? koboPanel(box, options.backdrop) : box;
  return probe(framed, { role: "component", name: "StepRailPage", variant: options.host, fixed: { width: false, height: false, holds: true } });
}

/**
 * The rail's steps scrolled where they were before it was drawn again, then as little more as puts
 * the current step in view.
 */
function keepInView(steps: HTMLElement, current: number, scrolled: number): void {
  const view = steps.closest<HTMLElement>(".k-scroll-view");
  const scroller = steps.closest<HTMLElement>(".k-scroll");
  const row = steps.querySelectorAll<HTMLElement>(":scope > .k-step")[current];
  if (!view || !scroller || !row) return;
  // The template is drawn before it is placed; it is in place by the next frame.
  if (!scroller.isConnected) {
    requestAnimationFrame(() => scroller.isConnected && keepInView(steps, current, scrolled));
    return;
  }
  view.scrollTop = scrolled;
  const room = scroller.getBoundingClientRect();
  const shown = row.getBoundingClientRect();
  // A sample may be drawn scaled down; the scroll moves in the box's own pixels.
  const ratio = scroller.offsetHeight / Math.max(1, room.height);
  if (shown.top < room.top) view.scrollTop -= Math.ceil((room.top - shown.top) * ratio);
  else if (shown.bottom > room.bottom) view.scrollTop += Math.ceil((shown.bottom - room.bottom) * ratio);
}

/**
 * A logo drawing (its SVG's text) in the live palette: it is written in the default theme's dark
 * `on_surface`, `primary` and `cold`, so each takes its role's colour now, as Kōbō swaps them for the
 * mode it draws in.
 */
function drawing(svg: string, className: string): HTMLElement {
  const written = palette(THEMES[0]!.slug, "dark");
  let shown = svg;
  for (const role of ["on_surface", "primary", "cold"]) {
    const hex = written[role];
    if (hex) shown = shown.replaceAll(new RegExp(hex, "gi"), `var(--color-${role})`);
  }
  const box = document.createElement("span");
  box.className = className;
  box.innerHTML = shown;
  return box;
}

/** Kōbō's lockup, the mark beside the wordmark, and the version line under it, `space.s4` apart. */
export function koboBrand(mark: string, wordmark: string, version: string): HTMLElement {
  const lockup = row(0, "centre", drawing(mark, "k-lockup-mark"), drawing(wordmark, "k-lockup-word"));
  return stack("s4", "start", lockup, text(version, "caption", "on_surface_variant"));
}

/** A window's title at the rail's top, and the line under it: the updater's. */
export function windowBrand(title: string, line: string): HTMLElement {
  return stack("s4", "start", text(title, "heading"), text(line, "caption", "on_surface_variant"));
}

/**
 * Kōbō's glass panel, the template inside it, over the wallpaper and its scrim. The panel is inset
 * `space.s32` from the screen's edges where the screen holds two columns of text side by side, else
 * `space.s16`, and no wider than the rail beside two columns and `space.s40` either side of them, so
 * lines stay short on a big screen; centred, on whole device pixels. That inset turns on the
 * screen's own width, which CSS can't compare with a length, so script places it.
 */
function koboPanel(template: HTMLElement, backdrop: Backdrop): HTMLElement {
  const screen = document.createElement("span");
  screen.className = "k-kobo";
  screen.style.backgroundImage = `url("${backdrop.wallpaper}")`;
  const glass = document.createElement("img");
  glass.className = "k-kobo-glass";
  glass.src = backdrop.blurred;
  glass.alt = "";
  const panel = document.createElement("span");
  panel.className = "k-kobo-panel";
  panel.append(glass, template);
  screen.append(panel);
  new ResizeObserver(() => {
    const scale = Number.parseFloat(getComputedStyle(screen).getPropertyValue("--scale")) || settings().scale;
    const width = screen.clientWidth / scale;
    const height = screen.clientHeight / scale;
    const margin = length(width >= 2 * length("size-column") ? "space-s32" : "space-s16");
    const widest = length("size-rail") + 2 * length("space-s40") + 2 * length("size-column");
    const wide = Math.min(width - 2 * margin, widest);
    panel.style.left = `${Math.round(((width - wide) / 2) * scale)}px`;
    panel.style.top = `${Math.round(margin * scale)}px`;
    panel.style.width = `${Math.round(wide * scale)}px`;
    panel.style.height = `${Math.round((height - 2 * margin) * scale)}px`;
    // The blurred wallpaper lies where the wallpaper does, so the glass shows what is behind it.
    glass.style.left = `-${panel.style.left}`;
    glass.style.top = `-${panel.style.top}`;
    glass.style.width = `${screen.clientWidth}px`;
    glass.style.height = `${screen.clientHeight}px`;
  }).observe(screen);
  return screen;
}
