// The Overview, open on every screen at once: each screen shows the workspace groups it holds as a
// grid of live miniatures (its own group, and on the primary the groups waiting there, each a
// quieter named section under it), and a window or a whole workspace is carried by hand onto
// another workspace, between two, onto a group's new-workspace slot, or across a screen's edge onto
// another screen's overview. A DRAFT part: the shell has no part like it yet.
//
// Every workspace and every window is one element for its whole life. A change moves it to its new
// place by its custom properties, which glide (`overview.css`); nothing is drawn again. The model
// (`overview-model.ts`) says where everything is; `place` works out each screen's grid from it and
// from what a drag is showing, and sets every element's place. Only the thing carried changes its
// parent: it rides in the world's lift layer while held, over every screen, and goes into its new
// workspace or screen once it has landed.
//
// Places are in the world's logical pixels: a screen's own in the screen view, the desk's in the
// whole-desk view, where every screen stands as it does on the desk and the world is scaled to fit.

import "./overview.css";
import { tipLine } from "../../lib/banner.ts";
import { focusRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { type Rolling, rollingNumber } from "../../lib/rolling-number.ts";
import { text } from "../../lib/text.ts";
import { length, number, raw } from "../../lib/tokens.ts";
import { eased } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";
import { barScreen } from "./bar-parts.ts";
import { face } from "./overview-apps.ts";
import * as model from "./overview-model.ts";
import type { Group as BarGroup } from "./workspaces.ts";

/** The focused screen at its true size, or every screen side by side as they stand. */
export type View = "screen" | "desk";

export interface Point {
  readonly x: number;
  readonly y: number;
}

/** A miniature's place on its screen: its box, and its scale from the screen's own size. */
interface Slot extends model.Rect {
  readonly s: number;
}

/** Where a carried thing would land, as the pointer finds it. */
export type Target =
  | { readonly kind: "space"; readonly space: string; readonly at?: model.Place }
  | { readonly kind: "between"; readonly group: string; readonly index: number }
  | { readonly kind: "plus"; readonly group: string }
  | { readonly kind: "refused"; readonly group: string }
  /** The delete slot's reach: let go there, it waits in the slot, a few seconds to take it back out. */
  | { readonly kind: "bin" };

/** A still state drawn on the overview for a sample: the same moves a hand makes, finished. */
export type Pin =
  | { readonly kind: "lit"; readonly space: string }
  | { readonly kind: "window"; readonly window: string; readonly to: Target; readonly crossing?: number }
  | { readonly kind: "space"; readonly space: string; readonly to: Target }
  /** A window or a workspace carried toward the delete slot: `over`, pulled in with the pointer on
   * the slot; `edge`, pulled in with the pointer just inside the reach; neither, on its way. */
  | { readonly kind: "binning"; readonly what: Carried; readonly over: boolean | "edge" }
  /**
   * Something waiting in the delete slot, `left` of its time left; with `next`, a second thing carried
   * into the slot meanwhile, or, `confirmed`, a first one deleted as `what` was let go there.
   */
  | { readonly kind: "pending"; readonly what: Carried; readonly left: number; readonly next?: Carried; readonly confirmed?: Carried }
  /** A window carried onto `to` and let go there: where it lands. */
  | { readonly kind: "placed"; readonly window: string; readonly to: Target }
  /** Three fingers on the touchpad, the Overview `progress` of the way open: opening from closed, or closing. */
  | { readonly kind: "swiping"; readonly opening: boolean; readonly progress: number }
  /** A thing dropped in the slot, then taken back out and carried half way home. */
  | { readonly kind: "picked"; readonly what: Carried };

/** A drag the tip teaches: a window's with the left button, a workspace's with the right. */
export type Learned = "window" | "space";

/** A window or a workspace, by its id. */
export type Carried = { readonly window: string } | { readonly space: string };

export interface OverviewOptions {
  readonly desk: model.DeskName;
  readonly fill?: model.Fill;
  readonly view: View;
  /** Drawn closed: the desktop it opens from. */
  readonly closed?: boolean;
  /** The screen the keyboard is on, when not the primary. */
  readonly focus?: string;
  readonly pin?: Pin;
  /**
   * The drags the person has already made with the pointer, a window's and a workspace's: once both,
   * the tip that teaches them is gone. The build keeps this in `~/.local/state/kubaos/overview.toml`.
   */
  readonly learned?: readonly Learned[];
  /** The delete slot's reach drawn round it, dashed: a sample's annotation, not the Overview's. */
  readonly reach?: boolean;
  /** Groups renamed, by id: a long name's sample. */
  readonly names?: Readonly<Record<string, string>>;
}

/** What a script driving the stage reaches: the world, and the hand's moves in world points. */
export interface Driver {
  readonly desk: () => model.Desk;
  readonly snapshot: () => ReturnType<typeof model.snapshot>;
  /** A point's place in the browser's viewport, for a real pointer to go to. */
  readonly client: (point: Point) => Point;
  /** The middle of a workspace's miniature, a window, a group's new-workspace slot, in world points. */
  readonly centreOf: (what: { space: string } | { window: string } | { plus: string }) => Point;
  /** Where a workspace is taken by its number, under its miniature, in world points. */
  readonly handleOf: (space: string) => Point;
  /** The gutter's middle before place `index` of `group`. */
  readonly between: (group: string, index: number) => Point;
  readonly target: () => Target | null;
  readonly lifted: () => string | null;
  readonly isOpen: () => boolean;
  /** Every element's place the overview is gliding it to, by its id: a script holds the glide to it. */
  readonly element: (id: string) => HTMLElement | null;
  /**
   * Holds every move running in the world at `at` milliseconds after it started: a capture's frame.
   * `commit` writes the moment into the elements and ends the moves, so a page clock can't run them on.
   */
  readonly freeze: (at: number, commit?: boolean) => void;
  /** Lets go of the frozen moves, finished. */
  readonly thaw: () => void;
  /** The middle of the delete slot where it stands, or would stand on the focused screen. */
  readonly bin: () => Point;
  /** A point over `side` of `window` on `space`, in world points, and the order a workspace tiles its windows in. */
  readonly sidePoint: (space: string, window: string, side: model.Side) => Point;
  readonly order: (space: string) => string[];
  /** A workspace's number as drawn. */
  readonly numberShown: (space: string) => string;
  /** Three fingers holding the Overview `p` of the way open, and lifting. */
  readonly swipe: (p: number) => void;
  readonly lift: () => void;
  /** The drags learned, and whether the tip still shows. */
  readonly learned: () => Learned[];
  readonly tipShown: () => boolean;
  /** What waits in the delete slot, or null. */
  readonly pending: () => Carried | null;
  /** Draws the overview again with other options: another desk, view or fill. */
  readonly redraw: (options: Partial<OverviewOptions>) => void;
}

// ── Measures ──────────────────────────────────────────────────────────────────────────────────

const L = (token: string): number => length(token);

/** A logical length rounded to whole device pixels at the scale picked. */
function device(value: number): number {
  const scale = settings().scale;
  return Math.round(value * scale) / scale;
}

const css = (value: number): string => `calc(${device(value)} * var(--px))`;

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** The room inside a screen's frame and Hyprland's gaps that its windows tile, as `looks.lua` sets them. */
function tilingArea(width: number, height: number): model.Rect {
  const side = L("space-s8") + L("space-s8");
  const top = L("size-bar") + L("space-s8");
  return { x: side, y: top, w: width - 2 * side, h: height - top - side };
}

// ── The grid ──────────────────────────────────────────────────────────────────────────────────

/** A place in a section held open for a workspace being carried there. */
const ROOM = "\u0000room";

interface SectionSpec {
  readonly group: string;
  readonly items: readonly string[];
  readonly plus: boolean;
  readonly waiting: boolean;
}

interface Section {
  readonly group: string;
  readonly waiting: boolean;
  /** Every place in reading order: the workspaces (and a room), then the new-workspace slot. */
  readonly places: readonly Slot[];
  readonly items: readonly string[];
  readonly plus: Slot | null;
  readonly label: Point | null;
  /** Its whole box, its name included. */
  readonly box: model.Rect;
  /** The row each place is on. */
  readonly rows: readonly number[];
}

interface Layout {
  readonly sections: readonly Section[];
  readonly spaces: Map<string, Slot>;
  readonly room: Slot | null;
  /** The size a miniature of the screen's own group is drawn at. */
  readonly scale: number;
}

/**
 * Lays out a screen `width` by `height` logical pixels: its own group's miniatures as large as
 * they fit, never larger than a three by three grid of the screen would draw them, a waiting
 * group's at half their size, every row centred and the whole centred on the screen.
 */
function arrange(width: number, height: number, specs: readonly SectionSpec[]): Layout {
  const margin = L("space-s48");
  const gap = L("space-s24");
  const tag = L("space-s8") + L("size-control_sm");
  const apart = L("space-s32");
  const label = L("size-chip") + L("space-s8");
  const aspect = height / width;
  const roomW = width - 2 * margin;
  const roomH = height - 2 * margin;
  const counts = specs.map((spec) => spec.items.length + (spec.plus ? 1 : 0));

  interface Plan {
    readonly tile: number;
    readonly cols: number;
    readonly rows: number;
  }
  const plan = (w: number): Plan[] =>
    specs.map((spec, i) => {
      const tile = spec.waiting ? w / 2 : w;
      const most = Math.max(1, Math.floor((roomW + gap) / (tile + gap)));
      const rows = Math.ceil(counts[i]! / most);
      return { tile, rows, cols: Math.ceil(counts[i]! / rows) };
    });
  const tall = (plans: readonly Plan[]): number =>
    plans.reduce((sum, each, i) => sum + (specs[i]!.waiting ? label : 0) + each.rows * (each.tile * aspect + tag) + (each.rows - 1) * gap, 0) + (plans.length - 1) * apart;

  const cap = Math.min((roomW - 2 * gap) / 3, ((roomH - 2 * gap) / 3 - tag) / aspect);
  let low = 1;
  let high = Math.max(low, cap);
  if (tall(plan(high)) > roomH) {
    for (let step = 0; step < 32; step++) {
      const middle = (low + high) / 2;
      if (tall(plan(middle)) <= roomH) low = middle;
      else high = middle;
    }
    high = low;
  }
  const plans = plan(Math.floor(high));
  let y = margin + (roomH - tall(plans)) / 2;
  const spaces = new Map<string, Slot>();
  let room: Slot | null = null;
  const sections = specs.map((spec, i): Section => {
    const { tile, cols } = plans[i]!;
    const th = device(tile * aspect);
    const tw = device(tile);
    const top = y;
    let at: Point | null = null;
    if (spec.waiting) {
      at = { x: 0, y: device(y) };
      y += label;
    }
    const places: Slot[] = [];
    const rows: number[] = [];
    let left = Number.POSITIVE_INFINITY;
    for (let start = 0, row = 0; start < counts[i]!; start += cols, row++) {
      const inRow = Math.min(cols, counts[i]! - start);
      const x0 = margin + (roomW - inRow * tw - (inRow - 1) * gap) / 2;
      left = Math.min(left, x0);
      for (let k = 0; k < inRow; k++) {
        places.push({ x: device(x0 + k * (tw + gap)), y: device(y), w: tw, h: th, s: tw / width });
        rows.push(row);
      }
      y += th + tag + gap;
    }
    y += apart - gap;
    spec.items.forEach((id, k) => {
      if (id === ROOM) room = places[k]!;
      else spaces.set(id, places[k]!);
    });
    const plus = spec.plus ? places.at(-1)! : null;
    return {
      group: spec.group,
      waiting: spec.waiting,
      places,
      items: spec.items,
      plus,
      rows,
      label: at ? { x: device(left), y: at.y } : null,
      box: { x: margin, y: top, w: roomW, h: y - apart - top },
    };
  });
  return { sections, spaces, room, scale: (plans[0]?.tile ?? 0) / width };
}

// ── Elements ──────────────────────────────────────────────────────────────────────────────────

function span(name: string, ...children: (Node | string)[]): HTMLElement {
  const element = document.createElement("span");
  element.className = name;
  element.append(...children);
  return element;
}

/** A workspace's elements: its box, the desktop scaled inside it, and its number under it. */
interface SpaceEl {
  readonly root: HTMLElement;
  readonly desk: HTMLElement;
  readonly tag: HTMLElement;
  readonly number: HTMLElement;
  /** Its number as it rolls when the workspaces before it change, and whether it is the shown one's. */
  rolling: Rolling | null;
  shown: boolean;
}

/** A screen drawn: the desktop with its frame and bar, and the overview's layer over it. */
interface ScreenView {
  readonly screen: model.Screen;
  size: { w: number; h: number };
  at: Point;
  readonly element: HTMLElement;
  readonly layer: HTMLElement;
  readonly bar: HTMLElement;
  /** What blurs and dims as it opens: the wallpaper. What steps aside: the frame's glass, the frame, the bar. */
  readonly veiled: readonly HTMLElement[];
  readonly aside: readonly HTMLElement[];
  layout: Layout;
}

type Motion = "none" | "reflow" | "open" | "close" | "land" | "arrive";

/** Each motion's duration and curve for places and sizes, by token. */
const GLIDES: Record<Exclude<Motion, "none">, readonly [string, string]> = {
  reflow: ["medium", "standard_decel"],
  open: ["long", "emphasized_decel"],
  close: ["long", "emphasized_decel"],
  land: ["medium", "emphasized_decel"],
  arrive: ["medium", "emphasized_decel"],
};

/** Sets an element's glide for its next change of place: its timing, or none. */
function glide(element: HTMLElement, motion: Motion, delay = "0s"): void {
  if (motion === "none" || settings().reduced) {
    element.style.setProperty("--ov-pt", "0s");
    element.style.setProperty("--ov-pd", "0s");
    return;
  }
  const [duration, curve] = GLIDES[motion];
  element.style.setProperty("--ov-pt", `var(--motion-${duration})`);
  element.style.setProperty("--ov-pe", `var(--easing-${curve})`);
  element.style.setProperty("--ov-pd", delay);
}

/** Puts every style change so far on screen, so the next change transitions from it. */
function commit(element: Element): void {
  void getComputedStyle(element).opacity;
}

// ── The overview ──────────────────────────────────────────────────────────────────────────────

export interface Overview {
  readonly element: HTMLElement;
  readonly driver: Driver;
}

export const DESKS: readonly { readonly desk: model.DeskName; readonly name: string }[] = [
  { desk: "three", name: "Three screens" },
  { desk: "laptop", name: "Laptop alone" },
  { desk: "ultrawide", name: "Ultrawide alone" },
];

export const VIEWS: readonly { readonly view: View; readonly name: string }[] = [
  { view: "screen", name: "This screen" },
  { view: "desk", name: "Whole desk" },
];

/**
 * The sample's own size as a length. HAZARD: written in another form than `css` writes the screens'
 * sizes inside it: a full screen or a capture re-sizes every property whose value is the sample's,
 * and a screen of the same width inside the whole desk would be re-sized with it.
 */
function rootLength(value: number): string {
  return `calc(var(--px) * ${value})`;
}

/** The focused screen's logical size, which the sample is drawn at. */
export function sizeOf(desk: model.DeskName, focus?: string): { width: number; height: number } {
  const made = model.sampleDesk(desk);
  const screen = made.screens.find((each) => each.id === (focus ?? made.focused))!;
  return { width: screen.width, height: screen.height };
}

/**
 * The overview, sized as a whole screen: open on the screens of `options.desk`, live, or drawn
 * still in a pinned state. The stage's switches draw it again on another desk or view.
 */
export function overview(options: OverviewOptions): Overview {
  const root = span("ov");
  const size = sizeOf(options.desk, options.focus);
  root.style.setProperty("--screen-width", rootLength(size.width));
  root.style.setProperty("--screen-height", rootLength(size.height));
  root.tabIndex = 0;
  let ours = root.style.getPropertyValue("--screen-width");
  let current = build(root, options);
  // A full screen, or a capture at a screen's size, lays the root out anew: the overview follows it.
  let seen = { w: 0, h: 0 };
  new ResizeObserver(() => {
    const scale = settings().scale;
    const now = { w: root.clientWidth / scale, h: root.clientHeight / scale };
    if (now.w === 0 || (Math.abs(now.w - seen.w) < 0.5 && Math.abs(now.h - seen.h) < 0.5)) return;
    const first = seen.w === 0;
    seen = now;
    if (first && Math.abs(now.w - size.width) < 0.5 && Math.abs(now.h - size.height) < 0.5) return;
    // A still sample is drawn again at the new size, its pin played on it afresh.
    if (options.pin) current = build(root, options, now);
    else current.resize(now.w, now.h);
  }).observe(root);
  const driver: Driver = new Proxy({} as Driver, { get: (_, key) => current.driver[key as keyof Driver] });
  (root as HTMLElement & { overview?: Driver }).overview = driver;

  function build(into: HTMLElement, made: OverviewOptions, at?: { w: number; h: number }): Built {
    into.replaceChildren();
    const built = engine(into, made, (next) => {
      // The sample takes the new desk's screen size, unless a full screen or a capture sized it.
      const resized = sizeOf(next.desk, next.focus);
      if (into.style.getPropertyValue("--screen-width") === ours) {
        ours = rootLength(resized.width);
        into.style.setProperty("--screen-width", ours);
        into.style.setProperty("--screen-height", rootLength(resized.height));
      }
      const scale = settings().scale;
      const now = { w: into.clientWidth / scale, h: into.clientHeight / scale };
      seen = now;
      current = build(into, next, now.w > 0 ? now : undefined);
    }, at);
    return built;
  }
  return { element: root, driver };
}

interface Built {
  readonly driver: Driver;
  readonly resize: (width: number, height: number) => void;
}

/** The overview's world drawn into `root`; `redraw` draws it again with other options (the switches). */
function engine(root: HTMLElement, options: OverviewOptions, redraw: (next: OverviewOptions) => void, drawnAt?: { w: number; h: number }): Built {
  const desk = model.sampleDesk(options.desk, options.fill);
  if (options.focus) desk.focused = options.focus;
  for (const [id, name] of Object.entries(options.names ?? {})) {
    const at = desk.groups.findIndex((group) => group.id === id);
    if (at >= 0) desk.groups[at] = { ...desk.groups[at]!, name };
  }
  const view = options.view;
  let open = !options.closed;
  const rootSize = sizeOf(options.desk, options.focus);
  let size = drawnAt ?? { w: rootSize.width, h: rootSize.height };

  const world = span("ov-world");
  const marks = span("ov-marks");
  const lift = span("ov-lift");
  // The keyboard's selection and a drop's target: a tube each, gliding from one miniature to the next.
  const select = span("ov-ring");
  focusRing(select, "control", "accent", false);
  select.dataset.pinned = "";
  const aim = span("ov-ring ov-aim");
  focusRing(aim, "control", "accent", false);
  aim.dataset.pinned = "";
  const room = span("ov-room");
  const slit = span("ov-slit");
  marks.append(room, slit, aim, select);

  // How both drags go, on the focused screen: a tip on the frame's glass in the bottom margin's left
  // end, under every miniature, the delete slot's corner left free. It shows every time until the
  // person has made both drags with the pointer, and then leaves for good. Each button is a phrase
  // of its own, which a mouse button's keycap can take the place of without the sentence changing.
  const tip = span("ov-tip", tipLine("Drag a window with the left button, a whole workspace with the right button."));
  const learned = new Set<Learned>(options.learned ?? []);
  tip.toggleAttribute("data-learned", learned.size === 2);
  marks.append(tip);

  /** A drag made with the pointer: learned; once both are, the tip leaves as the drag ends. */
  function learn(drag: Learned): void {
    learned.add(drag);
  }

  function tipLeaves(): void {
    if (learned.size === 2 && !tip.hasAttribute("data-learned")) tip.dataset.learned = "";
  }

  const views: ScreenView[] = [];
  const spaceEls = new Map<string, SpaceEl>();
  const winEls = new Map<string, HTMLElement>();
  const plusEls = new Map<string, HTMLElement>();
  const labelEls = new Map<string, HTMLElement>();
  /** Each window's tile on its workspace's desktop, in the screen's logical pixels. */
  const tiles = new Map<string, model.Rect>();
  const landing = span("ov-landing");

  // ── The screens ──

  function screensDrawn(): model.Screen[] {
    return view === "screen" ? [desk.screens.find((each) => each.id === desk.focused)!] : [...desk.screens];
  }

  /** The bar's groups for `screen`, as `workspaces.ts` draws them. */
  function barGroups(screen: model.Screen): { groups: BarGroup[]; active: number } {
    let active = 1;
    const groups = model.groupsOn(desk, screen).map((group, block): BarGroup => {
      const spaces = group.spaces.map((id, i) => {
        if (desk.shown.get(screen.id) === id) active = block * 10 + i + 1;
        return { id: block * 10 + i + 1, occupied: (desk.spaces.get(id)?.windows.length ?? 0) > 0 };
      });
      if (spaces.length < model.MOST) spaces.push({ id: block * 10 + spaces.length + 1, occupied: false });
      return model.waits(desk, group) ? { block, borrowed: true, name: group.name, spaces } : { block, spaces };
    });
    return { groups, active };
  }

  function barRow(screen: model.Screen, width: number): Element[] {
    const drawn = barScreen(width, "0px", { workspaces: barGroups(screen) });
    return [...drawn.element.querySelector(".k-bar")!.children];
  }

  function makeView(screen: model.Screen): ScreenView {
    const w = view === "screen" ? size.w : screen.width;
    const h = view === "screen" ? size.h : screen.height;
    const drawn = barScreen(w, css(h), { workspaces: barGroups(screen) });
    const element = drawn.element;
    element.classList.add("ov-screen");
    element.dataset.screen = screen.id;
    const layer = span("ov-layer");
    element.querySelector(".k-screen-dim")!.after(layer);
    const bar = element.querySelector<HTMLElement>(".k-bar")!;
    bar.addEventListener("click", (event) => {
      if (!open && event.target instanceof Element && event.target.closest(".shell-ws")) setOpen(true);
    });
    const veiled = [element.querySelector<HTMLElement>(".k-screen-wallpaper")!];
    const aside = [element.querySelector<HTMLElement>(".k-screen-glass")!, element.querySelector<HTMLElement>(".k-frame")!, bar];
    return { screen, size: { w, h }, at: { x: 0, y: 0 }, element, layer, bar, veiled, aside, layout: arrange(w, h, []) };
  }

  for (const screen of screensDrawn()) views.push(makeView(screen));
  world.append(...views.map((each) => each.element), marks, lift);

  /** The world's own size, and how the views stand in it. */
  let worldSize = { w: 0, h: 0 };
  function standViews(): void {
    if (view === "screen") {
      const only = views[0]!;
      only.size = { ...size };
      only.at = { x: 0, y: 0 };
      worldSize = { ...size };
      world.style.removeProperty("scale");
    } else {
      const left = Math.min(...views.map((each) => each.screen.x));
      const top = Math.min(...views.map((each) => each.screen.y));
      for (const each of views) each.at = { x: each.screen.x - left, y: each.screen.y - top };
      worldSize = {
        w: Math.max(...views.map((each) => each.at.x + each.size.w)),
        h: Math.max(...views.map((each) => each.at.y + each.size.h)),
      };
      // The desk scaled to fit the sample with a margin round it, centred: a picture of the desk.
      const margin = L("space-s48");
      const fit = Math.min((size.w - 2 * margin) / worldSize.w, (size.h - 2 * margin) / worldSize.h);
      world.style.setProperty("--ov-fit", String(fit));
      world.style.setProperty("--ov-left", css((size.w - worldSize.w * fit) / 2));
      world.style.setProperty("--ov-top", css((size.h - worldSize.h * fit) / 2));
    }
    world.style.setProperty("--ov-world-w", css(worldSize.w));
    world.style.setProperty("--ov-world-h", css(worldSize.h));
    for (const each of views) {
      each.element.style.setProperty("--screen-width", css(each.size.w));
      each.element.style.setProperty("--screen-height", css(each.size.h));
      each.element.style.setProperty("--ov-at-x", css(each.at.x));
      each.element.style.setProperty("--ov-at-y", css(each.at.y));
    }
  }
  standViews();
  world.dataset.view = view;

  function viewOf(screenId: string): ScreenView | undefined {
    return views.find((each) => each.screen.id === screenId);
  }

  function viewOfSpace(space: string): ScreenView | undefined {
    return viewOf(model.screenOf(desk, space).id);
  }

  // ── Elements of the world ──

  function windowEl(id: string): HTMLElement {
    let element = winEls.get(id);
    if (element) return element;
    const win = desk.windows.get(id)!;
    element = span("ov-win", face(win));
    element.dataset.window = id;
    element.dataset.app = win.app;
    winEls.set(id, element);
    return element;
  }

  function spaceEl(id: string): SpaceEl {
    let made = spaceEls.get(id);
    if (made) return made;
    const desktop = span("ov-ws-desk");
    const picture = document.createElement("img");
    picture.className = "ov-ws-wallpaper";
    picture.src = views[0]!.element.querySelector<HTMLImageElement>(".k-screen-wallpaper")!.src;
    picture.alt = "";
    picture.draggable = false;
    desktop.append(picture);
    const number = span("ov-ws-number");
    const tag = span("ov-ws-tag", number);
    const element = span("ov-ws", desktop, tag);
    element.dataset.space = id;
    made = { root: element, desk: desktop, tag, number, rolling: null, shown: false };
    spaceEls.set(id, made);
    return made;
  }

  function plusEl(group: string): HTMLElement {
    let element = plusEls.get(group);
    if (element) return element;
    element = span("ov-plus", icon("add", "var(--icon-s24)", "var(--color-outline)"));
    element.dataset.group = group;
    plusEls.set(group, element);
    return element;
  }

  function labelEl(group: model.Group): HTMLElement {
    let element = labelEls.get(group.id);
    if (element) return element;
    element = span("ov-label", text(group.name, "label", "secondary"));
    element.dataset.group = group.id;
    labelEls.set(group.id, element);
    return element;
  }

  /**
   * Its number as the bar draws it: rolled to its new value as the workspaces before it change, as a
   * gap closing moves every later one down; drawn afresh when it becomes or stops being the shown one.
   */
  function dressTag(id: string, made: SpaceEl, screen: model.Screen): void {
    const shown = desk.shown.get(screen.id) === id;
    const value = model.numberOf(desk, id);
    if (made.rolling && made.shown === shown) {
      made.rolling.set(value);
      return;
    }
    made.rolling = rollingNumber({ value, widest: String(value).replace(/\d/g, "0"), style: "button_small", color: shown ? "on_primary" : "on_surface" });
    made.shown = shown;
    made.number.replaceChildren(made.rolling.element);
    made.number.toggleAttribute("data-shown", shown);
  }

  // ── Where everything goes ──

  /** What a carry is showing now: the thing carried, where it would land. */
  interface Carry {
    readonly kind: "window" | "space";
    readonly id: string;
    readonly element: HTMLElement;
    /** The point held, in the thing's own unscaled pixels. */
    readonly grab: Point;
    /** Where it was lifted from: its workspace (a window's) or group and place (a workspace's). */
    readonly from: string;
    readonly fromIndex: number;
    /** Its scale where it was lifted: a window's workspace's, a workspace's own. */
    readonly scale: number;
    /** Its own box, in the same pixels as `grab`: a window's tile, a workspace's miniature. */
    readonly size: { readonly w: number; readonly h: number };
    target: Target | null;
    point: Point;
    /** Each screen's grid without the workspace carried, from which a carried workspace's place is found. */
    readonly bare: Map<string, Layout>;
  }
  let carry: Carry | null = null;

  /** The sections `screen` shows: a carried workspace taken out, and a room held where it would go. */
  function specsOf(screen: model.Screen, roomAt?: { group: string; index: number }): SectionSpec[] {
    return model.groupsOn(desk, screen).map((group) => {
      let items = group.spaces.filter((id) => !(carry?.kind === "space" && carry.id === id));
      if (roomAt && roomAt.group === group.id) items = [...items.slice(0, roomAt.index), ROOM, ...items.slice(roomAt.index)];
      return { group: group.id, items, plus: items.length < model.MOST, waiting: model.waits(desk, group) };
    });
  }

  /** The windows a workspace tiles now: one carried away left out, one carried over it added last. */
  function windowsShown(space: string): string[] {
    return [...tilesShown(space, { x: 0, y: 0, w: 1, h: 1 }).keys()];
  }

  /**
   * How a workspace tiles now, in `area`: a carried window out of the one it came from, and in the
   * one it is over, at the side of the window under the pointer, the others making room.
   */
  function tilesShown(space: string, area: model.Rect): Map<string, model.Rect> {
    let tree = model.treeOf(desk.spaces.get(space)!);
    // Lifted and not yet moved off, it still stands in its own place.
    const home = carry?.kind === "window" && carry.target?.kind === "space" && carry.target.space === space && !carry.target.at && space === carry.from;
    if (carry?.kind === "window" && !home) {
      tree = model.remove(tree, carry.id);
      if (carry.target?.kind === "space" && carry.target.space === space) tree = model.insert(tree, carry.id, carry.target.at);
    }
    return model.layout(tree, area, L("space-s8"));
  }

  /** Lays every screen out again and sends each element to its place, moving as `motion` says. */
  function place(motion: Motion): void {
    const roomAt = carry?.kind === "space" && carry.target && carry.target.kind !== "refused" && carry.target.kind !== "space" ? roomOf(carry.target) : undefined;
    for (const each of views) {
      each.layout = arrange(each.size.w, each.size.h, specsOf(each.screen, roomAt));
      const groups = model.groupsOn(desk, each.screen);
      for (const section of each.layout.sections) {
        const group = groups.find((g) => g.id === section.group)!;
        section.items.forEach((id, k) => {
          if (id !== ROOM) putSpace(id, section.places[k]!, each, motion);
        });
        const plus = plusEl(group.id);
        if (section.plus) {
          if (plus.parentElement !== each.layer) {
            each.layer.append(plus);
            setSlot(plus, section.plus, "none");
            commit(plus);
          }
          setSlot(plus, section.plus, motion);
          plus.dataset.shown = "";
        } else delete plus.dataset.shown;
        const label = labelEl(group);
        if (section.label) {
          if (label.parentElement !== each.layer) each.layer.append(label);
          glide(label, motion);
          label.style.setProperty("--ov-x", css(section.label.x));
          label.style.setProperty("--ov-y", css(section.label.y));
          label.dataset.shown = "";
        } else delete label.dataset.shown;
      }
      for (const [group, element] of plusEls) if (!groups.some((g) => g.id === group) && element.parentElement === each.layer) delete element.dataset.shown;
    }
    showRoom();
    showAim();
    showSelect();
    showTip();
  }

  function showTip(): void {
    const on = viewOf(desk.focused) ?? views[0]!;
    tip.style.setProperty("--ov-x", css(on.at.x + L("space-s48")));
    tip.style.setProperty("--ov-y", css(on.at.y + on.size.h - L("space-s48") / 2));
  }

  /** Sets a box's place and size, in its parent's logical pixels. */
  function setSlot(element: HTMLElement, slot: model.Rect, motion: Motion): void {
    glide(element, motion);
    element.style.setProperty("--ov-x", css(slot.x));
    element.style.setProperty("--ov-y", css(slot.y));
    element.style.setProperty("--ov-w", css(slot.w));
    element.style.setProperty("--ov-h", css(slot.h));
  }

  /** Sends workspace `id` to `slot` on `on`, and its windows to their tiles. */
  function putSpace(id: string, slot: Slot, on: ScreenView, motion: Motion): void {
    const made = spaceEl(id);
    const fresh = made.root.parentElement === null;
    const carried = made.root.hasAttribute("data-lifted") || made.root.hasAttribute("data-landing");
    if (made.root.parentElement !== on.layer && !carried) {
      on.layer.append(made.root);
      if (fresh) arriveFrom(made, slot, on);
    }
    if (carried) return;
    for (const part of [made.root, made.desk, made.tag]) glide(part, motion);
    made.root.style.setProperty("--ov-x", css(slot.x));
    made.root.style.setProperty("--ov-y", css(slot.y));
    made.root.style.setProperty("--ov-w", css(slot.w));
    made.root.style.setProperty("--ov-h", css(slot.h));
    made.desk.style.setProperty("--ov-s", String(slot.s));
    made.desk.style.setProperty("--ov-unscale", String(1 / slot.s));
    made.desk.style.setProperty("--ov-dw", css(on.size.w));
    made.desk.style.setProperty("--ov-dh", css(on.size.h));
    made.root.toggleAttribute("data-shown-here", desk.shown.get(on.screen.id) === id);
    dressTag(id, made, on.screen);
    tileSpace(id, on, motion);
  }

  /** A workspace made by a drop grows in from nothing where it will stand. */
  function arriveFrom(made: SpaceEl, slot: Slot, on: ScreenView): void {
    for (const part of [made.root, made.desk, made.tag]) {
      glide(part, "none");
      part.style.setProperty("--ov-arrive", "0");
      part.style.setProperty("--ov-fly", open ? "0" : "1");
    }
    made.root.style.setProperty("--ov-x", css(slot.x));
    made.root.style.setProperty("--ov-y", css(slot.y));
    made.root.style.setProperty("--ov-w", css(slot.w));
    made.root.style.setProperty("--ov-h", css(slot.h));
    made.desk.style.setProperty("--ov-s", String(slot.s));
    made.desk.style.setProperty("--ov-unscale", String(1 / slot.s));
    made.desk.style.setProperty("--ov-dw", css(on.size.w));
    made.desk.style.setProperty("--ov-dh", css(on.size.h));
    commit(made.root);
    for (const part of [made.root, made.desk, made.tag]) {
      arriving(part, "arrive");
      part.style.setProperty("--ov-arrive", open ? "1" : "0");
    }
  }

  /** Sets how an element arrives or leaves next: its duration, curve and cue. */
  function arriving(element: HTMLElement, motion: "open" | "close" | "arrive" | "collapse" | "settle" | "none", delay = "0s"): void {
    const reduced = settings().reduced;
    const [duration, curve] =
      motion === "open" ? ["long", "emphasized_decel"] : motion === "close" ? ["medium", "emphasized_accel"] : motion === "collapse" ? ["medium", "emphasized_accel"] : motion === "settle" ? ["medium", "standard_decel"] : ["medium", "emphasized_decel"];
    element.style.setProperty("--ov-at", motion === "none" ? "0s" : reduced ? "var(--motion-short)" : `var(--motion-${duration})`);
    element.style.setProperty("--ov-ae", reduced ? "var(--easing-standard)" : `var(--easing-${curve})`);
    element.style.setProperty("--ov-ad", reduced ? "0s" : delay);
  }

  /** Tiles workspace `id`'s windows on its desktop, a landing slot where a carried window would go. */
  function tileSpace(id: string, on: ScreenView, motion: Motion): void {
    const made = spaceEl(id);
    const rects = tilesShown(id, tilingArea(on.size.w, on.size.h));
    const focused = desk.spaces.get(id)!.focused;
    rects.forEach((rect, win) => {
      if (carry?.kind === "window" && carry.id === win) {
        // The place it would take, in its shape: it glides between sides as the pointer moves.
        const fresh = landing.parentElement !== made.desk || !landing.hasAttribute("data-shown");
        if (landing.parentElement !== made.desk) made.desk.append(landing);
        setTile(landing, rect, fresh ? "none" : "reflow");
        landing.dataset.shown = "";
        return;
      }
      const element = windowEl(win);
      tiles.set(win, rect);
      if (element.parentElement !== made.desk && !element.hasAttribute("data-lifted") && !element.hasAttribute("data-landing")) {
        made.desk.append(element);
        setTile(element, rect, "none");
        commit(element);
      }
      if (element.hasAttribute("data-lifted") || element.hasAttribute("data-landing")) return;
      setTile(element, rect, motion);
      element.toggleAttribute("data-focused", win === focused);
    });
    if (landing.parentElement === made.desk && !(carry?.kind === "window" && carry.target?.kind === "space" && carry.target.space === id)) delete landing.dataset.shown;
  }

  function setTile(element: HTMLElement, rect: model.Rect, motion: Motion): void {
    glide(element, motion);
    element.style.setProperty("--ov-tx", css(rect.x));
    element.style.setProperty("--ov-ty", css(rect.y));
    element.style.setProperty("--ov-tw", css(rect.w));
    element.style.setProperty("--ov-th", css(rect.h));
  }

  // ── The marks: the room, the target, the selection ──

  function roomOf(target: Target): { group: string; index: number } | undefined {
    if (target.kind === "between") return { group: target.group, index: target.index };
    if (target.kind === "plus") return { group: target.group, index: model.groupById(desk, target.group).spaces.filter((id) => id !== carry?.id).length };
    return undefined;
  }

  /** The room held open for a carried workspace, or the slit between two for a carried window. */
  function showRoom(): void {
    const holder = views.find((each) => each.layout.room);
    if (carry?.kind === "space" && holder?.layout.room) {
      const slot = holder.layout.room;
      const first = !room.hasAttribute("data-shown");
      setSlot(room, { x: holder.at.x + slot.x, y: holder.at.y + slot.y, w: slot.w, h: slot.h }, first ? "none" : "reflow");
      room.dataset.shown = "";
    } else delete room.dataset.shown;
    const target = carry?.kind === "window" ? carry.target : null;
    if (target?.kind === "between") {
      const point = betweenPoint(target.group, target.index);
      const on = views.find((each) => each.layout.sections.some((section) => section.group === target.group))!;
      const section = on.layout.sections.find((each) => each.group === target.group)!;
      const near = section.places[Math.min(target.index, section.places.length - 1)]!;
      const width = L("space-s4");
      setSlot(slit, { x: point.x - width / 2, y: on.at.y + near.y, w: width, h: near.h }, "none");
      slit.dataset.shown = "";
    } else delete slit.dataset.shown;
  }

  /** A box's place in the world: a workspace's miniature, a group's new-workspace slot. */
  function worldBox(target: Target): (model.Rect & { s: number }) | null {
    if (target.kind === "bin") return null;
    for (const each of views) {
      for (const section of each.layout.sections) {
        if (target.kind === "space") {
          const k = section.items.indexOf(target.space);
          if (k >= 0) {
            const slot = section.places[k]!;
            return { x: each.at.x + slot.x, y: each.at.y + slot.y, w: slot.w, h: slot.h, s: slot.s };
          }
        } else if (section.group === target.group && section.plus) {
          const slot = section.plus;
          return { x: each.at.x + slot.x, y: each.at.y + slot.y, w: slot.w, h: slot.h, s: slot.s };
        }
      }
    }
    return null;
  }

  function showAim(): void {
    const target = carry?.target ?? null;
    const box = target && (target.kind === "space" || target.kind === "plus") ? worldBox(target) : null;
    if (box && !(carry?.kind === "space" && target?.kind === "plus")) {
      const first = !aim.hasAttribute("data-shown");
      setSlot(aim, box, first ? "none" : "reflow");
      aim.dataset.shown = "";
    } else delete aim.dataset.shown;
    if (carry) carry.element.toggleAttribute("data-refused", target?.kind === "refused");
  }

  let selected: string | null = null;
  let keyed = false;

  function showSelect(): void {
    const box = selected && keyed && open && !carry ? worldBox({ kind: "space", space: selected }) : null;
    for (const [id, made] of spaceEls) made.root.toggleAttribute("data-selected", id === selected && keyed && open);
    if (!box) {
      delete select.dataset.shown;
      return;
    }
    const first = !select.hasAttribute("data-shown");
    setSlot(select, box, first ? "none" : "reflow");
    select.dataset.shown = "";
  }

  // ── Opening and closing ──

  /** How far each workspace stands from its screen's shown one, for the arrival's stagger. */
  function rankOf(id: string): number {
    const group = model.groupOf(desk, id);
    const screen = model.holderOf(desk, group);
    const shown = desk.shown.get(screen.id);
    if (model.waits(desk, group) || !shown || !group.spaces.includes(shown)) return 3;
    return Math.min(3, Math.abs(group.spaces.indexOf(id) - group.spaces.indexOf(shown)));
  }

  /** The cue of an arrival `rank` places from the shown workspace: half a `micro` a place. */
  const STAGGER = ["0s", "var(--motion-swap_lag)", "var(--motion-micro)", "calc(var(--motion-micro) + var(--motion-swap_lag))"];

  /** Sets every workspace's and screen's open or closed look, gliding as `motion` says. */
  /**
   * Sets every workspace's and screen's open or closed look, gliding as `motion` says: `settle` is a
   * swipe's lift, every part going on from where the fingers left it over `medium`, none staggered.
   */
  function setLook(isOpen: boolean, motion: "open" | "close" | "settle" | "none"): void {
    const settling = motion === "settle";
    for (const each of views) {
      each.element.toggleAttribute("data-open", isOpen);
      const reduced = settings().reduced;
      for (const part of each.veiled) {
        part.style.setProperty("--ov-vt", motion === "none" ? "0s" : reduced ? "var(--motion-short)" : settling ? "var(--motion-medium)" : "var(--motion-long)");
        part.style.setProperty("--ov-ve", settling ? "var(--easing-standard_decel)" : "var(--easing-standard)");
        part.style.setProperty("--ov-veil", isOpen ? "1" : "0");
      }
      // The frame steps aside as things leave a screen, and comes back as things arrive.
      for (const part of each.aside) {
        part.style.setProperty("--ov-vt", motion === "none" ? "0s" : reduced ? "var(--motion-short)" : isOpen || settling ? "var(--motion-medium)" : "var(--motion-long)");
        part.style.setProperty("--ov-ve", settling ? "var(--easing-standard_decel)" : isOpen ? "var(--easing-emphasized_accel)" : "var(--easing-emphasized_decel)");
        part.style.setProperty("--ov-aside", isOpen ? "1" : "0");
      }
      const shown = desk.shown.get(each.screen.id);
      for (const section of each.layout.sections) {
        for (const id of section.items) {
          if (id === ROOM) continue;
          const made = spaceEl(id);
          const home = id === shown;
          const parts = [made.root, made.desk, made.tag];
          const reduced = settings().reduced && motion !== "none";
          for (const part of parts) {
            part.dataset.rank = home ? "home" : String(rankOf(id));
            // Reduced, the screen's desktop doesn't fly: its miniature fades in where it stands, and out.
            if (reduced && home && !settling) {
              arriving(part, "none");
              part.style.setProperty("--ov-arrive", isOpen ? "0" : "1");
              commit(part);
            }
            if (motion === "none") arriving(part, "none");
            else if (settling) arriving(part, "settle");
            else arriving(part, motion === "open" ? "open" : "close", motion === "open" && !home ? STAGGER[rankOf(id)]! : "0s");
            flying(part, motion);
            part.style.setProperty("--ov-arrive", isOpen || (home && !reduced) ? "1" : "0");
            part.style.setProperty("--ov-fly", !isOpen && home ? "1" : "0");
          }
          // A number shows once its miniature has come to rest: the screen's own after its flight.
          const tag = made.tag;
          tag.style.setProperty("--ov-tt", motion === "none" ? "0s" : isOpen ? "var(--motion-short)" : "var(--motion-micro)");
          tag.style.setProperty("--ov-td", motion === "open" && home && !reduced ? "var(--motion-medium)" : "0s");
          if (settling) tag.style.setProperty("--ov-tt", "var(--motion-short)");
          if (motion === "open" && home) {
            tag.style.setProperty("--ov-tt", "0s");
            tag.style.setProperty("--ov-tag", "0");
            commit(tag);
            tag.style.setProperty("--ov-tt", "var(--motion-short)");
          }
          tag.style.setProperty("--ov-tag", isOpen ? "1" : "0");
        }
      }
      for (const element of [...each.layer.querySelectorAll<HTMLElement>(".ov-plus, .ov-label")]) {
        arriving(element, motion === "none" ? "none" : settling ? "settle" : motion === "open" ? "open" : "close", motion === "open" ? STAGGER[3]! : "0s");
        element.style.setProperty("--ov-arrive", isOpen ? "1" : "0");
      }
    }
    root.toggleAttribute("data-open", isOpen);
  }

  function flying(element: HTMLElement, motion: "open" | "close" | "settle" | "none"): void {
    if (motion === "none" || (settings().reduced && motion !== "settle")) {
      element.style.setProperty("--ov-ft", "0s");
      return;
    }
    const settling = motion === "settle";
    element.style.setProperty("--ov-ft", settling ? "var(--motion-medium)" : "var(--motion-long)");
    element.style.setProperty("--ov-fe", settling ? "var(--easing-standard_decel)" : "var(--easing-emphasized_decel)");
  }

  // ── A three-finger swipe: up opens, down closes, following the fingers ──

  /** How far open the swipe has the Overview, 0 to 1, while fingers are on the touchpad; null otherwise. */
  let swiping: number | null = null;

  /**
   * Draws the Overview `p` of the way open, every part where the fingers have it and nothing gliding:
   * the frame aside and the wallpaper veiled by `p`, the screen's desktop `p` of its way into its
   * miniature, and every other miniature, number, slot and name `p` of the way in.
   */
  function follow(p: number): void {
    if (swiping === null && !open) place("none");
    swiping = Math.max(0, Math.min(1, p));
    const at = String(swiping);
    for (const each of views) {
      for (const part of [...each.veiled, ...each.aside]) {
        part.style.setProperty("--ov-vt", "0s");
        part.style.setProperty(each.veiled.includes(part) ? "--ov-veil" : "--ov-aside", at);
      }
      const shown = desk.shown.get(each.screen.id);
      for (const section of each.layout.sections) {
        for (const id of section.items) {
          if (id === ROOM) continue;
          const made = spaceEl(id);
          const home = id === shown;
          for (const part of [made.root, made.desk, made.tag]) {
            part.dataset.rank = home ? "home" : String(rankOf(id));
            arriving(part, "none");
            flying(part, "none");
            part.style.setProperty("--ov-arrive", home ? "1" : at);
            part.style.setProperty("--ov-fly", home ? String(1 - swiping) : "0");
          }
          made.tag.style.setProperty("--ov-tt", "0s");
          made.tag.style.setProperty("--ov-td", "0s");
          made.tag.style.setProperty("--ov-tag", at);
        }
      }
      for (const element of [...each.layer.querySelectorAll<HTMLElement>(".ov-plus, .ov-label")]) {
        arriving(element, "none");
        element.style.setProperty("--ov-arrive", at);
      }
    }
    root.dataset.swiping = "";
  }

  /** The fingers lift: past half way it settles open, short of it falls back closed, from where it is. */
  function fingersLift(): void {
    if (swiping === null) return;
    const opens = swiping >= 0.5;
    swiping = null;
    delete root.dataset.swiping;
    const was = open;
    open = opens;
    if (opens && !was) {
      selected = desk.shown.get(desk.focused) ?? null;
      keyed = false;
    }
    setLook(opens, "settle");
    if (!opens && was) for (const each of views) each.bar.replaceChildren(...barRow(each.screen, each.size.w));
    if (opens) root.focus({ preventScroll: true });
  }

  // The mockup's stand-in for the touchpad: the wheel turned while the pointer is on the stage. Up
  // is the fingers going up; they lift once the wheel has been still for `short`.
  let wheelAt = 0;
  let wheelStill = 0;
  root.addEventListener(
    "wheel",
    (event) => {
      if (carry) return;
      event.preventDefault();
      const travel = number("scale-swipe") * (views.find((each) => each.screen.id === desk.focused) ?? views[0]!).size.h;
      if (swiping === null) wheelAt = open ? 1 : 0;
      wheelAt = Math.max(0, Math.min(1, wheelAt - event.deltaY / settings().scale / travel));
      follow(wheelAt);
      window.clearTimeout(wheelStill);
      wheelStill = window.setTimeout(fingersLift, ms("short"));
    },
    { passive: false },
  );

  function setOpen(isOpen: boolean, into?: string, focus?: string): void {
    if (open === isOpen) return;
    // Closing while something waits in the slot deletes it, as its timer would.
    if (!isOpen) deleteWaiting();
    if (!isOpen && into) {
      model.go(desk, into);
      if (focus) desk.spaces.get(into)!.focused = focus;
    }
    open = isOpen;
    if (isOpen) {
      selected = desk.shown.get(desk.focused) ?? null;
      keyed = false;
    }
    place("none");
    setLook(isOpen, isOpen ? "open" : "close");
    if (!isOpen) {
      // The bar comes back with the workspaces as they now are.
      for (const each of views) each.bar.replaceChildren(...barRow(each.screen, each.size.w));
    }
    if (isOpen) root.focus({ preventScroll: true });
  }

  // ── Carrying ──

  /** A point on the page's viewport as a point in the world. */
  function toWorld(clientX: number, clientY: number): Point {
    const box = world.getBoundingClientRect();
    return { x: ((clientX - box.left) / box.width) * worldSize.w, y: ((clientY - box.top) / box.height) * worldSize.h };
  }

  function toClient(point: Point): Point {
    const box = world.getBoundingClientRect();
    return { x: box.left + (point.x / worldSize.w) * box.width, y: box.top + (point.y / worldSize.h) * box.height };
  }

  const inside = (point: Point, box: model.Rect): boolean => point.x >= box.x && point.x < box.x + box.w && point.y >= box.y && point.y < box.y + box.h;

  /** What is under `point`: a window, a workspace, a group's slot, or the wallpaper. */
  function hit(point: Point): { kind: "window"; id: string; space: string } | { kind: "space"; id: string } | { kind: "plus"; group: string } | { kind: "waiting" } | { kind: "wallpaper" } {
    // The thing waiting in the slot is taken by anywhere on the slot.
    if (pending && binOn) {
      const on = viewOf(binOn);
      if (on && inside(point, slotBox(on))) return { kind: "waiting" };
    }
    for (const each of views) {
      const local = { x: point.x - each.at.x, y: point.y - each.at.y };
      for (const section of each.layout.sections) {
        for (const [k, id] of section.items.entries()) {
          if (id === ROOM) continue;
          const slot = section.places[k]!;
          // Its number under it is the workspace's too.
          if (!inside(local, { ...slot, h: slot.h + L("space-s8") + L("size-control_sm") })) continue;
          const at = { x: (local.x - slot.x) / slot.s, y: (local.y - slot.y) / slot.s };
          for (const win of [...windowsShown(id)].reverse()) {
            const rect = tiles.get(win);
            if (rect && inside(at, rect)) return { kind: "window", id: win, space: id };
          }
          return { kind: "space", id };
        }
        if (section.plus && inside(local, section.plus)) return { kind: "plus", group: section.group };
      }
    }
    return { kind: "wallpaper" };
  }

  /** The gutter's middle before place `index` of `group`'s section, in world points. */
  function betweenPoint(group: string, index: number): Point {
    for (const each of views) {
      const section = each.layout.sections.find((s) => s.group === group);
      if (!section) continue;
      const gap = L("space-s24");
      const after = section.places[index];
      const before = index > 0 ? section.places[index - 1] : undefined;
      if (after && (!before || section.rows[index] === section.rows[index - 1])) return { x: each.at.x + after.x - gap / 2, y: each.at.y + after.y + after.h / 2 };
      if (before) return { x: each.at.x + before.x + before.w + gap / 2, y: each.at.y + before.y + before.h / 2 };
    }
    return { x: 0, y: 0 };
  }

  /** Where a carried window would land at `point`. */
  function windowTarget(point: Point): Target | null {
    if (overBin(point)) return { kind: "bin" };
    const gap = L("space-s24");
    for (const each of views) {
      const local = { x: point.x - each.at.x, y: point.y - each.at.y };
      for (const section of each.layout.sections) {
        const full = model.groupById(desk, section.group).spaces.length >= model.MOST;
        const count = section.items.length;
        for (let k = 0; k < count; k++) {
          const slot = section.places[k]!;
          const band = local.y >= slot.y && local.y < slot.y + slot.h;
          if (!band) continue;
          const id = section.items[k]!;
          // A gutter with a reach into the miniatures either side: dropped there, a new workspace.
          const before = local.x >= slot.x - gap && local.x < slot.x + gap / 2 && (k === 0 || section.rows[k - 1] !== section.rows[k] || local.x >= slot.x - gap / 2);
          if (before) return full ? { kind: "refused", group: section.group } : { kind: "between", group: section.group, index: k };
          const lastInRow = k === count - 1 || section.rows[k + 1] !== section.rows[k];
          if (lastInRow && local.x >= slot.x + slot.w - gap / 2 && local.x < slot.x + slot.w + gap && !(section.plus && k + 1 === count)) {
            return full ? { kind: "refused", group: section.group } : { kind: "between", group: section.group, index: k + 1 };
          }
          if (inside(local, slot)) {
            const at = placeAt(id, { x: (local.x - slot.x) / slot.s, y: (local.y - slot.y) / slot.s });
            return at ? { kind: "space", space: id, at } : { kind: "space", space: id };
          }
        }
        if (section.plus && inside(local, section.plus)) return { kind: "plus", group: section.group };
      }
    }
    return null;
  }

  /**
   * Where a carried window would tile on workspace `space`, the pointer at `point` on its desktop:
   * beside the window under it, or the nearest, on the side it is nearest. Nothing on a workspace
   * with no other window: it takes the whole of it.
   */
  function placeAt(space: string, point: Point): model.Place | undefined {
    if (!carry) return undefined;
    const on = viewOfSpace(space);
    if (!on) return undefined;
    const base = model.layout(model.remove(model.treeOf(desk.spaces.get(space)!), carry.id), tilingArea(on.size.w, on.size.h), L("space-s8"));
    let best: { window: string; rect: model.Rect; far: number } | null = null;
    for (const [window, rect] of base) {
      const far = Math.max(rect.x - point.x, point.x - rect.x - rect.w, 0) + Math.max(rect.y - point.y, point.y - rect.y - rect.h, 0);
      if (!best || far < best.far) best = { window, rect, far };
    }
    return best ? { window: best.window, side: model.sideOf(best.rect, point) } : undefined;
  }

  /**
   * Where a carried workspace would go at `point`: the place in a section the point is in, counted
   * among the miniatures left when it is lifted, in reading order; refused by a full group.
   */
  function spaceTarget(point: Point): Target | null {
    if (!carry) return null;
    if (overBin(point)) return { kind: "bin" };
    const gap = L("space-s24");
    for (const each of views) {
      const bare = carry.bare.get(each.screen.id);
      if (!bare) continue;
      const local = { x: point.x - each.at.x, y: point.y - each.at.y };
      for (const section of bare.sections) {
        const reach = { x: section.box.x - gap, y: section.box.y - gap, w: section.box.w + 2 * gap, h: section.box.h + 2 * gap };
        if (!inside(local, reach)) continue;
        const group = model.groupById(desk, section.group);
        if (group.spaces.length >= model.MOST && !group.spaces.includes(carry.id)) return { kind: "refused", group: group.id };
        const tag = L("space-s8") + L("size-control_sm");
        let index = 0;
        section.items.forEach((_, k) => {
          const slot = section.places[k]!;
          const above = local.y >= slot.y + slot.h + tag + gap / 2;
          const sameRow = local.y >= slot.y - gap / 2 && local.y < slot.y + slot.h + tag + gap / 2;
          if (above || (sameRow && local.x > slot.x + slot.w / 2)) index = k + 1;
        });
        return { kind: "between", group: group.id, index };
      }
    }
    return null;
  }

  /** The scale a carried window is drawn at over `target`: the workspace's it would join. */
  function scaleOver(target: Target | null): number {
    if (!carry) return 1;
    if (!target || target.kind === "refused" || target.kind === "bin") return carry.scale;
    if (target.kind === "space") return worldBox(target)?.s ?? carry.scale;
    for (const each of views) {
      const section = each.layout.sections.find((s) => s.group === target.group);
      if (section) return section.places[0]?.s ?? carry.scale;
    }
    return carry.scale;
  }

  function liftWindow(id: string, point: Point): void {
    const space = model.spaceOf(desk, id).id;
    const on = viewOfSpace(space);
    const slot = on?.layout.spaces.get(space);
    const rect = tiles.get(id);
    const element = winEls.get(id);
    if (!on || !slot || !rect || !element) return;
    const x = on.at.x + slot.x + rect.x * slot.s;
    const y = on.at.y + slot.y + rect.y * slot.s;
    const grab = { x: (point.x - x) / slot.s, y: (point.y - y) / slot.s };
    // Off its miniature it keeps the corner the miniature gave it, as drawn at that moment.
    element.style.setProperty("--ov-corner", getComputedStyle(element).getPropertyValue("--ov-corner"));
    lift.append(element);
    element.dataset.lifted = "";
    glide(element, "none");
    element.style.setProperty("--ov-gx", css(grab.x));
    element.style.setProperty("--ov-gy", css(grab.y));
    element.style.setProperty("--ov-tx", css(point.x - grab.x));
    element.style.setProperty("--ov-ty", css(point.y - grab.y));
    element.style.setProperty("--ov-sx", String(slot.s));
    element.style.setProperty("--ov-sy", String(slot.s));
    commit(element);
    const index = model.groupOf(desk, space).spaces.indexOf(space);
    carry = { kind: "window", id, element, grab, from: space, fromIndex: index, scale: slot.s, size: { w: rect.w, h: rect.h }, target: { kind: "space", space }, point, bare: new Map() };
    posed.set(element, { x: point.x - grab.x, y: point.y - grab.y, k: 1 });
    root.dataset.carrying = "window";
    element.dataset.held = "";
    if (pending) dressBin();
    place("reflow");
  }

  function liftSpace(id: string, point: Point): void {
    const on = viewOfSpace(id);
    const slot = on?.layout.spaces.get(id);
    const made = spaceEls.get(id);
    if (!on || !slot || !made) return;
    const grab = { x: point.x - on.at.x - slot.x, y: point.y - on.at.y - slot.y };
    const group = model.groupOf(desk, id);
    carry = { kind: "space", id, element: made.root, grab, from: group.id, fromIndex: group.spaces.indexOf(id), scale: slot.s, size: { w: slot.w, h: slot.h }, target: null, point, bare: new Map() };
    posed.set(made.root, { x: point.x - grab.x, y: point.y - grab.y, k: 1 });
    for (const each of views) carry.bare.set(each.screen.id, arrange(each.size.w, each.size.h, specsOf(each.screen)));
    lift.append(made.root);
    made.root.dataset.lifted = "";
    for (const part of [made.root, made.desk, made.tag]) glide(part, "none");
    made.root.style.setProperty("--ov-gx", css(grab.x));
    made.root.style.setProperty("--ov-gy", css(grab.y));
    made.root.style.setProperty("--ov-x", css(point.x - grab.x));
    made.root.style.setProperty("--ov-y", css(point.y - grab.y));
    commit(made.root);
    root.dataset.carrying = "space";
    carry.target = spaceTarget(point);
    if (pending) dressBin();
    place("reflow");
  }

  function moveCarry(point: Point): void {
    if (!carry) return;
    carry.point = point;
    const element = carry.element;
    if (!caught && !flights.has(element)) pose(carry, pointerPose());
    showBin(point);
    const target = carry.kind === "window" ? windowTarget(point) : spaceTarget(point);
    if (sameTarget(target, carry.target)) return;
    carry.target = target;
    lightBin(target?.kind === "bin");
    if (carry.kind === "window") {
      const scale = scaleOver(target);
      element.style.setProperty("--ov-sx", String(scale));
      element.style.setProperty("--ov-sy", String(scale));
    }
    if ((target?.kind === "bin") !== caught) catchCarry(target?.kind === "bin");
    place("reflow");
  }

  // ── The delete slot pulling things in ──

  /** Where a carried or waiting thing is drawn: its box's place, in world points, and its shrink. */
  interface Pose {
    readonly x: number;
    readonly y: number;
    readonly k: number;
  }

  /** A thing the slot poses: carried, or waiting in it. */
  interface Posed {
    readonly kind: "window" | "space";
    readonly element: HTMLElement;
    /** The point it scales about, in its own pixels. */
    readonly grab: Point;
    /** Its own box, in the same pixels as `grab`. */
    readonly size: { readonly w: number; readonly h: number };
    /** A window's scale where it was lifted, which its box is drawn at; a workspace's own. */
    readonly scale: number;
  }

  /**
   * Flights toward a pose worked out afresh each frame, from where each thing is drawn when its
   * flight began, so a pointer that moves or turns back is followed from wherever the thing is, never
   * by a jump. A flight toward the pointer ends there and the pointer takes over.
   */
  const flights = new Map<HTMLElement, { readonly start: number; frame: number }>();
  /** Each posed thing's pose last drawn. */
  const posed = new Map<HTMLElement, Pose>();
  /** Whether the carried thing is in the slot's reach, so held in the slot and not under the pointer. */
  let caught = false;

  /** Under the pointer at its carried size, the point held under it. */
  function pointerPose(): Pose {
    if (!carry) return { x: 0, y: 0, k: 1 };
    return { x: carry.point.x - carry.grab.x, y: carry.point.y - carry.grab.y, k: 1 };
  }

  /** What the slot holds now, left to right: what waits there, then the carried thing pulled in. */
  function held(): Posed[] {
    return [...(pending ? [pending.posed] : []), ...(carry && caught ? [carry] : [])];
  }

  /** How wide `thing` is drawn in the slot: `size.drop_slot` tall, its own shape. */
  function heldWidth(thing: Posed): number {
    return (L("size-drop_slot") * thing.size.w) / thing.size.h;
  }

  /**
   * In the slot, `size.drop_slot` tall, `space.s16` from the slot's line on the left, the right and
   * the top, and `space.s16` above the timer's bar; two side by side the same `space.s16` apart, the
   * waiting one on the left. The slot is as wide as what it holds makes it (`slotBox`). The thing
   * scales about the point held, so its box stands where that point puts its top left corner.
   */
  function slotPose(thing: Posed): Pose {
    const slot = slotBox(viewOf(binOn ?? desk.focused) ?? views[0]!);
    const gap = L("space-s16");
    const things = held();
    const at = Math.max(0, things.indexOf(thing));
    const left = slot.x + gap + things.slice(0, at).reduce((sum, each) => sum + heldWidth(each) + gap, 0);
    const lift = number("scale-lift");
    const base = thing.kind === "window" ? thing.scale : 1;
    const k = L("size-drop_slot") / (thing.size.h * base * lift);
    const s = base * lift * k;
    return { x: left - thing.grab.x * (1 - s), y: slot.y + gap - thing.grab.y * (1 - s), k };
  }

  function pose(thing: Posed, at: Pose): void {
    posed.set(thing.element, at);
    const [x, y] = thing.kind === "window" ? ["--ov-tx", "--ov-ty"] : ["--ov-x", "--ov-y"];
    thing.element.style.setProperty(x, css(at.x));
    thing.element.style.setProperty(y, css(at.y));
    thing.element.style.setProperty("--ov-shrink", String(at.k));
  }

  /**
   * Flies `thing` to `to` over `medium` on `emphasized_decel`, as a carried thing lands; with
   * reduced motion, or in a still sample, it is there at once.
   */
  function fly(thing: Posed, to: () => Pose): void {
    land(thing.element);
    if (settings().reduced || root.hasAttribute("data-still")) {
      pose(thing, to());
      return;
    }
    const from = posed.get(thing.element) ?? to();
    const flight = { start: performance.now(), frame: 0 };
    flights.set(thing.element, flight);
    const step = (now: number): void => {
      if (flights.get(thing.element) !== flight) return;
      const p = Math.min(1, (now - flight.start) / ms("medium"));
      const e = eased("emphasized_decel", p);
      const end = to();
      pose(thing, { x: from.x + (end.x - from.x) * e, y: from.y + (end.y - from.y) * e, k: from.k + (end.k - from.k) * e });
      if (p < 1) flight.frame = requestAnimationFrame(step);
      else flights.delete(thing.element);
    };
    flight.frame = requestAnimationFrame(step);
  }

  /** Stops `element`'s flight where it is drawn. */
  function land(element: HTMLElement): void {
    const was = flights.get(element);
    if (was) cancelAnimationFrame(was.frame);
    flights.delete(element);
  }

  /**
   * The carried thing comes into the slot's reach or leaves it: into the slot, in front of one that
   * waits there, which steps back to make room; or back to the pointer, the waiting one back to the
   * middle.
   */
  function catchCarry(into: boolean): void {
    if (!carry) return;
    caught = into;
    const carried = carry;
    if (into) fly(carried, () => slotPose(carried));
    else fly(carried, pointerPose);
    if (pending) {
      const waiting = pending.posed;
      fly(waiting, () => slotPose(waiting));
    }
    lightBin(into);
    showBin();
  }

  function sameTarget(a: Target | null, b: Target | null): boolean {
    return JSON.stringify(a) === JSON.stringify(b);
  }

  /** Puts the carried thing down where it is aimed, or back where it came from. */
  function drop(cancelled = false): void {
    if (!carry) return;
    const was = carry;
    const target = cancelled ? null : was.target;
    let landed = false;
    if (was.kind === "window" && target && target.kind !== "refused" && target.kind !== "bin") {
      const landingAt: model.Landing = target.kind === "space" ? { space: target.space } : target.kind === "between" ? { group: target.group, index: target.index } : { group: target.group, index: model.groupById(desk, target.group).spaces.length };
      landed = model.moveWindow(desk, was.id, landingAt, target.kind === "space" ? target.at : undefined) !== null;
    }
    if (was.kind === "space" && target && target.kind !== "refused" && target.kind !== "space" && target.kind !== "bin") {
      const at = roomOf(target)!;
      landed = model.moveSpace(desk, was.id, at.group, at.index) !== null;
    }
    carry = null;
    delete root.dataset.carrying;
    // Let go in the reach it keeps its lift: it waits in the slot exactly as it is drawn.
    if (target?.kind !== "bin") delete was.element.dataset.held;
    delete was.element.dataset.refused;
    caught = false;
    // Dropped mid-pull, a window's landing and a workspace's glide take its shrink from where it is.
    if (was.kind === "space" && target?.kind !== "bin") {
      glide(was.element, "land");
      was.element.style.setProperty("--ov-shrink", "1");
    }
    delete landing.dataset.shown;
    if (target?.kind === "bin") {
      park(was);
      return;
    }
    land(was.element);
    if (was.kind === "window") landWindow(was.id, was.element);
    else landSpace(was.id, was.element);
    sweepGone();
    place("reflow");
    if (pending) dressBin();
    showBin();
    root.toggleAttribute("data-landed", landed);
  }

  /** Flies a carried window from where it is into its tile, then puts it on its workspace. */
  function landWindow(id: string, element: HTMLElement): void {
    const space = model.spaceOf(desk, id).id;
    const on = viewOfSpace(space);
    place("reflow");
    const slot = on?.layout.spaces.get(space);
    const rect = tiles.get(id) ?? tileOf(id);
    if (!on || !slot) return;
    // From where it is drawn now, its grab point held, to its tile's place and size.
    const style = getComputedStyle(element);
    // As drawn now, the lift aside: its scale and what it still has of a pull's shrink.
    const k = (Number.parseFloat(style.getPropertyValue("--ov-sx")) || 1) * (Number.parseFloat(style.getPropertyValue("--ov-shrink")) || 1);
    const scale = settings().scale;
    const tx = Number.parseFloat(style.getPropertyValue("--ov-tx")) / scale;
    const ty = Number.parseFloat(style.getPropertyValue("--ov-ty")) / scale;
    const gx = Number.parseFloat(style.getPropertyValue("--ov-gx")) / scale;
    const gy = Number.parseFloat(style.getPropertyValue("--ov-gy")) / scale;
    const ow = Number.parseFloat(style.getPropertyValue("--ov-tw")) / scale;
    const oh = Number.parseFloat(style.getPropertyValue("--ov-th")) / scale;
    const visible = { x: tx + gx * (1 - k), y: ty + gy * (1 - k), w: ow * k, h: oh * k };
    delete element.dataset.lifted;
    element.dataset.landing = "";
    glide(element, "none");
    element.style.setProperty("--ov-gx", "0px");
    element.style.setProperty("--ov-gy", "0px");
    element.style.setProperty("--ov-tx", css(visible.x));
    element.style.setProperty("--ov-ty", css(visible.y));
    element.style.setProperty("--ov-tw", css(rect.w));
    element.style.setProperty("--ov-th", css(rect.h));
    element.style.setProperty("--ov-sx", String(visible.w / rect.w));
    element.style.setProperty("--ov-sy", String(visible.h / rect.h));
    element.style.setProperty("--ov-shrink", "1");
    commit(element);
    glide(element, "land");
    element.style.setProperty("--ov-tx", css(on.at.x + slot.x + rect.x * slot.s));
    element.style.setProperty("--ov-ty", css(on.at.y + slot.y + rect.y * slot.s));
    element.style.setProperty("--ov-sx", String(slot.s));
    element.style.setProperty("--ov-sy", String(slot.s));
    later(ms("medium"), () => settleWindow(id));
  }

  function tileOf(id: string): model.Rect {
    const space = model.spaceOf(desk, id).id;
    const on = viewOfSpace(space)!;
    return tilesShown(space, tilingArea(on.size.w, on.size.h)).get(id)!;
  }

  /** A landed window goes onto its workspace's desktop, where it is drawn as it was. */
  function settleWindow(id: string): void {
    const element = winEls.get(id);
    if (!element || !element.hasAttribute("data-landing")) return;
    const space = model.spaceOf(desk, id).id;
    const made = spaceEls.get(space);
    if (!made) return;
    delete element.dataset.landing;
    made.desk.append(element);
    element.style.removeProperty("--ov-corner");
    glide(element, "none");
    element.style.setProperty("--ov-sx", "1");
    element.style.setProperty("--ov-sy", "1");
    setTile(element, tiles.get(id) ?? tileOf(id), "none");
  }

  /** Flies a carried workspace to its place, then puts it on its screen's layer. */
  function landSpace(id: string, element: HTMLElement): void {
    const made = spaceEls.get(id)!;
    const on = viewOfSpace(id);
    delete element.dataset.lifted;
    element.dataset.landing = "";
    place("reflow");
    const slot = on?.layout.spaces.get(id);
    if (!on || !slot) return;
    for (const part of [made.root, made.desk, made.tag]) glide(part, "land");
    made.root.style.setProperty("--ov-x", css(on.at.x + slot.x));
    made.root.style.setProperty("--ov-y", css(on.at.y + slot.y));
    made.root.style.setProperty("--ov-w", css(slot.w));
    made.root.style.setProperty("--ov-h", css(slot.h));
    made.desk.style.setProperty("--ov-s", String(slot.s));
    made.desk.style.setProperty("--ov-unscale", String(1 / slot.s));
    made.desk.style.setProperty("--ov-dw", css(on.size.w));
    made.desk.style.setProperty("--ov-dh", css(on.size.h));
    dressTag(id, made, on.screen);
    tileSpace(id, on, "land");
    later(ms("medium"), () => {
      if (!element.hasAttribute("data-landing")) return;
      delete element.dataset.landing;
      const home = viewOfSpace(id);
      const final = home?.layout.spaces.get(id);
      if (!home || !final) return;
      home.layer.append(element);
      for (const part of [made.root, made.desk, made.tag]) glide(part, "none");
      made.root.style.setProperty("--ov-x", css(final.x));
      made.root.style.setProperty("--ov-y", css(final.y));
    });
  }

  /** Workspaces the model no longer has collapse where they stand, and go. */
  function sweepGone(): void {
    for (const [id, made] of spaceEls) {
      // One waiting in the slot is out of the desk but not gone; one being deleted goes its own way.
      if (desk.spaces.has(id) || made.root.hasAttribute("data-gone") || made.root.hasAttribute("data-binned") || made.root.hasAttribute("data-waiting")) continue;
      collapse(id, made, "0s");
    }
  }

  function collapse(id: string, made: SpaceEl, delay: string): void {
    made.root.dataset.gone = "";
    for (const part of [made.root, made.desk, made.tag]) {
      arriving(part, "collapse", delay);
      part.style.setProperty("--ov-arrive", "0");
    }
    later(ms("medium") * 2 + ms("long"), () => {
      made.root.remove();
      spaceEls.delete(id);
    });
  }

  // ── The delete slot ──

  /**
   * What waits in the slot: the thing taken out of the desk, drawn where it sat as it was let go,
   * and its timer, the bar running down over `duration.undo`.
   */
  interface Pending {
    readonly taken: model.Taken;
    readonly what: Carried;
    readonly posed: Posed;
    readonly drain: Animation | null;
  }
  let pending: Pending | null = null;
  /** The screen the slot stands on, by id. */
  let binOn: string | null = null;

  // The slot: a workspace's shape on the frame's glass, its line in the Danger tone's glass at
  // rest, lit with its glow while something is pulled in; the delete symbol in its middle, and the
  // timer's bar along its foot while something waits.
  const bin = span("ov-bin", span("ov-slot-symbol", icon("delete", "var(--icon-s32)", "var(--color-burn_error)")), span("ov-slot-timer", span("ov-slot-fill")));
  focusRing(bin, "control", "burn_error", false);
  // A sample's annotation: the slot's reach, the screen clipping it to its corner.
  const reach = options.reach ? span("ov-reach") : null;
  marks.append(bin);

  /** The slot's marks: holding something, and a timer running. */
  function dressBin(left?: number): void {
    bin.toggleAttribute("data-holding", caught || pending !== null);
    bin.toggleAttribute("data-pending", pending !== null);
    if (left !== undefined) bin.style.setProperty("--hold", String(left));
  }

  /** The slot lit as a lit Danger control, while a carried thing is pulled in. */
  function lightBin(over: boolean): void {
    if (over) bin.dataset.pinned = "lit";
    else delete bin.dataset.pinned;
    dressBin();
  }

  /**
   * The slot's place on screen `on`: `space.s48` in from its bottom right corner, as far in as the
   * miniatures' margin. It holds things `size.drop_slot` tall, `space.s16` from its line on every
   * side and above the timer's bar, which keeps `space.s8` from its foot; so it is as wide as what it
   * holds, and empty (or `empty`), as wide as a workspace of the screen's shape would be.
   */
  function slotBox(on: ScreenView, empty = false): model.Rect {
    const gap = L("space-s16");
    const inner = L("size-drop_slot");
    const h = gap + inner + gap + L("size-progress") + L("space-s8");
    const things = empty ? [] : held();
    const w = things.length === 0 ? 2 * gap + (inner * on.size.w) / on.size.h : gap + things.reduce((sum, each) => sum + heldWidth(each) + gap, 0);
    const inset = L("space-s48");
    return { x: on.at.x + on.size.w - inset - w, y: on.at.y + on.size.h - inset - h, w, h };
  }

  /** Shows the slot on the screen under `point` while something is carried, or where one waits. */
  function showBin(point?: Point): void {
    const showing = carry !== null || pending !== null;
    if (!showing) {
      delete bin.dataset.shown;
      binOn = null;
      dressBin();
      return;
    }
    const under = point ? views.find((each) => inside(point, { x: each.at.x, y: each.at.y, w: each.size.w, h: each.size.h })) : undefined;
    const on = (pending && binOn ? viewOf(binOn) : undefined) ?? under ?? viewOf(binOn ?? desk.focused) ?? views[0]!;
    const first = !bin.hasAttribute("data-shown");
    const moved = on.screen.id !== binOn;
    binOn = on.screen.id;
    const box = slotBox(on);
    glide(bin, first || moved ? "none" : "reflow");
    // It rises into its corner from just under it: put there at once, before it shows.
    if (first) bin.style.transition = "none";
    bin.style.setProperty("--ov-x", css(box.x));
    bin.style.setProperty("--ov-y", css(box.y));
    bin.style.setProperty("--ov-w", css(box.w));
    bin.style.setProperty("--ov-h", css(box.h));
    if (first) {
      commit(bin);
      bin.style.removeProperty("transition");
    }
    bin.dataset.shown = "";
    dressBin();
    showReach();
  }

  /**
   * Whether `point` is in the slot's reach: on its screen, within `scale.drop_reach` of the screen's
   * shorter side from the slot's middle. The carried thing covers the slot, so the drop counts
   * well before the pointer is on it.
   */
  function overBin(point: Point): boolean {
    if (!bin.hasAttribute("data-shown") || !binOn) return false;
    const on = viewOf(binOn);
    if (!on || !inside(point, { x: on.at.x, y: on.at.y, w: on.size.w, h: on.size.h })) return false;
    const centre = binCentre();
    return Math.hypot(point.x - centre.x, point.y - centre.y) <= reachOf(on);
  }

  function reachOf(on: ScreenView): number {
    return number("scale-drop_reach") * Math.min(on.size.w, on.size.h);
  }

  /** The reach drawn round the slot, for a sample that shows it. */
  function showReach(): void {
    if (!reach) return;
    const on = viewOf(binOn ?? desk.focused) ?? views[0]!;
    const centre = binCentre();
    const r = reachOf(on);
    // On the screen's own layer, so the screen's edges cut it to its corner.
    if (reach.parentElement !== on.layer) on.layer.append(reach);
    reach.style.setProperty("--ov-x", css(centre.x - on.at.x - r));
    reach.style.setProperty("--ov-y", css(centre.y - on.at.y - r));
    reach.style.setProperty("--ov-w", css(2 * r));
    reach.dataset.shown = "";
  }

  /** The slot's middle, in world points. */
  function binCentre(): Point {
    // The empty slot's middle: the reach stays put however wide what the slot holds makes it.
    const box = slotBox(viewOf(binOn ?? desk.focused) ?? views[0]!, true);
    return { x: box.x + box.w / 2, y: box.y + box.h / 2 };
  }

  /**
   * The carried thing let go in the reach stays in the slot as it sat, out of the desk, its timer
   * running; one already waiting goes for good. A thing still on its way into the slot goes on there.
   */
  function park(was: Posed & { readonly id: string }): void {
    if (pending) deleteWaiting();
    const what: Carried = was.kind === "window" ? { window: was.id } : { space: was.id };
    const { taken } = model.take(desk, what);
    const thing: Posed = { kind: was.kind, element: was.element, grab: was.grab, size: was.size, scale: was.scale };
    pending = { taken, what, posed: thing, drain: null };
    was.element.dataset.waiting = "";
    fly(thing, () => slotPose(thing));
    sweepGone();
    place("reflow");
    showBin();
    drain(1);
  }

  /**
   * A thing deleted from the slot shrinks to nothing and fades where it is drawn, from its size at
   * that instant, settled in the slot or mid-flight. Its origin moves to its own middle
   * with its place made up for, and everything it is drawn with is held as it is first, so the first
   * frame is the last frame of the carry: nothing grows, nothing jumps.
   */
  function vanish(kind: "window" | "space", element: HTMLElement): void {
    const style = getComputedStyle(element);
    const px = settings().scale;
    const read = (name: string): number => Number.parseFloat(style.getPropertyValue(name));
    const [x, y, w, h] = kind === "window" ? ["--ov-tx", "--ov-ty", "--ov-tw", "--ov-th"] : ["--ov-x", "--ov-y", "--ov-w", "--ov-h"];
    const at = { x: read(x!) / px, y: read(y!) / px };
    const size = { w: read(w!) / px, h: read(h!) / px };
    const origin = { x: read("--ov-gx") / px, y: read("--ov-gy") / px };
    const sx = kind === "window" ? read("--ov-sx") : 1;
    const sy = kind === "window" ? read("--ov-sy") : 1;
    const shrink = read("--ov-shrink");
    const lift = 1 + (number("scale-lift") - 1) * read("--ov-held");
    const s = { x: sx * lift * shrink, y: sy * lift * shrink };
    element.style.transition = "none";
    element.style.setProperty(x!, css(at.x + (origin.x - size.w / 2) * (1 - s.x)));
    element.style.setProperty(y!, css(at.y + (origin.y - size.h / 2) * (1 - s.y)));
    element.style.setProperty("--ov-gx", css(size.w / 2));
    element.style.setProperty("--ov-gy", css(size.h / 2));
    if (kind === "window") {
      element.style.setProperty("--ov-sx", String(sx));
      element.style.setProperty("--ov-sy", String(sy));
    }
    element.style.setProperty("--ov-shrink", String(shrink));
    commit(element);
    element.style.removeProperty("transition");
    bins(element);
    element.dataset.binned = "";
    element.style.setProperty("--ov-shrink", "0");
  }

  /** Sets an element's glide as it is deleted: things leaving, over `medium`. */
  function bins(element: HTMLElement): void {
    if (settings().reduced) return glide(element, "none");
    element.style.setProperty("--ov-pt", "var(--motion-medium)");
    element.style.setProperty("--ov-pe", "var(--easing-emphasized_accel)");
    element.style.setProperty("--ov-pd", "0s");
  }

  /**
   * Runs the waiting thing's timer from `left` of it: the bar runs down on `linear` over what is left
   * of `duration.undo`, which reduced motion keeps, since it tells the time left; empty, it goes.
   */
  function drain(left: number): void {
    if (!pending) return;
    const total = Number.parseFloat(raw("duration-undo")) / settings().speed;
    const running = bin.animate([{ "--hold": left }, { "--hold": 0 }], { duration: total * left, easing: "linear", fill: "forwards" });
    pending = { ...pending, drain: running };
    dressBin();
    running.finished.then(
      () => {
        if (pending?.drain === running) deleteWaiting();
      },
      () => {},
    );
  }

  /**
   * The waiting thing goes for good, its windows closed: it shrinks to nothing and fades where it is
   * drawn as the symbol comes back, and the slot leaves once nothing is carried.
   */
  function deleteWaiting(): void {
    if (!pending) return;
    const was = pending;
    pending = null;
    was.drain?.cancel();
    land(was.posed.element);
    posed.delete(was.posed.element);
    model.forget(desk, was.taken);
    delete was.posed.element.dataset.waiting;
    vanish(was.posed.kind, was.posed.element);
    const windows = was.taken.kind === "window" ? [was.taken.window] : [...was.taken.space.windows];
    later(ms("medium"), () => {
      for (const id of windows) {
        winEls.get(id)?.remove();
        winEls.delete(id);
      }
      if (was.taken.kind === "space") {
        spaceEls.get(was.taken.space.id)?.root.remove();
        spaceEls.delete(was.taken.space.id);
      }
      was.posed.element.remove();
    });
    bin.style.removeProperty("--hold");
    if (carry && caught) fly(carry, () => slotPose(carry!));
    showBin();
  }

  /**
   * The waiting thing taken out of the slot again: back in the desk, carried as any carried thing,
   * held at its middle, still in the slot while the pointer is in the reach. Its timer stops and the
   * bar goes.
   */
  function pickUp(point: Point): void {
    if (!pending) return;
    const was = pending;
    pending = null;
    was.drain?.cancel();
    bin.style.removeProperty("--hold");
    model.restore(desk, was.taken);
    const element = was.posed.element;
    delete element.dataset.waiting;
    place("reflow");
    const size = was.posed.size;
    const grab = { x: size.w / 2, y: size.h / 2 };
    if (was.posed.kind === "window") {
      const id = (was.what as { window: string }).window;
      const space = model.spaceOf(desk, id).id;
      const slot = viewOfSpace(space)?.layout.spaces.get(space);
      const scale = slot?.s ?? was.posed.scale;
      carry = { kind: "window", id, element, grab, from: space, fromIndex: model.groupOf(desk, space).spaces.indexOf(space), scale, size, target: { kind: "bin" }, point, bare: new Map() };
      element.style.transition = "none";
      element.style.setProperty("--ov-sx", String(scale));
      element.style.setProperty("--ov-sy", String(scale));
    } else {
      const id = (was.what as { space: string }).space;
      const group = model.groupOf(desk, id);
      carry = { kind: "space", id, element, grab, from: group.id, fromIndex: group.spaces.indexOf(id), scale: was.posed.scale, size, target: { kind: "bin" }, point, bare: new Map() };
      for (const each of views) carry.bare.set(each.screen.id, arrange(each.size.w, each.size.h, specsOf(each.screen)));
      element.style.transition = "none";
    }
    // Held at its middle now: the same box, drawn about another point.
    element.style.setProperty("--ov-gx", css(grab.x));
    element.style.setProperty("--ov-gy", css(grab.y));
    caught = true;
    const held = carry;
    pose(held, slotPose(held));
    commit(element);
    element.style.removeProperty("transition");
    root.dataset.carrying = held.kind;
    held.element.dataset.held = "";
    lightBin(true);
    showBin();
    place("reflow");
  }

  // ── Timers that belong to the world: gone with it ──

  const timers = new Set<number>();
  function later(delay: number, run: () => void): void {
    const timer = window.setTimeout(() => {
      timers.delete(timer);
      if (root.contains(world)) run();
    }, delay);
    timers.add(timer);
  }

  // ── The pointer ──

  /** A press not yet a drag: the left button carries a window, the right one its workspace. */
  let press: { id: number; at: Point; client: Point; on: ReturnType<typeof hit>; right: boolean } | null = null;
  const threshold = (): number => L("space-s8");

  // The right button carries a workspace, so the browser's own menu never opens over the overview.
  root.addEventListener("contextmenu", (event) => event.preventDefault());

  world.addEventListener("pointerdown", (event) => {
    if ((event.button !== 0 && event.button !== 2) || !open || carry) return;
    const at = toWorld(event.clientX, event.clientY);
    press = { id: event.pointerId, at, client: { x: event.clientX, y: event.clientY }, on: hit(at), right: event.button === 2 };
    world.setPointerCapture(event.pointerId);
    root.focus({ preventScroll: true });
    event.preventDefault();
  });

  world.addEventListener("pointermove", (event) => {
    if (!press || event.pointerId !== press.id) return;
    const at = toWorld(event.clientX, event.clientY);
    if (!carry) {
      const box = world.getBoundingClientRect();
      const moved = (Math.hypot(event.clientX - press.client.x, event.clientY - press.client.y) / box.width) * worldSize.w * fitOf();
      if (moved < threshold()) return;
      const on = press.on;
      if (on.kind === "waiting") pickUp(press.at);
      else if (press.right && (on.kind === "window" || on.kind === "space")) {
        liftSpace(on.kind === "window" ? on.space : on.id, press.at);
        learn("space");
      } else if (!press.right && on.kind === "window") {
        liftWindow(on.id, press.at);
        learn("window");
      } else return;
      keyed = false;
    }
    moveCarry(at);
  });

  const release = (event: PointerEvent): void => {
    if (!press || event.pointerId !== press.id) return;
    const was = press;
    press = null;
    if (carry) {
      drop(event.type === "pointercancel");
      tipLeaves();
      return;
    }
    if (was.right || was.on.kind === "waiting") return;
    if (was.on.kind === "window") setOpen(false, was.on.space, was.on.id);
    else if (was.on.kind === "space") setOpen(false, was.on.id);
    else if (was.on.kind === "plus") newSpace(was.on.group);
    else setOpen(false);
  };
  world.addEventListener("pointerup", release);
  world.addEventListener("pointercancel", release);

  /** The world's scale on the page: the whole desk's fit, else 1. */
  function fitOf(): number {
    return view === "desk" ? Number(world.style.getPropertyValue("--ov-fit")) || 1 : 1;
  }

  /** A press on a group's slot: a new empty workspace there, and the overview closes into it. */
  function newSpace(group: string): void {
    const holder = model.holderOf(desk, model.groupById(desk, group));
    const id = `e${Date.now().toString(36)}`;
    desk.spaces.set(id, { id, windows: [], focused: null });
    model.groupById(desk, group).spaces.push(id);
    place("reflow");
    desk.focused = holder.id;
    setOpen(false, id);
  }

  // ── The keyboard ──

  function neighbour(from: string, key: string): string | null {
    const box = worldBox({ kind: "space", space: from });
    if (!box) return null;
    const centre = { x: box.x + box.w / 2, y: box.y + box.h / 2 };
    const [dx, dy] = key === "ArrowLeft" ? [-1, 0] : key === "ArrowRight" ? [1, 0] : key === "ArrowUp" ? [0, -1] : [0, 1];
    let best: string | null = null;
    let bestScore = Number.POSITIVE_INFINITY;
    for (const each of views) {
      for (const [id, slot] of each.layout.spaces) {
        if (id === from) continue;
        const x = each.at.x + slot.x + slot.w / 2 - centre.x;
        const y = each.at.y + slot.y + slot.h / 2 - centre.y;
        const along = x * dx + y * dy;
        if (along <= 0) continue;
        const across = Math.abs(x * dy) + Math.abs(y * dx);
        const score = along + 2 * across;
        if (score < bestScore) {
          bestScore = score;
          best = id;
        }
      }
    }
    return best;
  }

  /** Shift+arrow: the selected workspace's focused window goes to the next workspace that way. */
  function carryByKey(key: string): void {
    if (!selected) return;
    const from = desk.spaces.get(selected);
    const to = neighbour(selected, key);
    const win = from?.focused;
    if (!from || !to || !win) return;
    const element = winEls.get(win);
    const on = viewOfSpace(from.id);
    const slot = on?.layout.spaces.get(from.id);
    const rect = tiles.get(win);
    if (!element || !on || !slot || !rect) return;
    // Lifted where it stands and set down on the other: the same flight a drop makes.
    const centre = { x: on.at.x + slot.x + (rect.x + rect.w / 2) * slot.s, y: on.at.y + slot.y + (rect.y + rect.h / 2) * slot.s };
    liftWindow(win, centre);
    if (!carry) return;
    carry.target = { kind: "space", space: to };
    drop();
    selected = to;
    showSelect();
  }

  root.addEventListener("keydown", (event) => {
    const key = event.key;
    if (!open) {
      if (key.toLowerCase() === "w") {
        event.preventDefault();
        setOpen(true);
      }
      return;
    }
    if (key === "Escape") {
      event.preventDefault();
      if (carry) {
        press = null;
        drop(true);
      } else setOpen(false);
      return;
    }
    if (carry) return;
    if (key.toLowerCase() === "w") {
      event.preventDefault();
      setOpen(false);
      return;
    }
    if (key.startsWith("Arrow")) {
      event.preventDefault();
      if (!selected) selected = desk.shown.get(desk.focused) ?? null;
      if (event.shiftKey) {
        keyed = true;
        carryByKey(key);
        return;
      }
      if (keyed && selected) selected = neighbour(selected, key) ?? selected;
      keyed = true;
      showSelect();
      return;
    }
    if (key === "Enter" && selected) {
      event.preventDefault();
      setOpen(false, selected);
      return;
    }
  });

  // ── Drawn ──

  // A still sample moves nothing: its pin is drawn where each move ends, or at its moment.
  root.toggleAttribute("data-still", options.pin !== undefined);
  place("none");
  setLook(open, "none");
  applyPin();
  root.prepend(world);

  function applyPin(): void {
    const pin = options.pin;
    if (!pin) return;
    if (pin.kind === "lit") {
      selected = pin.space;
      keyed = true;
      showSelect();
      return;
    }
    if (pin.kind === "picked" || pin.kind === "binning" || pin.kind === "pending") return;
    if (pin.kind === "swiping") {
      follow(pin.progress);
      return;
    }
    if (pin.kind === "placed") {
      liftWindow(pin.window, centreOf({ window: pin.window }));
      moveCarry(pointOf(pin.to));
      drop();
      settleWindow(pin.window);
      settle();
      return;
    }
    if (pin.kind === "window") {
      const start = centreOf({ window: pin.window });
      liftWindow(pin.window, start);
      const goal = pointOf(pin.to);
      const towards = pin.crossing !== undefined ? crossingPoint(pin.window, goal, pin.crossing) : goal;
      moveCarry(towards);
      settle();
      return;
    }
    const start = centreOf({ space: pin.space });
    liftSpace(pin.space, start);
    moveCarry(pointOf(pin.to));
    // Drawn over the room it would take, a little short of it, as a hand holds it there.
    const holder = views.find((each) => each.layout.room);
    const slot = holder?.layout.room;
    if (holder && slot && carry) {
      const at = { x: holder.at.x + slot.x + slot.w / 2 - L("space-s24"), y: holder.at.y + slot.y + slot.h / 2 - L("space-s24") };
      carry.element.style.setProperty("--ov-x", css(at.x - carry.grab.x));
      carry.element.style.setProperty("--ov-y", css(at.y - carry.grab.y));
    }
    settle();
  }

  /** A pin on the delete slot, which is measured, so played once the sample is on the page. */
  function applyZonePin(pin: Pin): void {
    if (pin.kind === "binning") {
      const from = liftThing(pin.what);
      const goal = binPoint();
      if (pin.over === "edge") {
        // Toward the round from where it was taken, a tenth of the reach inside its edge.
        const centre = binCentre();
        const away = Math.hypot(from.x - centre.x, from.y - centre.y);
        const at = (reachOf(viewOf(binOn ?? desk.focused) ?? views[0]!) * 0.9) / away;
        moveCarry({ x: centre.x + (from.x - centre.x) * at, y: centre.y + (from.y - centre.y) * at });
      } else moveCarry(pin.over ? goal : { x: from.x + (goal.x - from.x) / 2, y: from.y + (goal.y - from.y) / 2 });
      settle();
      return;
    }
    if (pin.kind === "pending") {
      if (pin.confirmed) dropInBin(pin.confirmed);
      dropInBin(pin.what);
      // A still sample's timer stands where it is drawn.
      if (pending) {
        pending.drain?.cancel();
        pending = { ...pending, drain: null };
      }
      dressBin(pin.left);
      if (pin.next) {
        liftThing(pin.next);
        moveCarry(binPoint());
      }
      settle();
      return;
    }
  }

  /** A point a third of the way from `at`'s window's middle toward its side, in world points. */
  function sidePoint(space: string, at: model.Place): Point {
    const box = worldBox({ kind: "space", space });
    const on = viewOfSpace(space);
    if (!box || !on) return { x: 0, y: 0 };
    const rect = model.layout(model.treeOf(desk.spaces.get(space)!), tilingArea(on.size.w, on.size.h), L("space-s8")).get(at.window);
    if (!rect) return centreOf({ space });
    const [dx, dy] = at.side === "left" ? [-1, 0] : at.side === "right" ? [1, 0] : at.side === "top" ? [0, -1] : [0, 1];
    const x = rect.x + rect.w / 2 + (dx * rect.w) / 3;
    const y = rect.y + rect.h / 2 + (dy * rect.h) / 3;
    return { x: box.x + x * box.s, y: box.y + y * box.s };
  }

  /** Lifts a window by its middle, or a workspace by its miniature's middle; returns where it was taken. */
  function liftThing(what: Carried): Point {
    const at = "window" in what ? centreOf({ window: what.window }) : centreOf({ space: what.space });
    if ("window" in what) liftWindow(what.window, at);
    else liftSpace(what.space, at);
    return at;
  }

  /** Carries `what` into the delete slot and lets go there. */
  function dropInBin(what: Carried): void {
    liftThing(what);
    moveCarry(binPoint());
    drop();
  }

  /** The middle of the delete slot on the focused screen, or where it stands. */
  function binPoint(): Point {
    return binCentre();
  }

  /** On its way from its workspace to `goal`, `share` of the way: crossing a screen's edge. */
  function crossingPoint(_window: string, goal: Point, share: number): Point {
    const from = carry?.point ?? goal;
    return { x: from.x + (goal.x - from.x) * share, y: from.y + (goal.y - from.y) * share };
  }

  /** Every glide finished at once: a still sample's state as it stands once it has moved. */
  function settle(): void {
    void root.offsetWidth;
    for (const animation of ended()) animation.finish();
  }

  /** The moves running in the world that end: a motion player's loop on a sample runs forever. */
  function ended(): Animation[] {
    return root.getAnimations({ subtree: true }).filter((each) => Number.isFinite(Number(each.effect?.getComputedTiming().endTime)));
  }

  function pointOf(target: Target): Point {
    if (target.kind === "space" && target.at) return sidePoint(target.space, target.at);
    if (target.kind === "space") return centreOf({ space: target.space });
    if (target.kind === "plus") return centreOf({ plus: target.group });
    if (target.kind === "between") return betweenPoint(target.group, target.index);
    if (target.kind === "bin") return binPoint();
    const box = views.flatMap((each) => each.layout.sections.filter((s) => s.group === target.group).map((s) => ({ x: each.at.x + s.box.x + s.box.w / 2, y: each.at.y + s.box.y + s.box.h / 2 })))[0];
    return box ?? { x: 0, y: 0 };
  }

  function centreOf(what: { space: string } | { window: string } | { plus: string }): Point {
    if ("space" in what) {
      const box = worldBox({ kind: "space", space: what.space });
      return box ? { x: box.x + box.w / 2, y: box.y + box.h / 2 } : { x: 0, y: 0 };
    }
    if ("plus" in what) {
      const box = worldBox({ kind: "plus", group: what.plus });
      return box ? { x: box.x + box.w / 2, y: box.y + box.h / 2 } : { x: 0, y: 0 };
    }
    const space = model.spaceOf(desk, what.window).id;
    const box = worldBox({ kind: "space", space });
    const rect = tiles.get(what.window);
    if (!box || !rect) return { x: 0, y: 0 };
    return { x: box.x + (rect.x + rect.w / 2) * box.s, y: box.y + (rect.y + rect.h / 2) * box.s };
  }

  const driver: Driver = {
    desk: () => desk,
    snapshot: () => model.snapshot(desk),
    client: toClient,
    centreOf,
    handleOf(space) {
      const box = worldBox({ kind: "space", space });
      return box ? { x: box.x + box.w / 2, y: box.y + box.h + L("space-s8") + L("size-control_sm") / 2 } : { x: 0, y: 0 };
    },
    between: betweenPoint,
    target: () => carry?.target ?? null,
    lifted: () => carry?.id ?? null,
    isOpen: () => open,
    element: (id) => spaceEls.get(id)?.root ?? winEls.get(id) ?? null,
    freeze(at, commit = false) {
      // Transitions set just now start with the next style pass: run it, so they are caught too.
      void root.offsetWidth;
      for (const timer of timers) clearTimeout(timer);
      timers.clear();
      const running = ended();
      for (const animation of running) {
        animation.pause();
        animation.currentTime = at;
      }
      if (!commit) return;
      for (const animation of running) {
        animation.commitStyles();
        animation.cancel();
      }
    },
    thaw: settle,
    redraw: (next) => redraw({ ...options, ...next }),
    bin: () => binPoint(),
    swipe: (p) => follow(p),
    sidePoint: (space, window, side) => sidePoint(space, { window, side }),
    order: (space) => [...model.layout(model.treeOf(desk.spaces.get(space)!), { x: 0, y: 0, w: 1600, h: 900 }, 0).entries()].sort(([, a], [, b]) => a.y - b.y || a.x - b.x).map(([id]) => id),
    numberShown: (space) => spaceEls.get(space)?.rolling?.element.getAttribute("aria-label") ?? "",
    lift: () => fingersLift(),
    learned: () => [...learned].sort(),
    tipShown: () => !tip.hasAttribute("data-learned"),
    pending: () => pending?.what ?? null,
  };

  // A pin drawn mid-move plays the move once the sample is on the page, held at its moment.
  const moving = options.pin?.kind === "picked" || options.pin?.kind === "binning" || options.pin?.kind === "pending" ? options.pin : null;
  if (moving) {
    const seen = new ResizeObserver(() => {
      if (!root.isConnected) return;
      seen.disconnect();
      if (moving.kind === "binning" || moving.kind === "pending") {
        applyZonePin(moving);
        return;
      }
      // Dropped in the slot, then taken back out and carried off, half way to where it was taken.
      dropInBin(moving.what);
      const from = binCentre();
      pickUp(from);
      const home = "window" in moving.what ? centreOf({ space: model.spaceOf(desk, moving.what.window).id }) : betweenPoint(model.groupOf(desk, moving.what.space).id, 0);
      moveCarry({ x: (from.x + home.x) / 2, y: (from.y + home.y) / 2 });
    });
    seen.observe(root);
  }

  return {
    driver,
    resize(width, height) {
      size = { w: width, h: height };
      if (view === "screen") views[0]!.size = { ...size };
      standViews();
      for (const each of views) each.bar.replaceChildren(...barRow(each.screen, each.size.w));
      place("none");
      setLook(open, "none");
    },
  };
}
