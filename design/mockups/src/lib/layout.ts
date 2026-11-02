// The layout primitives a mock is built from: `kubaos-ui`'s `layout` module and `grid`, and
// `KubaOS.Components`' `Page`, `Container`, `Header`, `Section`, `Stack`, `Row`, `Grid`, `Split`,
// `Flow`, `Fill`, `Gap` and `Divider` (`docs/design/components/layout.md`). Every gap, padding and
// fixed size is a token's name, so a mock built from them can't leave the spacing scale. Each child
// is its own box, already whole device pixels wide, so every child starts on one.

import "./layout.css";
import { type Align, probe } from "./probe.ts";
import { text, wrapped } from "./text.ts";
import { length, names, raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** A spacing token's name, `s8`, or 0 for none. */
export type Space = string | 0;

/** Children side by side, `gap` apart, lined up by `align`. */
export function row(gap: Space, align: Align, ...children: Element[]): HTMLElement {
  return layout("x", gap, align, children);
}

/** Children one under another, `gap` apart, lined up by `align`. */
export function stack(gap: Space, align: Align, ...children: Element[]): HTMLElement {
  return layout("y", gap, align, children);
}

function layout(axis: "x" | "y", gap: Space, align: Align, children: Element[]): HTMLElement {
  const element = document.createElement("span");
  element.className = axis === "x" ? "k-row" : "k-stack";
  element.dataset.align = align;
  element.dataset.layout = "";
  element.style.setProperty("--gap", spaceOf(gap));
  element.append(...children);
  return probe(element, { role: "layout", axis, gap: gap === 0 ? 0 : length(`space-${gap}`), align });
}

function spaceOf(gap: Space): string {
  return gap === 0 ? "0px" : `var(--space-${gap})`;
}

/** `element` taking the room left along its row, the whole width across its stack, or its cell's height in a grid. */
export function filling<E extends HTMLElement>(element: E): E {
  element.dataset.fill = "";
  return element;
}

/**
 * `element` taking the height its page or stack has left, shared with the others that do: iced's
 * `height(Extent::Fill)`. A column of lists that runs to the page's foot.
 */
export function fillingHeight<E extends HTMLElement>(element: E): E {
  element.dataset.fillHeight = "";
  return element;
}

/** Room that grows across a row, pushing what follows to its far end; it never grows down. */
export function fill(): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-fill";
  room.dataset.fill = "";
  return probe(room, { role: "spacer", fill: true });
}

/** Empty room of one spacing step both ways, where a stack needs one larger step than its gap. */
export function gap(space: string): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-gap";
  room.style.setProperty("--space", `var(--space-${space})`);
  return probe(room, { role: "spacer", fill: false });
}

/** A hairline across all the room it is given, in the soft outline: `kubaos-ui`'s `layout::divider`. */
export function divider(): HTMLElement {
  const line = document.createElement("span");
  line.className = "k-divider";
  return probe(line, { role: "component", name: "Divider", variant: "", fixed: { width: false, height: false } });
}

/**
 * `element` padded inside its top and bottom by `y` and its sides by `x`, spacing tokens' names:
 * a layout's `padding(Inset::xy(x, y))`.
 */
export function inset<E extends HTMLElement>(element: E, y: Space, x: Space = 0): E {
  element.style.padding = `${spaceOf(y)} ${spaceOf(x)}`;
  return element;
}

/** Children one under another in a room `width` wide (a length's CSS), as wide as it whatever they hold. */
export function within(width: string, ...children: Element[]): HTMLElement {
  const room = stack(0, "start", ...children);
  room.classList.add("k-within");
  room.style.setProperty("--room", width);
  return room;
}

/** The scale an element is drawn at: a CSS pixel in a mock is a device pixel, a logical one is `scale` of them. */
function scaleIn(element: Element): number {
  return Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale")) || settings().scale;
}

/**
 * Splits `width` device pixels less the gaps into `weights`' shares, in whole device pixels, the
 * spare ones to the first columns, so no two differ by more than one past their shares: `kubaos-ui`'s
 * `grid::weighted`.
 */
