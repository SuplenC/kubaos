// Draws a page: its title and summary, then every part in the mockup skill's order, or a
// foundation's parts in theirs. A part that doesn't apply shows why, in its place.

import { type AnyPage, type Foundation, isFoundation, type Page } from "./page.ts";
import { el, label, prose } from "./dom.ts";
import { content } from "./parts/content.ts";
import { description } from "./parts/description.ts";
import { checks, shownAll } from "./parts/shown.ts";
import { keyboard } from "./parts/keyboard.ts";
import { matrix } from "./parts/matrix.ts";
import { measures } from "./parts/measures.ts";
import { motion } from "./parts/motion.ts";
import { partOr, section, subheading } from "./parts/section.ts";
import { tubes } from "./parts/tubes.ts";
import { variants } from "./parts/variants.ts";

const SCALES_WORDS =
  "The toolbar shows this page in light or dark, at 1, 1.25, 1.5 and 2, where a CSS pixel is a device pixel; every length above follows, and the shots render all eight.";

export function render(page: AnyPage): HTMLElement[] {
  return isFoundation(page) ? renderFoundation(page) : renderComponent(page);
}

function header(page: AnyPage): HTMLElement {
  return el(
    "header",
    "page-header",
    el("h1", "page-title", label(page.title, "title", "on_surface")),
    prose(page.summary, "body_large", "on_surface_variant"),
  );
}

function renderFoundation(page: Foundation): HTMLElement[] {
  return [
    header(page),
    section("Description", description(page.live, page.description)),
    section("Every token", shownAll(page.tokens)),
    section("Rules, worked", shownAll(page.rules)),
    section("Light and dark", partOr(page.modes, shownAll)),
    section("Across the themes", partOr(page.themes, shownAll)),
    section("At every scale, in device pixels", partOr(page.scales, shownAll)),
    section("Content", partOr(page.content, shownAll)),
    section("Motion", partOr(page.motion, motion)),
    section("What holds it", checks(page.checks)),
    section("Light and dark, at every scale", prose(SCALES_WORDS)),
  ];
}

function renderComponent(page: Page): HTMLElement[] {
  return [
    header(page),
    section("Description", description(page.live, page.description)),
    section(
      "Variants and sizes",
      subheading("Variants"),
      partOr(page.variants, variants),
      subheading("Sizes"),
      partOr(page.sizes, variants),
    ),
    section("States", matrix(page.states, page.focus), subheading("The lit line"), partOr(page.focus, tubes)),
    section("Content", content(page.content)),
    section("Motion", motion(page.motion)),
    section("Keyboard", partOr(page.keyboard, keyboard)),
    section("Measures and reflow", measures(page.measures)),
    section(
      "Light and dark, at every scale",
      prose(SCALES_WORDS),
    ),
  ];
}
