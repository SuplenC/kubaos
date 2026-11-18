// Displays' own values and its organisms: the screens the machine has seen, where each stands on
// the desk, and the "keep this?" question a new mode or scale asks before it stays.
//
// What KubaOS has today: Hyprland puts every screen at its preferred mode, placed and scaled by
// itself (`config/hypr/monitors.lua`); a layout of one's own is `hl.monitor` lines in the user
// layer's `desktop/hypr/hyprland.lua`. Workspace groups follow desk positions
// (`config/hypr/workspaces.lua`), pinned and named in `desktop/workspaces/groups.toml`. The laptop
// panel's backlight goes through brightnessctl (`config/quickshell/services/Brightness.qml`).
//
// Its organisms, kept in the Settings app because only it draws them:
// - `arrangement`: the library's `Canvas` (`lib/canvas-page.ts`) as a sub-section's part, its
//   selection the rows' screen, kept across the region's redraws;
// - `keepDialog`: the library's `dialogOver`, asking to keep a new mode or scale, with a timed
//   `ProgressBar` draining to the moment it goes back by itself.

import "./displays.css";
import { type Canvas, canvas, numbers, type Screen } from "../../../../lib/canvas-page.ts";
import { dialogOver } from "../../../../lib/dialog.ts";
import { captionedBar, progressBar, setCaption, setProgress } from "../../../../lib/progress.ts";
import { text } from "../../../../lib/text.ts";

/** One of a screen's modes: its size in pixels and the rates it runs at, fastest first. */
export interface Mode {
  readonly width: number;
  readonly height: number;
  readonly rates: readonly number[];
}

/** A screen the machine has seen, as `hyprctl monitors all` lists it. */
export interface KnownScreen {
  readonly id: string;
  readonly name: string;
  /** The laptop's own panel, `eDP-1`: it has a backlight, and a desk position of its own. */
  readonly panel: boolean;
  /** Its `availableModes`, the preferred one first. */
  readonly modes: readonly Mode[];
}

/**
 * The home desk of CanvasPage's page (`pages/templates/canvas-page.ts`), so the same screens look the
 * same in both: a 4K screen on an arm, the laptop under it, a 1080p screen to the right.
 */
export const SCREENS: readonly KnownScreen[] = [
  { id: "dell", name: "Dell U2723QE", panel: false, modes: [{ width: 3840, height: 2160, rates: [60, 30] }, { width: 2560, height: 1440, rates: [60] }, { width: 1920, height: 1080, rates: [60, 50] }] },
  { id: "laptop", name: "Built-in display", panel: true, modes: [{ width: 2880, height: 1800, rates: [120, 60] }, { width: 1920, height: 1200, rates: [120, 60] }, { width: 1440, height: 900, rates: [60] }] },
  { id: "lg", name: "LG 24MP60G", panel: false, modes: [{ width: 1920, height: 1080, rates: [75, 60] }, { width: 1280, height: 720, rates: [60] }] },
];

/** The scales KubaOS draws crisply at (`rules.scales`): the 4 px grid lands on whole device pixels. */
export const SCALES = [1, 1.25, 1.5, 2] as const;
/** Hyprland's `transform` 0 to 3, a quarter turn each. */
export const TURNS = ["Landscape", "Portrait", "Upside down", "Portrait, flipped"] as const;
/** The workspace groups the machine has: the positions its desks' externals have stood at. */
export const GROUP_COUNT = 2;
/** The other desks saved: a name is a desk's own. */
export const OTHER_DESKS = ["Desk at work"] as const;
/** How long a new mode or scale waits to be kept before it goes back by itself. */
const KEEP_SECONDS = 15;

/** A change waiting on "keep this?": what changed, and the values it goes back to. */
export interface Asking {
  /** The screen it changed, by `SCREENS`' order. */
  readonly screen: number;
  what: "mode" | "rate" | "scale" | "several";
  readonly back: { readonly mode: number[]; readonly rate: number[]; readonly scale: number[]; readonly x: number[]; readonly y: number[] };
  /** The seconds left, held there: a still sample's. */
  readonly left?: number;
}

