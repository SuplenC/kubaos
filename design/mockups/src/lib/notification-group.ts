// A notification group: `KubaOS.Components`' `NotificationGroup`, QML's `CardGroup` in the shell
// today (`docs/design/components/notification-group.md`). One app's notifications as one thing: at
// rest a stack, its newest card over one or two narrower edges; under a pointer resting `medium`, a
// preview of up to three cards spread over what is behind, lifted to the ladder's step 2; opened, every
// card under a heading with the app, the count and Show less. A single notification is the card alone.

import "./notification.css";
import { button } from "./button.ts";
import { chip } from "./chip.ts";
import { fill, row } from "./layout.ts";
import { type CardElement, card, CORNER, type Note } from "./notification-card.ts";
import { raise } from "./elevation.ts";
import { raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";
import { probe } from "./probe.ts";
import { clamped, text } from "./text.ts";

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

/** The duration token `name` in milliseconds under the speed setting. */
function ms(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
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
  element.toggleAttribute("data-stacked", count > 1);
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
    // At rest a tucked card draws over the newest card's glow, the one above it over it in turn.
    slot.style.setProperty("--rest-z", String(count * 3 + 1 + count - k));
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
      // A tucked card stands on the ladder's step 1, so its peek reads against the panel by day and
      // by night; the card above casts its contact layer onto it, cut to its shape.
      const tucked = raise(document.createElement("span"), 1);
      tucked.className += " k-notif-edge";
      const cast = document.createElement("span");
      cast.className = "k-notif-cast";
      tucked.append(cast);
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

  // Under the cards, the whole preview, gaps included, from the newest card's top to the line under
  // the spread: the pointer crossing from one spread card to the next never leaves the group. It
  // grows and shrinks with `--fan`, so it follows the cards as they move, and goes while it is open.
  const catcher = document.createElement("span");
  catcher.className = "k-notif-catcher k-notif-slot";
  for (const [name, value] of [["--k-rx", "0px"], ["--k-ry", "0px"], ["--k-spread", "1"], ["--k-off", "0px"]]) catcher.style.setProperty(name!, value!);

  // The stack's one shadow on what holds it, under every card: the ladder's step 1 from its lowest
  // edge, so no tucked card sinks another under its own shadow.
  const under = document.createElement("span");
  under.className = "k-notif-under";
  under.style.setProperty("--k-inset", `calc(var(--space-s8) * ${edges})`);
  under.hidden = edges === 0;

  element.append(under, catcher, ...slots, moreLine, heading);

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
    const spread = (offsets[fanLast] ?? 0) + (heights[fanLast] ?? 0);
    const line = (moreLine.firstElementChild as HTMLElement | null)?.offsetHeight ?? 0;
    catcher.style.setProperty("--k-h", `${more > 0 ? spread + gap + line : spread}px`);
  };
  new ResizeObserver(layout).observe(element);
  for (const face of faces) new ResizeObserver(layout).observe(face);

  /** Under reduce motion each state comes at once, its cards fading in over `short`. */
  function refade(): void {
    if (!element.closest("[data-reduced]")) return;
    element.removeAttribute("data-refade");
    void element.offsetWidth;
    element.setAttribute("data-refade", "");
  }

  function setOpen(to: boolean): void {
    if (count < 2 || to === open) return;
    open = to;
    refade();
    element.toggleAttribute("data-open", open);
    element.removeAttribute("data-fanning");
    element.style.setProperty("--opening", open ? "1" : "0");
    if (open) heading.setAttribute("data-shown", "");
    else if (element.closest("[data-reduced]")) heading.removeAttribute("data-shown");
    faces[0]?.querySelector(".k-chip")?.toggleAttribute("hidden", open);
    if (!open) shut = true;
    options.onOpen?.(open);
  }

  let armed = false;
  let shut = false;
  let dwell = 0;
  let settle = 0;
  const preview = (on: boolean) => {
    refade();
    element.toggleAttribute("data-fanning", on);
    element.style.setProperty("--fan", on ? "1" : "0");
    if (on) element.setAttribute("data-raised", "");
    else if (element.closest("[data-reduced]")) element.removeAttribute("data-raised");
    options.onRaise?.(on);
  };
  element.addEventListener("transitionend", (event) => {
    if (event.propertyName === "--fan" && getComputedStyle(element).getPropertyValue("--fan").trim() === "0") element.removeAttribute("data-raised");
    if (event.propertyName === "--opening" && !open) heading.removeAttribute("data-shown");
  });
  if (options.live && count > 1) {
    element.addEventListener("pointerenter", () => {
      window.clearTimeout(settle);
      if (open || shut || armed) return;
      window.clearTimeout(dwell);
      dwell = window.setTimeout(() => {
        armed = true;
        if (!open) preview(true);
      }, ms("medium"));
    });
    // Let go once the hover has settled, `duration.instant` later (a turn of the event loop), not as
    // it changes: a card moving under the pointer or the catcher growing can pass it between parts
    // of the group on the way, and letting go there would tuck the preview the moment it spreads.
    element.addEventListener("pointerleave", () => {
      window.clearTimeout(settle);
      settle = window.setTimeout(() => {
        if (element.matches(":hover")) return;
        window.clearTimeout(dwell);
        shut = false;
        if (armed) preview(false);
        armed = false;
      }, ms("instant"));
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

