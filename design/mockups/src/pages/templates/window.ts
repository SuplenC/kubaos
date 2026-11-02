// The window a template page shows its template in: an app's window, which the page's reader
// resizes by its corner from the app's minimum up to a full 1080p screen, its size in logical
// pixels under it; and a still window of a fixed size for the samples.

import "./templates.css";
import { el, label } from "../../site/dom.ts";
import { settings, windowSize } from "../../site/settings.ts";

/**
 * `content` in a window that opens at the toolbar's window size and resizes by its corner, held
 * between `size.app_min_width` × `size.app_min_height` and a 1080p screen.
 */
export function resizable(content: HTMLElement): HTMLElement {
  const size = windowSize();
  const frame = el("div", "t-window", content);
  frame.dataset.resizes = "";
  frame.style.width = size.width;
  frame.style.height = size.height;
  const words = label("", "mono", "on_surface_variant");
  new ResizeObserver(() => {
    const scale = settings().scale;
    words.textContent = `${Math.round(frame.clientWidth / scale)} × ${Math.round(frame.clientHeight / scale)} · drag the corner to resize`;
  }).observe(frame);
  return el("div", "t-room", frame, words);
}

/** `content` in a still window `width` × `height` (lengths' CSS): a state's sample. */
export function still(content: HTMLElement, width = "var(--size-app_min_width)", height = "var(--size-app_min_height)"): HTMLElement {
  const frame = el("div", "t-window", content);
  frame.style.width = width;
  frame.style.height = height;
  return frame;
}
