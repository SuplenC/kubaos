// A few actions or choices one under the other: `kubaos-ui`'s `menu::menu`, `menu::menu_item` and
// `menu::menu_separator`, `KubaOS.Components`' `Menu` and `MenuItem` (`docs/design/components/
// menu.md`), on a `Popover` (`lib/tooltip.ts`) raised to the ladder's step 2.
//
// An item is `size.control_sm` tall, `space.s12` inside, its corners concentric inside the
// popover's: an optional `body` symbol, its words in `body` on one line, the keys that do the same
// as keycaps, and a check whose room is kept when unchecked. A hairline `space.s4` clear of each
// side separates groups. The item the keyboard is on is lit, on `secondary_container`; the pointer
// moving onto an item puts the keyboard on it, so one item is lit at a time. One that destroys
// something is in `error`; one that can't be taken is dimmed and is passed over.

import { icon } from "./icon.ts";
import { keycap, keys } from "./keycap.ts";
import { divider, filling, row, stack } from "./layout.ts";
import { FIXED_HEIGHT, probe, variant } from "./probe.ts";
import { nested, role } from "./radius.ts";
import { clamped } from "./text.ts";

export interface MenuItemSpec {
  readonly label: string;
  readonly icon?: string;
  /** The keys that do the same, as `Ctrl+C`: one keycap each. */
  readonly keys?: string;
  /** Whether it shows a check, and whether it is ticked; none for an item that never does. */
  readonly checked?: boolean;
  readonly danger?: boolean;
  readonly disabled?: boolean;
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
  item.toggleAttribute("data-danger", spec.danger ?? false);
  item.toggleAttribute("data-current", current && !spec.disabled);
  if (pinned && !spec.disabled) item.dataset.pinned = pinned;
  if (spec.disabled) item.setAttribute("aria-disabled", "true");
  item.style.setProperty("--corner", nested(role("surface"), "s12").css);
  const ink = "var(--ink)";
  const words = clamped(spec.label, "body");
  words.style.color = ink;
  const parts: Element[] = [];
  if (spec.icon) parts.push(icon(spec.icon, "var(--type-body-icon)", ink));
  const room = document.createElement("span");
  room.className = "k-menu-words";
  room.append(words);
  parts.push(filling(room));
  if (spec.keys) parts.push(row("s4", "centre", ...keys(spec.keys).map(keycap)));
  if (spec.checked !== undefined) {
    const check = document.createElement("span");
    check.className = "k-menu-check";
    if (spec.checked) check.append(icon("check", "var(--type-body-icon)", ink));
    parts.push(check);
  }
  item.append(filling(row("s12", "centre", ...parts)));
  const words_ = variant(spec.icon && "icon", spec.checked !== undefined && "check", spec.danger && "danger");
  return probe(item, { role: "component", name: "MenuItem", variant: words_, fixed: FIXED_HEIGHT });
}

/** The hairline between two groups, `space.s4` clear of each. */
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
  /** The most items it shows before it scrolls: a select's eight. */
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
 * press; Esc leaves. Past `most` items it scrolls, the item the keyboard is on kept in view.
 */
export function menu(entries: readonly MenuEntry[], options: MenuOptions = {}): HTMLElement {
  const list = document.createElement("span");
  list.className = "k-menu";
  list.setAttribute("role", "menu");
  list.tabIndex = 0;
  if (options.most !== undefined) list.style.setProperty("--most", String(options.most));
  let current = options.current ?? -1;
  const items = entries.map((entry, index) => {
    if (entry === "separator") return menuSeparator();
    const item = menuItem(entry, index === current, index === current ? options.pinned : undefined);
    if (takes(entry)) {
      // Only a pointer that moves takes the keyboard: a menu opening under a resting pointer keeps
      // the keyboard on its picked item.
      item.addEventListener("pointermove", () => {
        if (current !== index) move(index);
      });
      item.addEventListener("click", () => options.onTake?.(index));
    }
    return item;
  });
  list.append(...items);

  /** Puts the keyboard on the item at `index`, scrolled into view. */
  function move(index: number): void {
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
      if (!event.repeat && takes(entries[current])) options.onTake?.(current);
    } else if (event.key === "Escape") options.onLeave?.();
    else return;
    event.preventDefault();
  });
  list.addEventListener("focus", () => {
    if (current < 0) jump(0, 1);
  });
  // Shown, the item the keyboard is on is in view.
  requestAnimationFrame(() => move(current));
  return probe(list, { role: "component", name: "Menu", variant: "", fixed: { width: false, height: false } });
}
