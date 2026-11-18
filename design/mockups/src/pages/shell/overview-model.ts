// The Overview's world: the screens on a desk, the workspace groups their desk positions hold, the
// workspaces in each group and the windows on each, and every change a drag or a key makes to them.
// It knows nothing of the page: the view (`overview-parts.ts`) draws it and asks it to change, and a
// script driving the stage reads it to check where everything went.
//
// A group belongs to a desk position. The screen standing there holds it; a group whose position
// has no screen waits on the primary screen, under its own (GLOSSARY.md, "Workspace group"). A
// workspace's number is its place in its group. An empty workspace between two in use stays as a
// gap, so the ones after it keep their numbers; deleting the gap moves them down one. An empty one
// at a group's end is gone unless a screen shows it. A group holds ten workspaces at most.

/** The apps the samples run, each drawn as it looks (`overview-apps.ts`). */
export type App =
  | "ghostty"
  | "vscode"
  | "firefox"
  | "chromium"
  | "yazi"
  | "discord"
  | "telegram"
  | "thunderbird"
  | "steam"
  | "showtime"
  | "spotify"
  | "strawberry"
  | "settings";

/** What a terminal shows, which is what tells two Ghostty windows apart. */
export type Shell = "mockups" | "btop" | "nvim" | "git";

export interface Win {
  readonly id: string;
  readonly app: App;
  readonly title: string;
  readonly shell?: Shell;
}

export interface Space {
  readonly id: string;
  /** Its windows in the order the tiling split them in, the first the largest. */
  windows: string[];
  /** The window last focused on it, which the keyboard's Shift+arrows carry. */
  focused: string | null;
  /** How its windows are tiled, as Hyprland's dwindle holds them; built from `windows` when unset. */
  tree?: Node | null;
}

export interface Group {
  readonly id: string;
  /** The desk position it belongs to. */
  readonly position: string;
  /** Its name, shown while it waits on another screen. */
  readonly name: string;
  spaces: string[];
}

/** A screen on the desk: its logical size, the position it stands at, and where it stands. */
export interface Screen {
  readonly id: string;
  readonly name: string;
  readonly width: number;
  readonly height: number;
  readonly position: string;
  readonly primary: boolean;
  /** Its place on the desk, in logical pixels: the whole desk's view draws the screens so. */
  readonly x: number;
  readonly y: number;
}

export interface Desk {
  readonly screens: readonly Screen[];
  readonly groups: Group[];
  readonly spaces: Map<string, Space>;
  readonly windows: Map<string, Win>;
  /** The workspace each screen shows. */
  readonly shown: Map<string, string>;
  /** The screen the keyboard and the pointer are on. */
  focused: string;
}

/** The most workspaces a group holds, as the bar's 1-10. */
export const MOST = 10;

// ── Reading ───────────────────────────────────────────────────────────────────────────────────

export function primaryOf(desk: Desk): Screen {
  return desk.screens.find((each) => each.primary) ?? desk.screens[0]!;
}

/** The screen a group shows on: the one at its position, or the primary while it waits. */
export function holderOf(desk: Desk, group: Group): Screen {
  return desk.screens.find((each) => each.position === group.position) ?? primaryOf(desk);
}

export function waits(desk: Desk, group: Group): boolean {
  return !desk.screens.some((each) => each.position === group.position);
}

/** The groups `screen` shows, its own first, then those waiting on it in the desk's order. */
export function groupsOn(desk: Desk, screen: Screen): Group[] {
  const own = desk.groups.filter((group) => group.position === screen.position);
  const waiting = screen.primary ? desk.groups.filter((group) => waits(desk, group)) : [];
  return [...own, ...waiting];
}

export function groupOf(desk: Desk, space: string): Group {
  const found = desk.groups.find((group) => group.spaces.includes(space));
  if (!found) throw new Error(`workspace ${space} is in no group`);
  return found;
}

