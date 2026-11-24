// The bar's workspaces, flat on the bar's glass: a row of cells `size.ws_cell` wide whatever their
// mark, so a switch moves nothing, each holding its workspace's mark (`Marks`, a setting: numbers,
// dots, dashes, or dot and dash), the shown one's lit in its tone. A digit is `workspace_digit`, 13
// and medium like the kanji in `accent`, one weight lit or not, with windows or empty, so nothing
// moves as it changes; an empty workspace's is `outline`, one with windows `on_surface_variant`.
// Groups come in block order, parted by the bar's hairline. A group waiting from a desk position no
// screen holds is borrowed: a stronger hairline before and after it, its name first in `label`,
// `secondary`. A group's workspaces are numbered from 1 with no gap, and a group with fewer than
// ten ends in its `+`: a faint `add` in `outline` at `opacity.placeholder`. The `+` and a borrowed
// group's name show only while the pointer is on the workspaces. Clicking a `+` adds a workspace
// at the next number and shows it; it stays, empty, until it is deleted.
//
// The shown workspace's mark is lit; switching, the leaving mark turns off over `short` and the
// arriving one turns on `duration.swap_lag` later, a dash also narrowing and widening over `medium`.
// Every change of the row glides: an entry grows in or shrinks out, its width over `long` on
// `emphasized_decel` (over `medium` on `standard_decel` while a workspace is carried), its opacity
// on `standard`; one growing in rounds its width up and one shrinking out rounds down, so the row
// holds its width to the pixel. A number that changes rolls, as the Overview's do
// (`lib/rolling-number.ts`): the same windows drawn at another number, a drop, a delete shifting
// the later ones down.
//
// The cell under the pointer takes a neutral circle a step up the container ladder,
// `surface_container_high`, and pressed `surface_container_highest`, fading over `short`, as any
// control's quiet hover does, while its mark dips as every bar item's content does; the part itself
// draws no line and no glow for the pointer. The shown workspace's cell takes none: it is already
// where you are. A `+` is a cell and takes it too.
//
// A workspace, empty or not, can be carried to another place; a `+` cannot. Pressed and moved
// `space.s8`, it lifts onto a disc raised to step 1, its number on it whatever the marks, that
// eases onto the pointer over `short` on `standard`, its place left as a hole, empty space a cell
// wide; nothing else moves. The disc is in the top layer, so nothing cuts it. The numbers are the
// places and stay; the windows move. While it is carried there is one hole: the row as it stood,
// the workspace taken out and the hole put where it would land, so one group gives up a slot as
// another takes one and the row keeps its width. Each place is hit-tested where the hole would
// stand there, with the Overview's magnet (`lib/magnet.ts`) on it: in a magnet the hole moves
// there, the cells between slide over and the disc snaps onto it, each over `medium` on
// `standard_decel`, the disc's number rolling to the one it would get there, so it lands already
// right; between magnets the hole stays and the disc follows the hand; off the part the hole goes
// home. Over another group, that group takes a `primary` outline, each slot drawing its piece;
// over a group's ghost, the hole is its last place; over a full group, ten already, the disc dims
// to `opacity.locked` and the hole is home, as the Overview refuses (`lib/magnet.ts`'s `refuse`).
// Let go, the disc flies into the hole over `medium` on `emphasized_decel`. A slide moves a slot's
// content, never its piece of a group's outline, so the outline stays whole, and a slot going out
// stays in its group. Carried itself, the shown workspace's place is a hole like any other's, and
// its mark is lit in the cell it lands in. Nothing in the part is selected by a press or a drag.
//
// A right click opens its menu (`lib/menu.ts`) on a popover under it, the workspace it is for marked
// by a circle of `primary` at `opacity.tint` behind its mark, fading over `short` on `standard`
// (none on the shown one, already lit): on a workspace, its own
// actions under "Workspace N", then its group's under "This group"; on a group's name or its ghost,
// the group's alone, under its name. The deletes are held. Delete group closes every window in the
// group: the screen's own group keeps one empty workspace and never goes; a borrowed group goes
// whole, its rule, name and workspaces shrinking out, until its own screen is connected again.
// Move to group and Move to screen open their lists in place, Back first, as every submenu.
//
// It isn't the library's `Pill`: its cells glide every frame. It has no part in `src/lib/` yet.

import "./workspaces.css";
import type { Pinned } from "../../lib/control.ts";
import { pin } from "../../lib/control.ts";
import { raise } from "../../lib/elevation.ts";
import { focusRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { withHint } from "../../lib/hint.ts";
import { catchShare, caught, type Rect, refuse, type Zone, zoneOf } from "../../lib/magnet.ts";
import { type MenuEntry, type MenuItemSpec, menu } from "../../lib/menu.ts";
import { declare, probe } from "../../lib/probe.ts";
import { type Rolling, rollingNumber } from "../../lib/rolling-number.ts";
import { advance, clamped, text, type TypeStyle } from "../../lib/text.ts";
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
  /** Held over group `over`, a full one, which refuses it: it dims there, its hole home. */
  readonly over?: number;
}

/** A still of the menu open: on `workspace`, or on `group`'s name, its delete held `holding` of the way. */
export interface MenuPin {
  readonly workspace?: number;
  readonly group?: number;
  readonly holding?: number;
}

/**
 * How a workspace is marked, a personalisation setting: its number (digits or kanji, all of them
 * or only the shown one's), a dot, a dash, or a dot that the shown one stretches into a dash.
 */
export type Marks = "numbers" | "dots" | "dashes" | "dotdash";

/** The marks, the setting's default first. */
export const MARKS: readonly Marks[] = ["dotdash", "numbers", "dots", "dashes"];

