// The notification surfaces' parts, as `config/quickshell/modules/notifications` draws them: the
// card (`CardFace.qml`), the group that stacks one app's cards (`CardGroup.qml`), and the lists the
// panel, the popups and the peek hold them in. Each is put together from the library's parts (the
// icon tile, the badge, the chip, the buttons and the focus ring); the boxes between them are the
// build's, by its tokens. The pages under `shell/` show them; this file is no page.

import "./notification-parts.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { badge } from "../../lib/badge.ts";
import { button } from "../../lib/button.ts";
import { chip } from "../../lib/chip.ts";
import { focusRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { keyHints } from "../../lib/keycap.ts";
import { fill, row, stack } from "../../lib/layout.ts";
import { type MenuEntry, menu } from "../../lib/menu.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { probe } from "../../lib/probe.ts";
import { nested, role } from "../../lib/radius.ts";
import { advance, centreFor, clamped, text } from "../../lib/text.ts";
import { popover, tooltip } from "../../lib/tooltip.ts";
import { onScreen } from "../templates/panels.ts";

/** A card's corner: the panel's surface radius, `space.s8` in. */
export const CORNER = nested(role("surface"), "s8");

/** One notification, as a card shows it. */
export interface Note {
  readonly app: string;
  readonly summary: string;
  readonly body: string;
  /** The day and time it came: the time alone today, the day's name before it otherwise. */
  readonly when: string;
  /** The main action's label, filled; none for a card with no actions. */
  readonly main?: string;
  readonly others?: readonly string[];
  readonly unread?: boolean;
  readonly critical?: boolean;
  /** The notification's image, over the tile. */
  readonly photo?: string;
}

export const OLA: Note = { app: "Slack", summary: "Ola", body: "Stand-up moved to 10:30 tomorrow.", when: "21:47", unread: true };
export const ANIA: Note = { app: "Slack", summary: "Ania", body: "Pushed the fix for the bar, can you check it before the stand-up?", when: "21:46", main: "Reply", unread: true };
export const MARCO: Note = { app: "Slack", summary: "Marco", body: "Lunch at 13:00?", when: "21:44", unread: true };
export const DEPLOYS: Note = { app: "Slack", summary: "#deploys", body: "kubaos-ui 0.4 is out on the staging mirror.", when: "20:58", unread: true };
export const BUILD: Note = {
  app: "Ghostty",
  summary: "Build finished",
  body: "kubaos-ui and kubaos-specimen built in 41 s; 312 tests passed, none skipped. The specimen's shots changed for the components page.",
  when: "21:41",
  main: "Open",
  others: ["Open log"],
  unread: true,
  photo: wallpaper,
};
export const KUBA_ONE: Note = { app: "Signal", summary: "Kuba", body: "Or ramen at Ippudo, if Fuunji is full.", when: "12:08" };
export const KUBA_TWO: Note = { app: "Signal", summary: "Kuba", body: "Lunch at Fuunji? The queue is short today.", when: "12:08" };
export const BATTERY: Note = {
  app: "KubaOS",
  summary: "Battery at 20%",
  body: "Plug in soon, or switch to Power Saver. At this rate it lasts about 40 min; Power Saver dims the screen and slows the apps in the background to stretch it.",
  when: "Yesterday 19:30",
};
export const UPDATE: Note = { app: "KubaOS", summary: "Update ready", body: "KubaOS 0.9 is ready to install.", when: "Yesterday 19:30" };
export const CRITICAL: Note = { app: "KubaOS", summary: "Battery at 5%", body: "Plug in now: it suspends in 3 min.", when: "21:48", main: "Power Saver", unread: true, critical: true };

export const SLACK: readonly Note[] = [OLA, ANIA, MARCO, DEPLOYS];
export const SIGNAL: readonly Note[] = [KUBA_ONE, KUBA_TWO];
export const KUBAOS: readonly Note[] = [BATTERY, UPDATE];

/** A card pinned in a state, as the specimens pin them. */
export type CardPinned = "hovered" | "pressed";

export interface CardOptions {
  /** How many the card stands for, past one a stack: its count chip, and its dismiss takes them all. */
  readonly count?: number;
  /** The keyboard is on it: its tube lit, its dismiss shown. */
  readonly selected?: boolean;
  readonly pinned?: CardPinned;
  readonly unfolded?: boolean;
  /** Its whole text laid out at its folded height, as a fold's move starts from. */
  readonly opened?: boolean;
  /** A popup, with its time left along its bottom. */
  readonly popup?: boolean;
  readonly remaining?: number;
  /** A click asks a stack to open rather than unfolding. */
  readonly opens?: boolean;
  /** Answers the pointer and Enter: unfolds and folds itself. */
  readonly live?: boolean;
  /** Takes Tab itself, as a card alone on a page does; in a list the list has the keyboard. */
  readonly focusable?: boolean;
  /** What a click on a card that opens its stack does. */
  readonly onExpand?: () => void;
  readonly onDismiss?: () => void;
}

/** Whether a label is the sender's own Mark read, so the card adds none of its own. */
function marksRead(label: string): boolean {
  return /^mark (as )?read$/i.test(label.trim());
}

/** The card's text: its summary and body, two lines each while folded. */
function words(note: Note): HTMLElement {
  const parts: HTMLElement[] = [];
  if (note.summary !== "") {
    const summary = clamped(note.summary, "body_strong", "on_surface", 2);
    summary.classList.add("k-notif-summary");
    summary.dataset.wrapped = "";
    parts.push(summary);
  }
  if (note.body !== "") {
    const body = clamped(note.body, "body", "on_surface_variant", 2);
    body.classList.add("k-notif-body");
    body.dataset.wrapped = "";
    parts.push(body);
  }
  const box = document.createElement("span");
  box.className = "k-notif-text";
  box.append(...parts);
  return box;
}

/** The buttons: the main action filled (Danger on a critical card), the others as text, then Mark read. */
function buttons(note: Note): HTMLElement | null {
  const others = note.others ?? [];
  const own = note.unread === true && ![note.main ?? "", ...others].some(marksRead);
  const list: HTMLElement[] = [];
  if (note.main) list.push(button({ label: note.main, kind: note.critical ? "Danger" : "Filled", size: "Sm" }));
  for (const other of others) list.push(button({ label: other, kind: "Text", size: "Sm" }));
  if (own) list.push(button({ label: "Mark read", kind: "Text", size: "Sm" }));
  if (list.length === 0) return null;
  const box = document.createElement("span");
  box.className = "k-notif-buttons";
  box.toggleAttribute("data-text-only", !note.main);
  box.append(...list);
  for (const each of list) each.tabIndex = -1;
  return box;
}

/** A card: its header band, its words and its buttons, its time left as a popup. */
export function card(note: Note, options: CardOptions = {}): HTMLElement {
  const face = document.createElement("span");
  face.className = "k-notif-card";
  face.style.setProperty("--notif-corner", CORNER.css);
  const count = options.count ?? 1;
  face.toggleAttribute("data-critical", note.critical ?? false);
  face.toggleAttribute("data-selected", options.selected ?? false);
  if (options.selected) face.dataset.pinned = "lit";
  if (options.pinned === "hovered") face.dataset.hovered = "";
  if (options.pinned === "pressed") face.dataset.pressed = "";

  const tile = document.createElement("span");
  tile.className = "k-notif-tile";
  tile.append(iconTile({ glyph: note.critical ? "warning" : "notifications", size: "Sm", danger: note.critical ?? false, ...(note.photo ? { photo: note.photo } : {}) }));
  if (note.unread) tile.append(badge({ accent: true }));
  const app = clamped(note.app || "Notification", "label");
  app.classList.add("k-notif-app");
  const who = document.createElement("span");
  who.className = "k-notif-who";
  who.append(tile, app);
  if (count > 1) who.append(chip({ label: String(count) }));
  if (note.when !== "") {
    const when = text(note.when, "small");
    when.classList.add("k-notif-when");
    who.append(when);
  }
  const chevron = document.createElement("span");
  chevron.className = "k-notif-chevron";
  const dismiss = iconButton({ icon: "close", label: count > 1 ? `Dismiss all ${count}` : "Dismiss", kind: "Text", size: "Sm", hint: false });
  dismiss.classList.add("k-notif-dismiss");
  dismiss.tabIndex = -1;
  dismiss.addEventListener("click", (event) => {
    event.stopPropagation();
    options.onDismiss?.();
  });
  const band = document.createElement("span");
  band.className = "k-notif-band";
  band.append(who, chevron, dismiss);

  const said = words(note);
  const body = document.createElement("span");
  body.className = "k-notif-words";
  body.append(said);
  const acts = buttons(note);
  if (acts) body.append(acts);
  face.append(band, body);
  if (options.popup && !note.critical) {
    const left = document.createElement("span");
    left.className = "k-notif-time-left";
    face.style.setProperty("--remaining", String(options.remaining ?? 1));
    face.append(left);
  }
  const ring = focusRing(face, "content", "accent", false);
  ring.style.setProperty("--corner", CORNER.css);

  let unfolded = options.unfolded ?? false;
  const opens = options.opens === true && count > 1;
  /** Whether the text is cut off folded, and the heights it unfolds between. */
  const measure = () => {
    const was = face.hasAttribute("data-open");
    face.setAttribute("data-open", "");
    face.style.setProperty("--unfold", "1");
    face.style.setProperty("--folded", "auto");
    face.style.setProperty("--full", "auto");
    said.style.height = "auto";
    const full = said.offsetHeight;
    face.removeAttribute("data-open");
    const folded = said.offsetHeight;
    said.style.removeProperty("height");
    face.style.setProperty("--folded", `${folded}px`);
    face.style.setProperty("--full", `${full}px`);
    face.style.removeProperty("--unfold");
    face.toggleAttribute("data-open", was);
    const foldable = full > folded + 0.5;
    face.toggleAttribute("data-foldable", foldable);
    face.toggleAttribute("data-clickable", opens || foldable);
    chevron.replaceChildren(...(foldable ? [icon(unfolded ? "expand_less" : "expand_more", "var(--type-button_small-icon)", "var(--fg-muted)")] : []));
  };
  if (unfolded || options.opened) face.setAttribute("data-open", "");
  face.style.setProperty("--unfold", unfolded ? "1" : "0");
  face.style.setProperty("--folded", "0px");
  face.style.setProperty("--full", "0px");
  let width = -1;
  new ResizeObserver(() => {
    if (face.offsetWidth === width) return;
    width = face.offsetWidth;
    measure();
  }).observe(face);

  /** Unfolds or folds the text, as a click, Enter or Esc does. */
  const unfold = (open: boolean) => {
    if (open === unfolded || !face.hasAttribute("data-foldable")) return;
    unfolded = open;
    face.toggleAttribute("data-unfolding", open);
    face.toggleAttribute("data-folding", !open);
    if (open) face.setAttribute("data-open", "");
    face.style.setProperty("--unfold", open ? "1" : "0");
    chevron.replaceChildren(icon(open ? "expand_less" : "expand_more", "var(--type-button_small-icon)", "var(--fg-muted)"));
    if (!open) {
      const done = (event: TransitionEvent) => {
        if (event.propertyName !== "--unfold" || unfolded) return;
        face.removeAttribute("data-open");
        face.removeEventListener("transitionend", done);
      };
      face.addEventListener("transitionend", done);
    }
  };
  const press = () => {
    if (opens) options.onExpand?.();
    else unfold(!unfolded);
  };
  (face as CardElement).press = press;
  (face as CardElement).fold = () => {
    const was = unfolded;
    unfold(false);
    return was;
  };
  if (options.live) {
    face.addEventListener("click", (event) => {
      if ((event.target as Element).closest(".k-notif-buttons, .k-notif-dismiss")) return;
      press();
    });
  }
  if (options.focusable) {
    face.tabIndex = 0;
    face.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        if (!event.repeat && !event.shiftKey) press();
      } else if (event.key === "Escape" && unfolded) {
        event.preventDefault();
        unfold(false);
      }
    });
  }
  return probe(face, { role: "component", name: "NotificationCard", variant: [note.critical ? "critical" : "", count > 1 ? "stack" : "", options.popup ? "popup" : ""].filter(Boolean).join(" "), fixed: { width: false, height: false, holds: true } });
}