/** Displays' own values, per screen by `SCREENS`' order. */
export interface DisplaysValues {
  plugged: boolean[];
  /** The screen picked on the canvas, whose settings the rows show. */
  picked: number;
  /** Each screen's mode, by its `modes`. */
  mode: number[];
  /** Each screen's rate, by its mode's `rates`. */
  rate: number[];
  scale: number[];
  turn: number[];
  /** The screen each one mirrors, or -1: it extends the desk. */
  mirror: number[];
  /** The workspace group each is pinned to in `groups.toml`, or 0: where it stands decides. */
  pin: number[];
  /** Where each stands, its top-left corner in logical pixels. */
  x: number[];
  y: number[];
  desk: string;
  autoDesk: boolean;
  /** Each group's name in `groups.toml`, blank for none. */
  names: string[];
  /** The panel's backlight, as a stop of `BRIGHTNESS`. */
  brightness: number;
  autoBrightness: boolean;
  night: number;
  warmth: number;
  asking: Asking | null;
}

/**
 * KubaOS's defaults, what every reset puts back: each screen at its preferred mode (the LG's is
 * 60 Hz, though it runs at 75), at the scale Hyprland's `auto` picks for it, the home desk's
 * arrangement, nothing mirrored or pinned, the night light off.
 */
export function displayDefaults(): DisplaysValues {
  return {
    plugged: [true, true, true],
    picked: 0,
    mode: [0, 0, 0],
    rate: [0, 0, 1],
    scale: [2, 2, 0],
    turn: [0, 0, 0],
    mirror: [-1, -1, -1],
    pin: [0, 0, 0],
    x: [0, 320, 2560],
    y: [0, 1440, 360],
    desk: "Desk at home",
    autoDesk: true,
    names: ["", ""],
    brightness: 13,
    autoBrightness: false,
    night: 0,
    warmth: 1,
    asking: null,
  };
}

/** Values of their own: a sample's seed is shared by every window drawn from it, its lists too. */
export function owned(values: DisplaysValues): DisplaysValues {
  for (const key of Object.keys(values) as (keyof DisplaysValues)[]) {
    const held = values[key];
    if (Array.isArray(held)) (values as unknown as Record<string, unknown>)[key] = [...held];
  }
  return values;
}

/** The screen the rows show: the one picked, or the first plugged in once it is gone. */
export function shown(v: DisplaysValues): number {
  return v.plugged[v.picked] ? v.picked : Math.max(0, v.plugged.indexOf(true));
}

/** The screens plugged in, by `SCREENS`' order. */
export function pluggedIn(v: DisplaysValues): number[] {
  return SCREENS.map((_, index) => index).filter((index) => v.plugged[index]);
}

/** The screen `index` mirrors, while that one is plugged in and mirrors nothing itself. */
export function mirroring(v: DisplaysValues, index: number): number | null {
  const of = v.mirror[index] ?? -1;
  return of >= 0 && v.plugged[of] && (v.mirror[of] ?? -1) < 0 ? of : null;
}

/** Where a screen stands on the desk (GLOSSARY.md, "Desk position"). */
export type Standing = { readonly at: "panel" } | { readonly at: "position"; readonly n: number } | { readonly at: "mirror"; readonly of: number };

/**
 * Where each screen plugged in stands, as `workspaces.lua` works it out: a pinned screen at its
 * group's number, the first listed winning; the laptop panel unpinned at its own place; the other
 * externals counted from the left, passing over every number a group pins.
 */
export function standings(v: DisplaysValues): Map<number, Standing> {
  const found = new Map<number, Standing>();
  const pinned = new Set(v.pin.filter((n) => n > 0));
  const taken = new Set<number>();
  const loose: number[] = [];
  for (const index of pluggedIn(v)) {
    const of = mirroring(v, index);
    if (of !== null) {
      found.set(index, { at: "mirror", of });
      continue;
    }
    const pin = v.pin[index] ?? 0;
    if (pin > 0 && !taken.has(pin)) {
      taken.add(pin);
      found.set(index, { at: "position", n: pin });
    } else if (SCREENS[index]!.panel) found.set(index, { at: "panel" });
    else loose.push(index);
  }
  loose.sort((a, b) => v.x[a]! - v.x[b]! || v.y[a]! - v.y[b]!);
  let next = 1;
  for (const index of loose) {
    while (pinned.has(next) || taken.has(next)) next++;
    taken.add(next);
    found.set(index, { at: "position", n: next });
  }
  return found;
}

