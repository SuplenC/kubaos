// A few actions or choices one under the other: `kubaos-ui`'s `menu::menu`, `menu::menu_item` and
// `menu::menu_separator`, `KubaOS.Components`' `Menu` and `MenuItem` (`docs/design/components/
// menu.md`), on a `Popover` (`lib/tooltip.ts`) raised to the ladder's step 2.
//
// It is built as every raised list is, a select's open list too: the popover `space.s4` inside, an
// item `size.control_sm` tall at the row radius, concentric inside the popover's (`radius.control`
// − `space.s4`), and `space.s8` inside, so its words start `space.s12` from the popover's edge. An
// item holds an optional symbol, 16, its words in `body` on one line, the keys that do the same as
// keycaps at its end, and a check whose room is kept when unchecked. A hairline, inset to the words
// and `space.s4` clear of each side, separates groups. The item the keyboard is on is lit, on
// `secondary_container`, its keycaps in `primary`; the pointer moving onto an item puts the
// keyboard on it, so one item is lit at a time. One that destroys something is in `error`; one that
// can't be taken is dimmed and is passed over.
//
// A held item destroys something that can't be taken back, so it is taken only by holding it for
// `duration.hold`: the pointer pressed on it, or Enter on it. While it is held, `error` sweeps across
// it from its start, its words turning `on_error` under the sweep, and it is taken when the sweep
// reaches its end; let go or left early, the sweep runs back over `short`. Lit, it takes `error` at
// `opacity.tint`, so the sweep has room to show. The sweep is a timer, not a move: reduced motion
// keeps it.

