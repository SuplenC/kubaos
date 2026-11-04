// One sample of a page's measures under inspection: the sample in its box with a drawing layer
// over it, its parts tree from the `[data-probe]` nesting, and the spec list of the part inspected.
// The layer draws one thing at a time, as `measures.ts` says: a part (its box, its size tag, its
// padding and its gaps) or every place one token is used. Nothing is drawn at rest.

import { length } from "../../lib/tokens.ts";
import { el } from "../dom.ts";
import { settings } from "../settings.ts";
import { type Kind, reader, type Side, type Use, tokensFor } from "./uses.ts";

/** What the layer draws: a part of one sample, or a token across a section's samples. */
export type Lit = { readonly part: Element; readonly in: Inspector } | { readonly token: string };

export interface Inspector {
  /** The sample and the layer over it. */
  readonly holder: HTMLElement;
  /** The parts tree and the spec list, beside the sample or under it. */
  readonly aside: HTMLElement;
  /** Draws `lit`, or clears the layer for null; the places a token was drawn in, else 0. */
  draw(lit: Lit | null, pinned: boolean): number;
  /** Whether the sample uses `token` anywhere, shown or not: on a stage's other sample, scrolled out. */
  uses(token: string): boolean;
  /** The sample's name, which says where a token not on show is used. */
  readonly name: string;
}

/** What the inspector tells its section: the reader pointed at a part, left it, or clicked one. */
export interface Hands {
  point(part: Element | null, from: Inspector): void;
  click(part: Element, from: Inspector): void;
  /** The tokens the section's table lists, which every part's spec tries. */
  readonly tokens: readonly string[];
}

const SIDES: readonly Side[] = ["top", "right", "bottom", "left"];

interface Probe {
  readonly role?: string;
  readonly name?: string;
  readonly variant?: string;
  readonly axis?: string;
  readonly fill?: boolean;
}

function probeOf(element: Element): Probe {
  return JSON.parse((element as HTMLElement).dataset.probe ?? "{}") as Probe;
}

/** `space-s12` as the tables write it, `space.s12`. */
export function tokenName(token: string): string {
  return token.replaceAll("-", ".");
}

const PLACES = new Intl.PluralRules();

const LIST = new Intl.ListFormat(undefined, { type: "conjunction" });

/**
 * A lit token and how many places it was drawn in, a count's own words for each count; with none
 * drawn, the samples that use it out of view, or that it is a rule.
 */
export function litWords(token: string, count: number, unseen: readonly string[]): string {
  const name = tokenName(token);
  if (count > 0) return PLACES.select(count) === "one" ? `${name} · ${count} place` : `${name} · ${count} places`;
  if (unseen.length > 0) return `${name} · none in view: used in ${LIST.format(unseen)}`;
  return `${name} · no place: a rule, not a box`;
}

const HINT = "Point at a part to read its measures; click to pin it, Esc to let go.";

function round(value: number): string {
  return String(Math.round(value * 100) / 100);
}

/** A part's name in the tree and the spec: its role, and its name and variant where it has them. */
function partName(element: Element): string {
  const probe = probeOf(element);
  switch (probe.role) {
    case "component":
      return probe.variant ? `${probe.name} · ${probe.variant}` : (probe.name ?? "Component");
    case "layout":
      return probe.axis === "x" ? "Row" : "Stack";
    case "grid":
      return "Grid";
    case "icon":
      return `Icon · ${probe.name}`;
    case "spacer":
      return probe.fill ? "Fill" : "Gap";
    case "text":
      return "Text";
    default:
      return "Box";
  }
}

/** A part's shape, words and symbols left out: two rows of a list differ only in what they say. */
function signature(element: Element): string {
  const probe = probeOf(element);
  const name = probe.role === "component" ? `${probe.name}:${probe.variant}` : "";
  return `${probe.role}:${name}:${probe.axis ?? ""}(${probedChildren(element).map(signature).join(",")})`;
}

