// Centring on whole device pixels, where CSS centres in room it can't know the size of: a row's
// children centred down it, a stack's across it, a part centred in a box its content sizes. CSS
// puts a centred box half the spare room in, which is half a device pixel whenever the spare is
// odd; the builds round that offset to a whole pixel, the odd one after the box (`kubaos-ui`'s
// `Scale::centre`, `Look.centre` in QML). A part that centres between two sizes the CSS knows
// rounds there, with `round(down, …, 1px)`; this does the rest, once the CSS has laid the page out.
//
// A box its parent centres (a flex line, a grid cell, a centred line of text) whose offset falls
// between device pixels gets the odd pixel as a margin after it, along the axis it is centred on.
// The centring shares that margin out, so the box moves half a pixel toward its start and nothing
// changes size: the box is shorter than its room by at least the pixel it takes. Along a flex
// line's main axis the line's last box takes it, so the whole line moves together.
//
// The page's CSS pixels are device pixels (`main.ts` zooms the page by the display's ratio), so a
// margin is in device pixels and a box read in the viewport's pixels is turned into them by
// `devicePixelRatio`. A box drawn scaled (a picture of a screen, a sample fitted to its pane) isn't
// on the device grid as drawn, and is left where the CSS puts it.

/** Under this much of a device pixel apart, two edges are the same: float error, not layout. */
const EPSILON = 1e-2;

type Axis = "x" | "y";
type Side = "marginRight" | "marginBottom";

const SIDE: Record<Axis, Side> = { x: "marginRight", y: "marginBottom" };

/** A margin this set, and the one the CSS gives, so the next pass knows where the CSS put the box. */
interface Correction {
  readonly base: number;
  readonly shift: number;
}

const corrections = new WeakMap<HTMLElement, Partial<Record<Side, Correction>>>();

function whole(device: number): boolean {
  return Math.abs(device - Math.round(device)) < EPSILON;
}

/**
 * `offset` device pixels, half a spare, on a whole device pixel as the builds round it: to the
 * nearest, a tie toward the start, so an odd spare pixel goes after the box.
 */
export function centredOffset(offset: number): number {
  // HAZARD: half of an odd count is a tie, which float error breaks either way; a hair under rounds
  // it down, as `Scale::centre` does.
  return Math.round(offset - 1e-3);
}

function isCentre(value: string): boolean {
  return value === "center" || value === "safe center" || value === "unsafe center";
}

/** Whether `parent` centres `child` along `axis`, and along its flex line's main axis if so. */
function centring(parentStyle: CSSStyleDeclaration, child: HTMLElement, axis: Axis): { readonly main: boolean } | null {
  const display = parentStyle.display;
  if (display.endsWith("flex")) {
    const main = parentStyle.flexDirection.startsWith("row") === (axis === "x");
    if (main) return parentStyle.flexWrap === "nowrap" && isCentre(parentStyle.justifyContent) ? { main } : null;
    const self = getComputedStyle(child).alignSelf;
    return isCentre(self === "auto" ? parentStyle.alignItems : self) ? { main } : null;
  }
  if (display.endsWith("grid")) {
    const style = getComputedStyle(child);
    const self = axis === "x" ? style.justifySelf : style.alignSelf;
    const items = axis === "x" ? parentStyle.justifyItems : parentStyle.alignItems;
    return isCentre(self === "auto" ? items : self) ? { main: false } : null;
  }
  if (axis === "x" && parentStyle.textAlign === "center" && getComputedStyle(child).display.startsWith("inline")) {
    return { main: true };
  }
  return null;
}

/** The boxes `parent` lays out in its flow, first to last: those a centred line is made of. */
function inFlow(parent: HTMLElement): HTMLElement[] {
  return [...parent.children].filter((child): child is HTMLElement => {
    if (!(child instanceof HTMLElement) || child.getClientRects().length === 0) return false;
    const position = getComputedStyle(child).position;
    return position !== "absolute" && position !== "fixed";
  });
}

/**
 * Moves every box under `root` that its parent centres onto whole device pixels, the odd pixel
 * after it. Every box is read, then every margin set: the page is laid out once more at most.
 */
