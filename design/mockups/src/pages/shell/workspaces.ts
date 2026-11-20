// The bar's workspaces, as `config/quickshell/modules/bar/Workspaces.qml` draws them: a pill of its
// own, `size.control_sm` tall on `surface_container`, `space.s4` round a row of entries. A
// workspace is a cell `size.chip` square, its number centred, semibold (`button_small`) with
// windows or shown, medium (`label`) empty, or its kanji in `accent`. Groups come in block order, a
// hairline `size.chip` / 2 tall in `outline_variant` in a `space.s12` gap between them. A group
// waiting from a desk position no screen holds is borrowed: a `secondary` tint at `opacity.tint`
// round it, its name first in `label`, `secondary`. Each group keeps an empty workspace after its
// last with windows, a ghost: a faint `add` in `outline` at `opacity.placeholder`.
//
// The shown workspace sits on the indicator, `primary` at the full radius, between the tints and
// the cells. Every change of shape glides: an entry grows in or shrinks out, its width over `long`
// on `emphasized_decel`, its opacity over `long` on `standard`. A new shown workspace stretches the
// indicator: its leading edge over `medium`, its trailing edge over `long`, both on
// `emphasized_decel`. A number's colour and the ghost's fade over `short` on `standard`.
//
// A workspace with windows can be carried to another place. Pressed and moved `space.s8`, it lifts
// onto a disc raised to step 1 that eases onto the pointer over `short` on `standard`, its place
// left as a hole, a dashed circle in `outline`. The numbers are the places and stay; the windows
// move. It snaps by the Overview's magnet (`lib/magnet.ts`): the pointer in a place's zone proposes
// it, the hole moves there, the cells between slide over and the disc snaps onto it, each over
// `medium` on `standard_decel`; in no zone the hole is back home and the disc at the hand. A place
// is caught where it stands at rest and, in another group, where it stands with the hole open there,
// so it stays proposed as the hole opening moves it. Over another group, that group takes a
// `primary` outline and opens the hole, while the home hole stays open, as the Overview's room does,
// so the home group stands still; over a group's ghost, the hole is its last place. Let go, the disc
// flies into the hole over `medium` on `emphasized_decel`, then a home hole left behind closes. A slide moves a slot's content,
// never its piece of a group's tint, so the tint stays one band. The shown workspace is where its
// windows are, so the indicator follows them. Nothing in the pill is selected by a press or a drag.
//
// A right click opens its menu (`lib/menu.ts`) on a popover under it: on a workspace, its own
// actions under "Workspace N", then its group's under "This group"; on a group's name or its ghost,
// the group's alone, under its name. The deletes are
// held. Move to group and Move to screen open their lists in place, Back first, as every submenu.
//
// It isn't the library's `Pill`: its cells glide every frame. It has no part in `src/lib/` yet.

import "./workspaces.css";
import type { Pinned } from "../../lib/control.ts";
import { pin } from "../../lib/control.ts";
import { raise } from "../../lib/elevation.ts";
import { focusRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { withHint } from "../../lib/hint.ts";
import { catchShare, caught, type Rect, type Zone, zoneOf } from "../../lib/magnet.ts";
import { type MenuEntry, type MenuItemSpec, menu } from "../../lib/menu.ts";
import { probe } from "../../lib/probe.ts";
import { advance, centreFor, text, type TypeStyle } from "../../lib/text.ts";
import { popover } from "../../lib/tooltip.ts";
import { cubic, durationMs, length } from "../../lib/tokens.ts";
import { settings } from "../../site/settings.ts";

/** A workspace: its id (1–10 in the first block, 11–20 in the second), and whether it has windows. */
export interface Space {
  readonly id: number;
  readonly occupied: boolean;
}

/** A group of workspaces: its block, its name when borrowed and named, and its workspaces. */
export interface Group {
  readonly block: number;
  readonly borrowed?: boolean;
  readonly name?: string;
  readonly spaces: readonly Space[];
}

/** Where a carried workspace lands: group `block`'s place `index`, counted with it in its new place. */
export interface Landing {
  readonly block: number;
  readonly index: number;
}

/** A still of a drag: workspace `id` carried, its hole where `to` puts it (home without), the disc on it or `loose` beside it. */
export interface DragPin {
  readonly id: number;
  readonly to?: Landing;
  readonly loose?: boolean;
}

/** A still of the menu open: on `workspace`, or on `group`'s name, its delete held `holding` of the way. */
export interface MenuPin {
  readonly workspace?: number;
  readonly group?: number;
  readonly holding?: number;
}

export interface WorkspacesOptions {
  readonly groups?: readonly Group[];
  /** The shown workspace's id. */
  readonly active?: number;
  readonly kanji?: boolean;
  /** Clicks and the wheel move the shown workspace, a ghost turning into a workspace; a drag reorders, a right click opens the menu. */
  readonly live?: boolean;
  readonly pinned?: Pinned;
  readonly drag?: DragPin;
  readonly menu?: MenuPin;
}

/** The shots' stand-ins: 1 with a window, 2 shown with one, 3 the ghost; Desk's 11 with a window, 12 its ghost. */
export const SHOTS_GROUPS: readonly Group[] = [
  { block: 0, spaces: [{ id: 1, occupied: true }, { id: 2, occupied: true }, { id: 3, occupied: false }] },
  { block: 1, borrowed: true, name: "Desk", spaces: [{ id: 11, occupied: true }, { id: 12, occupied: false }] },
];

/** The screens Move to screen lists: the Overview's stand-ins. */
const SCREENS = ["Laptop", "External 1"];

const KANJI = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];

