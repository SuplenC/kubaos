// The full-screen tool: a sample that is a whole screen shown on the whole real screen, live and at
// true size, so its motion, hit targets, hover edges and drags are judged as on a desktop.
//
// A sample is a screen when it is, or holds as its one outermost box, an element whose size is a
// screen's: the library's screens set it as `--screen-width` and `--screen-height` (`lib/sheet.ts`'s
// screen, the lock on it, the wallpaper picker's) or `--backdrop-width` and `--backdrop-height`
// (`lib/backdrop.ts`). Their CSS lays every part out from those two, so setting them to the real
// screen's size lays the screen out for it, not a picture of the small one scaled up.
//
// Shown, the sample is built afresh from its own description, in the mode it showed in and at the
// toolbar's settings, in a room over the whole page that the browser's full screen then takes. A
// tap of Esc goes to the mockup (Keyboard Lock, where the browser has it), and holding Esc leaves,
// as the browser's own full screen does; Ctrl+Esc always leaves. Leaving releases all of it and
// puts focus and the scroll back where they were.

import { el } from "./dom.ts";
import type { Move } from "./page.ts";
import { keyframes, FRAME, loop } from "./recipe.ts";
import { paint, palette } from "./recolour.ts";
import { ALL, apply, CHROME, type Mode, settings } from "./settings.ts";
import { settle } from "./settle.ts";

/** The properties a screen's size is set by, width then height. */
const SIZES = [
  ["--screen-width", "--screen-height"],
  ["--backdrop-width", "--backdrop-height"],
] as const;

/** An element with a screen's size set on it. */
const SIZED = SIZES.map(([width]) => `[style*="${width}:"]`).join(", ");

/** The screen `root` is, or holds as its one outermost sized box; null for none, or several. */
function screenIn(root: Element): HTMLElement | null {
  if (root instanceof HTMLElement && root.matches(SIZED)) return root;
  const outermost = [...root.querySelectorAll<HTMLElement>(SIZED)].filter((each) => {
    const above = each.parentElement?.closest(SIZED);
    return !above || !root.contains(above);
  });
  return outermost.length === 1 ? outermost[0]! : null;
}

/** Whether `drawn`, a sample's elements, is one whole screen. */
export function isScreen(drawn: readonly Element[]): boolean {
  return drawn.length === 1 && screenIn(drawn[0]!) !== null;
}

/** A move playing on the sample shown: how to stop it, and the keys that drive it. */
interface Playing {
  readonly stop: () => void;
  readonly key: (event: KeyboardEvent) => boolean;
}

/** What to show: the sample built afresh, the mode it is painted in, and a move to play on it. */
export interface Opening {
  readonly make: () => Element;
  readonly mode: Mode;
  readonly play?: (sample: Element) => Playing;
}

/** The Keyboard Lock API, which TypeScript's DOM types leave out: Chromium has it in full screen. */
interface KeyboardLock {
  lock(keys?: string[]): Promise<void>;
  unlock(): void;
}

function keyboardLock(): KeyboardLock | undefined {
  return (navigator as Navigator & { keyboard?: KeyboardLock }).keyboard;
}

/** How to leave what is shown now, or null. */
let leave: (() => void) | null = null;

/**
 * The control that shows `opening()` on the whole screen, as the site's other sample controls look;
 * null where the site draws no chrome (`?chrome=off`) or every choice at once (`?all`), so the shots
 * and the checks never see it.
 */
export function fullScreenControl(opening: () => Opening): HTMLButtonElement | null {
  if (!CHROME || ALL) return null;
  const control = el("button", "site-control page-full-screen", "Full screen");
  control.type = "button";
  control.title = "Full screen: Esc goes to the mockup; hold Esc or press Ctrl+Esc to leave";
  control.addEventListener("click", () => show(opening(), control));
  return control;
}

/** The control for a move's sample, playing from the player's place at its speed; null for none. */
export function fullScreenMove(move: Move, sample: Element, at: () => number, rate: () => number): HTMLButtonElement | null {
  if (!isScreen([sample])) return null;
  return fullScreenControl(() => ({ make: move.sample, mode: settings().mode, play: (shown) => played(move, shown, at(), rate()) }));
}

/**
 * `move` looping on `sample` from `t`, as its player plays it: a picture of the move, so it takes
 * no pointer or focus. Space pauses and plays, ← and → step a frame, Home starts it again.
 */
function played(move: Move, sample: Element, t: number, rate: number): Playing {
  sample.setAttribute("inert", "");
  const playing = loop(move);
  const animations = playing.timed.flatMap((timed) =>
    [...sample.querySelectorAll<HTMLElement>(timed.track.target)].map((target) =>
      target.animate(keyframes(timed, playing.total), { duration: playing.total, iterations: Number.POSITIVE_INFINITY, fill: "both" }),
    ),
  );
  for (const animation of animations) {
    animation.playbackRate = rate;
    animation.currentTime = t;
  }
  const seek = (to: number) => {
    for (const animation of animations) {
      animation.pause();
      animation.currentTime = ((to % playing.total) + playing.total) % playing.total;
    }
  };
  const now = () => Number(animations[0]?.currentTime ?? 0) % playing.total;
  const key = (event: KeyboardEvent): boolean => {
    if (event.key === " ") {
      const paused = animations[0]?.playState === "paused";
      for (const animation of animations) {
        if (paused) animation.play();
        else animation.pause();
      }
    } else if (event.key === "ArrowLeft" || event.key === "ArrowRight") seek((Math.round(now() / FRAME) + (event.key === "ArrowLeft" ? -1 : 1)) * FRAME);
    else if (event.key === "Home") seek(0);
    else return false;
    return true;
  };
  return { stop: () => animations.forEach((animation) => animation.cancel()), key };
}