/** The screen at desk position `n`, if one stands there. */
export function standingAt(v: DisplaysValues, n: number): number | null {
  for (const [index, standing] of standings(v)) if (standing.at === "position" && standing.n === n) return index;
  return null;
}

/** The primary screen, where a group without a screen waits: the panel, else the first external. */
export function primary(v: DisplaysValues): number {
  const all = pluggedIn(v);
  return all.find((index) => SCREENS[index]!.panel) ?? all[0] ?? 0;
}

/** The mode screen `index` runs at. */
export function modeOf(v: DisplaysValues, index: number): Mode {
  const modes = SCREENS[index]!.modes;
  return modes[v.mode[index] ?? 0] ?? modes[0]!;
}

/** A mode's size as words: "2560 × 1440". */
export function sizeOf(mode: Mode): string {
  return `${mode.width} × ${mode.height}`;
}

/** A rate as words: "120 Hz". */
export function hertz(rate: number): string {
  return `${rate} Hz`;
}

/** A scale as words: "1.5×". */
export function times(scale: number): string {
  return `${scale}×`;
}

/** The desk the canvas's chip names: the one saved for these screens, or none yet. */
export function deskWords(v: DisplaysValues): string {
  const all = pluggedIn(v);
  if (all.length === 1 && SCREENS[all[0]!]!.panel) return "Laptop alone";
  return v.desk.trim() || "New desk";
}

/** Each screen plugged in as the canvas draws it: its mode turned as it stands, a mirror on its source. */
export function onCanvas(v: DisplaysValues): Screen[] {
  const drawn = pluggedIn(v).map((index, at) => {
    const mode = modeOf(v, index);
    const side = (v.turn[index] ?? 0) % 2 === 1;
    const scale = SCALES[v.scale[index] ?? 0]!;
    return { id: SCREENS[index]!.id, name: SCREENS[index]!.name, number: at + 1, width: side ? mode.height : mode.width, height: side ? mode.width : mode.height, scale, x: v.x[index]!, y: v.y[index]! };
  });
  // A mirror stands where its source does, as big: Hyprland draws the source's picture on it.
  for (const index of pluggedIn(v)) {
    const of = mirroring(v, index);
    if (of === null) continue;
    const mine = drawn.find((each) => each.id === SCREENS[index]!.id)!;
    const theirs = drawn.find((each) => each.id === SCREENS[of]!.id)!;
    Object.assign(mine, { x: theirs.x, y: theirs.y, width: (theirs.width / theirs.scale) * mine.scale, height: (theirs.height / theirs.scale) * mine.scale });
  }
  return drawn;
}

/** The numbers line under a screen's name: its size, mode and scale, and where it sits. */
export function numbersOf(v: DisplaysValues, index: number): string {
  const drawn = onCanvas(v).find((each) => each.id === SCREENS[index]!.id) ?? onCanvas(v)[0]!;
  // A mirror is drawn its source's size; its own numbers are still its mode's.
  const mode = modeOf(v, index);
  const side = (v.turn[index] ?? 0) % 2 === 1;
  return numbers({ ...drawn, width: side ? mode.height : mode.width, height: side ? mode.width : mode.height });
}

/** Whether the canvas drawn shows `want`, box for box. */
function showing(drawn: readonly Screen[], want: readonly Screen[]): boolean {
  return drawn.length === want.length && drawn.every((each, at) => {
    const other = want[at]!;
    return each.id === other.id && each.x === other.x && each.y === other.y && each.width === other.width && each.height === other.height && each.scale === other.scale;
  });
}

