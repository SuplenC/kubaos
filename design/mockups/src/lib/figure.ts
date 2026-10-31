// A number that ticks: QML's `Figure` (`docs/design/components/figure.md`). Geist's figures are
// proportional, so the box is as wide as the same text with every digit a 0, the widest, and the
// text starts at its left edge: what sits beside it holds still as the digits change. Only digits
// are held; a colon, a sign or a unit takes its own width.

import { advance, text, type TypeStyle } from "./text.ts";

/** `words` with every digit a 0: the widest the same shape gets. */
export function widest(words: string): string {
  return words.replace(/\d/g, "0");
}

/** `words` in `style` and `color`, in a box as wide as `widest(words)`. */
export function figure(words: string, style: TypeStyle, color = "on_surface"): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-figure";
  box.append(text(words, style, color));
  hold(box, style, words);
  return box;
}

/** Shows `words` in a figure made by `figure`, its box held as wide as their widest. */
export function tick(box: HTMLElement, style: TypeStyle, words: string): void {
  const shown = box.firstElementChild;
  if (!(shown instanceof HTMLElement)) return;
  shown.textContent = words;
  shown.style.setProperty("--natural", String(advance(style, words)));
  hold(box, style, words);
}

function hold(box: HTMLElement, style: TypeStyle, words: string): void {
  box.style.setProperty("--natural", String(advance(style, widest(words))));
  box.style.setProperty("--t-line", `var(--type-${style}-line)`);
}
