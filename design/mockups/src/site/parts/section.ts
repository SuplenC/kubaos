// A section of a page: its heading, then what it shows, or why it doesn't apply.

import { applies, type DoesntApply } from "../page.ts";
import { el, label, prose } from "../dom.ts";

export function section(title: string, ...body: Element[]): HTMLElement {
  return el("section", "page-section", el("h2", "page-heading", label(title, "heading", "on_surface")), ...body);
}

/** `part` drawn by `draw`, or a line saying why it doesn't apply. */
export function partOr<T>(part: T | DoesntApply, draw: (part: T) => Element): Element {
  return applies(part) ? draw(part) : notApplying(part.doesntApply);
}

export function notApplying(why: string): HTMLElement {
  return el("div", "page-na", prose(`Doesn't apply: ${why}`, "body", "on_surface_variant"));
}

/** A sub-heading inside a section. */
export function subheading(title: string): HTMLElement {
  return el("h3", "page-subheading", label(title, "subheading", "on_surface"));
}
