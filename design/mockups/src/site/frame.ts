// The page's geometry frame: every mocked element's probe, with its box in logical pixels from the
// page's top left, as the QML specimen reports its items (`qml/specimen/main.qml`). An element that
// says nothing is left out and its children kept, but a layout's child that says nothing is a
// plain box, since the layout's rules hold its edges, and a plain box around one child with its
// bounds is that child, as iced reports them.
//
// A box drawn scaled (CSS `scale`, or a `transform` that scales) is a picture of UI, such as the
// Gallery's small screens or windows stepped back behind a lock: it and everything in it are
// reported at the size they are laid out at, from its own top left on a whole pixel, marked
// `picture` with the ratio it is drawn at.
//
// The root is the page's own box, which claims nothing, so every mock on the page is one frame.

interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface FrameNode {
  role: string;
  bounds: Bounds;
  children: FrameNode[];
  [key: string]: unknown;
}

export interface Frame {
  page: string;
  mode: string;
  scale: number;
  root: FrameNode;
}

/** A box as drawn in the viewport, mapped to logical pixels in the space being walked. */
type Space = (drawn: DOMRectReadOnly) => Bounds;

interface Walk {
  readonly space: Space;
  /** The CSS pixels a logical pixel takes, outside any picture. */
  readonly perCss: number;
}

function same(a: Bounds, b: Bounds): boolean {
  return a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height;
}

/** `element`'s nodes, appended to `into`. */
function walk(element: Element, at: Walk, into: FrameNode[]): void {
  if (!(element instanceof HTMLElement || element instanceof SVGElement)) return;
  const style = getComputedStyle(element);
  if (style.display === "none") return;
  const parent = element.parentElement;
  const laidOut = parent?.dataset.layout !== undefined && parent?.dataset.probe !== undefined;
  const drawn = element.getBoundingClientRect();
  const probe = element.dataset.probe;
  const declared = probe ? (JSON.parse(probe) as Record<string, unknown>) : null;
  const matrix = element instanceof HTMLElement ? transformOf(style) : null;
  const picture = matrix ? pictureOf(matrix) : null;
  const found: Record<string, unknown> | null = declared ?? (laidOut || picture ? { role: "box" } : null);
  if (!found) {
    for (const child of element.children) walk(child, at, into);
    return;
  }
  const role = { ...found };
  let bounds = at.space(drawn);
  let space = at.space;
  if (picture && matrix) {
    role.picture = Number(picture.ratio.toFixed(4));
    if (picture.turned) {
      // Turned or mirrored, its layout can't be read back from what is drawn: one box, as drawn.
      into.push({ ...role, role: String(role.role), bounds, children: [] });
      return;
    }
    ({ bounds, space } = unscaled(bounds, matrix, at));
  }
  const children: FrameNode[] = [];
  if (role.role !== "icon") {
    for (const child of element.children) walk(child, { ...at, space }, children);
  }
  if (role.role === "text") {
    role.content = element.textContent ?? "";
    const natural = Number(element.style.getPropertyValue("--natural"));
    const lines = Math.max(1, Math.round(bounds.height / lineOf(element)));
    role.natural = { width: natural, height: lines * lineOf(element) };
  }
  let kept = children;
  while (kept.length === 1 && kept[0]!.role === "box" && !marked(kept[0]!) && same(kept[0]!.bounds, bounds)) kept = kept[0]!.children;
  if (role.role === "box" && !marked(role) && kept.length === 1 && same(kept[0]!.bounds, bounds)) {
    into.push(kept[0]!);
    return;
  }
  into.push({ ...role, role: String(role.role), bounds, children: kept });
}

/** Whether a box says more than that it is a box: what folding it into its child would lose. */
function marked(node: Record<string, unknown>): boolean {
  return Object.keys(node).some((key) => key !== "role" && key !== "bounds" && key !== "children");
}





/**
 * A box's own transform, in its CSS pixels from its laid-out top left: `translate`, `rotate`,
 * `scale` and `transform` about its `transform-origin`, as CSS composes them.
 */
