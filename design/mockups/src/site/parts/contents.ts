// The page's contents: a chip per section in a row pinned under the toolbar, which jumps to its
// section and is on while that section is the one read, the last whose heading has scrolled up to
// the row.

import { el } from "../dom.ts";
import { settings } from "../settings.ts";
import { picker } from "./picker.ts";

/** The row on the page drawn, and its sections; none with `?chrome=off`. */
let drawn: { readonly bar: HTMLElement; readonly sections: readonly HTMLElement[]; set(index: number): void } | null = null;

/** A jump under way: the scroll lights nothing until it ends, so the chip pressed stays on. */
let jumping = false;

/** The pinned row's foot in the viewport's pixels, wherever the row is now: where it sticks. */
function pinnedFoot(bar: HTMLElement): number {
  const toolbar = Number.parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--site-toolbar-height")) || 0;
  return toolbar + bar.getBoundingClientRect().height;
}

/** The section read now: the last whose top is at or above the pinned row's foot. */
function reading(): number {
  if (!drawn) return 0;
  const foot = pinnedFoot(drawn.bar);
  const scroller = document.scrollingElement!;
  if (scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 1) return drawn.sections.length - 1;
  let at = 0;
  drawn.sections.forEach((section, i) => {
    if (section.getBoundingClientRect().top <= foot + 1) at = i;
  });
  return at;
}

let frame = 0;
addEventListener(
  "scroll",
  () => {
    if (jumping || frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      drawn?.set(reading());
    });
  },
  { passive: true },
);
addEventListener("scrollend", () => {
  jumping = false;
});

/** The pinned row of chips for `sections`, each a section with its heading's words in `title`. */
export function contents(sections: readonly { readonly title: string; readonly element: HTMLElement }[]): HTMLElement {
  const elements = sections.map((section) => section.element);
  const jump = (index: number): void => {
    const target = elements[index]!;
    const by = target.getBoundingClientRect().top - pinnedFoot(bar);
    if (Math.abs(by) < 1) return;
    jumping = true;
    scrollBy({ top: by, behavior: settings().reduced ? "instant" : "smooth" });
  };
  const chips = picker(
    sections.map((section) => section.title),
    0,
    jump,
    "The page's sections",
  );
  const bar = el("nav", "page-contents", chips.element);
  drawn = { bar, sections: elements, set: chips.set };
  return bar;
}

/** Forgets the row of the page drawn before, when a page is drawn without one. */
export function forgetContents(): void {
  drawn = null;
}
