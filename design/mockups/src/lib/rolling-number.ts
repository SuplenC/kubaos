// A count that rolls as it changes, as an odometer does: `RollingNumber`, new, in neither library
// yet. Its figures are tabular, each set in the middle of a slot one figure wide and the style's
// line tall that cuts what leaves it (a figure's advance rounded up to a whole device pixel, so each starts on one),
// and the box is as many slots wide as the widest value it is given, its figures held to its end, so nothing beside it moves as the count grows a figure.
//
// Only the figures that change roll. Going up, the new one comes in from below as the old one leaves
// upward; going down, from above as the old leaves downward: `--roll` 0 to 1 over `short` on
// `standard`, the pair moving together. A figure the count grows comes in from nothing, and one it
// sheds leaves into nothing. The box is as wide as its figures: when a roll changes how many there
// are, it eases from the old figures' width to the new over the roll's own `short` on `standard`,
// starting with it, so an owner that centres it moves it smoothly and it stands centred as the roll
// ends. A shed figure rolls out beside the figures that stay, outside the box's width but drawn,
// and goes as its roll ends. With reduce motion the figures and the width change in place.
//
// A number can be written with marks between its figures (`format`): a clock's colons, a minus.
// A mark sits in a slot as wide as its own advance, rounded up to a whole device pixel, and never
// rolls; the box is as wide as the widest value's figures and marks, so a clock that grows an hour
// keeps its place.
//
// Its figures are the digits unless it is given others (`figures`): a kanji numeral's, one glyph to
// a slot as wide as the widest of them, rolling as a digit does, 十 for ten.
//
// Its figures are centred for every figure's mass in their own style, unless an owner sets them on
// a baseline of its own (`centre`): the bar's workspaces centre each number for one figure in one
// style, whatever style a number is drawn in, so the pill's numbers share one baseline.

import "./rolling-number.css";
import { centreOnPixels } from "./centre.ts";
import { FIXED_HEIGHT, probe } from "./probe.ts";
import { advance, centreFor, tabularFigure, text, type TypeStyle, widestGlyph } from "./text.ts";
import { durationMs, raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

/** Every figure, so each one is centred for the same mass and they share one baseline. */
const FIGURES = "0123456789";

export interface RollingOptions {
  readonly value: number;
  /** The widest value it shows, its figures all 0: "000" for a count up to 999, "0:00:00" for a clock. */
  readonly widest: string;
  /** How a value is written, figures and marks: a clock's `1:02:10`. The locale's count unless given. */
  readonly format?: (value: number) => string;
  readonly style?: TypeStyle;
  readonly color?: string;
  /**
   * The glyphs that roll, one to a slot, when they are not the digits: a kanji numeral's, 一 to 十.
   * Each slot is as wide as the widest of them; anything else is a mark.
   */
  readonly figures?: string;
  /** The words and style every figure is centred for, when not every figure in its own style. */
  readonly centre?: { readonly style: TypeStyle; readonly words: string };
}

export interface Rolling {
  readonly element: HTMLElement;
  /** Rolls to `value`: the figures that differ roll, up or down by which way it went. */
  readonly set: (value: number) => void;
  readonly value: () => number;
}

/** The locale's figures for `value`: a count is never padded by hand. */
const COUNT = new Intl.NumberFormat("en-GB", { useGrouping: false });

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return durationMs(name) / settings().speed;
}

/** What a figure is centred for: `words` in `style`. */
interface Centre {
  readonly style: TypeStyle;
  readonly words: string;
}

/** One figure in its slot's ink, centred for `centre`'s mass, every figure's in its own style without one. */
function figure(char: string, style: TypeStyle, color: string, centre?: Centre, figures = FIGURES): HTMLElement {
  const made = text(char, style, color);
  made.classList.add("k-roll-figure");
  centreFor(made, centre?.style ?? style, centre?.words ?? figures);
  return made;
}

/** A mark in its slot's ink: as it is, on the figures' baseline. */
function markText(char: string, style: TypeStyle, color: string): HTMLElement {
  const made = text(char, style, color);
  made.classList.add("k-roll-figure");
  return made;
}

/** Whether `char` is a mark, written between figures and never rolled: a clock's colon. */
function isMark(char: string, figures = FIGURES): boolean {
  return char !== "" && char !== " " && !figures.includes(char);
}

