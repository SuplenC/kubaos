// The right rail: "On this page", a link per section, sticky beside the page. The section read now
// is lit (the last whose heading has scrolled up to the top bar's foot), and a link jumps to its
// section.

import { length } from "../lib/tokens.ts";
import { el, label } from "./dom.ts";
import { settings } from "./settings.ts";

/** A section of the page: its heading's words and its box. */
export interface Section {
  readonly title: string;
  readonly element: HTMLElement;
}

/** The rail drawn now and its sections; none with `?chrome=off`. */
let drawn: { readonly sections: readonly HTMLElement[]; readonly links: readonly HTMLElement[] } | null = null;

/** A jump under way: the scroll lights nothing until it ends, so the link followed stays lit. */
let jumping = false;

/**
 * Where a section's top lands when its link is followed, and from where it counts as read: a
 * `space.s24` under the top bar's foot, in the viewport's pixels.
 */
function foot(): number {
  const bar = Number.parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--site-toolbar-height")) || 0;
  return bar + length("space-s24");
}

/** The section read now: the last whose top is at or above the top bar's foot, or the last one. */
function reading(): number {
  if (!drawn) return 0;
  const scroller = document.scrollingElement!;
  if (scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 1) return drawn.sections.length - 1;
  let at = 0;
  const line = foot() + 1;
  drawn.sections.forEach((section, i) => {
    if (section.getBoundingClientRect().top <= line) at = i;
  });
  return at;
}

function light(index: number): void {
  drawn?.links.forEach((link, i) => {
    if (i === index) link.setAttribute("aria-current", "location");
    else link.removeAttribute("aria-current");
  });
}

let frame = 0;
addEventListener(
  "scroll",
  () => {
    if (jumping || frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      light(reading());
    });
  },
  { passive: true },
);
addEventListener("scrollend", () => {
  jumping = false;
});

/** The rail for `sections`, the first lit. */
export function rail(sections: readonly Section[]): HTMLElement {
  const elements = sections.map((section) => section.element);
  const links = sections.map((section, index) => {
    const words = label(section.title, "body");
    // The link's own colour, which its state sets.
    words.style.color = "";
    const link = el("a", "site-rail-link", words);
    link.href = `#${location.hash.slice(1)}`;
    link.addEventListener("click", (event) => {
      event.preventDefault();
      const by = elements[index]!.getBoundingClientRect().top - foot();
      light(index);
      if (Math.abs(by) < 1) return;
      jumping = true;
      scrollBy({ top: by, behavior: settings().reduced ? "instant" : "smooth" });
    });
    return link;
  });
  drawn = { sections: elements, links };
  light(reading());
  return el("nav", "site-rail", label("On this page", "label", "on_surface_variant"), ...links);
}

/** Forgets the rail of the page drawn before, when a page is drawn without one. */
export function forgetRail(): void {
  drawn = null;
}