/** A workspace in the model: its id, and its windows as `key`, which stays with them as they move. */
interface Place {
  readonly id: number;
  readonly key: string;
  readonly occupied: boolean;
}

interface Band {
  readonly block: number;
  readonly borrowed: boolean;
  readonly name: string;
  readonly spaces: readonly Place[];
}

type Kind = "cell" | "ghost" | "hole" | "gap" | "label";

interface Entry {
  readonly key: string;
  readonly kind: Kind;
  readonly block: number;
  readonly workspace: number;
  /** The windows' key of a cell, a ghost or a hole; the group's first for its name. */
  readonly space: string;
  readonly borrowed: boolean;
  readonly head: boolean;
  readonly tail: boolean;
  readonly label: string;
  readonly occupied: boolean;
  /** Its group takes the carried workspace from another. */
  readonly taking: boolean;
}

interface Point {
  readonly x: number;
  readonly y: number;
}

function slotOf(id: number): number {
  return ((id - 1) % 10) + 1;
}

function numberText(id: number, kanji: boolean): string {
  return kanji ? KANJI[slotOf(id) - 1]! : String(slotOf(id));
}

function numberStyle(strong: boolean, kanji: boolean): TypeStyle {
  if (kanji) return "accent";
  return strong ? "button_small" : "label";
}

/** Whether `space`, at `at` of `spaces`, is drawn as its group's ghost. */
function isGhost(spaces: readonly Place[], at: number, active: string): boolean {
  const space = spaces[at]!;
  return at === spaces.length - 1 && at > 0 && space.key !== active && !space.occupied;
}

/** What the pill draws, in order, as `Workspaces.qml`'s `entries`; `held`'s cell as its hole. */
function entriesOf(bands: readonly Band[], active: string, held: string | null, taking: number | null): Entry[] {
  const result: Entry[] = [];
  const blank = { workspace: 0, space: "", borrowed: false, head: false, tail: false, label: "", occupied: false, taking: false };
  bands.forEach((band, index) => {
    const name = band.borrowed ? band.name : "";
    const takes = band.block === taking;
    if (index > 0) result.push({ ...blank, key: `gap${band.block}`, kind: "gap", block: band.block });
    if (name !== "") result.push({ ...blank, key: `label${band.block}`, kind: "label", block: band.block, workspace: band.spaces[0]!.id, space: band.spaces[0]!.key, borrowed: true, head: true, label: name, taking: takes });
    band.spaces.forEach((space, at) => {
      const kind: Kind = space.key === held ? "hole" : isGhost(band.spaces, at, active) ? "ghost" : "cell";
      // A slot's key is its windows' in its group: carried to another group, it leaves one and
      // grows in the other, never sliding across the rule with the tint it is drawn on.
      result.push({ key: `${space.key}@${band.block}`, kind, block: band.block, workspace: space.id, space: space.key, borrowed: band.borrowed, head: at === 0 && name === "", tail: at === band.spaces.length - 1, label: "", occupied: space.occupied, taking: takes });
    });
  });
  return result;
}

/** A length token in logical pixels. */
function px(name: string): number {
  return length(name);
}

/** A length token in the page's CSS pixels, at the reader's scale. */
function cssPx(name: string): number {
  return length(name) * settings().scale;
}

/** The room a cell leaves before its number, which it centres. */
function numberRoom(id: number, kanji: boolean): number {
  return (px("size-chip") - advance(numberStyle(true, kanji), numberText(id, kanji))) / 2;
}

/** An entry's width in logical pixels, as `widthOf`. */
function widthOf(entry: Entry, kanji: boolean): number {
  if (entry.kind === "label") {
    const inset = Math.round(numberRoom(entry.workspace, kanji));
    const name = Math.min(advance("label", entry.label), px("space-s48") * 2);
    return Math.round(2 * inset + name - numberRoom(entry.workspace, kanji));
  }
  return entry.kind === "gap" ? px("space-s12") : px("size-chip");
}

/** The duration token `name` in milliseconds under the speed setting, nothing with reduce motion. */
function ms(name: string): number {
  return settings().reduced ? 0 : durationMs(name) / settings().speed;
}

function easing(name: string): string {
  return `cubic-bezier(${cubic(`easing-${name}`).join(", ")})`;
}

/** The ids `count` places of a group take: its own in order, then each one past the last. */
function idsFor(own: readonly number[], count: number): number[] {
  const ids = own.slice(0, count);
  while (ids.length < count) ids.push((ids.at(-1) ?? 0) + 1);
  return ids;
}

/** `list` on `band`'s places in order: windows moving onto the ids, the later ones shifting over. */
function renumber(band: Band, list: readonly Place[]): Place[] {
  const ids = idsFor(band.spaces.map((space) => space.id), list.length);
  return list.map((space, at) => ({ ...space, id: ids[at]! }));
}