/** Makes `box` a figure's slot, or a mark's as wide as `char`'s own advance. */
function shape(box: HTMLElement, char: string, style: TypeStyle, figures = FIGURES): void {
  const mark = isMark(char, figures);
  box.toggleAttribute("data-mark", mark);
  box.style.setProperty("--mark", mark ? String(advance(style, char)) : "");
}

/** A figure's room in `style`: a tabular digit's, or the widest of `figures` when they are not digits. */
function figureWidth(style: TypeStyle, figures: string): number {
  return figures === FIGURES ? tabularFigure(style) : widestGlyph(style, figures);
}

/** A slot one figure wide, holding `char`, or nothing for a figure the count hasn't got; a mark's as wide as it. */
function slot(char: string, style: TypeStyle, color: string, centre?: Centre, figures = FIGURES): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-roll-slot";
  shape(box, char, style, figures);
  if (char.trim()) box.append(isMark(char, figures) ? markText(char, style, color) : figure(char, style, color, centre, figures));
  return box;
}

/**
 * The figures' slots of a rolling number, in order: its own children of that class alone, since an
 * owner may hang a hint or anything else inside its box.
 */
function slotsOf(element: HTMLElement): HTMLElement[] {
  return [...element.querySelectorAll<HTMLElement>(":scope > .k-roll-slot")];
}