export function split(width: number, gap: number, weights: readonly number[]): number[] {
  const room = Math.max(0, width - gap * (weights.length - 1));
  const sum = weights.reduce((total, each) => total + each, 0) || 1;
  const widths = weights.map((each) => Math.floor((room * each) / sum));
  let spare = room - widths.reduce((total, each) => total + each, 0);
  for (let index = 0; spare > 0; index = (index + 1) % widths.length, spare--) widths[index]! += 1;
  return widths;
}

export interface GridOptions {
  /** Every row as tall as the tallest: Kōbō's four summary cards, one size. */
  readonly uniformRows?: boolean;
  /** All the height there is, shared between the rows: columns of lists that run to the page's foot. */
  readonly fillHeight?: boolean;
  /** Each row's cells centred down it, on whole device pixels: a label beside a taller value. */
  readonly centred?: boolean;
}

/** `columns` equal columns, `gap` apart both ways, filled row by row. */
export function grid(columns: number, gap: string, children: readonly Element[], options: GridOptions = {}): HTMLElement {
  return weighted(Array.from({ length: Math.max(columns, 1) }, () => 1), gap, children, options);
}

/** Columns as wide as their share of `weights`, `gap` apart: `[1, 3]` is a label column a quarter wide. */
export function splitColumns(weights: readonly number[], gap: string, children: readonly Element[], options: GridOptions = {}): HTMLElement {
  return weighted(weights.length > 0 ? weights : [1], gap, children, options);
}

/**
 * A grid of `weights`' columns. Each column's width, and each row's height where the grid fills
 * its height, is worked out in whole device pixels as `kubaos-ui`'s `Grid` lays them out; a row is
 * as tall as its tallest cell, and a cell that fills (`filling`) is stretched to it, so cards side by
 * side end together.
 */
function weighted(weights: readonly number[], gap: string, children: readonly Element[], options: GridOptions): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-grid";
  box.style.setProperty("--gap", spaceOf(gap));
  box.toggleAttribute("data-uniform", options.uniformRows ?? false);
  if (options.fillHeight) fillingHeight(box);
  box.append(...children);
  const count = weights.length;
  const arrange = () => {
    const scale = scaleIn(box);
    const apart = Math.round(length(`space-${gap}`) * scale);
    box.style.setProperty("--columns", split(Math.floor(box.clientWidth + 1e-3), apart, weights).map((each) => `${each}px`).join(" "));
    const rows = Math.ceil(box.children.length / count);
    if (options.fillHeight && rows > 0) {
      // Each row's share of the height, on a whole device pixel, as iced's `Scale::snap` puts it.
      const share = Math.max(0, Math.round((box.clientHeight - apart * (rows - 1)) / rows));
      box.style.setProperty("--rows", `${share}px`);
    }
    if (options.centred) centreRows(box, count);
  };
  new ResizeObserver(arrange).observe(box);
  return probe(box, { role: "grid", columns: count, gap: length(`space-${gap}`), align: options.centred ? "centre" : "start" });
}

/** Each row's cells centred down it, their offsets rounded to whole device pixels. */
function centreRows(box: HTMLElement, count: number): void {
  const cells = [...box.children] as HTMLElement[];
  for (const cell of cells) cell.style.marginTop = "0px";
  for (let first = 0; first < cells.length; first += count) {
    const line = cells.slice(first, first + count);
    const height = Math.max(...line.map((cell) => cell.offsetHeight));
    for (const cell of line) cell.style.marginTop = `${Math.round((height - cell.offsetHeight) / 2)}px`;
  }
}

/**
 * Children side by side, `gap` apart both ways, a new line started where the next doesn't fit:
 * chips, tags, checkboxes. Each child is no wider than the room. Its lines tell the geometry check
 * nothing, as iced's wrapping row's don't; each child still says what it is.
 */
