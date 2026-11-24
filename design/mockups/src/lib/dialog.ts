// A question over a dimmed screen: `kubaos-ui`'s `dialog::dialog`, `dialog::prompt` and
// `dialog::scrim`, `KubaOS.Components`' `Dialog` and `Scrim` (`docs/design/components/dialog.md`).
//
// The dialog is `size.dialog` wide, `space.s24` inside on every side, at the content radius, on the
// shell's glass (`lib/glass.ts`), the ladder's step 3 (`lib/elevation.ts`). Everything in it starts on that one content edge: a
// 24 symbol (`heading`'s) beside a `heading` title on one row, the `body` `space.s12` under them,
// what it lists or asks for (a labelled field) `space.s16` under the body, each across the whole
// width, and its button bar (`lib/button-bar.ts`) `space.s24` under the last, Cancel's label on
// the edge and the last button's box as far past the right one as Cancel's is past the left, and
// the dialog's padding under that bar `space.s12`, so its boxes sit alike on three sides. A
// Danger dialog shows the warning sign in `error` with an edge of `error`, and its action is a
// Danger button.
//
// Its kind is what it tells, by its symbol in that kind's fixed role: Info (a question, or news)
// `info` in `primary`, Warning the warning sign in `warning`, Error `error` in `error`. An Error
// dialog is not a Danger one: nothing is destroyed by answering it, so its rim and its buttons are
// a Normal dialog's. Where another program asks (zenity), a caller line over the title names it:
// the app's small IconTile and its name in `label`, `space.s12` above the title row, left out when
// the title says the name already (the name, in any case, with nothing after it but punctuation).
//
// The body is the program's words: plain, or in Pango's markup (`<b>`, `<i>`, `<a href>` and its
// entities, `lib/text.ts`'s `markup`), or kept to the caller's own lines in `mono` (`noWrap`), a
// line longer than the dialog still breaking. A dialog that answers itself after a while shows the
// time left as a timed bar with its caption, `space.s16` under the body, emptying to its end.
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
// asks for, else Cancel on a Danger dialog or one that makes Cancel the default, else the last
// action; Tab and Shift+Tab go round it
// and never leave, scrolling what they land on into view; the arrows and the page keys scroll a
// middle too tall to show whole; Esc is its dismiss button; closing gives the keyboard back to
// what opened it.

import { iconInLine } from "./button.ts";
import { type Action, buttonBar, type Tone } from "./button-bar.ts";
import { glass } from "./glass.ts";
import { iconTile } from "./icon-tile.ts";
import { divider, filling, gap, row, stack } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { captionedBar, progressBar } from "./progress.ts";
import { markup, text, wrapped } from "./text.ts";
import { settings } from "../site/settings.ts";
import { durationMs, length } from "./tokens.ts";

export type { Action, Tone } from "./button-bar.ts";

/** What a dialog tells, by its symbol: news or a question, a warning, or an error. */
export type DialogKind = "Info" | "Warning" | "Error";

/** Each kind's symbol and the fixed role it is drawn in. */
const KIND_SYMBOLS: Record<DialogKind, { readonly icon: string; readonly role: string }> = {
  Info: { icon: "info", role: "primary" },
  Warning: { icon: "warning", role: "warning" },
  Error: { icon: "error", role: "error" },
};

/** The app another program's dialog asks for: its name, and its icon where it has one. */
export interface Caller {
  readonly name: string;
  readonly picture?: string;
}

