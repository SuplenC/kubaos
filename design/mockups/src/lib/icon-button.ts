// A circle with a symbol, as tall as the label button of its size: `kubaos-ui`'s
// `button::icon_button`, `KubaOS.Components`' `IconButton`. Its label is required: an icon alone
// says nothing out loud, so the label is its accessible name and shows in its hint while the
// pointer is over it or the keyboard is on it (`docs/design/components/icon-button.md`). It takes a
// button's kinds, colours and states (`lib/button.ts`), and the icon of the label button of its
// size.

import { lightKind, METRICS, paintKind, type Kind, type Size } from "./button.ts";
import { pin, type Pinned } from "./control.ts";
import { withHint } from "./hint.ts";
import { icon } from "./icon.ts";
import { probe, variant } from "./probe.ts";

/** The kinds an icon button comes in, each a button's kind with its colours and states. */
export const ICON_KINDS = ["Filled", "Tonal", "Text", "Danger"] as const satisfies readonly Kind[];
export type IconKind = (typeof ICON_KINDS)[number];

export interface IconButtonOptions {
  readonly icon: string;
  readonly label: string;
  readonly kind?: IconKind;
  readonly size?: Size;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
  /** Whether its hint is drawn: a state matrix leaves it out, since it floats over the row below. */
  readonly hint?: boolean;
}

export function iconButton(options: IconButtonOptions): HTMLButtonElement {
  const kind = options.kind ?? "Filled";
  const size = options.size ?? "Md";
  const style = METRICS[size].label;
  const element = document.createElement("button");
  element.className = "k-button k-control";
  element.type = "button";
  element.dataset.iconOnly = "";
  element.disabled = options.disabled ?? false;
  element.setAttribute("aria-label", options.label);
  pin(element, options.pinned);
  paintKind(element, kind);
  element.style.setProperty("--height", `var(--size-${METRICS[size].height})`);
  const slot = document.createElement("span");
  slot.className = "k-icon-slot";
  slot.append(icon(options.icon, `var(--type-${style}-icon)`, "var(--shown)"));
  element.append(slot);
  lightKind(element, kind, "full", slot);
  if (options.hint ?? true) withHint(element, options.label);
  return probe(element, { role: "component", name: "IconButton", variant: variant(kind, size), fixed: { width: true, height: true } });
}
