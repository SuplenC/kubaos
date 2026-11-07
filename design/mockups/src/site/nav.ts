// The navigation: SidebarPage's sidebar (`lib/sidebar-page.ts`) holding the site's pages. Its
// search field on top, a hairline, then every level that has pages, in order, its heading over a
// section's row per page, the page shown current as SidebarPage's current section is. The rows
// scroll on their own under the field. What is typed filters them: a page whose title holds every
// word typed stays, and a level with none left hides. `/` reaches the field unless a sample with
// the keyboard takes that key, and Enter opens the first page left.
//
// It is drawn once and kept as pages change, so what is typed, the keyboard in the field and where
// the rows are scrolled stay.

import { field } from "../lib/field.ts";
import { keycap } from "../lib/keycap.ts";
import { divider } from "../lib/layout.ts";
import { scroll } from "../lib/scroll.ts";
import { sidebarLink } from "../lib/sidebar-page.ts";
import { length } from "../lib/tokens.ts";
import { pagesOf } from "../pages/index.ts";
import { type AnyPage, LEVELS } from "./page.ts";
import { el, label } from "./dom.ts";

export interface Nav {
  readonly element: HTMLElement;
  /** Makes `page` the current row, centred among the rows when it is out of their view. */
  select(page: AnyPage | undefined): void;
}

interface Entry {
  readonly page: AnyPage;
  readonly link: HTMLAnchorElement;
  readonly title: string;
  readonly group: HTMLElement;
}

export function nav(): Nav {
  const entries: Entry[] = [];
  const groups: HTMLElement[] = [];
  for (const level of LEVELS) {
    const pages = pagesOf(level.slug);
    if (pages.length === 0) continue;
    const group = el("div", "site-nav-level", label(level.title, "label", "on_surface_variant"));
    for (const page of pages) {
      const link = sidebarLink(page.title, null, false, `#/${page.level}/${page.slug}`);
      group.append(link);
      entries.push({ page, link, title: page.title.toLowerCase(), group });
    }
    groups.push(group);
  }
  const rows = el("div", "site-nav-rows", ...groups);
  const rowsScroll = scroll(rows, { height: "100%", above: 0, below: 0, noGutter: true });
  const view = rowsScroll.view;

  function filter(query: string): void {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    for (const each of entries) each.link.hidden = !words.every((word) => each.title.includes(word));
    for (const group of groups) group.hidden = entries.every((each) => each.group !== group || each.link.hidden);
  }

  const search = field({
    placeholder: "Search",
    icon: "search",
    onInput: (query) => {
      search.toggleAttribute("data-filled", query !== "");
      filter(query);
    },
  });
  search.classList.add("site-search");
  search.append(keycap("/"));
  search.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") return;
    const first = entries.find((each) => !each.link.hidden);
    if (first) location.hash = first.link.getAttribute("href")!;
  });
  addEventListener("keydown", (event) => {
    const typing = event.target instanceof HTMLElement && (event.target.isContentEditable || event.target.matches("input, textarea, [role=textbox]"));
    // A sample that handles the key has prevented its default: the key is the sample's.
    if (event.key !== "/" || typing || event.defaultPrevented) return;
    event.preventDefault();
    search.focus();
  });

  const head = el("div", "site-nav-head", search, el("div", "site-nav-rule", divider()));
  const element = el("nav", "site-nav", head, rowsScroll.box);

  return {
    element,
    select(page) {
      let shown: HTMLElement | undefined;
      for (const each of entries) {
        const current = each.page === page;
        each.link.toggleAttribute("data-current", current);
        if (current) {
          each.link.setAttribute("aria-current", "page");
          shown = each.link;
        } else each.link.removeAttribute("aria-current");
      }
      if (!shown || shown.hidden) return;
      const row = shown;
      // Once laid out: on load the navigation is put on the document after it is drawn.
      requestAnimationFrame(() => {
        const box = view.getBoundingClientRect();
        const top = row.getBoundingClientRect().top - box.top + view.scrollTop;
        const room = length("space-s12");
        const shown = top - room >= view.scrollTop && top + row.offsetHeight + room <= view.scrollTop + view.clientHeight;
        if (!shown) view.scrollTop = Math.round(top - (view.clientHeight - row.offsetHeight) / 2);
      });
    },
  };
}