/** The canvas each window's values drew, and the desk its chip names, kept across the region's redraws. */
const drawnFor = new WeakMap<DisplaysValues, { readonly canvas: Canvas; readonly desk: string }>();

/** The Settings region `element` is in, drawn again: the rows follow the values. */
function redraw(element: Element): void {
  element.closest(".st-region")?.dispatchEvent(new CustomEvent("st-redraw"));
}

/** The keyboard on setting `id`'s row in `within`, as a person landing on it. */
function onRow(within: Element | null, id: string): void {
  const row = within?.querySelector<HTMLElement>(`.k-list-row[data-setting="${id}"]`);
  if (!row) return;
  row.dispatchEvent(new PointerEvent("pointermove"));
  row.closest<HTMLElement>(".k-list")?.focus({ preventScroll: true });
}

/**
 * The screens as they stand, drawn by the library's `Canvas`. Dragging one places it;
 * picking one shows its settings in the rows under it. The canvas is kept while what it shows
 * holds, so a drag, a glide and the keyboard on it live through the region drawing its rows again;
 * a new mode, scale or mirror draws it anew.
 */
export function arrangement(v: DisplaysValues): HTMLElement {
  const want = onCanvas(v);
  const desk = deskWords(v);
  const kept = drawnFor.get(v);
  if (kept && kept.desk === desk && showing(kept.canvas.screens(), want)) {
    hold(kept.canvas.element);
    ask(v, kept.canvas.element);
    return kept.canvas.element;
  }
  const ids = SCREENS.map((each) => each.id);
  let held = false;
  // A pick or a move made while the pointer is down, whose rows follow once it is let go.
  let pending = false;
  const made = canvas({
    screens: want,
    selected: Math.max(0, want.findIndex((each) => each.id === SCREENS[shown(v)]!.id)),
    desk,
    onSelect: (screen) => {
      v.picked = ids.indexOf(screen.id);
      if (held) pending = true;
      else redraw(made.element);
    },
    onMove: (screen) => {
      const index = ids.indexOf(screen.id);
      if (mirroring(v, index) === null) {
        v.x[index] = screen.x;
        v.y[index] = screen.y;
      }
      // The picked screen's numbers follow it every frame, under its name.
      if (index === shown(v)) {
        const line = made.element.closest(".k-section")?.querySelector('[data-setting="screen"] .k-list-title + *');
        if (line) line.textContent = numbers(screen);
      }
      if (held) pending = true;
      else redraw(made.element);
    },
    onEnter: () => onRow(made.element.closest(".k-section"), "resolution"),
  });
  made.element.classList.add("dp-canvas");
  markMirrors(v, made.element, want);
  // A mirror is picked by a press but stays on its source: it has no place of its own to drag to.
  made.element.addEventListener(
    "pointerdown",
    (event) => {
      const box = (event.target as Element).closest<HTMLElement>(".k-cv-screen");
      const index = box ? ids.indexOf(want.find((each) => String(each.number) === box.dataset.n)?.id ?? "") : -1;
      if (index < 0 || mirroring(v, index) === null) {
        held = true;
        return;
      }
      event.stopPropagation();
      event.preventDefault();
      made.element.focus({ preventScroll: true });
      v.picked = index;
      redraw(made.element);
    },
    true,
  );
  // A drag's rows follow once it is let go: drawing them mid-drag would take the pointer from it.
  const letGo = () => {
    held = false;
    if (pending) redraw(made.element);
    pending = false;
  };
  made.element.addEventListener("pointerup", letGo);
  made.element.addEventListener("pointercancel", letGo);
  wireMirror(v, made.element);
  drawnFor.set(v, { canvas: made, desk });
  ask(v, made.element);
  return made.element;
}

/**
 * A mirror and its source share one box's place, so each of them shows both numbers, the
 * source's first: "2 = 1".
 */
