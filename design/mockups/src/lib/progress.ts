// Work that knows how far it is: `kubaos-ui`'s `progress::bar`, `danger_bar`, `timed_bar` and
// `danger_timed_bar`, `KubaOS.Components`' `ProgressBar` (`docs/design/components/progress-bar.md`).
// A pill `size.progress` tall across its room: the track, and over it the fill from the left edge
// to the work's fraction, rounded up to a device pixel, glowing in its own colour with the signs
// on. `feedback.css` draws it.
//
// The fill follows the work over `short` on `linear`, so it moves with the work instead of
// jumping; a bar that shows up already part full starts there. A timed bar is drawn where it is
// given: the app already times it against the clock (how long a button has been held).

import "./feedback.css";
import { probe } from "./probe.ts";

export interface ProgressOptions {
  /** How far the work is, 0 to 1. */
  readonly fraction: number;
  /** On a danger banner's red: the fill in `error` on a track of `on_error_container`'s tint. */
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
