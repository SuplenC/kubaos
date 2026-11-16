// The StepRailPage template: a step rail beside a page with a footer band, and how it moves between
// its pages (`docs/design/components/step-rail-page.md`). One template with two hosts: in an app's
// window (the updater) the rail and the page fill the window; in Kōbō they are the whole screen, on
// glass over the wallpaper, with no panel round them. Its look is `step-rail.css`.
//
// The rail is `size.rail` wide, `space.s16` in above and below and `space.s12` at its sides, so its
// steps' corners are concentric with its own: the brand or the title, `space.s20`, the steps back
// to back across its width (`Step`, each a mark at its end), scrolling where they don't fit, then
// `space.s16` and at its foot the key hints and, in Kōbō, the power button and the battery. A wire
// runs down the steps' symbols, lit down to the current one, and the symbols it reaches are lit and
// glow with it, one neon sign. The page
// beside it is a clear `Page`: its body, then `space.s16` above the footer band, which has no
// hairline: Back on the left, the one Filled button on the right, and a note just before it.

import "./step-rail.css";
import { backdrop } from "./backdrop.ts";
import { button } from "./button.ts";
import { hangs } from "./button-bar.ts";
import type { Pinned } from "./control.ts";
import { icon } from "./icon.ts";
import { keyHints, type KeyHint } from "./keycap.ts";
import { fill, filling, fillingHeight, gap, inset, page, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { scroll } from "./scroll.ts";
import { spinner } from "./spinner.ts";
import { pressable, setStepState, step, type StepOptions, type StepState } from "./step.ts";
import { text, wrapped } from "./text.ts";
import { cubic, raw } from "./tokens.ts";
import { palette } from "../site/recolour.ts";
import { settings } from "../site/settings.ts";
import { THEMES } from "../site/themes.ts";

/** Where the template is drawn: an app's window, or Kōbō's glass panel over its backdrop. */
export type RailHost = "window" | "kobo";

/** A value given as it is, or worked out each time it is read: a page's answers change it. */
export type Live<T> = T | (() => T);

function now<T>(value: Live<T>): T {
  return typeof value === "function" ? (value as () => T)() : value;
}

/** A page's own standing, as it is now. */
function statusOf(spec: RailPage): RailStatus | undefined {
  return spec.status === undefined ? undefined : now(spec.status);
}

/**
 * The event a page's body sends when its answers change what its footer says: the band is drawn
 * again in place, its blocker, note and Continue's words worked out anew.
 */
export const RECHECK = "k-step-recheck";

/**
 * The event a page's body sends to go on as Continue does: Enter on the row of its list that is
 * already its answer, which the list itself takes as a pick.
 */
export const GO_ON = "k-step-go-on";

/** One page of the flow: its step in the rail and what the page holds. */
export interface RailPage {
  /** Its step: a title and a symbol; where it stands comes from the flow. */
  readonly step: Pick<StepOptions, "title" | "icon">;
  /** The page's body, drawn each time the page shows. */
  readonly body: () => HTMLElement;
  /** Continue's words: Continue, Skip, Install; worked out again as the page changes. */
  readonly next?: Live<string>;
  /** Why Continue does nothing yet: Enter flashes it in the error colour. */
  readonly blocker?: Live<string | undefined>;
  /** What the footer says before Continue when nothing blocks it: 12 of 40 picked. */
  readonly note?: Live<string | undefined>;
  /**
   * Back closes its host: the one page of a flow opened on it alone (a first-boot card from the
   * Command menu), where there is no page before it to go back to. Back then reads Close.
   */
  readonly closes?: boolean;
  /** A page with no footer: the review, whose action is in its body, and the running steps. */
  readonly footless?: boolean;
  /** Its body reads the whole flow (the review), so it is drawn again each time it shows. */
  readonly fresh?: boolean;
  /** Where in its body the keyboard lands, when not on its first control: the review's hold. */
  readonly lands?: string;
  /** Going back past it is refused: it can't be undone (the install started). */
  readonly fixed?: boolean;
  /** Where it stands on its own, not by its place in the flow: first boot's cards (`free`). */
  readonly status?: Live<RailStatus>;
  /** A quieter way on, a `Text` button before the `Filled` one: Skip. */
  readonly skip?: Live<string | undefined>;
  /** The `Filled` button's symbol, before its words, when it does the page's own work: Enroll. */
  readonly nextIcon?: Live<string | undefined>;
  /** The `Filled` button's own work: true keeps the page, which shows the work; false goes on. */
  readonly act?: () => boolean;
  /** Told as Skip passes the page by, before the next one shows. */
  readonly skipped?: () => void;
  /**
   * The page's own work is running, in these words: the `Filled` button keeps its room, a spinner
   * and the words in its place, and Enter waits for the work.
   */
  readonly busy?: Live<string | undefined>;
}

/** A step's own standing in a free flow: its state, its other words, and whether it is there. */
export interface RailStatus {
  readonly state: StepState;
  readonly words?: StepOptions["words"];
  readonly failed?: boolean;
  readonly running?: boolean;
  /** Not on this machine: going on or back passes it by, though a press still opens it. */
  readonly out?: boolean;
}

/** What the rail shows at its foot besides the key hints: Kōbō's power button and battery. */
export interface RailFoot {
  readonly power: boolean;
  /** The battery's percent, or none on a desktop. */
  readonly battery?: number;
  readonly charging?: boolean;
}

/** The picture behind Kōbō's screen: the wallpaper blurred, under its glass. */
export interface Backdrop {
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
  /** Every page takes a press, in any order, each step showing its own `status`: first boot. */
  readonly free?: boolean;
  /** The keys the current page takes, at the rail's foot. */
  readonly hints: (page: number) => readonly KeyHint[];
  readonly foot?: RailFoot;
  /** Kōbō's blurred wallpaper, which shows through the glass the screen is. */
  readonly backdrop?: Backdrop;
  /** The blocker's flash, held this far through, as a still sample of it. */
  readonly nudged?: number;
  /** The current page arriving from this side, held this far in, as a still sample of the slide. */
  readonly arriving?: { readonly forward: boolean; readonly shown: number };
  /**
   * In Kōbō, the page the current one was reached from, drawn leaving over it: a move's first state,
   * which its player takes from there.
   */
  readonly leaving?: number;
  /**
   * What its host does past either end: Back or Esc on the first page (`forward` false), Continue
   * or Enter on the last (`forward` true). Kōbō's welcome before it, its install after it.
   */
  readonly onExit?: (forward: boolean) => void;
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

/** At or under this percent, running on it, a battery is low: an install outlasts it. */
const LOW_BATTERY = 20;

/**
 * The battery: its symbol by how full it is (charging, its own), and the percent in `label`, muted;
 * low and not charging, both in `warning`.
 */
export function battery(percent: number, charging = false): HTMLElement {
  const words = new Intl.NumberFormat(undefined, { style: "percent" }).format(percent / 100);
  const low = !charging && percent <= LOW_BATTERY;
  const glyph = charging ? "battery_charging_full" : percent >= 80 ? "battery_full" : percent >= 50 ? "battery_3_bar" : percent >= 25 ? "battery_2_bar" : "battery_1_bar";
  const tone = low ? "warning" : "on_surface_variant";
  const line = row("s4", "centre", icon(glyph, "var(--icon-s16)", `var(--color-${tone})`), text(words, "label", tone));
  line.classList.add("k-battery");
  return line;
}

/** The rail: the brand, `space.s20`, the steps on their wire, then the hints and the foot at its bottom. */
/** The rail, drawn once and moved to each page in place. */
interface Rail {
  readonly element: HTMLElement;
  /** Moves it to `current`, with every page up to `reached` reached. */
  set(current: number, reached: number): void;
}

function rail(options: StepRailOptions, current: number, reached: number, go: (to: number) => void): Rail {
  /** What step `index` shows with the rail on `at`. */
  const look = (index: number, at: number, upTo: number) => {
    const own = options.free ? statusOf(options.pages[index]!) : undefined;
    if (own) {
      return {
        state: own.state,
        ...(own.words ? { words: own.words } : {}),
        selected: index === at,
        pressable: index !== at && !(options.locked ?? false),
        failed: own.failed ?? false,
        running: own.running ?? false,
      };
    }
    const first = backTo(options.pages, at, options.locked ?? false);
    return {
      state: stateOf(index, at),
      selected: false,
      pressable: index !== at && index >= first && index <= upTo,
      failed: index === at && (options.failed ?? false),
      running: index === at && (options.locked ?? false),
    };
  };
  const make = (index: number, shown: ReturnType<typeof look>) => {
    const each = options.pages[index]!;
    const element = step({ ...each.step, state: shown.state, ...("words" in shown ? { words: shown.words } : {}), ...(shown.selected ? { selected: true } : {}), ...(shown.failed ? { failed: true } : {}), ...(shown.running ? { running: true } : {}), box: "rail", mark: true, pressable: shown.pressable });
    element.dataset.shown = JSON.stringify(shown);
    // Wired once: whether it takes a press is read as it is pressed, so the rail can change it.
    element.addEventListener("click", () => {
      if (element.dataset.pressable === "true") go(index);
    });
    // Enter stays the page's `Filled` action wherever the keyboard is: Space presses a step.
    element.addEventListener("keydown", (event) => {
      if (event.key === " " && !event.repeat && element.dataset.pressable === "true") {
        event.preventDefault();
        go(index);
      }
    });
    element.dataset.pressable = String(shown.pressable);
    element.dataset.railFailed = String(shown.failed);
    return element;
  };
  const steps = options.pages.map((_, index) => make(index, look(index, current, reached)));
  const doneRuns = (at: number) => (options.free ? runs(options.pages.map((_, index) => look(index, at, 0).state.is === "done")) : null);
  let theWire = wire(steps.length, current, doneRuns(current));
  const column = stack(0, "start", ...steps, theWire);
  column.classList.add("k-rail-steps");
  arrows(column);
  // Cut at its own box: the brand's `space.s20` and the hints' `space.s16` stay clear of the steps
  // scrolling between them.
  const { box: scroller } = scroll(column, { height: "100%", above: 0, below: 0, noGutter: true });
  const hintsRoom = document.createElement("span");
  hintsRoom.className = "k-rail-hints";
  const drawHints = (at: number) => {
    const hints = options.hints(at);
    hintsRoom.replaceChildren(...(hints.length > 0 ? [gap("s16"), inset(hintColumn(hints), 0, "s12")] : []));
  };
  drawHints(current);
  const parts: Element[] = [inset(options.brand, 0, "s12"), gap("s20"), fillingHeight(scroller), hintsRoom];
  if (options.foot) {
    const foot: Element[] = [];
    if (options.foot.power) {
      const power = button({ label: "Power", kind: "Text", size: "Sm", icon: "power_settings_new" });
      power.dataset.power = "";
      foot.push(power);
    }
    foot.push(fill());
    if (options.foot.battery !== undefined) foot.push(battery(options.foot.battery, options.foot.charging));
    parts.push(gap("s16"), inset(hangs(row("s8", "centre", ...foot)), 0, "s12"));
  }
  const box = document.createElement("span");
  box.className = "k-rail";
  box.append(fillingHeight(stack(0, "start", ...parts)));
  probe(box, { role: "component", name: "StepRail", variant: options.host, fixed: { width: true, height: false } });
  let hintsAt = current;
  return {
    element: box,
    set(at, upTo) {
      steps.forEach((element, index) => {
        const shown = look(index, at, upTo);
        // A failed symbol is drawn whole, as is any change to a free step's own standing; every
        // other change, taking a press included, is in place.
        if (element.dataset.railFailed !== String(shown.failed) || (options.free && element.dataset.shown !== JSON.stringify(shown))) {
          const fresh = make(index, shown);
          element.replaceWith(fresh);
          steps[index] = fresh;
          return;
        }
        setStepState(element, shown.state, shown.running);
        if (element.dataset.pressable !== String(shown.pressable)) {
          pressable(element, shown.pressable);
          element.dataset.pressable = String(shown.pressable);
        }
      });
      if (options.free) {
        // In any order the wire's light joins what is done, wherever the reader stands.
        const again = wire(steps.length, at, doneRuns(at));
        if (again.dataset.runs !== theWire.dataset.runs) {
          theWire.replaceWith(again);
          theWire = again;
        }
      } else for (const light of column.querySelectorAll<HTMLElement>(".k-rail-wire-lit")) light.style.setProperty("--wire-lit", String(at));
      // Two pages taking the same keys keep the hints they share as they are.
      if (JSON.stringify(options.hints(at)) !== JSON.stringify(options.hints(hintsAt))) drawHints(at);
      hintsAt = at;
    },
  };
}

/** The runs of two or more steps in a row that are done, as their first and last index. */
function runs(done: readonly boolean[]): [number, number][] {
  const found: [number, number][] = [];
  let start = -1;
  done.forEach((is, index) => {
    if (is && start < 0) start = index;
    if ((!is || index === done.length - 1) && start >= 0) {
      const end = is ? index : index - 1;
      if (end > start) found.push([start, end]);
      start = -1;
    }
  });
  return found;
}

/**
 * The progress wire down the steps' symbols: a length of tube between each symbol and the next,
 * stopping `step_rail.bead` short of both, so the tube runs into each symbol and no line crosses a
 * glyph. Each length is unlit glass with its light over it, filled as far as the light has come:
 * down to `current`, or, in a free flow, along each of `lit`'s runs of done steps.
 */
function wire(count: number, current: number, lit: readonly [number, number][] | null = null): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-rail-wire";
  const joined = (index: number) => lit?.some(([from, to]) => index >= from && index < to) ?? false;
  for (let index = 0; index < count - 1; index++) {
    const track = document.createElement("span");
    track.className = "k-rail-wire-track";
    track.style.setProperty("--i", String(index));
    const light = document.createElement("span");
    light.className = "k-rail-wire-lit";
    light.style.setProperty("--i", String(index));
    // A free flow's light is whole lengths, each lit where both its steps are done.
    light.style.setProperty("--wire-lit", String(lit ? (joined(index) ? index + 1 : index) : current));
    // The two steps it joins, so a move can pick out the light a step just joined.
    light.dataset.steps = `${index} ${index + 1}`;
    box.append(track, light);
  }
  box.dataset.runs = JSON.stringify(lit);
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
  // Only the lengths between the two steps change: each animated length is styled again every frame
  // on the main thread, so the rest are left alone.
  const lights = [...steps.querySelectorAll<HTMLElement>(".k-rail-wire-lit")].slice(Math.min(from, to), Math.max(from, to));
  for (const light of lights) light.animate([{ "--wire-lit": from }, { "--wire-lit": to }], { duration: long, easing: curve("emphasized_decel") });
  const rows = [...steps.querySelectorAll<HTMLElement>(":scope > .k-step")];
  const look = (index: number, at: number, fails: boolean) => ({ "--bead-lit": index <= at ? 1 : 0, "--bead-glow": index < at || (index === at && !fails) ? 1 : 0 });
  rows.forEach((each, index) => {
    const before = look(index, from, false);
    const after = look(index, to, failed);
    if (before["--bead-lit"] === after["--bead-lit"] && before["--bead-glow"] === after["--bead-glow"]) return;
    const passed = to > from ? (index - from) / (to - from) : (from - index) / (from - to);
    const delay = index === to ? timed("medium") : reaching("emphasized_decel", Math.max(0, passed)) * long;
    each.querySelector(".k-step-bead")?.animate([before, after], { duration: timed("short"), delay, easing: curve("standard"), fill: "backwards" });
  });
}

/** ↑ and ↓ (k and j) move the keyboard between the steps that take a press, as they stand now. */
function arrows(column: HTMLElement): void {
  column.addEventListener("keydown", (event) => {
    const by = event.key === "ArrowDown" || event.key === "j" ? 1 : event.key === "ArrowUp" || event.key === "k" ? -1 : 0;
    if (by === 0) return;
    const pressable = [...column.querySelectorAll<HTMLElement>(":scope > .k-step")].filter((each) => each.tabIndex === 0);
    const next = pressable[pressable.indexOf(event.target as HTMLElement) + by];
    if (!next) return;
    event.preventDefault();
    next.focus({ preventScroll: true });
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
 * in `error` as Enter is refused. A note or a reason longer than the room between the buttons
 * breaks between words, so it is read whole and pushes neither button.
 */
export function footer(spec: RailPage, onBack: () => void = () => {}, onNext: () => void = () => {}, pinned: FooterPins = {}, onSkip: () => void = () => {}): { band: HTMLElement; nudge: HTMLElement | null; next: HTMLElement } {
  const back = button({ label: spec.closes ? "Close" : "Back", kind: "Text", icon: spec.closes ? "close" : "arrow_back", ...(pinned.back ? { pinned: pinned.back } : {}) });
  back.addEventListener("click", onBack);
  const parts: Element[] = [back, fill()];
  let nudge: HTMLElement | null = null;
  const blocker = spec.blocker === undefined ? undefined : now(spec.blocker);
  const note = spec.note === undefined ? undefined : now(spec.note);
  if (blocker) {
    const reason = wrapped(blocker, "small");
    reason.style.color = "var(--nudge-ink)";
    nudge = row("s8", "centre", icon("error", "var(--type-small-icon)", "var(--nudge-ink)"), reason);
    nudge.classList.add("k-step-blocker");
    parts.push(nudge);
  } else if (note) {
    const shown = wrapped(note, "small", "on_surface_variant");
    shown.classList.add("k-step-note");
    parts.push(shown);
  }
  const skip = spec.skip === undefined ? undefined : now(spec.skip);
  if (skip) {
    const quiet = button({ label: skip, kind: "Text" });
    quiet.addEventListener("click", onSkip);
    parts.push(quiet);
  }
  const own = spec.nextIcon === undefined ? undefined : now(spec.nextIcon);
  const symbol = own ? { icon: own } : { icon: "arrow_forward", trailing: true };
  const next = button({ label: spec.next === undefined ? "Continue" : now(spec.next), kind: "Filled", ...symbol, ...(blocker ? { disabled: true } : {}), ...(pinned.next ? { pinned: pinned.next } : {}) });
  next.addEventListener("click", onNext);
  const busy = spec.busy === undefined ? undefined : now(spec.busy);
  if (busy) {
    // The button keeps its room, so nothing beside it moves while the work runs.
    next.inert = true;
    const working = row("s8", "centre", spinner("s20"), text(busy, "body", "on_surface_variant"));
    working.classList.add("k-step-busy-words");
    const room = document.createElement("span");
    room.className = "k-step-busy";
    room.append(next, working);
    parts.push(room);
  } else parts.push(next);
  const band = hangs(row("s16", "centre", ...parts));
  band.classList.add("k-step-footer");
  return { band, nudge, next };
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
 * on `emphasized_decel`, its offset on whole device pixels; with reduced motion it only fades. In
 * Kōbō, where the page is the screen, it waits the `short` the page leaving takes to go, so the two
 * never lie over each other, and its footer band follows it in a `micro` later, rising `space.s24`.
 */
export function arrive(room: HTMLElement, forward: boolean, staged = false): void {
  room.dataset.from = forward ? "end" : "start";
  const reduced = settings().reduced;
  const frames: Keyframe[] = reduced ? [{ opacity: 0 }, { opacity: 1 }] : [{ opacity: 0, "--step-slide": 1 }, { opacity: 1, "--step-slide": 0 }];
  const after = staged && !reduced ? timed("short") : 0;
  room.animate(frames, { duration: timed("long"), delay: after, easing: curve("emphasized_decel"), fill: "backwards" });
  const band = staged && !reduced ? room.querySelector<HTMLElement>(".k-step-footer") : null;
  band?.animate([{ opacity: 0, "--step-rise": 1 }, { opacity: 1, "--step-rise": 0 }], { duration: timed("long"), delay: after + timed("micro"), easing: curve("emphasized_decel"), fill: "backwards" });
}

/**
 * In Kōbō, the page left: it slides `space.s32` on, away from where the next comes in, and fades
 * out over `short` on `emphasized_accel`, drawn over the page arriving until it is gone; with
 * reduced motion it goes at once.
 */
export function leave(room: HTMLElement, forward: boolean): void {
  if (settings().reduced) return;
  room.dataset.from = forward ? "start" : "end";
  room.dataset.leaving = "";
  room.animate([{ opacity: 1, "--step-slide": 0 }, { opacity: 0, "--step-slide": 1 }], { duration: timed("short"), easing: curve("emphasized_accel"), fill: "forwards" }).finished.then(
    (run) => {
      delete room.dataset.leaving;
      run.cancel();
    },
    () => {},
  );
}

/** Enter refused: the blocker's reason in `error` at once, fading back over `slow`. */
function flash(nudge: HTMLElement): void {
  nudge.animate([{ "--nudge": 1 }, { "--nudge": 0 }], { duration: timed("slow"), easing: curve("standard_accel") });
}

/** What in a page's body takes the keyboard. */
const TAKES_KEYS = "[tabindex='0'], button:not(:disabled)";

/**
 * The page's room: the clear page, its body above the footer band; what slides and fades. `first`
 * is its first thing to do, where the keyboard lands: the body's first control, or Continue while
 * nothing blocks it.
 */
/** Whether something blocks Continue on `spec` now. */
function blocked(spec: RailPage): boolean {
  return spec.blocker !== undefined && Boolean(now(spec.blocker));
}

function pageRoom(spec: RailPage, onBack: () => void, onNext: () => void, onSkip: () => void): { room: HTMLElement; nudge: () => HTMLElement | null; first: HTMLElement | null } {
  const body = spec.body();
  const parts: Element[] = [fillingHeight(filling(body))];
  let nudge: HTMLElement | null = null;
  let first = body.querySelector<HTMLElement>(spec.lands ?? TAKES_KEYS);
  const room = document.createElement("span");
  const made = { nudge: null as HTMLElement | null };
  if (!spec.footless) {
    const drawn = footer(spec, onBack, onNext, {}, onSkip);
    nudge = drawn.nudge;
    made.nudge = nudge;
    first ??= blocked(spec) ? null : drawn.next;
    let band = filling(drawn.band);
    parts.push(gap("s16"), band);
    // The answers changed: the band is drawn again where it was, the keyboard kept on Continue.
    room.addEventListener(RECHECK, () => {
      const had = band.contains(document.activeElement);
      const again = footer(spec, onBack, onNext, {}, onSkip);
      const fresh = filling(again.band);
      band.replaceWith(fresh);
      band = fresh;
      made.nudge = again.nudge;
      if (had) again.next.focus({ preventScroll: true });
    });
  }
  room.className = "k-step-room";
  room.append(page(fillingHeight(stack(0, "start", ...parts)), { clear: true }));
  return { room, nudge: () => made.nudge, first };
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
  const shown: { nudge: () => HTMLElement | null } = { nudge: () => null };

  // Every page drawn so far, kept in the page's side, hidden, laid out: going back to one only
  // shows it again, and the page before and the first done step are drawn in idle time, so a press
  // on either builds nothing. A page whose body reads the whole flow (`fresh`) is drawn each time.
  const side = document.createElement("span");
  side.className = "k-step-side";
  const rooms = new Map<number, ReturnType<typeof pageRoom>>();
  const roomOf = (index: number): ReturnType<typeof pageRoom> | null => {
    const spec = options.pages[index];
    if (!spec) return null;
    const kept = rooms.get(index);
    if (kept && !spec.fresh) {
      kept.room.dispatchEvent(new Event(RECHECK));
      return kept;
    }
    kept?.room.remove();
    const skip = () => {
      spec.skipped?.();
      go(along(current, 1));
    };
    const made = pageRoom(spec, () => go(along(current, -1)), next, skip);
    rooms.set(index, made);
    side.append(made.room);
    return made;
  };
  // The rail is drawn once; each page change moves its steps in place.
  let theRail: Rail | null = null;
  const showOnly = (index: number) => {
    for (const [at, made] of rooms) {
      const hidden = at !== index;
      made.room.toggleAttribute("data-kept", hidden);
      made.room.inert = hidden;
    }
  };

  const draw = (from: number | null) => {
    const made = roomOf(current);
    if (!made) return;
    showOnly(current);
    shown.nudge = made.nudge;
    const scrolled = box.querySelector(".k-rail .k-scroll-view")?.scrollTop ?? 0;
    // Drawing the rail again replaces what has the keyboard, which would drop it out of the template.
    // A page change always comes from a press in the template, so a keyboard nowhere (the page's
    // own redraw took what had it, as Enter on a list does) was in it too.
    const keyboardIn = box.contains(document.activeElement) || document.activeElement === document.body;
    if (!theRail) {
      theRail = rail(options, current, reached, go);
      box.replaceChildren(theRail.element, side);
    } else theRail.set(current, reached);
    const steps = box.querySelector<HTMLElement>(".k-rail-steps");
    // Kept in view once the frame lays it out, not read back inside the press that drew it.
    if (steps) requestAnimationFrame(() => keepInView(steps, current, scrolled));
    if (from === null) {
      ahead();
      return;
    }
    const left = rooms.get(from)?.room;
    if (options.host === "kobo" && left && left !== made.room) leave(left, current > from);
    arrive(made.room, current > from, options.host === "kobo");
    if (steps && !options.free) travel(steps, from, current, options.failed ?? false);
    // The template holds the keys on a page with nothing to do yet: Enter and Esc still work. The
    // keyboard moves with the next frame, which lays the page out anyway, not inside the press.
    if (keyboardIn) {
      // The rail is moved in place, so what had the keyboard still has it this frame.
      requestAnimationFrame(() => {
        if (box.contains(document.activeElement) || document.activeElement === document.body) (made.first ?? box).focus({ preventScroll: true });
      });
    }
    ahead();
  };

  /** Draws, one an idle moment, the pages a press most likely goes back to. */
  const ahead = () => {
    const first = backTo(options.pages, current, options.locked ?? false);
    // The two a press most often takes: the page before (Back, Esc) and the first done step.
    const waiting = [...new Set([current - 1, first])].filter((index) => index >= first && index <= reached && index !== current && !options.pages[index]?.fresh);
    const one = () => {
      const index = waiting.shift();
      if (index === undefined || !box.isConnected) return;
      if (!rooms.has(index)) {
        roomOf(index);
        showOnly(current);
      }
      requestIdleCallback(one);
    };
    requestIdleCallback(one);
  };

  function go(to: number): void {
    if (options.onExit && !options.locked && (to === -1 || to === options.pages.length)) {
      options.onExit(to === options.pages.length);
      return;
    }
    if ((!options.free && to < backTo(options.pages, current, options.locked ?? false)) || to < 0 || to >= options.pages.length || to === current) return;
    const from = current;
    current = to;
    reached = Math.max(reached, current);
    draw(from);
  }

  function next(): void {
    const spec = options.pages[current];
    if (spec && blocked(spec)) {
      const nudge = shown.nudge();
      if (nudge) flash(nudge);
      return;
    }
    if (spec?.busy !== undefined && now(spec.busy)) return;
    if (spec?.act?.()) return;
    go(along(current, 1));
  }

  /** The page `by` one way from `from`, past any a free flow says isn't there. */
  function along(from: number, by: 1 | -1): number {
    let to = from + by;
    while (options.free && to >= 0 && to < options.pages.length && statusOf(options.pages[to]!)?.out) to += by;
    return to;
  }

  box.addEventListener("keydown", (event) => {
    if (event.defaultPrevented || event.repeat) return;
    if (event.key === "Enter") next();
    else if (event.key === "Escape") go(along(current, -1));
    else return;
    event.preventDefault();
  });

  box.addEventListener(GO_ON, () => next());

  // A page's answers or its work changed: in a free flow its step stands where the page says.
  if (options.free) box.addEventListener(RECHECK, () => theRail?.set(current, reached));
  draw(null);
  if (options.nudged !== undefined) shown.nudge()?.style.setProperty("--nudge", String(options.nudged));
  if (options.leaving !== undefined && options.leaving !== current) {
    const left = roomOf(options.leaving);
    showOnly(current);
    if (left) {
      left.room.dataset.leaving = "";
      left.room.dataset.from = options.leaving < current ? "start" : "end";
    }
  }
  if (options.arriving) {
    const room = box.querySelector<HTMLElement>(".k-step-room:not([data-kept])");
    if (room) {
      room.dataset.from = options.arriving.forward ? "end" : "start";
      room.style.opacity = String(options.arriving.shown);
      room.style.setProperty("--step-slide", String(1 - options.arriving.shown));
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

/** Kōbō's screen: the template filling it, on glass over the blurred wallpaper (`Backdrop`). */
function koboPanel(template: HTMLElement, pictures: Backdrop): HTMLElement {
  return backdrop({ width: "100%", height: "100%", picture: pictures.blurred, scrim: "glass", content: template });
}