function markMirrors(v: DisplaysValues, element: HTMLElement, drawn: readonly Screen[]): void {
  const numberOf = (index: number) => drawn.find((each) => each.id === SCREENS[index]!.id)?.number;
  for (const index of pluggedIn(v)) {
    const of = mirroring(v, index);
    if (of === null) continue;
    const pair = `${numberOf(of)} = ${pluggedIn(v).filter((each) => each === of || mirroring(v, each) === of).filter((each) => each !== of).map(numberOf).join(" = ")}`;
    for (const each of [index, of]) {
      const words = element.querySelector(`.k-cv-screen[data-n="${numberOf(each)}"] .k-cv-number`);
      // A text's box is as wide as the words it was made with: the pair is drawn anew.
      const both = text(pair, "headline");
      both.classList.add("k-cv-number");
      words?.replaceWith(both);
    }
  }
}

/** The keyboard kept on the canvas as the region moves it into its sub-section drawn again. */
function hold(element: HTMLElement): void {
  if (document.activeElement !== element) return;
  queueMicrotask(() => element.focus({ preventScroll: true }));
}

/**
 * The canvas's Mirror: every other screen plugged in mirrors the picked one, or, while they all
 * do, each goes back to extending the desk.
 */
function wireMirror(v: DisplaysValues, element: HTMLElement): void {
  const mirror = [...element.querySelectorAll<HTMLButtonElement>(".k-cv-band button")].find((each) => each.textContent?.includes("Mirror"));
  mirror?.addEventListener("click", () => {
    const to = shown(v);
    const others = pluggedIn(v).filter((index) => index !== to);
    const all = others.length > 0 && others.every((index) => v.mirror[index] === to);
    for (const index of others) v.mirror[index] = all ? -1 : to;
    if (!all) v.mirror[to] = -1;
    redraw(element);
  });
}

/** The words for what a change goes back to, in the dialog's countdown. */
function backWords(asking: Asking): string {
  const index = asking.screen;
  const modes = SCREENS[index]!.modes;
  const mode = modes[asking.back.mode[index] ?? 0] ?? modes[0]!;
  switch (asking.what) {
    case "mode":
      return sizeOf(mode);
    case "rate":
      return hertz(mode.rates[asking.back.rate[index] ?? 0] ?? mode.rates[0]!);
    case "scale":
      return times(SCALES[asking.back.scale[index] ?? 0]!);
    case "several":
      return "how it was";
  }
}

/** The dialog's title and body for what changed, said whole. */
function question(v: DisplaysValues, asking: Asking): { title: string; body: string } {
  const name = SCREENS[asking.screen]!.name;
  const mode = modeOf(v, asking.screen);
  const after = "If it looks wrong or nothing shows, wait: it goes back by itself.";
  switch (asking.what) {
    case "mode":
      return { title: "Keep this resolution?", body: `${name} runs at ${sizeOf(mode)} now. ${after}` };
    case "rate":
      return { title: "Keep this refresh rate?", body: `${name} redraws at ${hertz(mode.rates[v.rate[asking.screen] ?? 0] ?? mode.rates[0]!)} now. ${after}` };
    case "scale":
      return { title: "Keep this scale?", body: `${name} draws everything at ${times(SCALES[v.scale[asking.screen] ?? 0]!)} now. ${after}` };
    case "several":
      return { title: "Keep these display settings?", body: `${name} changed more than one thing. ${after}` };
  }
}

/** The overlay each window's values have open, so a redraw never asks twice. */
const asked = new WeakMap<DisplaysValues, HTMLElement>();

/** The countdown's caption: what it goes back to, and when. */
function countdown(asking: Asking, seconds: number): { words: string; icon: string } {
  return { words: `Back to ${backWords(asking)} in ${seconds} s`, icon: "timer" };
}

/** Asks to keep a change waiting on it, once the canvas is in its window. */
function ask(v: DisplaysValues, element: HTMLElement): void {
  if (!v.asking || asked.has(v)) return;
  const placeholder = document.createElement("span");
  asked.set(v, placeholder);
  requestAnimationFrame(() => {
    const frame = element.closest<HTMLElement>(".t-window");
    if (!frame || !v.asking) {
      asked.delete(v);
      return;
    }
    asked.set(v, keepDialog(v, v.asking, frame));
  });
}

