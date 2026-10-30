// The page's geometry frame: every mocked element's probe, with its box in logical pixels from the
// page's top left, as the QML specimen reports its items (`qml/specimen/main.qml`). An element that
// says nothing is left out and its children kept, but a layout's child that says nothing is a
// plain box, since the layout's rules hold its edges, and a plain box around one child with its
// bounds is that child, as iced reports them.
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

function same(a: Bounds, b: Bounds): boolean {
  return a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height;
}

/** `element`'s nodes, appended to `into`; `perLogical` is the CSS pixels a logical pixel takes. */
function walk(element: Element, origin: DOMRect, perLogical: number, into: FrameNode[]): void {
  if (!(element instanceof HTMLElement || element instanceof SVGElement)) return;
  if (getComputedStyle(element).display === "none") return;
  const parent = element.parentElement;
  const laidOut = parent?.dataset.layout !== undefined && parent?.dataset.probe !== undefined;
  const probe = element.dataset.probe;
  const role: Record<string, unknown> | null = probe ? (JSON.parse(probe) as Record<string, unknown>) : laidOut ? { role: "box" } : null;
  const children: FrameNode[] = [];
  if (!role) {
    for (const child of element.children) walk(child, origin, perLogical, into);
    return;
  }
  const box = element.getBoundingClientRect();
  const bounds = {
    x: (box.left - origin.left) / perLogical,
    y: (box.top - origin.top) / perLogical,
    width: box.width / perLogical,
    height: box.height / perLogical,
  };
  if (role.role !== "icon") {
    for (const child of element.children) walk(child, origin, perLogical, children);
  }
  if (role.role === "text") {
    role.content = element.textContent ?? "";
    const natural = Number(element.style.getPropertyValue("--natural"));
    const lines = Math.max(1, Math.round(box.height / perLogical / lineOf(element)));
    role.natural = { width: natural, height: lines * lineOf(element) };
  }
  let kept = children;
  while (kept.length === 1 && kept[0]!.role === "box" && same(kept[0]!.bounds, bounds)) kept = kept[0]!.children;
  if (role.role === "box" && kept.length === 1 && same(kept[0]!.bounds, bounds)) {
    into.push(kept[0]!);
    return;
  }
  into.push({ ...role, role: String(role.role), bounds, children: kept });
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
  const children: FrameNode[] = [];
  for (const child of page.children) walk(child, origin, perLogical, children);
  return {
    page: slug,
    mode,
    scale,
    root: { role: "box", bounds: { x: 0, y: 0, width: page.offsetWidth / scale, height: page.offsetHeight / scale }, children },
  };
}
