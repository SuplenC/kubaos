// Draws a page: its title and summary, then every part in the mockup skill's order, or a
// foundation's parts in theirs, each a section the page's contents jump to. A part that doesn't
// apply shows why, in its place.

import { type AnyPage, applies, type DoesntApply, type Foundation, isFoundation, type Page } from "./page.ts";
import { el, label, prose } from "./dom.ts";
import { content } from "./parts/content.ts";
import { description } from "./parts/description.ts";
import { checks, shownAll } from "./parts/shown.ts";
import { keyboard } from "./parts/keyboard.ts";
import { matrix } from "./parts/matrix.ts";
import { measures } from "./parts/measures.ts";
import { motion } from "./parts/motion.ts";
import { contents, forgetContents } from "./parts/contents.ts";
import { notApplying, partOr, section, subheading } from "./parts/section.ts";
import { shelves } from "./parts/shelf.ts";
import { tubes } from "./parts/tubes.ts";
import { variants } from "./parts/variants.ts";
import { CHROME } from "./settings.ts";

const SCALES_WORDS =
  "The toolbar shows this page in light or dark, at 1, 1.25, 1.5 and 2, where a CSS pixel is a device pixel; every length above follows, and the shots render all eight.";

/** The page's sections, each its heading's words and its box. */
type Sections = { readonly title: string; readonly element: HTMLElement }[];

/**
 * The page in its container: its header, the pinned row of its sections' chips when the site's
 * chrome shows, then its sections.
 */
export function render(page: AnyPage): HTMLElement[] {
  const sections = isFoundation(page) ? renderFoundation(page) : renderComponent(page);
  const parts = sections.map((each) => each.element);
  if (!CHROME) forgetContents();
  const container = el("div", "page-container", header(page), ...(CHROME ? [contents(sections)] : []), ...parts);
  return [container];
}

function header(page: AnyPage): HTMLElement {
  return el(
    "header",
    "page-header",
    el("h1", "page-title", label(page.title, "title", "on_surface")),
    prose(page.summary, "body_large", "on_surface_variant"),
  );
}

/** A section under `title`, kept with its title for the page's contents. */
function part(title: string, ...body: Element[]): Sections[number] {
  return { title, element: section(title, ...body) };
}

function renderFoundation(page: Foundation): Sections {
  const shown = (parts: Parameters<typeof shownAll>[0]) => shelves(shownAll(parts));
  const or = <T>(value: T | DoesntApply, draw: (value: T) => Element[]): Element[] => (applies(value) ? draw(value) : [notApplying(value.doesntApply)]);
  return [
    part("Description", description(page.live, page.description)),
    part("Every token", ...shown(page.tokens)),
    part("Rules, worked", ...shown(page.rules)),
    part("Light and dark", ...or(page.modes, shown)),
    part("Across the themes", ...or(page.themes, shown)),
    part("At every scale, in device pixels", ...or(page.scales, shown)),
    part("Content", ...or(page.content, shown)),
    part("Motion", partOr(page.motion, motion)),
    part("What holds it", checks(page.checks)),
    part("Light and dark, at every scale", prose(SCALES_WORDS)),
  ];
}

function renderComponent(page: Page): Sections {
  return [
    part("Description", description(page.live, page.description)),
    part("Variants and sizes", ...shelves(variants("Variants", page.variants), variants("Sizes", page.sizes))),
    part("States", ...shelves(...matrix(page.states, page.focus)), subheading("The lit line"), partOr(page.focus, tubes)),
    part("Content", ...shelves(...content(page.content))),
    part("Motion", motion(page.motion)),
    part("Keyboard", partOr(page.keyboard, keyboard)),
    part("Measures and reflow", ...shelves(...measures(page.measures))),
    part("Light and dark, at every scale", prose(SCALES_WORDS)),
  ];
}