/** Screen `index`'s size in logical pixels: its mode, turned as it stands, over its scale. */
function logical(v: DisplaysValues, index: number): { width: number; height: number } {
  const mode = modeOf(v, index);
  const side = (v.turn[index] ?? 0) % 2 === 1;
  const scale = SCALES[v.scale[index] ?? 0]!;
  return { width: Math.round((side ? mode.height : mode.width) / scale), height: Math.round((side ? mode.width : mode.height) / scale) };
}

/**
 * Screen `index` grown or shrunk from `was`: a screen that stood against its right or its bottom
 * edge moves with that edge, so the arrangement keeps touching instead of overlapping or parting.
 */
export function keepTouching(v: DisplaysValues, index: number, was: { width: number; height: number }): void {
  const now = logical(v, index);
  const right = v.x[index]! + was.width;
  const bottom = v.y[index]! + was.height;
  for (const other of pluggedIn(v)) {
    if (other === index || mirroring(v, other) !== null) continue;
    if (Math.abs(v.x[other]! - right) <= 1) v.x[other] = v.x[other]! + now.width - was.width;
    if (Math.abs(v.y[other]! - bottom) <= 1) v.y[other] = v.y[other]! + now.height - was.height;
  }
}

/** What changing a screen's mode, rate or scale does: it applies, its neighbours follow, and it asks to be kept. */
export function changeShown(v: DisplaysValues, what: "mode" | "rate" | "scale", apply: () => void): void {
  const screen = shown(v);
  if (v.asking && v.asking.screen === screen) {
    if (v.asking.what !== what) v.asking.what = "several";
  } else v.asking = { screen, what, back: { mode: [...v.mode], rate: [...v.rate], scale: [...v.scale], x: [...v.x], y: [...v.y] } };
  const was = logical(v, screen);
  apply();
  keepTouching(v, screen, was);
}

/** The row the keyboard goes back to once the question is answered. */
const ROW_OF: Record<Asking["what"], string> = { mode: "resolution", rate: "refresh", scale: "scale", several: "resolution" };

/**
 * "Keep this?", the library's Dialog over the window (`dialogOver`). A timed ProgressBar drains
 * over 15 s under the body, its caption counting down to what it goes back to. Keep keeps it; Go
 * back, Esc or the end of the time put back what it was.
 */
function keepDialog(v: DisplaysValues, asking: Asking, frame: HTMLElement): HTMLElement {
  const still = asking.left !== undefined;
  const left = asking.left ?? KEEP_SECONDS;
  const bar = progressBar({ fraction: left / KEEP_SECONDS, timed: true });
  const captioned = captionedBar(bar, countdown(asking, Math.ceil(left)));
  const words = question(v, asking);
  let done = false;
  const over = dialogOver(
    frame,
    { icon: "desktop_windows", title: words.title, body: words.body, asks: captioned, dismiss: "Go back", actions: [{ label: "Keep", icon: "check" }] },
    (label) => {
      done = true;
      const keep = label === "Keep";
      if (!keep) {
        v.mode.splice(0, v.mode.length, ...asking.back.mode);
        v.rate.splice(0, v.rate.length, ...asking.back.rate);
        v.scale.splice(0, v.scale.length, ...asking.back.scale);
        v.x.splice(0, v.x.length, ...asking.back.x);
        v.y.splice(0, v.y.length, ...asking.back.y);
      }
      v.asking = null;
      asked.delete(v);
      const region = frame.querySelector(".st-region");
      if (!keep && region) redraw(region);
      onRow(region, ROW_OF[asking.what]);
    },
    still,
  );
  if (still) return over.room;

  const start = performance.now();
  let said = Math.ceil(left);
  const tick = (now: number) => {
    if (done) return;
    const remaining = Math.max(0, KEEP_SECONDS * 1000 - (now - start));
    setProgress(bar, remaining / (KEEP_SECONDS * 1000));
    const seconds = Math.ceil(remaining / 1000);
    if (seconds !== said) {
      said = seconds;
      setCaption(captioned, countdown(asking, seconds));
    }
    if (remaining === 0) over.answer("Esc");
    else requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
  return over.room;
}
