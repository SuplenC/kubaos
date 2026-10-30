// The theme change and the dark ⇄ light switch across the whole page, as the toolbar's theme and
// mode pick them: the same moves the players show (`site/sequences.ts`), played once on the page's
// one clock on the document and the page's box, which every colour and every light follows. Reduced
// motion swaps the colours at once; the speed scales the whole sequence.

import type { Track } from "./page.ts";
import { between, bezier, cueOf, milliseconds, playing, stepsOf, struck, tracksOf } from "./recipe.ts";
import { paint, paintBetween, paintGlass, palette, unpaint } from "./recolour.ts";
import { themeChange, toDark, toLight } from "./sequences.ts";
import { type Mode, settings } from "./settings.ts";
import { THEMES } from "./themes.ts";

/** A palette: a theme in a mode. */
export interface Painted {
  readonly theme: string;
  readonly mode: Mode;
}

let running: { animations: Animation[]; done: () => void } | null = null;

/** Paints `roots` in `to` for good: the tokens' own palette is the mode's, so it is taken off. */
function settleOn(roots: readonly HTMLElement[], to: Painted): void {
  for (const root of roots) {
    unpaint(root);
    root.style.removeProperty("--before");
    root.removeAttribute("data-sequence");
    if (to.theme !== THEMES[0]!.slug) paint(root, palette(to.theme, to.mode));
  }
}

/** One track played once on `element`, from its cue, at the speed picked. */
function playTrack(element: HTMLElement, track: Track, speed: number): Animation {
  const value = (part: number) => between(track, part);
  const frames: Keyframe[] = struck(track.curve)
    ? [...stepsOf(track.curve).map((step, i, all) => ({ offset: i / all.length, easing: "step-end", [track.property]: value(step) })), { offset: 1, [track.property]: value(1) }]
    : [
        { offset: 0, easing: `cubic-bezier(${bezier(track.curve).join(", ")})`, [track.property]: value(0) },
        { offset: 1, [track.property]: value(1) },
      ];
  return element.animate(frames, { delay: cueOf(track) / speed, duration: milliseconds(track.duration) / speed, fill: "both" });
}

/** Paints `roots` in `to` at once, as the page is drawn. */
export function paintPage(roots: readonly HTMLElement[], to: Painted): void {
  running?.done();
  settleOn(roots, to);
}

/** Plays the change from `from` to `to` on `roots`: the theme change, or the switch to light or dark. */
export function playChange(roots: readonly HTMLElement[], from: Painted, to: Painted): void {
  running?.done();
  const { reduced, speed } = settings();
  const switched = from.mode !== to.mode;
  const words = { from: "", to: "" };
  const move = switched ? (to.mode === "light" ? toLight(words, () => roots[0]!) : toDark(words, () => roots[0]!)) : themeChange("", words, () => roots[0]!);
  const tracks = tracksOf(move, playing(move)) ?? move.tracks;
  if (reduced) {
    settleOn(roots, to);
    return;
  }
  const path = tracks.find((track) => track.colours)?.colours?.path ?? "through grey";
  const animations: Animation[] = [];
  for (const root of roots) {
    paintBetween(root, path, palette(from.theme, from.mode), palette(to.theme, to.mode));
    if (switched && to.mode === "light") paintGlass(root, palette(to.theme, to.mode), to.mode);
    // The mode's attribute changes at once: the lights before the change are the old mode's.
    if (switched) root.style.setProperty("--before", from.mode === "dark" ? "1" : "0");
    root.setAttribute("data-sequence", "");
    for (const track of tracks) animations.push(playTrack(root, track, speed));
  }
  const length = Math.max(...tracks.map((track) => cueOf(track) + milliseconds(track.duration))) + (move.tail ? milliseconds(move.tail) : 0);
  const timer = setTimeout(() => done(), length / speed);
  const done = () => {
    clearTimeout(timer);
    for (const animation of animations) animation.cancel();
    settleOn(roots, to);
    running = null;
  };
  running = { animations, done };
}