export function flow(gap: string, ...children: Element[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-flow";
  box.style.setProperty("--gap", spaceOf(gap));
  box.append(...children);
  return box;
}

/** The container's steps: from which width, in logical pixels, each padding applies, by its spacing token. */
export function containerSteps(): { readonly from: number; readonly space: string }[] {
  const steps: { from: number; space: string }[] = [];
  for (let i = 0; raw(`container-${i}-from`) !== ""; i++) {
    const padding = length(`container-${i}-padding`);
    const space = names("space-").find((name) => length(name) === padding)?.slice("space-".length);
    if (space === undefined) throw new Error(`the container's padding ${padding} is no spacing step`);
    steps.push({ from: Number(raw(`container-${i}-from`)), space });
  }
  return steps;
}

/** The padding at the sides of a page or a container `width` logical pixels wide: Tailwind's container. */
export function containerPadding(width: number): string {
  const steps = containerSteps();
  return (steps.filter((step) => width >= step.from).at(-1) ?? steps[0]!).space;
}

/** `box` padded at its sides by the container's step for its own width, kept as it resizes. */
function padByWidth(box: HTMLElement): void {
  const pad = () => {
    const space = containerPadding(box.clientWidth / scaleIn(box));
    box.style.setProperty("--padding", `var(--space-${space})`);
  };
  box.style.setProperty("--padding", `var(--space-${containerSteps()[0]!.space})`);
  new ResizeObserver(pad).observe(box);
}

export interface PageOptions {
  /** Nothing behind it, over a surface of its own: Kōbō's glass panel, a pane beside a rail (`layout::page_clear`). */
  readonly clear?: boolean;
}

/**
 * A whole page: the surface, filling the room both ways, with `content` the container padding in
 * at the sides, `space.s32` below its top and `space.s24` above its foot, where a footer's buttons
 * sit.
 */
export function page(content: HTMLElement, options: PageOptions = {}): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-page";
  box.toggleAttribute("data-clear", options.clear ?? false);
  box.append(content);
  padByWidth(box);
  return probe(box, { role: "component", name: "Page", variant: "", fixed: { width: false, height: false, holds: true } });
}

/**
 * A panel's body or a toolbar band: `content` the container padding in at the sides, as wide as the
 * room and as tall as the content, which keeps its own spacing above and below.
 */
export function container(content: HTMLElement): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-container";
  box.append(content);
  padByWidth(box);
  return probe(box, { role: "component", name: "Container", variant: "", fixed: { width: false, height: false, holds: true } });
}

export interface HeaderOptions {
  /** The page's Japanese accent, over the title in `primary`. */
  readonly accent: string;
  readonly title: string;
  /**
   * The line under the title, `space.s8` below. A surface decides once whether its headers have
   * one: given, even empty, the line keeps its height, so nothing under it moves when its words
   * change; left out, there is no line and no room for one.
   */
  readonly subtitle?: string;
  /** Buttons beside the header, centred on its title and subtitle. */
  readonly actions?: readonly Element[];
}

/** The title and the line under it, one block. */
function titled(options: HeaderOptions): HTMLElement[] {
  const title = wrapped(options.title, "title");
  if (options.subtitle === undefined) return [title];
  const line = options.subtitle === "" ? text("", "body", "on_surface_variant") : wrapped(options.subtitle, "body", "on_surface_variant");
  return [title, gap("s8"), line];
}

/** A page's header: the Japanese accent, the title, and the line under it, `space.s8` below. */
export function header(options: HeaderOptions): HTMLElement {
  const accent = text(options.accent, "accent", "primary");
  const block = options.actions?.length
    ? row("s16", "centre", filling(stack(0, "start", ...titled(options))), row("s8", "centre", ...options.actions))
    : null;
  return componentBox("k-header", block ? stack(0, "start", accent, filling(block)) : stack(0, "start", accent, ...titled(options)), "Header", false);
}

/** A component that is its stack: a header's or a section's box, as wide as its room. */
function componentBox(className: string, content: HTMLElement, name: string, holds: boolean): HTMLElement {
  const box = document.createElement("span");
  box.className = `k-block ${className}`;
  box.append(filling(content));
  return probe(box, { role: "component", name, variant: "", fixed: holds ? { width: false, height: false, holds: true } : { width: false, height: false } });
}

/** A titled part of a page: its heading, an optional line `space.s4` under it, the content `space.s16` below. */
export function section(heading: string, body: string | undefined, ...content: Element[]): HTMLElement {
  const lines = [wrapped(heading, "heading"), ...(body === undefined ? [] : [wrapped(body, "body", "on_surface_variant")])];
  const view = stack("s16", "start", filling(stack("s4", "start", ...lines)), ...content.map((part) => (part instanceof HTMLElement ? filling(part) : part)));
  return componentBox("k-section", view, "Section", true);
}
