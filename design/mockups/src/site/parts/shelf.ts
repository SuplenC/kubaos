// A page's samples, laid out by their size. Samples as small as a component sit in a wrapping grid
// of cards, each the live sample over its name and words, so they compare side by side; a sample as
// big as a window goes on its section's one stage, where a switcher picks the sample shown, at its
// real size, its name and words beside it or under it.
// `?all` draws every stage's samples one under another instead, for the shots and the checks.
//
// Which of the two a sample takes is decided here alone (`windowSized`), from the samples as drawn.

import { applies, type DoesntApply } from "../page.ts";
import { el, label, prose, relayout, sampleBox } from "../dom.ts";
import { ALL } from "../settings.ts";
import { picker } from "./picker.ts";
import { notApplying, subheading } from "./section.ts";

/** One thing a page shows: its name, what it is in words, and it, drawn. */
export interface Item {
  readonly name: string;
  readonly words?: string;
  readonly sample: Element;
  /**
   * Its place on a stage's switcher, a choice per row of chips, broadest first (a mode, a row, a
   * state); its name alone when it has none.
   */
  readonly at?: readonly string[];
  /**
   * It fills the room it has, as a foundation's table of bands or grid of symbols does: it is drawn
   * across the stage, as wide as the page.
   */
  readonly fluid?: boolean;
  /** What it stands in on a stage: a state's cell in its mode's painted box. */
  readonly frame?: (sample: Element) => Element;
  /** What reads it, after its words: the measures' parts tree and spec list. */
  readonly aside?: Element;
}

/** Samples shown together under a sub-heading. */
export interface Group {
  readonly title: string;
  /** Its name on a stage's switcher, where its title is left to the section's heading. */
  readonly chip?: string;
  readonly items: readonly Item[] | DoesntApply;
  /** How the group is drawn when its samples are small, from its items: cards when it has none. */
  readonly small?: (items: readonly Item[]) => Element;
}

/**
 * Whether `item`, `width` wide in the page's CSS pixels, is as big as a window: wider than a text
 * column, it would leave no room beside it, and a grid of such could hold only one a row. A whole
 * screen or template (`[data-whole]`, a shell panel on its screen among them) is one whatever its
 * width, and one that fills whatever room it has is drawn as wide as the page, so it is one too.
 */
function windowSized(item: Item, width: number, column: number): boolean {
  const whole = item.sample.matches("[data-whole]") || item.sample.querySelector("[data-whole]") !== null;
  return width > column || whole || (item.fluid ?? false);
}

/**
 * Each group's samples' widths as drawn at their own, in the page's CSS pixels, and a column's
 * width: the samples are laid out once, together, in a box of no width of its own on the page.
 */
function measure(groups: readonly (readonly Item[])[]): { widths: number[][]; column: number } {
  const page = document.getElementById("page") ?? document.body;
  const host = el("div", "page-measure");
  const column = el("div", "page-measure-column");
  const boxes = groups.map((items) => items.map((item) => el("div", "page-measure-item", item.sample)));
  host.append(column, ...boxes.flat());
  page.append(host);
  // The page is zoomed so its CSS pixels are device pixels: a box is read back in them.
  const zoom = page.offsetWidth / page.getBoundingClientRect().width || 1;
  const widths = boxes.map((list) => list.map((box) => box.getBoundingClientRect().width * zoom));
  const columnWidth = column.getBoundingClientRect().width * zoom;
  host.remove();
  return { widths, column: columnWidth };
}

/** A group's items by where each goes, and the widest of the cards'. */
interface Placed {
  readonly big: readonly Item[];
  readonly small: readonly Item[];
  readonly widest: number;
}

/**
 * `group`'s items by where each goes: cards hold the small and a stage the big. A group with a
 * layout of its own (`small`, a state matrix's grid) reads across its items, so it stays whole, on
 * the stage if any of them is big.
 */
function placed(group: Group & { items: readonly Item[] }, widths: readonly number[], column: number): Placed {
  const isBig = group.items.map((item, i) => windowSized(item, widths[i]!, column));
  if (group.small) return isBig.includes(true) ? { big: group.items, small: [], widest: 0 } : { big: [], small: group.items, widest: 0 };
  const widest = Math.max(0, ...widths.filter((_, i) => !isBig[i]));
  return { big: group.items.filter((_, i) => isBig[i]), small: group.items.filter((_, i) => !isBig[i]), widest };
}

/**
 * The parts of a section: each group under its sub-heading as cards, and its samples as big as a
 * window on one stage at the first such group's place; `between` parts stay where they are listed.
 */