/** Where a carried workspace's hole stands, by its group's block and its place there. */
function landingOf(bands: readonly Band[], key: string): Landing | null {
  for (const band of bands) {
    const index = band.spaces.findIndex((space) => space.key === key);
    if (index >= 0) return { block: band.block, index };
  }
  return null;
}

export function workspaces(options: WorkspacesOptions = {}): HTMLElement {
  const kanji = options.kanji ?? false;
  let mint = 0;
  const fresh = () => `n${mint++}`;
  let bands: Band[] = (options.groups ?? SHOTS_GROUPS).map((group) => ({
    block: group.block,
    borrowed: group.borrowed ?? false,
    name: group.name ?? "",
    spaces: group.spaces.map((space) => ({ id: space.id, key: `w${space.id}`, occupied: space.occupied })),
  }));
  let active = `w${options.active ?? 2}`;
  const element = document.createElement("span");
  element.className = "shell-ws k-control";
  element.dataset.hot = "";
  element.style.setProperty("--fill", "var(--color-surface_container)");
  element.style.setProperty("--ink", "var(--color-on_surface)");
  pin(element, options.pinned);
  const cells = document.createElement("span");
  cells.className = "shell-ws-cells";
  const indicator = document.createElement("span");
  indicator.className = "shell-ws-indicator";
  const content = document.createElement("span");
  content.className = "shell-ws-row shell-ws-content";
  cells.append(indicator, content);
  element.append(cells);
  focusRing(element, "full", "accent", false);
  if (options.pinned) withHint(element, "Workspaces");

  /** The carried workspace: its windows' key, the groups as they stood at the lift, and where it would land. */
  let carry: { key: string; from: Band[]; landing: Landing | null; number: number; disc: HTMLElement; landed: boolean } | null = null;
  /** The group whose name is being typed over, by block. */
  let renaming: number | null = null;
  let entries: Entry[] = [];

  function entriesNow(): Entry[] {
    const target = carry?.landing ?? null;
    const home = carry ? landingOf(carry.from, carry.key) : null;
    const taking = target && home && target.block !== home.block ? target.block : null;
    return entriesOf(bands, active, carry?.key ?? null, taking);
  }

  /** Keeps one ghost after a group's last workspace with windows or shown, the empty ones after that going. */
  function settle(list: readonly Place[]): Place[] {
    const real = [...list];
    const dropped: Place[] = [];
    while (real.length > 0 && !real.at(-1)!.occupied && real.at(-1)!.key !== active) dropped.push(real.pop()!);
    if (real.length === 0) return [list[0]!];
    const last = real.at(-1)!;
    if (real.length >= 10 || slotOf(last.id) >= 10) return real;
    const ghost = dropped.at(0);
    const id = ghost && ghost.id > last.id ? ghost.id : last.id + 1;
    return [...real, { id, key: ghost?.key ?? fresh(), occupied: false }];
  }

  /** The groups with `key`'s windows taken from their place, the later ones shifting down. */
  function without(from: readonly Band[], key: string): Band[] {
    return from.map((band) => (band.spaces.some((space) => space.key === key) ? { ...band, spaces: settle(renumber(band, band.spaces.filter((space) => space.key !== key))) } : band));
  }

  /** The groups with `key`'s windows carried to `landing`, the ones between shifting over by one. */
  function placed(from: readonly Band[], key: string, landing: Landing | null): Band[] {
    if (!landing) return [...from];
    const moving = from.flatMap((band) => band.spaces).find((space) => space.key === key);
    if (!moving) return [...from];
    return without(from, key).map((band) => {
      if (band.block !== landing.block) return band;
      const list = [...band.spaces];
      list.splice(Math.min(landing.index, list.length), 0, moving);
      return { ...band, spaces: settle(renumber(band, list)) };
    });
  }

  /**
   * The groups as drawn while `key` is carried toward `landing`: in its own group its one hole
   * moves there; in another, its home hole stays open all the while, as the Overview's room does, so
   * its group stands still, and a second opens where it would land.
   */
  function preview(from: readonly Band[], key: string, landing: Landing | null): Band[] {
    const home = landingOf(from, key);
    if (!landing || !home || landing.block === home.block) return placed(from, key, landing);
    const moving = from.find((band) => band.block === home.block)!.spaces[home.index]!;
    return from.map((band) => {
      if (band.block !== landing.block) return band;
      const list = [...band.spaces];
      list.splice(Math.min(landing.index, list.length), 0, moving);
      return { ...band, spaces: settle(renumber(band, list)) };
    });
  }

  const choose = (key: string) => {
    if (options.live && !carry && renaming === null) show(key);
  };

  /** One entry's slot: its piece of the tint and the drop outline behind, and its content. */
  function slot(entry: Entry): HTMLElement {
    const box = document.createElement("span");
    box.className = "shell-ws-slot";
    box.dataset.key = entry.key;
    box.dataset.kind = entry.kind;
    box.dataset.block = String(entry.block);
    box.style.setProperty("--slot-width", String(widthOf(entry, kanji)));
    const piece = (name: string) => {
      const made = document.createElement("span");
      made.className = name;
      made.toggleAttribute("data-head", entry.head);
      made.toggleAttribute("data-tail", entry.tail);
      box.append(made);
    };
    if (entry.borrowed) piece("shell-ws-tint");
    if (entry.taking) piece("shell-ws-drop");
    if (entry.kind === "gap") {
      const rule = document.createElement("span");
      rule.className = "shell-ws-rule";
      box.append(rule);
      return box;
    }
    if (options.live) {
      box.addEventListener("contextmenu", (event) => {
        event.preventDefault();
        if (carry || renaming !== null) return;
        openMenu(entry.kind === "cell" ? { workspace: entry.space } : { block: entry.block }, box);
      });
    }
    if (entry.kind === "label") {
      const name = text(entry.label, "label", "secondary");
      name.classList.add("shell-ws-name");
      name.style.setProperty("--inset", String(Math.round(numberRoom(entry.workspace, kanji))));
      box.append(name);
      if (renaming === entry.block) editName(name, entry, box);
      else box.addEventListener("click", () => choose(entry.space));
      return box;
    }
    if (entry.kind === "hole") {
      const hole = document.createElement("span");
      hole.className = "shell-ws-hole";
      box.append(hole);
      return box;
    }
    const shown = entry.space === active;
    const cell = document.createElement("span");
    cell.className = "shell-ws-cell";
    cell.toggleAttribute("data-ghost", entry.kind === "ghost");
    const number = text(numberText(entry.workspace, kanji), numberStyle(shown || entry.occupied, kanji), shown ? "on_primary" : entry.occupied ? "on_surface" : "on_surface_variant");
    number.classList.add("shell-ws-number");
    // Every cell centres on the same figure, not its own: a "1" and a "2" weigh differently, and
    // each centred on its own ink would put the pill's numbers on different baselines.
    centreFor(number, numberStyle(true, kanji), kanji ? KANJI[KANJI.length - 1]! : "0");
    const plus = icon("add", "var(--icon-s16)", "var(--color-outline)");
    plus.classList.add("shell-ws-ghost");
    cell.append(number, plus);
    cell.addEventListener("click", () => choose(entry.space));
    box.append(cell);
    return box;
  }

  /** Where the cell (or hole) of `key` starts among `entries`, in cells, or -1. */
  function cellsTo(key: string): number {
    let x = 0;
    for (const entry of entries) {
      // Carried to another group, its windows are where it would land, not in its home hole.
      const elsewhere = carry?.key === key && carry.landing !== null && entry.block !== carry.landing.block;
      if (entry.kind !== "gap" && entry.kind !== "label" && entry.space === key && !elsewhere) return x / px("size-chip");
      x += widthOf(entry, kanji);
    }
    return -1;
  }

  /** Draws the entries at once, nothing moving. */
  function draw(): void {
    entries = entriesNow();
    content.replaceChildren(...entries.map(slot));
    const at = cellsTo(active);
    indicator.style.setProperty("--ws-start", String(at));
    indicator.style.setProperty("--ws-end", String(at + 1));
  }

  /** What of a slot slides: its content, not its piece of the tint or of the drop outline. */
  function sliding(child: HTMLElement): HTMLElement[] {
    return [...child.children].filter((each) => !each.matches(".shell-ws-tint, .shell-ws-drop")) as HTMLElement[];
  }

  /** The drawn horizontal offset of `child`'s content: its box, and where a slide running on it has it. */
  function drawnLeft(child: HTMLElement): number {
    const first = sliding(child)[0];
    const moved = first ? getComputedStyle(first).translate : "none";
    return child.offsetLeft + (moved === "none" ? 0 : Number.parseFloat(moved) || 0);
  }

  /**
   * Draws the entries again: slots that stay keep their place and slide to their new one over
   * `medium` on `standard_decel` when `slides`; new ones grow from nothing; gone ones shrink out.
   */
  function render(slides: boolean): void {
    const children = [...content.children] as HTMLElement[];
    const leaving = children.filter((child) => child.hasAttribute("data-gone"));
    const old = new Map(children.filter((child) => !child.hasAttribute("data-gone")).map((child) => [child.dataset.key!, child]));
    const was = new Map([...old].map(([key, child]) => [key, drawnLeft(child)]));
    const at = new Map(children.map((child, index) => [child, index]));
    entries = entriesNow();
    const wanted = new Set(entries.map((entry) => entry.key));
    const next: HTMLElement[] = [];
    const kept: [string, HTMLElement][] = [];
    for (const entry of entries) {
      const made = slot(entry);
      if (old.has(entry.key)) {
        old.delete(entry.key);
        kept.push([entry.key, made]);
      } else {
        made.animate([{ "--present": 0 }, { "--present": 1 }], { duration: ms("long"), easing: easing("emphasized_decel") });
        made.animate([{ "--ws-shown": 0 }, { "--ws-shown": 1 }], { duration: ms("long"), easing: easing("standard") });
      }
      next.push(made);
    }
    for (const [key, gone] of old) {
      if (wanted.has(key)) continue;
      gone.dataset.gone = "";
      gone.animate([{ "--ws-shown": 1 }, { "--ws-shown": 0 }], { duration: ms("long"), easing: easing("standard"), fill: "forwards" });
      gone.animate([{ "--present": 1 }, { "--present": 0 }], { duration: ms("long"), easing: easing("emphasized_decel"), fill: "forwards" }).finished.then(() => gone.remove(), () => {});
      leaving.push(gone);
    }
    for (const gone of leaving) next.splice(Math.min(next.length, at.get(gone) ?? next.length), 0, gone);
    content.replaceChildren(...next);
    if (!slides) return;
    for (const [key, made] of kept) {
      const from = was.get(key)! - made.offsetLeft;
      if (Math.abs(from) < 0.5 || ms("medium") === 0) continue;
      // Its content slides out of its box, which shows it while it does.
      made.dataset.sliding = "";
      const slides_ = sliding(made).map((each) => each.animate([{ translate: `${from}px 0` }, { translate: "0 0" }], { duration: ms("medium"), easing: easing("standard_decel") }));
      Promise.all(slides_.map((each) => each.finished)).then(() => delete made.dataset.sliding, () => {});
    }
  }

  /** The indicator onto the shown workspace's place: its leading edge over `lead`, its trailing one over `trail`. */
  function indicate(lead: string, trail: string, curve: string): void {
    const target = cellsTo(active);
    const from = getComputedStyle(indicator);
    const start = Number.parseFloat(from.getPropertyValue("--ws-start"));
    const end = Number.parseFloat(from.getPropertyValue("--ws-end"));
    if (start === target && end === target + 1) return;
    const right = target > start;
    indicator.style.setProperty("--ws-start", String(target));
    indicator.style.setProperty("--ws-end", String(target + 1));
    indicator.animate([{ "--ws-start": start }, { "--ws-start": target }], { duration: ms(right ? trail : lead), easing: easing(curve) });
    indicator.animate([{ "--ws-end": end }, { "--ws-end": target + 1 }], { duration: ms(right ? lead : trail), easing: easing(curve) });
  }

  /** Shows the workspace of `key`: a ghost becomes a workspace and its group keeps a new ghost after it; an empty one left goes. */
  function show(key: string): void {
    if (key === active) return;
    const leaving = active;
    active = key;
    bands = bands.map((band) => ({ ...band, spaces: settle(band.spaces.filter((space, at) => space.occupied || space.key !== leaving || at === band.spaces.length - 1)) }));
    render(false);
    indicate("medium", "long", "emphasized_decel");
  }

  /** The groups changed by an action, drawn again with every change gliding. */
  function change(next: Band[]): void {
    bands = next;
    if (!bands.some((band) => band.spaces.some((space) => space.key === active))) active = bands[0]!.spaces[0]!.key;
    render(true);
    indicate("medium", "medium", "standard_decel");
  }

  // ---------------------------------------------------------------- carrying --

  /** A point of `event` in the pill's own CSS pixels: its box is scaled by the page's zoom like everything in it. */
  function local(event: PointerEvent): Point {
    const box = element.getBoundingClientRect();
    const ratio = element.offsetWidth / Math.max(1, box.width);
    return { x: (event.clientX - box.left) * ratio, y: (event.clientY - box.top) * ratio };
  }

  /** A slot's box in the pill's CSS pixels, as laid out, slides aside. */
  function boxOf(child: HTMLElement): { x: number; y: number; w: number; h: number } {
    return { x: cells.offsetLeft + child.offsetLeft, y: cells.offsetTop + child.offsetTop, w: child.offsetWidth, h: child.offsetHeight };
  }

  /** The middle of the carried workspace's hole. */
  function holeCentre(): Point {
    const block = carry ? (carry.landing ?? landingOf(carry.from, carry.key))?.block : undefined;
    const hole = carry ? content.querySelector<HTMLElement>(`.shell-ws-slot[data-kind="hole"][data-block="${block}"]:not([data-gone])`) : null;
    if (!hole) return { x: 0, y: 0 };
    const box = boxOf(hole);
    return { x: box.x + cssPx("size-chip") / 2, y: box.y + box.h / 2 };
  }

  /** The slots of the cells, ghosts and holes `list` draws, in the pill's CSS pixels: where they stand once laid out. */
  function slotsOf(list: readonly Entry[]): { entry: Entry; box: Rect }[] {
    const scale = settings().scale;
    const h = px("size-chip") * scale;
    let x = cells.offsetLeft;
    const result: { entry: Entry; box: Rect }[] = [];
    for (const entry of list) {
      const w = widthOf(entry, kanji) * scale;
      if (entry.kind === "cell" || entry.kind === "ghost" || entry.kind === "hole") result.push({ entry, box: { x, y: cells.offsetTop, w, h } });
      x += w;
    }
    return result;
  }

  /** The zones worked out for a carry: the groups do not change while a workspace is carried. */
  let zoneCache: { readonly key: string; readonly from: readonly Band[]; readonly zones: readonly Zone<Landing>[] } | null = null;

  /**
   * Every place's zone for the carried workspace, as the Overview's `zonesOf`: each group's places
   * where they stand at rest, with the hole at home, and, for a group it is not from, also where
   * they stand with the hole open first in it, so a place is caught from either. A cell or the hole
   * proposes its place; a ghost, the group's last. A group with no room left offers none.
   */
  function zones(): readonly Zone<Landing>[] {
    if (!carry) return [];
    const held = carry;
    if (zoneCache?.key === held.key && zoneCache.from === held.from) return zoneCache.zones;
    const home = landingOf(held.from, held.key)!;
    const rest = slotsOf(entriesOf(held.from, active, held.key, null));
    const result: Zone<Landing>[] = [];
    for (const band of held.from) {
      const others = band.spaces.filter((space) => space.key !== held.key);
      const room = others.length - (others.length > 0 && isGhost(others, others.length - 1, active) ? 1 : 0);
      if (band.block !== home.block && room >= 10) continue;
      const layouts = band.block === home.block ? [rest] : [slotsOf(entriesOf(preview(held.from, held.key, { block: band.block, index: 0 }), active, held.key, band.block)), rest];
      for (const layout of layouts) {
        let index = 0;
        for (const { entry, box } of layout) {
          if (entry.block !== band.block) continue;
          const landing = { block: band.block, index: entry.kind === "ghost" ? room : index++ };
          result.push(zoneOf(box, landing, catchShare(box)));
        }
      }
    }
    zoneCache = { key: held.key, from: held.from, zones: result };
    return result;
  }

  /** The disc's place: where it is drawn from, and the offset still easing out of it. */
  let base: Point = { x: 0, y: 0 };
  let easings: Animation[] = [];

  /** Puts the disc's base at `to`; with `over`, it eases there from where it is drawn now. */
  function discTo(disc: HTMLElement, to: Point, over?: { duration: string; curve: string }): Animation | null {
    const drawn = getComputedStyle(disc);
    const now = { x: base.x + (Number.parseFloat(drawn.getPropertyValue("--ws-dx")) || 0), y: base.y + (Number.parseFloat(drawn.getPropertyValue("--ws-dy")) || 0) };
    base = to;
    disc.style.setProperty("--ws-x", String(to.x));
    disc.style.setProperty("--ws-y", String(to.y));
    if (!over) return null;
    for (const each of easings) each.cancel();
    const timing = { duration: ms(over.duration), easing: easing(over.curve) };
    easings = [disc.animate([{ "--ws-dx": now.x - to.x }, { "--ws-dx": 0 }], timing), disc.animate([{ "--ws-dy": now.y - to.y }, { "--ws-dy": 0 }], timing)];
    return easings[0]!;
  }

  /** The disc the carried workspace rides on: its number, as at its lift. */
  function makeDisc(id: number): HTMLElement {
    const disc = raise(document.createElement("span"), 1);
    disc.className += " shell-ws-disc";
    const number = text(numberText(id, kanji), numberStyle(true, kanji), "on_surface");
    number.classList.add("shell-ws-number");
    centreFor(number, numberStyle(true, kanji), kanji ? KANJI[KANJI.length - 1]! : "0");
    disc.append(number);
    element.append(disc);
    return disc;
  }

  /**
   * Lifts the workspace of `key` onto its disc, its place left as a hole. Under `pointer`, it comes
   * up over `short` on `standard` to the place the pointer proposes, or to the hand; a still is
   * drawn with its hole at `landing`.
   */
  function lift(key: string, pointer: Point | null, landing: Landing | null = null): void {
    const space = bands.flatMap((band) => band.spaces).find((each) => each.key === key)!;
    carry = { key, from: bands, landing: null, number: space.id, disc: makeDisc(space.id), landed: false };
    zoneCache = null;
    element.dataset.carrying = "";
    render(false);
    const centre = holeCentre();
    base = centre;
    discTo(carry.disc, centre);
    if (!pointer) return void (landing && propose(landing, false));
    const proposed = caught(pointer, zones());
    if (proposed) propose(proposed, false);
    discTo(carry.disc, proposed ? holeCentre() : pointer, { duration: "short", curve: "standard" });
  }

  /** The hole moved to `landing`, or home with none: the cells between slide over. */
  function propose(landing: Landing | null, slides: boolean): void {
    if (!carry) return;
    carry.landing = landing;
    bands = preview(carry.from, carry.key, landing);
    if (!slides) return draw();
    render(true);
    indicate("medium", "medium", "standard_decel");
  }

  function same(a: Landing | null, b: Landing | null): boolean {
    return a?.block === b?.block && a?.index === b?.index;
  }

  /**
   * The pointer at `point` while carrying: in a place's zone it proposes that place, and the disc
   * snaps into the hole there; in none, the hole is home and the disc at the hand. A new proposal
   * eases the disc over `medium` on `standard_decel`; within one it stays put, or follows the hand.
   */
  function carryTo(point: Point): void {
    if (!carry || carry.landed) return;
    const landing = caught(point, zones());
    const moved = !same(landing, carry.landing);
    if (moved) propose(landing, true);
    const target = landing ? holeCentre() : point;
    discTo(carry.disc, target, moved ? { duration: "medium", curve: "standard_decel" } : undefined);
  }

  /** Let go: the disc flies into the hole over `medium` on `emphasized_decel`, and the hole becomes the workspace. */
  function drop(): void {
    if (!carry || carry.landed) return;
    const held = carry;
    held.landed = true;
    const flight = discTo(held.disc, holeCentre(), { duration: "medium", curve: "emphasized_decel" });
    // Landed, the move is made: a home hole left in its group shrinks out, the rest closing up.
    const done = () => {
      if (carry !== held) return;
      held.disc.remove();
      bands = placed(held.from, held.key, held.landing);
      carry = null;
      delete element.dataset.carrying;
      render(false);
      indicate("medium", "medium", "standard_decel");
    };
    if (flight) flight.finished.then(done, done);
    else done();
  }

  // ---------------------------------------------------------------- the menu --

  /** The popover's room: the pill, its menu under it. */
  const room = options.live || options.menu ? document.createElement("span") : null;
  let card: HTMLElement | null = null;

  function closeMenu(): void {
    card?.remove();
    card = null;
    document.removeEventListener("pointerdown", outside, true);
  }

  function outside(event: PointerEvent): void {
    if (card && !card.contains(event.target as Node)) closeMenu();
  }

  function bandOf(key: string): Band | undefined {
    return bands.find((band) => band.spaces.some((space) => space.key === key));
  }

  function groupName(band: Band): string {
    return band.name || `Group ${band.block + 1}`;
  }

  /** The workspaces a group has, its ghost aside. */
  function realOf(band: Band): Place[] {
    return band.spaces.filter((_, at) => !isGhost(band.spaces, at, active));
  }

  /** A menu's entry, with what taking it does. */
  type Row = "separator" | { readonly heading: string } | { readonly spec: MenuItemSpec; readonly act: () => void };

  /**
   * The menu of a workspace (`workspace`, its windows' key) or of a group (`block`): the
   * workspace's actions, then its group's. `still` draws it pinned, its delete held `holding` of the way.
   */
  function openMenu(on: { workspace: string } | { block: number }, anchor: HTMLElement, still?: { readonly holding?: number }): void {
    if (!room) return;
    closeMenu();
    const band = "workspace" in on ? bandOf(on.workspace)! : bands.find((each) => each.block === on.block)!;
    const others = bands.filter((each) => each !== band);
    const rows: Row[] = [];
    if ("workspace" in on) {
      const key = on.workspace;
      const space = band.spaces.find((each) => each.key === key)!;
      rows.push(
        { heading: `Workspace ${slotOf(space.id)}` },
        { spec: { label: "Move to group", submenu: true, disabled: others.length === 0 }, act: () => sub(others.map((each) => ({ spec: { label: groupName(each), variable: true, disabled: realOf(each).length >= 10 }, act: () => change(placed(bands, key, { block: each.block, index: realOf(each).length })) }))) },
        { spec: { label: "Close all windows", disabled: !space.occupied }, act: () => closeWindows(band, key) },
        { spec: { label: "Delete workspace", held: true }, act: () => deleteWorkspace(band, key) },
        "separator",
      );
    }
    rows.push(
      { heading: "workspace" in on ? "This group" : groupName(band) },
      { spec: { label: "Rename", disabled: !band.borrowed || band.name === "" }, act: () => rename(band.block) },
      { spec: { label: "Move to screen", submenu: true }, act: () => sub(SCREENS.map((name) => ({ spec: { label: name }, act: () => {} }))) },
      { spec: { label: "Delete group", held: true }, act: () => deleteGroup(band) },
    );
    const showRows = (list: readonly Row[], back: (() => void) | null) => {
      const holding = still?.holding;
      const held = holding === undefined ? -1 : list.findIndex((row) => row !== "separator" && "spec" in row && (row.spec.held ?? false));
      const entries: MenuEntry[] = list.map((row, index) => (row === "separator" || "heading" in row ? row : index === held && holding !== undefined ? { ...row.spec, holding } : row.spec));
      const list_ = menu(entries, {
        ...(held >= 0 ? { current: held, pinned: "lit" as const } : {}),
        ...(still
          ? {}
          : {
              onTake: (index: number) => {
                const row = list[index];
                if (!row || row === "separator" || "heading" in row) return;
                if (!row.spec.submenu) closeMenu();
                row.act();
              },
              onLeave: () => (back ? back() : closeMenu()),
            }),
      });
      card?.remove();
      card = popover(2, list_);
      card.classList.add("shell-ws-menu");
      card.style.setProperty("--menu-x", `${boxOf(anchor).x}px`);
      room.append(card);
      if (!still) list_.focus({ preventScroll: true });
    };
    const main = () => showRows(rows, null);
    const sub = (list: readonly Row[]) => showRows([{ spec: { label: "Back", icon: "arrow_back" }, act: main }, "separator", ...list], main);
    main();
    if (!still) document.addEventListener("pointerdown", outside, true);
  }

  /** Closes the windows of `key`: it stays, empty, unless it was its group's last, which turns into the ghost. */
  function closeWindows(band: Band, key: string): void {
    change(bands.map((each) => (each === band ? { ...each, spaces: settle(each.spaces.map((one) => (one.key === key ? { ...one, occupied: false } : one))) } : each)));
  }

  /** Closes the windows of `key` and takes it away, the later ones shifting down; a group's only workspace is emptied and stays. */
  function deleteWorkspace(band: Band, key: string): void {
    if (realOf(band).length <= 1) return closeWindows(band, key);
    if (key === active) {
      // The shown workspace's place shows what shifts into it.
      const at = band.spaces.findIndex((space) => space.key === key);
      active = band.spaces[at + 1]?.key ?? band.spaces[at - 1]!.key;
    }
    change(without(bands, key));
  }

  /** Closes every window in `band`, leaving it one empty workspace. */
  function deleteGroup(band: Band): void {
    const first = band.spaces[0]!;
    if (band.spaces.some((space) => space.key === active)) active = first.key;
    change(bands.map((each) => (each === band ? { ...each, spaces: [{ ...first, occupied: false }] } : each)));
  }

  /** Puts the name of group `block` up for typing over, in place. */
  function rename(block: number): void {
    renaming = block;
    render(false);
  }

  /** The name being typed over: Enter keeps it, Esc puts it back. */
  function editName(name: HTMLElement, entry: Entry, box: HTMLElement): void {
    name.contentEditable = "plaintext-only";
    name.spellcheck = false;
    const finish = (keep: boolean) => {
      if (renaming !== entry.block) return;
      renaming = null;
      const typed = (name.textContent ?? "").trim();
      if (keep && typed !== "") bands = bands.map((band) => (band.block === entry.block ? { ...band, name: typed } : band));
      render(true);
    };
    name.addEventListener("input", () => box.style.setProperty("--slot-width", String(widthOf({ ...entry, label: name.textContent ?? "" }, kanji))));
    name.addEventListener("keydown", (event) => {
      if (event.key === "Enter") finish(true);
      else if (event.key === "Escape") finish(false);
      else return;
      event.preventDefault();
    });
    name.addEventListener("blur", () => finish(false));
    requestAnimationFrame(() => {
      name.focus();
      const range = document.createRange();
      range.selectNodeContents(name);
      getSelection()?.removeAllRanges();
      getSelection()?.addRange(range);
    });
  }

  draw();

  if (options.live) {
    element.addEventListener("wheel", (event) => {
      event.preventDefault();
      if (carry) return;
      const keys = entries.filter((entry) => entry.kind === "cell" || entry.kind === "ghost").map((entry) => entry.space);
      const at = keys.indexOf(active);
      const step = event.deltaY > 0 ? 1 : -1;
      show(keys[(at + step + keys.length) % keys.length]!);
    }, { passive: false });

    let press: { key: string; at: Point; pointer: number } | null = null;
    let swallow = false;
    element.addEventListener("pointerdown", (event) => {
      swallow = false;
      if (event.button !== 0 || carry || renaming !== null) return;
      const box = (event.target as Element).closest<HTMLElement>(".shell-ws-slot");
      const entry = entries.find((each) => each.key === box?.dataset.key);
      if (entry?.kind === "cell" && entry.occupied) press = { key: entry.space, at: local(event), pointer: event.pointerId };
    });
    element.addEventListener("pointermove", (event) => {
      const point = local(event);
      if (carry) return carryTo(point);
      if (!press || event.pointerId !== press.pointer) return;
      if (Math.hypot(point.x - press.at.x, point.y - press.at.y) < cssPx("space-s8")) return;
      element.setPointerCapture(event.pointerId);
      lift(press.key, point);
      press = null;
    });
    const end = (event: PointerEvent) => {
      press = null;
      if (!carry || carry.landed) return;
      if (event.type === "pointercancel") propose(null, true);
      swallow = true;
      drop();
    };
    element.addEventListener("pointerup", end);
    element.addEventListener("pointercancel", end);
    // The release that drops a workspace is no click on what lies under it.
    element.addEventListener("click", (event) => {
      if (!swallow) return;
      swallow = false;
      event.stopPropagation();
    }, true);
  }

  const made = probe(element, { role: "component", name: "Workspaces", variant: kanji ? "kanji" : "", fixed: { width: false, height: true } });
  const still = options.drag;
  if (still) {
    // A still is laid out once it is on the page: the hole and the disc are placed from the slots.
    requestAnimationFrame(() => {
      const key = `w${still.id}`;
      lift(key, null, still.to ?? null);
      const centre = holeCentre();
      const off = cssPx("space-s12");
      if (carry) discTo(carry.disc, still.loose ? { x: centre.x + off, y: centre.y + off } : centre);
    });
  }
  if (!room) return made;
  room.className = "shell-ws-room";
  room.append(made);
  const open = options.menu;
  if (open) {
    requestAnimationFrame(() => {
      const key = open.workspace !== undefined ? `w${open.workspace}` : null;
      const band = open.group !== undefined ? bands.find((each) => each.block === open.group) : null;
      const anchorKey = key ?? (band?.name && band.borrowed ? `label${band.block}` : band?.spaces.at(-1)?.key);
      const anchor = content.querySelector<HTMLElement>(`.shell-ws-slot[data-key="${anchorKey}"]`);
      if (!anchor) return;
      openMenu(key ? { workspace: key } : { block: band!.block }, anchor, open.holding !== undefined ? { holding: open.holding } : {});
    });
  }
  return room;
}
