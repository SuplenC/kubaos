// Words in a fixed room that scroll when they run past it: `Marquee`, new, in neither library yet.
// Its room is a size token wide whatever the words, so nothing beside it moves when they change.
// Words that fit sit still at its start. Words that run past it hold at their start for
// `marquee_hold`, then scroll left at `space.s32` per `marquee_pace`, a copy `space.s32` behind
// them, until the copy stands where they began; that frame is the first one again, so the loop
// never jumps, and it holds again. A soft fade `space.s16` wide is drawn only at an edge where
// words are cut: the end always while they run past it, the start once they have moved, growing
// with their first `space.s16` of travel and shrinking as the copy lands.
//
// Held (paused music), the words glide to the nearer start over `medium` and stay there. New words
// take the room as the old fade out over `micro`; they fade in over `short` and rise `space.s8`
// over `medium` after it, and their scroll starts with its hold. With reduce motion nothing
// scrolls: words that run past end in an ellipsis at the room's edge.

import "./marquee.css";
import { probe } from "./probe.ts";
import { text, type TypeStyle } from "./text.ts";
import { durationMs, length, raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** One run of words in its style and colour: a song, a separator, an artist. */
export interface MarqueePart {
  readonly words: string;
  readonly style: TypeStyle;
  readonly color: string;
}

export interface MarqueeOptions {
  readonly parts: readonly MarqueePart[];
  /** The `size` token its room is: `now_playing`. */
  readonly room: string;
  /** Held at its start: nothing scrolls until it is let go. */
  readonly held?: boolean;
  /** Pinned this many `space.s32` along a pass, for a still sample: nothing plays. */
  readonly pinned?: number;
}

export interface Marquee {
  readonly element: HTMLElement;
  /** Puts new words in the room: the old fade out, the new fade in and rise, then scroll. */
  readonly change: (parts: readonly MarqueePart[]) => void;
  /** Holds it at its start, gliding there, or lets it scroll again from its hold. */
  readonly hold: (held: boolean) => void;
}

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return durationMs(name) / settings().speed;
}

function words(parts: readonly MarqueePart[], room: number): { readonly line: HTMLElement; readonly runs: boolean } {
  const line = document.createElement("span");
  line.className = "k-marquee-line";
  let used = 0;
  for (const part of parts) {
    const made = text(part.words, part.style, part.color);
    // Under reduce motion the words stop at the room's edge: the part that crosses it ends in an
    // ellipsis and those after it go, so the cut falls where a reader stops.
    if (used >= room) made.dataset.marqueePast = "";
    else if (used + Number(made.style.getPropertyValue("--natural")) > room) made.dataset.marqueeCut = "";
    used += Number(made.style.getPropertyValue("--natural"));
    line.append(made);
  }
  return { line, runs: used > room };
}

/** The words and, when they run past the room, their copy `space.s32` behind them. */
function track(parts: readonly MarqueePart[], room: number): { readonly element: HTMLElement; readonly runs: boolean } {
  const element = document.createElement("span");
  element.className = "k-marquee-track";
  const { line, runs } = words(parts, room);
  element.append(line);
  if (runs) {
    const copy = line.cloneNode(true) as HTMLElement;
    copy.dataset.copy = "";
    copy.setAttribute("aria-hidden", "true");
    element.append(copy);
  }
  element.toggleAttribute("data-runs", runs);
  return { element, runs };
}

/**
 * How many `space.s32` one pass travels: from the words' start to their copy's, measured as laid
 * out, since each part's width is rounded to a device pixel and the pass must land exactly.
 */
function passOf(element: HTMLElement): number {
  const [line, copy] = element.querySelectorAll<HTMLElement>(":scope > .k-marquee-line");
  if (!line || !copy) return 0;
  const step = Number.parseFloat(getComputedStyle(copy).marginInlineStart);
  return step > 0 ? (copy.offsetLeft - line.offsetLeft) / step : 0;
}