export interface DialogOptions {
  readonly tone?: Tone;
  /** Info unless named: Warning and Error change the symbol and its role, not the tone. */
  readonly kind?: DialogKind;
  /** A symbol of its own in place of Info's, in `primary`. */
  readonly icon?: string;
  readonly title: string;
  readonly body: string;
  /** The body is Pango's markup: `<b>`, `<i>`, `<a href>` and its entities. */
  readonly markup?: boolean;
  /** The body keeps its own lines, in `mono`. */
  readonly noWrap?: boolean;
  /** The app that asks, named over the title unless the title names it. */
  readonly caller?: Caller;
  /** The keyboard lands on the dismiss button, so Enter backs out. */
  readonly defaultCancel?: boolean;
  /** It answers itself: the seconds left of how many, a timed bar and its caption. */
  readonly timeout?: { readonly left: number; readonly of: number };
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
  const surface = glass(document.createElement("span"));
  surface.classList.add("k-dialog");
  surface.dataset.step = "3";
  surface.setAttribute("role", danger ? "alertdialog" : "dialog");
  surface.setAttribute("aria-modal", "true");
  surface.toggleAttribute("data-danger", danger);
  const kind = KIND_SYMBOLS[options.kind ?? "Info"];
  const symbol = danger
    ? iconInLine("warning", "heading", "var(--color-error)")
    : options.icon && (options.kind ?? "Info") === "Info"
      ? iconInLine(options.icon, "heading", "var(--color-primary)")
      : iconInLine(kind.icon, "heading", `var(--color-${kind.role})`);
  const body = options.markup ? markup(options.body, "body", "on_surface_variant") : wrapped(options.body, "body", "on_surface_variant");
  body.toggleAttribute("data-no-wrap", options.noWrap ?? false);
  const middle: Element[] = [body];
  if (options.lists) {
    const listed = stack(0, "start", ...options.lists.map((each) => text(each, "body")));
    middle.push(gap("s16"), listed);
  }
  if (options.asks) {
    middle.push(gap("s16"), options.asks);
  }
  if (options.timeout) {
    const { left, of } = options.timeout;
    const timed = captionedBar(progressBar({ fraction: left / of, timed: true }), { words: `Closes in ${left} s`, icon: "timer" });
    timed.classList.add("k-dialog-timeout");
    middle.push(gap("s16"), timed);
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
  const caller = options.caller && !namesCaller(options.title, options.caller.name) ? callerLine(options.caller) : undefined;
  const column = stack(0, "start", ...(caller ? [caller, gap("s12")] : []), head, gap("s12"), room);
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

  /** Where the keyboard lands: what it asks for, else Cancel on a Danger dialog or one that makes
   *  Cancel the default, else the last action. */
  function landing(): HTMLElement | undefined {
    const asked = options.asks ? stops(options.asks)[0] : undefined;
    const last = [...actions].reverse().find((each) => !each.disabled);
    return asked ?? (danger || options.defaultCancel ? (dismiss ?? last) : (last ?? dismiss ?? undefined));
  }

  function land(): void {
    landing()?.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
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
      round[next]!.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
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

  // A caller line moves every part under it, so a dialog with one is told apart from one without.
  probe(surface, { role: "component", name: "Dialog", variant: variant(options.tone ?? "Normal", caller && "caller"), fixed: { width: true, height: false } });
  return result;
}

/** The caller line: the app's small tile and its name, in `label`. */
function callerLine(caller: Caller): HTMLElement {
  const tile = iconTile({ name: caller.name, size: "Sm", ...(caller.picture ? { picture: caller.picture } : {}) });
  const line = row("s8", "centre", tile, text(caller.name, "label", "on_surface_variant"));
  line.classList.add("k-dialog-caller");
  return line;
}

/**
 * Whether `title` says no more than `name`: the same name, trimmed and in any case, or the name with
 * nothing after it but punctuation. Then the title carries the name and the caller line goes.
 */
export function namesCaller(title: string, name: string): boolean {
  const app = name.trim().toLowerCase();
  const said = title.trim().toLowerCase();
  return said.startsWith(app) && /^[\s\p{P}]*$/u.test(said.slice(app.length));
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

/** A dialog laid over a window, and how to answer it from outside, as a timer that runs out does. */
export interface DialogOver {
  readonly room: HTMLElement;
  readonly dialog: Dialog;
  /** Answers it as its button labelled `answer` would, or "Esc". */
  readonly answer: (answer: string) => void;
}

/**
 * `options`' dialog over the window `frame`, as an app asks a question of its own: the scrim across
 * the whole window to its rounded edge and the dialog centred in it, on whole device pixels as the
 * window resizes, and everything else the window holds out of the keyboard's reach until it is
 * answered. Answered, both fade out, the keyboard goes back where it was, and `answered` hears the
 * button's label, or "Esc". Drawn `still`, it is already there with the keyboard where it lands: a
 * sample's picture.
 */
export function dialogOver(frame: HTMLElement, options: DialogOptions, answered: (answer: string) => void, still = false): DialogOver {
  const room = document.createElement("span");
  room.className = "k-dialog-room";
  room.dataset.live = "";
  room.dataset.over = "";
  const dim = scrim();
  const asked = dialog(still ? { ...options, landed: true } : options);
  room.append(dim, asked.element);
  if (still) for (const each of [dim, asked.element]) each.dataset.atRest = "";
  const behind = [...frame.children].filter((each): each is HTMLElement => each instanceof HTMLElement);
  for (const each of behind) each.inert = true;
  frame.append(room);
  // The room has no size until it is on the page, and its size follows the window's.
  const centring = new ResizeObserver(() => centreIn(room, asked.element));
  centring.observe(room);
  const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  let done = false;
  const answer = (said: string) => {
    if (done) return;
    done = true;
    centring.disconnect();
    for (const each of behind) each.inert = false;
    for (const each of [dim, asked.element]) each.dataset.leaving = "";
    // Gone once it has faded, or at once where nothing fades.
    const gone = () => room.remove();
    asked.element.addEventListener("transitionend", (event) => {
      if (event.propertyName === "opacity") gone();
    });
    setTimeout(gone, durationMs("medium") / settings().speed);
    if (opener?.isConnected) opener.focus({ preventScroll: true });
    answered(said);
  };
  asked.onAnswer = answer;
  if (!still) requestAnimationFrame(() => asked.land());
  return { room, dialog: asked, answer };
}
