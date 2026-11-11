// A notification popup: `KubaOS.Components`' `NotificationPopup` (`docs/design/components/notification-card.md`).
// A new notification in the popup stack, two lines and quiet: the app's symbol on a small icon tile (or,
// for several from one app, their count in the symbol's place), the title, one line of the body, and
// the app and the time at its end. It has no unread dot: a popup is new by being one.
//
// Under the pointer it stays two lines and only its dismiss appears, over the app and the time. A
// click or Enter opens it in place into the NotificationCard it stands for, one at a time; a second
// click on its words, Esc or a click outside folds it. Delete dismisses it. Its time left is a hairline
// at its foot, which its stack runs and holds.

import "./notification.css";
import { focusRing } from "./focus.ts";
import { iconButton } from "./icon-button.ts";
import { iconTile } from "./icon-tile.ts";
import { card, CORNER, type Note } from "./notification-card.ts";
import { probe } from "./probe.ts";
import { clamped, text } from "./text.ts";

/** A notification as a popup shows it: one, or the newest of several from one app. */
export interface PopupNote extends Note {
  /** How many from its app it stands for; past one, the count takes the symbol's place. */
  readonly count?: number;
}

/** A popup pinned in a state, as the specimens pin them. */
export type PopupPinned = "pointed" | "lit";

export interface PopupOptions {
  readonly open?: boolean;
  /** `pointed`: the pointer on it, its dismiss shown; `lit`: the keyboard on it, its tube lit too. */
  readonly pinned?: PopupPinned;
  /** Its time left, 1 to 0. */
  readonly remaining?: number;
  /** Answers the pointer and the keys: opens, folds and dismisses. */
  readonly live?: boolean;
  readonly onDismiss?: () => void;
  /** It opened or folded, for an owner keeping one open at a time and holding its timer. */
  readonly onToggle?: (open: boolean) => void;
  /** One of its card's buttons ran its action. */
  readonly onAct?: () => void;
}

export interface PopupElement extends HTMLElement {
  setOpen: (open: boolean) => void;
  isOpen: () => boolean;
  /** Shows `note` instead: another from the same app joined it. */
  update: (note: PopupNote) => void;
}

/** The two lines: the tile, the title over one line of the body, and the app and time or the dismiss. */
function face(note: PopupNote, dismiss: HTMLElement): HTMLElement {
  const count = note.count ?? 1;
  const tile = iconTile({ glyph: "notifications", size: "Sm", ...(count > 1 ? { count } : {}) });
  const lines = document.createElement("span");
  lines.className = "k-notif-pop-lines";
  lines.append(clamped(note.summary, "label"), clamped(note.body, "small", "on_surface_variant"));
  const end = document.createElement("span");
  end.className = "k-notif-pop-end";
  const meta = text(`${note.app || "Notification"} · ${note.when}`, "small", "on_surface_variant");
  meta.classList.add("k-notif-pop-meta");
  end.append(meta, dismiss);
  const row = document.createElement("span");
  row.className = "k-notif-pop-face";
  row.append(tile, lines, end);
  return row;
}

export function notificationPopup(note: PopupNote, options: PopupOptions = {}): PopupElement {
  const element = document.createElement("span") as unknown as PopupElement;
  element.className = "k-notif-pop";
  element.style.setProperty("--notif-corner", CORNER.css);
  element.style.setProperty("--remaining", String(options.remaining ?? 1));
  if (options.pinned === "pointed") element.dataset.pointed = "";
  if (options.pinned === "lit") element.dataset.pinned = "lit";
  let open = options.open ?? false;
  element.style.setProperty("--open", open ? "1" : "0");
  element.toggleAttribute("data-open", open);

  const clip = document.createElement("span");
  clip.className = "k-notif-pop-clip";
  const faceRoom = document.createElement("span");
  faceRoom.className = "k-notif-pop-face-room";
  const fullRoom = document.createElement("span");
  fullRoom.className = "k-notif-pop-full";
  clip.append(faceRoom, fullRoom);
  const left = document.createElement("span");
  left.className = "k-notif-time-left";
  element.append(clip, left);
  const ring = focusRing(element, "content", "accent", false);
  ring.style.setProperty("--corner", CORNER.css);

  const dismissed = () => options.onDismiss?.();
  const fill = (shown: PopupNote) => {
    const dismiss = iconButton({ icon: "close", label: (shown.count ?? 1) > 1 ? `Dismiss all ${shown.count}` : "Dismiss", kind: "Text", size: "Sm" });
    dismiss.classList.add("k-notif-pop-dismiss");
    dismiss.tabIndex = -1;
    dismiss.addEventListener("click", (event) => {
      event.stopPropagation();
      dismissed();
    });
    faceRoom.replaceChildren(face(shown, dismiss));
    const full = card(shown, { count: shown.count ?? 1, dot: false, onDismiss: dismissed });
    for (const each of full.querySelectorAll<HTMLElement>(".k-notif-buttons .k-button")) {
      each.addEventListener("click", (event) => {
        event.stopPropagation();
        options.onAct?.();
      });
    }
    fullRoom.replaceChildren(full);
    element.setAttribute("aria-label", `${shown.app}: ${shown.summary}`);
  };
  fill(note);

  // Its height runs from the two lines' to the card's as it opens, which only layout knows.
  const measure = () => {
    element.style.setProperty("--face-h", `${faceRoom.offsetHeight}px`);
    element.style.setProperty("--full-h", `${fullRoom.offsetHeight}px`);
  };
  new ResizeObserver(measure).observe(faceRoom);
  new ResizeObserver(measure).observe(fullRoom);

  /** Folds it when a press lands anywhere else. */
  const outside = (event: Event) => {
    if (!element.contains(event.target as Node)) setOpen(false);
  };

  function setOpen(to: boolean): void {
    if (to === open) return;
    open = to;
    element.removeAttribute("data-opening");
    element.removeAttribute("data-folding");
    void element.offsetWidth;
    element.toggleAttribute(to ? "data-opening" : "data-folding", true);
    element.toggleAttribute("data-open", open);
    element.style.setProperty("--open", open ? "1" : "0");
    if (open) document.addEventListener("pointerdown", outside, true);
    else document.removeEventListener("pointerdown", outside, true);
    options.onToggle?.(open);
  }

  if (options.live) {
    element.tabIndex = 0;
    element.setAttribute("role", "button");
    element.addEventListener("click", (event) => {
      if ((event.target as Element).closest(".k-button")) return;
      // Open, only its words fold it: the band and the buttons are the card's.
      if (open && !(event.target as Element).closest(".k-notif-words")) return;
      setOpen(!open);
    });
    element.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && !event.repeat) {
        event.preventDefault();
        setOpen(!open);
      } else if (event.key === "Delete") {
        event.preventDefault();
        dismissed();
      } else if (event.key === "Escape" && open) {
        event.preventDefault();
        event.stopPropagation();
        setOpen(false);
      }
    });
  }

  Object.assign(element, {
    setOpen,
    isOpen: () => open,
    update: (next: PopupNote) => fill(next),
  });
  return probe(element, { role: "component", name: "NotificationPopup", variant: (note.count ?? 1) > 1 ? "count" : "", fixed: { width: false, height: false, holds: true } }) as PopupElement;
}