export function spaceOf(desk: Desk, window: string): Space {
  for (const space of desk.spaces.values()) if (space.windows.includes(window)) return space;
  throw new Error(`window ${window} is on no workspace`);
}

export function screenOf(desk: Desk, space: string): Screen {
  return holderOf(desk, groupOf(desk, space));
}

/** A workspace's number: its place in its group, from 1. */
export function numberOf(desk: Desk, space: string): number {
  return groupOf(desk, space).spaces.indexOf(space) + 1;
}

export function isShown(desk: Desk, space: string): boolean {
  return [...desk.shown.values()].includes(space);
}

/** Everything the desk holds, as plain data a script compares. */
export function snapshot(desk: Desk): { groups: { id: string; on: string; spaces: { id: string; windows: string[] }[] }[]; shown: Record<string, string> } {
  return {
    groups: desk.groups.map((group) => ({
      id: group.id,
      on: holderOf(desk, group).id,
      spaces: group.spaces.map((id) => ({ id, windows: [...desk.spaces.get(id)!.windows] })),
    })),
    shown: Object.fromEntries(desk.shown),
  };
}

// ── Tiling ────────────────────────────────────────────────────────────────────────────────────

export interface Rect {
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
}

/** Which side of a window a dropped one takes. */
export type Side = "left" | "right" | "top" | "bottom";

/**
 * A workspace's tiling as Hyprland's dwindle holds it: a window, or a split in two, side by side
 * (`row`), one over the other (`column`), or along the longer side of the room it gets (`auto`),
 * as dwindle splits when nothing chose. `a` takes the left or top half.
 */
export type Node = { readonly window: string } | { readonly split: "row" | "column" | "auto"; readonly a: Node; readonly b: Node };

/** Where a window goes in a workspace: beside `window`, on its `side`. */
export interface Place {
  readonly window: string;
  readonly side: Side;
}

/** The windows of `node`, in order. */
function leaves(node: Node): string[] {
  return "window" in node ? [node.window] : [...leaves(node.a), ...leaves(node.b)];
}

/**
 * `node` with `window` added. Placed, it splits that window: the new one takes the `side` half,
 * the window keeps the other, as a drop with a preselected direction does. Otherwise it splits the
 * last window along its longer side and takes the right or lower half, as `force_split = 2` does.
 */
export function insert(node: Node | null | undefined, window: string, place?: Place): Node {
  const leaf: Node = { window };
  if (!node) return leaf;
  const target = place?.window ?? leaves(node).at(-1)!;
  const walk = (at: Node): Node => {
    if ("window" in at) {
      if (at.window !== target) return at;
      if (!place) return { split: "auto", a: at, b: leaf };
      const split = place.side === "left" || place.side === "right" ? "row" : "column";
      const first = place.side === "left" || place.side === "top";
      return { split, a: first ? leaf : at, b: first ? at : leaf };
    }
    return { ...at, a: walk(at.a), b: walk(at.b) };
  };
  return walk(node);
}

/** `node` without `window`: its half goes to the window it was split from. */
export function remove(node: Node | null | undefined, window: string): Node | null {
  if (!node) return null;
  if ("window" in node) return node.window === window ? null : node;
  const a = remove(node.a, window);
  const b = remove(node.b, window);
  if (!a) return b;
  if (!b) return a;
  return { ...node, a, b };
}

/** A workspace's tiling: its own, or its windows split one after another as dwindle opened them. */
export function treeOf(space: Space): Node | null {
  if (space.tree !== undefined) return space.tree;
  let tree: Node | null = null;
  for (const window of space.windows) tree = insert(tree, window);
  return tree;
}

/**
 * Each window's rect in `area`, the room inside the frame and its gaps, `gap` between two: every
 * split halves its room, the first half rounded.
 */
