// The notification surfaces, as `config/quickshell/modules/notifications` lays them out: the
// notifications panel's list, the popup stack and the bell's peek, each holding the library's
// notification groups and cards (`lib/notification-group.ts`), and the screens the pages draw
// them on. This file is no page.

import "./notification-parts.css";
import "../notifications.css";
import { button } from "../../lib/button.ts";
import { type MenuEntry, menu } from "../../lib/menu.ts";
import type { Note } from "../../lib/notification-card.ts";
import { type GroupElement, group } from "../../lib/notification-group.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { hang, sheet } from "../../lib/sheet.ts";
import { advance, centreFor, text } from "../../lib/text.ts";
import { popover } from "../../lib/tooltip.ts";
import { BUILD, clearButton, dndButton, KUBAOS, keysButton, SIGNAL, SLACK, status } from "../notifications.ts";
import { CORNER_WIDTH, desk, onScreen, system } from "../templates/panels.ts";

/** `shown` in a room as wide as the popups hold a card. */
export function room(shown: HTMLElement, tall = false): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-notif-room";
  box.toggleAttribute("data-tall", tall);
  box.append(shown);
  return box;
}

/** A notification's right-click menu: its actions, then Open, Dismiss and Clear all. */
export function noteMenu(note: Note, current?: number): HTMLElement {
  const actions: MenuEntry[] = [...(note.main ? [{ label: note.main }] : []), ...(note.others ?? []).map((label) => ({ label }))];
  const entries: MenuEntry[] = [...actions, ...(actions.length > 0 ? ["separator" as const] : []), { label: "Open" }, { label: "Dismiss" }, { label: "Clear all", danger: true }];
  return popover(2, menu(entries, current === undefined ? {} : { current }));
}

/** A section of the panel's list: its label, then its groups. */
export interface Section {
  readonly label: "New" | "Earlier";
  readonly groups: readonly (readonly Note[])[];
}

export const SECTIONS: readonly Section[] = [
  { label: "New", groups: [SLACK.slice(0, 4), [BUILD]] },
  { label: "Earlier", groups: [SIGNAL, KUBAOS] },
];

export interface PanelOptions {
  readonly sections?: readonly Section[];
  readonly dnd?: boolean;
  /** The panel's groups opened, by their place down the list. */
  readonly opened?: readonly number[];
  readonly fanned?: number;
  /** The row the keyboard is on, by its place down the list, and the card inside an opened group. */
  readonly selected?: number;
  readonly inner?: number;
  readonly unfolded?: number;
  /** The keys' card shown, as under the pointer. */
  readonly keys?: boolean;
  /** Do not disturb's card shown, as under the pointer. */
  readonly dndTip?: boolean;
  /** Answers the pointer and the keys. */
  readonly live?: boolean;
}

/** Every group down the list, with its notes and its row. */
interface Placed {
  readonly notes: readonly Note[];
  readonly element: GroupElement;
}

