// One of a list too long for radio rows or tabs: `kubaos-ui`'s `select::select`,
// `KubaOS.Components`' `Select` (`docs/design/components/select.md`). A field's look,
// `size.control_md` tall at the control radius with a hairline inside its edge, the picked option in
// `body_large`, an optional `body_large` symbol before it and a chevron at its end.
//
// Enter, Space or a press opens its menu on a popover `space.s4` under it, as wide as it, the
// picked option checked and the keyboard on it; open, its hairline takes the accent and its chevron
// turns up. ↑ ↓ and Enter pick, Esc or a press anywhere else closes it, and the keyboard comes back
// to the select. It is a control in the state model (`.k-control` in `lib.css`), lit as a field is.

import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { icon } from "./icon.ts";
import { filling, row } from "./layout.ts";
import { menu } from "./menu.ts";
import { probe, variant } from "./probe.ts";
import { clamped } from "./text.ts";
import { popover } from "./tooltip.ts";

/** The most options a select's menu shows before it scrolls. */
const MOST = 8;

export interface SelectOptions {
  readonly options: readonly string[];
  /** The picked option, or none: the placeholder shows. */
  readonly picked?: number;
  readonly placeholder?: string;
  readonly icon?: string;
  /** Drawn open, its menu under it in the sample's flow: a sample of the open state. */
  readonly open?: boolean;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

/** The control alone, showing `words`: open or not. */
function control(options: SelectOptions, words: string, open: boolean): HTMLButtonElement {
  const box = document.createElement("button");
  box.type = "button";
  box.className = "k-select k-control";
  box.disabled = options.disabled ?? false;
  box.setAttribute("aria-haspopup", "listbox");
  box.setAttribute("aria-expanded", String(open));
  box.toggleAttribute("data-open", open);
  box.toggleAttribute("data-placeholder", options.picked === undefined || options.picked < 0);
  pin(box, options.pinned);
  const mark = "var(--mark)";
  const value = clamped(words, "body_large");
  value.style.color = "var(--value)";
  const room = document.createElement("span");
  room.className = "k-select-value";
  room.append(value);
  const parts: Element[] = [];
  if (options.icon) parts.push(icon(options.icon, "var(--type-body_large-icon)", mark));
  parts.push(filling(room), icon(open ? "expand_less" : "expand_more", "var(--type-body_large-icon)", mark));
  box.append(filling(row("s8", "centre", ...parts)));
  focusRing(box, "control", "accent", false).dataset.pressable = "";
  return probe(box, { role: "component", name: "Select", variant: variant(options.icon && "icon"), fixed: { width: false, height: true } });
}

/** The menu of `options` on a popover as wide as the select, `picked` checked and current. */
function dropped(options: SelectOptions, picked: number, take: (index: number) => void, leave: () => void): HTMLElement {
  const list = menu(
    options.options.map((label, index) => ({ label, checked: index === picked })),
    { current: Math.max(0, picked), most: MOST, onTake: take, onLeave: leave },
  );
  const card = popover(2, list);
  card.style.setProperty("--popover-width", "100%");
  return card;
}

/**
 * A select in a room as wide as it is given: shut, or `open` with its menu under it in the
 * sample's flow. A live one opens its menu floating over what is below, taking no room.
 */
export function select(options: SelectOptions): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-select-room";
  let picked = options.picked ?? -1;
  const words = () => options.options[picked] ?? options.placeholder ?? "";

  if (options.open) {
    room.dataset.open = "";
    room.dataset.shown = "";
    room.append(control(options, words(), true), dropped(options, picked, () => undefined, () => undefined));
    return room;
  }

  let open = false;
  let box = control(options, words(), false);
  let card: HTMLElement | null = null;

  /** Closes the menu; `back` gives the keyboard back to the select. */
  function close(back: boolean): void {
    if (!open) return;
    open = false;
    delete room.dataset.open;
    box.setAttribute("aria-expanded", "false");
    box.removeAttribute("data-open");
    swapChevron(false);
    document.removeEventListener("pointerdown", outside, true);
    if (back) box.focus({ focusVisible: true } as FocusOptions);
  }

  function show(): void {
    open = true;
    card?.remove();
    card = dropped(
      options,
      picked,
      (index) => {
        picked = index;
        const next = control(options, words(), false);
        wire(next);
        box.replaceWith(next);
        box = next;
        close(true);
      },
      () => close(true),
    );
    room.append(card);
    // Laid out shut first, so it fades in rather than showing at once.
    void card.offsetWidth;
    room.dataset.open = "";
    box.setAttribute("aria-expanded", "true");
    box.setAttribute("data-open", "");
    swapChevron(true);
    document.addEventListener("pointerdown", outside, true);
    card.querySelector<HTMLElement>(".k-menu")?.focus();
  }

  function outside(event: PointerEvent): void {
    if (!room.contains(event.target as Node)) close(false);
  }

  function swapChevron(up: boolean): void {
    const chevron = box.querySelector(".k-row > .k-icon:last-child");
    chevron?.replaceWith(icon(up ? "expand_less" : "expand_more", "var(--type-body_large-icon)", "var(--mark)"));
  }

  function wire(target: HTMLButtonElement): void {
    target.addEventListener("click", () => (open ? close(true) : show()));
  }

  wire(box);
  room.append(box);
  return room;
}