export function rollingNumber(options: RollingOptions): Rolling {
  const style = options.style ?? "mono_large";
  const color = options.color ?? "on_surface";
  const centre = options.centre;
  const figures = options.figures ?? FIGURES;
  const element = document.createElement("span");
  element.className = "k-rolling";
  element.setAttribute("role", "status");
  const format = options.format ?? ((value: number) => COUNT.format(value));
  const widest = [...options.widest];
  element.style.setProperty("--slots", String(widest.filter((char) => !isMark(char, figures)).length));
  element.style.setProperty("--marks", widest.filter((char) => isMark(char, figures)).map((char) => `round(up, calc(${advance(style, char)} * var(--px) - 0.001px), 1px)`).join(" + ") || "0px");
  element.style.setProperty("--figure", String(figureWidth(style, figures)));
  element.style.setProperty("--t-line", `var(--type-${style}-line)`);
  let value = options.value;
  let shown = format(value);
  element.append(...[...shown].map((char) => slot(char, style, color, centre, figures)));
  element.setAttribute("aria-label", shown);
  let resizing: Animation | null = null;
  // Its widest value's width, the least it is ever drawn at, as `.k-rolling`'s `min-width` has it.
  const least = widest.reduce((sum, char) => sum + roomOf(char), 0);

  /** A slot's width, as the CSS rounds it: a figure's or a mark's advance at the box's scale, up to a whole pixel. */
  function roomOf(char: string): number {
    const scale = (element.isConnected && Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale"))) || settings().scale;
    return Math.ceil((isMark(char, figures) ? advance(style, char) : figureWidth(style, figures)) * scale - 0.001);
  }

  /** What `boxes` take side by side, the box never under its widest value. */
  function across(boxes: readonly HTMLElement[]): number {
    return Math.max(least, boxes.reduce((sum, box) => sum + roomOf(box.hasAttribute("data-mark") ? box.textContent ?? "" : figures[0]!), 0));
  }

  function set(next: number): void {
    if (next === value) return;
    // The width it is drawn at now, mid-ease or at rest, before anything about it changes; a box
    // not yet on the page, which an owner may roll as it builds it, has its slots' width.
    const drawn = element.isConnected ? Number.parseFloat(getComputedStyle(element).width) : Number.NaN;
    delete element.dataset.resizing;
    const before = Number.isFinite(drawn) ? drawn : across(slotsOf(element).filter((box) => !box.hasAttribute("data-shed")));
    resizing?.cancel();
    // A figure still rolling out from the last change goes now: the slots line up with the value again.
    for (const shed of element.querySelectorAll(":scope > .k-roll-slot[data-shed]")) shed.remove();
    const way = next > value ? 1 : -1;
    const words = format(next);
    const width = Math.max(words.length, shown.length);
    const [was, now] = [shown.padStart(width, " "), words.padStart(width, " ")];
    while (slotsOf(element).length < width) element.prepend(slot("", style, color, centre, figures));
    const slots = slotsOf(element);
    const reduced = settings().reduced;
    slots.forEach((box, index) => {
      if (was[index] === now[index]) return;
      const char = now[index]!.trim();
      for (const left of box.querySelectorAll("[data-out]")) left.remove();
      // A mark changes in place, never rolling: a slot that turns from a figure to a mark or back
      // takes its new width at once.
      if (isMark(char, figures) || isMark(was[index]!, figures) || box.hasAttribute("data-mark")) {
        shape(box, char, style, figures);
        box.replaceChildren(...(char ? [isMark(char, figures) ? markText(char, style, color) : figure(char, style, color, centre, figures)] : []));
        return;
      }
      const old = box.querySelector<HTMLElement>(".k-roll-figure");
      const fresh = char ? figure(char, style, color, centre, figures) : null;
      if (reduced) {
        old?.remove();
        if (fresh) box.append(fresh);
        else box.remove();
        return;
      }
      box.style.setProperty("--way", String(way));
      if (old) old.dataset.out = "";
      if (fresh) {
        fresh.dataset.in = "";
        box.append(fresh);
      }
      if (!fresh) box.dataset.shed = "";
      const roll = box.animate([{ "--roll": 0 }, { "--roll": 1 }], { duration: ms("short"), easing: raw("easing-standard") });
      // A slot the count shed goes once its figure has rolled away, on the roll's own end.
      roll.finished.then(() => {
        old?.remove();
        if (fresh) delete fresh.dataset.in;
        else if (!box.querySelector(".k-roll-figure:not([data-out])")) box.remove();
      }, () => {});
    });
    // A shed slot stands just before the figures that stay, out of the box's flow, so the box's own
    // width is already the new figures' and nothing moves when the slot goes.
    const staying = slotsOf(element).filter((box) => !box.hasAttribute("data-shed"));
    const after = across(staying);
    let right = staying.reduce((sum, box) => sum + roomOf(box.hasAttribute("data-mark") ? box.textContent ?? "" : figures[0]!), 0);
    for (const box of slotsOf(element).filter((each) => each.hasAttribute("data-shed")).reverse()) {
      box.style.right = `${right}px`;
      right += roomOf(figures[0]!);
    }
    if (!reduced && Math.abs(after - before) >= 0.5) {
      element.dataset.resizing = "";
      const going = element.animate([{ "--roll-width": `${before}px` }, { "--roll-width": `${after}px` }], { duration: ms("short"), easing: raw("easing-standard") });
      resizing = going;
      // A box its owner centres lands on whole device pixels at rest, the odd one after it
      // (`lib/centre.ts`); while it eases its width it is put there every frame too, so the ease
      // ends where the box rests and nothing steps by half a pixel as it lands.
      const centre = () => element.isConnected && element.parentElement && centreOnPixels(element.parentElement);
      const follow = () => {
        if (resizing !== going) return;
        centre();
        if (going.playState !== "finished") requestAnimationFrame(follow);
      };
      requestAnimationFrame(follow);
      going.finished.then(() => {
        if (resizing !== going) return;
        delete element.dataset.resizing;
        centre();
      }, () => {});
    }
    value = next;
    shown = words;
    element.setAttribute("aria-label", shown);
  }

  probe(element, { role: "component", name: "RollingNumber", variant: style, fixed: FIXED_HEIGHT });
  // An owner centres it by layout, which can leave it half a device pixel off when its owner is
  // laid out after the page put its boxes on pixels (the Overview places its workspaces a frame
  // later): each time it is laid out at a size, in that same frame, it is put on pixels too.
  new ResizeObserver(() => {
    if (element.isConnected && element.parentElement) centreOnPixels(element.parentElement);
  }).observe(element);
  return { element, set, value: () => value };
}

/**
 * A rolling number caught as it starts to roll from `from` to `to`, its leaving and arriving figures
 * both in their slots at `--roll` 0: what a move's player rolls.
 */
export function rollingMidway(from: number, to: number, widest: string, style: TypeStyle = "mono_large"): HTMLElement {
  const made = rollingNumber({ value: from, widest, style });
  const way = to > from ? 1 : -1;
  const [a, b] = [COUNT.format(from), COUNT.format(to)];
  const width = Math.max(a.length, b.length);
  const [was, now] = [a.padStart(width, " "), b.padStart(width, " ")];
  while (slotsOf(made.element).length < width) made.element.prepend(slot("", style, "on_surface"));
  slotsOf(made.element).forEach((box, index) => {
    if (was[index] === now[index]) return;
    box.style.setProperty("--way", String(way));
    box.querySelector<HTMLElement>(".k-roll-figure")?.setAttribute("data-out", "");
    const char = now[index]!.trim();
    if (char) {
      const fresh = figure(char, style, "on_surface");
      fresh.dataset.in = "";
      box.append(fresh);
    }
  });
  return made.element;
}
