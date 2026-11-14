// The navigation: SidebarPage's sidebar (`lib/sidebar-page.ts`) holding the site's pages. Its
// search field on top, a hairline, then every level that has pages, in order: a section's row with
// its symbol and a chevron, which opens and shuts its pages under it as a Collapsible does, each a
// sub-section's row. The current page shows as SidebarPage's current sub-section does, its words
// lit and the bar of neon beside it, and its level as the current section, filled. The rows scroll
// on their own under the field.
//
// A level opens with its page, and stays as the viewer left it: the levels opened are kept in the
// browser. What is typed filters the rows, a page whose title holds every word typed staying, and
// opens every level with a match while it is there; a level with none hides. `/` reaches the field
// unless a sample with the keyboard takes that key, and Enter opens the first page left.
//
// It is drawn once and kept as pages change, so what is typed, the keyboard in the field and where
// the rows are scrolled stay.

import { field } from "../lib/field.ts";
import { keycap } from "../lib/keycap.ts";
import { divider } from "../lib/layout.ts";
import { scroll } from "../lib/scroll.ts";
import { putBar, sidebarFold, sidebarLink, subBar } from "../lib/sidebar-page.ts";
import { length } from "../lib/tokens.ts";
import { pagesOf } from "../pages/index.ts";
import { type AnyPage, type Level, LEVELS } from "./page.ts";
import { el } from "./dom.ts";
import { onChange, settings } from "./settings.ts";

export interface Nav {
  readonly element: HTMLElement;
  /** Makes `page` the current row, its level open, centred among the rows when out of their view. */
  select(page: AnyPage | undefined): void;
}

/** Each level's symbol: what the level holds, drawn from KubaOS Symbols. */
const SYMBOLS: Record<Level, string> = {
  foundations: "draw",
  layout: "list_alt",
  atoms: "circle",
  molecules: "deployed_code",
  templates: "draft",
  shell: "desktop_windows",
  kobo: "rocket_launch",
  apps: "computer",
};

/** Where the levels the viewer opened are kept. */
const STORED = "kubaos-mockups-nav-open";

function storedOpen(): Set<string> | null {
  try {
    const raw = localStorage.getItem(STORED);
    if (raw === null) return null;
    const list: unknown = JSON.parse(raw);
    return Array.isArray(list) ? new Set(list.filter((each): each is string => typeof each === "string")) : null;
  } catch {
    return null;
  }
}

function storeOpen(open: ReadonlySet<string>): void {
  try {
    localStorage.setItem(STORED, JSON.stringify([...open]));
  } catch {
    // Storage off or full: the levels open now still hold for this visit.
  }
}

interface Group {
  readonly slug: Level;
  readonly element: HTMLElement;
  readonly fold: HTMLButtonElement;
  readonly body: HTMLElement;
}

interface Entry {
  readonly page: AnyPage;
  readonly link: HTMLAnchorElement;
  readonly title: string;
  readonly group: Group;
}

export function nav(): Nav {
  const entries: Entry[] = [];
  const groups: Group[] = [];
  /** The levels the viewer has open, as kept: a search opens more while it is there. */
  const open = storedOpen() ?? new Set<string>();
  let query: string[] = [];

  for (const level of LEVELS) {
    const pages = pagesOf(level.slug);
    if (pages.length === 0) continue;
    const fold = sidebarFold(level.title, SYMBOLS[level.slug]);
    const subs = el("div", "k-sp-subs");
    const content = el("div", "k-collapsible-content", subs);
    const body = el("div", "k-collapsible-body", content);
    body.setAttribute("role", "region");
    body.id = `site-nav-${level.slug}`;
    fold.setAttribute("aria-controls", body.id);
    const element = el("div", "site-nav-level k-collapsible", fold, body);
    const group: Group = { slug: level.slug, element, fold, body };
    // The room eases between nothing and its rows, which only layout knows.
    new ResizeObserver(() => element.style.setProperty("--content-height", `${content.offsetHeight}px`)).observe(content);
    element.addEventListener("transitionend", (event) => {
      if (event.target === element && event.propertyName === "--collapsible-open") element.toggleAttribute("data-settled", element.hasAttribute("data-open"));
    });
    fold.addEventListener("click", () => {
      if (open.has(level.slug)) open.delete(level.slug);
      else open.add(level.slug);
      storeOpen(open);
      show();
    });
    for (const page of pages) {
      const link = sidebarLink("sub", page.title, null, false, `#/${page.level}/${page.slug}`);
      subs.append(link);
      entries.push({ page, link, title: page.title.toLowerCase(), group });
    }
    groups.push(group);
  }
  const rows = el("div", "site-nav-rows", ...groups.map((group) => group.element));
  const rowsScroll = scroll(rows, { height: "100%", above: 0, below: 0, noGutter: true });
  const view = rowsScroll.view;
  const bar = subBar();
  let current: Entry | undefined;

  /** Each level open or shut, shown or hidden by the search, and the bar beside the current page. */
  function show(): void {
    for (const each of entries) each.link.hidden = !query.every((word) => each.title.includes(word));
    for (const group of groups) {
      const matched = entries.some((each) => each.group === group && !each.link.hidden);
      group.element.hidden = !matched;
      const opened = query.length > 0 ? matched : open.has(group.slug);
      if (opened === group.element.hasAttribute("data-open")) continue;
      group.element.removeAttribute("data-settled");
      group.element.toggleAttribute("data-open", opened);
      group.fold.setAttribute("aria-expanded", String(opened));
      group.body.inert = !opened;
      // With nothing to play (reduced motion, or drawn at once) the room is where it ends at once.
      if (getComputedStyle(group.element).transitionDuration.split(",").every((each) => Number.parseFloat(each) === 0)) {
        group.element.toggleAttribute("data-settled", opened);
      }
    }
    for (const group of groups) group.fold.toggleAttribute("data-current", current?.group === group);
    bar.hidden = !current || current.link.hidden;
    if (!current || current.link.hidden) return;
    current.link.parentElement!.append(bar);
    putBar(bar, current.link);
  }

  const search = field({
    placeholder: "Search",
    icon: "search",
    onInput: (typed) => {
      search.toggleAttribute("data-filled", typed !== "");
      query = typed.toLowerCase().split(/\s+/).filter(Boolean);
      show();
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
  // The levels as they open: Collapsible's motion, or at once with reduced motion.
  const reduce = () => element.toggleAttribute("data-reduced", settings().reduced);
  reduce();
  onChange(reduce);

  let drawn = false;

  return {
    element,
    select(page) {
      current = entries.find((each) => each.page === page);
      for (const each of entries) {
        const picked = each === current;
        each.link.toggleAttribute("data-current", picked);
        if (picked) each.link.setAttribute("aria-current", "page");
        else each.link.removeAttribute("aria-current");
      }
      // The current page's level opens with it; the first drawing shows every level at rest.
      if (current) open.add(current.group.slug);
      if (!drawn) element.dataset.sequence = "";
      show();
      if (!drawn) {
        drawn = true;
        requestAnimationFrame(() => delete element.dataset.sequence);
      }
      if (!current || current.link.hidden) return;
      const row = current.link;
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
