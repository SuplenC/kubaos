// A question over a dimmed screen: `kubaos-ui`'s `dialog::dialog`, `dialog::prompt` and
// `dialog::scrim`, `KubaOS.Components`' `Dialog` and `Scrim` (`docs/design/components/dialog.md`).
//
// The dialog is `size.dialog` wide, `space.s24` inside on every side, at the content radius, raised
// to the ladder's step 3 (`lib/elevation.ts`). A 24 symbol (`heading`'s) beside a `heading` title,
// the `body` `space.s12` under the title and in line with it, what it lists or asks for (a labelled
// field) `space.s16` under the body, in line with it too, and its button bar (`lib/button-bar.ts`)
// `space.s24` under the last, across its whole width. A Danger dialog shows the warning sign in
// `error` with an edge of `error`, and its action is a Danger button.
//
// It is never shorter than `aspect.dialog` of its width: a dialog that asks little gains the room
// above its buttons, which stay at its foot, and its words stay at the top. It is never taller
// than its room less `space.s48` above and below, so the dimmed screen shows round it; past that
// only the middle, the body and what follows it, scrolls, with no bar of its own, between the
// title and the buttons, which stay. While more of it is above or below, a soft hairline across
// the dialog's padding marks that edge.
//
// The scrim dims everything behind it and takes every press, which does nothing; the dialog sits
// in its middle on whole device pixels. Opening, the keyboard goes into the dialog, onto what it
// asks for, else Cancel on a Danger dialog, else the last action; Tab and Shift+Tab go round it
// and never leave, scrolling what they land on into view; the arrows and the page keys scroll a
// middle too tall to show whole; Esc is its dismiss button; closing gives the keyboard back to
// what opened it.

