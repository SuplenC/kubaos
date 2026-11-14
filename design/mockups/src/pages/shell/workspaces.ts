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
// It isn't the library's `Pill`: its cells glide every frame. It has no part in `src/lib/` yet.

import "./workspaces.css";
import type { Pinned } from "../../lib/control.ts";
import { pin } from "../../lib/control.ts";
import { focusRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { withHint } from "../../lib/hint.ts";
import { probe } from "../../lib/probe.ts";
import { advance, text, type TypeStyle } from "../../lib/text.ts";
import { cubic, length, raw } from "../../lib/tokens.ts";
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

export interface WorkspacesOptions {
  readonly groups?: readonly Group[];
  /** The shown workspace's id. */
  readonly active?: number;
  readonly kanji?: boolean;
  /** Clicks and the wheel move the shown workspace, a ghost turning into a workspace. */
  readonly live?: boolean;
  readonly pinned?: Pinned;
}

/** The shots' stand-ins: 1 with a window, 2 shown with one, 3 the ghost; Desk's 11 with a window, 12 its ghost. */
export const SHOTS_GROUPS: readonly Group[] = [
  { block: 0, spaces: [{ id: 1, occupied: true }, { id: 2, occupied: true }, { id: 3, occupied: false }] },
  { block: 1, borrowed: true, name: "Desk", spaces: [{ id: 11, occupied: true }, { id: 12, occupied: false }] },
];

const KANJI = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];

type Kind = "cell" | "ghost" | "gap" | "label";

