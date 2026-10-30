// What the foundations pages draw their tokens with: a box painted in a theme's palette in a mode,
// a colour role's swatch, a table of the page's own words and samples, and a token's name with its
// value.

import { paint, palette } from "../recolour.ts";
import type { Mode } from "../settings.ts";
import { THEMES } from "../themes.ts";
import { el, label } from "../dom.ts";

/**
 * `children` on a surface painted in `theme`'s palette in `mode`, whatever the toolbar picks: its
 * derived roles computed from its own tones, its lights lit at night and unlit glass by day.
 */
export function painted(theme: string, mode: Mode, ...children: Element[]): HTMLElement {
  const box = el("div", "f-painted", ...children);
  box.dataset.mode = mode;
  paint(box, palette(theme, mode));
  return box;
}

/** The colour role `role` as a square, the soft outline round it so a pale one shows on a pale page. */
export function swatch(role: string): HTMLElement {
  const box = el("div", "f-swatch");
  box.style.setProperty("--swatch", `var(--color-${role})`);
  return box;
}

/** `role`'s swatch over its value as written: its hex in a palette, or how it is worked out. */
export function swatchWith(role: string, value: string): HTMLElement {
  return el("div", "f-swatch-cell", swatch(role), label(value, "mono", "on_surface_variant"));
}

/** A token's name as code, with its value after it. */
export function token(name: string, value?: string): HTMLElement {
  return el("code", "page-code", value === undefined ? name : `${name} = ${value}`);
}

/**
 * A table: its headings, then its rows, each cell the page's words or a sample. Wider than the
 * page at a large scale, it scrolls on its own rather than the page.
 */
export function table(heads: readonly string[], rows: readonly (readonly (Element | string)[])[]): HTMLElement {
  const head = el("tr", "", ...heads.map((words) => el("th", "", label(words, "label"))));
  const body = rows.map((cells) =>
    el("tr", "", ...cells.map((cell) => el("td", "", typeof cell === "string" ? label(cell, "body", "on_surface") : cell))),
  );
  return el("div", "f-scroll", el("table", "page-table", el("thead", "", head), el("tbody", "", ...body)));
}

/** Every theme the toolbar paints in, by slug, with its name. */
export function themes(): { slug: string; name: string }[] {
  return THEMES.map((theme) => ({ slug: theme.slug, name: theme.name }));
}

export const MODES: readonly Mode[] = ["dark", "light"];

/** A mode's name, as a heading says it. */
export function modeName(mode: Mode): string {
  return mode === "dark" ? "Dark" : "Light";
}

/** `children` drawn at `scale`, whatever the page's: a box of its own whose lengths follow it. */
export function atScale(scale: number, ...children: Element[]): HTMLElement {
  const box = el("div", "mock f-at-scale", ...children);
  box.style.setProperty("--scale", String(scale));
  return el("div", "f-swatch-cell", label(`${scale}×`, "label"), box);
}