/** The probed parts directly inside `element`: its probed descendants with no probed box between. */
function probedChildren(element: Element): Element[] {
  const out: Element[] = [];
  for (const child of element.children) {
    if ((child as HTMLElement).dataset.probe !== undefined) out.push(child);
    else out.push(...probedChildren(child));
  }
  return out;
}

/**
 * The part of `element`'s box its scrolling or clipping boxes inside `holder` let show, or null where
 * none of it does: a list's rows scrolled out of its well are measured where they are seen.
 */
function shown(element: Element, holder: HTMLElement): DOMRect | null {
  const clip = clipOf(element, holder);
  return clip ? meet(element.getBoundingClientRect(), clip) : null;
}

/** What the boxes round `element` inside `holder` let show of anything in their place. */
function clipOf(element: Element, holder: HTMLElement): DOMRect | null {
  let clip: DOMRect | null = holder.getBoundingClientRect();
  for (let at = element.parentElement; at && at !== holder && clip; at = at.parentElement) {
    if (getComputedStyle(at).overflow === "visible") continue;
    clip = meet(clip, at.getBoundingClientRect());
  }
  return clip;
}

/** Where `a` and `b` overlap, or null where they don't by a pixel. */
function meet(a: DOMRect, b: DOMRect): DOMRect | null {
  const left = Math.max(a.left, b.left);
  const top = Math.max(a.top, b.top);
  const right = Math.min(a.right, b.right);
  const bottom = Math.min(a.bottom, b.bottom);
  if (right - left < 1 || bottom - top < 1) return null;
  return new DOMRect(left, top, right - left, bottom - top);
}

/** The spaces a part keeps: in its CSS pixels, which are the page's device pixels. */
function sides(element: Element, prefix: "padding" | "margin"): Record<Side, number> {
  const style = getComputedStyle(element);
  const read = (side: Side) => Number.parseFloat(style.getPropertyValue(`${prefix}-${side}`)) || 0;
  return { top: read("top"), right: read("right"), bottom: read("bottom"), left: read("left") };
}

/** A tree line: a part, or several alike side by side, and the parts inside them. */
interface Line {
  /** Every part the line stands for, in every instance of the lines above it. */
  readonly parts: readonly Element[];
  /** How many alike siblings it folds together, each still inspectable by its chip. */
  readonly count: number;
  children: readonly Line[];
  readonly parent: Line | null;
  readonly row: HTMLElement;
  readonly words: HTMLElement;
  readonly chips: readonly HTMLElement[];
  /** The part the line stands for now: inside the part its parent line stands for. */
  shown: Element;
}