export function layout(node: Node | null, area: Rect, gap: number): Map<string, Rect> {
  const rects = new Map<string, Rect>();
  const walk = (at: Node, room: Rect): void => {
    if ("window" in at) {
      rects.set(at.window, room);
      return;
    }
    const row = at.split === "row" || (at.split === "auto" && room.w >= room.h);
    if (row) {
      const first = Math.round((room.w - gap) / 2);
      walk(at.a, { x: room.x, y: room.y, w: first, h: room.h });
      walk(at.b, { x: room.x + first + gap, y: room.y, w: room.w - first - gap, h: room.h });
    } else {
      const first = Math.round((room.h - gap) / 2);
      walk(at.a, { x: room.x, y: room.y, w: room.w, h: first });
      walk(at.b, { x: room.x, y: room.y + first + gap, w: room.w, h: room.h - first - gap });
    }
  };
  if (node) walk(node, area);
  return rects;
}

/**
 * The side of `rect` `point` is nearest, the rect cut along its diagonals into four: the side a
 * window dropped there takes.
 */
export function sideOf(rect: Rect, point: { readonly x: number; readonly y: number }): Side {
  const dx = (point.x - (rect.x + rect.w / 2)) / rect.w;
  const dy = (point.y - (rect.y + rect.h / 2)) / rect.h;
  if (Math.abs(dx) >= Math.abs(dy)) return dx < 0 ? "left" : "right";
  return dy < 0 ? "top" : "bottom";
}

// ── Changing ──────────────────────────────────────────────────────────────────────────────────

let made = 0;

/** A new workspace's id, never one in use. */
function freshSpace(desk: Desk): string {
  let id: string;
  do id = `n${++made}`;
  while (desk.spaces.has(id));
  return id;
}

/** Where a lifted thing lands: a workspace, or a new one at `index` of `group`. */
export type Landing = { readonly space: string } | { readonly group: string; readonly index: number };

export function groupById(desk: Desk, id: string): Group {
  const found = desk.groups.find((group) => group.id === id);
  if (!found) throw new Error(`no group ${id}`);
  return found;
}

/** Whether `landing` asks a full group for a new workspace. */
export function refuses(desk: Desk, landing: Landing, moving?: string): boolean {
  if ("space" in landing) return false;
  const group = groupById(desk, landing.group);
  return group.spaces.length >= MOST && !(moving !== undefined && group.spaces.includes(moving));
}

/**
 * Drops the empty workspaces at a group's end that no screen shows. An empty one between two in
 * use stays, a gap holding its number, so the ones after it keep theirs. Returns the ids dropped.
 */
function sweep(desk: Desk): string[] {
  const gone: string[] = [];
  for (const group of desk.groups) {
    while (group.spaces.length > 0) {
      const last = group.spaces.at(-1)!;
      if (desk.spaces.get(last)!.windows.length > 0 || isShown(desk, last)) break;
      gone.push(last);
      group.spaces.pop();
    }
  }
  for (const id of gone) desk.spaces.delete(id);
  return gone;
}

export interface Change {
  /** Workspaces made, and those gone (emptied, or deleted). */
  readonly added: readonly string[];
  readonly removed: readonly string[];
  /** Windows closed. */
  readonly closed: readonly string[];
}

const NOTHING: Change = { added: [], removed: [], closed: [] };

/**
 * Moves `window` to `landing`: onto a workspace's end, or onto a new workspace. Refused (null) by
 * a full group; nothing changes on its own workspace.
 */
export function moveWindow(desk: Desk, window: string, landing: Landing, place?: Place): Change | null {
  const from = spaceOf(desk, window);
  if ("space" in landing && landing.space === from.id && !place) return NOTHING;
  if (refuses(desk, landing)) return null;
  const added: string[] = [];
  let to: Space;
  if ("space" in landing) to = desk.spaces.get(landing.space)!;
  else {
    to = { id: freshSpace(desk), windows: [], focused: null };
    desk.spaces.set(to.id, to);
    groupById(desk, landing.group).spaces.splice(landing.index, 0, to.id);
    added.push(to.id);
  }
  from.tree = remove(treeOf(from), window);
  from.windows = from.windows.filter((each) => each !== window);
  if (from.focused === window) from.focused = from.windows.at(-1) ?? null;
  const into = place && place.window !== window && to.windows.includes(place.window) ? place : undefined;
  to.tree = insert(treeOf(to), window, into);
  to.windows.push(window);
  to.focused = window;
  return { added, removed: sweep(desk), closed: [] };
}

