// A foundation's parts: each shown thing under its name, its words beside it, the thing itself
// below at the page's full width; and what holds the family to its rules.

import type { Check, NonEmpty, Shown } from "../page.ts";
import { el, label, prose } from "../dom.ts";
import { subheading } from "./section.ts";

export function shownAll(parts: NonEmpty<Shown>): HTMLElement {
  return el(
    "div",
    "page-part",
    ...parts.flatMap((part) => [subheading(part.name), prose(part.described), el("div", "page-shown", part.sample())]),
  );
}

export function checks(list: NonEmpty<Check>): HTMLElement {
  return el(
    "dl",
    "page-list",
    ...list.flatMap((check) => [el("dt", "", label(check.check, "body_strong", "on_surface")), el("dd", "", prose(check.holds))]),
  );
}
