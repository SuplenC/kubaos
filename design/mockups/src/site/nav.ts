// The navigation: every level in order, its pages under it, and a line for a level with none yet.

import { pagesOf } from "../pages/index.ts";
import { type AnyPage, LEVELS } from "./page.ts";
import { el } from "./dom.ts";

export function nav(current: AnyPage | undefined): HTMLElement {
  const list = el("nav", "site-nav");
  for (const level of LEVELS) {
    const pages = pagesOf(level.slug);
    const group = el("div", "site-nav-level", el("div", "site-nav-title", level.title));
    if (pages.length === 0) group.append(el("div", "site-nav-empty", "None yet"));
    for (const page of pages) {
      const link = el("a", "site-nav-link", page.title);
      link.href = `#/${page.level}/${page.slug}`;
      if (page === current) link.setAttribute("aria-current", "page");
      group.append(link);
    }
    list.append(group);
  }
  return list;
}
