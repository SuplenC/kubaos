// A card: `kubaos-ui`'s `card::card` and `card::choice_card`, `KubaOS.Components`' `Card` and
// `ChoiceCard` (`docs/design/components/card.md`). A region of the page, flat on it: one container
// role above what it sits on, with the sunken soft hairline for its edge, at the content radius,
// `space.s16` inside. What it holds is a header, a body and a button bar at its foot, `space.s16`
// apart.
//
// A plain card is a surface and takes no input. A pressable one is a control in the state model
// (`.k-control` in `lib.css`): the whole card is one press target, lit with the accent's tube
// inside its edge. A choice card is one of a set to pick from, its tube cold since the accent
// already means picked; picked, its title and symbol are the accent's words (`primary_lit`) and its
// mark (a radio's disc or a checkbox's box) is filled. Disabled, either keeps its resting surface
// and dims what it holds.

import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { iconTile } from "./icon-tile.ts";
import { fill, filling, row, split, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { length } from "./tokens.ts";
import { milliseconds, strikeSteps } from "../site/recipe.ts";
import { settings } from "../site/settings.ts";
import { text, wrapped } from "./text.ts";

export interface HeaderOptions {
  /** A symbol before the title, `subheading`'s 24. */
  readonly glyph?: string;
  readonly title: string;
  /** A line under the title, `small`. */
  readonly subtitle?: string;
  /** A mark at the header's end, on the title's first line: a choice card's disc or box. */
  readonly end?: Element;
}

/**
 * A card's header: its symbol and title on one line, `space.s8` apart, the title in `subheading`;
 * the line under it `small` in `on_surface_variant`, `space.s4` below, from the card's padding.
 */
export function cardHeader(options: HeaderOptions): HTMLElement {
  const title = wrapped(options.title, "subheading");
  title.classList.add("k-card-title");
  title.style.color = "var(--title-colour, var(--color-on_surface))";
  const glyph = options.glyph ? [icon(options.glyph, "var(--type-subheading-icon)", "var(--title-colour, var(--color-on_surface))")] : [];
  const line = row("s8", "start", ...glyph, filling(title), ...(options.end ? [options.end] : []));
  line.classList.add("k-card-title-line");
  if (options.subtitle === undefined) return line;
  const subtitle = wrapped(options.subtitle, "small", "on_surface_variant");
  subtitle.classList.add("k-card-subtitle");
  subtitle.style.color = "var(--subtitle-colour, var(--color-on_surface_variant))";
  return stack("s4", "start", filling(line), filling(subtitle));
}

export interface CardOptions {
  readonly header?: HeaderOptions;
  /** What it holds between its header and its foot. */
  readonly body?: readonly Element[];
  /** Its button bar, at its foot: `lib/button-bar.ts`. A pressable card holds none. */
  readonly actions?: Element;
  /** Pressable: the whole card one press target, and what a press does. */
  readonly onPress?: () => void;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

/** A card's surface: flat, with the sunken edge, at the content radius, `space.s16` inside. */
function surface(className: string): HTMLElement {
  const element = document.createElement("span");
  element.className = `k-card k-sunken ${className}`.trim();
  return element;
}

/** What a card holds, one under another `space.s16` apart, dimmed together while it is disabled. */
function held(parts: readonly Element[]): HTMLElement {
  const content = stack("s16", "start", ...parts.map((part) => (part instanceof HTMLElement ? filling(part) : part)));
  content.classList.add("k-card-content");
  return content;
}

export function card(options: CardOptions): HTMLElement {
  const element = surface("");
  const parts = [...(options.header ? [cardHeader(options.header)] : []), ...(options.body ?? []), ...(options.actions ? [options.actions] : [])];
  element.append(held(parts));
  const disabled = options.disabled ?? false;
  if (disabled) element.setAttribute("aria-disabled", "true");
  if (options.onPress) {
    const press = options.onPress;
    element.classList.add("k-control");
    element.dataset.pressable = "";
    element.dataset.hot = "";
    element.setAttribute("role", "button");
    if (!disabled) element.tabIndex = 0;
    pin(element, options.pinned);
    focusRing(element, "content", "accent", false).dataset.pressable = "";
    element.addEventListener("click", () => {
      if (!disabled) press();
    });
    element.addEventListener("keydown", (event) => {
      if (disabled || (event.key !== "Enter" && event.key !== " ")) return;
      event.preventDefault();
      if (!event.repeat) press();
    });
  }
  return probe(element, { role: "component", name: "Card", variant: "", fixed: { width: false, height: false, holds: true } });
}

/** How a set of choice cards is picked: exactly one, as radios; or any of them, as checkboxes. */
export type Picks = "one" | "many";

/** A choice card's size: md, its symbol before its title; lg, on an lg icon tile above it. */
export type ChoiceCardSize = "Md" | "Lg";

/** A line at an lg card's foot: something the choice brings, by its symbol and name. */
export interface ChoiceLine {
  readonly glyph: string;
  readonly label: string;
}

export interface ChoiceCardOptions {
  readonly glyph?: string;
  /**
   * An lg card's foot, the room under its words: what picking it brings, a line each, the last
   * one's words alone (and 9 more). Lit with the card.
   */
  readonly foot?: readonly ChoiceLine[];
  /** Lg for a few choices that are a page's whole question: Make it yours's profiles. */
  readonly size?: ChoiceCardSize;
  readonly title: string;
  readonly subtitle?: string;
  readonly picked?: boolean;
  /** One of a set picked as radios (a disc), or any as checkboxes (a box). */
  readonly picks?: Picks;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

/** The mark at the title's end: a radio's disc or a checkbox's box, lit and pressed with the card. */
function mark(picks: Picks): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-check-box";
  box.dataset.shape = picks === "one" ? "disc" : "box";
  const holder = document.createElement("span");
  holder.className = "k-check-mark";
  if (picks === "one") {
    const dot = document.createElement("span");
    dot.className = "k-check-dot";
    holder.append(dot);
  } else {
    const tick = icon("check", "var(--icon-s16)", "var(--mark)");
    tick.classList.add("k-check-tick");
    holder.append(tick);
  }
  box.append(holder);
  const slot = document.createElement("span");
  slot.className = "k-check k-choice-mark";
  slot.dataset.tick = "Full";
  slot.append(box);
  return slot;
}

/** Shows whether `element`, a choice card, is picked. */
export function setPicked(element: HTMLElement, picked: boolean): void {
  const was = element.getAttribute("aria-checked") === "true";
  if (picked && !was && element.dataset.size === "Lg" && element.isConnected) strikeOn(element);
  element.setAttribute("aria-checked", String(picked));
  element.style.setProperty("--checked", picked ? "1" : "0");
  element.style.setProperty("--picked", picked ? "1" : "0");
}

export function choiceCard(options: ChoiceCardOptions): HTMLElement {
  const picks = options.picks ?? "one";
  const disabled = options.disabled ?? false;
  const element = surface("k-choice-card k-control");
  element.dataset.hot = "";
  element.setAttribute("role", picks === "one" ? "radio" : "checkbox");
  if (disabled) element.setAttribute("aria-disabled", "true");
  element.tabIndex = -1;
  pin(element, options.pinned);
  const size = options.size ?? "Md";
  element.dataset.size = size;
  if (size === "Lg") element.append(held(large(options, picks)));
  else {
    const header = cardHeader({
      title: options.title,
      end: mark(picks),
      ...(options.glyph ? { glyph: options.glyph } : {}),
      ...(options.subtitle === undefined ? {} : { subtitle: options.subtitle }),
    });
    element.append(held([header]));
  }
  focusRing(element, "content", "cold", false).dataset.pressable = "";
  setPicked(element, options.picked ?? false);
  return probe(element, { role: "component", name: "ChoiceCard", variant: size === "Lg" ? "Lg" : "", fixed: { width: false, height: false, holds: true } });
}

/**
 * An lg card lit by a pick strikes on as the neon does: its light, `--choice-burn`, steps through
 * `neon.strike` over `strike`; with reduced motion it simply comes on over `short`, as `--picked`
 * moves. Going out is `--picked` fading over `short`.
 */
function strikeOn(element: HTMLElement): void {
  const { reduced, speed, flicker } = settings();
  if (reduced) return;
  const steps = flicker ? strikeSteps() : [0, 1];
  element.animate(
    steps.map((value) => ({ "--choice-burn": value })),
    { duration: milliseconds(flicker ? "strike" : "short") / speed, easing: flicker ? `steps(${steps.length}, jump-none)` : "linear" },
  );
}

/**
 * An lg choice card's parts: its symbol on an lg icon tile with the mark at the line's end, then,
 * `space.s16` under them, the title in `heading` over the subtitle in `body`, `space.s4` apart.
 */
function large(options: ChoiceCardOptions, picks: Picks): HTMLElement[] {
  const tile = iconTile({ glyph: options.glyph ?? "check", size: "Lg" });
  tile.classList.add("k-choice-tile");
  // The tile and its symbol are lit with the card, so their colours are the card's to say.
  tile.style.setProperty("--tint", "var(--tile-tint)");
  tile.querySelector<SVGElement>(".k-icon")?.style.setProperty("color", "var(--tile-ink)");
  const lead = row(0, "start", tile, fill(), mark(picks));
  const title = wrapped(options.title, "heading");
  title.classList.add("k-card-title");
  title.style.color = "var(--title-colour, var(--color-on_surface))";
  const words: HTMLElement[] = [filling(title)];
  if (options.subtitle !== undefined) {
    const subtitle = wrapped(options.subtitle, "body", "on_surface_variant");
    subtitle.classList.add("k-card-subtitle");
    subtitle.style.color = "var(--subtitle-colour, var(--color-on_surface_variant))";
    words.push(filling(subtitle));
  }
  const named = stack("s4", "start", ...words);
  named.classList.add("k-choice-words");
  if (!options.foot?.length) return [filling(lead), filling(named)];
  const lines = options.foot.map((line) => row("s8", "centre", icon(line.glyph, "var(--type-small-icon)", "var(--foot-ink)"), text(line.label, "small", "on_surface_variant")));
  const foot = stack("s8", "start", ...lines);
  foot.classList.add("k-choice-foot");
  // The foot stands at the card's bottom: the room between is the layout's fill, which a check reads.
  return [filling(lead), filling(named), fill(), filling(foot)];
}

/** A key's step through a set of `columns` across: back or on by one, or by a row. */
function step(key: string, columns: number): number | null {
  switch (key) {
    case "ArrowLeft":
    case "h":
      return -1;
    case "ArrowRight":
    case "l":
      return 1;
    case "ArrowUp":
    case "k":
      return -columns;
    case "ArrowDown":
    case "j":
      return columns;
    default:
      return null;
  }
}

/** How a set's cells are picked and focused, where a cell isn't the card itself (a picture). */
export interface GroupOptions {
  /** The element in a cell the keyboard is on and that takes the press: the cell by default. */
  readonly target?: (cell: HTMLElement) => HTMLElement;
  /** Shows whether a cell is picked: a choice card's `setPicked` by default. */
  readonly set?: (cell: HTMLElement, picked: boolean) => void;
  /** Whether the arrows pick as they move in a set of one, as radios do: yes by default. */
  readonly arrowsPick?: boolean;
}

/**
 * `cells` as one set, `columns` across `space.s16` apart, one stop for Tab. Picking one of `one`
 * unpicks the rest, and the arrows pick as they move unless `arrowsPick` is off; of `many`, the
 * arrows move the keyboard and Space or Enter turns the cell it is on. A disabled cell is skipped
 * and takes no press.
 */
export function choiceGroup(picks: Picks, columns: number, cells: readonly HTMLElement[], options: GroupOptions = {}): HTMLElement {
  const target = options.target ?? ((cell: HTMLElement) => cell);
  const set = options.set ?? setPicked;
  const arrowsPick = picks === "one" && (options.arrowsPick ?? true);
  const grid = document.createElement("span");
  grid.className = "k-choice-group";
  grid.style.setProperty("--columns", String(columns));
  grid.setAttribute("role", picks === "one" ? "radiogroup" : "group");
  grid.append(...cells);
  // Equal shares in whole device pixels, the spare ones to the first columns, as the layout's
  // grid lays them: a fraction of the room each would put the cards' edges between pixels.
  new ResizeObserver(() => {
    const scale = Number.parseFloat(getComputedStyle(grid).getPropertyValue("--scale")) || settings().scale;
    const apart = Math.round(length("space-s16") * scale);
    const widths = split(Math.floor(grid.clientWidth + 1e-3), apart, Array.from({ length: columns }, () => 1));
    grid.style.setProperty("--tracks", widths.map((each) => `${each}px`).join(" "));
  }).observe(grid);
  const targets = cells.map(target);
  const usable = (at: number) => targets[at]!.getAttribute("aria-disabled") !== "true";
  const isPicked = (at: number) => cells[at]!.getAttribute("aria-checked") === "true";
  const indices = cells.map((_, at) => at);
  const stop = indices.find((at) => usable(at) && (picks === "many" || isPicked(at))) ?? indices.find(usable);
  if (stop !== undefined) targets[stop]!.tabIndex = 0;

  function choose(at: number): void {
    if (picks === "one") for (const other of indices) set(cells[other]!, other === at);
    else set(cells[at]!, !isPicked(at));
  }

  function focusOn(at: number): void {
    for (const other of indices) targets[other]!.tabIndex = other === at ? 0 : -1;
    targets[at]!.focus({ preventScroll: true });
  }

  for (const at of indices) {
    const element = targets[at]!;
    element.addEventListener("click", () => {
      if (!usable(at)) return;
      choose(at);
      for (const other of indices) targets[other]!.tabIndex = other === at ? 0 : -1;
    });
    element.addEventListener("keydown", (event) => {
      if (!usable(at)) return;
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        if (!event.repeat) choose(at);
        return;
      }
      const ends = event.key === "Home" ? 0 : event.key === "End" ? cells.length - 1 : null;
      const by = step(event.key, columns);
      if (ends === null && by === null) return;
      event.preventDefault();
      const toward = by ?? (ends === 0 ? 1 : -1);
      let next = ends ?? at + toward;
      while (next >= 0 && next < cells.length && !usable(next)) next += Math.sign(toward);
      if (next < 0 || next >= cells.length) return;
      focusOn(next);
      if (arrowsPick) choose(next);
    });
  }
  return probe(grid, { role: "grid", columns, gap: length("space-s16"), align: "start" });
}
