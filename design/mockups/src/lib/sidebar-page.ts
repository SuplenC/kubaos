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
import { clamped, type Run, text } from "./text.ts";
import { cubic, length, raw } from "./tokens.ts";
import { popover } from "./tooltip.ts";
import { settings } from "../site/settings.ts";

/** A sub-section of a section's page: its heading, a line under it, and its wells of rows. */
export interface SubSection {
  readonly title: string;
  readonly body?: string;
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
}

/** A section: its row in the sidebar, its page's header and its sub-sections. */
export interface Section {
  readonly icon: string;
  readonly title: string;
  readonly accent: string;
  readonly lede: string;
  readonly subs: readonly SubSection[];
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

/** Stops `view`'s glide where it is: the wheel took over. */
function stopGlide(view: HTMLElement): void {
  glides.set(view, (glides.get(view) ?? 0) + 1);
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
  const frame = (now: number) => {
    if (glides.get(view) !== glide) return;
    const t = Math.min(1, (now - start) / duration);
    view.scrollTop = Math.round(from + (target - from) * curve(t));
    if (t < 1) requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}

/** A row of the sidebar: a section's (Home's too) or a sub-section's, pinned in a state for a sample. */
export function sidebarRow(kind: "section" | "sub", words: string, glyph: string | null, current: boolean, pinned?: Pinned): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "k-sp-row k-control";
  button.dataset.kind = kind;
  button.dataset.hot = "";
  button.tabIndex = -1;
  button.toggleAttribute("data-current", current);
  if (current) button.setAttribute("aria-current", "page");
  pin(button, pinned);
  const name = clamped(words, kind === "section" ? "body" : "small");
  name.style.color = "var(--words)";
  const parts: Element[] = [];
  if (glyph) parts.push(icon(glyph, "var(--type-body-icon)", "var(--glyph)"));
  parts.push(filling(name));
  button.append(filling(row("s12", "centre", ...parts)));
  focusRing(button, "control", "accent", false).dataset.pressable = "";
  return probe(button, { role: "component", name: "SidebarRow", variant: kind, fixed: FIXED_HEIGHT });
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
  let sub = options.sub ?? 0;
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
  const bar = document.createElement("span");
  bar.className = "k-sp-bar";

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

  function drawSidebar(): void {
    items = buildItems();
    itemRows = [];
    const parts: Element[] = [];
    const home = sidebarRow("section", outline.home.name, "manage_search", current === -1);
    itemRows.push(home);
    const rule = document.createElement("span");
    rule.className = "k-sp-rule";
    rule.append(divider());
    parts.push(home, rule);
    for (const [index, each] of sections.entries()) {
      const head = sidebarRow("section", each.title, each.icon, index === current);
      itemRows.push(head);
      parts.push(head);
      if (index !== current || names[index]!.length === 0) continue;
      const subs = document.createElement("span");
      subs.className = "k-sp-subs";
      for (const [s, title] of names[index]!.entries()) {
        const line = sidebarRow("sub", title, null, s === sub);
        itemRows.push(line);
        subs.append(line);
      }
      subs.append(bar);
      parts.push(subs);
    }
    sideRows.replaceChildren(...parts);
    itemRows.forEach((button, at) => {
      button.addEventListener("click", () => pick(items[at]!, false));
    });
    sideAt = Math.max(0, Math.min(items.length - 1, sideAt));
    rove();
    requestAnimationFrame(() => placeBar(false));
  }

  /** The sidebar's one stop for Tab: the row the keyboard is on. */
  function rove(): void {
    itemRows.forEach((button, at) => (button.tabIndex = at === sideAt ? 0 : -1));
  }

  /** The bar beside the current sub-section: gliding there, or put there at once. */
  function placeBar(glide: boolean): void {
    const subs = itemRows.filter((_, at) => items[at]?.kind === "sub");
    subs.forEach((each, s) => {
      each.toggleAttribute("data-current", s === sub);
      if (s === sub) each.setAttribute("aria-current", "location");
      else each.removeAttribute("aria-current");
    });
    const target = subs[sub];
    if (!target) return;
    const to = `${target.offsetTop}px`;
    const drawn = getComputedStyle(bar).top;
    if (!glide || !bar.isConnected || ms("medium") === 0) {
      bar.dataset.still = "";
      bar.style.setProperty("--bar-from", to);
      bar.style.setProperty("--bar-to", to);
      bar.style.setProperty("--bar-at", "1");
      return;
    }
    delete bar.dataset.still;
    bar.style.transition = "none";
    bar.style.setProperty("--bar-from", drawn);
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
    const head = header({ accent: each.accent, title: each.title, subtitle: each.lede });
    const host: RegionHost = {
      rename: (index, title) => {
        const list = names[current];
        if (!list || list[index] === title) return;
        list[index] = title;
        const subs = itemRows.filter((_, at) => items[at]?.kind === "sub");
        const words = subs[index]?.querySelector(".k-text");
        if (words) words.textContent = title;
      },
      spyOn: (box, lead) => {
        spyView = box ?? view;
        spyLead = () => (lead ?? length("space-s32")) * scaleOf(root);
        spyView.addEventListener("scroll", spy, { passive: true });
      },
      enter: (index) => enterPage(index),
      refresh: () => collectRows(),
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
    wells = [...rise.querySelectorAll<HTMLElement>(".k-list")];
    const boxes = subBoxes();
    for (const box of wells) {
      const subOf = boxes.findIndex((each) => each.contains(box));
      box.tabIndex = -1;
      if (!wired.has(box)) box.addEventListener("keydown", pageKeys);
      wired.add(box);
      for (const element of box.querySelectorAll<HTMLElement>(".k-list-row")) {
        const spec = specs.get(element) ?? { title: element.querySelector(".k-list-title")?.textContent ?? "" };
        rows.push({ spec, sub: Math.max(0, subOf), element });
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
            const runs = matchRuns(spec.title, query);
            if (runs) found.push({ spec, section, sub: subIndex, row: at, runs });
            at++;
          }
        }
      });
    });
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
        matched: each.runs,
        subtitle: `${sections[each.section]!.title} › ${sections[each.section]!.subs[each.sub]!.title}`,
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
    current = found.section;
    sub = found.sub;
    change(true);
    const at = rows.findIndex((each) => each.sub === found.sub) + found.row;
    const element = rows[at]?.element;
    if (!element) return;
    pageAt = at;
    requestAnimationFrame(() => {
      const top = topIn(element, view) - length("space-s32") * scaleOf(root) - length(`type-heading-line`) * scaleOf(root);
      view.scrollTop = Math.round(Math.max(0, top));
      pressed = { sub: found.sub, top: view.scrollTop };
      placeBar(false);
      showRow(false);
      wellOf(at)?.focus({ preventScroll: true });
      element.removeAttribute("data-landed");
      void element.offsetTop;
      element.dataset.landed = "";
      spy();
    });
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

  /** Changes what the page shows to the section now current: the rise, at the top, the bar in place. */
  function change(rises: boolean): void {
    if (find.open) closeFind(false);
    drawSidebar();
    drawPage();
    view.scrollTop = 0;
    pressed = null;
    if (!rises || ms("short") === 0) return;
    rise.style.transition = "none";
    rise.style.setProperty("--section-rise", "0");
    void rise.offsetTop;
    rise.style.transition = "";
    rise.style.setProperty("--section-rise", "1");
  }

  /** Goes to section `index` (-1 Home), its first sub-section current. */
  function go(index: number, keyboard = false): void {
    if (index === current) return;
    current = index;
    sub = 0;
    pageAt = -1;
    change(true);
    sideAt = items.findIndex((item) => (index === -1 ? item.kind === "home" : item.kind === "section" && item.index === index));
    rove();
    // The sidebar is drawn again, so the row pressed is a new one: the keyboard goes onto it,
    // quietly for a press. Home pressed puts it in Home's field, where the window opens.
    if (index === -1 && !keyboard) homeField?.focus({ preventScroll: true });
    else itemRows[sideAt]?.focus({ preventScroll: true });
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
    if (current === -1) return;
    const boxes = subBoxes();
    if (boxes.length === 0) return;
    // A page shorter than where a press sent it stops at its end: the press still holds there.
    const end = spyView.scrollHeight - spyView.clientHeight;
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
      requestAnimationFrame(() => box.setAttribute("data-shown", ""));
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
      const runs = blank ? null : matchRuns(each.spec.title, query);
      const fresh = listRow(runs ? { ...each.spec, matched: runs } : each.spec);
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
      go(next, inSidebar);
      if (!inSidebar && next !== -1) enterPage(0);
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

  drawSidebar();
  drawPage();
  if (options.find !== undefined) openFind(options.find, options.focus ?? false);
  if (options.landed !== undefined) rows[options.landed]?.element.setAttribute("data-landed", "");
  requestAnimationFrame(() => {
    if (options.sub && current !== -1) {
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
