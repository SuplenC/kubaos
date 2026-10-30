// A short fact or pick: `kubaos-ui`'s `chip::chip`, `KubaOS.Components`' `Chip`. `size.chip` tall
// at the control radius, which on a chip is a pill, its label `label` type; an optional icon
// before it, `space.s4` from it; an optional close button ending it, a circle as tall as the chip
// (`docs/design/components/chip.md`). Its left side pads `space.s12`, or `space.s8` with an icon;
// its right `space.s12`, or nothing when the close button ends it.
//
// A plain chip is a fact and has no states. A pressable one is a control in the state model
// (`.k-control` in `lib.css`): it lights in its kind's tone, and a Filled one rests on the unlit
// fill with the core inside its edge. Its close button is a control of its own, a circle that
// lights as a Text icon button does. A chip that toggles (a setting a toolbar turns on and off)
// fades between Neutral and its kind on `--checked`.

import { pin, type Pinned } from "./control.ts";
import { focusRing, type Tone } from "./focus.ts";
import { withHint } from "./hint.ts";
import { icon } from "./icon.ts";
import { row } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { text } from "./text.ts";

export const CHIP_KINDS = ["Neutral", "Selected", "Accent", "Cold", "Filled"] as const;
export type ChipKind = (typeof CHIP_KINDS)[number];

/** A colour role as CSS. */
const role = (name: string) => `var(--color-${name})`;

/** A colour role as a tint, at `opacity.tint`. */
const tint = (name: string) => `rgb(from var(--color-${name}) r g b / var(--opacity-tint))`;

/**
 * A kind's colours as CSS: its fill at rest and pressed, and its label at rest and lit with the
 * signs on; lit with the signs off, a chip is its rest colours. A Filled chip names its line's
 * roles, the core and its glass inside the fill's edge; the others draw the tube in their tone.
 */
interface Colours {
  readonly fill: string;
  readonly fillPressed: string;
  readonly label: string;
  readonly labelLit: string;
  readonly edge?: string;
  readonly edgeOff?: string;
}

export const CHIP_COLOURS: Record<ChipKind, Colours> = {
  Neutral: { fill: role("surface_container_high"), fillPressed: role("surface_container_high"), label: role("on_surface"), labelLit: role("on_surface") },
  Selected: {
    fill: role("secondary_container"),
    fillPressed: role("secondary_container_pressed"),
    label: role("on_secondary_container_rest"),
    labelLit: role("on_secondary_container"),
  },
  Accent: { fill: tint("primary"), fillPressed: tint("primary"), label: role("primary"), labelLit: role("primary") },
  Cold: { fill: tint("cold"), fillPressed: tint("cold"), label: role("cold"), labelLit: role("cold") },
  Filled: {
    fill: role("primary_fill"),
    fillPressed: role("primary_fill_pressed"),
    label: role("on_primary_fill"),
    labelLit: role("on_primary_fill_lit"),
    edge: role("primary_fill_line"),
    edgeOff: role("primary_fill_glass"),
  },
};

/** The tone each kind lights in: the cold accent on a Cold chip, the accent on every other. */
export const CHIP_TONES: Record<ChipKind, Tone> = { Neutral: "accent", Selected: "accent", Accent: "accent", Cold: "cold", Filled: "accent" };

/** Whether a kind is filled with its tone, so its line is the core inside the fill's edge. */
export function chipFilled(kind: ChipKind): boolean {
  return kind === "Filled";
}

/**
 * What a chip does: nothing (a fact), take a press, or turn on and off with a press, fading
 * between Neutral and its kind.
 */
export type Press = "none" | "press" | "toggle";

export interface ChipOptions {
  readonly label: string;
  readonly kind?: ChipKind;
  readonly icon?: string;
  readonly removable?: boolean;
  readonly press?: Press;
  /** For a toggling chip: on, its kind; off, Neutral. */
  readonly on?: boolean;
  /** Whether Tab stops on a pressable chip; a click still presses it. */
  readonly tabStop?: boolean;
  /** A pressable chip's name, in a hint under it while it is lit. */
  readonly tooltip?: string;
  readonly disabled?: boolean;
  /** A state the chip is pinned in, or its close button: `close` names which. */
  readonly pinned?: Pinned;
  readonly pinnedClose?: Pinned;
}

