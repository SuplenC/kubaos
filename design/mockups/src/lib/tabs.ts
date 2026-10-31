// One of a few views or values, picked from a row of small buttons: `kubaos-ui`'s `tabs::tabs`,
// `KubaOS.Components`' `Tabs` (`docs/design/components/tabs.md`). The picked one is a `Tonal`
// button and the rest `Text`, all small (sm), `space.s4` apart, each with an optional symbol before
// its label. The same row is a setting's segmented control (Always, On AC, Never).
//
// The row is one stop for Tab, on the picked choice: ← and → (h and l) pick the neighbour, round
// from either end, and the keyboard follows the pick, so the picked one is the one lit. Disabled,
// every choice dims, the picked one still tonal, and the row takes no press and is no stop for Tab.

import { button } from "./button.ts";
import type { Pinned } from "./control.ts";
import { row } from "./layout.ts";
import { FIXED_HEIGHT, probe } from "./probe.ts";

export interface Tab {
  readonly label: string;
  readonly icon?: string;
}

export interface TabsOptions {
  readonly choices: readonly Tab[];
  readonly picked?: number;
  readonly disabled?: boolean;
  /** The picked choice pinned in a state, as the keyboard on the row would light it. */
  readonly pinned?: Pinned;
  /** What a pick does besides picking: the view it shows. */
  readonly onPick?: (index: number) => void;
}

export function tabs(options: TabsOptions): HTMLElement {
  const line = row("s4", "centre");
  line.setAttribute("role", "tablist");
  let picked = Math.max(0, Math.min(options.choices.length - 1, options.picked ?? 0));

  /** Draws every choice for the pick now; `focus` puts the keyboard on the picked one. */
  function draw(focus: boolean): void {
    const buttons = options.choices.map((choice, index) => {
      const current = index === picked;
      const each = button({
        label: choice.label,
        kind: current ? "Tonal" : "Text",
        size: "Sm",
        ...(choice.icon ? { icon: choice.icon } : {}),
        ...(options.disabled ? { disabled: true } : {}),
        ...(current && options.pinned ? { pinned: options.pinned } : {}),
      });
      each.setAttribute("role", "tab");
      each.setAttribute("aria-selected", String(current));
      each.tabIndex = current && !options.disabled ? 0 : -1;
      each.addEventListener("click", () => pick(index, false));
      each.addEventListener("keydown", (event) => {
        const by = event.key === "ArrowLeft" || event.key === "h" ? -1 : event.key === "ArrowRight" || event.key === "l" ? 1 : 0;
        if (by === 0) return;
        event.preventDefault();
        pick((picked + by + options.choices.length) % options.choices.length, true);
      });
      return each;
    });
    line.replaceChildren(...buttons);
    if (focus) buttons[picked]?.focus({ focusVisible: true } as FocusOptions);
  }

  function pick(index: number, focus: boolean): void {
    picked = index;
    draw(focus);
    options.onPick?.(index);
  }

  draw(false);
  const box = document.createElement("span");
  box.className = "k-tabs";
  box.append(line);
  return probe(box, { role: "component", name: "Tabs", variant: "", fixed: FIXED_HEIGHT });
}
