// The navigation: every level that has pages, in order, under its heading, the page shown lit. The
// top bar's search filters it: a page whose title holds every word typed stays, and a level with
// none left hides.

import { pagesOf } from "../pages/index.ts";
import { type AnyPage, LEVELS } from "./page.ts";
import { el, label } from "./dom.ts";

export interface Nav {
  readonly element: HTMLElement;
  /** Shows only the pages whose titles hold every word of `query`. */
  filter(query: string): void;
  /** The address of the first page shown, if any. */
  first(): string | null;
}

export function nav(current: AnyPage | undefined): Nav {
  const element = el("nav", "site-nav");
  const links: { link: HTMLAnchorElement; title: string; group: HTMLElement }[] = [];
  for (const level of LEVELS) {
    const pages = pagesOf(level.slug);
    if (pages.length === 0) continue;
    const group = el("div", "site-nav-level", label(level.title, "label", "on_surface_variant"));
    for (const page of pages) {
      const words = label(page.title, "body");
      // The link's own colour, which its state sets.
      words.style.color = "";
      const link = el("a", "site-nav-link", words);
      link.href = `#/${page.level}/${page.slug}`;
      if (page === current) link.setAttribute("aria-current", "page");
      group.append(link);
      links.push({ link, title: page.title.toLowerCase(), group });
    }
    element.append(group);
  }
  return {
    element,
    filter(query) {
      const words = query.toLowerCase().split(/\s+/).filter(Boolean);
      for (const each of links) each.link.hidden = !words.every((word) => each.title.includes(word));
      for (const group of new Set(links.map((each) => each.group))) group.hidden = [...group.querySelectorAll("a")].every((link) => link.hidden);
    },
    first() {
      return links.find((each) => !each.link.hidden)?.link.getAttribute("href") ?? null;
    },
  };
}
