// A button with a label: `kubaos-ui`'s `button::button`, `KubaOS.Components`' `Button`. Four kinds
// by what it is for, three sizes; an optional icon leads the label, or trails it on a button that
// goes forward. Its measures are the specimens' (`docs/design/components/button.md`).

import { focusRing, type Tone, toneColour } from "./focus.ts";
import { icon } from "./icon.ts";
import { row } from "./layout.ts";
import { FIXED_HEIGHT, probe, variant } from "./probe.ts";
import { type TypeStyle, text } from "./text.ts";

export const KINDS = ["Filled", "Tonal", "Text", "Danger"] as const;
export type Kind = (typeof KINDS)[number];

export const SIZES = ["Sm", "Md", "Lg"] as const;
export type Size = (typeof SIZES)[number];

/**
 * A state shown whatever the pointer and the keyboard do, as the specimens pin them: `unlit` is
 * focused in its unlit look, the line as unlit glass with no glow, whatever the mode.
 */
export type Pinned = "hovered" | "pressed" | "focused" | "unlit";

/** A size's measures, every one a token. */
export interface Metrics {
  readonly height: string;
  readonly label: TypeStyle;
  readonly gap: string;
  /** Inside the sides of a filled, tonal or danger button. */
  readonly padding: string;
  /** Inside a text button, which has no edge to keep the label from. */
  readonly textPadding: string;
}

export const METRICS: Record<Size, Metrics> = {
  Sm: { height: "control_sm", label: "button_small", gap: "s4", padding: "s12", textPadding: "s12" },
  Md: { height: "control_md", label: "button", gap: "s8", padding: "s24", textPadding: "s12" },
  Lg: { height: "control_lg", label: "button_large", gap: "s8", padding: "s32", textPadding: "s16" },
};

/** Each kind's fill and content colours (`docs/design/components/button.md`). */
export const COLOURS: Record<Kind, { fill: string | null; content: string }> = {
  Filled: { fill: "primary", content: "on_primary" },
  Tonal: { fill: "secondary_container", content: "on_secondary_container" },
  Text: { fill: null, content: "primary" },
  Danger: { fill: "error", content: "on_error" },
};

/** The tone each kind's focus tube lights in: a Danger button's is red, the others the accent. */
export const TONES: Record<Kind, Tone> = {
  Filled: "accent",
  Tonal: "accent",
  Text: "accent",
  Danger: "error",
};

/**
 * Whether a kind is filled with its tone's colour, so its fill is the tube and the tone's core
 * rides outside it.
 */
export function filledWithTone(kind: Kind): boolean {
  return COLOURS[kind].fill === toneColour(TONES[kind]);
}

export interface ButtonOptions {
  readonly label: string;
  readonly kind?: Kind;
  readonly size?: Size;
  readonly icon?: string;
  readonly trailing?: boolean;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

/** A symbol in a box as wide as itself and as tall as the label's line beside it. */
function iconInLine(name: string, style: TypeStyle, color: string): HTMLElement {
  const slot = document.createElement("span");
  slot.className = "k-icon-line";
  slot.style.setProperty("--line", `var(--type-${style}-line)`);
  slot.style.setProperty("--icon-size", `var(--type-${style}-icon)`);
  slot.append(icon(name, `var(--type-${style}-icon)`, color));
  return slot;
}

export function button(options: ButtonOptions): HTMLButtonElement {
  const kind = options.kind ?? "Filled";
  const size = options.size ?? "Md";
  const m = METRICS[size];
  const element = document.createElement("button");
  element.className = "k-button";
  element.type = "button";
  element.dataset.kind = kind;
  element.disabled = options.disabled ?? false;
  if (options.pinned) element.dataset.pinned = options.pinned;
  const colours = COLOURS[kind];
  element.style.setProperty("--height", `var(--size-${m.height})`);
  element.style.setProperty("--line", `var(--type-${m.label}-line)`);
  element.style.setProperty("--padding", `var(--space-${kind === "Text" ? m.textPadding : m.padding})`);
  element.style.setProperty("--fill", colours.fill ? `var(--color-${colours.fill})` : "transparent");
  element.style.setProperty("--content", `var(--color-${colours.content})`);
  const shown = "var(--shown)";
  const label = text(options.label, m.label);
  label.style.color = shown;
  const glyph = options.icon ? iconInLine(options.icon, m.label, shown) : null;
  const children = glyph ? (options.trailing ? [label, glyph] : [glyph, label]) : [label];
  const layer = document.createElement("span");
  layer.className = "k-layer";
  element.append(layer, row(m.gap, "start", ...children));
  focusRing(element, "control", TONES[kind], filledWithTone(kind));
  const placement = !options.icon ? "" : options.trailing ? "trailing-icon" : "leading-icon";
  return probe(element, { role: "component", name: "Button", variant: variant(kind, size, placement), fixed: FIXED_HEIGHT });
}
