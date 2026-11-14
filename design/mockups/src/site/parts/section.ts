// A section of a page: its heading, then what it shows; or, for a part that doesn't apply, one
// muted line in its place saying why.

import { applies, type DoesntApply } from "../page.ts";
import { el, label, prose } from "../dom.ts";

export function section(title: string, ...body: Element[]): HTMLElement {
  return el("section", "page-section", el("h2", "page-heading", label(title, "headline", "on_surface")), ...body);
}

/** `part` drawn by `draw`, or a line saying why it doesn't apply, named `title` when given. */
export function partOr<T>(part: T | DoesntApply, draw: (part: T) => Element, title = ""): Element {
  return applies(part) ? draw(part) : notApplying(part.doesntApply, title);
}

/** One muted line: `title`, when given, doesn't apply, and why. */
export function notApplying(why: string, title = ""): HTMLElement {
  return el("div", "page-na", prose(`${title ? `${title}: doesn't` : "Doesn't"} apply, ${why.replace(/\.$/, "")}.`, "body", "on_surface_variant"));
}

/** A sub-heading inside a section. */
export function subheading(title: string): HTMLElement {
  return el("h3", "page-subheading", label(title, "subheading", "on_surface"));
}