/**
 * Moves workspace `space` to place `index` of `group`, counted without it; to another screen it
 * goes with its windows. Refused (null) by a full group. A workspace a screen shows leaves the
 * screen its neighbour, or a new empty one, so no screen is left showing nothing.
 */
export function moveSpace(desk: Desk, space: string, group: string, index: number): Change | null {
  const from = groupOf(desk, space);
  const to = groupById(desk, group);
  if (from !== to && to.spaces.length >= MOST) return null;
  const added: string[] = [];
  const screenFrom = holderOf(desk, from);
  const leaving = from !== to && holderOf(desk, to) !== screenFrom;
  from.spaces = from.spaces.filter((each) => each !== space);
  to.spaces.splice(Math.min(index, to.spaces.length), 0, space);
  if (leaving && desk.shown.get(screenFrom.id) === space) {
    const left = groupsOn(desk, screenFrom).flatMap((each) => each.spaces);
    let next = left[0];
    if (next === undefined) {
      const fresh: Space = { id: freshSpace(desk), windows: [], focused: null };
      desk.spaces.set(fresh.id, fresh);
      groupsOn(desk, screenFrom)[0]?.spaces.push(fresh.id);
      added.push(fresh.id);
      next = fresh.id;
    }
    desk.shown.set(screenFrom.id, next);
  }
  return { added, removed: sweep(desk), closed: [] };
}

/** Shows `space` on its screen, which takes the keyboard. */
export function go(desk: Desk, space: string): Change {
  const screen = screenOf(desk, space);
  desk.shown.set(screen.id, space);
  desk.focused = screen.id;
  return { added: [], removed: sweep(desk), closed: [] };
}

// ── Deleting with a way back ──────────────────────────────────────────────────────────────────

/**
 * A window or a workspace dropped on the delete zone: out of the overview, but kept as it was until
 * its timer ends, so undoing it never reopens an app. A workspace keeps its windows with it.
 */
export type Taken =
  | {
      readonly kind: "window";
      readonly window: string;
      readonly space: Space;
      readonly group: string;
      readonly spaceIndex: number;
      readonly index: number;
      /** Its workspace's tiling as it was, which undo puts back while nothing else has changed there. */
      readonly tree: Node | null;
    }
  | { readonly kind: "space"; readonly space: Space; readonly group: string; readonly index: number };

/** Takes `what` out of the overview into the delete zone. A screen left showing nothing shows a neighbour. */
export function take(desk: Desk, what: { readonly window: string } | { readonly space: string }): { taken: Taken; change: Change } {
  if ("window" in what) {
    const space = spaceOf(desk, what.window);
    const group = groupOf(desk, space.id);
    const tree = treeOf(space);
    const taken: Taken = { kind: "window", window: what.window, space, group: group.id, spaceIndex: group.spaces.indexOf(space.id), index: space.windows.indexOf(what.window), tree };
    space.tree = remove(tree, what.window);
    space.windows = space.windows.filter((each) => each !== what.window);
    if (space.focused === what.window) space.focused = space.windows.at(-1) ?? null;
    return { taken, change: { added: [], removed: sweep(desk), closed: [] } };
  }
  const space = desk.spaces.get(what.space)!;
  const group = groupOf(desk, what.space);
  const taken: Taken = { kind: "space", space, group: group.id, index: group.spaces.indexOf(what.space) };
  const holder = holderOf(desk, group);
  group.spaces = group.spaces.filter((each) => each !== what.space);
  desk.spaces.delete(what.space);
  const added: string[] = [];
  if (desk.shown.get(holder.id) === what.space) {
    let next = groupsOn(desk, holder).flatMap((each) => each.spaces)[0];
    if (next === undefined) {
      const fresh: Space = { id: freshSpace(desk), windows: [], focused: null };
      desk.spaces.set(fresh.id, fresh);
      group.spaces.push(fresh.id);
      added.push(fresh.id);
      next = fresh.id;
    }
    desk.shown.set(holder.id, next);
  }
  return { taken, change: { added, removed: [what.space, ...sweep(desk)], closed: [] } };
}