/** A card with what a press, Enter and Esc do on it. */
export interface CardElement extends HTMLElement {
  press: () => void;
  /** Folds it; whether it was unfolded. */
  fold: () => boolean;
}

export interface GroupOptions {
  /** Drawn spread into its preview, or opened. */
  readonly fan?: number;
  readonly open?: boolean;
  /** Answers the pointer: a rest previews, a click opens, Show less closes. */
  readonly live?: boolean;
  /** The keyboard on it: on its newest while stacked, on `current` opened. */
  readonly selected?: boolean;
  readonly current?: number;
  readonly popup?: boolean;
  /** What a dismiss on a card does: the card's index, or -1 for the whole stack. */
  readonly onDismiss?: (index: number) => void;
  /** Its preview spreading or tucking in, for an owner fading the rows behind it. */
  readonly onRaise?: (raised: boolean) => void;
  readonly onOpen?: (open: boolean) => void;
}

/** A group with what its owner moves it by. */
export interface GroupElement extends HTMLElement {
  readonly count: number;
  isOpen: () => boolean;
  setOpen: (open: boolean) => void;
  /** Where card `index` sits in it, opened, for keeping it in view. */
  cardTop: (index: number) => number;
  select: (on: boolean, current: number) => void;
  /** What a click, Enter or Space on it does. */
  press: (current: number) => void;
  /** Folds an unfolded card; whether there was one. */
  fold: () => boolean;
  cards: () => CardElement[];
}

