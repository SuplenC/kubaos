// One of a few views or values, picked along a rail of neon: `kubaos-ui`'s `tabs::tabs`,
// `KubaOS.Components`' `Tabs` (`docs/design/components/tabs.md`). The choices sit in one well, a
// pill as tall as a small control, with an unlit rail along its bottom edge; the picked choice's
// stretch of the rail is lit, and its words are the accent. The well is as wide as its room and
// its choices share it equally, never narrower than the widest choice each. The same well is a
// setting's segmented control (Always, On AC, Never). `.k-tabs` in `lib.css` draws it.
//
// A pick sends the lit stretch from the old choice to the new one: its leading edge goes over
// `short` and its trailing edge follows a `micro` later over `medium`, both on `emphasized_decel`,
// so it stretches as it travels and catches up, and its core flashes toward white as it lands. A
// pick made mid-flight starts from wherever the stretch is. With reduce motion the stretch is on
// the new choice at once and only the colours fade.
//
// The row is one stop for Tab, on the picked choice, and the focus tube lights the whole well's
// edge: ← and → (h and l) pick the neighbour, round from either end, and the keyboard follows the
// pick. Disabled, everything dims, the pick still shown, and the row takes no press and is no stop
// for Tab. A single choice disabled dims alone and takes no press, and the arrows pass over it. A
// pick that is disabled stays picked, dimmed with its stretch, until the owner moves it; the stop
// for Tab is then the well itself, and an arrow picks the nearest choice that isn't disabled.
//
// A row may have no pick (`picked: null`): no stretch is lit and every choice's words rest. It is
// for a row that doesn't apply, disabled, as Calls and Music are for a device that isn't Bluetooth;
// a pick on one that isn't disabled lays the stretch under it at once.
//
// A choice may carry a tip, what picking it means: the library's Tooltip, its words as the title,
// floating `space.s8` above the well while the pointer is on the choice or the keyboard is, lined up
// with the well's start, or its end for a choice in the row's second half, so the card stays over
// the row. It takes no room, so nothing round the row moves as it comes and goes.

import { iconInLine } from "./button.ts";
import { type Pinned, pin } from "./control.ts";
import { focusRing } from "./focus.ts";
import { row } from "./layout.ts";
import { FIXED_HEIGHT, probe } from "./probe.ts";
import { text } from "./text.ts";
import { tooltip } from "./tooltip.ts";
import { raw } from "./tokens.ts";
import { CHROME, settings } from "../site/settings.ts";

export interface Tab {
  readonly label: string;
  readonly icon?: string;
  /** Dimmed, no press, passed over by the arrows: Input while a headset is in Music mode. */
  readonly disabled?: boolean;
  /** What picking it means, in a Tooltip over the row while the pointer or the keyboard is on it. */
  readonly tip?: string;
}

export interface TabsOptions {
  readonly choices: readonly Tab[];
  /** The picked choice, the first by default; null for none, on a row that doesn't apply. */
  readonly picked?: number | null;
  readonly disabled?: boolean;
  /**
   * The row pinned in a state: lit or lit with the signs off, as the keyboard on the row lights
   * it; pressed, the choice after the picked one held under the pointer.
   */
  readonly pinned?: Pinned;
  /**
   * The choice a pick comes from, for a move's sample: the lit stretch and the words start there
   * and the move's player takes them to the picked one.
   */
  readonly from?: number;
  /** The choice whose tip is drawn shown, as under the pointer: a sample of it. */
  readonly tipShown?: number;
  /** What a pick does besides picking: the view it shows. */
  readonly onPick?: (index: number) => void;
}

/** A lit stretch's two edges along the well, in CSS pixels from its left edge. */
interface Edges {
  readonly left: number;
  readonly right: number;
}

/** Each row's own way of disabling a choice from outside, by its well. */
const disablers = new WeakMap<Element, (index: number, disabled: boolean) => void>();

/**
 * Disables or enables choice `index` of the row `well` from outside, as its owner's state changes:
 * Input while a headset is in Music mode. A pick that becomes disabled stays picked.
 */
