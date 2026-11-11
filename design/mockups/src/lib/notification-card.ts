// A notification card: `KubaOS.Components`' `NotificationCard`, QML's `CardFace` in the shell today
// (`docs/design/components/notification-card.md`). One notification: a header band with who it is
// from and when (the app's icon or the notification's image on a small icon tile, an accent dot
// while unread, the app's name, a stack's count, the time), the dismiss button and a chevron while
// the text is cut off; under it the summary, two lines of the body, and its buttons. A click,
// Enter or Space unfolds the cut-off text, or asks a stack to open; Esc folds it. Hover and the
// keyboard light the accent's tube inside its edge, as on a pressable card.

import "./notification.css";
import { badge } from "./badge.ts";
import { button } from "./button.ts";
import { chip } from "./chip.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { iconButton } from "./icon-button.ts";
import { iconTile } from "./icon-tile.ts";
import { probe } from "./probe.ts";
import { nested, role } from "./radius.ts";
import { type Pinned } from "./control.ts";
import { clamped, refit, text, unclamp } from "./text.ts";

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


export interface CardOptions {
  /** How many the card stands for, past one a stack: its count chip, and its dismiss takes them all. */
  readonly count?: number;
  /** The keyboard is on it in its list: its tube lit, its dismiss shown. */
  readonly selected?: boolean;
  /** A state shown whatever the pointer and the keyboard do, as the specimens pin them. */
  readonly pinned?: Pinned;
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
  const pinned = options.pinned ?? (options.selected ? "lit" : undefined);
  if (pinned) face.dataset.pinned = pinned;

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
  const dismiss = iconButton({ icon: "close", label: count > 1 ? `Dismiss all ${count}` : "Dismiss", kind: "Text", size: "Sm" });
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
    unclamp(said);
    const full = said.offsetHeight;
    face.removeAttribute("data-open");
    const folded = said.offsetHeight;
    said.style.removeProperty("height");
    face.style.setProperty("--folded", `${folded}px`);
    face.style.setProperty("--full", `${full}px`);
    face.style.removeProperty("--unfold");
    face.toggleAttribute("data-open", was);
    refit(said);
    const foldable = full > folded + 0.5;
    face.toggleAttribute("data-foldable", foldable);
    // A card a click does something on is a control: hover and the keyboard light its tube.
    face.classList.toggle("k-control", opens || foldable);
    face.toggleAttribute("data-hot", opens || foldable);
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
    // Taken off and set again, so a fade under reduce motion plays each time.
    face.removeAttribute("data-unfolding");
    face.removeAttribute("data-folding");
    void face.offsetWidth;
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