function transformOf(style: CSSStyleDeclaration): DOMMatrix | null {
  if (style.transform === "none" && style.scale === "none" && style.translate === "none" && style.rotate === "none") return null;
  const [width, height] = laidOutSize(style);
  const [ox = 0, oy = 0] = style.transformOrigin.split(" ").map(Number.parseFloat);
  const matrix = new DOMMatrix().translateSelf(ox, oy);
  if (style.translate !== "none") {
    const [tx = "0px", ty = "0px"] = style.translate.split(" ");
    matrix.translateSelf(lengthOn(tx, width), lengthOn(ty, height));
  }
  if (style.rotate !== "none") matrix.rotateSelf(Number.parseFloat(style.rotate.split(" ").at(-1)!));
  if (style.scale !== "none") {
    const [sx = 1, sy = sx] = style.scale.split(" ").map(Number);
    matrix.scaleSelf(sx, sy);
  }
  if (style.transform !== "none") matrix.multiplySelf(new DOMMatrix(style.transform));
  return matrix.translateSelf(-ox, -oy);
}

/** A translation's length in px, a percentage being of `of`. */
function lengthOn(value: string, of: number): number {
  return value.endsWith("%") ? (Number.parseFloat(value) / 100) * of : Number.parseFloat(value);
}

/** A box's border box as laid out, before any transform, in its CSS pixels. */
function laidOutSize(style: CSSStyleDeclaration): [number, number] {
  const px = (value: string) => Number.parseFloat(value) || 0;
  const width = px(style.width);
  const height = px(style.height);
  if (style.boxSizing === "border-box") return [width, height];
  const across = px(style.paddingLeft) + px(style.paddingRight) + px(style.borderLeftWidth) + px(style.borderRightWidth);
  const down = px(style.paddingTop) + px(style.paddingBottom) + px(style.borderTopWidth) + px(style.borderBottomWidth);
  return [width + across, height + down];
}

/**
 * The ratio a transform draws its box at when it scales it, and whether it also turns or mirrors
 * it; null when it keeps the box's size: a move, or a turn by quarters.
 */
function pictureOf(m: DOMMatrix): { ratio: number; turned: boolean } | null {
  const across = Math.hypot(m.a, m.b);
  const down = Math.hypot(m.c, m.d);
  if (m.is2D && Math.abs(across - 1) < 1e-4 && Math.abs(down - 1) < 1e-4) return null;
  const turned = !m.is2D || Math.abs(m.b) > 1e-6 || Math.abs(m.c) > 1e-6 || m.a <= 0 || m.d <= 0;
  return { ratio: Math.min(across, down), turned };
}


/**
 * A picture's box and the space of what it holds: the box where it is laid out, at the size it is
 * laid out at, worked back from where it is drawn (`bounds`) and its transform; everything inside
 * at its own size from there. Its own grid starts at its own top left, as the screen it is a
 * picture of does, so that corner is put on the whole device pixel nearest where it is laid out:
 * a gallery lays its pictures out wherever its shares fall.
 */
function unscaled(bounds: Bounds, m: DOMMatrix, at: Walk): { bounds: Bounds; space: Space } {
  const onPixel = (logical: number) => Math.round(logical * at.perCss) / at.perCss;
  const laid: Bounds = {
    x: onPixel(bounds.x - m.e / at.perCss),
    y: onPixel(bounds.y - m.f / at.perCss),
    width: bounds.width / m.a,
    height: bounds.height / m.d,
  };
  const outer = at.space;
  const space: Space = (drawn) => {
    const b = outer(drawn);
    return { x: laid.x + (b.x - bounds.x) / m.a, y: laid.y + (b.y - bounds.y) / m.d, width: b.width / m.a, height: b.height / m.d };
  };
  return { bounds: laid, space };
}

/** A text's line height in logical pixels. */
function lineOf(element: Element): number {
  const line = Number.parseFloat(getComputedStyle(element).lineHeight);
  const scale = Number(getComputedStyle(element).getPropertyValue("--scale")) || 1;
  return line / scale;
}

/** The frame of `page`, the page's box, drawn at `scale` in `mode`. */
export function frame(page: HTMLElement, slug: string, mode: string, scale: number): Frame {
  const origin = page.getBoundingClientRect();
  // The page's box is zoomed so its CSS pixels are device pixels; a logical pixel is `scale` of
  // them, which the zoom shrinks in the viewport's pixels.
  const perLogical = (scale * origin.width) / page.offsetWidth;
  const space: Space = (drawn) => ({
    x: (drawn.left - origin.left) / perLogical,
    y: (drawn.top - origin.top) / perLogical,
    width: drawn.width / perLogical,
    height: drawn.height / perLogical,
  });
  const children: FrameNode[] = [];
  const at: Walk = { space, perCss: scale };
  for (const child of page.children) walk(child, at, children);
  return {
    page: slug,
    mode,
    scale,
    root: { role: "box", bounds: { x: 0, y: 0, width: page.offsetWidth / scale, height: page.offsetHeight / scale }, children },
  };
}