/** The notifications panel's content: the header, the sections and their groups, the keys. */
export function panel(options: PanelOptions = {}): HTMLElement {
  let sections = (options.sections ?? SECTIONS).map((each) => ({ label: each.label, groups: [...each.groups] }));
  let dnd = options.dnd ?? false;
  const list = document.createElement("span");
  list.className = "k-notif-list";
  let placed: Placed[] = [];
  let at = options.selected ?? -1;
  let inner = options.inner ?? 0;

  let dndParts = dndButton(dnd, options.dndTip ?? false);
  const clear = clearButton();
  const page = panelPage({ title: "Notifications", subtitle: status(0, 1, dnd ? "manual" : undefined), actions: [dndParts.anchor, clear], body: list });
  page.element.classList.add("k-notif-panel");

  const count = () => sections.reduce((sum, each) => sum + each.groups.reduce((n, g) => n + g.length, 0), 0);
  const unread = () => sections.filter((each) => each.label === "New").reduce((sum, each) => sum + each.groups.reduce((n, g) => n + g.length, 0), 0);

  function say(): void {
    const words = status(unread(), count(), dnd ? "manual" : undefined);
    const line = page.element.querySelectorAll<HTMLElement>(".k-panel-header .k-text")[1];
    if (line) {
      line.textContent = words;
      line.style.setProperty("--natural", String(advance("small", words)));
      centreFor(line, "small", words);
    }
    clear.hidden = count() === 0;
  }

  function draw(): void {
    placed = [];
    const rows: HTMLElement[] = [];
    let index = 0;
    for (const section of sections) {
      if (section.groups.length === 0) continue;
      const label = text(section.label, "label", "on_surface_variant");
      const holder = document.createElement("span");
      holder.className = "k-notif-label k-notif-behind";
      holder.append(label);
      rows.push(holder);
      for (const notes of section.groups) {
        const mine = index;
        const element = group(notes, {
          open: (options.opened ?? []).includes(mine),
          fan: options.fanned === mine ? 1 : 0,
          live: options.live ?? false,
          selected: at === mine,
          current: inner,
          onRaise: (raised) => recede(raised ? element : null),
          onDismiss: (which) => dismiss(notes, which),
          onOpen: () => {
            at = mine;
            inner = 0;
            select();
          },
        });
        element.classList.add("k-notif-behind");
        rows.push(element);
        placed.push({ notes, element });
        index++;
      }
    }
    if (rows.length === 0) {
      const nothing = document.createElement("span");
      nothing.className = "k-notif-nothing";
      nothing.append(text("Nothing new.", "body", "on_surface_variant"));
      rows.push(nothing);
    }
    list.replaceChildren(...rows);
    say();
    if (options.fanned !== undefined) {
      const lifted = placed[options.fanned]?.element;
      if (lifted) requestAnimationFrame(() => recede(lifted, true));
    }
    if (options.unfolded !== undefined) placed[options.unfolded]?.element.cards()[0]?.press();
  }

  /** Every row but `lifted` sits back, and those it covers go. */
  function recede(lifted: GroupElement | null, still = false): void {
    list.toggleAttribute("data-receding", lifted !== null);
    if (!lifted) return;
    const top = lifted.offsetTop;
    const cover = Math.max(lifted.offsetHeight, lifted.scrollHeight);
    for (const each of list.children) {
      const row = each as HTMLElement;
      if (row === lifted) {
        row.classList.remove("k-notif-behind");
        continue;
      }
      row.classList.add("k-notif-behind");
      const covered = row.offsetTop < top + cover && row.offsetTop + row.offsetHeight > top;
      row.toggleAttribute("data-covered", covered);
      if (still) row.style.setProperty("--recede", "1");
    }
  }

  function dismiss(notes: readonly Note[], which: number): void {
    sections = sections.map((section) => ({
      label: section.label,
      groups: section.groups.map((g) => (g === notes ? (which < 0 ? [] : g.filter((_, k) => k !== which)) : g)).filter((g) => g.length > 0),
    }));
    draw();
  }

  function select(scroll = true): void {
    placed.forEach((each, index) => each.element.select(at === index, inner));
    const current = placed[at]?.element;
    if (!scroll || !current) return;
    const view = page.view;
    const top = current.offsetTop + (current.isOpen() ? current.cardTop(inner) : 0);
    const bottom = top + (current.isOpen() ? (current.cards()[inner]?.offsetHeight ?? 0) : current.offsetHeight);
    if (top < view.scrollTop) view.scrollTop = top;
    else if (bottom > view.scrollTop + view.clientHeight) view.scrollTop = bottom - view.clientHeight;
  }

  draw();

  page.element.append(keysButton(options.keys ?? false));

  if (options.live) {
    dndParts.button.addEventListener("click", () => toggleDnd());
    clear.addEventListener("click", () => {
      sections = [];
      at = -1;
      draw();
    });
    page.view.tabIndex = 0;
    page.view.addEventListener("keydown", (event) => {
      const current = placed[at]?.element;
      const ctrl = event.ctrlKey;
      const down = event.key === "ArrowDown" || (!ctrl && event.key === "j") || (ctrl && (event.key === "j" || event.key === "n"));
      const up = event.key === "ArrowUp" || (!ctrl && event.key === "k") || (ctrl && (event.key === "k" || event.key === "p"));
      if (down || up) {
        event.preventDefault();
        const step = down ? 1 : -1;
        if (current?.isOpen() && inner + step >= 0 && inner + step < current.count) inner += step;
        else {
          at = Math.max(0, Math.min(placed.length - 1, at + step));
          inner = up && placed[at]?.element.isOpen() ? placed[at]!.element.count - 1 : 0;
        }
        select();
        return;
      }
      if (!current) return;
      if (event.key === "Enter" && event.shiftKey) event.preventDefault();
      else if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        if (!event.repeat) current.press(inner);
      } else if (event.key === "ArrowRight" || event.key === "l") {
        current.setOpen(true);
        inner = 0;
        select();
      } else if (event.key === "ArrowLeft" || event.key === "h") {
        current.setOpen(false);
        inner = 0;
        select();
      } else if (event.key === "Delete") {
        const notes = placed[at]!.notes;
        dismiss(notes, current.isOpen() ? inner : -1);
        inner = Math.max(0, Math.min(inner, notes.length - 2));
        at = Math.min(at, placed.length - 1);
        select();
      } else if (event.key === "d" && !event.ctrlKey && !event.altKey && !event.metaKey) toggleDnd();
      else if (event.key === "Escape") {
        if (current.fold() || current.isOpen()) {
          event.preventDefault();
          event.stopPropagation();
          if (!current.fold()) current.setOpen(false);
        }
      } else return;
    });
  }

  function toggleDnd(): void {
    dnd = !dnd;
    const next = dndButton(dnd);
    next.button.addEventListener("click", () => toggleDnd());
    dndParts.anchor.replaceWith(next.anchor);
    dndParts = next;
    // The keyboard stays where it was: on the button the pointer pressed.
    next.button.focus({ preventScroll: true, focusVisible: false } as FocusOptions);
    say();
  }

  if (at >= 0) requestAnimationFrame(() => select(false));
  return page.element;
}