/**
 * One app's notifications as one thing: a stack at rest, a preview while the pointer rests on it, every
 * card under its heading opened. A single notification is the card alone.
 */
export function group(notes: readonly Note[], options: GroupOptions = {}): GroupElement {
  const count = notes.length;
  const element = document.createElement("span") as unknown as GroupElement;
  element.className = "k-notif-group";
  element.style.setProperty("--notif-corner", CORNER.css);
  const fanCount = Math.min(count, 3);
  const edges = Math.max(0, Math.min(count - 1, 2));
  let open = (options.open ?? false) && count > 1;
  element.style.setProperty("--fan", String(options.fan ?? 0));
  element.style.setProperty("--opening", open ? "1" : "0");
  element.toggleAttribute("data-open", open);
  element.toggleAttribute("data-raised", (options.fan ?? 0) > 0);

  const slots: HTMLElement[] = [];
  const faces: CardElement[] = [];
  notes.forEach((note, k) => {
    const slot = document.createElement("span");
    slot.className = "k-notif-slot";
    const edge = Math.min(k, edges);
    slot.style.setProperty("--k-rx", `calc(var(--space-s8) * ${edge})`);
    slot.style.setProperty("--k-ry", `calc(var(--space-s8) * ${edge})`);
    slot.style.setProperty("--k-spread", k < fanCount ? "1" : "0");
    slot.style.zIndex = String((count - k) * 3 + 1);
    slot.toggleAttribute("data-newest", k === 0);
    const face = card(note, {
      count: k === 0 && !open ? count : 1,
      opens: k === 0 && count > 1,
      live: options.live ?? false,
      popup: options.popup ?? false,
      selected: (options.selected ?? false) && (open ? (options.current ?? 0) === k : k === 0),
      onExpand: () => setOpen(true),
      onDismiss: () => options.onDismiss?.(k === 0 && !open ? -1 : k),
    }) as CardElement;
    if (k > 0) {
      const tucked = document.createElement("span");
      tucked.className = "k-notif-edge";
      slot.append(tucked);
    }
    if (k < fanCount) {
      const lift = document.createElement("span");
      lift.className = "k-notif-lift";
      slot.prepend(lift);
    }
    slot.append(face);
    slots.push(slot);
    faces.push(face);
  });

  // Each card's shadow cast onto the edges tucked under it.
  const casts: HTMLElement[] = [];
  for (let k = 0; k < edges; k++) {
    const cast = document.createElement("span");
    cast.className = "k-notif-cast";
    cast.style.zIndex = String((count - k) * 3 - 1);
    cast.append(document.createElement("span"));
    casts.push(cast);
  }

  const more = count - fanCount;
  const moreLine = document.createElement("span");
  moreLine.className = "k-notif-more k-notif-slot";
  moreLine.style.zIndex = String((count + 1) * 3);
  if (more > 0) moreLine.append(text(`+${more} more · click to open`, "small", "primary"));

  const less = button({ label: "Show less", kind: "Text", size: "Sm" });
  less.tabIndex = -1;
  less.addEventListener("click", (event) => {
    event.stopPropagation();
    setOpen(false);
  });
  const heading = row("s8", "centre", clamped(notes[0]?.app || "Notification", "label"), chip({ label: String(count) }), fill(), less);
  heading.classList.add("k-notif-heading");
  heading.style.zIndex = String((count + 2) * 3);
  heading.toggleAttribute("data-shown", open);

  element.append(...slots, ...casts, moreLine, heading);

  /** The cards' heights and where each sits spread, as the CSS works the boxes out from. */
  const layout = () => {
    const heights = faces.map((face) => face.offsetHeight);
    const h0 = heights[0] ?? 0;
    element.style.setProperty("--h0", `${h0}px`);
    const gap = Number.parseFloat(getComputedStyle(element).getPropertyValue("--space-s8")) || 8;
    const under = (Number.parseFloat(getComputedStyle(element).getPropertyValue("--size-control_sm")) || 32) + gap;
    let y = 0;
    const offsets = heights.map((h) => {
      const at = y;
      y += h + gap;
      return at;
    });
    slots.forEach((slot, k) => {
      slot.style.setProperty("--k-off", `${offsets[k]}px`);
      slot.style.setProperty("--k-h", `${heights[k]}px`);
    });
    const last = count - 1;
    element.style.setProperty("--rest-h", `${h0 + edges * gap}px`);
    element.style.setProperty("--open-h", count > 1 ? `${under + (offsets[last] ?? 0) + (heights[last] ?? 0)}px` : `${h0}px`);
    const fanLast = fanCount - 1;
    moreLine.style.setProperty("--k-rx", `calc(var(--space-s8) * ${Math.min(fanLast, edges)})`);
    moreLine.style.setProperty("--k-ry", `calc(var(--space-s8) * ${Math.min(fanLast, edges)})`);
    moreLine.style.setProperty("--k-spread", "1");
    moreLine.style.setProperty("--k-off", `${(offsets[fanLast] ?? 0) + (heights[fanLast] ?? 0) + gap}px`);
    moreLine.style.setProperty("--k-h", "0px");
    // The cast shadows: card k's, cut to the edges under it.
    casts.forEach((cast, k) => {
      const below = Math.min(k + 1, edges);
      const top = below * gap;
      cast.style.left = `${below * gap}px`;
      cast.style.right = `${below * gap}px`;
      cast.style.top = `${top}px`;
      cast.style.height = `${h0 + (edges - below) * gap}px`;
      const shadow = cast.firstElementChild as HTMLElement;
      shadow.style.left = `${k * gap - below * gap}px`;
      shadow.style.width = `calc(100% + ${2 * (below - k) * gap}px)`;
      shadow.style.top = `${k * gap - top}px`;
      shadow.style.height = `${h0}px`;
    });
  };
  new ResizeObserver(layout).observe(element);
  for (const face of faces) new ResizeObserver(layout).observe(face);

  function setOpen(to: boolean): void {
    if (count < 2 || to === open) return;
    open = to;
    element.toggleAttribute("data-open", open);
    element.removeAttribute("data-fanning");
    element.style.setProperty("--opening", open ? "1" : "0");
    if (open) heading.setAttribute("data-shown", "");
    faces[0]?.querySelector(".k-chip")?.toggleAttribute("hidden", open);
    if (!open) shut = true;
    options.onOpen?.(open);
  }

  let armed = false;
  let shut = false;
  let dwell = 0;
  const preview = (on: boolean) => {
    element.toggleAttribute("data-fanning", on);
    element.style.setProperty("--fan", on ? "1" : "0");
    if (on) element.setAttribute("data-raised", "");
    options.onRaise?.(on);
  };
  element.addEventListener("transitionend", (event) => {
    if (event.propertyName === "--fan" && getComputedStyle(element).getPropertyValue("--fan").trim() === "0") element.removeAttribute("data-raised");
    if (event.propertyName === "--opening" && !open) heading.removeAttribute("data-shown");
  });
  if (options.live && count > 1) {
    element.addEventListener("pointerenter", () => {
      if (open || shut) return;
      window.clearTimeout(dwell);
      dwell = window.setTimeout(() => {
        armed = true;
        if (!open) preview(true);
      }, Number.parseFloat(getComputedStyle(element).getPropertyValue("--motion-medium")) || 250);
    });
    element.addEventListener("pointerleave", () => {
      window.clearTimeout(dwell);
      shut = false;
      if (armed) preview(false);
      armed = false;
    });
    // A click on the tucked edges or the preview under the newest card opens the group.
    element.addEventListener("click", (event) => {
      if (open || (event.target as Element).closest(".k-notif-slot[data-newest], .k-notif-heading")) return;
      preview(false);
      armed = false;
      setOpen(true);
    });
    faces[0]?.addEventListener("click", () => {
      if (!open) {
        preview(false);
        armed = false;
      }
    });
  }

  Object.assign(element, {
    count,
    isOpen: () => open,
    setOpen,
    cardTop: (index: number) => (slots[index]?.offsetTop ?? 0),
    select: (on: boolean, current: number) => {
      faces.forEach((face, k) => {
        const lit = on && (open ? current === k : k === 0);
        face.toggleAttribute("data-selected", lit);
        if (lit) face.dataset.pinned = "lit";
        else delete face.dataset.pinned;
      });
    },
    press: (current: number) => {
      if (count > 1 && !open) setOpen(true);
      else faces[open ? current : 0]?.press();
    },
    fold: () => faces.some((face) => face.fold()),
    cards: () => faces,
  });
  return probe(element, { role: "component", name: "NotificationGroup", variant: count > 1 ? "stack" : "single", fixed: { width: false, height: false, holds: true } }) as GroupElement;
}

