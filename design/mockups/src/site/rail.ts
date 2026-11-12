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

// Where each section's top stands against the foot's line, from an observer of the room above the
// line, which hears every top that crosses it and the boxes as laid out for the frame: scrolling
// reads no layout of its own.
const above = new Map<Element, boolean>();
let lineObserver: IntersectionObserver | null = null;
/** The document's height, as laid out last: read when it changes, not as the page scrolls. */
let documentHeight = 0;
new ResizeObserver(() => {
  documentHeight = document.scrollingElement!.scrollHeight;
  if (!jumping) light(reading());
}).observe(document.documentElement);

/** Watches the sections drawn against the foot's line where it stands now: the bar's height moved it. */
export function watchLine(): void {
  lineObserver?.disconnect();
  above.clear();
  if (!drawn) return;
  const line = foot() + 1;
  lineObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) above.set(entry.target, entry.boundingClientRect.top <= line);
      if (!jumping) light(reading());
    },
    { rootMargin: `0px 0px ${-Math.max(0, innerHeight - line)}px 0px` },
  );
  for (const section of drawn.sections) lineObserver.observe(section);
}
addEventListener("resize", watchLine);

/** The section read now: the last whose top is at or above the top bar's foot, or the last one. */
function reading(): number {
  if (!drawn) return 0;
  if (scrollY + innerHeight >= documentHeight - 1) return drawn.sections.length - 1;
  let at = 0;
  drawn.sections.forEach((section, i) => {
    if (above.get(section)) at = i;
  });
  return at;
}

let lit = -1;
function light(index: number): void {
  if (index === lit) return;
  lit = index;
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
  lit = -1;
  watchLine();
  light(reading());
  return el("nav", "site-rail", label("On this page", "label", "on_surface_variant"), ...links);
}

/** Forgets the rail of the page drawn before, when a page is drawn without one. */
export function forgetRail(): void {
  drawn = null;
  watchLine();
}
