// One of a list too long for radio rows or tabs: `kubaos-ui`'s `select::select`,
// `KubaOS.Components`' `Select` (`docs/design/components/select.md`). Shut, a field's look:
// `size.control_md` tall at the control radius with a hairline inside its edge, the picked option
// in `body_large`, an optional `body_large` symbol before it and a chevron at its end. It is a
// control in the state model (`.k-control` in `lib.css`), lit as a field is.
//
// Open, the select's own box grows into its list: one surface raised to the ladder's step 2, the
// value row where it was and the options under it, past a faint rule, or over it where the room
// below is too short. Its options are menu items (`lib/menu.ts`), at most eight in view before the
// list scrolls. The keyboard stays on the select throughout: it moves the highlight in the list,
// and Tab goes on from the select as from any control.
//
// Each option may carry a symbol of its own (a device's, by how it connects): the list's rows show
// theirs, and the value row leads with the pick's. An option may also carry a short description,
// a second line under its words in `small` and the muted colour, which makes every row of the list
// a two-line row's height, and a symbol at its row's end in the muted colour (a device's crossed-out
// mic). Both show only in the list: the value row shows the pick's words alone.
//
// The inline look is a choice inside a line of text, an app's device under its name: no border and
// no box, the pick in `small` and `on_surface_variant` with a `small` chevron `space.s4` after it.
// Under the pointer or the keyboard only the words and the chevron turn `primary_lit`, a lit Text
// button's words, with no tube and no fill. Open, the chevron turns up and its list is a Menu built
// in, on a popover raised to step 2 `space.s4` under the words (over them where the room below is
// too short), its options' words in line with the pick's; it fades in over `short` and out over
// `micro`, as a popover does.

import { pin, type Pinned } from "./control.ts";
import { raise } from "./elevation.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { filling, row } from "./layout.ts";
import { menuItem } from "./menu.ts";
import { probe, variant } from "./probe.ts";
import { clamped } from "./text.ts";
import { popover } from "./tooltip.ts";

/** The most options in view before the list scrolls. */
const MOST = 8;

/** The rows kept above the pick as the list opens scrolled, so it reads in its neighbours. */
const ABOVE = 3;