/** The screen's size, as its properties and the values they are set to. */
interface Size {
  readonly width: string;
  readonly height: string;
  readonly was: readonly [string, string];
}

function sizeOf(screen: HTMLElement): Size {
  const [width, height] = SIZES.find(([each]) => screen.style.getPropertyValue(each) !== "")!;
  return { width, height, was: [screen.style.getPropertyValue(width), screen.style.getPropertyValue(height)] };
}

/**
 * `elements` sized as the screen is: the screen, and a screen drawn inside it at its size, as the
 * wallpaper picker draws the lock on its wall.
 */
function sizedAs(size: Size, elements: Iterable<HTMLElement>): HTMLElement[] {
  return [...elements].filter((each) => each.style.getPropertyValue(size.width) === size.was[0] && each.style.getPropertyValue(size.height) === size.was[1]);
}

/** `root` and every element in it with a screen's size set. */
function withSizes(root: Element): HTMLElement[] {
  return [...(root instanceof HTMLElement ? [root] : []), ...root.querySelectorAll<HTMLElement>(SIZED)];
}

/** Shows `opening` on the whole screen, until it is left; focus goes back to `from`. */
function show(opening: Opening, from: HTMLElement): void {
  if (leave) return;
  const scrolled = { x: scrollX, y: scrollY };
  const room = el("div", "mock site-full-screen");
  apply(room);
  room.dataset.mode = opening.mode;
  paint(room, palette(settings().theme, opening.mode));
  // As the page's box is: a CSS pixel inside it is one device pixel.
  room.style.zoom = String(1 / devicePixelRatio);
  room.tabIndex = -1;
  const sample = opening.make();
  const screen = screenIn(sample);
  if (!screen) throw new Error("a full screen was asked of a sample that isn't a screen");
  const size = sizeOf(screen);
  const sized = sizedAs(size, withSizes(sample));
  room.append(sample);
  document.body.append(room);

  /** The room's size, its own pixels: the screen's, once the browser's full screen has it. */
  const fit = (): void => {
    for (const each of sized) {
      each.style.setProperty(size.width, `${room.clientWidth}px`);
      each.style.setProperty(size.height, `${room.clientHeight}px`);
    }
    settle(room);
  };
  const resized = new ResizeObserver(fit);
  resized.observe(room);
  // A screen the sample draws later at its size (the wallpaper picker's wall, built for each view)
  // takes the real screen's too.
  const added = new MutationObserver((records) => {
    const nodes = records.flatMap((record) => [...record.addedNodes]).filter((node): node is HTMLElement => node instanceof HTMLElement);
    const more = sizedAs(size, nodes.flatMap(withSizes)).filter((each) => !sized.includes(each));
    if (more.length === 0) return;
    sized.push(...more);
    fit();
  });
  added.observe(room, { childList: true, subtree: true });
  fit();
  const playing = opening.play?.(sample);

  const keys = (event: KeyboardEvent): void => {
    if (event.key === "Escape" && event.ctrlKey) {
      event.preventDefault();
      event.stopImmediatePropagation();
      close();
      return;
    }
    if (playing && room.contains(event.target as Node) && playing.key(event)) event.preventDefault();
  };
  const changed = (): void => {
    if (document.fullscreenElement !== room) close();
  };
  let full = false;

  const close = (): void => {
    leave = null;
    removeEventListener("keydown", keys, { capture: true });
    document.removeEventListener("fullscreenchange", changed);
    resized.disconnect();
    added.disconnect();
    playing?.stop();
    for (const animation of room.getAnimations({ subtree: true })) animation.cancel();
    if (full) keyboardLock()?.unlock();
    if (document.fullscreenElement === room) void document.exitFullscreen();
    room.remove();
    from.focus({ preventScroll: true });
    scrollTo(scrolled.x, scrolled.y);
    // Leaving the browser's full screen lays the page out again a frame later.
    requestAnimationFrame(() => scrollTo(scrolled.x, scrolled.y));
  };
  leave = close;
  addEventListener("keydown", keys, { capture: true });
  (screen.tabIndex >= 0 ? screen : room).focus({ preventScroll: true });

  // Where the browser refuses its full screen, or has none, the room stays over the whole window,
  // and Ctrl+Esc leaves it.
  room
    .requestFullscreen?.({ navigationUI: "hide" })
    ?.then(() => {
      if (leave !== close) return;
      full = true;
      document.addEventListener("fullscreenchange", changed);
      return keyboardLock()?.lock(["Escape"]);
    })
    .catch(() => undefined);
}