import { icon } from "./icon.ts";
import { keycap, keys } from "./keycap.ts";
import { divider, filling, row, stack } from "./layout.ts";
import { FIXED_HEIGHT, probe, variant } from "./probe.ts";
import { clamped } from "./text.ts";
import { raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

export interface MenuItemSpec {
  readonly label: string;
  /** A short line under the words, in `small` and the muted colour: the item is two lines tall. */
  readonly description?: string;
  readonly icon?: string;
  /** A symbol at the item's end, in the muted colour: what the item lacks, as a device's crossed-out mic. */
  readonly trailing?: string;
  /** An app's own icon in the symbol's place, at the symbol's size: a player's app. */
  readonly picture?: string;
  /** The keys that do the same, as `Ctrl+C`: one keycap each. */
  readonly keys?: string;
  /** Whether it shows a check, and whether it is ticked; none for an item that never does. */
  readonly checked?: boolean;
  readonly danger?: boolean;
  readonly disabled?: boolean;
  /** It opens a submenu: a chevron at its end says so. */
  readonly submenu?: boolean;
  /** Taken only by holding it for `duration.hold`: a danger step that can't be taken back. */
  readonly held?: boolean;
  /** How far a held item's sweep is drawn, 0 to 1: a sample of it part-way held. */
  readonly holding?: number;
}

/** An entry of a menu: an item, or the hairline between two groups. */
export type MenuEntry = MenuItemSpec | "separator";

/** A state an item is pinned in, as the specimens pin them: the keyboard on it, or pressed. */
export type ItemPinned = "lit" | "pressed";

/** One item, the keyboard on it when `current`. */
export function menuItem(spec: MenuItemSpec, current = false, pinned?: ItemPinned): HTMLElement {
  const item = document.createElement("span");
  item.className = "k-menu-item";
  item.setAttribute("role", spec.checked === undefined ? "menuitem" : "menuitemradio");
  if (spec.checked !== undefined) item.setAttribute("aria-checked", String(spec.checked));
  item.toggleAttribute("data-danger", (spec.danger ?? false) || (spec.held ?? false));
  item.toggleAttribute("data-held", spec.held ?? false);
  item.toggleAttribute("data-current", current && !spec.disabled);
  if (pinned && !spec.disabled) item.dataset.pinned = pinned;
  if (spec.disabled) item.setAttribute("aria-disabled", "true");
  if (spec.submenu) item.setAttribute("aria-haspopup", "menu");
  if (spec.description) item.dataset.lines = "two";
  item.append(filling(content(spec, "var(--ink)", "var(--muted)")));
  if (spec.held) {
    // The sweep holds the same content in `on_error`, cut to how far it has gone, so the words
    // read on either side of its edge.
    const sweep = document.createElement("span");
    sweep.className = "k-menu-hold";
    sweep.setAttribute("aria-hidden", "true");
    sweep.append(filling(content(spec, "var(--color-on_error)", "var(--color-on_error)")));
    // The item's own content is what the geometry check reads; the sweep's copy is paint.
    for (const each of sweep.querySelectorAll<HTMLElement | SVGElement>("[data-probe]")) delete each.dataset.probe;
    item.prepend(sweep);
    if (spec.holding !== undefined) item.style.setProperty("--hold", String(spec.holding));
  }
  const words_ = variant((spec.icon || spec.picture) && "icon", spec.checked !== undefined && "check", (spec.danger || spec.held) && "danger", spec.submenu && "submenu", spec.description && "two-line");
  return probe(item, { role: "component", name: "MenuItem", variant: words_, fixed: FIXED_HEIGHT });
}

/** An item's symbol, words, keys, check and chevron, in `ink`; its description and trailing symbol in `muted`. */
function content(spec: MenuItemSpec, ink: string, muted: string): HTMLElement {
  const words = clamped(spec.label, "body");
  words.style.color = ink;
  const parts: Element[] = [];
  if (spec.picture) parts.push(picture(spec.picture));
  else if (spec.icon) parts.push(icon(spec.icon, "var(--icon-s16)", ink));
  const room = document.createElement("span");
  room.className = "k-menu-words";
  if (spec.description) {
    const line = clamped(spec.description, "small");
    line.style.color = muted;
    room.append(stack(0, "start", filling(words), filling(line)));
  } else room.append(words);
  parts.push(filling(room));
  if (spec.trailing) parts.push(icon(spec.trailing, "var(--icon-s16)", muted));
  if (spec.keys) {
    const caps = row("s4", "centre", ...keys(spec.keys).map(keycap));
    caps.classList.add("k-menu-keys");
    parts.push(caps);
  }
  if (spec.checked !== undefined) {
    const check = document.createElement("span");
    check.className = "k-menu-check";
    if (spec.checked) check.append(icon("check", "var(--icon-s16)", ink));
    parts.push(check);
  }
  if (spec.submenu) parts.push(icon("chevron_right", "var(--icon-s16)", ink));
  return row("s8", "centre", ...parts);
}

/** An app's icon in a symbol's box, kept whole. */
function picture(source: string): HTMLElement {
  const image = document.createElement("img");
  image.className = "k-menu-picture";
  image.src = source;
  image.alt = "";
  return image;
}

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** What holds a held item: pressing starts its sweep, letting go runs it back. */
export interface Holding {
  readonly press: () => void;
  readonly release: () => void;
}

/**
 * Wires `item`, a held item, to be held: `press` sweeps `--hold` on to 1 over what is left of
 * `duration.hold`, on `linear`, and calls `taken` once it gets there; `release` before then runs it
 * back to 0 over `short` on `standard`. Neither is a move reduce motion caps.
 */
export function holdable(item: HTMLElement, taken: () => void): Holding {
  let going: Animation | null = null;
  const now = () => Number.parseFloat(getComputedStyle(item).getPropertyValue("--hold")) || 0;
  const settle = (to: number) => {
    going?.cancel();
    going = null;
    item.style.setProperty("--hold", String(to));
  };
  const press = () => {
    const from = now();
    settle(from);
    item.dataset.holding = "";
    going = item.animate([{ "--hold": from }, { "--hold": 1 }], { duration: ms("hold") * (1 - from), easing: "linear", fill: "forwards" });
    going.finished.then(() => {
      settle(1);
      delete item.dataset.holding;
      taken();
    }, () => {});
  };
  const release = () => {
    if (!item.hasAttribute("data-holding")) return;
    delete item.dataset.holding;
    const from = now();
    settle(from);
    going = item.animate([{ "--hold": from }, { "--hold": 0 }], { duration: ms("short"), easing: raw("easing-standard"), fill: "forwards" });
    going.finished.then(() => settle(0), () => {});
  };
  item.addEventListener("pointerdown", (event) => {
    if (event.button === 0) press();
  });
  item.addEventListener("pointerup", release);
  item.addEventListener("pointerleave", release);
  return { press, release };
}

/** The hairline between two groups, `space.s4` clear of each and inset to the items' words. */
export function menuSeparator(): HTMLElement {
  const rule = stack(0, "start", divider());
  rule.classList.add("k-menu-separator");
  return rule;
}

export interface MenuOptions {
  /** The item the keyboard is on, by its index among the entries. */
  readonly current?: number;
  /** The current item pinned in a state, whatever the pointer does. */
  readonly pinned?: ItemPinned;
  /** The most items it shows before it scrolls. */
  readonly most?: number;
  /** What taking an item does, by its index among the entries. */
  readonly onTake?: (index: number) => void;
  /** What Esc does: the popover closes. */
  readonly onLeave?: () => void;
}

/** Whether `entry` can be taken: an item that isn't disabled. */
function takes(entry: MenuEntry | undefined): entry is MenuItemSpec {
  return entry !== undefined && entry !== "separator" && !entry.disabled;
}

/**
 * `entries` one under the other. ↑ ↓ (j k, Ctrl+J/K/N/P) move over the items that can be taken,
 * round from either end; Home and End go to the first and the last; Enter takes one, once per
 * press, or holds a held one while it is down; Esc leaves. Past `most` items it scrolls, the item the keyboard is on kept in view.
 */
export function menu(entries: readonly MenuEntry[], options: MenuOptions = {}): HTMLElement {
  const list = document.createElement("span");
  list.className = "k-menu";
  list.setAttribute("role", "menu");
  list.tabIndex = 0;
  if (options.most !== undefined) list.style.setProperty("--most", String(options.most));
  let current = options.current ?? -1;
  const holds = new Map<number, Holding>();
  const items = entries.map((entry, index) => {
    if (entry === "separator") return menuSeparator();
    const item = menuItem(entry, index === current, index === current ? options.pinned : undefined);
    if (takes(entry)) {
      // Only a pointer that moves takes the keyboard: a menu opening under a resting pointer keeps
      // the keyboard on its picked item.
      item.addEventListener("pointermove", () => {
        if (current !== index) move(index);
      });
      // A held item is taken once it has been held for `duration.hold`, never by a click.
      if (entry.held) holds.set(index, holdable(item, () => options.onTake?.(index)));
      else item.addEventListener("click", () => options.onTake?.(index));
    }
    return item;
  });
  list.append(...items);

  /** Puts the keyboard on the item at `index`, scrolled into view. */
  function move(index: number): void {
    holds.get(current)?.release();
    items[current]?.removeAttribute("data-current");
    current = index;
    const item = items[index];
    if (!item) return;
    item.setAttribute("data-current", "");
    if (item.offsetTop < list.scrollTop) list.scrollTop = item.offsetTop;
    else if (item.offsetTop + item.offsetHeight > list.scrollTop + list.clientHeight) list.scrollTop = item.offsetTop + item.offsetHeight - list.clientHeight;
  }

  /** To the next item that can be taken `by` away, round from either end. */
  function step(by: 1 | -1): void {
    const count = entries.length;
    let index = current;
    for (let i = 0; i < count; i++) {
      index = (index + by + count) % count;
      if (takes(entries[index])) return move(index);
    }
  }

  /** To the first item that can be taken from `from` on, `by`'s way. */
  function jump(from: number, by: 1 | -1): void {
    for (let index = from; index >= 0 && index < entries.length; index += by) {
      if (takes(entries[index])) return move(index);
    }
  }

  list.addEventListener("keydown", (event) => {
    const ctrl = event.ctrlKey;
    const key = event.key.toLowerCase();
    if (event.key === "ArrowDown" || (!ctrl && key === "j") || (ctrl && (key === "j" || key === "n"))) step(1);
    else if (event.key === "ArrowUp" || (!ctrl && key === "k") || (ctrl && (key === "k" || key === "p"))) step(-1);
    else if (event.key === "Home") jump(0, 1);
    else if (event.key === "End") jump(entries.length - 1, -1);
    else if (event.key === "Enter") {
      const held = holds.get(current);
      if (event.repeat || !takes(entries[current])) {
        // A held item's Enter, kept down, is still holding it.
      } else if (held) held.press();
      else options.onTake?.(current);
    } else if (event.key === "Escape") options.onLeave?.();
    else return;
    event.preventDefault();
  });
  // Enter let go, or the keyboard moved off, before a held item was taken: its sweep runs back.
  list.addEventListener("keyup", (event) => {
    if (event.key === "Enter") holds.get(current)?.release();
  });
  list.addEventListener("focus", () => {
    if (current < 0) jump(0, 1);
  });
  // Shown, the item the keyboard is on is in view.
  requestAnimationFrame(() => move(current));
  return probe(list, { role: "component", name: "Menu", variant: "", fixed: { width: false, height: false } });
}