/** `kind`'s colour for `slot`, from Neutral's while `--checked` is 0 to the kind's at 1. */
function between(kind: ChipKind, slot: Exclude<keyof Colours, "edge" | "edgeOff">): string {
  return `color-mix(in srgb, ${CHIP_COLOURS.Neutral[slot]}, ${CHIP_COLOURS[kind][slot]} calc(var(--checked) * 100%))`;
}

/** The close button: a circle as tall as the chip, its symbol `space.s4` in. */
function close(kind: ChipKind, disabled: boolean, pinned: Pinned | undefined): HTMLButtonElement {
  const button = document.createElement("button");
  button.className = "k-chip-close k-control";
  button.type = "button";
  button.dataset.hot = "";
  button.disabled = disabled;
  button.setAttribute("aria-label", "Remove");
  pin(button, pinned);
  button.append(icon("close", "var(--type-label-icon)", "var(--shown)"));
  const ring = focusRing(button, "full", CHIP_TONES[kind], chipFilled(kind));
  ring.dataset.pressable = "";
  return button;
}

export function chip(options: ChipOptions): HTMLElement {
  const kind = options.kind ?? "Neutral";
  const press = options.press ?? "none";
  const disabled = options.disabled ?? false;
  const element = document.createElement("span");
  element.className = "k-chip k-kind";
  element.dataset.kind = kind;
  element.style.setProperty("--checked", press === "toggle" && !options.on ? "0" : "1");
  element.style.setProperty("--fill", between(kind, "fill"));
  element.style.setProperty("--fill-lit", "var(--fill)");
  element.style.setProperty("--fill-off", "var(--fill)");
  element.style.setProperty("--fill-pressed", between(kind, "fillPressed"));
  element.style.setProperty("--label", between(kind, "label"));
  element.style.setProperty("--label-lit", between(kind, "labelLit"));
  element.style.setProperty("--label-off", "var(--label)");
  element.style.setProperty("--tone", role(CHIP_TONES[kind] === "cold" ? "cold" : "primary"));
  const colours = CHIP_COLOURS[kind];
  if (colours.edge && colours.edgeOff) {
    element.style.setProperty("--edge", colours.edge);
    element.style.setProperty("--edge-off", colours.edgeOff);
  }
  element.toggleAttribute("data-icon", options.icon !== undefined);
  element.toggleAttribute("data-removable", options.removable ?? false);
  const words = text(options.label, "label");
  words.style.color = "var(--shown)";
  const glyph = options.icon ? icon(options.icon, "var(--type-label-icon)", "var(--shown)") : null;
  const parts: Element[] = [...(glyph ? [glyph] : []), words, ...(options.removable ? [close(kind, disabled, options.pinnedClose)] : [])];
  element.append(row("s4", "centre", ...parts));
  if (disabled) element.setAttribute("aria-disabled", "true");
  if (press !== "none") {
    element.classList.add("k-control");
    element.setAttribute("role", "button");
    if (!disabled) element.tabIndex = (options.tabStop ?? true) ? 0 : -1;
    element.toggleAttribute("data-hot", !chipFilled(kind) && kind !== "Selected");
    pin(element, options.pinned);
    const ring = focusRing(element, "control", CHIP_TONES[kind], chipFilled(kind));
    ring.dataset.pressable = "";
    if (press === "toggle") {
      element.setAttribute("aria-pressed", String(options.on ?? false));
      element.addEventListener("click", (event) => {
        if (disabled || (event.target as Element).closest(".k-chip-close")) return;
        const on = element.getAttribute("aria-pressed") !== "true";
        element.setAttribute("aria-pressed", String(on));
        element.style.setProperty("--checked", on ? "1" : "0");
      });
    }
    if (options.tooltip) withHint(element, options.tooltip);
  }
  const named = variant(kind, options.icon ? "icon" : "", options.removable ? "removable" : "");
  return probe(element, { role: "component", name: "Chip", variant: named, fixed: { width: false, height: true } });
}