export interface SelectOptions {
  readonly options: readonly string[];
  /** The picked option, or none: the placeholder shows. */
  readonly picked?: number;
  readonly placeholder?: string;
  readonly icon?: string;
  /** Each option's own symbol, in its row; the value row leads with the pick's, over `icon`. */
  readonly icons?: readonly string[];
  /** Each option's short line under its words, in the list only: every row is two lines tall. */
  readonly descriptions?: readonly string[];
  /** Each option's symbol at its row's end in the muted colour, in the list only; none where undefined. */
  readonly trailing?: readonly (string | undefined)[];
  /** A field's box, or the inline look: the words and chevron alone, in a line of text. */
  readonly look?: SelectLook;
  /** Drawn open in the sample's flow, the room taking the open surface's height. */
  readonly open?: boolean;
  /** The highlighted option of a select drawn open; the pick by default. */
  readonly highlight?: number;
  /** A select drawn open grows upward, as one too near the bottom of its room does. */
  readonly up?: boolean;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

/** How a select looks shut: a field's box, or inline in a line of text. */
export type SelectLook = "field" | "inline";

let made = 0;

/**
 * A select in a room as wide as it is given and a shut select tall: shut, or drawn `open` in the
 * sample's flow. A live one grows over what is below it (or above it), taking no room.
 */
export function select(options: SelectOptions): HTMLElement {
  const id = `k-select-${made++}`;
  let picked = options.picked ?? -1;
  const disabled = options.disabled ?? false;

  const room = document.createElement("span");
  room.className = "k-select-room";
  room.style.setProperty("--rows", String(Math.max(1, Math.min(MOST, options.options.length))));
  const described = Boolean(options.descriptions);
  if (described) room.style.setProperty("--select-option", "var(--size-row_two_line)");

  const inline = options.look === "inline";
  const iconOf = (index: number): string | undefined => options.icons?.[index] ?? options.icon;
  const hasIcon = Boolean(options.icons ?? options.icon);

  const surface = inline ? document.createElement("span") : raise(document.createElement("span"), 2);
  surface.classList.add(...(inline ? ["k-select-inline"] : ["k-select", "k-sunken"]), "k-control");
  surface.setAttribute("role", "combobox");
  surface.setAttribute("aria-haspopup", "listbox");
  surface.setAttribute("aria-controls", `${id}-list`);
  surface.setAttribute("aria-expanded", "false");
  surface.tabIndex = disabled ? -1 : 0;
  if (disabled) surface.setAttribute("aria-disabled", "true");
  pin(surface, options.pinned);

  const value = document.createElement("span");
  value.className = "k-select-value";
  const markSize = inline ? "var(--type-small-icon)" : "var(--type-body_large-icon)";
  const chevron = icon("expand_more", markSize, "var(--mark)");
  chevron.classList.add("k-select-chevron");
  // The value row's symbol, swapped as the pick changes.
  const lead = document.createElement("span");
  lead.className = "k-select-lead";

  const list = document.createElement("span");
  list.className = inline ? "k-menu k-select-menu" : "k-select-list";
  list.id = `${id}-list`;
  list.setAttribute("role", "listbox");
  // A scrolling box takes Tab of its own in Chromium; the keyboard stays on the select instead.
  list.tabIndex = -1;
  const items = options.options.map((label, index) => {
    const symbol = options.icons?.[index];
    const description = options.descriptions?.[index];
    const trailing = options.trailing?.[index];
    const item = menuItem({ label, ...(symbol ? { icon: symbol } : {}), ...(description ? { description } : {}), ...(trailing ? { trailing } : {}) });
    item.id = `${id}-${index}`;
    item.setAttribute("role", "option");
    return item;
  });
  list.append(...items);

  if (inline) {
    room.dataset.look = "inline";
    room.toggleAttribute("data-icons", Boolean(options.icons));
    surface.append(row("s4", "centre", filling(value), chevron));
    const card = popover(2, list);
    card.classList.add("k-select-popover");
    room.append(surface, card);
  } else {
    const parts: Element[] = [];
    if (hasIcon) parts.push(lead);
    parts.push(filling(value), chevron);
    const head = document.createElement("span");
    head.className = "k-select-head";
    head.append(row("s8", "centre", ...parts));
    focusRing(head, "control", "accent", false).dataset.pressable = "";
    const rule = document.createElement("span");
    rule.className = "k-select-rule";
    const pane = document.createElement("span");
    pane.className = "k-select-pane";
    pane.append(list, rule);
    surface.append(pane, head);
    room.append(surface);
  }

  /** Shows the pick: its words in the value row and its option marked picked. */
  function showPick(): void {
    const words = options.options[picked] ?? options.placeholder ?? "";
    const text = clamped(words, inline ? "small" : "body_large");
    text.style.color = "var(--value)";
    value.replaceChildren(text);
    const symbol = iconOf(picked);
    lead.replaceChildren(...(symbol ? [icon(symbol, "var(--type-body_large-icon)", "var(--mark)")] : []));
    lead.hidden = !symbol;
    surface.toggleAttribute("data-placeholder", picked < 0);
    items.forEach((item, index) => {
      item.toggleAttribute("data-picked", index === picked);
      item.setAttribute("aria-selected", String(index === picked));
    });
  }
  showPick();

  let highlight = -1;

  /** Highlights the option at `index`, scrolled into view with the list's padding round it. */
  function light(index: number): void {
    items[highlight]?.removeAttribute("data-current");
    highlight = index;
    const item = items[index];
    if (!item) return;
    item.setAttribute("data-current", "");
    surface.setAttribute("aria-activedescendant", item.id);
    const pad = items[0]!.offsetTop;
    const top = item.offsetTop - pad;
    const bottom = item.offsetTop + item.offsetHeight + pad - list.clientHeight;
    if (list.scrollTop > top) list.scrollTop = top;
    else if (list.scrollTop < bottom) list.scrollTop = bottom;
  }

  /** Scrolls the list so the option at `index` has `ABOVE` rows over it, as far as the list goes. */
  function scrollTo(index: number): void {
    const item = items[index];
    if (!item) return;
    list.scrollTop = item.offsetTop - items[0]!.offsetTop - ABOVE * item.offsetHeight;
  }

  room.toggleAttribute("data-up", options.up ?? false);
  probe(surface, {
    role: "component",
    name: "Select",
    variant: variant(inline && "inline", hasIcon && "icon", described && "two-line", options.open && "open"),
    fixed: { width: false, height: inline || !options.open },
  });

  if (options.open) {
    room.dataset.open = "";
    room.dataset.shown = "";
    surface.setAttribute("aria-expanded", "true");
    const lit = options.highlight ?? Math.max(0, picked);
    // Marked at once, so a player finds it; scrolled once the list is laid out.
    items[lit]?.setAttribute("data-current", "");
    highlight = lit;
    requestAnimationFrame(() => {
      scrollTo(Math.max(0, picked));
      light(lit);
    });
    return room;
  }
  if (disabled) return room;

  let open = false;

  /** Whether the list fits under the select in the room that clips it, or the window. */
  function growsUp(): boolean {
    const here = room.getBoundingClientRect();
    // A field's list is laid out while shut, so its height is the open surface's growth; an inline
    // select's Menu is measured once open. Both in the same viewport pixels as the rooms'.
    const drop = (inline ? list.parentElement! : list).getBoundingClientRect().height;
    let top = 0;
    let bottom = window.innerHeight;
    for (let up = room.parentElement; up; up = up.parentElement) {
      if (getComputedStyle(up).overflowY === "visible") continue;
      const clip = up.getBoundingClientRect();
      top = Math.max(top, clip.top);
      bottom = Math.min(bottom, clip.bottom);
      break;
    }
    const below = bottom - here.bottom;
    return below < drop && here.top - top > below;
  }

  function show(): void {
    if (open) return;
    open = true;
    delete room.dataset.folding;
    delete room.dataset.taken;
    for (const item of items) item.removeAttribute("data-taken");
    // An inline select's Menu is laid out only while open, so it opens before it is measured.
    if (inline) room.dataset.open = "";
    room.toggleAttribute("data-up", growsUp());
    scrollTo(Math.max(0, picked));
    light(Math.max(0, picked));
    room.dataset.open = "";
    surface.setAttribute("aria-expanded", "true");
    document.addEventListener("pointerdown", outside, true);
  }

  /** Folds the list away; the keyboard is still on the select, or Tab has taken it on. */
  function close(): void {
    if (!open) return;
    open = false;
    room.dataset.folding = "";
    delete room.dataset.open;
    surface.setAttribute("aria-expanded", "false");
    // The highlight stays as the list folds, so a pick is seen going.
    surface.removeAttribute("aria-activedescendant");
    document.removeEventListener("pointerdown", outside, true);
    // With nothing to play (reduced motion) the fold is done at once.
    if (getComputedStyle(room).getPropertyValue("--select-open").trim() === "0") folded();
  }

  function folded(): void {
    delete room.dataset.folding;
    delete room.dataset.taken;
  }

  /** Picks the option at `index`: it shows pressed for `micro`, then the surface folds. */
  function take(index: number): void {
    if (!open || !items[index]) return;
    items[index]!.setAttribute("data-taken", "");
    room.dataset.taken = "";
    picked = index;
    showPick();
    close();
  }

  /** The next option after the highlight whose words start with `letter`, round from the end. */
  function typeAhead(letter: string): void {
    const count = items.length;
    const wanted = letter.toLocaleLowerCase();
    for (let i = 1; i <= count; i++) {
      const index = (highlight + i) % count;
      if (options.options[index]!.toLocaleLowerCase().startsWith(wanted)) return light(index);
    }
  }

  function outside(event: PointerEvent): void {
    if (!room.contains(event.target as Node)) close();
  }

  room.addEventListener("transitionend", (event) => {
    if (event.target === room && event.propertyName === "--select-open" && !open) folded();
  });
  (inline ? surface : surface.querySelector<HTMLElement>(".k-select-head")!).addEventListener("click", () => (open ? close() : show()));
  items.forEach((item, index) => {
    // Only a pointer that moves takes the highlight: a list opening under a resting pointer keeps
    // it on the pick.
    item.addEventListener("pointermove", () => {
      if (open && highlight !== index) light(index);
    });
    item.addEventListener("click", () => take(index));
  });

  surface.addEventListener("keydown", (event) => {
    const key = event.key;
    const ctrl = event.ctrlKey;
    if (!open) {
      if (key === "Enter" || key === " " || key === "ArrowDown" || key === "ArrowUp") {
        if (!event.repeat) show();
        event.preventDefault();
      }
      return;
    }
    const last = items.length - 1;
    if (key === "ArrowDown" || (ctrl && (key === "n" || key === "j"))) light(Math.min(last, highlight + 1));
    else if (key === "ArrowUp" || (ctrl && (key === "p" || key === "k"))) light(Math.max(0, highlight - 1));
    else if (key === "Home") light(0);
    else if (key === "End") light(last);
    else if (key === "Enter" || key === " ") {
      if (!event.repeat) take(highlight);
    } else if (key === "Escape") {
      close();
      event.stopPropagation();
    } else if (key === "Tab") return close();
    else if (key.length === 1 && !ctrl && !event.altKey && !event.metaKey) typeAhead(key);
    else return;
    event.preventDefault();
  });

  return room;
}
