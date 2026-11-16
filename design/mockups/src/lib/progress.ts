// Work that knows how far it is: `kubaos-ui`'s `progress::bar`, `danger_bar`, `timed_bar` and
// `danger_timed_bar`, `KubaOS.Components`' `ProgressBar` (`docs/design/components/progress-bar.md`).
// A pill `size.progress` tall across its room: the track, and over it the fill from the left edge
// to the work's fraction, rounded up to a device pixel, glowing in its own colour with the signs
// on. `feedback.css` draws it.
//
// The fill follows the work over `short` on `linear`, so it moves with the work instead of
// jumping; a bar that shows up already part full starts there. A timed bar is drawn where it is
// given: the app already times it against the clock (how long a button has been held).
//
// A bar can carry a caption over its end (how long is left): `space.s8` above it, its end on the
// bar's end, in `small` after its symbol, on a line that keeps a line's height with nothing on it.

import "./feedback.css";
import { icon } from "./icon.ts";
import { filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { wrapped } from "./text.ts";

export interface ProgressOptions {
  /** How far the work is, 0 to 1. */
  readonly fraction: number;
  /** In a Danger banner: the fill in `error` on a track of `on_error_container`'s tint. */
  readonly danger?: boolean;
  /** Drawn where it is given, with no easing of its own to trail behind it. */
  readonly timed?: boolean;
}

/** Sets how far `bar`, a progress bar, is filled: it eases there unless it is timed. */
export function setProgress(bar: HTMLElement, fraction: number): void {
  bar.style.setProperty("--progress", String(Math.min(1, Math.max(0, fraction))));
}

export function progressBar(options: ProgressOptions): HTMLElement {
  const bar = document.createElement("span");
  bar.className = "k-progress";
  bar.setAttribute("role", "progressbar");
  bar.toggleAttribute("data-danger", options.danger ?? false);
  bar.toggleAttribute("data-timed", options.timed ?? false);
  setProgress(bar, options.fraction);
  const fill = document.createElement("span");
  fill.className = "k-progress-fill";
  // The fill's row, as `kubaos-ui` reports it to the geometry check.
  const parts = probe(document.createElement("span"), { role: "layout", axis: "x", gap: 0, align: "start" });
  parts.className = "k-progress-row";
  parts.append(fill);
  bar.append(parts);
  return probe(bar, { role: "component", name: "ProgressBar", variant: options.danger ? "danger" : "", fixed: { width: false, height: true } });
}

/** What a bar's caption says: its words, after a symbol if it has one. */
export interface Caption {
  readonly words: string;
  readonly icon?: string;
}

/**
 * Says `caption` over the bar's end in `captioned`, or nothing: the line keeps its height either
 * way, so the bar under it never moves. Its words are drawn anew, as a text's box is as wide as the
 * words it was made with.
 */
export function setCaption(captioned: HTMLElement, caption: Caption | null): void {
  const line = captioned.querySelector(":scope > .k-progress-caption");
  if (!line) return;
  if (!caption) {
    line.replaceChildren();
    return;
  }
  const words = wrapped(caption.words, "small", "on_surface_variant");
  line.replaceChildren(caption.icon ? row("s8", "centre", icon(caption.icon, "var(--type-small-icon)", "var(--color-on_surface_variant)"), words) : words);
}

/**
 * `bar` with a caption over its end: the caption's line `space.s8` above it, as wide as the bar,
 * the caption's end on the bar's end, its words breaking from the end where the line is short. Its
 * own line, so nothing under the bar shares room with it.
 */
export function captionedBar(bar: HTMLElement, caption: Caption | null): HTMLElement {
  const line = document.createElement("span");
  line.className = "k-progress-caption";
  const box = stack("s8", "start", filling(line), bar);
  box.classList.add("k-progress-captioned");
  setCaption(box, caption);
  return box;
}