/** `sample` with its layer, its parts tree and its spec list, telling `hands` what the reader does. */
export function inspector(name: string, sample: Element, hands: Hands): Inspector {
  const marks = el("div", "page-overlay");
  const holder = el("div", "page-measured", sample, marks);
  const tree = el("div", "page-tree");
  const spec = el("dl", "page-spec");
  const aside = el("div", "page-inspect", tree, spec);
  let lines: Line[] = [];
  let roots: Line[] = [];
  const read = reader(sample);
  const self: Inspector = { holder, aside, draw, name, uses: (token) => read.token(token).size > 0 };

  /** The tokens an element's spec tries: the table's, and every one that could reach it. */
  const candidates = (element: Element): string[] => [...new Set([...hands.tokens, ...tokensFor(element, sample)])];

  // A sample that changes is read again: where it uses each token the next time it is asked, and
  // its tree once its parts change (a menu laid out for its screen again, the pane showing
  // another Result).
  let rebuilding = false;
  new MutationObserver((records) => {
    read.forget();
    if (rebuilding || !records.some((record) => record.type === "childList" || record.attributeName === "data-probe")) return;
    rebuilding = true;
    requestAnimationFrame(() => {
      rebuilding = false;
      buildTree();
      redraw();
    });
  }).observe(sample, { subtree: true, childList: true, attributes: true, attributeFilter: ["style", "class", "data-probe", "hidden"] });

  function buildTree(): void {
    lines = [];
    roots = linesOf((sample as HTMLElement).dataset.probe !== undefined ? [sample] : probedChildren(sample), null, 0);
    tree.replaceChildren(...lines.map((line) => line.row));
    showUnder(roots, null);
  }

  /**
   * The lines for `parts`, grouped as their first parent instance's are: alike siblings side by
   * side fold into one line, in every instance of the line above at once.
   */
  function linesOf(parts: readonly Element[], parent: Line | null, depth: number): Line[] {
    const instances = parent ? parent.parts : [null];
    const perInstance = instances.map((instance) => (instance ? probedChildren(instance) : parts));
    const first = perInstance[0] ?? [];
    const runs: { start: number; count: number }[] = [];
    for (let i = 0; i < first.length; i++) {
      const last = runs.at(-1);
      // Texts never fold: two lines of a row say different things in different styles.
      if (last && probeOf(first[i]!).role !== "text" && signature(first[last.start]!) === signature(first[i]!)) last.count++;
      else runs.push({ start: i, count: 1 });
    }
    const out: Line[] = [];
    for (const run of runs) {
      const members = perInstance.flatMap((kids) => kids.slice(run.start, run.start + run.count));
      const words = el("span", "page-tree-words");
      const chips = run.count > 1 ? Array.from({ length: run.count }, (_, j) => el("span", "page-tree-chip", String(j + 1))) : [];
      const row = el("div", "page-tree-line", words, ...chips);
      row.style.setProperty("--depth", String(depth));
      const line: Line = { parts: members, count: run.count, children: [], parent, row, words, chips, shown: members[0]! };
      lines.push(line);
      chips.forEach((chip, j) => {
        chip.addEventListener("pointerenter", () => hands.point(siblingsOf(line)[j] ?? line.shown, self));
        chip.addEventListener("click", (event) => {
          event.stopPropagation();
          hands.click(siblingsOf(line)[j] ?? line.shown, self);
        });
      });
      row.addEventListener("pointerenter", () => hands.point(line.shown, self));
      row.addEventListener("click", () => hands.click(line.shown, self));
      line.children = linesOf(members, line, depth + 1);
      out.push(line);
    }
    return out;
  }

  tree.addEventListener("pointerleave", () => hands.point(null, self));

  /** The parts of `line`'s fold inside the part its parent line stands for. */
  function siblingsOf(line: Line): Element[] {
    const parent = line.parent?.shown;
    return line.parts.filter((part) => !parent || parent.contains(part));
  }

  /** Points every line at the part it stands for: the one holding `part` where it can. */
  function showUnder(list: readonly Line[], part: Element | null): void {
    for (const line of list) {
      const inside = siblingsOf(line);
      line.shown = inside.find((each) => part !== null && each.contains(part)) ?? inside[0] ?? line.parts[0]!;
      line.words.replaceChildren(el("span", "", partName(line.shown)), ...lineDetail(line.shown), ...(line.count > 1 ? [el("span", "page-tree-count", `×${line.count}`)] : []));
      line.chips.forEach((chip, j) => chip.toggleAttribute("data-shown", inside[j] === line.shown));
      showUnder(line.children, part);
    }
  }

  /** What tells one part from its siblings in the tree: a text's words. */
  function lineDetail(part: Element): HTMLElement[] {
    return probeOf(part).role === "text" ? [el("span", "page-tree-detail", part.textContent ?? "")] : [];
  }

  holder.addEventListener("pointermove", (event) => {
    const part = event.target instanceof Element ? event.target.closest("[data-probe]") : null;
    hands.point(part && sample.contains(part) ? part : null, self);
  });
  holder.addEventListener("pointerleave", () => hands.point(null, self));
  // A click in a measured sample is the inspector's, never the sample's, so measuring it doesn't
  // change it: no row is selected, nothing takes focus. `site/scroll.ts` has already heard it, on
  // the window, and given the sample the wheel.
  for (const type of ["pointerdown", "mousedown", "pointerup", "mouseup", "dblclick", "auxclick"]) {
    holder.addEventListener(type, (event) => {
      event.preventDefault();
      event.stopPropagation();
    }, { capture: true });
  }
  holder.addEventListener("click", (event) => {
    event.preventDefault();
    event.stopPropagation();
    const part = event.target instanceof Element ? event.target.closest("[data-probe]") : null;
    if (part && sample.contains(part)) hands.click(part, self);
  }, { capture: true });
  let queued = false;
  // A list scrolled inside the sample moves what is drawn over it.
  holder.addEventListener("scroll", () => {
    if (queued) return;
    queued = true;
    requestAnimationFrame(() => {
      queued = false;
      redraw();
    });
  }, { capture: true });

  let last: { lit: Lit | null; pinned: boolean } = { lit: null, pinned: false };
  const redraw = () => draw(last.lit, last.pinned);

  function draw(lit: Lit | null, pinned: boolean): number {
    last = { lit, pinned };
    marks.replaceChildren();
    const part = lit && "part" in lit && lit.in === self && sample.contains(lit.part) ? lit.part : null;
    if (part) showUnder(roots, part);
    for (const line of lines) {
      line.row.toggleAttribute("data-lit", line.shown === part);
      line.row.toggleAttribute("data-pinned", pinned && line.shown === part);
    }
    if (part) {
      const at = lines.find((line) => line.shown === part);
      if (at) keepInView(at.row);
      drawPart(part);
      spec.replaceChildren(...specOf(part));
      return 0;
    }
    const places = lit && "token" in lit && holder.offsetParent !== null ? drawToken(lit.token) : 0;
    const token = lit && "token" in lit ? lit.token : null;
    spec.replaceChildren(el("dd", "page-spec-hint", token ? litWords(token, places, places === 0 && self.uses(token) ? [name] : []) : HINT));
    return places;
  }

  /** Scrolls the tree, never the page, so `row` is in view in it. */
  function keepInView(row: HTMLElement): void {
    const top = row.offsetTop - tree.offsetTop;
    if (top < tree.scrollTop) tree.scrollTop = top;
    else if (top + row.offsetHeight > tree.scrollTop + tree.clientHeight) tree.scrollTop = top + row.offsetHeight - tree.clientHeight;
  }

  // ------------------------------------------------------------------------------- drawing --

  /** The layer's frame: from the viewport's pixels to the holder's device pixels, whole ones. */
  function frame() {
    const origin = holder.getBoundingClientRect();
    const zoom = holder.offsetWidth / origin.width || 1;
    const x = (value: number) => Math.round((value - origin.left) * zoom);
    const y = (value: number) => Math.round((value - origin.top) * zoom);
    const put = (className: string, box: DOMRect, kind?: Kind): HTMLElement => {
      const mark = el("div", className);
      if (kind) mark.dataset.kind = kind;
      const left = x(box.left);
      const top = y(box.top);
      mark.style.left = `${left}px`;
      mark.style.top = `${top}px`;
      mark.style.width = `${x(box.right) - left}px`;
      mark.style.height = `${y(box.bottom) - top}px`;
      marks.append(mark);
      return mark;
    };
    return { zoom, origin, put };
  }

  /** The bands of `element`'s spaces of `prefix` on `which` sides, clipped to `clip`; how many. */
  function spaceBands(element: Element, prefix: "padding" | "margin", which: readonly Side[], clip: DOMRect, put: ReturnType<typeof frame>["put"], zoom: number): number {
    const box = element.getBoundingClientRect();
    const space = sides(element, prefix);
    const d = (side: Side) => space[side] / zoom;
    const outside = prefix === "margin";
    const rects: Record<Side, DOMRect> = outside
      ? {
          top: new DOMRect(box.left, box.top - d("top"), box.width, d("top")),
          bottom: new DOMRect(box.left, box.bottom, box.width, d("bottom")),
          left: new DOMRect(box.left - d("left"), box.top, d("left"), box.height),
          right: new DOMRect(box.right, box.top, d("right"), box.height),
        }
      : {
          top: new DOMRect(box.left, box.top, box.width, d("top")),
          bottom: new DOMRect(box.left, box.bottom - d("bottom"), box.width, d("bottom")),
          left: new DOMRect(box.left, box.top + d("top"), d("left"), box.height - d("top") - d("bottom")),
          right: new DOMRect(box.right - d("right"), box.top + d("top"), d("right"), box.height - d("top") - d("bottom")),
        };
    let drawn = 0;
    for (const side of which) {
      const band = meet(rects[side], clip);
      if (!band) continue;
      put("page-band", band, prefix === "margin" ? "gap" : "padding");
      drawn++;
    }
    return drawn;
  }

  /**
   * The gaps between `element`'s children shown, each its gap wide after the one before it: across
   * a line of a row or a grid, under it in a stack or between a grid's rows; how many.
   */
  function gapBands(element: Element, put: ReturnType<typeof frame>["put"], zoom: number): number {
    const own = shown(element, holder);
    if (!own) return 0;
    const style = getComputedStyle(element);
    const across = (Number.parseFloat(style.columnGap) || 0) / zoom;
    const down = (Number.parseFloat(style.rowGap) || 0) / zoom;
    if (across === 0 && down === 0) return 0;
    const kids = [...element.children]
      .filter((kid) => getComputedStyle(kid).position !== "absolute")
      .map((kid) => kid.getBoundingClientRect())
      .filter((box) => box.width > 0 || box.height > 0);
    const box = element.getBoundingClientRect();
    const single = style.display.includes("flex") && style.flexWrap === "nowrap";
    let drawn = 0;
    for (let i = 1; i < kids.length; i++) {
      const before = kids[i - 1]!;
      const after = kids[i]!;
      const sameLine = after.top < before.bottom && before.top < after.bottom && after.left >= before.right - 0.5;
      const band = sameLine
        ? across > 0
          ? single
            ? new DOMRect(before.right, box.top, across, box.height)
            : new DOMRect(before.right, Math.min(before.top, after.top), across, Math.max(before.bottom, after.bottom) - Math.min(before.top, after.top))
          : null
        : down > 0
          ? new DOMRect(box.left, before.bottom, box.width, down)
          : null;
      const seen = band && meet(band, own);
      if (!seen) continue;
      put("page-band", seen, "gap");
      drawn++;
    }
    return drawn;
  }

  /** `part`'s box, its size tag outside it, its padding and its gaps. */
  function drawPart(part: Element): void {
    const box = shown(part, holder);
    if (!box) return;
    const { zoom, put } = frame();
    const spacer = probeOf(part).role === "spacer" && !probeOf(part).fill;
    if (spacer) put("page-band", box, "gap");
    else {
      spaceBands(part, "padding", SIDES, box, put, zoom);
      gapBands(part, put, zoom);
    }
    put("page-mark", box);
    const whole = part.getBoundingClientRect();
    const scale = settings().scale;
    tag(`${round((whole.width * zoom) / scale)} × ${round((whole.height * zoom) / scale)}`, box, zoom);
  }

  /**
   * The size tag, outside the part's box so it never covers the part's own words: under it, over
   * it, right of it or left of it, the first of those in the sample's box that covers no other
   * words or symbol either, else the first in the sample's box; inside the box's foot only where the
   * sample's edge leaves no room round it. Under or over, its left edge is the part's where it can be.
   */
  function tag(words: string, box: DOMRect, zoom: number): void {
    const { origin } = frame();
    const name = el("span", "page-tag", words);
    marks.append(name);
    const width = name.offsetWidth;
    const height = name.offsetHeight;
    const apart = Math.round(length("space-s4") * settings().scale);
    const room = { width: holder.offsetWidth, height: holder.offsetHeight };
    const device = (r: DOMRect) => ({
      left: Math.round((r.left - origin.left) * zoom),
      top: Math.round((r.top - origin.top) * zoom),
      right: Math.round((r.right - origin.left) * zoom),
      bottom: Math.round((r.bottom - origin.top) * zoom),
    });
    const part = device(box);
    const middle = Math.round((part.top + part.bottom - height) / 2);
    const along = Math.max(0, Math.min(part.left, room.width - width));
    const around = [
      { left: along, top: part.bottom + apart },
      { left: along, top: part.top - apart - height },
      { left: part.right + apart, top: middle },
      { left: part.left - apart - width, top: middle },
    ].filter((at) => at.left >= 0 && at.top >= 0 && at.left + width <= room.width && at.top + height <= room.height);
    const marked = [...sample.querySelectorAll('[data-probe*=\'"role":"text"\'], [data-probe*=\'"role":"icon"\']')]
      .map((each) => shown(each, holder))
      .filter((each): each is DOMRect => each !== null)
      .map(device);
    const covers = (at: { left: number; top: number }) =>
      marked.some((other) => at.left < other.right && other.left < at.left + width && at.top < other.bottom && other.top < at.top + height);
    const at = around.find((each) => !covers(each)) ?? around[0] ?? { left: along, top: part.bottom - apart - height };
    name.style.left = `${at.left}px`;
    name.style.top = `${at.top}px`;
  }

  /** Every place `token` is used in the sample and shown: a box, a padding, a margin or a gap. */
  function drawToken(token: string): number {
    const { zoom, put } = frame();
    let places = 0;
    for (const [element, list] of read.token(token)) {
      for (const use of list) places += drawUse(element, use, put, zoom);
    }
    return places;
  }

  function drawUse(element: Element, use: Use, put: ReturnType<typeof frame>["put"], zoom: number): number {
    const box = shown(element, holder);
    if (!box) return 0;
    switch (use.kind) {
      case "padding":
        return spaceBands(element, "padding", use.sides, box, put, zoom) > 0 ? 1 : 0;
      case "margin": {
        const clip = clipOf(element, holder);
        return clip && spaceBands(element, "margin", use.sides, clip, put, zoom) > 0 ? 1 : 0;
      }
      case "gap":
        if (probeOf(element).role === "spacer") {
          put("page-band", box, "gap");
          return 1;
        }
        return gapBands(element, put, zoom);
      case "box":
        put("page-mark", box).dataset.lit = "";
        return 1;
    }
  }

  // ---------------------------------------------------------------------------------- spec --

  /** The inspected part's measures, each a line in its band's colour, with the tokens it takes. */
  function specOf(part: Element): HTMLElement[] {
    const zoom = frame().zoom;
    const scale = settings().scale;
    const logical = (devicePx: number) => round(devicePx / scale);
    const mine = read.element(part, () => candidates(part));
    const named = (pick: (use: Use) => boolean) => [...new Set(mine.filter(pick).map((use) => tokenName(use.token)))];
    const out: HTMLElement[] = [];
    const line = (term: string, kind: Kind | null, value: string, tokens: readonly string[]) => {
      // A measure drawn in no colour keeps the swatch's room, so the names line up.
      const swatch = el("span", "page-swatch");
      swatch.dataset.kind = kind ?? "none";
      out.push(el("dt", "", swatch, term), el("dd", "", value, ...tokens.map((token) => el("code", "page-code", token))));
    };
    const whole = part.getBoundingClientRect();
    const sizes = /^(width|height|min-|max-|flex-basis|aspect-ratio|grid-)/;
    line("Size", "box", `${logical(whole.width * zoom)} × ${logical(whole.height * zoom)}`, named((use) => use.kind === "box" && (use.properties.length === 0 || use.properties.some((p) => sizes.test(p)))));
    const padding = sides(part, "padding");
    const paddingTokens = named((use) => use.kind === "padding");
    if (SIDES.some((side) => padding[side] > 0) || paddingTokens.length > 0) line("Padding", "padding", fourSides(padding, logical), paddingTokens);
    const style = getComputedStyle(part);
    const spacer = probeOf(part).role === "spacer" && !probeOf(part).fill;
    const gaps = [style.rowGap, style.columnGap].map((value) => Number.parseFloat(value) || 0);
    const gapTokens = named((use) => use.kind === "gap");
    if (spacer) line("Gap", "gap", `${logical(Math.max(whole.width, whole.height) * zoom)}`, gapTokens);
    else if (gaps.some((value) => value > 0) || gapTokens.length > 0) line("Gap", "gap", gaps[0] === gaps[1] ? logical(gaps[0]!) : `${logical(gaps[0]!)} ${logical(gaps[1]!)}`, gapTokens);
    const margin = sides(part, "margin");
    if (SIDES.some((side) => margin[side] !== 0)) line("Margin", "gap", fourSides(margin, logical), named((use) => use.kind === "margin"));
    const radius = Number.parseFloat(style.borderTopLeftRadius) || 0;
    const radiusTokens = named((use) => use.properties.some((p) => p.endsWith("-radius")));
    if (radius > 0 || radiusTokens.length > 0) line("Radius", null, logical(Math.min(radius, (Math.min(whole.width, whole.height) * zoom) / 2)), radiusTokens);
    const inset = insetOf(part, zoom, logical);
    if (inset) line("Inset", null, inset.words, inset.tokens);
    const others = named((use) => use.kind === "box" && use.properties.length > 0 && !use.properties.some((p) => sizes.test(p) || p.endsWith("-radius")));
    if (others.length > 0) line("Also", null, "", others);
    return out;
  }

  /**
   * How far `part` sits inside its probed parent, each side, and the parent's padding token on the
   * sides where it sits on the parent's padding edge exactly.
   */
  function insetOf(part: Element, zoom: number, logical: (devicePx: number) => string): { words: string; tokens: string[] } | null {
    const parent = part.parentElement?.closest("[data-probe]");
    if (!parent || !sample.contains(parent)) return null;
    const outer = parent.getBoundingClientRect();
    const inner = part.getBoundingClientRect();
    const offsets: Record<Side, number> = {
      top: (inner.top - outer.top) * zoom,
      right: (outer.right - inner.right) * zoom,
      bottom: (outer.bottom - inner.bottom) * zoom,
      left: (inner.left - outer.left) * zoom,
    };
    const padding = sides(parent, "padding");
    const onEdge = SIDES.filter((side) => padding[side] > 0 && Math.abs(offsets[side] - padding[side]) < 0.5);
    const tokens = [...new Set(read.element(parent, () => candidates(parent)).filter((use) => use.kind === "padding" && use.sides.some((side) => onEdge.includes(side))).map((use) => tokenName(use.token)))];
    return { words: `${fourSides(offsets, logical)} in ${partName(parent)}`, tokens };
  }

  buildTree();
  spec.append(el("dd", "page-spec-hint", HINT));
  return self;
}

/** Four sides as CSS writes them, as few numbers as say them. */
function fourSides(space: Record<Side, number>, logical: (devicePx: number) => string): string {
  const [top, right, bottom, left] = SIDES.map((side) => logical(space[side]));
  if (top === right && right === bottom && bottom === left) return top!;
  if (top === bottom && right === left) return `${top} ${right}`;
  return `${top} ${right} ${bottom} ${left}`;
}