/** Puts a taken thing back where it was: a window on its workspace, made again if it had gone. */
export function restore(desk: Desk, taken: Taken): Change {
  const group = groupById(desk, taken.group);
  const added: string[] = [];
  let space = desk.spaces.get(taken.space.id);
  if (!space) {
    space = taken.kind === "space" ? taken.space : { id: taken.space.id, windows: [], focused: null };
    desk.spaces.set(space.id, space);
    group.spaces.splice(Math.min(taken.kind === "space" ? taken.index : taken.spaceIndex, group.spaces.length), 0, space.id);
    added.push(space.id);
  }
  if (taken.kind === "window") {
    const now = treeOf(space);
    const same = taken.tree !== null && JSON.stringify(remove(taken.tree, taken.window)) === JSON.stringify(now);
    space.tree = same ? taken.tree : insert(now, taken.window);
    space.windows.splice(Math.min(taken.index, space.windows.length), 0, taken.window);
    space.focused = taken.window;
  }
  return { added, removed: sweep(desk), closed: [] };
}

/** The timer has ended: the windows taken close for real. */
export function forget(desk: Desk, taken: Taken): readonly string[] {
  const closed = taken.kind === "window" ? [taken.window] : [...taken.space.windows];
  for (const window of closed) desk.windows.delete(window);
  return closed;
}

// ── The samples ───────────────────────────────────────────────────────────────────────────────

interface Seed {
  readonly group: string;
  readonly position: string;
  readonly name: string;
  readonly spaces: readonly (readonly Win[])[];
}

const win = (id: string, app: App, title: string, shell?: Shell): Win => ({ id, app, title, ...(shell ? { shell } : {}) });

/** The groups every sample desk shares: what Kuba has open, wherever the screens are. */
const SEEDS: readonly Seed[] = [
  {
    group: "g-laptop",
    position: "laptop",
    name: "Laptop",
    spaces: [
      [win("w-mail", "thunderbird", "Inbox — 3 unread")],
      [win("w-spotify", "spotify", "Spotify — Discover Weekly"), win("w-telegram", "telegram", "Telegram — Family")],
    ],
  },
  {
    group: "g-ext1",
    position: "ext1",
    name: "External 1",
    spaces: [
      [win("w-docs", "firefox", "Hyprland Wiki — Dispatchers"), win("w-git", "ghostty", "lazygit · kubaos", "git")],
      [win("w-discord", "discord", "Discord — #kubaos")],
      [win("w-btop", "ghostty", "btop", "btop")],
    ],
  },
  {
    group: "g-ext2",
    position: "ext2",
    name: "External 2",
    spaces: [
      [win("w-code", "vscode", "overview-parts.ts — kubaos"), win("w-term", "ghostty", "just mockups", "mockups")],
      [win("w-pr", "chromium", "Pull requests · kubaos"), win("w-settings", "settings", "Settings")],
      [win("w-game", "steam", "Hades II")],
      [win("w-files", "yazi", "Pictures"), win("w-nvim", "ghostty", "nvim · tokens.toml", "nvim"), win("w-video", "showtime", "render-v3.mp4")],
    ],
  },
  {
    group: "g-studio",
    position: "ext3",
    name: "Studio",
    spaces: [
      [win("w-cut", "showtime", "Showtime — take-07.mov")],
      [win("w-music", "strawberry", "Strawberry — Night Drive"), win("w-notes", "ghostty", "nvim · notes.md", "nvim")],
    ],
  },
];