export function setTabDisabled(well: Element, index: number, disabled: boolean): void {
  disablers.get(well)?.(index, disabled);
}

let made = 0;

/** The duration token `name` in milliseconds under the speed setting. */
function duration(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

export function tabs(options: TabsOptions): HTMLElement {
  const count = options.choices.length;
  const clamp = (index: number) => Math.max(0, Math.min(count - 1, index));
  let picked = options.picked === null ? -1 : clamp(options.picked ?? 0);
  const from = options.from === undefined ? null : clamp(options.from);

  const id = `k-tabs-${made++}`;
  const well = document.createElement("span");
  well.className = "k-tabs";
  well.setAttribute("role", "tablist");
  if (options.disabled) well.setAttribute("aria-disabled", "true");
  if (options.pinned !== "pressed") pin(well, options.pinned);
  focusRing(well, "full", "accent", false);

  const buttons = options.choices.map((choice, index) => {
    const each = document.createElement("button");
    each.className = "k-tab k-control";
    each.type = "button";
    each.setAttribute("role", "tab");
    each.disabled = (options.disabled ?? false) || (choice.disabled ?? false);
    const words = text(choice.label, "button_small");
    words.style.color = "var(--words)";
    const glyph = choice.icon ? iconInLine(choice.icon, "button_small", "var(--words)") : null;
    const content = row("s4", "start", ...(glyph ? [glyph, words] : [words]));
    content.classList.add("k-tab-words");
    each.append(content);
    if (choice.tip) {
      const tip = tooltip(choice.tip, "");
      tip.id = `${id}-tip-${index}`;
      tip.setAttribute("role", "tooltip");
      // Read out as the choice's description, never as part of its name.
      tip.setAttribute("aria-hidden", "true");
      tip.dataset.held = index >= count / 2 ? "end" : "start";
      tip.toggleAttribute("data-shown", options.tipShown === index);
      each.setAttribute("aria-describedby", tip.id);
      each.append(tip);
    }
    each.addEventListener("click", () => pick(index, false));
    return each;
  });
  const off = options.choices.map((choice) => choice.disabled ?? false);
  const open = (index: number) => !options.disabled && !off[index];
  well.addEventListener("keydown", (event) => {
    const by = event.key === "ArrowLeft" || event.key === "h" ? -1 : event.key === "ArrowRight" || event.key === "l" ? 1 : 0;
    if (by === 0) return;
    event.preventDefault();
    const next = beside(by);
    if (next !== null) pick(next, true);
  });

  /** The nearest choice from the pick toward `by`'s side that isn't disabled, round from either end. */
  function beside(by: -1 | 1): number | null {
    for (let step = 1; step < count; step++) {
      const index = (((picked + by * step) % count) + count) % count;
      if (open(index)) return index;
    }
    return null;
  }

  const pressed = [1, 2, 3].map((step) => (picked + step) % count).find((index) => index !== picked && open(index));
  if (options.pinned === "pressed" && pressed !== undefined) pin(buttons[pressed]!, "pressed");
  if (from !== null && from !== picked) buttons[from]!.dataset.was = "";

  const rail = document.createElement("span");
  rail.className = "k-tabs-rail";
  const lit = document.createElement("span");
  lit.className = "k-tabs-lit";
  well.append(...buttons, rail, lit);

  /**
   * Marks the pick on every choice: its words, its stop for Tab and what it reads out. A disabled
   * pick can't take the keyboard, so the well takes the row's stop in its place.
   */
  function mark(): void {
    for (const [index, each] of buttons.entries()) {
      const current = index === picked;
      each.setAttribute("aria-selected", String(current));
      each.tabIndex = current && open(index) ? 0 : -1;
    }
    well.toggleAttribute("data-none", picked < 0);
    const stranded = !options.disabled && picked >= 0 && !open(picked);
    well.toggleAttribute("data-stranded", stranded);
    if (stranded) well.tabIndex = 0;
    else well.removeAttribute("tabindex");
  }

  // Where a choice's stretch of the rail runs is its words' width, which only layout knows: the
  // choice's box less its label padding on each side, read back from the laid-out well. The page's
  // box is zoomed to the display's ratio, so a box read in the viewport's pixels is brought back to
  // the well's own by its width over its layout width.
  /** `part`'s edges in the well's own CSS pixels from its padding edge. */
  function within(part: Element): Edges {
    const outer = well.getBoundingClientRect();
    const box = part.getBoundingClientRect();
    const ratio = well.offsetWidth / Math.max(1, outer.width);
    const origin = outer.left;
    return { left: (box.left - origin) * ratio - well.clientLeft, right: (box.right - origin) * ratio - well.clientLeft };
  }

  /** The lit stretch's edges under choice `index`: its box less the label padding either side. */
  function under(index: number): Edges {
    const each = buttons[index]!;
    const box = within(each);
    const style = getComputedStyle(each);
    return { left: box.left + Number.parseFloat(style.paddingLeft), right: box.right - Number.parseFloat(style.paddingRight) };
  }

  /** Where the lit stretch is drawn now, mid-flight or at rest. */
  function where(): Edges {
    return within(lit);
  }

  /** Sets the stretch's travel, from `start` to `end`, its leading edge the one in the way of travel. */
  function travel(start: Edges, end: Edges): void {
    well.style.setProperty("--from-left", `${start.left}px`);
    well.style.setProperty("--from-right", `${start.right}px`);
    well.style.setProperty("--to-left", `${end.left}px`);
    well.style.setProperty("--to-right", `${end.right}px`);
    well.dataset.toward = end.left >= start.left ? "right" : "left";
  }

  let flying: Animation[] = [];

  /**
   * Plays the stretch's travel and its flash on the page's clock: the leading edge over `short`,
   * the trailing one a `micro` later over `medium`, and the core rising to its flash over `micro`
   * and settling over `long`. Reduce motion leaves the stretch where it lands and keeps the flash,
   * settling within its cap.
   */
  function fly(): void {
    for (const each of flying) each.cancel();
    flying = [];
    if (!CHROME || !well.isConnected) return;
    const reduced = settings().reduced;
    const decel = raw("easing-emphasized_decel");
    const standard = raw("easing-standard");
    if (!reduced) {
      flying.push(lit.animate([{ "--lead": "0" }, { "--lead": "1" }], { duration: duration("short"), easing: decel }));
      flying.push(
        lit.animate([{ "--trail": "0" }, { "--trail": "1" }], { duration: duration("medium"), delay: duration("micro"), easing: decel, fill: "backwards" }),
      );
    }
    const rise = duration("micro");
    const settle = duration(reduced ? "short" : "long");
    flying.push(
      lit.animate(
        [
          { "--flash": "0", easing: standard },
          { "--flash": "1", offset: rise / (rise + settle), easing: standard },
          { "--flash": "0" },
        ],
        { duration: rise + settle },
      ),
    );
  }

  function pick(index: number, focus: boolean): void {
    if (index === picked || !open(index)) return;
    // From no pick the stretch has nowhere to travel from: it starts where it lands.
    const start = picked < 0 ? under(index) : where();
    picked = index;
    mark();
    if (focus) buttons[index]?.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
    travel(start, under(picked));
    fly();
    options.onPick?.(index);
  }

  /** Lays the stretch on the pick, or for a move's sample its travel from `from`, whenever the well's size changes. */
  function lay(): void {
    if (!well.isConnected || picked < 0 || flying.some((each) => each.playState === "running")) return;
    const end = under(picked);
    travel(from !== null && from !== picked ? under(from) : end, end);
  }

  disablers.set(well, (index, disabled) => {
    const button = buttons[index];
    if (!button || off[index] === disabled) return;
    const had = document.activeElement === button || document.activeElement === well;
    off[index] = disabled;
    button.disabled = (options.disabled ?? false) || disabled;
    mark();
    // The row keeps the keyboard it had: on the well while its pick is disabled, back on the pick after.
    const holder = picked >= 0 && open(picked) ? buttons[picked] : well;
    if (had) holder?.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
  });

  mark();
  new ResizeObserver(lay).observe(well);
  return probe(well, { role: "component", name: "Tabs", variant: "", fixed: FIXED_HEIGHT });
}
