// An app's window as its sections in a sidebar beside a page that scrolls: the template
// `SidebarPage` (`docs/design/components/sidebar-page.md`), Settings' layout. Its look is
// `sidebar-page.css`.
//
// The sidebar is a band `size.sidebar` wide in `surface_container_low`, its rows `space.s12` in
// and `space.s4` apart: Home, a hairline `space.s8` clear of the rows round it, then the sections.
// Only the current section opens its sub-sections under it, and a short bar of neon beside the
// current one says where the page is, gliding as the page scrolls. The page is the section's
// header, then its sub-sections, each a heading over wells of rows, in a `Page` that scrolls; the
// sidebar follows the scroll. Home is a page that is only search, across every setting.
//
// The template owns where the person is: the section, the sub-section the page is scrolled to,
// where the keyboard is, the find box and its matches, and the motion between pages. What the app
// gives it is its outline, the rows each well holds and, for a section with a region of its own
// (CanvasPage), what draws that region.

import "./sidebar-page.css";
import { type Pinned, pin } from "./control.ts";
import { field, setValue } from "./field.ts";
import { figure, tick } from "./figure.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { keyHints } from "./keycap.ts";
import { containerPadding, divider, filling, header, page, row, section, stack } from "./layout.ts";
import { type ListRowSpec, listRow, scaleOf, well } from "./list.ts";
import { FIXED_HEIGHT, probe } from "./probe.ts";
import { scroll } from "./scroll.ts";
import { clamped, type Run, text, type TypeStyle } from "./text.ts";
import { cubic, length, raw } from "./tokens.ts";
import { popover } from "./tooltip.ts";
import { settings } from "../site/settings.ts";

/** A sub-section of a section's page: its heading, a line under it, and its wells of rows. */
export interface SubSection {
  readonly title: string;
  readonly body?: string;
  /** A tip about the whole sub-section, keys as `[Key]`: Home's search finds the sub-section by it. */
  readonly tip?: string;
  readonly wells: readonly (readonly ListRowSpec[])[];
}

/** What the template offers a region drawn in its page. */
export interface RegionHost {
  /** The sub-section `index`'s name in the sidebar, which follows what the region shows. */
  readonly rename: (index: number, title: string) => void;
  /** The box the region's sub-sections scroll in, and their lead line in its own pixels, or the page again. */
  readonly spyOn: (view: HTMLElement | null, lead?: number) => void;
  /** Takes the keyboard to the region's sub-section `index`, at its first row. */
  readonly enter: (index: number) => void;
  /** Reads the region's rows again, after it drew a sub-section anew: the selection changed. */
  readonly refresh: () => void;
  /**
   * Turns the page sideways to a page of the region's own: `deeper` into one (it comes in from the
   * end, at its top), or back out (from the start, scrolled where it was). `swap` draws it once the
   * page left has gone; `arrived` runs once it is in. Going deeper from sub-section `from`, the
   * sidebar's bar stays beside it while the page is open.
   */
  readonly turn: (deeper: boolean, swap: () => void, arrived?: () => void, from?: number) => void;
}

/** A section: its row in the sidebar, its page's header and its sub-sections. */
export interface Section {
  readonly icon: string;
  readonly title: string;
  readonly accent: string;
  readonly lede: string;
  readonly subs: readonly SubSection[];
  /** A small button at the end of the page's title, as the page is drawn: Settings' hold to reset the section. */
  readonly trailing?: () => HTMLElement;
  /** A region of its own in place of the sub-sections' wells, below the header: CanvasPage. */
  readonly region?: (host: RegionHost) => HTMLElement;
}

/** The window's outline: Home's words, then its sections. */
export interface Outline {
  readonly home: { readonly name: string; readonly accent: string; readonly title: string; readonly lede: string; readonly placeholder: string };
  readonly sections: readonly Section[];
}

/** Where the keyboard is, for a sample pinned in a state. */
export type Keyboard = "home" | "sidebar" | "page" | "find";

export interface SidebarPageOptions {
  readonly outline: Outline;
  /** The section it shows, by index; Home when left out, as the window opens. */
  readonly section?: number;
  /** The sub-section the page is scrolled to. */
  readonly sub?: number;
  /**
   * The sub-section a page of the region's own, open as it is drawn, was opened from: the bar is
   * held beside it, as a turn deeper holds it.
   */
  readonly heldSub?: number;
  /** What is typed in Home's field. */
  readonly query?: string;
  /** The find box open on the section's page with this typed, on its match `findAt`. */
  readonly find?: string;
  readonly findAt?: number;
  /** The keyboard there, lit as it shows the keyboard: a still sample. */
  readonly keyboard?: Keyboard;
  /** The page's row the keyboard is on, by its index among the page's rows. */
  readonly row?: number;
  /** The sidebar's row the keyboard is on, by its index among the sidebar's rows. */
  readonly sidebarRow?: number;
  /**
   * The section the sidebar glides from to `section`: drawn at the glide's end, with its start set,
   * for a move's player to play.
   */
  readonly glideFrom?: number;
  /** The page's row, by its index, Home's search just landed on: its highlight fading. */
  readonly landed?: number;
  /** The keyboard put in Home's field as the template is drawn: the live window opening. */
  readonly focus?: boolean;
}

/** One of the sidebar's rows: Home, a section, or a sub-section of the current one. */
type Item = { readonly kind: "home" } | { readonly kind: "section"; readonly index: number } | { readonly kind: "sub"; readonly index: number };

/** A row of the page, as the app describes it, and where it is. */
interface PageRow {
  readonly spec: ListRowSpec;
  readonly sub: number;
  element: HTMLElement;
}

/** A setting found from Home, and where it is. */
interface Found {
  readonly spec: ListRowSpec;
  readonly section: number;
  readonly sub: number;
  readonly row: number;
  readonly runs: readonly Run[];
  /** Found by its sub-section's tip: the result is the sub-section, landing on its first row. */
  readonly heading?: boolean;
}

/**
 * Where `query` matches `words`, case and accents aside: a run of it whole, or else each of its
 * letters in order. It stands in for Kotodama's engine, which Settings' search runs, so the
 * mockup lights the letters as the engine's results do; null where it doesn't match.
 */
export function matchRuns(words: string, query: string): Run[] | null {
  const fold = (s: string) => [...s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase()];
  const hay = fold(words);
  const needle = fold(query.trim());
  if (needle.length === 0) return null;
  const joined = hay.join("");
  const at = joined.indexOf(needle.join(""));
  if (at >= 0) return [{ start: at, end: at + needle.length }];
  const runs: Run[] = [];
  let from = 0;
  for (const letter of needle) {
    if (letter === " ") continue;
    const found = hay.indexOf(letter, from);
    if (found < 0) return null;
    const last = runs.at(-1);
    if (last && last.end === found) runs[runs.length - 1] = { start: last.start, end: found + 1 };
    else runs.push({ start: found, end: found + 1 });
    from = found + 1;
  }
  return runs;
}

/**
 * Where `query` finds the row `spec`: by its name, its letters lit, as `matchRuns` does; else as a
 * whole run in its line or its tip, each `[Key]` read as the key's name ("super tab"), with nothing
 * in its name to light; null where it doesn't.
 */
export function findRow(spec: ListRowSpec, query: string): Run[] | null {
  const named = matchRuns(spec.title, query);
  if (named) return named;
  const fold = (s: string) => s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase().replace(/\s+/g, " ");
  const needle = fold(query.trim());
  const said = [spec.subtitle, spec.tip?.replace(/\[([^\]]+)\]/g, "$1")].filter((each): each is string => each !== undefined);
  return said.some((each) => fold(each).includes(needle)) ? [] : null;
}

