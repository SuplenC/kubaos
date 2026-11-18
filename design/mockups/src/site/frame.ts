// The page's geometry frame: every mocked element's probe, with its box in logical pixels from the
// page's top left, as the QML specimen reports its items (`qml/specimen/main.qml`). An element that
// says nothing is left out and its children kept, but a layout's child that says nothing is a
// plain box, since the layout's rules hold its edges, and a plain box around one child with its
// bounds is that child, as iced reports them.
//
// The frame is what a person sees, so the walk reports what the DOM alone doesn't say:
//
// - A box drawn scaled (CSS `scale`, or a `transform` that scales) is a picture of UI, such as
//   the Gallery's small screens or windows stepped back behind a lock: it and everything in it
//   are reported at the size they are laid out at, from its own top left on a whole pixel, marked
//   `picture` with the ratio it is drawn at.
// - What isn't drawn is left out: a box at no opacity or hidden, and one parked wholly outside a
//   box that cuts it off (`overflow`, `clip-path`) when the cut is on purpose: the box it is cut by
//   is a part shut to nothing, or it was moved or placed out of the flow to be there. A layout's child
//   left out keeps its place as a plain box, so its siblings' gaps still add up. Content a layout
//   pushed past its box is still reported, and sticks out of it.
// - A row whose content is changing, rooms opening and closing in it with their gaps (a pill's
//   content mid-change), is a plain box until they settle: its gap isn't held, its boxes are.
// - A box out of the flow (`position: absolute`) is marked `over`, so its parent's layout leaves
//   it out; one its probe says floats (a hint's bubble) is marked `floats` too, so it is held to
//   its own boxes, not its control's.
// - A box whose content scrolls past it is a `scroll`: a well's rows' says so (`scrolls` in
//   `lib/probe.ts`), and a box that says nothing and scrolls is one too. A probed box that scrolls its own content (a
//   dialog's middle) is reported inside a scroll, as tall as its content. A box the site lets
//   scroll sideways (a screen wider than the page under `?all`) is a `scroll` marked `across`.
//
// - A marquee's words are reported at the start of their pass, where it holds, whenever the frame
//   is taken, so every mode and scale report the same frame.
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

/** A box that cuts off what it holds, as drawn in the viewport. */
interface Clip {
  readonly element: Element;
  readonly rect: DOMRectReadOnly;
  /** Shut to nothing on an axis: a closed sheet, a shut section. */
  readonly shut: boolean;
  /**
   * It is a part's own box, or inside one: shut, it is a part closed on purpose. A box of the page
   * shut to nothing hides what it holds by fault, and its content is still reported.
   */
  readonly part: boolean;
  /** What is under it here was moved or placed out of the flow on its way: parked on purpose. */
  readonly moved: boolean;
}

interface Walk {
  readonly space: Space;
  /** Inside a probed component. */
  readonly part: boolean;
  readonly clips: readonly Clip[];
  /** Inside an element that says nothing and is out of the flow: what it holds is over its parent too. */
  readonly over: boolean;
  /** The CSS pixels a logical pixel takes, outside any picture. */
  readonly perCss: number;
}

/** How near two lengths are, in CSS pixels, to count as one: Chromium lays out in 1/64 px. */
const NEAR = 1 / 64;

function same(a: Bounds, b: Bounds): boolean {
  return a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height;
}