export interface WorkspacesOptions {
  readonly groups?: readonly Group[];
  /** The shown workspace's id. */
  readonly active?: number;
  /** The marks: Dot and dash unless given. */
  readonly marks?: Marks;
  /** Numbers in kanji rather than digits. */
  readonly kanji?: boolean;
  /** Numbers: only the shown workspace's at rest, the rest while the pointer is on them. */
  readonly onlyCurrent?: boolean;
  /** A workspace asking for attention, by id: its mark lit in `error`. */
  readonly urgent?: number;
  /** Clicks and the wheel move the shown workspace, a group's `+` adds one; a drag reorders, a right click opens the menu. */
  readonly live?: boolean;
  readonly pinned?: Pinned;
  /** The workspace whose cell the pointer is on while pinned hovered or pressed: the one after the shown one, unless named. */
  readonly pointer?: number;
  readonly drag?: DragPin;
  readonly menu?: MenuPin;
}

/**
 * The shots' stand-ins: 1 with a window, 2 shown with one, 3 the ghost; Desk's 11 with a window, 12
 * its ghost. A group's last workspace, empty and not shown, is read as the group's ghost, its `+`.
 */
export const SHOTS_GROUPS: readonly Group[] = [
  { block: 0, spaces: [{ id: 1, occupied: true }, { id: 2, occupied: true }, { id: 3, occupied: false }] },
  { block: 1, borrowed: true, name: "Desk", spaces: [{ id: 11, occupied: true }, { id: 12, occupied: false }] },
];

/** The screens Move to screen lists: the Overview's stand-ins. */
const SCREENS = ["Laptop", "External 1"];

const KANJI = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];

/**
 * A workspace in the model: its id, and its windows as `key`, which stays with them as they move. A
 * group's workspaces are numbered from its block's first, one after the other, with no gap.
 */
interface Place {
  readonly id: number;
  readonly key: string;
  readonly occupied: boolean;
}

interface Band {
  readonly block: number;
  readonly borrowed: boolean;
  readonly name: string;
  /** Its workspaces, every one a real one: its `+` is drawn after them, not kept among them. */
  readonly spaces: readonly Place[];
}

/** The most workspaces a group holds: a block's ten. */
const MOST = 10;

type Kind = "cell" | "ghost" | "hole" | "gap" | "label";