/** Which way a page turn goes: along the sidebar (`y`) or deeper (`x`), the new page coming from the `sign` side. */
interface Way {
  readonly axis: "x" | "y";
  readonly sign: 1 | -1;
}

/** A page turn asked for: the page to draw once the old one has gone, and what follows its arrival. */
interface Turn {
  readonly way: Way;
  readonly swap: () => void;
  readonly arrived: (() => void) | undefined;
}

/** A duration token in milliseconds under the speed setting, whatever reduced motion says. */
function played(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** The easing token `name` as a CSS timing function. */
function easing(name: string): string {
  return `cubic-bezier(${cubic(`easing-${name}`).join(", ")})`;
}

/** Animates a page's `--page-shift` and `--page-shown` from `from` to `to`, held at its end. */
function moving(element: HTMLElement, from: readonly [number, number], to: readonly [number, number], duration: number, curve: string): Animation {
  const frame = ([shift, shown]: readonly [number, number]) => ({ "--page-shift": String(shift), "--page-shown": String(shown) });
  return element.animate([frame(from), frame(to)], { duration, easing: easing(curve), fill: "both" });
}

/** The duration token `name` in milliseconds, as the page plays it: none with reduced motion. */
function ms(name: string): number {
  return settings().reduced ? 0 : Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** The easing token `name` as a function of time, 0 to 1, solved from its cubic Bézier. */
function eased(name: string): (t: number) => number {
  const [x1, y1, x2, y2] = cubic(`easing-${name}`) as [number, number, number, number];
  const at = (a: number, b: number, t: number) => 3 * a * t * (1 - t) ** 2 + 3 * b * t * t * (1 - t) + t ** 3;
  return (t) => {
    let lo = 0;
    let hi = 1;
    for (let i = 0; i < 24; i++) {
      const mid = (lo + hi) / 2;
      if (at(x1, x2, mid) < t) lo = mid;
      else hi = mid;
    }
    return at(y1, y2, (lo + hi) / 2);
  };
}

/** `element`'s top in `view`'s scrolled content, in its own pixels: the page's box is zoomed. */
function topIn(element: Element, view: HTMLElement): number {
  const box = view.getBoundingClientRect();
  const ratio = view.offsetHeight / Math.max(1, box.height);
  return (element.getBoundingClientRect().top - box.top) * ratio + view.scrollTop;
}

/** Each scroll's latest glide: a newer one, or the wheel, takes over from the one before. */
const glides = new WeakMap<HTMLElement, number>();

/** The scrolls with a glide in flight. */
const gliding = new WeakSet<HTMLElement>();

/** Stops `view`'s glide where it is: the wheel took over. */
function stopGlide(view: HTMLElement): void {
  glides.set(view, (glides.get(view) ?? 0) + 1);
  gliding.delete(view);
}

/** Glides `view`'s scroll to `to` over `medium` on `standard`, or jumps with reduced motion. */
function glideScroll(view: HTMLElement, to: number): void {
  const target = Math.max(0, Math.min(view.scrollHeight - view.clientHeight, Math.round(to)));
  const duration = ms("medium");
  const from = view.scrollTop;
  stopGlide(view);
  const glide = glides.get(view);
  if (duration === 0 || from === target) {
    view.scrollTop = target;
    return;
  }
  const curve = eased("standard");
  const start = performance.now();
  gliding.add(view);
  const frame = (now: number) => {
    if (glides.get(view) !== glide) return;
    const t = Math.min(1, (now - start) / duration);
    view.scrollTop = Math.round(from + (target - from) * curve(t));
    if (t < 1) requestAnimationFrame(frame);
    else gliding.delete(view);
  };
  requestAnimationFrame(frame);
}

/** A row of the sidebar: a section's (Home's too) or a sub-section's, pinned in a state for a sample. */
export function sidebarRow(kind: "section" | "sub", words: string, glyph: string | null, current: boolean, pinned?: Pinned): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.tabIndex = -1;
  return dressRow(button, kind, words, glyph, current, pinned);
}

/** A row that goes to the page at `href`: a sidebar of pages, the mockup site's own. */
export function sidebarLink(kind: "section" | "sub", words: string, glyph: string | null, current: boolean, href: string): HTMLAnchorElement {
  const link = document.createElement("a");
  link.href = href;
  return dressRow(link, kind, words, glyph, current, undefined);
}

/** The bar beside the current sub-section, in the box of rows it is placed among. */
export function subBar(): HTMLElement {
  const bar = document.createElement("span");
  bar.className = "k-sp-bar";
  return bar;
}

/** Puts `bar` beside the sub-section's row `target` at once, with no glide. */
export function putBar(bar: HTMLElement, target: HTMLElement): void {
  const to = `${target.offsetTop}px`;
  bar.dataset.still = "";
  bar.style.setProperty("--bar-from", to);
  bar.style.setProperty("--bar-to", to);
  bar.style.setProperty("--bar-at", "1");
}

/**
 * A section's row that opens and shuts the rows under it, a sidebar of pages' level: its symbol, its
 * words in `body_strong`, and at its end Collapsible's chevron, which turns as a `.k-collapsible`
 * round it opens.
 */
export function sidebarFold(words: string, glyph: string | null): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  dressRow(button, "section", words, glyph, false, undefined, "body_strong");
  button.dataset.fold = "";
  const chevron = icon("expand_more", "var(--type-body-icon)", "var(--color-on_surface_variant)");
  chevron.classList.add("k-collapsible-chevron");
  button.querySelector(".k-row")!.append(chevron);
  return button;
}

/** `element` drawn as a row of the sidebar: its symbol, its words, its states and its ring. */
function dressRow<E extends HTMLElement>(
  element: E,
  kind: "section" | "sub",
  words: string,
  glyph: string | null,
  current: boolean,
  pinned: Pinned | undefined,
  style: TypeStyle = kind === "section" ? "body" : "small",
): E {
  element.className = "k-sp-row k-control";
  element.dataset.kind = kind;
  element.dataset.hot = "";
  element.toggleAttribute("data-current", current);
  if (current) element.setAttribute("aria-current", "page");
  pin(element, pinned);
  const name = clamped(words, style);
  name.style.color = "var(--words)";
  const parts: Element[] = [];
  if (glyph) parts.push(icon(glyph, "var(--type-body-icon)", "var(--glyph)"));
  parts.push(filling(name));
  element.append(filling(row("s12", "centre", ...parts)));
  focusRing(element, "control", "accent", false).dataset.pressable = "";
  return probe(element, { role: "component", name: "SidebarRow", variant: kind, fixed: FIXED_HEIGHT });
}

/**
 * The template, drawn in the room it is given: the sidebar, and the page beside it taking the
 * rest. It is live: the pointer, the wheel and the keys drive it as the window would.
 */
