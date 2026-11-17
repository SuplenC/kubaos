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
import { focusRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { text } from "../../lib/text.ts";
import { length, raw } from "../../lib/tokens.ts";
import { picker } from "../../site/parts/picker.ts";
import { settings } from "../../site/settings.ts";
import { barScreen } from "./bar-parts.ts";
import { BADGED, face, ICONS } from "./overview-apps.ts";
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
  | { readonly kind: "space"; readonly space: string }
  | { readonly kind: "between"; readonly group: string; readonly index: number }
  | { readonly kind: "plus"; readonly group: string }
  | { readonly kind: "refused"; readonly group: string };

/** A still state drawn on the overview for a sample: the same moves a hand makes, finished. */
export type Pin =
  | { readonly kind: "lit"; readonly space: string }
  | { readonly kind: "window"; readonly window: string; readonly to: Target; readonly crossing?: number }
  | { readonly kind: "space"; readonly space: string; readonly to: Target }
  | { readonly kind: "delete"; readonly space: string; readonly held: number }
  /** A held delete let go of once it filled, drawn `at` milliseconds on. */
  | { readonly kind: "collapsing"; readonly space: string; readonly at: number };

export interface OverviewOptions {
  readonly desk: model.DeskName;
  readonly fill?: model.Fill;
  readonly view: View;
  /** Drawn closed: the desktop it opens from. */
  readonly closed?: boolean;
  /** The screen the keyboard is on, when not the primary. */
  readonly focus?: string;
  /** The stage's switches of desk and view: the mockup's, not the overview's. */
  readonly switches?: boolean;
  readonly pin?: Pin;
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
  /** The gutter's middle before place `index` of `group`. */
  readonly between: (group: string, index: number) => Point;
  readonly target: () => Target | null;
  readonly lifted: () => string | null;
  readonly isOpen: () => boolean;
  /** Every element's place the overview is gliding it to, by its id: a script holds the glide to it. */
  readonly element: (id: string) => HTMLElement | null;
  /** Holds every move running in the world at `at` milliseconds after it started: a capture's frame. */
  readonly freeze: (at: number) => void;
  /** Lets go of the frozen moves, finished. */
  readonly thaw: () => void;
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
  const label = L("type-label-line") + L("space-s8");
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

/** A workspace's elements: its box, the desktop scaled inside it, and its number and delete under it. */
interface SpaceEl {
  readonly root: HTMLElement;
  readonly desk: HTMLElement;
  readonly tag: HTMLElement;
  readonly number: HTMLElement;
  readonly remove: HTMLElement;
  /** The words its delete button says, so it is made again only when they change. */
  said: string;
  heldAt: number;
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

/**
 * The cue every glide set now waits for: a delete's neighbours close its gap once its windows have
 * gone. A move sets it round the changes it makes, and puts it back.
 */
let cue = "0s";

/** Sets an element's glide for its next change of place: its timing, or none. */
function glide(element: HTMLElement, motion: Motion, delay = cue): void {
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

const DESKS: readonly { readonly desk: model.DeskName; readonly name: string }[] = [
  { desk: "three", name: "Three screens" },
  { desk: "laptop", name: "Laptop alone" },
  { desk: "ultrawide", name: "Ultrawide alone" },
];

const VIEWS: readonly { readonly view: View; readonly name: string }[] = [
  { view: "screen", name: "This screen" },
  { view: "desk", name: "Whole desk" },
];

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
  root.style.setProperty("--screen-width", `calc(${size.width} * var(--px))`);
  root.style.setProperty("--screen-height", `calc(${size.height} * var(--px))`);
  root.tabIndex = 0;
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
      const resized = sizeOf(next.desk, next.focus);
      if (!into.closest(".site-full-screen")) {
        into.style.setProperty("--screen-width", `calc(${resized.width} * var(--px))`);
        into.style.setProperty("--screen-height", `calc(${resized.height} * var(--px))`);
      }
      current = build(into, next);
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
    const picture = ICONS[win.app];
    if (picture && BADGED.has(win.app)) {
      const badge = iconTile({ picture, size: "Sm", name: win.title });
      badge.classList.add("ov-badge");
      element.append(badge);
    }
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
    const remove = span("ov-ws-remove");
    const tag = span("ov-ws-tag", number, remove);
    const element = span("ov-ws", desktop, tag);
    element.dataset.space = id;
    made = { root: element, desk: desktop, tag, number, remove, said: "", heldAt: -1 };
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

  /** The delete's words for a workspace of `count` windows, one message per count. */
  function removeWords(count: number): string {
    if (count === 0) return "Hold to remove";
    if (count === 1) return "Hold to close 1 window";
    return `Hold to close ${count} windows`;
  }

  function dressTag(id: string, made: SpaceEl, screen: model.Screen): void {
    const shown = desk.shown.get(screen.id) === id;
    const words = String(model.numberOf(desk, id));
    if (made.number.textContent !== words || made.number.hasAttribute("data-shown") !== shown) {
      made.number.replaceChildren(text(words, "button_small", shown ? "on_primary" : "on_surface"));
      made.number.toggleAttribute("data-shown", shown);
    }
    const count = desk.spaces.get(id)!.windows.length;
    const said = removeWords(count);
    const heldAt = options.pin?.kind === "delete" && options.pin.space === id ? options.pin.held : -1;
    if (made.said === said && made.heldAt === heldAt) return;
    made.said = said;
    made.heldAt = heldAt;
    const button = iconButton({
      icon: "delete",
      label: said,
      kind: "Danger",
      size: "Sm",
      ...(heldAt >= 0 ? { held: heldAt, pinned: "lit" as const } : { hold: () => removeSpace(id) }),
    });
    made.remove.replaceChildren(button);
    made.remove.toggleAttribute("data-pinned", heldAt >= 0);
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
    let list = desk.spaces.get(space)!.windows.filter((id) => !(carry?.kind === "window" && carry.id === id));
    if (carry?.kind === "window" && carry.target?.kind === "space" && carry.target.space === space) list = [...list, carry.id];
    return list;
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
    made.desk.style.setProperty("--ov-dw", css(on.size.w));
    made.desk.style.setProperty("--ov-dh", css(on.size.h));
    made.desk.style.setProperty("--ov-badge", String(1 / slot.s));
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
    made.desk.style.setProperty("--ov-dw", css(on.size.w));
    made.desk.style.setProperty("--ov-dh", css(on.size.h));
    commit(made.root);
    for (const part of [made.root, made.desk, made.tag]) {
      arriving(part, "arrive");
      part.style.setProperty("--ov-arrive", open ? "1" : "0");
    }
  }

  /** Sets how an element arrives or leaves next: its duration, curve and cue. */
  function arriving(element: HTMLElement, motion: "open" | "close" | "arrive" | "collapse" | "none", delay = "0s"): void {
    const reduced = settings().reduced;
    const [duration, curve] =
      motion === "open" ? ["long", "emphasized_decel"] : motion === "close" ? ["medium", "emphasized_accel"] : motion === "collapse" ? ["medium", "emphasized_accel"] : ["medium", "emphasized_decel"];
    element.style.setProperty("--ov-at", motion === "none" ? "0s" : reduced ? "var(--motion-short)" : `var(--motion-${duration})`);
    element.style.setProperty("--ov-ae", reduced ? "var(--easing-standard)" : `var(--easing-${curve})`);
    element.style.setProperty("--ov-ad", reduced ? "0s" : delay);
  }

  /** Tiles workspace `id`'s windows on its desktop, a landing slot where a carried window would go. */
  function tileSpace(id: string, on: ScreenView, motion: Motion): void {
    const made = spaceEl(id);
    const list = windowsShown(id);
    const rects = model.tile(list.length, tilingArea(on.size.w, on.size.h), L("space-s8"));
    const focused = desk.spaces.get(id)!.focused;
    list.forEach((win, k) => {
      const rect = rects[k]!;
      if (carry?.kind === "window" && carry.id === win) {
        if (landing.parentElement !== made.desk) made.desk.append(landing);
        setTile(landing, rect, "none");
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
  function setLook(isOpen: boolean, motion: "open" | "close" | "none"): void {
    for (const each of views) {
      each.element.toggleAttribute("data-open", isOpen);
      const reduced = settings().reduced;
      for (const part of each.veiled) {
        part.style.setProperty("--ov-vt", motion === "none" ? "0s" : reduced ? "var(--motion-short)" : "var(--motion-long)");
        part.style.setProperty("--ov-ve", "var(--easing-standard)");
        part.style.setProperty("--ov-veil", isOpen ? "1" : "0");
      }
      // The frame steps aside as things leave a screen, and comes back as things arrive.
      for (const part of each.aside) {
        part.style.setProperty("--ov-vt", motion === "none" ? "0s" : reduced ? "var(--motion-short)" : isOpen ? "var(--motion-medium)" : "var(--motion-long)");
        part.style.setProperty("--ov-ve", isOpen ? "var(--easing-emphasized_accel)" : "var(--easing-emphasized_decel)");
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
            if (reduced && home) {
              arriving(part, "none");
              part.style.setProperty("--ov-arrive", isOpen ? "0" : "1");
              commit(part);
            }
            if (motion === "none") arriving(part, "none");
            else arriving(part, motion === "open" ? "open" : "close", motion === "open" && !home ? STAGGER[rankOf(id)]! : "0s");
            flying(part, motion);
            part.style.setProperty("--ov-arrive", isOpen || (home && !reduced) ? "1" : "0");
            part.style.setProperty("--ov-fly", !isOpen && home ? "1" : "0");
          }
        }
      }
      for (const element of [...each.layer.querySelectorAll<HTMLElement>(".ov-plus, .ov-label")]) {
        arriving(element, motion === "none" ? "none" : motion === "open" ? "open" : "close", motion === "open" ? STAGGER[3]! : "0s");
        element.style.setProperty("--ov-arrive", isOpen ? "1" : "0");
      }
    }
    root.toggleAttribute("data-open", isOpen);
  }

  function flying(element: HTMLElement, motion: "open" | "close" | "none"): void {
    if (motion === "none" || settings().reduced) {
      element.style.setProperty("--ov-ft", "0s");
      return;
    }
    element.style.setProperty("--ov-ft", "var(--motion-long)");
    element.style.setProperty("--ov-fe", "var(--easing-emphasized_decel)");
  }

  function setOpen(isOpen: boolean, into?: string, focus?: string): void {
    if (open === isOpen) return;
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
  function hit(point: Point): { kind: "window"; id: string; space: string } | { kind: "space"; id: string } | { kind: "plus"; group: string } | { kind: "wallpaper" } {
    for (const each of views) {
      const local = { x: point.x - each.at.x, y: point.y - each.at.y };
      for (const section of each.layout.sections) {
        for (const [k, id] of section.items.entries()) {
          if (id === ROOM) continue;
          const slot = section.places[k]!;
          if (!inside(local, slot)) continue;
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
          if (inside(local, slot)) return { kind: "space", space: id };
        }
        if (section.plus && inside(local, section.plus)) return { kind: "plus", group: section.group };
      }
    }
    return null;
  }

  /**
   * Where a carried workspace would go at `point`: the place in a section the point is in, counted
   * among the miniatures left when it is lifted, in reading order; refused by a full group.
   */
  function spaceTarget(point: Point): Target | null {
    if (!carry) return null;
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
    if (!target || target.kind === "refused") return carry.scale;
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
    lift.append(element);
    element.dataset.lifted = "";
    glide(element, "none");
    element.style.setProperty("--ov-gx", css(grab.x));
    element.style.setProperty("--ov-gy", css(grab.y));
    element.style.setProperty("--ov-tx", css(point.x - grab.x));
    element.style.setProperty("--ov-ty", css(point.y - grab.y));
    element.style.setProperty("--ov-sx", String(slot.s));
    element.style.setProperty("--ov-sy", String(slot.s));
    element.style.setProperty("--ov-badge", String(1 / slot.s));
    commit(element);
    const index = model.groupOf(desk, space).spaces.indexOf(space);
    carry = { kind: "window", id, element, grab, from: space, fromIndex: index, scale: slot.s, target: { kind: "space", space }, point, bare: new Map() };
    root.dataset.carrying = "window";
    element.dataset.held = "";
    place("reflow");
  }

  function liftSpace(id: string, point: Point): void {
    const on = viewOfSpace(id);
    const slot = on?.layout.spaces.get(id);
    const made = spaceEls.get(id);
    if (!on || !slot || !made) return;
    const grab = { x: point.x - on.at.x - slot.x, y: point.y - on.at.y - slot.y };
    const group = model.groupOf(desk, id);
    carry = { kind: "space", id, element: made.root, grab, from: group.id, fromIndex: group.spaces.indexOf(id), scale: slot.s, target: null, point, bare: new Map() };
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
    place("reflow");
  }

  function moveCarry(point: Point): void {
    if (!carry) return;
    carry.point = point;
    const element = carry.element;
    if (carry.kind === "window") {
      element.style.setProperty("--ov-tx", css(point.x - carry.grab.x));
      element.style.setProperty("--ov-ty", css(point.y - carry.grab.y));
    } else {
      element.style.setProperty("--ov-x", css(point.x - carry.grab.x));
      element.style.setProperty("--ov-y", css(point.y - carry.grab.y));
    }
    const target = carry.kind === "window" ? windowTarget(point) : spaceTarget(point);
    if (sameTarget(target, carry.target)) return;
    carry.target = target;
    if (carry.kind === "window") {
      const scale = scaleOver(target);
      element.style.setProperty("--ov-sx", String(scale));
      element.style.setProperty("--ov-sy", String(scale));
      element.style.setProperty("--ov-badge", String(1 / scale));
    }
    place("reflow");
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
    if (was.kind === "window" && target && target.kind !== "refused") {
      const landingAt: model.Landing = target.kind === "space" ? { space: target.space } : target.kind === "between" ? { group: target.group, index: target.index } : { group: target.group, index: model.groupById(desk, target.group).spaces.length };
      landed = model.moveWindow(desk, was.id, landingAt) !== null;
    }
    if (was.kind === "space" && target && target.kind !== "refused" && target.kind !== "space") {
      const at = roomOf(target)!;
      landed = model.moveSpace(desk, was.id, at.group, at.index) !== null;
    }
    carry = null;
    delete root.dataset.carrying;
    delete was.element.dataset.held;
    delete was.element.dataset.refused;
    delete landing.dataset.shown;
    if (was.kind === "window") landWindow(was.id, was.element);
    else landSpace(was.id, was.element);
    sweepGone();
    place("reflow");
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
    const k = Number.parseFloat(style.getPropertyValue("--ov-sx")) || 1;
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
    commit(element);
    glide(element, "land");
    element.style.setProperty("--ov-tx", css(on.at.x + slot.x + rect.x * slot.s));
    element.style.setProperty("--ov-ty", css(on.at.y + slot.y + rect.y * slot.s));
    element.style.setProperty("--ov-sx", String(slot.s));
    element.style.setProperty("--ov-sy", String(slot.s));
    element.style.setProperty("--ov-badge", String(1 / slot.s));
    later(ms("medium"), () => settleWindow(id));
  }

  function tileOf(id: string): model.Rect {
    const space = model.spaceOf(desk, id).id;
    const on = viewOfSpace(space)!;
    const list = windowsShown(space);
    return model.tile(list.length, tilingArea(on.size.w, on.size.h), L("space-s8"))[list.indexOf(id)]!;
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
    glide(element, "none");
    element.style.setProperty("--ov-sx", "1");
    element.style.setProperty("--ov-sy", "1");
    element.style.removeProperty("--ov-badge");
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
    made.desk.style.setProperty("--ov-dw", css(on.size.w));
    made.desk.style.setProperty("--ov-dh", css(on.size.h));
    made.desk.style.setProperty("--ov-badge", String(1 / slot.s));
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
      if (desk.spaces.has(id) || made.root.hasAttribute("data-gone")) continue;
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

  /** Closes every window on workspace `id`, then folds it away; the others close the gap. */
  function removeSpace(id: string): void {
    if (!desk.spaces.has(id)) return;
    const made = spaceEls.get(id)!;
    const closing = [...desk.spaces.get(id)!.windows];
    closing.forEach((win, k) => {
      const element = winEls.get(win);
      if (!element) return;
      element.dataset.closing = "";
      element.style.setProperty("--ov-cd", `calc(var(--motion-swap_lag) * ${k})`);
    });
    model.removeSpace(desk, id);
    // The miniature folds once its windows have gone, and the others close the gap as it does.
    const wait = closing.length > 0 ? `calc(var(--motion-medium) + var(--motion-swap_lag) * ${closing.length - 1})` : "0s";
    collapse(id, made, wait);
    cue = settings().reduced ? "0s" : wait;
    place("reflow");
    cue = "0s";
    later(ms("medium") * 3 + ms("swap_lag") * closing.length, () => {
      for (const win of closing) {
        winEls.get(win)?.remove();
        winEls.delete(win);
      }
    });
    if (selected === id) selected = desk.shown.get(desk.focused) ?? null;
    showSelect();
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

  let press: { id: number; at: Point; client: Point; on: ReturnType<typeof hit> } | null = null;
  const threshold = (): number => L("space-s8");

  world.addEventListener("pointerdown", (event) => {
    if (event.button !== 0 || !open || carry) return;
    if (event.target instanceof Element && event.target.closest(".ov-ws-remove, .ov-switches")) return;
    const at = toWorld(event.clientX, event.clientY);
    press = { id: event.pointerId, at, client: { x: event.clientX, y: event.clientY }, on: hit(at) };
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
      if (press.on.kind === "window") liftWindow(press.on.id, press.at);
      else if (press.on.kind === "space") liftSpace(press.on.id, press.at);
      else return;
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
      return;
    }
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

  let holding: HTMLButtonElement | null = null;

  root.addEventListener("keydown", (event) => {
    if (event.target instanceof Element && event.target.closest(".ov-switches")) return;
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
    if (key === "Delete" && selected && !event.repeat) {
      event.preventDefault();
      keyed = true;
      showSelect();
      holding = spaceEls.get(selected)?.remove.querySelector("button") ?? null;
      holding?.dispatchEvent(new PointerEvent("pointerdown", { button: 0, bubbles: true }));
    }
  });

  root.addEventListener("keyup", (event) => {
    if (event.key !== "Delete" || !holding) return;
    holding.dispatchEvent(new PointerEvent("pointerup", { button: 0, bubbles: true }));
    holding = null;
  });

  // ── The stage's switches ──

  if (options.switches) {
    const desks = picker(DESKS.map((each) => each.name), DESKS.findIndex((each) => each.desk === options.desk), (index) => redraw({ ...options, desk: DESKS[index]!.desk }), "Desk");
    const views = picker(VIEWS.map((each) => each.name), VIEWS.findIndex((each) => each.view === options.view), (index) => redraw({ ...options, view: VIEWS[index]!.view }), "View");
    root.append(span("ov-switches", desks.element, views.element));
  }

  // ── Drawn ──

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
    if (pin.kind === "collapsing") return;
    if (pin.kind === "delete") {
      selected = pin.space;
      keyed = true;
      showSelect();
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
    settle();
  }

  /** On its way from its workspace to `goal`, `share` of the way: crossing a screen's edge. */
  function crossingPoint(_window: string, goal: Point, share: number): Point {
    const from = carry?.point ?? goal;
    return { x: from.x + (goal.x - from.x) * share, y: from.y + (goal.y - from.y) * share };
  }

  /** Every glide finished at once: a still sample's state as it stands once it has moved. */
  function settle(): void {
    for (const animation of root.getAnimations({ subtree: true })) animation.finish();
  }

  function pointOf(target: Target): Point {
    if (target.kind === "space") return centreOf({ space: target.space });
    if (target.kind === "plus") return centreOf({ plus: target.group });
    if (target.kind === "between") return betweenPoint(target.group, target.index);
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
    between: betweenPoint,
    target: () => carry?.target ?? null,
    lifted: () => carry?.id ?? null,
    isOpen: () => open,
    element: (id) => spaceEls.get(id)?.root ?? winEls.get(id) ?? null,
    freeze(at) {
      for (const timer of timers) clearTimeout(timer);
      timers.clear();
      for (const animation of root.getAnimations({ subtree: true })) {
        animation.pause();
        animation.currentTime = at;
      }
    },
    thaw: settle,
  };

  // A pin drawn mid-move plays the move once the sample is on the page, held at its moment.
  if (options.pin?.kind === "collapsing") {
    const pin = options.pin;
    const seen = new ResizeObserver(() => {
      if (!root.isConnected) return;
      seen.disconnect();
      removeSpace(pin.space);
      driver.freeze(pin.at);
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