export function centreOnPixels(root: ParentNode): void {
  const perCss = devicePixelRatio;
  const origin = document.getElementById("page") ?? document.documentElement;
  const rects = new Map<Element, DOMRect>();
  const rectOf = (element: Element): DOMRect => {
    let rect = rects.get(element);
    if (!rect) {
      rect = element.getBoundingClientRect();
      rects.set(element, rect);
    }
    return rect;
  };
  const startOf = (element: Element, axis: Axis) => (axis === "x" ? rectOf(element).left : rectOf(element).top) * perCss;
  const off = (element: Element, axis: Axis, from: Element) => !whole(startOf(element, axis) - startOf(from, axis));

  const planned = new Map<HTMLElement, Partial<Record<Side, number>>>();
  const looked = new Set<string>();
  let looks = 0;
  const lookedAt = new WeakMap<Element, number>();
  const keyOf = (element: Element, what: string) => {
    let id = lookedAt.get(element);
    if (id === undefined) {
      id = looks++;
      lookedAt.set(element, id);
    }
    return `${id}:${what}`;
  };

  // The boxes the geometry check reads: every probed one, and every child of a layout, which it
  // reads as a plain box.
  for (const element of root.querySelectorAll<HTMLElement>("[data-probe], [data-layout] > *")) {
    if (!origin.contains(element)) continue;
    const own = rectOf(element);
    if (own.width === 0 && own.height === 0) continue;
    for (const axis of ["x", "y"] as const) {
      // The box that put it between pixels: itself, or the nearest box round it whose start falls
      // between them against its parent's.
      let box: HTMLElement = element;
      if (!element.parentElement || !off(element, axis, element.parentElement)) {
        if (!off(element, axis, origin)) continue;
        let up = element.parentElement;
        while (up && up !== origin && up.parentElement && !off(up, axis, up.parentElement)) up = up.parentElement;
        if (!up || up === origin || !up.parentElement) continue;
        box = up;
      }
      plan(box, axis);
    }
  }

  for (const [carrier, sides] of planned) {
    const had = corrections.get(carrier) ?? {};
    for (const side of ["marginRight", "marginBottom"] as const) {
      const shift = sides[side];
      if (shift === undefined) continue;
      const before = had[side];
      if (before && Math.abs(before.shift - shift) < EPSILON) continue;
      const base = before?.base ?? Number.parseFloat(getComputedStyle(carrier)[side]);
      had[side] = { base, shift };
      carrier.style[side] = `${base + shift}px`;
    }
    corrections.set(carrier, had);
  }

  function plan(box: HTMLElement, axis: Axis): void {
    let parent = box.parentElement;
    let parentStyle = parent ? getComputedStyle(parent) : null;
    while (parent && parentStyle?.display === "contents") {
      parent = parent.parentElement;
      parentStyle = parent ? getComputedStyle(parent) : null;
    }
    if (!parent || !parentStyle) return;
    const how = centring(parentStyle, box, axis);
    if (!how) return;
    const key = keyOf(how.main ? parent : box, `${axis}${how.main ? "main" : "cross"}`);
    if (looked.has(key)) return;
    looked.add(key);
    const drawn = rectOf(parent);
    if (Math.abs(drawn.width * perCss - parent.offsetWidth) > 1 || Math.abs(drawn.height * perCss - parent.offsetHeight) > 1) return;
    const line = how.main ? inFlow(parent) : [box];
    const lead = line[0];
    const carrier = line.at(-1);
    if (!lead || !carrier) return;
    const side = SIDE[axis];
    const edge = axis === "x" ? "Left" : "Top";
    const contentStart = startOf(parent, axis) + Number.parseFloat(parentStyle[`border${edge}Width`]) + Number.parseFloat(parentStyle[`padding${edge}`]);
    // Where the CSS centres the line, before a correction moved it half its margin toward the start.
    const offset = startOf(lead, axis) - contentStart + (corrections.get(carrier)?.[side]?.shift ?? 0) / 2;
    const shift = 2 * (offset - centredOffset(offset));
    planned.set(carrier, { ...planned.get(carrier), [side]: shift });
  }
}