interface Entry {
  readonly key: string;
  readonly kind: Kind;
  readonly workspace: number;
  readonly borrowed: boolean;
  readonly head: boolean;
  readonly tail: boolean;
  readonly label: string;
  readonly occupied: boolean;
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

/** What the pill draws, in order, as `Workspaces.qml`'s `entries`. */
function entriesOf(groups: readonly Group[], active: number): Entry[] {
  const result: Entry[] = [];
  groups.forEach((group, index) => {
    const name = group.borrowed ? (group.name ?? "") : "";
    const borrowed = group.borrowed ?? false;
    if (index > 0) result.push({ key: `gap${group.block}`, kind: "gap", workspace: 0, borrowed: false, head: false, tail: false, label: "", occupied: false });
    if (name !== "") result.push({ key: `label${group.block}`, kind: "label", workspace: group.spaces[0]!.id, borrowed: true, head: true, tail: false, label: name, occupied: false });
    group.spaces.forEach((space, at) => {
      const last = at === group.spaces.length - 1;
      const ghost = last && at > 0 && space.id !== active && !space.occupied;
      result.push({ key: String(space.id), kind: ghost ? "ghost" : "cell", workspace: space.id, borrowed, head: at === 0 && name === "", tail: last, label: "", occupied: space.occupied });
    });
  });
  return result;
}

/** A length token in logical pixels. */
function px(name: string): number {
  return length(name);
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
  return settings().reduced ? 0 : Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

function easing(name: string): string {
  return `cubic-bezier(${cubic(`easing-${name}`).join(", ")})`;
}

/** One entry's slot: its piece of the tint behind, and its content. */
function slot(entry: Entry, kanji: boolean, active: number, choose: (id: number) => void): HTMLElement {
  const box = document.createElement("span");
  box.className = "shell-ws-slot";
  box.dataset.key = entry.key;
  box.dataset.kind = entry.kind;
  box.style.setProperty("--slot-width", String(widthOf(entry, kanji)));
  if (entry.borrowed) {
    const tint = document.createElement("span");
    tint.className = "shell-ws-tint";
    tint.toggleAttribute("data-head", entry.head);
    tint.toggleAttribute("data-tail", entry.tail);
    box.append(tint);
  }
  if (entry.kind === "gap") {
    const rule = document.createElement("span");
    rule.className = "shell-ws-rule";
    box.append(rule);
    return box;
  }
  if (entry.kind === "label") {
    const name = text(entry.label, "label", "secondary");
    name.classList.add("shell-ws-name");
    name.style.setProperty("--inset", String(Math.round(numberRoom(entry.workspace, kanji))));
    box.append(name);
    box.addEventListener("click", () => choose(entry.workspace));
    return box;
  }
  const shown = entry.workspace === active;
  const cell = document.createElement("span");
  cell.className = "shell-ws-cell";
  cell.toggleAttribute("data-ghost", entry.kind === "ghost");
  const number = text(numberText(entry.workspace, kanji), numberStyle(shown || entry.occupied, kanji), shown ? "on_primary" : entry.occupied ? "on_surface" : "on_surface_variant");
  number.classList.add("shell-ws-number");
  const plus = icon("add", "var(--icon-s16)", "var(--color-outline)");
  plus.classList.add("shell-ws-ghost");
  cell.append(number, plus);
  cell.addEventListener("click", () => choose(entry.workspace));
  box.append(cell);
  return box;
}

/** Where the cell of `id` starts among `entries`, in cells, or -1. */
function cellsTo(entries: readonly Entry[], id: number, kanji: boolean): number {
  let x = 0;
  for (const entry of entries) {
    if ((entry.kind === "cell" || entry.kind === "ghost") && entry.workspace === id) return x / px("size-chip");
    x += widthOf(entry, kanji);
  }
  return -1;
}

export function workspaces(options: WorkspacesOptions = {}): HTMLElement {
  const kanji = options.kanji ?? false;
  let groups: Group[] = (options.groups ?? SHOTS_GROUPS).map((group) => ({ ...group, spaces: [...group.spaces] }));
  let active = options.active ?? 2;
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

  let entries = entriesOf(groups, active);
  const choose = (id: number) => {
    if (options.live) move(id);
  };

  function draw(): void {
    content.replaceChildren(...entries.map((entry) => slot(entry, kanji, active, choose)));
    const at = cellsTo(entries, active, kanji);
    indicator.style.setProperty("--ws-start", String(at));
    indicator.style.setProperty("--ws-end", String(at + 1));
  }
  draw();

  /** Shows workspace `id`: a ghost becomes a workspace and its group keeps a new ghost after it; an empty one left goes. */
  function move(id: number): void {
    if (id === active) return;
    const before = cellsTo(entries, active, kanji);
    const leaving = active;
    groups = groups.map((group) => {
      let spaces = group.spaces.filter((space) => space.occupied || space.id !== leaving || space === group.spaces.at(-1));
      const last = spaces.at(-1)!;
      if (last.id === id && spaces.length < 10) spaces = [...spaces, { id: last.id + 1, occupied: false }];
      return { ...group, spaces };
    });
    active = id;
    const old = new Map([...content.children].map((child) => [(child as HTMLElement).dataset.key!, child as HTMLElement]));
    entries = entriesOf(groups, active);
    const wanted = new Set(entries.map((entry) => entry.key));
    // Slots that stay keep their element, grown or recoloured; new ones grow from nothing; gone ones shrink out.
    const next: HTMLElement[] = [];
    for (const entry of entries) {
      const fresh = slot(entry, kanji, active, choose);
      const kept = old.get(entry.key);
      if (kept) {
        kept.replaceWith(fresh);
        old.delete(entry.key);
      } else {
        fresh.animate([{ "--present": 0 }, { "--present": 1 }], { duration: ms("long"), easing: easing("emphasized_decel") });
        fresh.animate([{ "--ws-shown": 0 }, { "--ws-shown": 1 }], { duration: ms("long"), easing: easing("standard") });
      }
      next.push(fresh);
    }
    for (const [key, gone] of old) {
      if (wanted.has(key)) continue;
      next.splice(Math.min(next.length, [...content.children].indexOf(gone)), 0, gone);
      gone.animate([{ "--ws-shown": 1 }, { "--ws-shown": 0 }], { duration: ms("long"), easing: easing("standard"), fill: "forwards" });
      gone.animate([{ "--present": 1 }, { "--present": 0 }], { duration: ms("long"), easing: easing("emphasized_decel"), fill: "forwards" }).finished.then(() => gone.remove(), () => {});
    }
    content.replaceChildren(...next);
    const target = cellsTo(entries, active, kanji);
    const right = target > before;
    const lead = ms("medium");
    const trail = ms("long");
    const from = getComputedStyle(indicator);
    const start = from.getPropertyValue("--ws-start");
    const end = from.getPropertyValue("--ws-end");
    indicator.style.setProperty("--ws-start", String(target));
    indicator.style.setProperty("--ws-end", String(target + 1));
    indicator.animate([{ "--ws-start": start }, { "--ws-start": String(target) }], { duration: right ? trail : lead, easing: easing("emphasized_decel") });
    indicator.animate([{ "--ws-end": end }, { "--ws-end": String(target + 1) }], { duration: right ? lead : trail, easing: easing("emphasized_decel") });
  }

  if (options.live) {
    element.addEventListener("wheel", (event) => {
      event.preventDefault();
      const ids = entries.filter((entry) => entry.kind === "cell" || entry.kind === "ghost").map((entry) => entry.workspace);
      const at = ids.indexOf(active);
      const step = event.deltaY > 0 ? 1 : -1;
      move(ids[(at + step + ids.length) % ids.length]!);
    }, { passive: false });
  }
  return probe(element, { role: "component", name: "Workspaces", variant: kanji ? "kanji" : "", fixed: { width: false, height: true } });
}
