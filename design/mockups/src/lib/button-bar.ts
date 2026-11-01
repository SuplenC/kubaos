// The row of buttons that ends a surface: `kubaos-ui`'s `button_bar::button_bar`,
// `KubaOS.Components`' `ButtonBar` (`docs/design/components/button-bar.md`).
//
// The dismiss button (Cancel, a Text button) at the start and the actions at the end, the one Enter
// presses last: Filled, or Danger on a Danger surface, and any other action before it Tonal.
// Buttons are `space.s8` apart, regular unless the surface asks for small. It has no padding of its
// own: it sits in its surface's padding like any other content, as wide as the room it is given.
//
// Where its buttons don't fit on one line, they stack `space.s8` apart, each the bar's full width,
// the one Enter presses on top and the dismiss button at the bottom.

import { button, type Kind, type Size } from "./button.ts";
import { row } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { length } from "./tokens.ts";

export type Tone = "Normal" | "Danger";

export interface Action {
  readonly label: string;
  /** Its kind where the rule's isn't it: the last action is Filled (Danger on a Danger surface), the rest Tonal. */
  readonly kind?: Kind;
  readonly icon?: string;
  readonly disabled?: boolean;
}

export interface ButtonBarOptions {
  /** The button that backs out, at the start: Cancel. */
  readonly dismiss?: string;
  /** The buttons at the end, the one Enter presses last. */
  readonly actions: readonly Action[];
  readonly tone?: Tone;
  /** `Sm` where its surface's controls are small; regular otherwise. */
  readonly size?: Extract<Size, "Sm" | "Md">;
}

export interface ButtonBar {
  readonly element: HTMLElement;
  readonly dismiss: HTMLButtonElement | null;
  /** The actions' buttons, in order: the one Enter presses last. */
  readonly actions: readonly HTMLButtonElement[];
}

/** The kind the rule gives the action at `index` of `count`. */
function kindAt(index: number, count: number, tone: Tone): Kind {
  if (index < count - 1) return "Tonal";
  return tone === "Danger" ? "Danger" : "Filled";
}

export function buttonBar(options: ButtonBarOptions): ButtonBar {
  const size = options.size ?? "Md";
  const tone = options.tone ?? "Normal";
  const dismiss = options.dismiss ? button({ label: options.dismiss, kind: "Text", size }) : null;
  const actions = options.actions.map((each, index) =>
    button({
      label: each.label,
      kind: each.kind ?? kindAt(index, options.actions.length, tone),
      size,
      ...(each.icon ? { icon: each.icon } : {}),
      ...(each.disabled ? { disabled: true } : {}),
    }),
  );
  const spacer = probe(document.createElement("span"), { role: "spacer", fill: true });
  spacer.className = "k-fill";
  const line = row("s8", "centre", ...(dismiss ? [dismiss] : []), spacer, ...actions);
  const bar = document.createElement("span");
  bar.className = "k-button-bar";
  bar.append(line);
  stackWhenCramped(bar, line);
  return { element: bar, dismiss, actions };
}

/**
 * Stacks `bar` while `line`, its buttons on one line, would run past its width, and lays them on
 * one line again once they fit. It measures the line each time the bar's width changes, before the
 * frame is drawn, so a cut line is never seen.
 */
function stackWhenCramped(bar: HTMLElement, line: HTMLElement): void {
  const gap = length("space-s8");
  const mark = (stacked: boolean) => {
    bar.toggleAttribute("data-stacked", stacked);
    probe(bar, { role: "component", name: "ButtonBar", variant: variant(stacked && "stacked"), fixed: { width: false, height: false } });
    probe(line, { role: "layout", axis: stacked ? "y" : "x", gap, align: "centre" });
  };
  // The buttons' own widths, not the line's scroll width: their glow reaches past their edges.
  const measure = () => {
    mark(false);
    const parts = [...line.children] as HTMLElement[];
    const buttons = parts.filter((each) => !each.classList.contains("k-fill"));
    const gaps = (parts.length - 1) * Number.parseFloat(getComputedStyle(line).columnGap);
    mark(buttons.reduce((sum, each) => sum + each.offsetWidth, 0) + gaps > line.clientWidth);
  };
  mark(false);
  let width = -1;
  new ResizeObserver(() => {
    if (bar.clientWidth === width) return;
    width = bar.clientWidth;
    measure();
  }).observe(bar);
  // The labels' widths change once their font has loaded, while the bar's width doesn't.
  void document.fonts.ready.then(measure);
}