/** `shown` in a room as wide as the popups hold a card. */
export function room(shown: HTMLElement, tall = false): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-notif-room";
  box.toggleAttribute("data-tall", tall);
  box.append(shown);
  return box;
}

/** The panel's line under its title: why it is silenced, how many are new, or that there's nothing. */
export function status(unread: number, count: number, reason?: "manual" | "sharing" | "fullscreen"): string {
  const silenced = { manual: "Silenced until you turn it off", sharing: "Silenced while you share your screen", fullscreen: "Silenced while a window is fullscreen" };
  if (reason) return silenced[reason];
  if (unread > 0) return `${unread} new`;
  return count > 0 ? "All read" : "Silence is golden";
}

/** Do not disturb's tooltip, off and on. */
export function dndTip(on: boolean): HTMLElement {
  return on ? tooltip("Do not disturb is on", "Until you turn it off") : tooltip("Do not disturb", "Turns on by itself while you share your screen or a window is fullscreen");
}

/** The keys button's tooltip: every key the panel takes. */
export function keysTip(): HTMLElement {
  const lines = stack(
    "s8",
    "start",
    keyHints([["↑ ↓", "Move"], ["↵", "Unfold or open"]]),
    keyHints([["⇧↵", "Run its main action"]]),
    keyHints([["→ ←", "Open or close a group"]]),
    keyHints([["Del", "Dismiss"], ["D", "Do not disturb"]]),
    keyHints([["Esc", "Fold, close or leave"]]),
  );
  return tooltip("Keys", "", lines);
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

  let dndButton = iconButton({ icon: "dark_mode", label: dnd ? "Do not disturb is on" : "Do not disturb", kind: dnd ? "Filled" : "Text", size: "Sm" });
  const clear = iconButton({ icon: "clear_all", label: "Clear all", kind: "Text", size: "Sm" });
  const page = panelPage({ title: "Notifications", subtitle: status(0, 1, dnd ? "manual" : undefined), actions: [dndButton, clear], body: list });
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

  const keys = iconButton({ icon: "keyboard", label: "Keys", kind: "Tonal", size: "Sm", hint: false });
  keys.tabIndex = -1;
  const keysBox = document.createElement("span");
  keysBox.className = "k-notif-keys";
  if (options.keys) {
    const tip = keysTip();
    tip.style.position = "absolute";
    tip.style.right = "0";
    tip.style.bottom = "calc(100% + var(--space-s8))";
    keysBox.append(tip);
    keys.dataset.pinned = "lit";
  }
  keysBox.append(keys);
  page.element.append(keysBox);

  if (options.live) {
    dndButton.addEventListener("click", () => toggleDnd());
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
    const next = iconButton({ icon: "dark_mode", label: dnd ? "Do not disturb is on" : "Do not disturb", kind: dnd ? "Filled" : "Text", size: "Sm" });
    next.addEventListener("click", () => toggleDnd());
    dndButton.replaceWith(next);
    dndButton = next;
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

/** A stack on its screen's corner, under the bar, as the popups and the peek hang. */
export function cornerOnScreen(content: HTMLElement, height = "var(--size-app_height)"): HTMLElement {
  return onScreen(content, "right", undefined, height);
}