export function sidebarPage(options: SidebarPageOptions): HTMLElement {
  const outline = options.outline;
  const sections = outline.sections;
  const root = document.createElement("span");
  root.className = "k-sp";
  root.dataset.whole = "";

  let current = options.section ?? -1;
  let sub = options.heldSub ?? options.sub ?? 0;
  /** The sub-section a press in the sidebar went to, held while the page stays where it went. */
  let pressed: { sub: number; top: number } | null = null;
  const names = sections.map((each) => each.subs.map((s) => s.title));

  // The sidebar: its rows in a scroll of their own, the band's padding the thumb's room.
  const sideRows = stack("s4", "start");
  sideRows.classList.add("k-sp-rows");
  const sideScroll = scroll(sideRows, { height: "100%", above: 0, below: 0, noGutter: true });
  const side = document.createElement("span");
  side.className = "k-sp-side";
  side.append(sideScroll.box);
  let items: Item[] = [];
  let itemRows: HTMLButtonElement[] = [];
  let sideAt = options.sidebarRow ?? 0;
  // The sidebar's parts, made once: a row a section (Home's first), each section's list of
  // sub-sections with its bar, and the highlight that travels between the sections' rows.
  let homeRow: HTMLButtonElement | null = null;
  const sectionRows: HTMLButtonElement[] = [];
  const subLists: (HTMLElement | null)[] = [];
  const subRows: HTMLButtonElement[][] = [];
  const bars: HTMLElement[] = [];
  const highlight = document.createElement("span");
  /** The section the sidebar shows as current: what its highlight and its open list are on. */
  let shownSection: number | null = null;
  /** The highlight's glide and the lists' folds, running. */
  let glides: Animation[] = [];

  // The page: what scrolls, the rise a change of section plays on, and the find box over it.
  const rise = document.createElement("span");
  rise.className = "k-sp-rise";
  const pageScroll = scroll(rise, { height: "100%", above: 0, below: 0, noGutter: true });
  const view = pageScroll.view;
  const pane = document.createElement("span");
  pane.className = "k-sp-pane";
  pane.append(pageScroll.box);
  root.append(side, pane);

  let rows: PageRow[] = [];
  let pageAt = options.row ?? -1;
  let wells: HTMLElement[] = [];
  let homeField: HTMLElement | null = null;
  let homeAnswers: HTMLElement | null = null;
  let homeQuery = options.query ?? "";
  let results: { found: Found[]; rows: HTMLElement[]; at: number } | null = null;
  let spyView: HTMLElement = view;
  let spyLead = () => length("space-s32") * scaleOf(root);

  // The find box, and what it puts back when it closes.
  const find = {
    open: false,
    query: "",
    matches: [] as number[],
    at: 0,
    saved: null as { top: number; sub: number; row: number; focus: Element | null } | null,
    box: null as HTMLElement | null,
    field: null as HTMLElement | null,
    count: null as HTMLElement | null,
    note: null as HTMLElement | null,
  };

  function buildItems(): Item[] {
    const list: Item[] = [{ kind: "home" }];
    sections.forEach((_, index) => {
      list.push({ kind: "section", index });
      if (index === current) names[index]!.forEach((__, s) => list.push({ kind: "sub", index: s }));
    });
    return list;
  }

  /** The section's row by its index, Home's at -1. */
  function sectionRow(index: number): HTMLButtonElement {
    return index === -1 ? homeRow! : sectionRows[index]!;
  }

  /**
   * Makes the sidebar's rows once: Home, the hairline, then each section's row and the list of its
   * sub-sections under it, folded but the current one's. A change of section moves what is
   * already there, so no row is drawn again: hover and focus stay where they are.
   */
  function buildSidebar(): void {
    highlight.className = "k-sp-highlight k-control";
    highlight.setAttribute("aria-hidden", "true");
    focusRing(highlight, "control", "accent", false);
    homeRow = sidebarRow("section", outline.home.name, "manage_search", false);
    const rule = document.createElement("span");
    rule.className = "k-sp-rule";
    rule.append(divider());
    const parts: Element[] = [highlight, homeRow, rule];
    for (const [index, each] of sections.entries()) {
      const head = sidebarRow("section", each.title, each.icon, false);
      sectionRows.push(head);
      parts.push(head);
      const lines = names[index]!.map((title) => sidebarRow("sub", title, null, false));
      subRows.push(lines);
      if (lines.length === 0) {
        subLists.push(null);
        bars.push(subBar());
        continue;
      }
      const list = document.createElement("span");
      list.className = "k-sp-subs";
      list.dataset.folded = "";
      list.inert = true;
      const own = subBar();
      list.append(...lines, own);
      subLists.push(list);
      bars.push(own);
      parts.push(list);
    }
    sideRows.dataset.glides = "";
    sideRows.replaceChildren(...parts);
    for (const button of [homeRow, ...sectionRows, ...subRows.flat()]) {
      button.addEventListener("click", () => {
        const at = itemRows.indexOf(button);
        if (at >= 0) pick(items[at]!, false);
      });
    }
  }

  /**
   * Shows the section now current in the sidebar: its row current, its list open and the others
   * folded, the highlight on its row. With `glide` the highlight travels there and the lists fold
   * and unfold over `medium` on `standard`, on one clock, so the rows under them slide and the
   * highlight lands on the row as the row lands; a change asked for meanwhile carries on from
   * where everything is. With reduced motion the lists change at once and the highlight fades in
   * on its row over `short`.
   */
  function drawSidebar(glide = false, play = true): void {
    items = buildItems();
    itemRows = items.map((item) => (item.kind === "home" ? homeRow! : item.kind === "section" ? sectionRows[item.index]! : subRows[current]![item.index]!));
    const from = shownSection;
    shownSection = current;
    for (const [index, row] of [[-1, homeRow!] as const, ...sectionRows.map((row, at) => [at, row] as const)]) {
      row.toggleAttribute("data-current", index === current);
      if (index === current) row.setAttribute("aria-current", "page");
      else row.removeAttribute("aria-current");
    }
    const moving = glide && from !== null && from !== current && root.isConnected;
    const reduced = settings().reduced;
    const duration = played("medium");

    // Read everything first, to the fraction of a pixel a fold leaves mid-way: each list's height
    // as drawn now, its full height, the rows' places, the highlight's.
    // A folded list at rest is out of the layout: laid out at no height while it is measured.
    for (const list of subLists) list?.setAttribute("data-measuring", "");
    const box = sideRows.getBoundingClientRect();
    // The page's zoom, from the rows' width: their height is mid-fold, and `offsetHeight` rounds it.
    const ratio = sideRows.offsetWidth / Math.max(1, box.width);
    const topOf = (element: Element) => (element.getBoundingClientRect().top - box.top) * ratio;
    const heights = subLists.map((list) => (list ? list.getBoundingClientRect().height * ratio : 0));
    // A list's full height is its rows': the bar, placed out of the flow, may reach past them.
    const full = subLists.map((list, index) => {
      const last = subRows[index]?.at(-1);
      return list && last ? last.offsetTop + last.offsetHeight : 0;
    });
    const highlightAt = moving ? currentY() : null;
    const target = sectionRow(current);
    // Where the row will stand once every list has its new height: the lists above it change by
    // the difference.
    const targetTop = topOf(target);
    let targetY = targetTop;
    subLists.forEach((list, index) => {
      if (!list || topOf(list) >= targetTop) return;
      const end = index === current ? full[index]! : 0;
      targetY -= heights[index]! - end;
    });
    for (const list of subLists) list?.removeAttribute("data-measuring");

    for (const each of glides) each.cancel();
    glides = [];
    // A glide's ends are custom properties and its progress one number each (`--fold-at`,
    // `--highlight-at`), so a move's player plays the same thing from its recipe.
    const runs = (element: HTMLElement, property: string, done?: () => void) => {
      if (!play) {
        element.style.setProperty(property, "1");
        return;
      }
      const run = element.animate([{ [property]: "0" }, { [property]: "1" }], { duration, easing: easing("standard") });
      glides.push(run);
      run.finished.then(
        () => done?.(),
        () => {},
      );
    };
    subLists.forEach((list, index) => {
      if (!list) return;
      const open = index === current;
      list.toggleAttribute("data-folded", !open);
      list.inert = !open;
      const end = open ? full[index]! : 0;
      if (!moving || reduced || Math.abs(heights[index]! - end) < 0.01) {
        delete list.dataset.folding;
        return;
      }
      list.dataset.folding = "";
      list.style.setProperty("--fold-from", `${heights[index]}px`);
      list.style.setProperty("--fold-to", `${end}px`);
      list.style.setProperty("--fade-from", String(heights[index]! / Math.max(1, full[index]!)));
      list.style.setProperty("--fade-to", open ? "1" : "0");
      runs(list, "--fold-at", () => {
        delete list.dataset.folding;
        // At rest the list gives its gap back to the stack: its bar is placed again, unmoved on screen.
        if (open) placeBar(false);
      });
    });
    highlight.style.setProperty("--highlight-y", `${targetY}px`);
    if (moving && reduced) {
      if (play) glides.push(highlight.animate([{ opacity: 0 }, { opacity: 1 }], { duration: played("short"), easing: easing("standard") }));
    } else if (moving && highlightAt !== null) {
      highlight.style.setProperty("--highlight-from", `${highlightAt}px`);
      runs(highlight, "--highlight-at");
    }

    sideAt = Math.max(0, Math.min(items.length - 1, sideAt));
    rove();
    placeBar(false);
    if (moving) {
      // The rows move under a pointer that stays where it was: the row it is over now isn't
      // pointed at, so nothing lights under it until the pointer moves again.
      side.dataset.still = "";
    }
  }

  /** The highlight's place as drawn now, mid-glide included, in the rows' own pixels. */
  function currentY(): number {
    const translate = getComputedStyle(highlight).translate.split(" ");
    const y = Number.parseFloat(translate[1] ?? translate[0] ?? "0");
    return Number.isFinite(y) ? y : 0;
  }

  /** The sidebar's one stop for Tab: the row the keyboard is on. */
  function rove(): void {
    itemRows.forEach((button, at) => (button.tabIndex = at === sideAt ? 0 : -1));
  }

  /** The bar beside the current sub-section: gliding there, or put there at once. */
  function placeBar(glide: boolean): void {
    const bar = bars[current] ?? subBar();
    const subs = subRows[current] ?? [];
    subs.forEach((each, s) => {
      each.toggleAttribute("data-current", s === sub);
      if (s === sub) each.setAttribute("aria-current", "location");
      else each.removeAttribute("aria-current");
    });
    const target = subs[sub];
    if (!target) return;
    if (!glide || !bar.isConnected || ms("medium") === 0) {
      putBar(bar, target);
      return;
    }
    const to = `${target.offsetTop}px`;
    const drawn = getComputedStyle(bar).top;
    delete bar.dataset.still;
    bar.style.transition = "none";
    // Where it is drawn is its row's top and its inset in the row: the start is the row's top alone,
    // or the glide would begin a step lower and come back.
    bar.style.setProperty("--bar-from", `calc(${drawn} - var(--bar-inset))`);
    bar.style.setProperty("--bar-to", to);
    bar.style.setProperty("--bar-at", "0");
    void bar.offsetTop;
    bar.style.transition = "";
    bar.style.setProperty("--bar-at", "1");
  }

  /** A sub-section's box in the page, by its index. */
  function subBoxes(): HTMLElement[] {
    return [...rise.querySelectorAll<HTMLElement>(".k-section")];
  }

  function drawPage(): void {
    rows = [];
    wells = [];
    homeField = null;
    homeAnswers = null;
    results = null;
    spyView = view;
    spyLead = () => length("space-s32") * scaleOf(root);
    if (current === -1) {
      rise.replaceChildren(homePage());
      pane.dataset.home = "";
      return;
    }
    delete pane.dataset.home;
    const each = sections[current]!;
    const head = header({ accent: each.accent, title: each.title, subtitle: each.lede, ...(each.trailing ? { trailing: each.trailing() } : {}) });
    const host: RegionHost = {
      rename: (index, title) => {
        const list = names[current];
        if (!list || list[index] === title) return;
        list[index] = title;
        const words = subRows[current]?.[index]?.querySelector(".k-text");
        if (words) words.textContent = title;
      },
      spyOn: (box, lead) => {
        spyView = box ?? view;
        spyLead = () => (lead ?? length("space-s32")) * scaleOf(root);
        spyView.addEventListener("scroll", spy, { passive: true });
      },
      enter: (index) => enterPage(index),
      refresh: () => collectRows(),
      turn: (deeper, swap, arrived, from) => {
        const kept = deeper ? view.scrollTop : (depths.pop() ?? 0);
        if (deeper) depths.push(kept);
        // A page of the region's own belongs to the sub-section it was opened from: the bar stays there.
        heldSub = deeper && from !== undefined ? from : null;
        if (heldSub !== null && heldSub !== sub) {
          sub = heldSub;
          placeBar(true);
        }
        turnPage(
          { axis: "x", sign: deeper ? 1 : -1 },
          () => {
            swap();
            view.scrollTop = deeper ? 0 : kept;
          },
          arrived,
        );
      },
    };
    const body = each.region ? each.region(host) : stack("s32", "start", ...each.subs.map((s, index) => filling(subSection(s, index))));
    rise.replaceChildren(page(stack("s32", "start", filling(head), filling(body))));
    collectRows();
  }

  /** A sub-section: its heading, its line, its wells `space.s16` apart. */
  function subSection(spec: SubSection, index: number): HTMLElement {
    const boxes = spec.wells.map((specs) => {
      const box = settingsWell(specs);
      box.dataset.sub = String(index);
      return box;
    });
    return section(spec.title, spec.body, ...boxes);
  }

  /** The page's rows and wells, in order, wired to the pointer and the keyboard. */
  function collectRows(): void {
    rows = [];
    // The page's rows are its settings' wells' (`settingsWell`): a list of the page's own that a
    // region draws beside them (the networks in range, the layouts) keeps its own keys, and isn't
    // counted, so search and a landing find the setting they name.
    wells = [...rise.querySelectorAll<HTMLElement>(".k-list")].filter((box) => [...box.querySelectorAll(".k-list-row")].some((row) => specs.has(row as HTMLElement)));
    const boxes = subBoxes();
    for (const box of wells) {
      const subOf = boxes.findIndex((each) => each.contains(box));
      box.tabIndex = -1;
      if (!wired.has(box)) box.addEventListener("keydown", pageKeys);
      wired.add(box);
      for (const element of box.querySelectorAll<HTMLElement>(".k-list-row")) {
        const spec = specs.get(element);
        if (spec) rows.push({ spec, sub: Math.max(0, subOf), element });
      }
    }
    for (const each of rows) wire(each);
    pageAt = Math.min(pageAt, rows.length - 1);
    showRow(false);
  }

  /** The pointer on a row puts the keyboard there; a press selects it quietly, with no tube. */
  function wire(each: PageRow): void {
    const element = each.element;
    if (wired.has(element)) return;
    wired.add(element);
    const at = () => rows.findIndex((row) => row.element === element);
    element.addEventListener("pointermove", () => {
      if (pageAt === at() || find.open) return;
      pageAt = at();
      showRow(false);
    });
    element.addEventListener("click", () => {
      pageAt = at();
      showRow(false);
      wellOf(pageAt)?.focus({ preventScroll: true });
    });
  }

  function wellOf(at: number): HTMLElement | undefined {
    return rows[at]?.element.closest<HTMLElement>(".k-list") ?? undefined;
  }

  /** The page's keyboard row lit, its well the page's stop for Tab, and in view if `reveal`. */
  function showRow(reveal: boolean): void {
    rows.forEach((each, at) => each.element.toggleAttribute("data-current", at === pageAt && !find.open));
    const stop = wellOf(pageAt) ?? wellOf(rows.findIndex((each) => each.sub === sub)) ?? wells[0];
    for (const box of wells) box.tabIndex = box === stop ? 0 : -1;
    if (!reveal || pageAt < 0) return;
    const element = rows[pageAt]!.element;
    const room = length("space-s24") * scaleOf(root);
    const top = topIn(element, view);
    const bottom = top + element.offsetHeight;
    if (top - room < view.scrollTop) view.scrollTop = Math.round(top - room);
    else if (bottom + room > view.scrollTop + view.clientHeight) view.scrollTop = Math.round(bottom + room - view.clientHeight);
  }

  /** Home: its header, the search field across every setting, then the results or the sections. */
  function homePage(): HTMLElement {
    const home = outline.home;
    const answers = document.createElement("span");
    answers.className = "k-sp-answers";
    homeAnswers = answers;
    homeField = field({
      icon: "search",
      placeholder: home.placeholder,
      ...(homeQuery ? { value: homeQuery } : {}),
      onInput: (value) => {
        homeQuery = value;
        answers.replaceChildren(answer(value));
      },
      ...(options.keyboard === "home" ? { pinned: "lit" as const } : {}),
    });
    homeField.addEventListener("keydown", homeKeys);
    answers.append(answer(homeQuery));
    const column = stack("s24", "start", filling(header({ accent: home.accent, title: home.title, subtitle: home.lede })), filling(homeField), filling(answers));
    column.classList.add("k-sp-column");
    return page(column);
  }

  /** Under Home's field: the sections while it is empty, else the settings that match. */
  function answer(query: string): HTMLElement {
    if (query.trim() === "") {
      results = null;
      const specs: ListRowSpec[] = sections.map((each) => ({ icon: each.icon, title: each.title, subtitle: each.lede, opens: true }));
      const elements = specs.map((each) => listRow(each));
      elements.forEach((element, index) => element.addEventListener("click", () => go(index)));
      return well("List", stack("s4", "start", ...elements));
    }
    const found: Found[] = [];
    sections.forEach((each, section) => {
      each.subs.forEach((s, subIndex) => {
        let at = 0;
        for (const specs of s.wells) {
          for (const spec of specs) {
            const runs = findRow(spec, query);
            if (runs) found.push({ spec, section, sub: subIndex, row: at, runs });
            at++;
          }
        }
        if (s.tip !== undefined && findRow({ title: "", tip: s.tip }, query)) found.push({ spec: { title: s.title }, section, sub: subIndex, row: 0, runs: [], heading: true });
      });
    });
    // What its name matches first, then what its line or tip does, each in the sidebar's order.
    found.sort((a, b) => Number(a.runs.length === 0) - Number(b.runs.length === 0));
    if (found.length === 0) {
      results = null;
      const none = document.createElement("span");
      none.className = "k-sp-none";
      none.append(text(`Nothing called "${query.trim()}". Yet.`, "body", "on_surface_variant"));
      return none;
    }
    const elements = found.map((each) =>
      listRow({
        title: each.spec.title,
        ...(each.runs.length ? { matched: each.runs } : {}),
        subtitle: each.heading ? sections[each.section]!.title : `${sections[each.section]!.title} › ${sections[each.section]!.subs[each.sub]!.title}`,
        ...(each.spec.detail !== undefined ? { detail: each.spec.detail } : each.spec.toggle !== undefined ? { detail: each.spec.toggle ? "On" : "Off" } : {}),
      }),
    );
    const list = well("List", stack("s4", "start", ...elements));
    results = { found, rows: elements, at: 0 };
    elements.forEach((element, at) => {
      element.addEventListener("pointermove", () => lightResult(at));
      element.addEventListener("click", () => land(found[at]!));
    });
    lightResult(0);
    return list;
  }

  /** Home's field set to `query`, as typed, with its answers under it. */
  function ask(query: string): void {
    if (homeField) setValue(homeField, query);
    homeQuery = query;
    homeAnswers?.replaceChildren(answer(query));
  }

  function lightResult(at: number): void {
    if (!results) return;
    results.at = at;
    results.rows.forEach((each, index) => each.toggleAttribute("data-current", index === at));
  }

  /** Goes to a setting found from Home: its section, scrolled to its row, the row landed on. */
  function land(found: Found): void {
    const from = current;
    current = found.section;
    sub = found.sub;
    let element: HTMLElement | undefined;
    // The page turns in already scrolled to the row; the highlight plays once it has arrived.
    const drawn = () => {
      const at = rows.findIndex((each) => each.sub === found.sub) + found.row;
      element = rows[at]?.element;
      if (!element) return;
      pageAt = at;
      const top = topIn(element, view) - length("space-s32") * scaleOf(root) - length(`type-heading-line`) * scaleOf(root);
      view.scrollTop = Math.round(Math.max(0, top));
      pressed = { sub: found.sub, top: view.scrollTop };
      placeBar(false);
      showRow(false);
      wellOf(at)?.focus({ preventScroll: true });
      spy();
    };
    const arrived = () => {
      if (!element) return;
      element.removeAttribute("data-landed");
      void element.offsetTop;
      element.dataset.landed = "";
    };
    change(true, along(from, current), drawn, arrived);
  }

  function homeKeys(event: KeyboardEvent): void {
    const key = event.key.toLowerCase();
    const ctrl = event.ctrlKey;
    if (event.key === "ArrowDown" || (ctrl && (key === "j" || key === "n"))) {
      if (results) lightResult((results.at + 1) % results.rows.length);
    } else if (event.key === "ArrowUp" || (ctrl && (key === "k" || key === "p"))) {
      if (results) lightResult((results.at - 1 + results.rows.length) % results.rows.length);
    } else if (event.key === "Enter") {
      if (!event.repeat && results) land(results.found[results.at]!);
    } else if (event.key === "Escape") {
      if (homeQuery !== "") ask("");
      else toSidebar();
    } else return;
    event.preventDefault();
    event.stopPropagation();
  }

  // A page turn, as a panel swapped for its twin moves: the page shown leaves `space.s24` toward one
  // side and fades over `micro` on `emphasized_accel`, then the next is drawn and comes in from the
  // other side over `short` on `emphasized_decel`. A turn asked for meanwhile retargets it: a page still leaving goes on
  // leaving and the latest turn's page comes in; a page coming in leaves from where it is. With
  // reduced motion the page left goes at once and the next fades in over `short` on `standard`.
  let turning: "idle" | "leaving" | "arriving" = "idle";
  let latest: Turn | null = null;
  let animations: Animation[] = [];
  /** Each turn's own count: a stopped animation's end tells nobody. */
  let turnCount = 0;

  /** The page element the turn moves: the section's or Home's page, its title with it. */
  function pageShown(): HTMLElement | null {
    return rise.firstElementChild as HTMLElement | null;
  }

  /** Where the page is in its move now: its shift and how much it shows. */
  function standing(element: HTMLElement): [number, number] {
    const style = getComputedStyle(element);
    const read = (name: string, otherwise: number) => {
      const value = Number.parseFloat(style.getPropertyValue(name));
      return Number.isFinite(value) ? value : otherwise;
    };
    return [read("--page-shift", 0), read("--page-shown", 1)];
  }

  function stopAnimations(): void {
    for (const each of animations) each.cancel();
    animations = [];
  }

  /** Plays `turn`: the page shown leaves, then `turn`'s is drawn and arrives. */
  function turnPage(way: Way, swap: () => void, arrived?: () => void): void {
    latest = { way, swap, arrived };
    if (turning === "leaving") return;
    const page = pageShown();
    const caught = turning === "arriving";
    const from: [number, number] = page && caught ? standing(page) : [0, 1];
    stopAnimations();
    const count = ++turnCount;
    if (!page || settings().reduced || !root.isConnected) {
      bringIn(count);
      return;
    }
    turning = "leaving";
    rise.dataset.moving = "";
    // A page caught coming in leaves along the way it came, so it never jumps sideways.
    if (!caught) rise.dataset.axis = way.axis;
    const leave = moving(page, from, [-way.sign, 0], played("micro") * from[1], "emphasized_accel");
    animations = [leave];
    leave.finished.then(
      () => count === turnCount && bringIn(count),
      () => {},
    );
  }

  /** The latest turn's page drawn, then coming in. */
  function bringIn(count: number): void {
    const turn = latest;
    latest = null;
    if (!turn) return;
    stopAnimations();
    turn.swap();
    const page = pageShown();
    if (!page || !root.isConnected) {
      settle();
      turn.arrived?.();
      return;
    }
    turning = "arriving";
    rise.dataset.moving = "";
    rise.dataset.axis = turn.way.axis;
    // The new page waits hidden where its arrival starts until it has been drawn once: a page drawn
    // for the first time costs a frame's style, layout and paint, and an arrival started before it
    // would spend its first frames unseen while that frame is made.
    const reduced = settings().reduced;
    const from: [number, number] = reduced ? [0, 0] : [turn.way.sign, 0];
    page.style.setProperty("--page-shift", String(from[0]));
    page.style.setProperty("--page-shown", "0");
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        if (count !== turnCount) return;
        const arrive = reduced ? moving(page, from, [0, 1], played("short"), "standard") : moving(page, from, [0, 1], played("short"), "emphasized_decel");
        animations = [arrive];
        arrive.finished.then(
          () => {
            if (count !== turnCount) return;
            settle();
            turn.arrived?.();
          },
          () => {},
        );
      }),
    );
  }

  /** The turn done: the page at rest, its animation let go. */
  function settle(): void {
    stopAnimations();
    turning = "idle";
    delete rise.dataset.moving;
    const page = pageShown();
    page?.style.removeProperty("--page-shift");
    page?.style.removeProperty("--page-shown");
  }

  /** The way from section `from` to `to` (Home, -1, the top): down the list rises in from below. */
  function along(from: number, to: number): Way {
    return { axis: "y", sign: to > from ? 1 : -1 };
  }

  /** The scroll a page turned deeper left, for Back to put it back. */
  const depths: number[] = [];
  /** The sub-section a page of the region's own was opened from, the bar held beside it, or none. */
  let heldSub: number | null = options.heldSub ?? null;

  /**
   * Changes what the page shows to the section now current. The sidebar follows at once; with
   * `turns`, the page turns along `way`, else it is drawn at once. `drawn` runs as the new page is
   * drawn, at its top; `arrived` once it is in.
   */
  function change(turns: boolean, way: Way = { axis: "y", sign: 1 }, drawn?: () => void, arrived?: () => void): void {
    if (find.open) closeFind(false);
    drawSidebar(turns);
    depths.length = 0;
    heldSub = null;
    const swap = () => {
      drawPage();
      view.scrollTop = 0;
      pressed = null;
      drawn?.();
    };
    if (turns) turnPage(way, swap, arrived);
    else {
      swap();
      arrived?.();
    }
  }

  /** Goes to section `index` (-1 Home), its first sub-section current. */
  function go(index: number, keyboard = false, drawn?: () => void): void {
    if (index === current) return;
    const from = current;
    current = index;
    sub = 0;
    pageAt = -1;
    const home = index === -1 && !keyboard;
    change(true, along(from, index), () => {
      // Home pressed puts the keyboard in Home's field, where the window opens, once it is drawn.
      if (home) homeField?.focus({ preventScroll: true });
      drawn?.();
    });
    sideAt = items.findIndex((item) => (index === -1 ? item.kind === "home" : item.kind === "section" && item.index === index));
    rove();
    // The sidebar is drawn again, so the row pressed is a new one: the keyboard goes onto it,
    // quietly for a press.
    if (!home) itemRows[sideAt]?.focus({ preventScroll: true });
  }

  /** Scrolls the page to sub-section `index`, its heading at the lead line, the bar gliding there. */
  function toSub(index: number): void {
    const box = subBoxes()[index];
    if (!box) return;
    sub = index;
    const top = Math.max(0, Math.min(spyView.scrollHeight - spyView.clientHeight, Math.round(topIn(box, spyView) - spyLead())));
    pressed = { sub: index, top };
    glideScroll(spyView, top);
    placeBar(true);
  }

  function pick(item: Item, keyboard: boolean): void {
    sideAt = items.indexOf(item);
    if (item.kind === "home") go(-1, keyboard);
    else if (item.kind === "section") go(item.index, keyboard);
    else toSub(item.index);
    rove();
  }

  /** Follows the page's scroll: the current sub-section is the last whose heading reached the lead line. */
  function spy(): void {
    if (current === -1 || heldSub !== null) return;
    const boxes = subBoxes();
    if (boxes.length === 0) return;
    // A page shorter than where a press sent it stops at its end: the press still holds there.
    const end = spyView.scrollHeight - spyView.clientHeight;
    // On its way to where a press sent it, the page passes other headings: the press holds through
    // them, or the bar would turn back to each before going on.
    if (pressed && gliding.has(spyView)) return;
    if (pressed && Math.abs(spyView.scrollTop - Math.min(pressed.top, end)) <= 1) {
      if (sub !== pressed.sub) {
        sub = pressed.sub;
        placeBar(true);
      }
      return;
    }
    pressed = null;
    const line = spyView.scrollTop + spyLead() + 1;
    let at = 0;
    boxes.forEach((box, index) => {
      if (topIn(box, spyView) <= line) at = index;
    });
    if (end > 1 && spyView.scrollTop >= end - 1) at = boxes.length - 1;
    if (at === sub) return;
    sub = at;
    placeBar(true);
  }
  view.addEventListener("scroll", spy, { passive: true });
  view.addEventListener("wheel", () => stopGlide(view), { passive: true });

  function toSidebar(): void {
    const target = current === -1 ? 0 : items.findIndex((item) => item.kind === "sub" && item.index === (rows[pageAt]?.sub ?? sub));
    sideAt = target >= 0 ? target : Math.max(0, items.findIndex((item) => item.kind === "section" && item.index === current));
    rove();
    itemRows[sideAt]?.focus({ preventScroll: true });
    showRow(false);
  }

  /** Into the page: Home's field, or the first row of sub-section `index`. */
  function enterPage(index = sub): void {
    if (current === -1) {
      homeField?.focus({ preventScroll: true });
      return;
    }
    const at = rows.findIndex((each) => each.sub === index);
    if (at < 0) {
      rise.querySelector<HTMLElement>("[tabindex='0']")?.focus({ preventScroll: true });
      return;
    }
    pageAt = at;
    showRow(true);
    wellOf(at)?.focus({ preventScroll: true });
  }

  /** How many sidebar rows show whole: Page Up and Page Down's step. */
  function sidePage(): number {
    const room = sideScroll.view.clientHeight;
    const step = (length("size-row") + length("space-s4")) * scaleOf(root);
    return Math.max(1, Math.floor(room / step));
  }

  function keepInView(button: HTMLElement): void {
    const pad = length("space-s12") * scaleOf(root);
    const top = topIn(button, sideScroll.view);
    const scroller = sideScroll.view;
    if (top - pad < scroller.scrollTop) scroller.scrollTop = Math.round(top - pad);
    else if (top + button.offsetHeight + pad > scroller.scrollTop + scroller.clientHeight) scroller.scrollTop = Math.round(top + button.offsetHeight + pad - scroller.clientHeight);
  }

  function sideKeys(event: KeyboardEvent): void {
    if (!(event.target instanceof HTMLElement) || !event.target.classList.contains("k-sp-row")) return;
    const key = event.key;
    const lower = key.toLowerCase();
    const ctrl = event.ctrlKey;
    const step = (to: number) => {
      sideAt = ((to % items.length) + items.length) % items.length;
      const item = items[sideAt]!;
      pick(item, true);
      const button = itemRows[sideAt];
      button?.focus({ preventScroll: true });
      if (button) keepInView(button);
    };
    if (key === "ArrowDown" || (!ctrl && lower === "j") || (ctrl && (lower === "j" || lower === "n"))) step(sideAt + 1);
    else if (key === "ArrowUp" || (!ctrl && lower === "k") || (ctrl && (lower === "k" || lower === "p"))) step(sideAt - 1);
    else if (key === "Home") step(0);
    else if (key === "End") step(items.length - 1);
    else if (key === "PageDown" && !ctrl) step(Math.min(items.length - 1, sideAt + sidePage()));
    else if (key === "PageUp" && !ctrl) step(Math.max(0, sideAt - sidePage()));
    else if (key === "ArrowRight" || (!ctrl && lower === "l") || key === "Enter") {
      if (event.repeat && key === "Enter") return;
      const item = items[sideAt]!;
      enterPage(item.kind === "sub" ? item.index : sub);
    } else return;
    event.preventDefault();
    event.stopPropagation();
  }
  side.addEventListener("keydown", sideKeys);

  function pageKeys(event: KeyboardEvent): void {
    if (rows.length === 0) return;
    const key = event.key;
    const lower = key.toLowerCase();
    const ctrl = event.ctrlKey;
    const to = (at: number, wrap: boolean) => {
      pageAt = wrap ? ((at % rows.length) + rows.length) % rows.length : Math.max(0, Math.min(rows.length - 1, at));
      showRow(true);
      wellOf(pageAt)?.focus({ preventScroll: true });
    };
    const page = () => {
      const each = rows[Math.max(0, pageAt)]!.element;
      return Math.max(1, Math.floor(view.clientHeight / (each.offsetHeight + length("space-s4") * scaleOf(root))));
    };
    if (key === "ArrowDown" || (!ctrl && lower === "j") || (ctrl && (lower === "j" || lower === "n"))) to(pageAt + 1, true);
    else if (key === "ArrowUp" || (!ctrl && lower === "k") || (ctrl && (lower === "k" || lower === "p"))) to(pageAt - 1, true);
    else if (key === "Home") to(0, false);
    else if (key === "End") to(rows.length - 1, false);
    else if (key === "PageDown" && !ctrl) to(pageAt + page(), false);
    else if (key === "PageUp" && !ctrl) to(Math.max(0, pageAt) - page(), false);
    else if (key === "ArrowLeft" || (!ctrl && lower === "h") || key === "Escape") toSidebar();
    else if (key === "Enter" || key === " ") {
      if (!event.repeat) rows[pageAt]?.element.click();
    } else return;
    event.preventDefault();
    event.stopPropagation();
  }

  /** The find box: a field and its count on a popover in the page's top-right corner. */
  function openFind(query = "", focus = true): void {
    if (current === -1) return;
    if (!find.open) {
      find.saved = { top: view.scrollTop, sub, row: pageAt, focus: document.activeElement };
      find.open = true;
    }
    if (!find.box) {
      find.count = figure("", "mono", "on_surface_variant");
      find.count.classList.add("k-sp-count");
      find.field = field({ icon: "search", placeholder: "Find on this page", ...(query ? { value: query } : {}), onInput: (value) => search(value, 0) });
      find.field.addEventListener("keydown", findKeys);
      find.note = keyHints([["Enter", "Search every setting from Home"]]);
      find.note.classList.add("k-sp-note");
      const content = stack("s8", "start", filling(row("s12", "centre", filling(find.field), find.count)), find.note);
      const box = popover(2, content);
      box.classList.add("k-sp-find");
      find.box = box;
      pane.append(box);
      // Opened by a key it fades in; a sample drawn with it open shows it at rest from the start.
      if (focus) requestAnimationFrame(() => box.setAttribute("data-shown", ""));
      else box.setAttribute("data-shown", "");
    }
    placeFind();
    if (focus) find.field?.focus({ preventScroll: true });
    else if (options.keyboard === "find") find.field?.setAttribute("data-pinned", "lit");
    search(query, options.findAt ?? 0);
  }

  function placeFind(): void {
    if (!find.box) return;
    const width = pane.clientWidth / scaleOf(root);
    find.box.style.setProperty("--padding", `var(--space-${containerPadding(width)})`);
  }
  new ResizeObserver(placeFind).observe(pane);

  /** Lights what `query` matches on the page, on match `at`, the page scrolled to it. */
  function search(query: string, at: number): void {
    find.query = query;
    find.matches = [];
    const blank = query.trim() === "";
    rows.forEach((each, index) => {
      const runs = blank ? null : findRow(each.spec, query);
      const fresh = listRow(runs?.length ? { ...each.spec, matched: runs } : each.spec);
      specs.set(fresh, each.spec);
      each.element.replaceWith(fresh);
      each.element = fresh;
      wire(each);
      if (blank) delete fresh.dataset.found;
      else fresh.dataset.found = runs ? "match" : "miss";
      if (runs) find.matches.push(index);
    });
    find.at = find.matches.length === 0 ? 0 : Math.max(0, Math.min(find.matches.length - 1, at));
    if (find.count) {
      // The count keeps its room while nothing is typed, so the field doesn't narrow under the
      // first letter.
      tick(find.count, "mono", blank ? "0 of 0" : `${find.matches.length === 0 ? 0 : find.at + 1} of ${find.matches.length}`);
      if (blank && find.count.firstElementChild) find.count.firstElementChild.textContent = "";
    }
    find.box?.toggleAttribute("data-none", !blank && find.matches.length === 0);
    showMatch();
  }

  function showMatch(): void {
    rows.forEach((each, index) => {
      if (each.element.dataset.found === "current") each.element.dataset.found = "match";
      if (find.matches[find.at] === index) each.element.dataset.found = "current";
    });
    const index = find.matches[find.at];
    if (index === undefined) return;
    pageAt = index;
    showRow(false);
    if (!root.isConnected) return;
    const element = rows[index]!.element;
    const box = find.box;
    // The row comes into view below the box, which covers the page's top-right corner.
    const clear = box ? box.offsetTop + box.offsetHeight + length("space-s16") * scaleOf(root) : length("space-s32") * scaleOf(root);
    const top = topIn(element, view);
    if (top - clear < view.scrollTop || top + element.offsetHeight > view.scrollTop + view.clientHeight) view.scrollTop = Math.round(Math.max(0, top - clear));
    spy();
  }

  function findKeys(event: KeyboardEvent): void {
    const count = find.matches.length;
    if (event.key === "Escape") closeFind(true);
    else if ((event.key === "Enter" && !event.shiftKey) || event.key === "ArrowDown") {
      if (event.key === "Enter" && event.repeat) return;
      if (count === 0 && event.key === "Enter" && find.query.trim() !== "") {
        const query = find.query;
        closeFind(false);
        go(-1);
        ask(query);
        homeField?.focus({ preventScroll: true });
      } else if (count > 0) {
        find.at = (find.at + 1) % count;
        search(find.query, find.at);
      }
    } else if ((event.key === "Enter" && event.shiftKey) || event.key === "ArrowUp") {
      if (count > 0) search(find.query, (find.at - 1 + count) % count);
    } else return;
    event.preventDefault();
    event.stopPropagation();
  }

  /** Closes the find box: the page put back as it was, and the keyboard where it was if `back`. */
  function closeFind(back: boolean): void {
    if (!find.open) return;
    find.open = false;
    const saved = find.saved;
    const box = find.box;
    find.box = null;
    if (box) {
      box.removeAttribute("data-shown");
      const gone = () => box.remove();
      if (ms("short") === 0) gone();
      else box.addEventListener("transitionend", gone, { once: true });
    }
    search("", 0);
    find.open = false;
    if (saved && back) {
      view.scrollTop = saved.top;
      sub = saved.sub;
      pageAt = saved.row;
      pressed = { sub: saved.sub, top: view.scrollTop };
      placeBar(false);
      if (saved.focus instanceof HTMLElement && root.contains(saved.focus)) saved.focus.focus({ preventScroll: true });
      else toSidebar();
    }
    showRow(false);
  }

  /** The keys the template takes from anywhere in it: find, the next section, and Tab round its stops. */
  root.addEventListener("keydown", (event) => {
    const key = event.key;
    const ctrl = event.ctrlKey;
    if (ctrl && (key === "PageDown" || key === "PageUp")) {
      const count = sections.length + 1;
      const next = ((current + 1 + (key === "PageDown" ? 1 : -1)) % count + count) % count - 1;
      // The keyboard stays on its side: in the sidebar, or in the new page.
      const inSidebar = document.activeElement instanceof HTMLElement && side.contains(document.activeElement);
      go(next, inSidebar, () => {
        if (!inSidebar && next !== -1) enterPage(0);
      });
    } else if ((key === "/" && !isTyping(event)) || (ctrl && key.toLowerCase() === "f")) {
      if (current === -1) homeField?.focus({ preventScroll: true });
      else openFind(find.query);
    } else if (key === "Tab") {
      tab(event.shiftKey ? -1 : 1);
    } else return;
    event.preventDefault();
  });

  function isTyping(event: KeyboardEvent): boolean {
    return event.target instanceof HTMLElement && event.target.classList.contains("k-field");
  }

  /** Tab and Shift+Tab round the window's stops: the find box or Home's field, the sidebar, the page. */
  function tab(by: 1 | -1): void {
    const stops: HTMLElement[] = [];
    if (current === -1 && homeField) stops.push(homeField);
    if (find.open && find.field) stops.push(find.field);
    const row = itemRows[sideAt];
    if (row) stops.push(row);
    if (current !== -1) stops.push(...[...rise.querySelectorAll<HTMLElement>("[tabindex='0'], button:not([tabindex='-1']):not(:disabled)")].filter((each) => !each.closest(".k-list") || each.classList.contains("k-list")));
    if (stops.length === 0) return;
    const active = document.activeElement;
    const at = stops.findIndex((each) => each === active || each.contains(active));
    const next = stops[((at + by) % stops.length + stops.length) % stops.length]!;
    next.focus({ preventScroll: true });
    if (next.classList.contains("k-list") && pageAt < 0) enterPage();
    if (next.classList.contains("k-sp-row") && !active?.classList.contains("k-sp-row")) itemRows[sideAt]?.focus({ preventScroll: true });
  }

  buildSidebar();
  drawSidebar();
  // A move's sample marks its two lists as folding at once, so its player finds them; their
  // measured ends are set once the sample is laid out, below.
  if (options.glideFrom !== undefined) {
    for (const index of [options.glideFrom, current]) {
      const list = subLists[index];
      if (!list) continue;
      list.dataset.folding = "";
      for (const [name, value] of [["--fold-from", "0px"], ["--fold-to", "0px"], ["--fade-from", "0"], ["--fade-to", "0"], ["--fold-at", "1"]] as const) list.style.setProperty(name, value);
    }
  }
  drawPage();
  // Laid out once it is in its window: the highlight on the current row, the bar beside its sub-section.
  requestAnimationFrame(() => {
    highlight.style.setProperty("--highlight-y", `${sectionRow(current).offsetTop}px`);
    placeBar(false);
    if (options.glideFrom === undefined) return;
    // A move's sample: drawn at the section glided from, then at the glide's end, unplayed.
    const to = current;
    current = options.glideFrom;
    drawSidebar();
    highlight.style.setProperty("--highlight-y", `${sectionRow(current).offsetTop}px`);
    current = to;
    drawSidebar(true, false);
  });
  // The pointer moving again, after the rows moved under it, may light what it is over.
  let pointer: { x: number; y: number } | null = null;
  side.addEventListener("pointermove", (event) => {
    if (side.dataset.still === undefined) pointer = { x: event.clientX, y: event.clientY };
    else if (!pointer || Math.abs(event.clientX - pointer.x) + Math.abs(event.clientY - pointer.y) > 0) {
      delete side.dataset.still;
      pointer = { x: event.clientX, y: event.clientY };
    }
  });
  side.addEventListener("pointerleave", () => delete side.dataset.still);
  if (options.find !== undefined) openFind(options.find, options.focus ?? false);
  if (options.landed !== undefined) rows[options.landed]?.element.setAttribute("data-landed", "");
  requestAnimationFrame(() => {
    if (options.sub && current !== -1 && heldSub === null) {
      const box = subBoxes()[options.sub];
      if (box) view.scrollTop = Math.round(topIn(box, view) - spyLead());
      sub = options.sub;
      pressed = { sub, top: view.scrollTop };
      placeBar(false);
    }
    if (options.keyboard === "sidebar") itemRows[sideAt]?.setAttribute("data-pinned", "lit");
    if (options.keyboard === "page" && pageAt >= 0) {
      showRow(false);
      const box = wellOf(pageAt);
      if (box) box.dataset.pinned = "lit";
    }
    if (options.find !== undefined) showMatch();
    if (options.focus) homeField?.focus({ preventScroll: true });
  });
  return probe(root, { role: "component", name: "SidebarPage", variant: "", fixed: { width: false, height: false, holds: true } });
}

/** The wells and rows already listening, so a region drawn again doesn't wire them twice. */
const wired = new WeakSet<HTMLElement>();

/** Each page row's spec, kept by its element, so the find box can draw it again lit. */
const specs = new WeakMap<HTMLElement, ListRowSpec>();

/**
 * A well of a sub-section's rows, `space.s4` apart, each remembering what it was drawn from: a
 * region's wells are made with it too, so the page's keys and the find box reach their rows.
 */
export function settingsWell(rows: readonly ListRowSpec[]): HTMLElement {
  const elements = rows.map((spec) => {
    const element = listRow(spec);
    specs.set(element, spec);
    return element;
  });
  const rowsBox = stack("s4", "start", ...elements);
  rowsBox.classList.add("k-list-rows");
  return well("List", rowsBox);
}