/** Nine windows: what a busy first workspace on the centre screen looks like. */
const CROWD: readonly Win[] = [
  win("w-code", "vscode", "overview-parts.ts — kubaos"),
  win("w-term", "ghostty", "just mockups", "mockups"),
  win("c-docs", "firefox", "MDN — FLIP animations"),
  win("c-git", "ghostty", "lazygit · kubaos", "git"),
  win("c-chat", "telegram", "Telegram — KubaOS"),
  win("c-btop", "ghostty", "btop", "btop"),
  win("c-files", "yazi", "Downloads"),
  win("c-music", "spotify", "Spotify — Focus"),
  win("c-mail", "thunderbird", "Inbox"),
];

/** Which screens stand on the desk. */
export type DeskName = "three" | "laptop" | "ultrawide";

/** How a sample desk is filled. */
export type Fill = "everyday" | "crowded" | "full" | "empty" | "gap";

const LAPTOP = { id: "laptop", name: "Laptop", width: 1920, height: 1200, position: "laptop" } as const;

function screensOf(name: DeskName): Screen[] {
  if (name === "laptop") return [{ ...LAPTOP, primary: true, x: 0, y: 0 }];
  if (name === "ultrawide") return [{ id: "ultrawide", name: "Ultrawide", width: 3440, height: 1440, position: "ext1", primary: true, x: 0, y: 0 }];
  // The portrait screen on the left, the big one in the middle, the laptop on the right, their
  // bottom edges in a line as they stand on the desk.
  return [
    { id: "portrait", name: "Portrait", width: 1080, height: 1920, position: "ext1", primary: false, x: 0, y: 0 },
    { id: "centre", name: "Centre", width: 2560, height: 1440, position: "ext2", primary: true, x: 1080, y: 480 },
    { ...LAPTOP, primary: false, x: 3640, y: 720 },
  ];
}

/** A desk of sample screens and groups, filled as `fill` says, the primary screen focused. */
export function sampleDesk(name: DeskName, fill: Fill = "everyday"): Desk {
  const screens = screensOf(name);
  const desk: Desk = { screens, groups: [], spaces: new Map(), windows: new Map(), shown: new Map(), focused: "" };
  desk.focused = primaryOf(desk).id;
  let counter = 0;
  const seeds = fill === "empty" ? SEEDS.filter((seed) => seed.position === "laptop").map((seed) => ({ ...seed, spaces: [[]] })) : SEEDS;
  for (const seed of seeds) {
    let lists: readonly (readonly Win[])[] = seed.spaces;
    if (fill === "crowded" && seed.position === "ext2") lists = [CROWD, ...seed.spaces.slice(1)];
    // 1, 2 and 4 in use, 3 an empty gap between them: the game's windows were moved away.
    if (fill === "gap" && seed.position === "ext2") lists = [seed.spaces[0]!, seed.spaces[1]!, [], seed.spaces[3]!];
    if (fill === "full" && seed.position === "ext2") {
      const extra = Array.from({ length: MOST - seed.spaces.length }, (_, i): Win[] => [win(`f-${i}`, i % 2 === 0 ? "ghostty" : "firefox", i % 2 === 0 ? `zsh · ${i + 5}` : `Tab ${i + 5}`, "git")]);
      lists = [...seed.spaces, ...extra];
    }
    const group: Group = { id: seed.group, position: seed.position, name: seed.name, spaces: [] };
    for (const list of lists) {
      const id = `s${++counter}`;
      for (const each of list) desk.windows.set(each.id, each);
      desk.spaces.set(id, { id, windows: list.map((each) => each.id), focused: list[0]?.id ?? null });
      group.spaces.push(id);
    }
    desk.groups.push(group);
  }
  for (const screen of screens) {
    const first = groupsOn(desk, screen)[0]?.spaces[0];
    if (first) desk.shown.set(screen.id, first);
  }
  return desk;
}