interface Entry {
  readonly key: string;
  readonly kind: Kind;
  readonly block: number;
  readonly workspace: number;
  /** The windows' key of a cell or a hole; the group's first for its name; the group's `+` key for its ghost. */
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

/** A numeral's style: one for every digit, lit or not, with windows or empty, so none moves as it changes. */
function numberStyle(kanji: boolean): TypeStyle {
  return kanji ? "accent" : "workspace_digit";
}

/** The key of group `block`'s `+`. */
function plusKey(block: number): string {
  return `plus@${block}`;
}

/**
 * What the pill draws, in order, as `Workspaces.qml`'s `entries`; `held`'s cell as its hole. A group
 * with room ends in its `+`, unless `plus` says otherwise: while a workspace is carried, a group
 * keeps or lacks its `+` as it did at the lift, so the pill keeps its width.
 */
function entriesOf(bands: readonly Band[], held: string | null, taking: number | null, plus: (band: Band) => boolean = (band) => band.spaces.length < MOST): Entry[] {
  const result: Entry[] = [];
  const blank = { workspace: 0, space: "", borrowed: false, head: false, tail: false, label: "", occupied: false, taking: false };
  bands.forEach((band, index) => {
    const name = band.borrowed ? band.name : "";
    const takes = band.block === taking;
    const ends = plus(band);
    const first = band.block * 10 + 1;
    // A borrowed group is set off by a stronger hairline before it and after it.
    if (index > 0) result.push({ ...blank, key: `gap${band.block}`, kind: "gap", block: band.block, borrowed: band.borrowed || (bands[index - 1]?.borrowed ?? false) });
    if (name !== "") result.push({ ...blank, key: `label${band.block}`, kind: "label", block: band.block, workspace: first, space: band.spaces[0]?.key ?? "", borrowed: true, head: true, label: name, taking: takes });
    band.spaces.forEach((space, at) => {
      // A slot's key is its windows' in its group: carried to another group, it leaves one and
      // grows in the other, never sliding across the rule between the groups.
      result.push({ key: `${space.key}@${band.block}`, kind: space.key === held ? "hole" : "cell", block: band.block, workspace: space.id, space: space.key, borrowed: band.borrowed, head: at === 0 && name === "", tail: at === band.spaces.length - 1 && !ends, label: "", occupied: space.occupied, taking: takes });
    });
    if (ends) result.push({ key: plusKey(band.block), kind: "ghost", block: band.block, workspace: first + band.spaces.length, space: plusKey(band.block), borrowed: band.borrowed, head: band.spaces.length === 0 && name === "", tail: true, label: "", occupied: false, taking: takes });
    if (band.borrowed && index === bands.length - 1) result.push({ ...blank, key: `end${band.block}`, kind: "gap", block: band.block, borrowed: true });
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
  return (px("size-ws_cell") - advance(numberStyle(kanji), numberText(id, kanji))) / 2;
}

/** An entry's width in logical pixels, as `widthOf`. */
function widthOf(entry: Entry, kanji: boolean): number {
  if (entry.kind === "label") {
    const inset = Math.round(numberRoom(entry.workspace, kanji));
    const name = Math.min(advance("label", entry.label), px("space-s48") * 2);
    return Math.round(2 * inset + name - numberRoom(entry.workspace, kanji));
  }
  return entry.kind === "gap" ? px("space-s20") : px("size-ws_cell");
}

/** The duration token `name` in milliseconds under the speed setting, nothing with reduce motion. */
function ms(name: string): number {
  return settings().reduced ? 0 : durationMs(name) / settings().speed;
}

function easing(name: string): string {
  return `cubic-bezier(${cubic(`easing-${name}`).join(", ")})`;
}

/** `list` in `band`'s places in order, numbered from its block's first: windows moving onto the numbers, the later ones shifting over. */
function renumber(band: Band, list: readonly Place[]): Place[] {
  return list.map((space, at) => ({ ...space, id: band.block * 10 + at + 1 }));
}

/** Where a carried workspace's hole stands, by its group's block and its place there. */
function landingOf(bands: readonly Band[], key: string): Landing | null {
  for (const band of bands) {
    const index = band.spaces.findIndex((space) => space.key === key);
    if (index >= 0) return { block: band.block, index };
  }
  return null;
}

/**
 * How far inside its cell the first workspace's mark starts, as a length's CSS: the bar spaces its
 * start from the mark's ink, not the cell's edge. A numeral's is its advance's, the one figure the
 * layout can't express.
 */
function inkInset(marks: Marks, kanji: boolean): string {
  if (marks === "dashes") return "calc((var(--size-ws_cell) - var(--size-ws_dash)) / 2)";
  if (marks !== "numbers") return "calc((var(--size-ws_cell) - var(--size-ws_dot)) / 2)";
  const room = (px("size-ws_cell") - advance(numberStyle(kanji), numberText(1, kanji))) / 2;
  return `calc(${room} * var(--px))`;
}

export function workspaces(options: WorkspacesOptions = {}): HTMLElement {
  const marks = options.marks ?? "dotdash";
  const kanji = marks === "numbers" && (options.kanji ?? false);
  let mint = 0;
  const fresh = () => `n${mint++}`;
  let active = `w${options.active ?? 2}`;
  const pointer = options.pointer ?? (options.active ?? 2) + 1;
  let bands: Band[] = (options.groups ?? SHOTS_GROUPS).map((group) => {
    const spaces = group.spaces.map((space) => ({ id: space.id, key: `w${space.id}`, occupied: space.occupied }));
    const last = spaces.at(-1);
    // A group's last workspace, empty and not shown, stands for its `+`, which is drawn, not kept.
    if (last && spaces.length > 1 && !last.occupied && last.key !== active) spaces.pop();
    return { block: group.block, borrowed: group.borrowed ?? false, name: group.name ?? "", spaces };
  });
  const element = document.createElement("span");
  element.className = "shell-ws k-control";
  element.dataset.hot = "";
  element.dataset.marks = marks;
  element.toggleAttribute("data-only-current", marks === "numbers" && (options.onlyCurrent ?? false));
  element.style.setProperty("--ws-ink", inkInset(marks, kanji));
  pin(element, options.pinned);
  const cells = document.createElement("span");
  cells.className = "shell-ws-cells";
  const content = document.createElement("span");
  content.className = "shell-ws-row shell-ws-content";
  cells.append(content);
  element.append(cells);
  focusRing(element, "full", "accent", false);
  if (options.pinned) withHint(element, "Workspaces");

  /** The carried workspace: its windows' key, the groups as they stood at the lift, and where it would land. */
  /**
   * The carried workspace: its windows' key, the groups as they stood at the lift, where its one
   * hole stands, whether the disc is snapped onto it, and where the disc is held when it is drawn
   * at a place of the pill's rather than at the hand.
   */
  type Carry = { key: string; from: Band[]; home: Landing; landing: Landing; snapped: boolean; disc: HTMLElement; landed: boolean; anchor: (() => Point) | null; number: number; roll: Rolling | null };
  let carry: Carry | null = null;
  /** The group whose name is being typed over, by block. */
  let renaming: number | null = null;
  let entries: Entry[] = [];

  function entriesNow(): Entry[] {
    const taking = carry && carry.landing.block !== carry.home.block ? carry.landing.block : null;
    return entriesOf(bands, carry?.key ?? null, taking, carry ? plusAtLift(carry.from) : undefined);
  }

  /** Whether a group ends in its `+` while a workspace is carried: as it did at the lift. */
  function plusAtLift(from: readonly Band[]): (band: Band) => boolean {
    return (band) => (from.find((each) => each.block === band.block)?.spaces.length ?? 0) < MOST;
  }

  /** A group left with no workspace keeps an empty one: a group always has its first. */
  function kept(band: Band, list: readonly Place[]): Place[] {
    return renumber(band, list.length > 0 ? list : [{ id: 0, key: fresh(), occupied: false }]);
  }

  /** The groups with `key`'s windows taken from their place, the later ones shifting down. */
  function without(from: readonly Band[], key: string): Band[] {
    return from.map((band) => (band.spaces.some((space) => space.key === key) ? { ...band, spaces: kept(band, band.spaces.filter((space) => space.key !== key)) } : band));
  }

  /** The groups with `key`'s windows carried to `landing`, the ones between shifting over by one. */
  function placed(from: readonly Band[], key: string, landing: Landing | null): Band[] {
    if (!landing) return [...from];
    const moving = from.flatMap((band) => band.spaces).find((space) => space.key === key);
    if (!moving) return [...from];
    const source = from.find((band) => band.spaces.includes(moving))!;
    return from.map((band) => {
      const list = band.spaces.filter((space) => space.key !== key);
      if (band.block === landing.block) list.splice(Math.min(landing.index, list.length), 0, moving);
      return band === source || band.block === landing.block ? { ...band, spaces: kept(band, list) } : band;
    });
  }

  /**
   * The groups as drawn while `key` is carried: as they stood at the lift with it taken out and its
   * one hole put at `landing`, nothing else tidied. One group gives up a slot as another takes one,
   * so the pill is as wide wherever the hole is.
   */
  function carried(from: readonly Band[], key: string, landing: Landing): Band[] {
    const moving = from.flatMap((band) => band.spaces).find((space) => space.key === key)!;
    return from.map((band) => {
      const list = band.spaces.filter((space) => space.key !== key);
      if (band.block === landing.block) list.splice(Math.min(landing.index, list.length), 0, moving);
      return list.length === band.spaces.length && band.block !== landing.block ? band : { ...band, spaces: renumber(band, list) };
    });
  }

  const choose = (key: string) => {
    if (options.live && !carry && renaming === null) show(key);
  };

  /**
   * One entry's slot: its piece of the drop outline behind, and its content. A cell's
   * number that stood at `was` for the same windows rolls from it to its own.
   */
  function slot(entry: Entry, was?: number): HTMLElement {
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
    if (entry.taking) piece("shell-ws-drop");
    if (entry.kind === "gap") {
      const rule = document.createElement("span");
      rule.className = "shell-ws-rule";
      rule.toggleAttribute("data-strong", entry.borrowed);
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
      // A name that varies: cut with an ellipsis at the slot's widest, `space.s48` twice.
      const name = declare(clamped(entry.label, "label", "secondary"), { variable: true });
      name.classList.add("shell-ws-name");
      name.style.setProperty("--inset", String(Math.round(numberRoom(entry.workspace, kanji))));
      box.append(name);
      if (renaming === entry.block) editName(name, entry, box);
      else box.addEventListener("click", () => choose(entry.space));
      return box;
    }
    if (entry.kind === "hole") {
      // The hole stands for the number the disc shows, so the cell landing there has nothing to roll.
      if (carry) box.dataset.number = String(carry.landing.index + 1);
      return box;
    }
    const cell = document.createElement("span");
    cell.className = "shell-ws-cell";
    cell.dataset.space = entry.space;
    cell.toggleAttribute("data-shown", entry.space === active);
    cell.toggleAttribute("data-urgent", entry.kind === "cell" && entry.workspace === options.urgent && entry.space !== active);
    cell.toggleAttribute("data-occupied", entry.occupied);
    cell.toggleAttribute("data-pointer", entry.workspace === pointer);
    cell.toggleAttribute("data-ghost", entry.kind === "ghost");
    if (entry.kind === "ghost") {
      const plus = icon("add", "var(--icon-s16)", "var(--color-outline)");
      plus.classList.add("shell-ws-ghost");
      cell.append(plus);
      // Its cell is the box the `+` is centred in, whatever marks the others show.
      probe(cell, { role: "box" });
    } else {
      const mark = document.createElement("span");
      mark.className = "shell-ws-mark";
      if (marks === "numbers") mark.append(numberOf(entry, was), litNumber(entry));
      cell.append(mark);
      box.dataset.number = String(slotOf(entry.workspace));
    }
    cell.addEventListener("click", () => choose(entry.space));
    box.append(cell);
    return box;
  }

  /**
   * A cell's number, rolling from `was` when it stood at another for the same windows, as the
   * Overview's numbers do; a kanji is set in place.
   */
  function numberOf(entry: Entry, was?: number): HTMLElement {
    const style = numberStyle(kanji);
    const color = entry.occupied ? "on_surface_variant" : "outline";
    const number = document.createElement("span");
    number.className = "shell-ws-number";
    number.dataset.look = `${style} ${color}`;
    const value = slotOf(entry.workspace);
    const rolling = numeral(was ?? value, style, color);
    number.append(rolling.element);
    rolling.set(value);
    return number;
  }

  /**
   * A number's light, over it: the same numeral, `primary` (an urgent one's `error`), with its glow
   * by night, in the unlit numeral's style. It shows as far as the cell is lit, the unlit numeral fading under it.
   */
  function litNumber(entry: Entry): HTMLElement {
    const urgent = entry.workspace === options.urgent;
    const lit = text(numberText(entry.workspace, kanji), numberStyle(kanji), urgent ? "error" : "primary");
    lit.classList.add("shell-ws-lit-number");
    return lit;
  }

  /**
   * A workspace's number, 1 to 10, as a rolling number: its digits, or its kanji numeral (十 for
   * ten), one glyph to a slot. Every number centres on the same figure, not its own: a "1" and a
   * "2" weigh differently, and each centred on its own ink would put the numbers on different
   * baselines. One figure at least: the box is its figures', so a cell centres a 10 as it does a 1.
   */
  function numeral(value: number, style: TypeStyle, color: string): Rolling {
    const centre = { style: numberStyle(kanji), words: kanji ? KANJI[KANJI.length - 1]! : "0" };
    if (!kanji) return rollingNumber({ value, widest: "0", style, color, centre });
    return rollingNumber({ value, widest: KANJI[0]!, format: (each) => KANJI[each - 1]!, figures: KANJI.join(""), style, color, centre });
  }

  /** Draws the entries at once, nothing moving. */
  function draw(): void {
    entries = entriesNow();
    content.replaceChildren(...entries.map((entry) => slot(entry)));
  }

  /** The drawn horizontal offset of `child`'s content: its box, and where a slide running on it has it. */
  function drawnLeft(child: HTMLElement): number {
    return child.offsetLeft + (Number.parseFloat(getComputedStyle(child).getPropertyValue("--ws-slide")) || 0) * cssPx("size-ws_cell");
  }

  /** How far `child` is in, 0 to 1: its width's share, and its content's opacity. */
  function presence(child: HTMLElement): { present: number; shown: number } {
    const drawn = getComputedStyle(child);
    return { present: Number.parseFloat(drawn.getPropertyValue("--present")) || 0, shown: Number.parseFloat(drawn.getPropertyValue("--ws-shown")) || 0 };
  }

  /**
   * Grows `child` in, or shrinks it out (`to` 0, then it goes), from how far in it is: its width
   * over `duration` on `curve`, its opacity on `standard`. A slot growing rounds its width up and
   * one shrinking rounds down, so two moving on one clock, one in as the other goes, always add up
   * to a whole slot and the row's width holds to the pixel.
   */
  function grow(child: HTMLElement, from: { present: number; shown: number }, to: 0 | 1, duration: string, curve: string): void {
    child.toggleAttribute("data-gone", to === 0);
    child.toggleAttribute("data-coming", to === 1);
    const timing = { duration: ms(duration), fill: "forwards" as const };
    child.animate([{ "--ws-shown": from.shown }, { "--ws-shown": to }], { ...timing, easing: easing("standard") });
    child.animate([{ "--present": from.present }, { "--present": to }], { ...timing, easing: easing(curve) }).finished.then(
      () => (to === 0 ? child.remove() : delete child.dataset.coming),
      () => {},
    );
  }

  /**
   * Hands `made`, a slot drawn again, what is still running on `stays`, the one it stands in for, at
   * the same point in time: a slot still growing in goes on growing on one clock with the slot
   * going out beside it, instead of standing whole at once.
   */
  function carryOn(stays: HTMLElement, made: HTMLElement): void {
    const coming = stays.hasAttribute("data-coming");
    made.toggleAttribute("data-coming", coming);
    for (const running of stays.getAnimations()) {
      const effect = running.effect as KeyframeEffect | null;
      if (!effect || running.playState === "finished") continue;
      const copy = made.animate(effect.getKeyframes().map(({ offset, easing: ease, ...rest }) => ({ ...rest, offset, easing: ease })), effect.getTiming());
      // HAZARD: copy the start time, not the current time: a current time set on a new animation
      // starts it a frame later, a frame behind the slot going out beside it, and the row's width
      // dips by that frame's worth.
      if (running.startTime !== null) copy.startTime = running.startTime;
      else copy.currentTime = running.currentTime;
      if (coming) copy.finished.then(() => delete made.dataset.coming, () => {});
    }
  }

  /**
   * Draws the entries again: slots that stay keep their place, their content sliding to their new
   * one over `medium` on `standard_decel` when `slides`; new ones grow in and gone ones shrink out,
   * over `medium` on `standard_decel` while a workspace is carried, else over `long` on
   * `emphasized_decel`. A slot coming back while it is still going grows on from where it is.
   */
  function render(slides: boolean): void {
    const [duration, curve] = carry ? ["medium", "standard_decel"] : ["long", "emphasized_decel"];
    const children = [...content.children] as HTMLElement[];
    const going = new Map(children.filter((child) => child.hasAttribute("data-gone")).map((child) => [child.dataset.key!, child]));
    const old = new Map(children.filter((child) => !child.hasAttribute("data-gone")).map((child) => [child.dataset.key!, child]));
    const was = new Map([...old].map(([key, child]) => [key, drawnLeft(child)]));
    const at = new Map(children.map((child, index) => [child, index]));
    // The numbers the windows stood at, wherever they were drawn: a number that changes rolls.
    const numbers = new Map(children.filter((child) => child.dataset.number && !child.hasAttribute("data-gone")).map((child) => [child.dataset.key!.split("@")[0]!, Number(child.dataset.number)]));
    entries = entriesNow();
    const wanted = new Set(entries.map((entry) => entry.key));
    const next: HTMLElement[] = [];
    const kept: [string, HTMLElement][] = [];
    for (const entry of entries) {
      const made = slot(entry, numbers.get(entry.space));
      const stays = old.get(entry.key);
      if (stays) {
        old.delete(entry.key);
        kept.push([entry.key, made]);
        carryOn(stays, made);
        // A number standing as it was keeps its element, so a roll still running runs on.
        const before = stays.querySelector<HTMLElement>(".shell-ws-number");
        const now = made.querySelector<HTMLElement>(".shell-ws-number");
        if (before && now && before.dataset.look === now.dataset.look && numbers.get(entry.space) === slotOf(entry.workspace)) now.replaceWith(before);
      } else {
        const back = going.get(entry.key);
        going.delete(entry.key);
        grow(made, back ? presence(back) : { present: 0, shown: 0 }, 1, duration, curve);
      }
      next.push(made);
    }
    const leaving = [...going.values()];
    for (const [key, gone] of old) {
      if (wanted.has(key)) continue;
      grow(gone, presence(gone), 0, duration, curve);
      leaving.push(gone);
    }
    // A slot going out keeps its group's outline only while its group still takes the drop.
    const taking = entries.find((entry) => entry.taking)?.block;
    for (const gone of leaving) if (Number(gone.dataset.block) !== taking) gone.querySelector(".shell-ws-drop")?.remove();
    // A slot going out stays in its group: just before what followed it, or after what came before.
    const order = [...at.keys()];
    for (const gone of leaving.sort((a, b) => at.get(a)! - at.get(b)!)) {
      const from = at.get(gone)!;
      const after = order.slice(from + 1).find((each) => next.some((made) => made.dataset.key === each.dataset.key && !made.hasAttribute("data-gone")));
      const before = order.slice(0, from).reverse().find((each) => next.some((made) => made.dataset.key === each.dataset.key));
      const anchor = after ? next.findIndex((made) => made.dataset.key === after.dataset.key && !made.hasAttribute("data-gone")) : before ? next.findIndex((made) => made.dataset.key === before.dataset.key) + 1 : next.length;
      next.splice(anchor, 0, gone);
    }
    content.replaceChildren(...next);
    if (!slides) return;
    for (const [key, made] of kept) {
      const from = was.get(key)! - made.offsetLeft;
      // Only a workspace's slot slides, and only by whole slots: a group's name and the rule move
      // with the widths round them, and a pixel left by a slot still growing or going is no slide.
      const kind = made.dataset.kind;
      if (kind === "label" || kind === "gap" || Math.abs(from) < cssPx("size-ws_cell") / 2 || ms("medium") === 0) continue;
      // Its content slides out of its box, which shows it while it does.
      made.dataset.sliding = "";
      // In cells: a slide is always by whole places.
      made.animate([{ "--ws-slide": from / cssPx("size-ws_cell") }, { "--ws-slide": 0 }], { duration: ms("medium"), easing: easing("standard_decel") }).finished.then(() => delete made.dataset.sliding, () => {});
    }
  }

  /**
   * Shows the workspace of `key`. A group's `+` adds a workspace after its last, at the next number,
   * and shows it; it stays, empty, until it is deleted.
   */
  function show(key: string): void {
    if (key === active) return;
    const adding = bands.find((band) => plusKey(band.block) === key);
    if (adding) {
      if (adding.spaces.length >= MOST) return;
      const made = { id: 0, key: fresh(), occupied: false };
      bands = bands.map((band) => (band === adding ? { ...band, spaces: renumber(band, [...band.spaces, made]) } : band));
      active = made.key;
    } else {
      // The marks stay where they are: the leaving one turns off and the arriving one on (`light`).
      active = key;
      return light();
    }
    render(false);
    // A workspace just added lights as any arriving mark does, once it is drawn unlit.
    const added = content.querySelector<HTMLElement>(`.shell-ws-cell[data-space="${active}"]`);
    if (!added) return;
    delete added.dataset.shown;
    requestAnimationFrame(() => requestAnimationFrame(light));
  }

  /** Lights the shown workspace's mark and puts the others out, each fading as its CSS says. */
  function light(): void {
    for (const cell of content.querySelectorAll<HTMLElement>(".shell-ws-cell")) cell.toggleAttribute("data-shown", cell.dataset.space === active);
  }

  /** The groups changed by an action, drawn again with every change gliding. */
  function change(next: Band[]): void {
    bands = next;
    if (!bands.some((band) => band.spaces.some((space) => space.key === active))) active = bands[0]!.spaces[0]!.key;
    render(true);
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

  /** The middle of the carried workspace's hole, where it stands once laid out, whatever is still growing or going. */
  function holeCentre(): Point {
    const hole = slotsOf(entries).find(({ entry }) => entry.kind === "hole");
    return hole ? { x: hole.box.x + hole.box.w / 2, y: hole.box.y + hole.box.h / 2 } : { x: 0, y: 0 };
  }

  /** The slots of the cells, ghosts and holes `list` draws, in the pill's CSS pixels: where they stand once laid out. */
  function slotsOf(list: readonly Entry[]): { entry: Entry; box: Rect }[] {
    const scale = settings().scale;
    const h = px("size-ws_cell") * scale;
    let x = cells.offsetLeft;
    const result: { entry: Entry; box: Rect }[] = [];
    for (const entry of list) {
      const w = widthOf(entry, kanji) * scale;
      if (entry.kind === "cell" || entry.kind === "ghost" || entry.kind === "hole") result.push({ entry, box: { x, y: cells.offsetTop, w, h } });
      x += w;
    }
    return result;
  }

  /** Where each group's entries run on the pill, its name to its `+`, in the pill's CSS pixels. */
  function spansOf(list: readonly Entry[]): Map<number, Rect> {
    const scale = settings().scale;
    const h = px("size-ws_cell") * scale;
    const spans = new Map<number, Rect>();
    let x = cells.offsetLeft;
    for (const entry of list) {
      const w = widthOf(entry, kanji) * scale;
      if (entry.kind !== "gap") {
        const had = spans.get(entry.block);
        spans.set(entry.block, had ? { ...had, w: x + w - had.x } : { x, y: cells.offsetTop, w, h });
      }
      x += w;
    }
    return spans;
  }

  /** What a place proposes: a landing, or a full group's refusal. */
  type Aim = Landing | "refused";

  /** The zones worked out for a carry: the groups do not change while a workspace is carried. */
  let zoneCache: { readonly key: string; readonly from: readonly Band[]; readonly zones: readonly Zone<Aim>[] } | null = null;

  /**
   * Every place the carried workspace can land in, hit-tested where its hole would stand if it were
   * there: the layout drawn with the hole at that place, so the hole opens exactly where the
   * pointer found it and a pointer that stays never changes the answer. Each place's zone is the
   * Overview's magnet on the hole's box. A group's ghost is a place too: its last, the ghost's box
   * as it stands with the hole there. A full group, ten already, offers no place to a workspace from
   * another: held anywhere over it, as it stands, the workspace is refused, as the Overview's are.
   */
  function zones(): readonly Zone<Aim>[] {
    if (!carry) return [];
    const held = carry;
    if (zoneCache?.key === held.key && zoneCache.from === held.from) return zoneCache.zones;
    const result: Zone<Aim>[] = [];
    const rest = spansOf(entriesOf(carried(held.from, held.key, held.home), held.key, null, plusAtLift(held.from)));
    for (const band of held.from) {
      const room = band.spaces.filter((space) => space.key !== held.key).length;
      if (band.block !== held.home.block && room >= MOST) {
        const span = rest.get(band.block);
        if (span) result.push(zoneOf<Aim>(span, "refused", 1));
        continue;
      }
      for (let index = 0; index <= room; index++) {
        const landing = { block: band.block, index };
        const layout = slotsOf(entriesOf(carried(held.from, held.key, landing), held.key, null, plusAtLift(held.from)));
        const hole = layout.find(({ entry }) => entry.kind === "hole")!;
        result.push(zoneOf<Aim>(hole.box, landing, catchShare(hole.box)));
        const ghost = index === room ? layout.find(({ entry }) => entry.kind === "ghost" && entry.block === band.block) : undefined;
        if (ghost) result.push(zoneOf<Aim>(ghost.box, landing, catchShare(ghost.box)));
      }
    }
    zoneCache = { key: held.key, from: held.from, zones: result };
    return result;
  }

  /** The disc's place: where it is drawn from, and the offset still easing out of it. */
  let base: Point = { x: 0, y: 0 };
  let easings: Animation[] = [];

  /**
   * A point in the pill's CSS pixels, from the viewport's corner in the disc's own pixels: the page
   * may be zoomed, and the disc, fixed to the viewport, is zoomed with it, so the pill's corner is
   * taken out of the zoom the pill is drawn at.
   */
  function client(point: Point): Point {
    const box = element.getBoundingClientRect();
    const unzoom = element.offsetWidth / Math.max(1, box.width);
    return { x: box.left * unzoom + point.x, y: box.top * unzoom + point.y };
  }

  /** Puts the disc's base at `at`, a point of the pill's; with `over`, it eases there from where it is drawn now. */
  function discTo(disc: HTMLElement, at: Point, over?: { duration: string; curve: string }): Animation | null {
    const to = client(at);
    const drawn = getComputedStyle(disc);
    const glide = Number.parseFloat(drawn.getPropertyValue("--ws-glide")) || 0;
    const offset = (name: string) => Number.parseFloat(drawn.getPropertyValue(name)) || 0;
    const now = { x: base.x + offset("--ws-ox") * glide, y: base.y + offset("--ws-oy") * glide };
    base = to;
    disc.style.setProperty("--ws-x", String(to.x));
    disc.style.setProperty("--ws-y", String(to.y));
    if (!over) return null;
    for (const each of easings) each.cancel();
    // How far it has to come, fixed as it starts; `--ws-glide` takes it from all of that to none.
    disc.style.setProperty("--ws-ox", String(now.x - to.x));
    disc.style.setProperty("--ws-oy", String(now.y - to.y));
    easings = [disc.animate([{ "--ws-glide": 1 }, { "--ws-glide": 0 }], { duration: ms(over.duration), easing: easing(over.curve) })];
    return easings[0]!;
  }

  /**
   * The disc the carried workspace rides on, its number the one it would get where it is proposed:
   * a rolling number, as the cells' are, that rolls as the place changes. A kanji is set in place.
   */
  function makeDisc(id: number): { disc: HTMLElement; roll: Rolling } {
    const disc = raise(document.createElement("span"), 1);
    disc.className += " shell-ws-disc";
    const roll = numeral(slotOf(id), numberStyle(kanji), "on_surface");
    disc.append(roll.element);
    // In the top layer, the disc is never cut by the pill or by any box the pill is drawn in; it
    // stays in the pill's tree, so it is drawn in the pill's own tokens.
    // It floats over the page, out of the pill's box, as a select's open list does: a plain box,
    // held to its own boxes rather than the pill's.
    probe(disc, { role: "box" }, { floats: true });
    disc.popover = "manual";
    element.append(disc);
    // A still's pill may not be on the page yet: its disc shows once it is (`onPage`).
    if (element.isConnected) disc.showPopover();
    return { disc, roll };
  }

  /**
   * Lifts the workspace of `key` onto its disc, its place left as its hole: nothing else moves.
   * Under `pointer`, the disc comes up over `short` on `standard`, onto the hole if the pointer is
   * in its magnet, else to the hand. A still is drawn with its hole at `landing`.
   */
  function lift(key: string, pointer: Point | null, landing: Landing | null = null): void {
    const home = landingOf(bands, key)!;
    const number = bands.flatMap((band) => band.spaces).find((each) => each.key === key)!.id;
    const { disc, roll } = makeDisc(number);
    carry = { key, from: bands, home, landing: home, snapped: false, disc, roll, landed: false, anchor: null, number };
    zoneCache = null;
    element.dataset.carrying = "";
    render(false);
    base = client(holeCentre());
    discTo(carry.disc, holeCentre());
    if (!pointer) return landing ? propose(landing, false) : draw();
    const aimed = caught(pointer, zones());
    carry.snapped = aimed !== "refused" && same(aimed, home);
    discTo(carry.disc, carry.snapped ? holeCentre() : pointer, { duration: "short", curve: "standard" });
  }

  /** The hole moved to `landing`: the cells between slide over, a slot given up by one group as another takes it. */
  function propose(landing: Landing, slides: boolean): void {
    if (!carry) return;
    carry.landing = landing;
    if (slides) carry.roll?.set(landing.index + 1);
    else if (carry.roll) {
      // A still shows the disc's number as it stands there, rolled already.
      const now = numeral(landing.index + 1, numberStyle(kanji), "on_surface");
      carry.roll.element.replaceWith(now.element);
      carry.roll = now;
    }
    bands = carried(carry.from, carry.key, landing);
    if (!slides) return draw();
    render(true);
  }

  function same(a: Landing | null, b: Landing | null): boolean {
    return a?.block === b?.block && a?.index === b?.index;
  }

  /** Whether `point` is on the pill. */
  function onPill(point: Point): boolean {
    return point.x >= 0 && point.y >= 0 && point.x < element.offsetWidth && point.y < element.offsetHeight;
  }

  /**
   * The pointer at `point` while carrying: in a place's magnet it proposes that place and the disc
   * snaps onto the hole there; between magnets on the pill, the hole stays where it was last
   * proposed and the disc follows the hand; off the pill, the hole goes home. The hole moving, or
   * the disc snapping on or off, eases the disc over `medium` on `standard_decel`.
   */
  function carryTo(point: Point): void {
    if (!carry || carry.landed) return;
    const aimed = caught(point, zones());
    const refused = aimed === "refused";
    refuse(carry.disc, refused);
    const magnet = refused ? null : aimed;
    const landing = refused ? carry.home : (magnet ?? (onPill(point) ? carry.landing : carry.home));
    const moved = !same(landing, carry.landing);
    if (moved) propose(landing, true);
    const snapped = magnet !== null;
    const glide = moved || snapped !== carry.snapped;
    carry.snapped = snapped;
    discTo(carry.disc, snapped ? holeCentre() : point, glide ? { duration: "medium", curve: "standard_decel" } : undefined);
  }

  /** Let go: the disc flies into the hole over `medium` on `emphasized_decel`, and the hole becomes the workspace there. */
  function drop(): void {
    if (!carry || carry.landed) return;
    const held = carry;
    held.landed = true;
    const flight = discTo(held.disc, holeCentre(), { duration: "medium", curve: "emphasized_decel" });
    // Landed, the move is made: the cell there already shows the number the disc carried.
    const done = () => {
      if (carry !== held) return;
      held.disc.remove();
      bands = placed(held.from, held.key, held.landing);
      carry = null;
      delete element.dataset.carrying;
      render(false);
    };
    if (flight) flight.finished.then(done, done);
    else done();
  }

  // ---------------------------------------------------------------- the menu --

  /** The popover's room: the pill, its menu under it. */
  const room = options.live || options.menu ? document.createElement("span") : null;
  let card: HTMLElement | null = null;

  /** The workspace a menu is open for, marked so it reads as the menu's. */
  let marked: HTMLElement | null = null;

  function closeMenu(): void {
    card?.remove();
    card = null;
    if (marked) delete marked.dataset.menu;
    marked = null;
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
        { spec: { label: "Move to group", submenu: true, disabled: others.length === 0 }, act: () => sub(others.map((each) => ({ spec: { label: groupName(each), variable: true, disabled: each.spaces.length >= MOST }, act: () => change(placed(bands, key, { block: each.block, index: each.spaces.length })) }))) },
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
      const placed = card;
      // Built before the pill is on the page, the menu stands at its slot once it is laid out.
      onPage(() => placed.style.setProperty("--menu-x", `${boxOf(anchor).x}px`));
      room.append(card);
      if (!still) list_.focus({ preventScroll: true });
    };
    // The workspace it is for takes a light tint of primary, unless it is the shown one, already
    // lit; a group's menu marks nothing.
    if ("workspace" in on && on.workspace !== active) {
      marked = anchor;
      anchor.dataset.menu = "";
    }
    const main = () => showRows(rows, null);
    const sub = (list: readonly Row[]) => showRows([{ spec: { label: "Back", icon: "arrow_back" }, act: main }, "separator", ...list], main);
    main();
    if (!still) document.addEventListener("pointerdown", outside, true);
  }

  /** Closes the windows of `key`: it stays, empty. */
  function closeWindows(band: Band, key: string): void {
    change(bands.map((each) => (each === band ? { ...each, spaces: each.spaces.map((one) => (one.key === key ? { ...one, occupied: false } : one)) } : each)));
  }

  /** Closes the windows of `key` and takes it away, the later ones shifting down; a group's only workspace is emptied and stays. */
  function deleteWorkspace(band: Band, key: string): void {
    if (band.spaces.length <= 1) return closeWindows(band, key);
    if (key === active) {
      // The shown workspace's place shows what shifts into it.
      const at = band.spaces.findIndex((space) => space.key === key);
      active = band.spaces[at + 1]?.key ?? band.spaces[at - 1]!.key;
    }
    change(without(bands, key));
  }

  /** Closes every window in `band`, leaving it one empty workspace. */
  function deleteGroup(band: Band): void {
    const showing = band.spaces.some((space) => space.key === active);
    if (band.borrowed) return dropGroup(band, showing);
    const first = band.spaces[0]!;
    if (showing) active = first.key;
    change(bands.map((each) => (each === band ? { ...each, spaces: renumber(each, [{ ...first, occupied: false }]) } : each)));
  }

  /**
   * Closes every window in `band`, a borrowed group, and takes the whole group away: its rule, its
   * name and its workspaces shrink out, and it is back, empty, once its
   * own screen is connected. Showing one of its workspaces, the screen shows its own group's first
   * instead, its mark lighting there.
   */
  function dropGroup(band: Band, showing: boolean): void {
    bands = bands.filter((each) => each !== band);
    if (showing) active = bands[0]!.spaces[0]!.key;
    render(true);
    for (const gone of content.querySelectorAll<HTMLElement>(`.shell-ws-slot[data-gone][data-block="${band.block}"]`)) gone.dataset.groupGone = "";
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
      const keys = entries.filter((entry) => entry.kind === "cell").map((entry) => entry.space);
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
      if (entry?.kind === "cell") press = { key: entry.space, at: local(event), pointer: event.pointerId };
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
      if (event.type === "pointercancel") propose(carry.home, true);
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
  /**
   * Runs `then` once the pill is on the page and laid out, at once if it is: a still is built
   * whole before a page puts it anywhere, so its motion's tracks find every part, and placed after.
   * A pill never put on a page gives up after a few seconds of frames.
   */
  function onPage(then: () => void, tries = 300): void {
    if (element.isConnected) return then();
    if (tries > 0) requestAnimationFrame(() => onPage(then, tries - 1));
  }

  const still = options.drag;
  if (still) {
    lift(`w${still.id}`, null, still.to ?? null);
    // HAZARD: read as its declared type: the checker can't see that `lift` set it, and takes it
    // for the null it was given.
    const held = carry as Carry | null;
    if (held) {
      const off = still.loose ? cssPx("space-s12") : 0;
      // Where its last glide came from, for its move's player: the cell it lifted from, or the hand.
      const from = still.loose ? -cssPx("space-s12") : cssPx("space-s12");
      held.disc.style.setProperty("--ws-ox", String(from));
      held.disc.style.setProperty("--ws-oy", String(from));
      const over = still.over;
      if (over !== undefined) refuse(held.disc, true);
      held.anchor = () => {
        const span = over === undefined ? undefined : spansOf(entries).get(over);
        return span ? { x: span.x + span.w / 2, y: span.y + span.h / 2 } : { x: holeCentre().x + off, y: holeCentre().y + off };
      };
      onPage(() => {
        if (!held.disc.matches(":popover-open")) held.disc.showPopover();
        discTo(held.disc, held.anchor!());
        // The disc is drawn in the viewport's pixels: as the page scrolls it keeps to the hole.
        const follow = () => {
          if (!element.isConnected || carry !== held) return removeEventListener("scroll", follow, true);
          discTo(held.disc, held.anchor!());
        };
        addEventListener("scroll", follow, { capture: true, passive: true });
        addEventListener("resize", follow, { passive: true });
      });
    }
  }
  if (!room) return made;
  room.className = "shell-ws-room";
  room.append(made);
  const open = options.menu;
  if (open) {
    const key = open.workspace !== undefined ? `w${open.workspace}` : null;
    const band = open.group !== undefined ? bands.find((each) => each.block === open.group) : null;
    const anchorKey = key ? `${key}@${bandOf(key)?.block}` : band?.name && band.borrowed ? `label${band.block}` : `${band?.spaces.at(-1)?.key}@${band?.block}`;
    const anchor = content.querySelector<HTMLElement>(`.shell-ws-slot[data-key="${anchorKey}"]`);
    if (anchor) openMenu(key ? { workspace: key } : { block: band!.block }, anchor, open.holding !== undefined ? { holding: open.holding } : {});
  }
  return room;
}