import { iconInLine } from "./button.ts";
import { type Action, buttonBar, type Tone } from "./button-bar.ts";
import { raise } from "./elevation.ts";
import { divider, filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { text, wrapped } from "./text.ts";
import { length } from "./tokens.ts";

export type { Action, Tone } from "./button-bar.ts";

export interface DialogOptions {
  readonly tone?: Tone;
  readonly icon?: string;
  readonly title: string;
  readonly body: string;
  /** What it is about, a line each under the body: the files it deletes. */
  readonly lists?: readonly string[];
  /** What it asks for, between the body and the buttons: a labelled field. */
  readonly asks?: HTMLElement;
  /** The button that backs out, at the start: Cancel. */
  readonly dismiss?: string;
  /** The buttons at the end, the one Enter presses last; none, with no dismiss, for a dialog with
   *  nothing to answer (a progress). */
  readonly actions: readonly Action[];
  /** Drawn with the keyboard on the control it lands on, as a picture of it open. */
  readonly landed?: boolean;
}

/** A dialog, and what closing it by each of its buttons or Esc does. */
export interface Dialog {
  readonly element: HTMLElement;
  /** Puts the keyboard where it lands as the dialog opens. */
  readonly land: () => void;
  /** Called with the button's label, or "Esc", when the dialog is answered. */
  onAnswer: (answer: string) => void;
}

/** The keys that scroll the middle, and which way. */
const SCROLLS: Readonly<Record<string, number>> = { ArrowUp: -1, ArrowDown: 1, PageUp: -1, PageDown: 1 };

/** The controls Tab stops on inside `element`, in reading order. */
function stops(element: HTMLElement): HTMLElement[] {
  return [...element.querySelectorAll<HTMLElement>("[tabindex], button")].filter(
    (each) => each.tabIndex >= 0 && !(each as HTMLButtonElement).disabled && each.getAttribute("aria-disabled") !== "true",
  );
}

export function dialog(options: DialogOptions): Dialog {
  const danger = options.tone === "Danger";
  const surface = raise(document.createElement("span"), 3);
  surface.className += " k-dialog";
  surface.setAttribute("role", danger ? "alertdialog" : "dialog");
  surface.setAttribute("aria-modal", "true");
  surface.toggleAttribute("data-danger", danger);
  const tint = danger ? "error" : "primary";
  const symbol = iconInLine(danger ? "warning" : (options.icon ?? "info"), "heading", `var(--color-${tint})`);
  const body = wrapped(options.body, "body", "on_surface_variant");
  const middle: Element[] = [body];
  if (options.lists) {
    const listed = stack(0, "start", ...options.lists.map((each) => text(each, "body")));
    listed.classList.add("k-dialog-follows");
    middle.push(listed);
  }
  if (options.asks) {
    options.asks.classList.add("k-dialog-follows");
    middle.push(options.asks);
  }
  const scroller = stack(0, "start", ...middle);
  scroller.classList.add("k-dialog-scroll");
  const above = divider();
  above.classList.add("k-dialog-rule");
  above.dataset.at = "above";
  const below = divider();
  below.classList.add("k-dialog-rule");
  below.dataset.at = "below";
  const room = document.createElement("span");
  room.className = "k-dialog-middle";
  room.append(scroller, above, below);
  const head = row("s12", "start", symbol, filling(wrapped(options.title, "heading")));
  const answered = options.dismiss !== undefined || options.actions.length > 0;
  const bar = answered
    ? buttonBar({ ...(options.dismiss ? { dismiss: options.dismiss } : {}), actions: options.actions, tone: options.tone ?? "Normal" })
    : undefined;
  const dismiss = bar?.dismiss ?? null;
  const actions = bar?.actions ?? [];
  const column = stack(0, "start", head, room);
  column.classList.add("k-dialog-column");
  if (bar) {
    // What the dialog's least height leaves over goes here, above its buttons.
    const spare = probe(document.createElement("span"), { role: "spacer", fill: true });
    spare.className = "k-dialog-spare";
    bar.element.classList.add("k-dialog-buttons");
    column.append(spare, bar.element);
  } else column.dataset.bare = "";
  surface.append(column);
  showEdges(scroller, room);

  const result: Dialog = { element: surface, land, onAnswer: () => undefined };

  /** Where the keyboard lands: what it asks for, else Cancel on a Danger dialog, else the last action. */
  function landing(): HTMLElement | undefined {
    const asked = options.asks ? stops(options.asks)[0] : undefined;
    const last = [...actions].reverse().find((each) => !each.disabled);
    return asked ?? (danger ? (dismiss ?? last) : (last ?? dismiss ?? undefined));
  }

  function land(): void {
    landing()?.focus({ focusVisible: true } as FocusOptions);
  }

  if (options.landed) {
    const at = landing();
    if (at) at.dataset.pinned = "lit";
    if (at?.classList.contains("k-field")) at.dataset.keyboard = "";
  }

  dismiss?.addEventListener("click", () => result.onAnswer(options.dismiss ?? ""));
  actions.forEach((each, index) => each.addEventListener("click", () => result.onAnswer(options.actions[index]!.label)));

  surface.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      result.onAnswer("Esc");
    } else if (event.key === "Tab") {
      const round = stops(surface);
      if (round.length === 0) return;
      const at = round.indexOf(document.activeElement as HTMLElement);
      const next = at < 0 ? 0 : (at + (event.shiftKey ? -1 : 1) + round.length) % round.length;
      round[next]!.focus({ focusVisible: true } as FocusOptions);
    } else if (event.key === "Enter" && options.asks?.contains(event.target as Node)) {
      // Enter in the field does what its action does.
      if (!event.repeat) [...actions].reverse().find((each) => !each.disabled)?.click();
    } else if (event.key in SCROLLS && !options.asks?.contains(event.target as Node) && scroller.scrollHeight > scroller.clientHeight) {
      // From its buttons, the arrows and the page keys scroll a middle too tall to show whole.
      const line = length("type-body-line");
      const by = SCROLLS[event.key]! * (event.key.startsWith("Page") ? scroller.clientHeight - line : line);
      const reduced = surface.closest("[data-reduced]") !== null;
      scroller.scrollBy({ top: by, behavior: reduced ? "instant" : "smooth" });
    } else return;
    event.preventDefault();
    event.stopPropagation();
  });

  probe(surface, { role: "component", name: "Dialog", variant: options.tone ?? "Normal", fixed: { width: true, height: false } });
  return result;
}

/**
 * Marks `room` while `scroller` has more above (`data-above`) or below (`data-below`) than it
 * shows, as it scrolls and whenever it or what it holds changes size.
 */
function showEdges(scroller: HTMLElement, room: HTMLElement): void {
  const mark = () => {
    // A zoomed page scrolls by fractions of a pixel, and its end can be one short of the height.
    room.toggleAttribute("data-above", scroller.scrollTop >= 1);
    room.toggleAttribute("data-below", scroller.scrollTop + scroller.clientHeight < scroller.scrollHeight - 1);
  };
  scroller.addEventListener("scroll", mark, { passive: true });
  const sizes = new ResizeObserver(mark);
  sizes.observe(scroller);
  for (const each of scroller.children) sizes.observe(each);
}

/** Everything behind a dialog, dimmed: it fills its room and takes every press, which does nothing. */
export function scrim(...content: Element[]): HTMLElement {
  const dim = document.createElement("span");
  dim.className = "k-scrim";
  dim.append(...content);
  return probe(dim, { role: "component", name: "Scrim", variant: "", fixed: { width: false, height: false } });
}

/** `shown` centred in `room` on whole device pixels, the odd one after it. */
export function centreIn(room: HTMLElement, shown: HTMLElement): void {
  shown.style.left = `${Math.floor((room.clientWidth - shown.offsetWidth) / 2)}px`;
  shown.style.top = `${Math.floor((room.clientHeight - shown.offsetHeight) / 2)}px`;
}