export function marquee(options: MarqueeOptions): Marquee {
  const room = length(`size-${options.room}`);
  const element = document.createElement("span");
  element.className = "k-marquee";
  element.style.setProperty("--marquee-room", `var(--size-${options.room})`);
  let held = options.held ?? false;
  let current = track(options.parts, room);
  let scroll: Animation | null = null;
  // What the words scroll in, as a toolkit's scrollable: the room is the component's, fixed.
  const window_ = document.createElement("span");
  window_.className = "k-marquee-window";
  window_.append(current.element);
  element.append(window_);
  element.setAttribute("aria-label", options.parts.map((part) => part.words).join(""));

  function stop(): void {
    scroll?.cancel();
    scroll = null;
  }

  /** One pass after another, each holding at its start, until it is held or the words change. */
  function play(): void {
    stop();
    if (held || !current.runs || !element.isConnected) return;
    const pass = passOf(current.element);
    if (pass <= 0) return;
    element.style.setProperty("--marquee-end", String(pass));
    const hold = ms("marquee_hold");
    const total = hold + pass * ms("marquee_pace");
    scroll = element.animate(
      [
        { "--marquee-x": 0, offset: 0 },
        { "--marquee-x": 0, offset: hold / total },
        { "--marquee-x": pass, offset: 1 },
      ],
      { duration: total, iterations: Infinity, easing: "linear" },
    );
  }

  function hold(next: boolean): void {
    if (next === held) return;
    held = next;
    if (!held) return play();
    const at = Number(getComputedStyle(element).getPropertyValue("--marquee-x")) || 0;
    const pass = Number(element.style.getPropertyValue("--marquee-end")) || 0;
    stop();
    // The copy at its start is the same frame as the words at theirs: past halfway, going on is the shorter glide.
    const to = at > pass / 2 ? pass : 0;
    const glide = element.animate([{ "--marquee-x": at }, { "--marquee-x": to }], { duration: ms("medium"), easing: raw("easing-standard") });
    glide.finished.then(() => element.style.setProperty("--marquee-x", "0"), () => {});
    element.style.setProperty("--marquee-x", "0");
  }

  function change(parts: readonly MarqueePart[]): void {
    const old = current.element;
    // The old words fade where they stand, not from the start the room goes back to.
    old.style.setProperty("--marquee-x", getComputedStyle(element).getPropertyValue("--marquee-x"));
    stop();
    current = track(parts, room);
    const fresh = current.element;
    element.setAttribute("aria-label", parts.map((part) => part.words).join(""));
    element.style.setProperty("--marquee-x", "0");
    if (settings().reduced) {
      old.replaceWith(fresh);
      return play();
    }
    old.dataset.out = "";
    fresh.dataset.in = "";
    window_.append(fresh);
    old.animate([{ opacity: 1 }, { opacity: 0 }], { duration: ms("micro"), easing: raw("easing-standard_accel"), fill: "forwards" }).finished.then(() => old.remove(), () => {});
    const cue = ms("micro");
    fresh.animate([{ "--marquee-fade": 0 }, { "--marquee-fade": 1 }], { duration: ms("short"), delay: cue, easing: raw("easing-standard"), fill: "backwards" });
    const rise = fresh.animate([{ "--marquee-rise": 0 }, { "--marquee-rise": 1 }], { duration: ms("medium"), delay: cue, easing: raw("easing-emphasized_decel"), fill: "backwards" });
    rise.finished.then(() => {
      delete fresh.dataset.in;
      if (current.element === fresh) play();
    }, () => {});
  }

  if (options.pinned !== undefined) {
    element.style.setProperty("--marquee-x", String(options.pinned));
    requestAnimationFrame(() => element.style.setProperty("--marquee-end", String(passOf(current.element))));
  } else {
    // It starts once it is on the page, where its parts are laid out and a pass can be measured.
    requestAnimationFrame(() => requestAnimationFrame(play));
  }
  // Where its words stand is theirs, scrolled, not the component's: it holds them.
  probe(element, { role: "component", name: "Marquee", variant: options.room, fixed: { width: true, height: true, holds: true } });
  return { element, change, hold };
}

/**
 * A marquee caught as new words take its room: the old track and the new both in it, at the start
 * of the change, for a move's player to play.
 */
export function marqueeChanging(from: readonly MarqueePart[], to: readonly MarqueePart[], room: string): HTMLElement {
  const made = marquee({ parts: from, room, pinned: 0 });
  const old = made.element.querySelector<HTMLElement>(".k-marquee-track")!;
  old.dataset.out = "";
  const fresh = track(to, length(`size-${room}`)).element;
  fresh.dataset.in = "";
  fresh.style.setProperty("--marquee-fade", "0");
  fresh.style.setProperty("--marquee-rise", "0");
  old.after(fresh);
  return made.element;
}