/** `element`'s nodes, appended to `into`. */
function walk(element: Element, at: Walk, into: FrameNode[]): void {
  if (!(element instanceof HTMLElement || element instanceof SVGElement)) return;
  const style = getComputedStyle(element);
  if (style.display === "none") return;
  const parent = element.parentElement;
  const out = style.position === "absolute" || style.position === "fixed";
  const laidOut = !out && parent?.dataset.layout !== undefined && parent?.dataset.probe !== undefined;
  const drawn = element.getBoundingClientRect();
  const matrix = element instanceof HTMLElement ? transformOf(style) : null;
  const moved = out || (matrix !== null && displaced(style, matrix));
  // Scaled to nothing, as a mark before it pops in, it is drawn nowhere.
  const shrunk = matrix !== null && (pictureOf(matrix)?.ratio ?? 1) < 1e-4;
  const hidden = unseen(element, style, drawn, at.clips, moved, shrunk);
  if (hidden) {
    // A layout's child that isn't drawn still takes its place in the layout.
    if (laidOut) into.push({ role: "box", unseen: hidden, bounds: at.space(drawn), children: [] });
    return;
  }
  const probe = element.dataset.probe;
  const declared = probe ? (JSON.parse(probe) as Record<string, unknown>) : element.dataset.scrollProbe !== undefined ? { role: "scroll" } : null;
  const picture = matrix ? pictureOf(matrix) : null;
  const scroller = sideways(element, style);
  const scrollsDown = downward(element, style);
  let role: Record<string, unknown> | null = declared ?? (laidOut || picture ? { role: "box" } : scroller || scrollsDown ? { role: "scroll" } : null);
  const part = at.part || declared?.role === "component";
  const clips = holding(element, style, drawn, at.clips, moved, part);
  if (!role) {
    for (const child of element.children) walk(child, { ...at, clips, over: at.over || out }, into);
    return;
  }
  role = { ...role };
  // A row whose content is changing (a pill's, `lib/pill.ts`) has rooms opening and closing in it,
  // their gaps with them: a box until they settle, its boxes held to every rule but its gap.
  const changing = role.role === "layout" && element.querySelector(":scope > .k-pill-slot") !== null;
  if (changing) role = { role: "box" };
  const { floats, ...said } = role;
  role = said;
  if (out || at.over) role.over = true;
  if (out && floats === true) role.floats = true;
  if (scroller && role.role === "scroll") role.across = true;
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
    for (const child of element.children) walk(child, { ...at, space, clips, part, over: false }, children);
  }
  if (role.role === "text") {
    role.content = element.textContent ?? "";
    role.natural = naturalOf(element, style, drawn, bounds, at.perCss);
  }
  // A layout drawn in reverse (a stacked button bar, the dismiss button at the bottom but first for
  // Tab) lists its children in the order they are drawn, as a toolkit's column would hold them.
  if (role.role === "layout" && style.flexDirection.endsWith("-reverse")) children.reverse();
  let kept = children;
  while (kept.length === 1 && kept[0]!.role === "box" && !marked(kept[0]!) && !mid.has(kept[0]!) && same(kept[0]!.bounds, bounds)) kept = kept[0]!.children;
  // A box whose one child is placed out of the flow over it (a select's surface in its room) is
  // what its layout lays out, so it stays: standing in for that child would leave the layout a gap.
  if (role.role === "box" && !marked(role) && kept.length === 1 && kept[0]!.over !== true && same(kept[0]!.bounds, bounds)) {
    into.push(kept[0]!);
    return;
  }
  const node: FrameNode = { ...role, role: String(role.role), bounds, children: kept };
  if (changing) mid.add(node);
  into.push((scrollsDown || scroller) && role.role !== "scroll" ? scrolledOf(element, style, node, scroller) : node);
}

/**
 * A box that scrolls its own content (a dialog's middle, a long menu, a layout's room for a sample
 * wider than it) as a scroll round it, as the toolkits report it: the scroll is the box drawn, and
 * the box inside it is as tall as its content, and as wide where it scrolls sideways (`across`),
 * from where its content starts.
 */
function scrolledOf(element: Element, style: CSSStyleDeclaration, node: FrameNode, across: boolean): FrameNode {
  const [width] = laidOutSize(style);
  const perCss = width > 0 ? node.bounds.width / width : 0;
  const sideways = across ? { x: node.bounds.x - element.scrollLeft * perCss, width: element.scrollWidth * perCss } : {};
  const inner = { ...node, bounds: { ...node.bounds, y: node.bounds.y - element.scrollTop * perCss, height: element.scrollHeight * perCss, ...sideways } };
  return { role: "scroll", ...(across ? { across: true } : {}), bounds: node.bounds, children: [inner] };
}