/** The panel on its screen: the right edge from the bar to the frame's foot. */
export function panelOnScreen(options: PanelOptions = {}, height = "var(--size-app_min_height)"): HTMLElement {
  return onScreen(panel(options), "right", undefined, height);
}

/** The popups' or the peek's stack: its groups, `space.s8` apart, `space.s8` inside. */
export function stackOf(...parts: HTMLElement[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-notif-stack";
  box.append(...parts);
  return box;
}

/** The peek's footer: +N more, then Mark all read and Clear. */
export function peekFooter(more: number): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-notif-footer";
  const spacer = document.createElement("span");
  spacer.className = "k-notif-fill";
  const quiet = (label: string) => {
    const each = button({ label, kind: "Text", size: "Sm" });
    each.tabIndex = -1;
    return each;
  };
  box.append(...(more > 0 ? [quiet(`+${more} more`)] : []), spacer, quiet("Mark all read"), quiet("Clear"));
  return box;
}

/** A stack on its screen's corner sheet: the top edge, flush with the right, under the bar. */
export function cornerOnScreen(content: HTMLElement, height = "var(--size-app_height)"): HTMLElement {
  const { screen: room } = desk(CORNER_WIDTH, height);
  hang(room, sheet({ content, edge: "top", corner: true }));
  return room.element;
}

/**
 * Critical popups while an open panel covers the corner: the system monitor open from its pill, and
 * under it, on the right edge `radius.surface` and `space.s12` below it, a stack of the critical ones
 * alone. Live, the pill opens and closes the panel; the stack comes and goes with it, and keeps its
 * place as the panel goes, so it retracts where it is.
 */
export function belowOnScreen(critical: readonly (readonly Note[])[], live = false, height = "calc(var(--size-app_height) + var(--space-s64) * 2)"): HTMLElement {
  const { screen: room, buttons } = desk(CORNER_WIDTH, height);
  const cover = sheet({ content: system({ count: 1 }).element, edge: "right" });
  hang(room, cover);
  const stack = stackOf(...critical.map((notes) => group(notes, { popup: true, live })));
  const below = sheet({ content: stack, edge: "right", start: "0px" });
  hang(room, below);
  room.element.dataset.open = "";
  // The stack starts where the panel ends, which only layout knows.
  const place = () => {
    for (const part of below.parts) part.style.setProperty("--sheet-start", `calc(${cover.box.offsetHeight}px + var(--radius-surface) + var(--space-s12))`);
  };
  new ResizeObserver(place).observe(cover.box);
  if (live) {
    for (const part of [...cover.parts, ...below.parts]) part.dataset.live = "";
    const show = (on: boolean) => {
      for (const part of [...cover.parts, ...below.parts]) {
        part.toggleAttribute("data-shown", on);
        part.toggleAttribute("data-settled", on);
      }
      room.element.toggleAttribute("data-open", on);
    };
    let open = true;
    requestAnimationFrame(() => show(true));
    buttons[0]?.addEventListener("click", () => show((open = !open)));
  }
  return room.element;
}
