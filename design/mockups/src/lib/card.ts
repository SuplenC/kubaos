// A card: `kubaos-ui`'s `card::card` and `card::choice_card`, `KubaOS.Components`' `Card` and
// `ChoiceCard` (`docs/design/components/card.md`). A surface inside the page, raised one step on the
// ladder (`lib/elevation.ts`), at the surface radius, `space.s16` inside: what it holds is a header,
// a body and a button bar at its foot, `space.s16` apart.
//
// A plain card is a surface and takes no input. A pressable one is a control in the state model
// (`.k-control` in `lib.css`): the whole card is one press target, lit with the accent's tube
// inside its edge. A choice card is one of a set to pick from, its tube cold since the accent
// already means picked; picked, its title and symbol are the accent's words (`primary_lit`) and its
// mark (a radio's disc or a checkbox's box) is filled. Disabled, either keeps its resting surface
// and dims what it holds.

import { pin, type Pinned } from "./control.ts";
import { raise } from "./elevation.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { wrapped } from "./text.ts";
import { length } from "./tokens.ts";

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

/** A card's surface: raised one step, at the surface radius, `space.s16` inside. */
function surface(className: string): HTMLElement {
  const element = raise(document.createElement("span"), 1);
  element.className = `${element.className} k-card ${className}`.trim();
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
    focusRing(element, "surface", "accent", false).dataset.pressable = "";
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

export interface ChoiceCardOptions {
  readonly glyph?: string;
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
  const header = cardHeader({
    title: options.title,
    end: mark(picks),
    ...(options.glyph ? { glyph: options.glyph } : {}),
    ...(options.subtitle === undefined ? {} : { subtitle: options.subtitle }),
  });
  element.append(held([header]));
  focusRing(element, "surface", "cold", false).dataset.pressable = "";
  setPicked(element, options.picked ?? false);
  return probe(element, { role: "component", name: "ChoiceCard", variant: "", fixed: { width: false, height: false, holds: true } });
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
    targets[at]!.focus();
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