/** Whether a box says more than that it is a box: what folding it into its child would lose. */
/** Rows caught mid-change, reported as boxes: each stays one, never taken for its parent's room. */
const mid = new WeakSet<FrameNode>();

function marked(node: Record<string, unknown>): boolean {
  return Object.keys(node).some((key) => key !== "role" && key !== "bounds" && key !== "children");
}

/**
 * Why `element` isn't drawn, or null: no opacity, scaled to nothing, hidden, or parked wholly outside a box that cuts
 * it off. A box past a cut is only parked when the cut is a part shut, or it was moved there or placed
 * out of the flow (`moved`): content a layout pushed out of its box is a fault, and stays.
 */
function unseen(element: Element, style: CSSStyleDeclaration, drawn: DOMRectReadOnly, clips: readonly Clip[], moved: boolean, shrunk: boolean): string | null {
  if (Number(style.opacity) === 0) return "opacity";
  if (shrunk) return "shrunk";
  if (style.visibility !== "visible") return "hidden";
  for (const clip of cutting(element, style, clips)) {
    if (clip.shut && clip.part) return "shut";
    const c = clip.rect;
    const outside = drawn.right <= c.left + NEAR || drawn.left >= c.right - NEAR || drawn.bottom <= c.top + NEAR || drawn.top >= c.bottom - NEAR;
    if (outside && (clip.moved || moved)) return "parked";
  }
  return null;
}

/**
 * The clips that cut `element`: all of them for a box in the flow; for one placed absolutely, those
 * around the box it is placed in; none for a fixed one.
 */
function cutting(element: Element, style: CSSStyleDeclaration, clips: readonly Clip[]): readonly Clip[] {
  if (style.position === "fixed") return [];
  if (style.position !== "absolute" || !(element instanceof HTMLElement)) return clips;
  const placedIn = element.offsetParent;
  return placedIn ? clips.filter((clip) => clip.element.contains(placedIn)) : [];
}

/**
 * The clips what `element` holds is under: those around it, marked moved where it was moved or
 * placed out of the flow, then its own when it cuts off what it holds.
 */
function holding(element: Element, style: CSSStyleDeclaration, drawn: DOMRectReadOnly, clips: readonly Clip[], moved: boolean, part: boolean): readonly Clip[] {
  const cuts = cutting(element, style, clips);
  const under = moved ? clips.map((clip) => (cuts.includes(clip) ? { ...clip, moved: true } : clip)) : clips;
  const cutsOff =
    style.overflowX !== "visible" || style.overflowY !== "visible" || style.clipPath !== "none" || /paint|strict|content/.test(style.contain);
  if (!cutsOff) return under;
  return [...under, { element, rect: drawn, shut: drawn.width < NEAR || drawn.height < NEAR, part, moved: false }];
}

/**
 * Whether `element`, saying nothing, is a box the site lets scroll sideways and whose content runs
 * past it: a screen wider than the page, under `?all`.
 */
function sideways(element: Element, style: CSSStyleDeclaration): boolean {
  return element.getAttribute("data-probe") === null && /auto|scroll/.test(style.overflowX) && element.scrollWidth > element.clientWidth + 1;
}

/**
 * Whether `element` scrolls its content down and the content runs past it: a list's well, a long
 * dialog, a menu taller than its room. One whose probe already says it is a scroll needn't.
 */
