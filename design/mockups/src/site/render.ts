// Draws a page: its title and summary, then every part in the mockup skill's order. A part that
// doesn't apply shows why, in its place.

import type { Page } from "./page.ts";
import { el, label, prose } from "./dom.ts";
import { content } from "./parts/content.ts";
import { description } from "./parts/description.ts";
import { keyboard } from "./parts/keyboard.ts";
import { matrix } from "./parts/matrix.ts";
import { measures } from "./parts/measures.ts";
import { motion } from "./parts/motion.ts";
import { partOr, section, subheading } from "./parts/section.ts";
import { tubes } from "./parts/tubes.ts";
import { variants } from "./parts/variants.ts";

export function render(page: Page): HTMLElement[] {
  return [
    el(
      "header",
      "page-header",
      el("h1", "page-title", label(page.title, "title", "on_surface")),
      prose(page.summary, "body_large", "on_surface_variant"),
    ),
    section("Description", description(page.live, page.description)),
    section(
      "Variants and sizes",
      subheading("Variants"),
      partOr(page.variants, variants),
      subheading("Sizes"),
      partOr(page.sizes, variants),
    ),
    section("States", matrix(page.states), subheading("The focus tube"), partOr(page.focus, tubes)),
    section("Content", content(page.content)),
    section("Motion", motion(page.motion)),
    section("Keyboard", partOr(page.keyboard, keyboard)),
    section("Measures and reflow", measures(page.measures)),
    section(
      "Light and dark, at every scale",
      prose(
        "The toolbar shows this page in light or dark, at 1, 1.25, 1.5 and 2, where a CSS pixel is a device pixel; every length above follows, and the shots render all eight.",
      ),
    ),
  ];
}
