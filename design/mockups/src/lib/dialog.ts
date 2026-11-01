// A question over a dimmed screen: `kubaos-ui`'s `dialog::dialog`, `dialog::prompt` and
// `dialog::scrim`, `KubaOS.Components`' `Dialog` and `Scrim` (`docs/design/components/dialog.md`).
//
// The dialog is `size.dialog` wide, `space.s24` inside on every side, at the dialog radius, raised
// to the ladder's step 3 (`lib/elevation.ts`). A 24 symbol (`heading`'s) beside a `heading` title,
// the `body` `space.s12` under the title and in line with it, what it asks for (a labelled field)
// `space.s16` under the body, in line with it too, and its button bar (`lib/button-bar.ts`)
// `space.s24` under the last, across its whole width. A Danger dialog shows the warning sign in
// `error` with an edge of `error`, and its action is a Danger button.
//
// The scrim dims everything behind it and takes every press, which does nothing; the dialog sits
// in its middle on whole device pixels. Opening, the keyboard goes into the dialog, onto what it
// asks for, else Cancel on a Danger dialog, else the last action; Tab and Shift+Tab go round it
// and never leave; Esc is its dismiss button; closing gives the keyboard back to what opened it.

import { iconInLine } from "./button.ts";
import { type Action, buttonBar, type Tone } from "./button-bar.ts";
import { raise } from "./elevation.ts";
import { filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { wrapped } from "./text.ts";

export type { Action, Tone } from "./button-bar.ts";

export interface DialogOptions {
  readonly tone?: Tone;
  readonly icon?: string;
  readonly title: string;
  readonly body: string;
  /** What it asks for, between the body and the buttons: a labelled field. */
  readonly asks?: HTMLElement;
  /** The button that backs out, at the start: Cancel. */
  readonly dismiss?: string;
  /** The buttons at the end, the one Enter presses last. */
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
  body.classList.add("k-dialog-body");
  const words: Element[] = [wrapped(options.title, "heading"), body];
  if (options.asks) {
    options.asks.classList.add("k-dialog-asks");
    words.push(options.asks);
  }
  const said = stack(0, "start", ...words);
  said.classList.add("k-dialog-said");
  const head = row("s12", "start", symbol, filling(said));
  const bar = buttonBar({ ...(options.dismiss ? { dismiss: options.dismiss } : {}), actions: options.actions, tone: options.tone ?? "Normal" });
  const { dismiss, actions } = bar;
  bar.element.classList.add("k-dialog-buttons");
  const column = stack(0, "start", head, bar.element);
  column.classList.add("k-dialog-column");
  surface.append(column);

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
    } else return;
    event.preventDefault();
    event.stopPropagation();
  });

  probe(surface, { role: "component", name: "Dialog", variant: options.tone ?? "Normal", fixed: { width: true, height: false } });
  return result;
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