function downward(element: Element, style: CSSStyleDeclaration): boolean {
  return /auto|scroll/.test(style.overflowY) && element.scrollHeight > element.clientHeight + 1;
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

/** Whether `m` draws its box somewhere other than where it is laid out. */
function displaced(style: CSSStyleDeclaration, m: DOMMatrix): boolean {
  const [width, height] = laidOutSize(style);
  const corners = [new DOMPoint(0, 0), new DOMPoint(width, 0), new DOMPoint(0, height), new DOMPoint(width, height)].map((p) => m.transformPoint(p));
  const xs = corners.map((p) => p.x);
  const ys = corners.map((p) => p.y);
  return [Math.min(...xs), Math.min(...ys), Math.max(...xs) - width, Math.max(...ys) - height].some((v) => Math.abs(v) > NEAR);
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

/**
 * The room a text needs, in logical pixels. A line of words in its type style needs its words'
 * advance (`--natural`, measured as iced measures it) and its line. Words that break onto lines
 * need their widest line and every line they break onto, as they are laid out in the room they
 * have. Words whose size the CSS sets (a lock's clock, sized from its screen) need their advance
 * grown to that size, and the line the CSS gives them.
 */
function naturalOf(element: HTMLElement | SVGElement, style: CSSStyleDeclaration, drawn: DOMRectReadOnly, bounds: Bounds, perCss: number): { width: number; height: number } {
  const line = lineOf(element);
  const advance = Number(element.style.getPropertyValue("--natural"));
  const breaks = element.matches("[data-wrapped], [data-clamped='lines']");
  const grown = grownByCss(element, style, perCss);
  if (!breaks) return { width: advance * grown, height: grown === 1 ? Math.max(1, Math.round(bounds.height / line)) * line : bounds.height };
  const perViewport = drawn.width > 0 ? bounds.width / drawn.width : 0;
  const rows = linesOf(element);
  const widest = rows.reduce((most, row) => Math.max(most, (row.right - drawn.left) * perViewport), 0);
  return { width: widest, height: Math.max(1, rows.length) * line };
}

/**
 * How many times its type style's size the CSS draws `element`'s words at: 1, or the size the CSS
 * gives them over the style's, which `text()` notes on it (`data-size`, in logical pixels).
 */
function grownByCss(element: HTMLElement | SVGElement, style: CSSStyleDeclaration, perCss: number): number {
  const own = Number(element.dataset.size);
  if (!own) return 1;
  const drawnAt = Number.parseFloat(style.fontSize) / perCss;
  return Math.abs(drawnAt - own) > own * 1e-3 ? drawnAt / own : 1;
}

/** The lines `element`'s words are laid out on, as drawn: each line's extent, left to right. */
function linesOf(element: Element): { left: number; right: number }[] {
  const range = document.createRange();
  range.selectNodeContents(element);
  const rows = new Map<number, { left: number; right: number }>();
  for (const rect of range.getClientRects()) {
    if (rect.width === 0) continue;
    const key = Math.round(rect.top + rect.height / 2);
    const known = rows.get(key);
    rows.set(key, known ? { left: Math.min(known.left, rect.left), right: Math.max(known.right, rect.right) } : { left: rect.left, right: rect.right });
  }
  return [...rows.values()];
}

/** A text's line height in logical pixels. */
function lineOf(element: Element): number {
  const line = Number.parseFloat(getComputedStyle(element).lineHeight);
  const scale = Number(getComputedStyle(element).getPropertyValue("--scale")) || 1;
  return line / scale;
}

/** The frame of `page`, the page's box, drawn at `scale` in `mode`. */
/**
 * Puts every marquee's pass back at its start, where it holds, for the walk, and returns how to put
 * them back where they were. A pass loops for as long as the page shows, so where its words stand
 * is only when the frame was taken: at the start, every mode and scale report the same frame.
 */
function atStart(page: HTMLElement): () => void {
  const passes = page
    .getAnimations({ subtree: true })
    .filter((animation) => animation.effect instanceof KeyframeEffect && animation.effect.target instanceof HTMLElement && animation.effect.target.classList.contains("k-marquee"));
  const were = passes.map((animation) => animation.currentTime);
  for (const animation of passes) animation.currentTime = 0;
  return () => passes.forEach((animation, index) => (animation.currentTime = were[index] ?? 0));
}

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
  const at: Walk = { space, part: false, clips: [], over: false, perCss: scale };
  const passes = atStart(page);
  for (const child of page.children) walk(child, at, children);
  passes();
  return {
    page: slug,
    mode,
    scale,
    root: { role: "box", bounds: { x: 0, y: 0, width: page.offsetWidth / scale, height: page.offsetHeight / scale }, children },
  };
}