export function shelves(...parts: readonly (Group | Element)[]): Element[] {
  const groups = parts.filter((part): part is Group => !(part instanceof Element));
  const drawn = groups.filter((group): group is Group & { items: readonly Item[] } => applies(group.items) && group.items.length > 0);
  const { widths, column } = measure(drawn.map((group) => group.items));
  const parted = new Map<Group, Placed>(drawn.map((group, i) => [group, placed(group, widths[i]!, column)]));
  const staged = drawn.filter((group) => parted.get(group)!.big.length > 0);
  const out: Element[] = [];
  for (const part of parts) {
    if (part instanceof Element) {
      out.push(part);
      continue;
    }
    if (!applies(part.items) || part.items.length === 0) {
      out.push(...(applies(part.items) ? titled(part.title) : [notApplying(part.items.doesntApply, part.title)]));
      continue;
    }
    const { big, small, widest } = parted.get(part)!;
    if (small.length > 0) out.push(...titled(part.title), part.small ? part.small(small) : cards(small, widest));
    if (big.length === 0 || part !== staged[0]) continue;
    // One group keeps its heading over the stage; several are its switcher's first row.
    if (staged.length === 1) out.push(...(small.length > 0 ? [] : titled(part.title)), stage(big));
    else out.push(stage(staged.flatMap((group) => parted.get(group)!.big.map((item) => ({ ...item, at: [group.chip ?? group.title, ...(item.at ?? [item.name])] })))));
  }
  return out;
}

function titled(title: string): HTMLElement[] {
  return title ? [subheading(title)] : [];
}

/** Small samples in a grid of cards, as many a row as the room holds at the widest's width. */
export function cards(items: readonly Item[], widest: number): HTMLElement {
  const grid = el(
    "div",
    "page-cards",
    ...items.map((item) =>
      el(
        "div",
        "page-card",
        sampleBox(el("div", "page-card-sample", item.sample)),
        el("div", "page-card-words", label(item.name, "label", "on_surface"), ...(item.words ? [prose(item.words, "body", "on_surface_variant")] : []), ...(item.aside ? [item.aside] : [])),
      ),
    ),
  );
  grid.style.setProperty("--widest", `${Math.ceil(widest)}px`);
  return grid;
}

/**
 * Small samples with their parts trees: cards across the page's whole width, as many a row as hold
 * a column of text (`site/settle.ts`), so a tree has room beside its spec list.
 */
export function inspected(items: readonly Item[]): HTMLElement {
  const grid = cards(items, 0);
  grid.dataset.filled = String(items.length);
  return grid;
}

/**
 * An item on a stage: it at its real size in a box of its own, its name and words beside or under
 * it; one that fills its room, across the stage, under its words.
 */
function shown(item: Item): HTMLElement {
  const body = el(
    "div",
    "page-stage-item",
    sampleBox(el("div", "page-view", item.frame ? item.frame(item.sample) : item.sample)),
    el("div", "page-stage-words", label(item.name, "body_strong", "on_surface"), ...(item.words ? [prose(item.words)] : []), ...(item.aside ? [item.aside] : [])),
  );
  body.toggleAttribute("data-fluid", item.fluid ?? false);
  return body;
}

/**
 * Samples as big as a window on one stage, a row of chips per level of their places picking the
 * one shown; with `?all`, every one of them, one under another.
 */
export function stage(items: readonly Item[]): HTMLElement {
  const bodies = items.map(shown);
  const box = el("div", "page-stage");
  if (ALL) {
    box.append(...bodies);
    return box;
  }
  const places = items.map((item) => item.at ?? [item.name]);
  const depth = Math.max(...places.map((place) => place.length));
  const picked: string[] = places[0]!.slice();
  const switcher = el("div", "page-stage-switch");
  const rows: HTMLElement[] = [];

  /** The choices at `level` among the items whose places start as the picks above it. */
  const choicesAt = (level: number): string[] => [
    ...new Set(places.filter((place) => picked.slice(0, level).every((value, i) => place[i] === value) && place[level] !== undefined).map((place) => place[level]!)),
  ];

  /** Draws the rows of chips from `level` down, each keeping its pick where it still can. */
  const drawFrom = (level: number): void => {
    for (let at = level; at < depth; at++) {
      const choices = choicesAt(at);
      if (!choices.includes(picked[at]!)) picked[at] = choices[0]!;
      const row = picker(
        choices,
        choices.indexOf(picked[at]!),
        (index) => {
          picked[at] = choices[index]!;
          drawFrom(at + 1);
          show();
        },
        "Which sample the stage shows",
      ).element;
      // A level with one choice says nothing: its row is left out.
      row.hidden = choices.length < 2;
      rows[at]?.replaceWith(row);
      if (!rows[at]) switcher.append(row);
      rows[at] = row;
    }
    picked.length = depth;
  };

  /** Shows the item the picks name, and lays the page out again for it. */
  const show = (): void => {
    const at = places.findIndex((place) => place.every((value, i) => picked[i] === value));
    bodies.forEach((body, i) => {
      body.hidden = i !== at;
    });
    if (box.isConnected) relayout(box);
  };

  drawFrom(0);
  show();
  box.append(switcher, ...bodies);
  return box;
}
